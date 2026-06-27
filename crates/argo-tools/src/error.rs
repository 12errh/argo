use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum ToolError {
    #[error("Permission denied: {resource}")]
    PermissionDenied { resource: String },

    #[error("Execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Timeout after {elapsed:?}")]
    Timeout { elapsed: Duration },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Output too large: {size} bytes")]
    OutputTooLarge { size: usize },
}
