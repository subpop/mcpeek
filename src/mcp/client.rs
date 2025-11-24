use super::protocol::*;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, warn};

pub struct McpClient {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    request_id: AtomicI64,
    #[allow(dead_code)]
    response_tx: mpsc::UnboundedSender<ResponseMessage>,
    #[allow(dead_code)]
    response_rx: Arc<Mutex<mpsc::UnboundedReceiver<ResponseMessage>>>,
    pending_requests: Arc<Mutex<HashMap<i64, oneshot::Sender<JsonRpcResponse>>>>,
    server_info: Arc<Mutex<Option<InitializeResult>>>,
    log_rx: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
}

enum ResponseMessage {
    #[allow(dead_code)]
    Response(JsonRpcResponse),
    #[allow(dead_code)]
    Notification(JsonRpcRequest),
}

impl McpClient {
    pub async fn new(command: &str, args: &[String]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let stdin = child
            .stdin
            .take()
            .context("Failed to get stdin of child process")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to get stdout of child process")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to get stderr of child process")?;

        let (response_tx, response_rx) = mpsc::unbounded_channel();
        let (log_tx, log_rx) = mpsc::unbounded_channel();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        let client = Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            request_id: AtomicI64::new(1),
            response_tx: response_tx.clone(),
            response_rx: Arc::new(Mutex::new(response_rx)),
            pending_requests: pending_requests.clone(),
            server_info: Arc::new(Mutex::new(None)),
            log_rx: Arc::new(Mutex::new(log_rx)),
        };

        tokio::spawn(Self::read_loop(stdout, response_tx, pending_requests));
        tokio::spawn(Self::log_loop(stderr, log_tx));

        Ok(client)
    }

    async fn read_loop(
        stdout: ChildStdout,
        response_tx: mpsc::UnboundedSender<ResponseMessage>,
        pending_requests: Arc<Mutex<HashMap<i64, oneshot::Sender<JsonRpcResponse>>>>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Server stdout closed");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", trimmed);

                    if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                        if let Value::Number(id) = &response.id {
                            if let Some(id) = id.as_i64() {
                                let mut pending = pending_requests.lock().await;
                                if let Some(sender) = pending.remove(&id) {
                                    let _ = sender.send(response);
                                    continue;
                                }
                            }
                        }
                        let _ = response_tx.send(ResponseMessage::Response(response));
                    } else if let Ok(notification) = serde_json::from_str::<JsonRpcRequest>(trimmed)
                    {
                        let _ = response_tx.send(ResponseMessage::Notification(notification));
                    } else {
                        warn!("Failed to parse message: {}", trimmed);
                    }
                }
                Err(e) => {
                    error!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    }

    async fn log_loop(stderr: ChildStderr, log_tx: mpsc::UnboundedSender<String>) {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Server stderr closed");
                    break;
                }
                Ok(_) => {
                    if !line.trim().is_empty() {
                        let _ = log_tx.send(line.clone());
                    }
                }
                Err(e) => {
                    error!("Error reading stderr from server: {}", e);
                    break;
                }
            }
        }
    }

    async fn send_request(&self, request: JsonRpcRequest) -> Result<()> {
        let json = serde_json::to_string(&request)?;
        debug!("Sending: {}", json);

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        Ok(())
    }

    async fn call_method<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<P>,
    ) -> Result<R> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let params = params
            .map(|p| serde_json::to_value(p))
            .transpose()
            .context("Failed to serialize params")?;

        let request = JsonRpcRequest::new(id, method, params);

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        self.send_request(request).await?;

        let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .context("Request timed out")??;

        if let Some(error) = response.error {
            anyhow::bail!("RPC error: {} (code: {})", error.message, error.code);
        }

        let result = response.result.context("Response missing result field")?;

        debug!(
            "Deserializing result: {}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
        serde_json::from_value(result.clone()).with_context(|| {
            format!(
                "Failed to deserialize result: {}",
                serde_json::to_string_pretty(&result).unwrap_or_default()
            )
        })
    }

    pub async fn initialize(&self) -> Result<InitializeResult> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let result: InitializeResult = self.call_method("initialize", Some(params)).await?;

        *self.server_info.lock().await = Some(result.clone());

        let notification = JsonRpcRequest::notification("notifications/initialized", None);
        self.send_request(notification).await?;

        Ok(result)
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result: ListToolsResult = self.call_method("tools/list", None::<()>).await?;
        Ok(result.tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<CallToolResult> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        self.call_method("tools/call", Some(params)).await
    }

    pub async fn list_prompts(&self) -> Result<Vec<Prompt>> {
        let result: ListPromptsResult = self.call_method("prompts/list", None::<()>).await?;
        Ok(result.prompts)
    }

    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<GetPromptResult> {
        let params = GetPromptParams {
            name: name.to_string(),
            arguments,
        };

        self.call_method("prompts/get", Some(params)).await
    }

    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        let result: ListResourcesResult = self.call_method("resources/list", None::<()>).await?;
        Ok(result.resources)
    }

    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContents>> {
        let params = ReadResourceParams {
            uri: uri.to_string(),
        };

        let result: ReadResourceResult = self
            .call_method("resources/read", Some(params))
            .await
            .context("Failed to call resources/read")?;
        Ok(result.contents)
    }

    pub async fn get_server_info(&self) -> Option<InitializeResult> {
        self.server_info.lock().await.clone()
    }

    pub async fn get_logs(&self) -> Vec<String> {
        let mut logs = Vec::new();
        let mut rx = self.log_rx.lock().await;

        while let Ok(log) = rx.try_recv() {
            logs.push(log);
        }

        logs
    }

    pub async fn shutdown(&self) -> Result<()> {
        let _ = self.child.lock().await.kill().await;
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let child = self.child.clone();
        tokio::spawn(async move {
            let _ = child.lock().await.kill().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_id_increments() {
        let request_id = AtomicI64::new(1);

        let id1 = request_id.fetch_add(1, Ordering::SeqCst);
        let id2 = request_id.fetch_add(1, Ordering::SeqCst);
        let id3 = request_id.fetch_add(1, Ordering::SeqCst);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(1, "initialize", Some(json!({"test": "value"})));
        let json_str = serde_json::to_string(&request).unwrap();

        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"id\":1"));
        assert!(json_str.contains("\"method\":\"initialize\""));
        assert!(json_str.contains("\"params\""));
    }

    #[test]
    fn test_notification_serialization() {
        let notification = JsonRpcRequest::notification("notifications/initialized", None);
        let json_str = serde_json::to_string(&notification).unwrap();

        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"method\":\"notifications/initialized\""));
        assert!(!json_str.contains("\"id\""));
    }

    #[tokio::test]
    async fn test_response_message_enum() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: Some(json!({"success": true})),
            error: None,
        };

        let msg = ResponseMessage::Response(response.clone());
        match msg {
            ResponseMessage::Response(r) => {
                assert_eq!(r.id, Value::Number(1.into()));
                assert!(r.result.is_some());
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[tokio::test]
    async fn test_notification_message() {
        let notification = JsonRpcRequest::notification("test", None);
        let msg = ResponseMessage::Notification(notification.clone());

        match msg {
            ResponseMessage::Notification(n) => {
                assert_eq!(n.method, "test");
                assert!(n.id.is_none());
            }
            _ => panic!("Expected Notification variant"),
        }
    }

    #[test]
    fn test_initialize_params_construction() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        assert_eq!(params.protocol_version, "2024-11-05");
        assert_eq!(params.client_info.name, env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_call_tool_params_construction() {
        let mut args = HashMap::new();
        args.insert("param1".to_string(), json!("value1"));
        args.insert("param2".to_string(), json!(42));

        let params = CallToolParams {
            name: "my_tool".to_string(),
            arguments: Some(args),
        };

        assert_eq!(params.name, "my_tool");
        assert_eq!(params.arguments.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_get_prompt_params_construction() {
        let mut args = HashMap::new();
        args.insert("arg1".to_string(), "val1".to_string());

        let params = GetPromptParams {
            name: "test_prompt".to_string(),
            arguments: Some(args),
        };

        assert_eq!(params.name, "test_prompt");
        assert!(params.arguments.is_some());
    }

    #[test]
    fn test_read_resource_params_construction() {
        let params = ReadResourceParams {
            uri: "file:///path/to/resource".to_string(),
        };

        assert_eq!(params.uri, "file:///path/to/resource");
    }

    #[tokio::test]
    async fn test_pending_requests_map() {
        let pending: Arc<Mutex<HashMap<i64, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (tx, _rx) = oneshot::channel();

        {
            let mut map = pending.lock().await;
            map.insert(1i64, tx);
        }

        let map = pending.lock().await;
        assert!(map.contains_key(&1i64));
    }

    #[tokio::test]
    async fn test_server_info_storage() {
        let server_info = Arc::new(Mutex::new(None));

        let info = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: "test_server".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        *server_info.lock().await = Some(info.clone());

        let stored = server_info.lock().await;
        assert!(stored.is_some());
        assert_eq!(stored.as_ref().unwrap().server_info.name, "test_server");
    }

    #[test]
    fn test_jsonrpc_error_structure() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(json!({"details": "Additional info"})),
        };

        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
        assert!(error.data.is_some());
    }
}
