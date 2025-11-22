use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: i64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(id.into())),
            method: method.into(),
            params,
        }
    }

    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// MCP Protocol Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

// Tools

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContents },
}

// Prompts

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptMessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptMessageContent {
    Single(PromptContent),
    Multiple(Vec<PromptContent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContents },
}

// Resources

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContents {
    Text {
        uri: String,
        text: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
    },
    Blob {
        uri: String,
        blob: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_new() {
        let request = JsonRpcRequest::new(1, "test_method", Some(json!({"key": "value"})));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(Value::Number(1.into())));
        assert_eq!(request.method, "test_method");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_jsonrpc_request_notification() {
        let notification = JsonRpcRequest::notification("test_notification", None);

        assert_eq!(notification.jsonrpc, "2.0");
        assert!(notification.id.is_none());
        assert_eq!(notification.method, "test_notification");
        assert!(notification.params.is_none());
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(42, "initialize", Some(json!({"version": "1.0"})));
        let json_str = serde_json::to_string(&request).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.id, Some(Value::Number(42.into())));
        assert_eq!(parsed.method, "initialize");
    }

    #[test]
    fn test_jsonrpc_response_with_result() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json_str).unwrap();

        assert!(parsed.result.is_some());
        assert!(parsed.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_with_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json_str).unwrap();

        assert!(parsed.result.is_none());
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, -32600);
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: true }),
                sampling: None,
            },
            client_info: Implementation {
                name: "test_client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("protocolVersion"));
        assert!(json_str.contains("clientInfo"));

        let parsed: InitializeParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.protocol_version, "2024-11-05");
        assert_eq!(parsed.client_info.name, "test_client");
    }

    #[test]
    fn test_initialize_params_optional_fields_skipped() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: Implementation {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
        };

        let json_str = serde_json::to_string(&params).unwrap();
        assert!(!json_str.contains("roots"));
        assert!(!json_str.contains("sampling"));
    }

    #[test]
    fn test_server_capabilities_default() {
        let caps = ServerCapabilities::default();
        assert!(caps.tools.is_none());
        assert!(caps.prompts.is_none());
        assert!(caps.resources.is_none());
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "arg1": {"type": "string"}
                }
            }),
        };

        let json_str = serde_json::to_string(&tool).unwrap();
        assert!(json_str.contains("inputSchema"));

        let parsed: Tool = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.name, "test_tool");
        assert_eq!(parsed.description, Some("A test tool".to_string()));
    }

    #[test]
    fn test_tool_content_text_variant() {
        let content = ToolContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json_str = serde_json::to_string(&content).unwrap();
        assert!(json_str.contains(r#""type":"text"#));
        assert!(json_str.contains("Hello, world!"));

        let parsed: ToolContent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            ToolContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_tool_content_image_variant() {
        let content = ToolContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json_str = serde_json::to_string(&content).unwrap();
        assert!(json_str.contains(r#""type":"image"#));
        assert!(json_str.contains("mimeType"));

        let parsed: ToolContent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_call_tool_params() {
        let mut args = HashMap::new();
        args.insert("param1".to_string(), json!("value1"));

        let params = CallToolParams {
            name: "my_tool".to_string(),
            arguments: Some(args),
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: CallToolParams = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.name, "my_tool");
        assert!(parsed.arguments.is_some());
    }

    #[test]
    fn test_call_tool_result() {
        let result = CallToolResult {
            content: vec![ToolContent::Text {
                text: "Success".to_string(),
            }],
            is_error: Some(false),
        };

        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("isError"));

        let parsed: CallToolResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.content.len(), 1);
        assert_eq!(parsed.is_error, Some(false));
    }

    #[test]
    fn test_prompt_with_arguments() {
        let prompt = Prompt {
            name: "test_prompt".to_string(),
            description: Some("Test description".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "arg1".to_string(),
                description: Some("First arg".to_string()),
                required: Some(true),
            }]),
        };

        let json_str = serde_json::to_string(&prompt).unwrap();
        let parsed: Prompt = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.name, "test_prompt");
        assert_eq!(parsed.arguments.unwrap().len(), 1);
    }

    #[test]
    fn test_prompt_content_variants() {
        let text_content = PromptContent::Text {
            text: "Prompt text".to_string(),
        };

        let json_str = serde_json::to_string(&text_content).unwrap();
        assert!(json_str.contains(r#""type":"text"#));

        let parsed: PromptContent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            PromptContent::Text { text } => assert_eq!(text, "Prompt text"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "test.txt".to_string(),
            description: Some("A test file".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json_str = serde_json::to_string(&resource).unwrap();
        assert!(json_str.contains("mimeType"));

        let parsed: Resource = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.uri, "file:///test.txt");
        assert_eq!(parsed.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_resource_contents_text_variant() {
        let contents = ResourceContents::Text {
            uri: "file:///test.txt".to_string(),
            text: "File contents".to_string(),
            mime_type: Some("text/plain".to_string()),
        };

        let json_str = serde_json::to_string(&contents).unwrap();
        assert!(json_str.contains(r#""text":"File contents"#));

        let parsed: ResourceContents = serde_json::from_str(&json_str).unwrap();
        match parsed {
            ResourceContents::Text { uri, text, .. } => {
                assert_eq!(uri, "file:///test.txt");
                assert_eq!(text, "File contents");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_resource_contents_blob_variant() {
        let contents = ResourceContents::Blob {
            uri: "file:///test.bin".to_string(),
            blob: "base64data".to_string(),
            mime_type: Some("application/octet-stream".to_string()),
        };

        let json_str = serde_json::to_string(&contents).unwrap();
        assert!(json_str.contains(r#""blob":"base64data"#));

        let parsed: ResourceContents = serde_json::from_str(&json_str).unwrap();
        match parsed {
            ResourceContents::Blob { blob, .. } => {
                assert_eq!(blob, "base64data");
            }
            _ => panic!("Expected Blob variant"),
        }
    }

    #[test]
    fn test_get_prompt_params() {
        let mut args = HashMap::new();
        args.insert("key".to_string(), "value".to_string());

        let params = GetPromptParams {
            name: "my_prompt".to_string(),
            arguments: Some(args),
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: GetPromptParams = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.name, "my_prompt");
        assert!(parsed.arguments.is_some());
    }

    #[test]
    fn test_read_resource_params() {
        let params = ReadResourceParams {
            uri: "file:///test.txt".to_string(),
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: ReadResourceParams = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.uri, "file:///test.txt");
    }

    #[test]
    fn test_list_tools_result() {
        let result = ListToolsResult {
            tools: vec![Tool {
                name: "tool1".to_string(),
                description: None,
                input_schema: json!({}),
            }],
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let parsed: ListToolsResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "tool1");
    }

    #[test]
    fn test_camelcase_conversion() {
        // Test that snake_case fields are converted to camelCase
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: None,
                sampling: None,
            },
            client_info: Implementation {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let json_value: Value = serde_json::from_str(&json_str).unwrap();

        assert!(json_value.get("protocolVersion").is_some());
        assert!(json_value.get("clientInfo").is_some());
        assert!(json_value.get("protocol_version").is_none());
        assert!(json_value.get("client_info").is_none());
    }
}
