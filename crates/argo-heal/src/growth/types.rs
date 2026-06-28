use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    PromptUpdate,
    PreCheck,
    StrategyReorder,
    ToolSwap,
    ScopeAdjustment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub id: Uuid,
    pub proposal_type: ProposalType,
    pub risk_level: ProposalRisk,
    pub target: String,
    pub content: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub occurrences: usize,
    pub error_types: Vec<String>,
    pub tool_names: Vec<String>,
    pub time_range_hours: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    RecurringError,
    ToolFailure,
    TaskTypeSuccess,
    StrategyEffectiveness,
    TimeDegradation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthReport {
    pub id: Uuid,
    pub agent_id: String,
    pub cycle_start: DateTime<Utc>,
    pub cycle_end: DateTime<Utc>,
    pub errors_analyzed: usize,
    pub patterns_detected: usize,
    pub proposals_generated: usize,
    pub low_risk_applied: usize,
    pub high_risk_flagged: usize,
    pub patterns: Vec<DetectedPattern>,
    pub proposals: Vec<ImprovementProposal>,
}
