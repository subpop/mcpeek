use crate::logging::LogEntry;
use crate::mcp::protocol::*;
use crate::mcp::McpClient;
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tools,
    Prompts,
    Resources,
    ServerInfo,
    ServerLogs,
    DebugLogs,
}

impl Tab {
    pub fn next(&self, debug_mode: bool) -> Self {
        match self {
            Tab::Tools => Tab::Prompts,
            Tab::Prompts => Tab::Resources,
            Tab::Resources => Tab::ServerInfo,
            Tab::ServerInfo => Tab::ServerLogs,
            Tab::ServerLogs => {
                if debug_mode {
                    Tab::DebugLogs
                } else {
                    Tab::Tools
                }
            }
            Tab::DebugLogs => Tab::Tools,
        }
    }

    pub fn previous(&self, debug_mode: bool) -> Self {
        match self {
            Tab::Tools => {
                if debug_mode {
                    Tab::DebugLogs
                } else {
                    Tab::ServerLogs
                }
            }
            Tab::Prompts => Tab::Tools,
            Tab::Resources => Tab::Prompts,
            Tab::ServerInfo => Tab::Resources,
            Tab::ServerLogs => Tab::ServerInfo,
            Tab::DebugLogs => Tab::ServerLogs,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Tab::Tools => "Tools",
            Tab::Prompts => "Prompts",
            Tab::Resources => "Resources",
            Tab::ServerInfo => "Server Info",
            Tab::ServerLogs => "Server Logs",
            Tab::DebugLogs => "Debug Logs",
        }
    }
}

pub struct App {
    pub current_tab: Tab,
    pub tools: Vec<Tool>,
    pub prompts: Vec<Prompt>,
    pub resources: Vec<Resource>,
    pub server_info: Option<InitializeResult>,
    pub logs: Vec<String>,
    pub debug_logs: Vec<LogEntry>,
    pub debug_mode: bool,
    pub selected_tool: usize,
    pub selected_prompt: usize,
    pub selected_resource: usize,
    pub log_scroll: usize,
    pub debug_log_scroll: usize,
    pub detail_scroll: usize,
    pub server_info_scroll: usize,
    pub loading: bool,
    pub error_message: Option<String>,
    pub detail_view: Option<String>,
    pub should_quit: bool,
    // Tool calling state
    pub tool_call_input_mode: bool,
    pub tool_call_inputs: HashMap<String, String>,
    pub tool_call_result: Option<CallToolResult>,
    pub input_field_index: usize,
    pub input_fields: Vec<InputField>,
    pub tool_input_scroll: usize,
    // Prompt input state
    pub prompt_input_mode: bool,
    pub prompt_inputs: HashMap<String, String>,
    pub prompt_result: Option<GetPromptResult>,
    // Resource read state
    pub resource_read_result: Option<Vec<ResourceContents>>,
}

#[derive(Debug, Clone)]
pub struct InputField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: Option<String>,
}

impl App {
    pub fn new(debug_mode: bool) -> Self {
        Self {
            current_tab: Tab::Tools,
            tools: Vec::new(),
            prompts: Vec::new(),
            resources: Vec::new(),
            server_info: None,
            logs: Vec::new(),
            debug_logs: Vec::new(),
            debug_mode,
            selected_tool: 0,
            selected_prompt: 0,
            selected_resource: 0,
            log_scroll: 0,
            debug_log_scroll: 0,
            detail_scroll: 0,
            server_info_scroll: 0,
            loading: true,
            error_message: None,
            detail_view: None,
            should_quit: false,
            tool_call_input_mode: false,
            tool_call_inputs: HashMap::new(),
            tool_call_result: None,
            input_field_index: 0,
            input_fields: Vec::new(),
            tool_input_scroll: 0,
            prompt_input_mode: false,
            prompt_inputs: HashMap::new(),
            prompt_result: None,
            resource_read_result: None,
        }
    }

