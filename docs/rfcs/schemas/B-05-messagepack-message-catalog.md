# B-05: MessagePack Message Catalog

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Overview

Every message type serialized over the actor bus, with field definitions and versioning.

## Message Format

All messages are serialized with MessagePack (rmp-serde). Each message includes a version header for forward compatibility.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub version: u32,
    pub message_type: String,
    pub payload: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<uuid::Uuid>,
}
```

## Message Types

### ExecuteTask

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTask {
    pub task_id: uuid::Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_mode: MemoryMode,
}
```

### ToolCall

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: uuid::Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub timeout_ms: Option<u64>,
}
```

### ToolResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: uuid::Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

### MemoryRead / MemoryWrite

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRead {
    pub op_id: uuid::Uuid,
    pub store: MemoryStore,
    pub key: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWrite {
    pub op_id: uuid::Uuid,
    pub store: MemoryStore,
    pub key: String,
    pub value: serde_json::Value,
    pub namespace: Option<String>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStore {
    Redis,
    SurrealDB,
    Qdrant,
}
```

### HealRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealRequest {
    pub error: AgentError,
    pub context: HealContext,
    pub max_attempts: Option<usize>,
}
```

### SpawnAgent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgent {
    pub agent_id: uuid::Uuid,
    pub config: AgentConfig,
    pub goal: String,
    pub parent_id: Option<uuid::Uuid>,
    pub memory_mode: MemoryMode,
}
```

### AgentDone / AgentFailed

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDone {
    pub task_id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub run_id: uuid::Uuid,
    pub result: TaskResult,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailed {
    pub task_id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub run_id: uuid::Uuid,
    pub error: AgentError,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success { output: String },
    Partial { output: String, reason: String },
    Failed { error: AgentError },
}
```

### InspectRequest / InspectResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectRequest {
    pub run_id: uuid::Uuid,
    pub include_trace: bool,
    pub include_heal_steps: bool,
    pub include_lessons: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResponse {
    pub trace: Option<AgentTrace>,
    pub heal_steps: Option<Vec<HealStepRecord>>,
    pub lessons: Option<Vec<LessonRecord>>,
}
```

### AssignTask / TaskComplete / TaskFailed

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTask {
    pub task_id: uuid::Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_mode: MemoryMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplete {
    pub task_id: uuid::Uuid,
    pub result: TaskResult,
    pub duration_ms: u64,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailed {
    pub task_id: uuid::Uuid,
    pub error: AgentError,
    pub partial_result: Option<String>,
}
```

## Versioning

Message format version is tracked in the `MessageEnvelope.version` field. When the message format changes:

1. Increment the version number
2. Add backward compatibility for the previous version
3. Deprecate the old version after 2 releases

Current version: `1`
