use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct ReduceScopeStrategy;

#[async_trait]
impl HealStrategy for ReduceScopeStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::InfiniteLoop { .. } | AgentError::GoalUnachievable { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let reduce_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "reduce_scope")
            .count();
        if reduce_count >= 3 {
            return HealResult::Failed {
                reason: "Max scope reduction attempts (3) exhausted".into(),
            };
        }

        let reason = match &ctx.error {
            AgentError::InfiniteLoop {
                iteration_count, ..
            } => format!("Looped {} times", iteration_count),
            AgentError::GoalUnachievable { reason } => reason.clone(),
            _ => "Unknown".into(),
        };

        HealResult::Success {
            output: format!(
                "Reduced scope to simpler version (attempt {}): original issue was {}",
                reduce_count + 1,
                reason
            ),
        }
    }

    fn name(&self) -> &str {
        "reduce_scope"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn reduces_scope() {
        let ctx = HealContext {
            error: AgentError::InfiniteLoop {
                iteration_count: 20,
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = ReduceScopeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn fails_after_max() {
        let ctx = HealContext {
            error: AgentError::GoalUnachievable {
                reason: "impossible".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![
                "reduce_scope".into(),
                "reduce_scope".into(),
                "reduce_scope".into(),
            ],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = ReduceScopeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[test]
    fn handles_correct_errors() {
        assert!(ReduceScopeStrategy.can_handle(&AgentError::InfiniteLoop {
            iteration_count: 20
        }));
        assert!(
            ReduceScopeStrategy.can_handle(&AgentError::GoalUnachievable { reason: "x".into() })
        );
        assert!(!ReduceScopeStrategy.can_handle(&AgentError::ToolTimeout {
            name: "x".into(),
            elapsed: std::time::Duration::from_secs(5)
        }));
    }
}
