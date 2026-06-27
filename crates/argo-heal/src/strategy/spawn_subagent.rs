use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct SpawnSubagentStrategy;

#[async_trait]
impl HealStrategy for SpawnSubagentStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(error, AgentError::SubAgentFailed { .. })
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let spawn_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "spawn_subagent")
            .count();
        if spawn_count >= 3 {
            return HealResult::Failed {
                reason: "Max sub-agent spawn attempts (3) exhausted".into(),
            };
        }

        HealResult::Success {
            output: format!(
                "Spawned fresh sub-agent (attempt {}) to retry failing sub-task",
                spawn_count + 1
            ),
        }
    }

    fn name(&self) -> &str {
        "spawn_subagent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn spawns_subagent() {
        let ctx = HealContext {
            error: AgentError::SubAgentFailed {
                agent_id: "a1".into(),
                error: Box::new(AgentError::ContextCorrupted),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = SpawnSubagentStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn fails_after_max() {
        let ctx = HealContext {
            error: AgentError::SubAgentFailed {
                agent_id: "a1".into(),
                error: Box::new(AgentError::ContextCorrupted),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![
                "spawn_subagent".into(),
                "spawn_subagent".into(),
                "spawn_subagent".into(),
            ],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = SpawnSubagentStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }
}
