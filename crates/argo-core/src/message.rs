use actix::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTask {
    pub task_id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRead {
    pub op_id: Uuid,
    pub store: String,
    pub key: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWrite {
    pub op_id: Uuid,
    pub store: String,
    pub key: String,
    pub value: serde_json::Value,
    pub namespace: Option<String>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgent {
    pub agent_id: Uuid,
    pub config: serde_json::Value,
    pub goal: String,
    pub parent_id: Option<Uuid>,
    pub memory_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success { output: String },
    Partial { output: String, reason: String },
    Failed { error: AgentError },
}

impl Message for ExecuteTask {
    type Result = TaskResult;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDone {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub run_id: Uuid,
    pub result: TaskResult,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailed {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub run_id: Uuid,
    pub error: AgentError,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectRequest {
    pub run_id: Uuid,
    pub include_trace: bool,
    pub include_heal_steps: bool,
    pub include_lessons: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub call_id: Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallRecord {
    pub provider: String,
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrace {
    pub run_id: Uuid,
    pub agent_name: String,
    pub goal: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub output: Option<String>,
    pub iterations: usize,
    pub quality_score: Option<f32>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub llm_calls: Vec<LlmCallRecord>,
    pub heal_steps: Vec<serde_json::Value>,
    pub lessons: Vec<serde_json::Value>,
    pub errors: Vec<AgentError>,
}
