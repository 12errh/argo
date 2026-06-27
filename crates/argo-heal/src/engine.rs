use crate::strategy::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct HealEngine {
    strategies: Vec<Box<dyn HealStrategy>>,
    pub max_attempts: usize,
}

impl HealEngine {
    pub fn new(strategies: Vec<Box<dyn HealStrategy>>) -> Self {
        Self {
            strategies,
            max_attempts: 20,
        }
    }

    pub fn with_max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    pub async fn heal(&self, ctx: &HealContext) -> HealResult {
        let ordered = self.select_strategies_for_error(&ctx.error);

        for strategy in &ordered {
            if !strategy.can_handle(&ctx.error) {
                continue;
            }

            let result = strategy.apply(ctx).await;

            match result {
                HealResult::Success { output } => {
                    tracing::info!(
                        strategy = strategy.name(),
                        output = %output,
                        "Heal strategy succeeded"
                    );
                    return HealResult::Success { output };
                }
                HealResult::Failed { reason } => {
                    tracing::debug!(
                        strategy = strategy.name(),
                        reason = %reason,
                        "Heal strategy failed, trying next"
                    );
                }
            }
        }

        HealResult::Failed {
            reason: format!(
                "All {} strategies exhausted for error: {}",
                ordered.len(),
                ctx.error
            ),
        }
    }

    fn select_strategies_for_error(&self, error: &AgentError) -> Vec<&dyn HealStrategy> {
        let initial = crate::ErrorClassifier::initial_strategy(error);

        let mut ordered: Vec<&dyn HealStrategy> = self
            .strategies
            .iter()
            .filter(|s| s.name() == initial)
            .map(|s| s.as_ref())
            .collect();

        for strategy in &self.strategies {
            if strategy.name() != initial && strategy.can_handle(error) {
                ordered.push(strategy.as_ref());
            }
        }

        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    struct MockStrategy {
        name: &'static str,
        can_handle_result: bool,
        apply_result: HealResult,
    }

    #[async_trait::async_trait]
    impl HealStrategy for MockStrategy {
        fn can_handle(&self, _error: &AgentError) -> bool {
            self.can_handle_result
        }

        async fn apply(&self, _ctx: &HealContext) -> HealResult {
            self.apply_result.clone()
        }

        fn name(&self) -> &str {
            self.name
        }
    }

    fn make_ctx(error: AgentError) -> HealContext {
        HealContext {
            error,
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        }
    }

    #[tokio::test]
    async fn stops_at_first_success() {
        let engine = HealEngine::new(vec![
            Box::new(MockStrategy {
                name: "retry",
                can_handle_result: true,
                apply_result: HealResult::Success {
                    output: "ok".into(),
                },
            }),
            Box::new(MockStrategy {
                name: "reframe",
                can_handle_result: true,
                apply_result: HealResult::Success {
                    output: "should not reach".into(),
                },
            }),
        ]);

        let ctx = make_ctx(AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "test".into(),
        });

        let result = engine.heal(&ctx).await;
        match result {
            HealResult::Success { output } => assert_eq!(output, "ok"),
            _ => panic!("Expected first strategy to succeed"),
        }
    }

    #[tokio::test]
    async fn exhausts_all_strategies() {
        let engine = HealEngine::new(vec![
            Box::new(MockStrategy {
                name: "retry",
                can_handle_result: true,
                apply_result: HealResult::Failed {
                    reason: "failed".into(),
                },
            }),
            Box::new(MockStrategy {
                name: "reframe",
                can_handle_result: true,
                apply_result: HealResult::Failed {
                    reason: "failed".into(),
                },
            }),
        ]);

        let ctx = make_ctx(AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "test".into(),
        });

        let result = engine.heal(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[tokio::test]
    async fn skips_non_matching_strategies() {
        let engine = HealEngine::new(vec![
            Box::new(MockStrategy {
                name: "swap_tool",
                can_handle_result: false,
                apply_result: HealResult::Failed {
                    reason: "should not be called".into(),
                },
            }),
            Box::new(MockStrategy {
                name: "retry",
                can_handle_result: true,
                apply_result: HealResult::Success {
                    output: "ok".into(),
                },
            }),
        ]);

        let ctx = make_ctx(AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "test".into(),
        });

        let result = engine.heal(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn tries_fallback_after_initial_failure() {
        let engine = HealEngine::new(vec![
            Box::new(MockStrategy {
                name: "retry",
                can_handle_result: true,
                apply_result: HealResult::Failed {
                    reason: "exhausted".into(),
                },
            }),
            Box::new(MockStrategy {
                name: "reframe",
                can_handle_result: true,
                apply_result: HealResult::Success {
                    output: "reframed".into(),
                },
            }),
        ]);

        let ctx = make_ctx(AgentError::LlmHallucination {
            evidence: "bad".into(),
            confidence: 0.8,
        });

        let result = engine.heal(&ctx).await;
        match result {
            HealResult::Success { output } => assert_eq!(output, "reframed"),
            _ => panic!("Expected reframe to succeed"),
        }
    }
}
