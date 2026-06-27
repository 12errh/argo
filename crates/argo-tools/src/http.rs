use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct HttpTool {
    allowed_domains: Vec<String>,
    max_execution_time: Duration,
}

#[derive(Serialize, Deserialize)]
struct HttpInput {
    method: Option<String>,
    url: String,
    headers: Option<std::collections::HashMap<String, String>>,
    body: Option<String>,
}

impl HttpTool {
    pub fn new(allowed_domains: Vec<String>, max_execution_time: Duration) -> Self {
        Self {
            allowed_domains,
            max_execution_time,
        }
    }

    fn is_domain_allowed(&self, request_url: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(request_url) {
            if let Some(host) = parsed.host_str() {
                return self
                    .allowed_domains
                    .iter()
                    .any(|domain| host.ends_with(domain) || domain.ends_with(host));
            }
        }
        false
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http"
    }

    fn description(&self) -> &str {
        "Make HTTP requests"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"],
                    "description": "HTTP method"
                },
                "url": {
                    "type": "string",
                    "description": "URL to request"
                },
                "headers": {
                    "type": "object",
                    "description": "Request headers"
                },
                "body": {
                    "type": "string",
                    "description": "Request body"
                }
            },
            "required": ["url"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "status": { "type": "integer" },
                "body": { "type": "string" },
                "success": { "type": "boolean" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: false,
            allow_network: true,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: Vec::new(),
            allowed_domains: self.allowed_domains.clone(),
            max_execution_time: self.max_execution_time,
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        let http_input: HttpInput = serde_json::from_value(input).map_err(|e| {
            ToolError::InvalidInput {
                reason: e.to_string(),
            }
        })?;

        if !self.is_domain_allowed(&http_input.url) {
            return Err(ToolError::PermissionDenied {
                resource: http_input.url,
            });
        }

        let client = Client::builder()
            .timeout(self.max_execution_time)
            .build()
            .map_err(|e| ToolError::ExecutionFailed {
                reason: e.to_string(),
            })?;

        let method = http_input
            .method
            .as_deref()
            .unwrap_or("GET")
            .to_uppercase();

        let mut request = match method.as_str() {
            "POST" => client.post(&http_input.url),
            "PUT" => client.put(&http_input.url),
            "DELETE" => client.delete(&http_input.url),
            "PATCH" => client.patch(&http_input.url),
            _ => client.get(&http_input.url),
        };

        if let Some(headers) = &http_input.headers {
            for (key, value) in headers {
                request = request.header(key.as_str(), value.as_str());
            }
        }

        if let Some(body) = &http_input.body {
            request = request.body(body.clone());
        }

        let response = request
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                reason: e.to_string(),
            })?;

        let status = response.status().as_u16();
        let body_text = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                reason: e.to_string(),
            })?;

        Ok(json!({
            "status": status,
            "body": body_text,
            "success": status >= 200 && status < 300
        }))
    }
}
