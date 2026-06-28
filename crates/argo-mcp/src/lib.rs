//! Argo MCP — MCP protocol connector for Argo agents.

pub mod auth;
pub mod connector;
pub mod error;
pub mod types;

pub use connector::{McpConnector, McpToolWrapper};
pub use error::McpError;
pub use types::{McpTool, McpToolCall, McpToolResult};
