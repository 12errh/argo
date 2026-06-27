use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct DecomposeStrategy;

#[async_trait]
impl HealStrategy for DecomposeStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::GoalUnachievable { .. }
                | AgentError::InfiniteLoop { .. }
                | AgentError::PlanInvalid { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let decompose_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "decompose")
            .count();
        if decompose_count >= 3 {
            return HealResult::Failed {
                reason: "Max decompose attempts (3) exhausted".into(),
            };
        }

        let reason = match &ctx.error {
            AgentError::GoalUnachievable { reason } => reason.clone(),
            AgentError::InfiniteLoop {
                iteration_count, ..
            } => format!("Looped {} times without progress", iteration_count),
            AgentError::PlanInvalid { reason, .. } => reason.clone(),
            _ => "Unknown".into(),
        };

        HealResult::Success {
            output: format!(
                "Decomposed into smaller sub-tasks (attempt {}): {}",
                decompose_count + 1,
                reason
            ),
        }
    }

    fn name(&self) -> &str {
        "decompose"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn decomposes_goal() {
        let ctx = HealContext {
            error: AgentError::GoalUnachievable {
                reason: "too complex".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = DecomposeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn fails_after_max() {
        let ctx = HealContext {
            error: AgentError::PlanInvalid {
                plan: "x".into(),
                reason: "bad".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec!["decompose".into(), "decompose".into(), "decompose".into()],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = DecomposeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }
}
