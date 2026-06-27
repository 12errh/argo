use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct ReframeStrategy;

#[async_trait]
impl HealStrategy for ReframeStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::LlmHallucination { .. }
                | AgentError::LlmRefusal { .. }
                | AgentError::ToolOutputInvalid { .. }
                | AgentError::PlanInvalid { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let reframe_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "reframe")
            .count();
        if reframe_count >= 3 {
            return HealResult::Failed {
                reason: "Max reframe attempts (3) exhausted".into(),
            };
        }

        let clarification = match &ctx.error {
            AgentError::LlmHallucination { evidence, .. } => {
                format!("Previous output contained hallucination: {}. Please be factual and only state verifiable information.", evidence)
            }
            AgentError::LlmRefusal { reason, .. } => {
                format!("The model refused to complete: {}. Rephrase the request to be more specific and actionable.", reason)
            }
            AgentError::ToolOutputInvalid { output, .. } => {
                format!("Tool produced invalid output: {}. Parse the output carefully and handle edge cases.", output)
            }
            AgentError::PlanInvalid { reason, .. } => {
                format!(
                    "Plan was invalid: {}. Create a simpler, more concrete plan.",
                    reason
                )
            }
            _ => "Rephrase the instruction with clearer constraints and explicit expectations."
                .into(),
        };

        HealResult::Success {
            output: format!(
                "Prompt reframed (attempt {}): {}",
                reframe_count + 1,
                clarification
            ),
        }
    }

    fn name(&self) -> &str {
        "reframe"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_ctx(strategy: &str) -> HealContext {
        HealContext {
            error: AgentError::LlmHallucination {
                evidence: "made up a fact".into(),
                confidence: 0.8,
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![strategy.into()],
            previous_errors: vec![],
            current_plan: None,
        }
    }

    #[tokio::test]
    async fn reframes_prompt() {
        let ctx = make_ctx("retry");
        let result = ReframeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn fails_after_max_reframes() {
        let ctx = HealContext {
            error: AgentError::LlmHallucination {
                evidence: "x".into(),
                confidence: 0.5,
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec!["reframe".into(), "reframe".into(), "reframe".into()],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = ReframeStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[test]
    fn handles_correct_errors() {
        assert!(ReframeStrategy.can_handle(&AgentError::LlmHallucination {
            evidence: "x".into(),
            confidence: 0.5
        }));
        assert!(ReframeStrategy.can_handle(&AgentError::LlmRefusal {
            reason: "x".into(),
            provider: "x".into()
        }));
        assert!(!ReframeStrategy.can_handle(&AgentError::ToolTimeout {
            name: "x".into(),
            elapsed: std::time::Duration::from_secs(5)
        }));
    }
}
