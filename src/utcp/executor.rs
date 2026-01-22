use super::protocol::*;
use super::template::TemplateProcessor;
use crate::protocol::{ContentItem, ToolCallResult};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Executes UTCP tools (HTTP and CLI)
pub struct ToolExecutor {
    http_client: reqwest::Client,
    template_processor: TemplateProcessor,
}

impl ToolExecutor {
    /// Create a new tool executor with the given template processor
    pub fn new(template_processor: TemplateProcessor) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            template_processor,
        }
    }

    /// Execute a tool with the given arguments
    pub async fn execute_tool(
        &self,
        tool: &UtcpTool,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<ToolCallResult> {
        match &tool.tool_call_template {
            CallTemplate::Http(http_template) => self.execute_http(http_template, arguments).await,
            CallTemplate::Cli(cli_template) => self.execute_cli(cli_template, arguments).await,
        }
    }

    /// Execute an HTTP tool
    async fn execute_http(
        &self,
        template: &HttpTemplate,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<ToolCallResult> {
        // 1. Substitute variables in URL
        let url = self
            .template_processor
            .substitute(&template.url)
            .context("Failed to substitute variables in URL")?;

        // 2. Substitute URL parameters from arguments
        let url = self.substitute_url_params(&url, &arguments)?;

        // 3. Build request
        let mut request = match template.http_method {
            HttpMethod::Get => self.http_client.get(&url),
            HttpMethod::Post => self.http_client.post(&url),
            HttpMethod::Put => self.http_client.put(&url),
            HttpMethod::Delete => self.http_client.delete(&url),
            HttpMethod::Patch => self.http_client.patch(&url),
        };

        // 4. Add authentication
        if let Some(auth) = &template.auth {
            request = self.add_auth(request, auth)?;
        }

        // 5. Add headers
        let headers = self
            .template_processor
            .substitute_map(&template.headers)
            .context("Failed to substitute variables in headers")?;
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // 6. Add body if applicable
        if let Some(body_field) = &template.body_field {
            if let Some(args) = &arguments {
                if let Some(body_value) = args.get(body_field) {
                    request = request.json(body_value);
                }
            }
        }

        // 7. Execute request
        let response = request.send().await.context("HTTP request failed")?;

        let status = response.status();
        let is_error = !status.is_success();
        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        Ok(ToolCallResult {
            content: vec![ContentItem::Text(body)],
            is_error,
        })
    }

    /// Execute a CLI tool
    async fn execute_cli(
        &self,
        template: &CliTemplate,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<ToolCallResult> {
        use tokio::process::Command;

        let mut output_parts = Vec::new();

        for command_template in &template.commands {
            // Substitute variables
            let command = self
                .template_processor
                .substitute(command_template)
                .context("Failed to substitute variables in command")?;

            // Substitute argument placeholders
            let command = self.substitute_command_args(&command, &arguments)?;

            // Parse command (simple split by whitespace)
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }

            let output = cmd
                .output()
                .await
                .with_context(|| format!("Failed to execute command: {}", command))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if template.append_to_final_output {
                output_parts.push(stdout);
            } else {
                // Return immediately on first command
                let is_error = !output.status.success();
                let content = if !stderr.is_empty() {
                    format!("{}\nSTDERR:\n{}", stdout, stderr)
                } else {
                    stdout
                };

                return Ok(ToolCallResult {
                    content: vec![ContentItem::Text(content)],
                    is_error,
                });
            }
        }

        // Concatenate all outputs
        Ok(ToolCallResult {
            content: vec![ContentItem::Text(output_parts.join("\n"))],
            is_error: false,
        })
    }

    /// Substitute URL path parameters from arguments
    fn substitute_url_params(
        &self,
        url: &str,
        arguments: &Option<HashMap<String, Value>>,
    ) -> Result<String> {
        let mut result = url.to_string();

        if let Some(args) = arguments {
            for (key, value) in args {
                let placeholder = format!("{{{}}}", key);
                if result.contains(&placeholder) {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        v => v.to_string(),
                    };
                    result = result.replace(&placeholder, &value_str);
                }
            }
        }

        Ok(result)
    }

    /// Substitute command arguments from tool arguments
    fn substitute_command_args(
        &self,
        command: &str,
        arguments: &Option<HashMap<String, Value>>,
    ) -> Result<String> {
        self.substitute_url_params(command, arguments)
    }

    /// Add authentication to an HTTP request
    fn add_auth(
        &self,
        request: reqwest::RequestBuilder,
        auth: &AuthConfig,
    ) -> Result<reqwest::RequestBuilder> {
        let request = match auth {
            AuthConfig::ApiKey {
                api_key,
                header_name,
                query_param_name,
            } => {
                let key = self
                    .template_processor
                    .substitute(api_key)
                    .context("Failed to substitute API key")?;

                if let Some(header) = header_name {
                    request.header(header, key)
                } else if let Some(param) = query_param_name {
                    request.query(&[(param, key)])
                } else {
                    // Default to Authorization header
                    request.header("Authorization", format!("ApiKey {}", key))
                }
            }
            AuthConfig::Bearer { token } => {
                let token = self
                    .template_processor
                    .substitute(token)
                    .context("Failed to substitute bearer token")?;
                request.bearer_auth(token)
            }
            AuthConfig::Basic { username, password } => {
                let user = self
                    .template_processor
                    .substitute(username)
                    .context("Failed to substitute username")?;
                let pass = self
                    .template_processor
                    .substitute(password)
                    .context("Failed to substitute password")?;
                request.basic_auth(user, Some(pass))
            }
        };

        Ok(request)
    }
}
