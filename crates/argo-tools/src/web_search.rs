use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct WebSearchTool {
    api_key: Option<String>,
    search_url: String,
}

impl WebSearchTool {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            search_url: "https://api.search.brave.com/res/v1/web/search".to_string(),
        }
    }

    pub fn with_url(api_key: Option<String>, search_url: &str) -> Self {
        Self {
            api_key,
            search_url: search_url.to_string(),
        }
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web and return results. Provides titles, URLs, and snippets from search results."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return (default: 5)",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "results": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "url": { "type": "string" },
                            "snippet": { "type": "string" }
                        }
                    }
                }
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
            max_execution_time: Duration::from_secs(15),
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        let query = input
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "Missing 'query' parameter".to_string(),
            })?;

        let num_results = input
            .get("num_results")
            .and_then(|n| n.as_u64())
            .unwrap_or(5) as usize;

        let client = reqwest::Client::new();
        let mut request = client
            .get(&self.search_url)
            .query(&[("q", query), ("count", &num_results.to_string())]);

        if let Some(ref api_key) = self.api_key {
            request = request.header("X-Subscription-Token", api_key.as_str());
            request = request.header("Accept", "application/json");
        }

        let response = request
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                reason: format!("Search request failed: {}", e),
            })?;

        let body: serde_json::Value = response.json().await.map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to parse search response: {}", e),
        })?;

        let results = body
            .get("web")
            .and_then(|w| w.get("results"))
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .take(num_results)
                    .map(|r| {
                        json!({
                            "title": r.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                            "url": r.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                            "snippet": r.get("description").and_then(|d| d.as_str()).unwrap_or("")
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(json!({"results": results}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_name() {
        let tool = WebSearchTool::new(None);
        assert_eq!(tool.name(), "web_search");
    }

    #[test]
    fn test_web_search_permissions() {
        let tool = WebSearchTool::new(None);
        let perms = tool.permissions();
        assert!(perms.allow_network);
        assert!(!perms.allow_filesystem);
    }

    #[test]
    fn test_web_search_input_schema() {
        let tool = WebSearchTool::new(None);
        let schema = tool.input_schema();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("required").is_some());
    }
}
