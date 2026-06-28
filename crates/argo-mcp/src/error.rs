use thiserror::Error;

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum McpError {
    #[error("Connection failed to {server}: {reason}")]
    ConnectionFailed { server: String, reason: String },

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Tool not found on server: {tool_name}")]
    ToolNotFound { tool_name: String },

    #[error("Tool invocation failed: {reason}")]
    ToolInvocationFailed { reason: String },

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Server disconnected")]
    Disconnected,

    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("OAuth2 error: {reason}")]
    OAuth2Error { reason: String },
}
