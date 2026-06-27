use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct SwapToolStrategy;

#[async_trait]
impl HealStrategy for SwapToolStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::ToolNotFound { .. }
                | AgentError::ToolExecutionFailed { .. }
                | AgentError::ToolTimeout { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let swap_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "swap_tool")
            .count();
        if swap_count >= 3 {
            return HealResult::Failed {
                reason: "Max swap attempts (3) exhausted".into(),
            };
        }

        let tool_name = match &ctx.error {
            AgentError::ToolNotFound { name } => name.clone(),
            AgentError::ToolExecutionFailed { name, .. } => name.clone(),
            AgentError::ToolTimeout { name, .. } => name.clone(),
            _ => {
                return HealResult::Failed {
                    reason: "Cannot swap: error type not related to a specific tool".into(),
                }
            }
        };

        let fallbacks = [
            ("bash", "python"),
            ("files", "bash"),
            ("http", "python"),
            ("web_search", "browser"),
        ];

        let new_tool = fallbacks
            .iter()
            .find(|(primary, _)| *primary == tool_name.as_str())
            .map(|(_, fallback)| fallback.to_string())
            .unwrap_or_else(|| format!("{}_alt", tool_name));

        HealResult::Success {
            output: format!(
                "Swapped tool '{}' → '{}' (attempt {})",
                tool_name,
                new_tool,
                swap_count + 1
            ),
        }
    }

    fn name(&self) -> &str {
        "swap_tool"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn swaps_to_fallback() {
        let ctx = HealContext {
            error: AgentError::ToolNotFound {
                name: "bash".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec![],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = SwapToolStrategy.apply(&ctx).await;
        match result {
            HealResult::Success { output } => assert!(output.contains("python")),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn fails_after_max_swaps() {
        let ctx = HealContext {
            error: AgentError::ToolNotFound {
                name: "bash".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: vec!["swap_tool".into(), "swap_tool".into(), "swap_tool".into()],
            previous_errors: vec![],
            current_plan: None,
        };
        let result = SwapToolStrategy.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[test]
    fn handles_tool_errors() {
        assert!(SwapToolStrategy.can_handle(&AgentError::ToolNotFound { name: "x".into() }));
        assert!(!SwapToolStrategy.can_handle(&AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "x".into()
        }));
    }
}
