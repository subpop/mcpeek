use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Common wrapper type for tools across different protocols
#[derive(Debug, Clone)]
pub struct ProtocolTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// Common wrapper type for prompts across different protocols
#[derive(Debug, Clone)]
pub struct ProtocolPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// Common wrapper type for resources across different protocols
#[derive(Debug, Clone)]
pub struct ProtocolResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Result of calling a tool
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub content: Vec<ContentItem>,
    pub is_error: bool,
}

/// Content item that can be text, image, or binary data
#[derive(Debug, Clone)]
pub enum ContentItem {
    Text(String),
    Image { data: String, mime_type: String },
    Binary { data: Vec<u8>, mime_type: String },
}

/// Result of getting a prompt
#[derive(Debug, Clone)]
pub struct PromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
}

/// Result of reading a resource
#[derive(Debug, Clone)]
pub struct ResourceReadResult {
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Clone)]
pub enum ResourceContent {
    Text {
        uri: String,
        text: String,
        mime_type: Option<String>,
    },
    Binary {
        uri: String,
        data: Vec<u8>,
        mime_type: Option<String>,
    },
}

/// Server/manual information
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_type: String, // "MCP" or "UTCP"
    pub capabilities: Vec<String>,
}

/// Trait that both MCP and UTCP clients implement
#[async_trait]
pub trait ProtocolClient: Send + Sync {
    /// Initialize the client and return server info
    async fn initialize(&self) -> Result<ServerInfo>;

    /// Shutdown the client
    async fn shutdown(&self) -> Result<()>;

    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ProtocolTool>>;

    /// Call a tool with the given name and arguments
    async fn call_tool(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<ToolCallResult>;

    /// List available prompts
    async fn list_prompts(&self) -> Result<Vec<ProtocolPrompt>>;

    /// Get a prompt with the given name and arguments
    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<PromptResult>;

    /// List available resources
    async fn list_resources(&self) -> Result<Vec<ProtocolResource>>;

    /// Read a resource with the given URI
    async fn read_resource(&self, uri: &str) -> Result<ResourceReadResult>;

    /// Get server/manual information
    async fn get_server_info(&self) -> Option<ServerInfo>;

    /// Get logs from the client
    async fn get_logs(&self) -> Vec<String>;
}
