use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct RetryStrategy {
    pub max_retries: usize,
    pub base_delay_ms: u64,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 1000,
        }
    }
}

#[async_trait]
impl HealStrategy for RetryStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::LlmRateLimit { .. }
                | AgentError::LlmContextOverflow { .. }
                | AgentError::LlmTimeout { .. }
                | AgentError::ToolExecutionFailed { .. }
                | AgentError::ToolTimeout { .. }
                | AgentError::McpConnectionFailed { .. }
                | AgentError::NetworkTimeout { .. }
                | AgentError::MemoryUnavailable { .. }
                | AgentError::LlmProviderDown { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let retry_count = ctx.past_strategies.iter().filter(|s| *s == "retry").count();
        if retry_count >= self.max_retries {
            return HealResult::Failed {
                reason: format!("Max retries ({}) exhausted", self.max_retries),
            };
        }
        let delay = self.base_delay_ms * 2u64.pow(retry_count as u32);
        let jitter = (rand::random::<f64>() * delay as f64 * 0.1) as u64;
        let actual_delay = delay + jitter;
        tokio::time::sleep(tokio::time::Duration::from_millis(actual_delay)).await;
        HealResult::Success {
            output: format!(
                "Retried after {}ms (attempt {})",
                actual_delay,
                retry_count + 1
            ),
        }
    }

    fn name(&self) -> &str {
        "retry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_ctx(strategy: &str) -> HealContext {
        HealContext {
            error: AgentError::LlmRateLimit {
                retry_after: std::time::Duration::from_secs(1),
                provider: "test".into(),
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
    async fn retries_up_to_max() {
        let s = RetryStrategy {
            max_retries: 2,
            base_delay_ms: 10,
        };
        let ctx = make_ctx("retry");
        let result = s.apply(&ctx).await;
        assert!(matches!(result, HealResult::Success { .. }));
    }

    #[tokio::test]
    async fn fails_after_max_retries() {
        let s = RetryStrategy {
            max_retries: 2,
            base_delay_ms: 10,
        };
        let ctx = HealContext {
            error: AgentError::LlmRateLimit {
                retry_after: std::time::Duration::from_secs(1),
                provider: "test".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec!["retry".into(), "retry".into()],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = s.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[test]
    fn handles_correct_error_types() {
        let s = RetryStrategy::default();
        assert!(s.can_handle(&AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "test".into()
        }));
        assert!(s.can_handle(&AgentError::ToolTimeout {
            name: "x".into(),
            elapsed: std::time::Duration::from_secs(5)
        }));
        assert!(!s.can_handle(&AgentError::ToolPermissionDenied {
            name: "x".into(),
            resource: "/etc".into()
        }));
    }
}
