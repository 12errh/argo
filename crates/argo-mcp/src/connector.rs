use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::auth::{AuthManager, McpAuth};
use crate::error::McpError;
use crate::types::{
    CallToolParams, ClientInfo, InitializeParams, JsonRpcRequest, JsonRpcResponse, McpTool,
    McpToolResult,
};
use argo_tools::error::ToolError;
use argo_tools::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct McpConnector {
    server_url: String,
    http_client: Client,
    auth: Arc<Mutex<AuthManager>>,
    request_id: AtomicU64,
    connected: Arc<Mutex<bool>>,
    tools_cache: Arc<Mutex<Vec<McpTool>>>,
    reconnect_delay: Duration,
    max_reconnect_delay: Duration,
}

impl McpConnector {
    pub fn new(server_url: &str, auth: McpAuth) -> Self {
        Self {
            server_url: server_url.to_string(),
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            auth: Arc::new(Mutex::new(AuthManager::new(auth))),
            request_id: AtomicU64::new(1),
            connected: Arc::new(Mutex::new(false)),
            tools_cache: Arc::new(Mutex::new(Vec::new())),
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(60),
        }
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, McpError> {
        let id = self.next_request_id();
        let request = JsonRpcRequest::new(id, method, params);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());

        {
            let mut auth = self.auth.lock().await;
            if let Some(auth_header) =
                auth.get_auth_header()
                    .await
                    .map_err(|e| McpError::ConnectionFailed {
                        server: self.server_url.clone(),
                        reason: e.to_string(),
                    })?
            {
                headers.insert(
                    "Authorization",
                    auth_header.parse().map_err(|_| McpError::ProtocolError {
                        reason: "Invalid auth header".to_string(),
                    })?,
                );
            }
        }

        let body = serde_json::to_string(&request).map_err(|e| McpError::SerializationError {
            reason: e.to_string(),
        })?;

        let response = self
            .http_client
            .post(&self.server_url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| McpError::ConnectionFailed {
                server: self.server_url.clone(),
                reason: e.to_string(),
            })?;

        let response_body: JsonRpcResponse =
            response.json().await.map_err(|e| McpError::ProtocolError {
                reason: format!("Failed to parse response: {}", e),
            })?;

        if let Some(err) = response_body.error {
            return Err(McpError::ProtocolError {
                reason: format!("JSON-RPC error {}: {}", err.code, err.message),
            });
        }

        response_body.result.ok_or_else(|| McpError::ProtocolError {
            reason: "No result in response".to_string(),
        })
    }

    pub async fn connect(&self) -> Result<(), McpError> {
        info!("Connecting to MCP server: {}", self.server_url);

        let params = serde_json::to_value(&InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({
                "tools": { "listChanged": true }
            }),
            client_info: ClientInfo {
                name: "argo-agent".to_string(),
                version: "0.1.0".to_string(),
            },
        })
        .map_err(|e| McpError::SerializationError {
            reason: e.to_string(),
        })?;

        self.send_request("initialize", Some(params)).await?;

        self.send_request("notifications/initialized", None).await?;

        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }

        info!("Connected to MCP server: {}", self.server_url);
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let result = self.send_request("tools/list", None).await?;

        let tools: Vec<McpTool> =
            serde_json::from_value(result).map_err(|e| McpError::SerializationError {
                reason: e.to_string(),
            })?;

        let mut cache = self.tools_cache.lock().await;
        *cache = tools.clone();

        info!("Discovered {} tools from MCP server", tools.len());
        Ok(tools)
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult, McpError> {
        let params = serde_json::to_value(&CallToolParams {
            name: tool_name.to_string(),
            arguments: Some(arguments),
        })
        .map_err(|e| McpError::SerializationError {
            reason: e.to_string(),
        })?;

        let result = self.send_request("tools/call", Some(params)).await?;

        serde_json::from_value(result).map_err(|e| McpError::SerializationError {
            reason: e.to_string(),
        })
    }

    pub async fn reconnect(&self) -> Result<(), McpError> {
        let mut delay = self.reconnect_delay;
        let max_attempts = 10;

        for attempt in 1..=max_attempts {
            info!(
                "Reconnecting to MCP server (attempt {}/{}), delay {:?}",
                attempt, max_attempts, delay
            );

            tokio::time::sleep(delay).await;

            match self.connect().await {
                Ok(()) => {
                    info!("Reconnected to MCP server successfully");
                    return Ok(());
                }
                Err(e) => {
                    warn!("Reconnect attempt {} failed: {}", attempt, e);
                    delay = std::cmp::min(delay * 2, self.max_reconnect_delay);
                }
            }
        }

        Err(McpError::ConnectionFailed {
            server: self.server_url.clone(),
            reason: format!("Failed to reconnect after {} attempts", max_attempts),
        })
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    pub async fn get_cached_tools(&self) -> Vec<McpTool> {
        self.tools_cache.lock().await.clone()
    }
}

pub struct McpToolWrapper {
    connector: Arc<McpConnector>,
    tool: McpTool,
    #[allow(dead_code)]
    server_name: String,
}

impl McpToolWrapper {
    pub fn new(connector: Arc<McpConnector>, tool: McpTool, server_name: &str) -> Self {
        Self {
            connector,
            tool,
            server_name: server_name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for McpToolWrapper {
    fn name(&self) -> &str {
        &self.tool.name
    }

    fn description(&self) -> &str {
        &self.tool.description
    }

    fn input_schema(&self) -> serde_json::Value {
        self.tool.input_schema.clone()
    }

    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": { "type": "array" }
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

    async fn execute(
        &self,
        input: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let result = self
            .connector
            .call_tool(&self.tool.name, input)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                reason: format!("MCP tool call failed: {}", e),
            })?;

        if result.is_error {
            let text = result
                .content
                .iter()
                .filter_map(|c| match c {
                    crate::types::McpContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(", ");

            return Err(ToolError::ExecutionFailed { reason: text });
        }

        let output: Vec<serde_json::Value> = result
            .content
            .iter()
            .map(|c| match c {
                crate::types::McpContent::Text { text } => {
                    serde_json::json!({"type": "text", "text": text})
                }
                crate::types::McpContent::Image { data, mime_type } => {
                    serde_json::json!({"type": "image", "data": data, "mime_type": mime_type})
                }
                crate::types::McpContent::Resource { uri, text } => {
                    serde_json::json!({"type": "resource", "uri": uri, "text": text})
                }
            })
            .collect();

        Ok(serde_json::json!({"content": output}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_connector_creation() {
        let connector = McpConnector::new(
            "https://example.com/mcp",
            McpAuth::Bearer {
                token: "test".to_string(),
            },
        );
        assert_eq!(connector.server_url, "https://example.com/mcp");
        assert_eq!(connector.reconnect_delay, Duration::from_secs(1));
        assert_eq!(connector.max_reconnect_delay, Duration::from_secs(60));
    }

    #[test]
    fn test_json_rpc_request() {
        let req = JsonRpcRequest::new(1, "tools/list", None);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "tools/list");
        assert!(req.params.is_none());
    }

    #[test]
    fn test_mcp_tool_wrapper_name() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({}),
        };
        let connector = Arc::new(McpConnector::new("http://localhost", McpAuth::None));
        let wrapper = McpToolWrapper::new(connector, tool, "test_server");
        assert_eq!(wrapper.name(), "test_tool");
    }
}
