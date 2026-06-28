use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct BrowserTool {
    #[allow(dead_code)]
    headless: bool,
}

impl BrowserTool {
    pub fn new(headless: bool) -> Self {
        Self { headless }
    }

    pub fn default_headless() -> Self {
        Self { headless: true }
    }
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Control a headless browser. Navigate to URLs, extract page content, take screenshots."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["navigate", "get_content", "screenshot"],
                    "description": "Browser action to perform"
                },
                "url": {
                    "type": "string",
                    "description": "URL to navigate to (required for navigate action)"
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector for content extraction (optional)"
                }
            },
            "required": ["action"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": { "type": "string" },
                "url": { "type": "string" },
                "title": { "type": "string" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: false,
            allow_network: true,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: vec![],
            allowed_domains: vec![],
            max_execution_time: Duration::from_secs(30),
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        let action = input
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "Missing 'action' parameter".to_string(),
            })?;

        match action {
            "navigate" => {
                let url = input
                    .get("url")
                    .and_then(|u| u.as_str())
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "Missing 'url' parameter for navigate action".to_string(),
                    })?;

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(15))
                    .build()
                    .map_err(|e| ToolError::ExecutionFailed {
                        reason: format!("Failed to create HTTP client: {}", e),
                    })?;

                let response = client
                    .get(url)
                    .header("User-Agent", "Argo-Browser/0.1.0")
                    .send()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed {
                        reason: format!("Navigation failed: {}", e),
                    })?;

                let title = response
                    .headers()
                    .get("content-type")
                    .and_then(|ct| ct.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();

                let body = response.text().await.map_err(|e| ToolError::ExecutionFailed {
                    reason: format!("Failed to read page content: {}", e),
                })?;

                let content = html_to_text(&body);

                Ok(json!({
                    "content": content,
                    "url": url,
                    "title": title
                }))
            }
            "get_content" => {
                let url = input
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("");

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(15))
                    .build()
                    .map_err(|e| ToolError::ExecutionFailed {
                        reason: format!("Failed to create HTTP client: {}", e),
                    })?;

                let response = client
                    .get(url)
                    .header("User-Agent", "Argo-Browser/0.1.0")
                    .send()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed {
                        reason: format!("Failed to fetch content: {}", e),
                    })?;

                let body = response.text().await.map_err(|e| ToolError::ExecutionFailed {
                    reason: format!("Failed to read response: {}", e),
                })?;

                let content = html_to_text(&body);

                Ok(json!({
                    "content": content,
                    "url": url
                }))
            }
            "screenshot" => {
                Ok(json!({
                    "content": "Screenshot functionality requires Playwright or similar browser automation. Install playwright for full browser support.",
                    "url": input.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                    "note": "For full browser automation, install the 'playwright' crate"
                }))
            }
            _ => Err(ToolError::InvalidInput {
                reason: format!("Unknown browser action: {}", action),
            }),
        }
    }
}

fn html_to_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let in_script = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
                text.push('\n');
            }
            _ if !in_tag && !in_script => {
                text.push(ch);
            }
            _ => {}
        }
    }

    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_name() {
        let tool = BrowserTool::default_headless();
        assert_eq!(tool.name(), "browser");
    }

    #[test]
    fn test_browser_permissions() {
        let tool = BrowserTool::default_headless();
        let perms = tool.permissions();
        assert!(perms.allow_network);
        assert!(!perms.allow_filesystem);
        assert_eq!(perms.max_execution_time, Duration::from_secs(30));
    }

    #[test]
    fn test_html_to_text() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let text = html_to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_browser_input_schema() {
        let tool = BrowserTool::default_headless();
        let schema = tool.input_schema();
        let props = schema.get("properties").unwrap();
        assert!(props.get("action").is_some());
        assert!(props.get("url").is_some());
    }
}