    pub async fn load_data(&mut self, client: &McpClient) -> Result<()> {
        self.loading = true;
        self.error_message = None;

        match self.current_tab {
            Tab::Tools => match client.list_tools().await {
                Ok(tools) => {
                    self.tools = tools;
                    if self.selected_tool >= self.tools.len() && !self.tools.is_empty() {
                        self.selected_tool = self.tools.len() - 1;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load tools: {}", e));
                }
            },
            Tab::Prompts => match client.list_prompts().await {
                Ok(prompts) => {
                    self.prompts = prompts;
                    if self.selected_prompt >= self.prompts.len() && !self.prompts.is_empty() {
                        self.selected_prompt = self.prompts.len() - 1;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load prompts: {}", e));
                }
            },
            Tab::Resources => match client.list_resources().await {
                Ok(resources) => {
                    self.resources = resources;
                    if self.selected_resource >= self.resources.len() && !self.resources.is_empty()
                    {
                        self.selected_resource = self.resources.len() - 1;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load resources: {}", e));
                }
            },
            Tab::ServerInfo => {
                self.server_info = client.get_server_info().await;
            }
            Tab::ServerLogs => {
                let new_logs = client.get_logs().await;
                self.logs.extend(new_logs);
            }
            Tab::DebugLogs => {
                // Debug logs are updated separately via update_debug_logs
            }
        }

        self.loading = false;
        Ok(())
    }

    pub async fn update_logs(&mut self, client: &McpClient) {
        let new_logs = client.get_logs().await;
        self.logs.extend(new_logs);
    }

    pub fn update_debug_logs(&mut self, logs: Vec<LogEntry>) {
        self.debug_logs = logs;
    }

    pub fn next_item(&mut self) {
        if self.detail_view.is_some() {
            // Scroll detail view
            self.detail_scroll = self.detail_scroll.saturating_add(1);
            return;
        }

        match self.current_tab {
            Tab::Tools if !self.tools.is_empty() => {
                self.selected_tool = (self.selected_tool + 1) % self.tools.len();
            }
            Tab::Prompts if !self.prompts.is_empty() => {
                self.selected_prompt = (self.selected_prompt + 1) % self.prompts.len();
            }
            Tab::Resources if !self.resources.is_empty() => {
                self.selected_resource = (self.selected_resource + 1) % self.resources.len();
            }
            Tab::ServerInfo => {
                self.server_info_scroll = self.server_info_scroll.saturating_add(1);
            }
            Tab::ServerLogs if !self.logs.is_empty() => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            Tab::DebugLogs if !self.debug_logs.is_empty() => {
                self.debug_log_scroll = self.debug_log_scroll.saturating_add(1);
            }
            _ => {}
        }
    }

    pub fn previous_item(&mut self) {
        if self.detail_view.is_some() {
            // Scroll detail view
            self.detail_scroll = self.detail_scroll.saturating_sub(1);
            return;
        }

        match self.current_tab {
            Tab::Tools if !self.tools.is_empty() => {
                self.selected_tool = if self.selected_tool == 0 {
                    self.tools.len() - 1
                } else {
                    self.selected_tool - 1
                };
            }
            Tab::Prompts if !self.prompts.is_empty() => {
                self.selected_prompt = if self.selected_prompt == 0 {
                    self.prompts.len() - 1
                } else {
                    self.selected_prompt - 1
                };
            }
            Tab::Resources if !self.resources.is_empty() => {
                self.selected_resource = if self.selected_resource == 0 {
                    self.resources.len() - 1
                } else {
                    self.selected_resource - 1
                };
            }
            Tab::ServerInfo => {
                self.server_info_scroll = self.server_info_scroll.saturating_sub(1);
            }
            Tab::ServerLogs => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            Tab::DebugLogs => {
                self.debug_log_scroll = self.debug_log_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        match self.current_tab {
            Tab::ServerLogs if !self.logs.is_empty() => {
                self.log_scroll = self.logs.len().saturating_sub(1);
            }
            Tab::DebugLogs if !self.debug_logs.is_empty() => {
                self.debug_log_scroll = self.debug_logs.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn page_down(&mut self) {
        const PAGE_SIZE: usize = 10;

        if self.detail_view.is_some() {
            // Scroll detail view down by page
            self.detail_scroll = self.detail_scroll.saturating_add(PAGE_SIZE);
            return;
        }

        match self.current_tab {
            Tab::ServerInfo => {
                self.server_info_scroll = self.server_info_scroll.saturating_add(PAGE_SIZE);
            }
            Tab::ServerLogs if !self.logs.is_empty() => {
                self.log_scroll = self.log_scroll.saturating_add(PAGE_SIZE);
            }
            Tab::DebugLogs if !self.debug_logs.is_empty() => {
                self.debug_log_scroll = self.debug_log_scroll.saturating_add(PAGE_SIZE);
            }
            _ => {}
        }
    }

    pub fn page_up(&mut self) {
        const PAGE_SIZE: usize = 10;

        if self.detail_view.is_some() {
            // Scroll detail view up by page
            self.detail_scroll = self.detail_scroll.saturating_sub(PAGE_SIZE);
            return;
        }

        match self.current_tab {
            Tab::ServerInfo => {
                self.server_info_scroll = self.server_info_scroll.saturating_sub(PAGE_SIZE);
            }
            Tab::ServerLogs => {
                self.log_scroll = self.log_scroll.saturating_sub(PAGE_SIZE);
            }
            Tab::DebugLogs => {
                self.debug_log_scroll = self.debug_log_scroll.saturating_sub(PAGE_SIZE);
            }
            _ => {}
        }
    }

    pub fn show_detail(&mut self) {
        match self.current_tab {
            Tab::Tools if !self.tools.is_empty() => {
                let tool = &self.tools[self.selected_tool];
                let detail = format!(
                    "Tool: {}\n\nDescription: {}\n\nInput Schema:\n{}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("No description"),
                    serde_json::to_string_pretty(&tool.input_schema).unwrap_or_default()
                );
                self.detail_view = Some(detail);
            }
            Tab::Prompts if !self.prompts.is_empty() => {
                let prompt = &self.prompts[self.selected_prompt];
                let args = if let Some(arguments) = &prompt.arguments {
                    arguments
                        .iter()
                        .map(|arg| {
                            format!(
                                "  - {} ({}): {}",
                                arg.name,
                                if arg.required.unwrap_or(false) {
                                    "required"
                                } else {
                                    "optional"
                                },
                                arg.description.as_deref().unwrap_or("No description")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    "  None".to_string()
                };

                let detail = format!(
                    "Prompt: {}\n\nDescription: {}\n\nArguments:\n{}",
                    prompt.name,
                    prompt.description.as_deref().unwrap_or("No description"),
                    args
                );
                self.detail_view = Some(detail);
            }
            Tab::Resources if !self.resources.is_empty() => {
                let resource = &self.resources[self.selected_resource];
                let detail = format!(
                    "Resource: {}\n\nURI: {}\n\nDescription: {}\n\nMIME Type: {}",
                    resource.name,
                    resource.uri,
                    resource.description.as_deref().unwrap_or("No description"),
                    resource.mime_type.as_deref().unwrap_or("Unknown")
                );
                self.detail_view = Some(detail);
            }
            Tab::ServerInfo => {
                if let Some(info) = &self.server_info {
                    let caps = &info.capabilities;
                    let mut detail = format!(
                        "Server: {} v{}\n\nProtocol Version: {}\n\nCapabilities:\n  Tools: {}\n  Prompts: {}\n  Resources: {}\n  Logging: {}",
                        info.server_info.name,
                        info.server_info.version,
                        info.protocol_version,
                        if caps.tools.is_some() { "Yes" } else { "No" },
                        if caps.prompts.is_some() { "Yes" } else { "No" },
                        if caps.resources.is_some() { "Yes" } else { "No" },
                        if caps.logging.is_some() { "Yes" } else { "No" },
                    );
                    if let Some(instructions) = &info.instructions {
                        detail.push_str("\n\nInstructions:\n");
                        for line in instructions.lines() {
                            detail.push_str(&format!("  {}\n", line));
                        }
                    }
                    self.detail_view = Some(detail);
                }
            }
            _ => {}
        }
    }

    pub fn close_detail(&mut self) {
        self.detail_view = None;
        self.detail_scroll = 0;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn start_tool_call(&mut self) {
        if self.current_tab != Tab::Tools || self.tools.is_empty() {
            return;
        }

        let tool = &self.tools[self.selected_tool];
        self.input_fields = parse_input_schema(&tool.input_schema);
        self.tool_call_inputs.clear();
        self.input_field_index = 0;
        self.tool_input_scroll = 0;
        self.tool_call_input_mode = true;
        self.tool_call_result = None;
    }

    pub fn next_input_field(&mut self) {
        if !self.input_fields.is_empty() {
            self.input_field_index = (self.input_field_index + 1) % self.input_fields.len();
        }
    }

    pub fn previous_input_field(&mut self) {
        if !self.input_fields.is_empty() {
            self.input_field_index = if self.input_field_index == 0 {
                self.input_fields.len() - 1
            } else {
                self.input_field_index - 1
            };
        }
    }

    pub fn update_current_input(&mut self, c: char) {
        if self.input_fields.is_empty() {
            return;
        }
        let field_name = &self.input_fields[self.input_field_index].name;

        if self.tool_call_input_mode {
            self.tool_call_inputs
                .entry(field_name.clone())
                .or_default()
                .push(c);
        } else if self.prompt_input_mode {
            self.prompt_inputs
                .entry(field_name.clone())
                .or_default()
                .push(c);
        }
    }

    pub fn delete_current_input(&mut self) {
        if self.input_fields.is_empty() {
            return;
        }
        let field_name = &self.input_fields[self.input_field_index].name;

        if self.tool_call_input_mode {
            if let Some(value) = self.tool_call_inputs.get_mut(field_name) {
                value.pop();
            }
        } else if self.prompt_input_mode {
            if let Some(value) = self.prompt_inputs.get_mut(field_name) {
                value.pop();
            }
        }
    }

    pub async fn execute_tool_call(&mut self, client: &McpClient) {
        if self.tools.is_empty() {
            return;
        }

        let tool = &self.tools[self.selected_tool];

        // Validate required fields
        for field in &self.input_fields {
            if field.required {
                let value = self
                    .tool_call_inputs
                    .get(&field.name)
                    .map(|s| s.trim())
                    .unwrap_or("");
                if value.is_empty() {
                    self.error_message = Some(format!("Required field '{}' is empty", field.name));
                    return;
                }
            }
        }

        // Convert inputs to JSON values
        let mut arguments = HashMap::new();
        for field in &self.input_fields {
            if let Some(value_str) = self.tool_call_inputs.get(&field.name) {
                let value_str = value_str.trim();
                if !value_str.is_empty() {
                    let json_value = match field.field_type.as_str() {
                        "number" | "integer" => {
                            if let Ok(num) = value_str.parse::<i64>() {
                                Value::Number(num.into())
                            } else if let Ok(num) = value_str.parse::<f64>() {
                                Value::Number(
                                    serde_json::Number::from_f64(num).unwrap_or_else(|| 0.into()),
                                )
                            } else {
                                self.error_message =
                                    Some(format!("'{}' must be a number", field.name));
                                return;
                            }
                        }
                        "boolean" => match value_str.to_lowercase().as_str() {
                            "true" | "yes" | "1" => Value::Bool(true),
                            "false" | "no" | "0" => Value::Bool(false),
                            _ => {
                                self.error_message =
                                    Some(format!("'{}' must be true or false", field.name));
                                return;
                            }
                        },
                        "array" | "object" => {
                            // Try to parse as JSON
                            match serde_json::from_str(value_str) {
                                Ok(v) => v,
                                Err(_) => {
                                    self.error_message =
                                        Some(format!("'{}' must be valid JSON", field.name));
                                    return;
                                }
                            }
                        }
                        _ => Value::String(value_str.to_string()),
                    };
                    arguments.insert(field.name.clone(), json_value);
                }
            }
        }

        // Call the tool
        let tool_name = tool.name.clone();
        match client
            .call_tool(
                &tool_name,
                if arguments.is_empty() {
                    None
                } else {
                    Some(arguments)
                },
            )
            .await
        {
            Ok(result) => {
                self.tool_call_result = Some(result.clone());
                self.tool_call_input_mode = false;

                // Show result in detail view
                let detail = format_tool_result(&tool_name, &result);
                self.detail_view = Some(detail);
            }
            Err(e) => {
                self.error_message = Some(format!("Tool call failed: {}", e));
            }
        }
    }

    pub fn cancel_tool_call(&mut self) {
        self.tool_call_input_mode = false;
        self.tool_call_inputs.clear();
        self.input_fields.clear();
        self.input_field_index = 0;
        self.tool_input_scroll = 0;
    }

    pub fn scroll_tool_input_up(&mut self) {
        self.tool_input_scroll = self.tool_input_scroll.saturating_sub(1);
    }

    pub fn scroll_tool_input_down(&mut self) {
        // We'll add bounds checking in the UI render function
        self.tool_input_scroll = self.tool_input_scroll.saturating_add(1);
    }

    pub fn start_prompt_get(&mut self) {
        if self.current_tab != Tab::Prompts || self.prompts.is_empty() {
            return;
        }

        let prompt = &self.prompts[self.selected_prompt];

        // Convert PromptArgument to InputField
        self.input_fields = if let Some(arguments) = &prompt.arguments {
            arguments
                .iter()
                .map(|arg| InputField {
                    name: arg.name.clone(),
                    field_type: "string".to_string(),
                    required: arg.required.unwrap_or(false),
                    description: arg.description.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };

        self.prompt_inputs.clear();
        self.input_field_index = 0;
        self.tool_input_scroll = 0;
        self.prompt_input_mode = true;
        self.prompt_result = None;
    }

    pub async fn execute_prompt_get(&mut self, client: &McpClient) {
        if self.prompts.is_empty() {
            return;
        }

        let prompt = &self.prompts[self.selected_prompt];

        // Validate required fields
        for field in &self.input_fields {
            if field.required {
                let value = self
                    .prompt_inputs
                    .get(&field.name)
                    .map(|s| s.trim())
                    .unwrap_or("");
                if value.is_empty() {
                    self.error_message = Some(format!("Required field '{}' is empty", field.name));
                    return;
                }
            }
        }

        // Build arguments map (only non-empty values)
        let mut arguments = HashMap::new();
        for field in &self.input_fields {
            if let Some(value_str) = self.prompt_inputs.get(&field.name) {
                let value_str = value_str.trim();
                if !value_str.is_empty() {
                    arguments.insert(field.name.clone(), value_str.to_string());
                }
            }
        }

        // Get the prompt
        let prompt_name = prompt.name.clone();
        match client
            .get_prompt(
                &prompt_name,
                if arguments.is_empty() {
                    None
                } else {
                    Some(arguments)
                },
            )
            .await
        {
            Ok(result) => {
                self.prompt_result = Some(result.clone());
                self.prompt_input_mode = false;

                // Show result in detail view
                let detail = format_prompt_result(&prompt_name, &result);
                self.detail_view = Some(detail);
            }
            Err(e) => {
                self.error_message = Some(format!("Prompt get failed: {}", e));
            }
        }
    }

    pub fn cancel_prompt_input(&mut self) {
        self.prompt_input_mode = false;
        self.prompt_inputs.clear();
        self.input_fields.clear();
        self.input_field_index = 0;
        self.tool_input_scroll = 0;
    }

    pub async fn read_resource(&mut self, client: &McpClient) {
        if self.resources.is_empty() {
            return;
        }

        let resource = &self.resources[self.selected_resource];
        let uri = resource.uri.clone();
        let resource_name = resource.name.clone();

        match client.read_resource(&uri).await {
            Ok(contents) => {
                self.resource_read_result = Some(contents.clone());

                // Show result in detail view
                let detail = format_resource_read_result(&resource_name, &uri, &contents);
                self.detail_view = Some(detail);
                self.error_message = None; // Clear any previous errors
            }
            Err(e) => {
                let error_msg = format!("Failed to read resource '{}': {:#}", resource_name, e);
                self.error_message = Some(error_msg);
            }
        }
    }

    pub fn export_logs(&self) -> Result<String> {
        #[derive(Serialize)]
        struct LogExport {
            metadata: ExportMetadata,
            server_logs: Vec<String>,
            debug_logs: Vec<LogEntry>,
        }

        #[derive(Serialize)]
        struct ExportMetadata {
            export_timestamp: String,
            application_version: String,
            server_log_count: usize,
            debug_log_count: usize,
        }

        let export = LogExport {
            metadata: ExportMetadata {
                export_timestamp: chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                application_version: env!("CARGO_PKG_VERSION").to_string(),
                server_log_count: self.logs.len(),
                debug_log_count: self.debug_logs.len(),
            },
            server_logs: self.logs.clone(),
            debug_logs: self.debug_logs.clone(),
        };

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("mcpeek_logs_{}.json", timestamp);

        let json = serde_json::to_string_pretty(&export)?;
        std::fs::write(&filename, json)?;

        Ok(filename)
    }
}

fn parse_input_schema(schema: &Value) -> Vec<InputField> {
    let mut fields = Vec::new();

    // Handle JSON Schema object
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let required_fields: Vec<String> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        for (name, prop) in properties {
            let field_type = prop
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("string")
                .to_string();

            let description = prop
                .get("description")
                .and_then(|d| d.as_str())
                .map(String::from);

            let required = required_fields.contains(name);

            fields.push(InputField {
                name: name.clone(),
                field_type,
                required,
                description,
            });
        }
    }

    // Sort required fields first
    fields.sort_by_key(|f| !f.required);
    fields
}

fn format_tool_result(tool_name: &str, result: &CallToolResult) -> String {
    let mut output = format!("Tool Call Result: {}\n\n", tool_name);

    if result.is_error.unwrap_or(false) {
        output.push_str("Status: ERROR\n\n");
    } else {
        output.push_str("Status: SUCCESS\n\n");
    }

    output.push_str("Content:\n");
    for (i, content) in result.content.iter().enumerate() {
        if i > 0 {
            output.push_str("\n---\n\n");
        }
        match content {
            ToolContent::Text { text } => {
                output.push_str(text);
            }
            ToolContent::Image { data, mime_type } => {
                output.push_str(&format!("[Image: {} ({} bytes)]\n", mime_type, data.len()));
            }
            ToolContent::Resource { resource } => match resource {
                ResourceContents::Text {
                    uri,
                    text,
                    mime_type,
                } => {
                    output.push_str(&format!("[Resource: {}]\n", uri));
                    if let Some(mt) = mime_type {
                        output.push_str(&format!("MIME Type: {}\n\n", mt));
                    }
                    output.push_str(text);
                }
                ResourceContents::Blob { uri, mime_type, .. } => {
                    output.push_str(&format!("[Binary Resource: {}]\n", uri));
                    if let Some(mt) = mime_type {
                        output.push_str(&format!("MIME Type: {}\n", mt));
                    }
                }
            },
        }
    }

    output
}

fn format_prompt_result(prompt_name: &str, result: &GetPromptResult) -> String {
    let mut output = format!("Prompt Result: {}\n\n", prompt_name);

    if let Some(desc) = &result.description {
        output.push_str(&format!("Description: {}\n\n", desc));
    }

    output.push_str(&format!("Messages ({}):\n\n", result.messages.len()));

    for (i, message) in result.messages.iter().enumerate() {
        if i > 0 {
            output.push_str("\n---\n\n");
        }

        output.push_str(&format!("Role: {}\n\n", message.role));
        output.push_str("Content:\n");

        match &message.content {
            PromptMessageContent::Single(content) => {
                format_prompt_content(&mut output, content);
            }
            PromptMessageContent::Multiple(contents) => {
                for (j, content) in contents.iter().enumerate() {
                    if j > 0 {
                        output.push('\n');
                    }
                    format_prompt_content(&mut output, content);
                }
            }
        }
        output.push('\n');
    }

    output
}

fn format_prompt_content(output: &mut String, content: &PromptContent) {
    match content {
        PromptContent::Text { text } => {
            output.push_str(text);
        }
        PromptContent::Image { data, mime_type } => {
            output.push_str(&format!("[Image: {} ({} bytes)]", mime_type, data.len()));
        }
        PromptContent::Resource { resource } => match resource {
            ResourceContents::Text {
                uri,
                text,
                mime_type,
            } => {
                output.push_str(&format!("[Resource: {}]\n", uri));
                if let Some(mt) = mime_type {
                    output.push_str(&format!("MIME Type: {}\n\n", mt));
                }
                output.push_str(text);
            }
            ResourceContents::Blob { uri, mime_type, .. } => {
                output.push_str(&format!("[Binary Resource: {}]\n", uri));
                if let Some(mt) = mime_type {
                    output.push_str(&format!("MIME Type: {}", mt));
                }
            }
        },
    }
}

fn format_resource_read_result(
    resource_name: &str,
    uri: &str,
    contents: &[ResourceContents],
) -> String {
    let mut output = format!(
        "Resource Read Result: {}\n\nURI: {}\n\n",
        resource_name, uri
    );

    if contents.is_empty() {
        output.push_str("(empty resource)\n");
        return output;
    }

    output.push_str(&format!("Contents ({}):\n\n", contents.len()));

    for (i, content) in contents.iter().enumerate() {
        if i > 0 {
            output.push_str("\n---\n\n");
        }

        match content {
            ResourceContents::Text {
                uri: content_uri,
                text,
                mime_type,
            } => {
                if content_uri != uri {
                    output.push_str(&format!("URI: {}\n", content_uri));
                }
                if let Some(mt) = mime_type {
                    output.push_str(&format!("MIME Type: {}\n\n", mt));
                }
                output.push_str(text);
            }
            ResourceContents::Blob {
                uri: content_uri,
                blob,
                mime_type,
            } => {
                if content_uri != uri {
                    output.push_str(&format!("URI: {}\n", content_uri));
                }
                output.push_str("[Binary Content]\n");
                if let Some(mt) = mime_type {
                    output.push_str(&format!("MIME Type: {}\n", mt));
                }
                output.push_str(&format!("Size: {} bytes (base64 encoded)\n", blob.len()));
            }
        }
    }

    output
}
