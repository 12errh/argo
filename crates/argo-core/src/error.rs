use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum AgentError {
    #[error("LLM rate limited, retry after {retry_after:?}")]
    LlmRateLimit {
        retry_after: Duration,
        provider: String,
    },

    #[error("LLM context overflow: {current} tokens exceeds limit {limit}")]
    LlmContextOverflow { current: usize, limit: usize },

    #[error("LLM hallucination detected: {evidence}")]
    LlmHallucination { evidence: String, confidence: f32 },

    #[error("LLM refused: {reason}")]
    LlmRefusal { reason: String, provider: String },

    #[error("LLM timeout after {elapsed:?}")]
    LlmTimeout { elapsed: Duration, provider: String },

    #[error("LLM provider {provider} is down: {reason}")]
    LlmProviderDown { provider: String, reason: String },

    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("Tool {name} failed: {reason}")]
    ToolExecutionFailed {
        name: String,
        reason: String,
        exit_code: Option<i32>,
    },

    #[error("Tool {name} timed out after {elapsed:?}")]
    ToolTimeout { name: String, elapsed: Duration },

    #[error("Tool {name} permission denied for {resource}")]
    ToolPermissionDenied { name: String, resource: String },

    #[error("Infinite loop after {iteration_count} iterations")]
    InfiniteLoop { iteration_count: usize },

    #[error("Goal unachievable: {reason}")]
    GoalUnachievable { reason: String },

    #[error("Memory store {store} unavailable: {reason}")]
    MemoryUnavailable { store: String, reason: String },

    #[error("MCP connection to {server} failed: {reason}")]
    McpConnectionFailed { server: String, reason: String },

    #[error("Network timeout for {url} after {elapsed:?}")]
    NetworkTimeout { url: String, elapsed: Duration },

    #[error("Sub-agent {agent_id} failed: {error}")]
    SubAgentFailed {
        agent_id: String,
        error: Box<AgentError>,
    },

    #[error("Orchestrator failed: {reason}")]
    OrchestratorFailed { reason: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Context corrupted")]
    ContextCorrupted,
}

#[derive(Debug, Clone, Error, serde::Serialize, serde::Deserialize)]
pub enum LlmError {
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Context overflow: {current} tokens, limit {limit}")]
    ContextOverflow { current: usize, limit: usize },

    #[error("Auth failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },

    #[error("Request timed out after {elapsed_ms}ms")]
    Timeout { elapsed_ms: u64 },

    #[error("Provider error: {status} {message}")]
    ProviderError { status: u16, message: String },

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("Invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("Streaming error: {reason}")]
    StreamingError { reason: String },
}

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

impl From<LlmError> for AgentError {
    fn from(e: LlmError) -> Self {
        match e {
            LlmError::RateLimited { retry_after_ms } => AgentError::LlmRateLimit {
                retry_after: Duration::from_millis(retry_after_ms),
                provider: String::new(),
            },
            LlmError::ContextOverflow { current, limit } => {
                AgentError::LlmContextOverflow { current, limit }
            }
            LlmError::AuthenticationFailed { reason } => AgentError::LlmProviderDown {
                provider: String::new(),
                reason,
            },
            LlmError::Timeout { elapsed_ms } => AgentError::LlmTimeout {
                elapsed: Duration::from_millis(elapsed_ms),
                provider: String::new(),
            },
            LlmError::NetworkError { reason: _ } => AgentError::NetworkTimeout {
                url: String::new(),
                elapsed: Duration::from_secs(0),
            },
            _ => AgentError::LlmProviderDown {
                provider: String::new(),
                reason: e.to_string(),
            },
        }
    }
}
