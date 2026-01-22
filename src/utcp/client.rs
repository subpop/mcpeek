use super::executor::ToolExecutor;
use super::protocol::*;
use super::template::TemplateProcessor;
use crate::protocol::*;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// UTCP client that loads and executes tools from a manual file
pub struct UtcpClient {
    manual: UtcpManual,
    executor: Arc<ToolExecutor>,
    logs: Arc<Mutex<Vec<String>>>,
    server_info: Option<ServerInfo>,
}

impl UtcpClient {
    /// Create a new UTCP client by loading a manual from the given path
    pub async fn new(manual_path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(manual_path.as_ref())
            .await
            .context("Failed to read UTCP manual file")?;

        let manual: UtcpManual =
            serde_json::from_str(&content).context("Failed to parse UTCP manual JSON")?;

        let template_processor = TemplateProcessor::new(manual.variables.clone());
        let executor = Arc::new(ToolExecutor::new(template_processor));

        let server_info = Some(ServerInfo {
            name: manual.info.title.clone(),
            version: manual.info.version.clone(),
            protocol_type: "UTCP".to_string(),
            capabilities: vec![
                format!("{} tools", manual.tools.len()),
                "HTTP execution".to_string(),
                "CLI execution".to_string(),
            ],
        });

        Ok(Self {
            manual,
            executor,
            logs: Arc::new(Mutex::new(Vec::new())),
            server_info,
        })
    }

    /// Add a log message
    async fn log(&self, message: String) {
        self.logs.lock().await.push(message);
    }
}

#[async_trait::async_trait]
impl ProtocolClient for UtcpClient {
    async fn initialize(&self) -> Result<ServerInfo> {
        self.log("UTCP client initialized".to_string()).await;
        self.server_info
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Server info not available"))
    }

    async fn shutdown(&self) -> Result<()> {
        self.log("UTCP client shutdown".to_string()).await;
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ProtocolTool>> {
        Ok(self
            .manual
            .tools
            .iter()
            .map(|tool| ProtocolTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.inputs.clone(),
            })
            .collect())
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<ToolCallResult> {
        let tool = self
            .manual
            .tools
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", name))?;

        self.log(format!("Calling tool: {} with args: {:?}", name, arguments))
            .await;

        let result = self.executor.execute_tool(tool, arguments).await?;

        self.log(format!("Tool '{}' execution completed", name))
            .await;

        Ok(result)
    }

    async fn list_prompts(&self) -> Result<Vec<ProtocolPrompt>> {
        // UTCP doesn't have prompts - return empty list
        Ok(Vec::new())
    }

    async fn get_prompt(
        &self,
        _name: &str,
        _arguments: Option<HashMap<String, String>>,
    ) -> Result<PromptResult> {
        anyhow::bail!("UTCP does not support prompts")
    }

    async fn list_resources(&self) -> Result<Vec<ProtocolResource>> {
        // UTCP doesn't have resources - return empty list
        Ok(Vec::new())
    }

    async fn read_resource(&self, _uri: &str) -> Result<ResourceReadResult> {
        anyhow::bail!("UTCP does not support resources")
    }

    async fn get_server_info(&self) -> Option<ServerInfo> {
        self.server_info.clone()
    }

    async fn get_logs(&self) -> Vec<String> {
        self.logs.lock().await.clone()
    }
}
