use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Top-level UTCP manual structure
#[derive(Debug, Clone, Deserialize)]
pub struct UtcpManual {
    pub manual_version: String,
    pub utcp_version: String,
    pub info: ManualInfo,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    pub tools: Vec<UtcpTool>,
}

/// Manual metadata
#[derive(Debug, Clone, Deserialize)]
pub struct ManualInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

/// A tool definition in the UTCP manual
#[derive(Debug, Clone, Deserialize)]
pub struct UtcpTool {
    pub name: String,
    pub description: Option<String>,
    pub inputs: Value,  // JSON Schema
    pub outputs: Value, // JSON Schema
    #[serde(default)]
    pub tags: Vec<String>,
    pub tool_call_template: CallTemplate,
}

/// Call template specifying how to invoke a tool
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "call_template_type")]
pub enum CallTemplate {
    #[serde(rename = "http")]
    Http(HttpTemplate),
    #[serde(rename = "cli")]
    Cli(CliTemplate),
}

/// HTTP call template
#[derive(Debug, Clone, Deserialize)]
pub struct HttpTemplate {
    pub url: String,
    pub http_method: HttpMethod,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub body_field: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// HTTP methods
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "auth_type")]
pub enum AuthConfig {
    #[serde(rename = "api_key")]
    ApiKey {
        api_key: String,
        #[serde(default)]
        header_name: Option<String>,
        #[serde(default)]
        query_param_name: Option<String>,
    },
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "basic")]
    Basic { username: String, password: String },
}

/// CLI call template
#[derive(Debug, Clone, Deserialize)]
pub struct CliTemplate {
    pub commands: Vec<String>,
    #[serde(default)]
    pub append_to_final_output: bool,
}
