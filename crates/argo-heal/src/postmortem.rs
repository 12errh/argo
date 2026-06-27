use chrono::Utc;
use uuid::Uuid;

use crate::types::{HealContext, HealStep, Lesson};

pub struct PostMortem;

impl PostMortem {
    pub fn generate_lesson(ctx: &HealContext, heal_step: &HealStep) -> Lesson {
        let error_type = format!("{:?}", ctx.error);
        let error_type_short = error_type
            .split('{')
            .next()
            .unwrap_or(&error_type)
            .trim()
            .to_string();

        let context_summary = format!(
            "Agent '{}' during run '{}' encountered error at iteration {}: {}",
            ctx.agent_id, ctx.run_id, ctx.iteration, ctx.error
        );

        let root_cause = Self::infer_root_cause(ctx);
        let resolution = heal_step
            .output
            .clone()
            .unwrap_or_else(|| "Strategy applied".into());
        let prevention = Self::suggest_prevention(&error_type_short);

        Lesson {
            id: Uuid::new_v4(),
            error_type: error_type_short,
            context_summary,
            root_cause,
            resolution,
            prevention,
            confidence: 0.7,
            created_at: Utc::now(),
        }
    }

    fn infer_root_cause(ctx: &HealContext) -> String {
        match &ctx.error {
            argo_core::error::AgentError::LlmRateLimit { provider, .. } => {
                format!("Rate limited by {} provider", provider)
            }
            argo_core::error::AgentError::ToolExecutionFailed { name, reason, .. } => {
                format!("Tool '{}' failed: {}", name, reason)
            }
            argo_core::error::AgentError::ToolNotFound { name } => {
                format!("Tool '{}' not registered in the tool registry", name)
            }
            argo_core::error::AgentError::ToolTimeout { name, elapsed } => {
                format!("Tool '{}' exceeded timeout of {:?}", name, elapsed)
            }
            argo_core::error::AgentError::LlmContextOverflow { current, limit } => {
                format!(
                    "Context overflow: {} tokens exceeds limit of {}",
                    current, limit
                )
            }
            argo_core::error::AgentError::LlmHallucination { evidence, .. } => {
                format!("LLM produced hallucinated content: {}", evidence)
            }
            argo_core::error::AgentError::NetworkTimeout { url, .. } => {
                format!("Network request to '{}' timed out", url)
            }
            _ => format!("Error: {}", ctx.error),
        }
    }

    fn suggest_prevention(error_type: &str) -> String {
        match error_type {
            "LlmRateLimit" => "Implement exponential backoff and respect retry-after headers",
            "ToolExecutionFailed" => "Validate tool inputs before execution and handle edge cases",
            "ToolNotFound" => "Ensure all required tools are registered before agent execution",
            "ToolTimeout" => "Increase timeout or optimize tool execution for large inputs",
            "LlmContextOverflow" => "Monitor token count and trigger overflow handling proactively",
            "LlmHallucination" => {
                "Add explicit constraints to the system prompt about factual accuracy"
            }
            "LlmRefusal" => "Rephrase the request to be more specific and within policy boundaries",
            "LlmProviderDown" => {
                "Configure fallback providers and implement provider health checks"
            }
            "LlmTimeout" => "Increase timeout or reduce request complexity",
            "ToolPermissionDenied" => "Request appropriate permissions or use alternative tools",
            "ToolOutputInvalid" => "Add output validation and parsing with error recovery",
            "InfiniteLoop" => "Add iteration limits and progress detection to break loops",
            "GoalUnachievable" => "Decompose complex goals into achievable sub-tasks",
            "PlanInvalid" => "Validate plans before execution and include error recovery steps",
            "MemoryUnavailable" => {
                "Implement graceful degradation when memory stores are unavailable"
            }
            "McpConnectionFailed" => "Add reconnection logic with exponential backoff",
            "NetworkTimeout" => "Configure appropriate timeouts and retry policies",
            "SubAgentFailed" => "Add sub-agent health monitoring and automatic restart",
            "OrchestratorFailed" => "Implement orchestrator health checks and failover",
            _ => "Review error logs and adjust configuration accordingly",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HealContext, HealStep};
    use argo_core::error::AgentError;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_ctx_and_step() -> (HealContext, HealStep) {
        let ctx = HealContext {
            error: AgentError::ToolExecutionFailed {
                name: "bash".into(),
                reason: "command not found".into(),
                exit_code: Some(127),
            },
            agent_id: "coder".into(),
            run_id: "run-001".into(),
            task_id: Uuid::new_v4(),
            iteration: 2,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        };

        let step = HealStep {
            id: Uuid::new_v4(),
            error: ctx.error.clone(),
            strategy_name: "swap_tool".into(),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            success: true,
            output: Some("Swapped bash -> python (attempt 1)".into()),
        };

        (ctx, step)
    }

    #[test]
    fn generates_lesson_with_all_fields() {
        let (ctx, step) = make_ctx_and_step();
        let lesson = PostMortem::generate_lesson(&ctx, &step);

        assert!(!lesson.error_type.is_empty());
        assert!(!lesson.context_summary.is_empty());
        assert!(!lesson.root_cause.is_empty());
        assert!(!lesson.resolution.is_empty());
        assert!(!lesson.prevention.is_empty());
        assert!(lesson.confidence > 0.0 && lesson.confidence <= 1.0);
    }

    #[test]
    fn lesson_error_type_matches() {
        let (ctx, step) = make_ctx_and_step();
        let lesson = PostMortem::generate_lesson(&ctx, &step);
        assert!(lesson.error_type.contains("ToolExecutionFailed"));
    }

    #[test]
    fn lesson_resolution_includes_strategy_output() {
        let (ctx, step) = make_ctx_and_step();
        let lesson = PostMortem::generate_lesson(&ctx, &step);
        assert!(lesson.resolution.contains("Swapped bash -> python"));
    }

    #[test]
    fn lesson_root_cause_includes_tool_name() {
        let (ctx, step) = make_ctx_and_step();
        let lesson = PostMortem::generate_lesson(&ctx, &step);
        assert!(lesson.root_cause.contains("bash"));
    }

    #[test]
    fn lesson_prevention_is_nonempty() {
        let (ctx, step) = make_ctx_and_step();
        let lesson = PostMortem::generate_lesson(&ctx, &step);
        assert!(lesson.prevention.len() > 10);
    }
}
