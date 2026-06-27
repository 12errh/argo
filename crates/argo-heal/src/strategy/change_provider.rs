use async_trait::async_trait;

use super::HealStrategy;
use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub struct ChangeProviderStrategy {
    pub providers: Vec<String>,
}

#[async_trait]
impl HealStrategy for ChangeProviderStrategy {
    fn can_handle(&self, error: &AgentError) -> bool {
        matches!(
            error,
            AgentError::LlmProviderDown { .. }
                | AgentError::LlmRateLimit { .. }
                | AgentError::LlmRefusal { .. }
        )
    }

    async fn apply(&self, ctx: &HealContext) -> HealResult {
        let change_count = ctx
            .past_strategies
            .iter()
            .filter(|s| *s == "change_provider")
            .count();

        let current_provider = match &ctx.error {
            AgentError::LlmProviderDown { provider, .. } => provider.clone(),
            AgentError::LlmRateLimit { provider, .. } => provider.clone(),
            AgentError::LlmRefusal { provider, .. } => provider.clone(),
            _ => "unknown".into(),
        };

        let default_providers = vec![
            "anthropic".into(),
            "openai".into(),
            "gemini".into(),
            "ollama".into(),
        ];
        let providers = if self.providers.is_empty() {
            &default_providers
        } else {
            &self.providers
        };

        if change_count >= providers.len() {
            return HealResult::Failed {
                reason: format!("All {} providers exhausted", providers.len()),
            };
        }

        let new_provider = providers
            .iter()
            .find(|p| *p != &current_provider)
            .cloned()
            .unwrap_or_else(|| "fallback".into());

        HealResult::Success {
            output: format!(
                "Changed LLM provider '{}' -> '{}' (attempt {})",
                current_provider,
                new_provider,
                change_count + 1
            ),
        }
    }

    fn name(&self) -> &str {
        "change_provider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_ctx(provider: &str, past: Vec<&str>) -> HealContext {
        HealContext {
            error: AgentError::LlmProviderDown {
                provider: provider.into(),
                reason: "down".into(),
            },
            agent_id: "a".into(),
            run_id: "r".into(),
            task_id: Uuid::new_v4(),
            iteration: 0,
            past_strategies: past.into_iter().map(String::from).collect(),
            previous_errors: vec![],
            current_plan: None,
        }
    }

    #[tokio::test]
    async fn switches_provider() {
        let s = ChangeProviderStrategy {
            providers: vec!["anthropic".into(), "openai".into()],
        };
        let ctx = make_ctx("anthropic", vec![]);
        let result = s.apply(&ctx).await;
        match result {
            HealResult::Success { output } => assert!(output.contains("openai")),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn fails_after_all_providers() {
        let s = ChangeProviderStrategy {
            providers: vec!["anthropic".into(), "openai".into()],
        };
        let ctx = make_ctx("ollama", vec!["change_provider", "change_provider"]);
        let result = s.apply(&ctx).await;
        assert!(matches!(result, HealResult::Failed { .. }));
    }

    #[test]
    fn handles_provider_errors() {
        let s = ChangeProviderStrategy { providers: vec![] };
        assert!(s.can_handle(&AgentError::LlmProviderDown {
            provider: "x".into(),
            reason: "down".into()
        }));
        assert!(s.can_handle(&AgentError::LlmRateLimit {
            retry_after: std::time::Duration::from_secs(1),
            provider: "x".into()
        }));
        assert!(!s.can_handle(&AgentError::ToolTimeout {
            name: "x".into(),
            elapsed: std::time::Duration::from_secs(5)
        }));
    }
}
