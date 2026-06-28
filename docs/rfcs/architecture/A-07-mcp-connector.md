# A-07: MCP Connector

**Status:** Implemented
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define how Argo implements the MCP client protocol, tool discovery, authentication, format conversion, and reconnection handling.

## Motivation

MCP (Model Context Protocol) enables agents to connect to external tool servers and use their tools as if they were native. Argo must implement the full MCP client protocol to integrate with the growing ecosystem of MCP servers.

## Detailed Design

### MCP Client Protocol

```rust
#[async_trait]
pub trait McpClient: Send + Sync {
    async fn connect(&self, url: &str, auth: Option<AuthConfig>) -> Result<(), McpError>;
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError>;
    async fn disconnect(&self) -> Result<(), McpError>;
    fn is_connected(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Bearer { token: String },
    OAuth2 { client_id: String, client_secret: String },
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum McpError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("Tool execution failed: {reason}")]
    ToolExecutionFailed { reason: String },

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },

    #[error("Server disconnected")]
    Disconnected,
}
```

### SSE Transport

```rust
pub struct SseMcpClient {
    client: reqwest::Client,
    endpoint: String,
    session_id: Option<String>,
    connected: bool,
}

#[async_trait]
impl McpClient for SseMcpClient {
    async fn connect(&self, url: &str, auth: Option<AuthConfig>) -> Result<(), McpError> {
        // 1. Send initialize request
        // 2. Receive server capabilities
        // 3. Store session ID from headers
        // 4. Start SSE listener for server messages
        todo!()
    }

    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        // Send tools/list request, parse response
        todo!()
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        // Send tools/call request, wait for response, parse result
        todo!()
    }
}
```

### Tool Registration

```rust
pub async fn register_mcp_tools(
    registry: &mut ToolRegistry,
    client: &dyn McpClient,
    server_name: &str,
) -> Result<(), McpError> {
    let tools = client.list_tools().await?;

    for tool in tools {
        let mcp_tool = McpToolAdapter {
            client: client.clone(),
            server_name: server_name.to_string(),
            tool: tool.clone(),
        };
        registry.register(Arc::new(mcp_tool));
    }

    Ok(())
}

struct McpToolAdapter {
    client: Arc<dyn McpClient>,
    server_name: String,
    tool: McpTool,
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> &str { &self.tool.name }
    fn description(&self) -> &str { &self.tool.description }
    fn input_schema(&self) -> Value { self.tool.input_schema.clone() }
    fn output_schema(&self) -> Value { serde_json::json!({ "type": "object" }) }

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
        self.client.call_tool(&self.tool.name, input).await
            .map_err(|e| ToolError::ExecutionFailed { reason: e.to_string() })
    }
}
```

### Reconnection

```rust
pub struct ResilientMcpClient {
    inner: SseMcpClient,
    max_reconnect_attempts: u32,
    reconnect_delay: Duration,
}

impl ResilientMcpClient {
    pub async fn ensure_connected(&self) -> Result<(), McpError> {
        if self.inner.is_connected() {
            return Ok(());
        }

        for attempt in 0..self.max_reconnect_attempts {
            match self.inner.connect(&self.inner.endpoint, self.inner.auth.clone()).await {
                Ok(()) => return Ok(()),
                Err(_) => {
                    let delay = self.reconnect_delay * 2u32.pow(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(McpError::ConnectionFailed { reason: "Max reconnect attempts exceeded".into() })
    }
}
```

## Alternatives Considered

1. **stdio transport**: Simpler, but requires spawning a subprocess for each MCP server.
2. **HTTP transport**: More standard, but MCP specification uses SSE.
3. **WebSocket transport**: More efficient, but not part of MCP specification.

## Drawbacks

- SSE requires maintaining a long-lived connection
- Reconnection logic adds complexity
- MCP server availability is external dependency

## Unresolved Questions

- Should MCP tools be namespaced (e.g., `asana.create_task`) or use bare names?
- How to handle MCP server rate limits?
- Should we support MCP resources (not just tools)?
