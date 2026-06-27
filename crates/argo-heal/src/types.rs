use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use argo_core::error::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Recoverable,
    Degradable,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealStep {
    pub id: Uuid,
    pub error: AgentError,
    pub strategy_name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: Uuid,
    pub error_type: String,
    pub context_summary: String,
    pub root_cause: String,
    pub resolution: String,
    pub prevention: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HealContext {
    pub error: AgentError,
    pub agent_id: String,
    pub run_id: String,
    pub task_id: Uuid,
    pub iteration: usize,
    pub past_strategies: Vec<String>,
    pub previous_errors: Vec<AgentError>,
    pub current_plan: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HealResult {
    Success { output: String },
    Failed { reason: String },
}
