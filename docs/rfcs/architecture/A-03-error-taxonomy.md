# A-03: Error Taxonomy

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Complete classification of every error type in Argo, with metadata requirements and classification rules for the self-healing system.

## Motivation

The self-healing system needs a structured error taxonomy to select appropriate recovery strategies. Every error must be classified into a known type with enough metadata for the heal engine to act on it.

## Detailed Design

### Error Enum

```rust
use thiserror::Error;
use std::time::Duration;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum AgentError {
    // === LLM Errors ===

    #[error("Rate limited by LLM provider, retry after {retry_after:?}")]
    LlmRateLimit {
        retry_after: Duration,
        provider: String,
    },

    #[error("LLM context overflow: {current} tokens exceeds limit {limit}")]
    LlmContextOverflow {
        current: usize,
        limit: usize,
    },

    #[error("LLM hallucination detected: {evidence}")]
    LlmHallucination {
        evidence: String,
        confidence: f32,
    },

    #[error("LLM refused to complete: {reason}")]
    LlmRefusal {
        reason: String,
        provider: String,
    },

    #[error("LLM timeout after {elapsed:?}")]
    LlmTimeout {
        elapsed: Duration,
        provider: String,
    },

    #[error("LLM provider {provider} is down: {reason}")]
    LlmProviderDown {
        provider: String,
        reason: String,
    },

    // === Tool Errors ===

    #[error("Tool not found: {name}")]
    ToolNotFound {
        name: String,
    },

    #[error("Tool {name} execution failed: {reason}")]
    ToolExecutionFailed {
        name: String,
        reason: String,
        exit_code: Option<i32>,
    },

    #[error("Tool {name} timed out after {elapsed:?}")]
    ToolTimeout {
        name: String,
        elapsed: Duration,
    },

    #[error("Tool {name} permission denied for {resource}")]
    ToolPermissionDenied {
        name: String,
        resource: String,
    },

    #[error("Tool {name} produced invalid output: {output}")]
    ToolOutputInvalid {
        name: String,
        output: String,
    },

    // === Logic Errors ===

    #[error("Infinite loop detected after {iteration_count} iterations")]
    InfiniteLoop {
        iteration_count: usize,
    },

    #[error("Goal appears unachievable: {reason}")]
    GoalUnachievable {
        reason: String,
    },

    #[error("Plan is invalid: {reason}")]
    PlanInvalid {
        plan: String,
        reason: String,
    },

    #[error("Agent context is corrupted")]
    ContextCorrupted,

    // === Infrastructure Errors ===

    #[error("Memory store {store} is unavailable: {reason}")]
    MemoryUnavailable {
        store: MemoryStore,
        reason: String,
    },

    #[error("MCP connection to {server} failed: {reason}")]
    McpConnectionFailed {
        server: String,
        reason: String,
    },

    #[error("Network timeout for {url} after {elapsed:?}")]
    NetworkTimeout {
        url: String,
        elapsed: Duration,
    },

    // === Agent Errors ===

    #[error("Sub-agent {agent_id} failed: {error}")]
    SubAgentFailed {
        agent_id: String,
        error: Box<AgentError>,
    },

    #[error("Orchestrator failed: {reason}")]
    OrchestratorFailed {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStore {
    Redis,
    SurrealDB,
    Qdrant,
}
```

### Classification Rules

When an error occurs, the classifier:

1. Inspects the error type (Rust enum variant)
2. Extracts metadata fields
3. Classifies severity: `recoverable`, `degradable`, `fatal`
4. Selects initial heal strategy based on error type

| Error Type | Severity | Initial Strategy |
|---|---|---|
| `LlmRateLimit` | Recoverable | Retry with backoff |
| `LlmContextOverflow` | Recoverable | Context overflow handling |
| `LlmHallucination` | Recoverable | Reframe prompt |
| `LlmRefusal` | Degradable | Reframe prompt |
| `LlmTimeout` | Recoverable | Retry with backoff |
| `LlmProviderDown` | Degradable | Change provider |
| `ToolNotFound` | Degradable | Swap tool |
| `ToolExecutionFailed` | Recoverable | Retry, then swap tool |
| `ToolTimeout` | Recoverable | Retry with backoff |
| `ToolPermissionDenied` | Fatal | Report to user |
| `ToolOutputInvalid` | Recoverable | Reframe prompt |
| `InfiniteLoop` | Degradable | Reduce scope |
| `GoalUnachievable` | Fatal | Report to user |
| `PlanInvalid` | Recoverable | Decompose |
| `ContextCorrupted` | Fatal | Report to user |
| `MemoryUnavailable` | Degradable | Retry, then continue without memory |
| `McpConnectionFailed` | Recoverable | Retry with backoff |
| `NetworkTimeout` | Recoverable | Retry with backoff |
| `SubAgentFailed` | Degradable | Spawn new sub-agent |
| `OrchestratorFailed` | Fatal | Report to user |

### Error Metadata

Every error carries:

```rust
pub struct ErrorContext {
    pub error: AgentError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: String,
    pub run_id: String,
    pub task_id: uuid::Uuid,
    pub iteration: usize,
    pub previous_errors: Vec<AgentError>,
    pub current_plan: Option<String>,
}
```

## Alternatives Considered

1. **Flat error codes (E001, E002)**: Simpler, but loses semantic meaning and makes heal strategy selection harder.
2. **String-based errors**: Maximum flexibility, but no type safety, no exhaustive matching.
3. **Error hierarchy (Error → LlmError → RateLimitError)**: More granular, but adds complexity without clear benefit for heal strategy selection.

## Drawbacks

- Large enum with many variants increases compile time slightly
- Adding new error types requires updating heal strategy mappings
- Some errors may be misclassified (e.g., a tool error that's actually an LLM error)

## Unresolved Questions

- Should errors carry a `retry_count` field to track how many times this specific error has been retried?
- How to handle compound errors (e.g., LLM timeout while a tool is running)?
- Should we support custom user-defined error types?
