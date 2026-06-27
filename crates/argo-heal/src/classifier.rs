use crate::types::ErrorSeverity;
use argo_core::error::AgentError;

pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn classify(error: &AgentError) -> ErrorSeverity {
        match error {
            AgentError::LlmRateLimit { .. } => ErrorSeverity::Recoverable,
            AgentError::LlmContextOverflow { .. } => ErrorSeverity::Recoverable,
            AgentError::LlmHallucination { .. } => ErrorSeverity::Recoverable,
            AgentError::LlmRefusal { .. } => ErrorSeverity::Degradable,
            AgentError::LlmTimeout { .. } => ErrorSeverity::Recoverable,
            AgentError::LlmProviderDown { .. } => ErrorSeverity::Degradable,
            AgentError::ToolNotFound { .. } => ErrorSeverity::Degradable,
            AgentError::ToolExecutionFailed { .. } => ErrorSeverity::Recoverable,
            AgentError::ToolTimeout { .. } => ErrorSeverity::Recoverable,
            AgentError::ToolPermissionDenied { .. } => ErrorSeverity::Fatal,
            AgentError::ToolOutputInvalid { .. } => ErrorSeverity::Recoverable,
            AgentError::InfiniteLoop { .. } => ErrorSeverity::Degradable,
            AgentError::GoalUnachievable { .. } => ErrorSeverity::Fatal,
            AgentError::PlanInvalid { .. } => ErrorSeverity::Recoverable,
            AgentError::ContextCorrupted => ErrorSeverity::Fatal,
            AgentError::MemoryUnavailable { .. } => ErrorSeverity::Degradable,
            AgentError::McpConnectionFailed { .. } => ErrorSeverity::Recoverable,
            AgentError::NetworkTimeout { .. } => ErrorSeverity::Recoverable,
            AgentError::SubAgentFailed { .. } => ErrorSeverity::Degradable,
            AgentError::OrchestratorFailed { .. } => ErrorSeverity::Fatal,
            AgentError::Config(_) => ErrorSeverity::Fatal,
        }
    }

    pub fn initial_strategy(error: &AgentError) -> &'static str {
        match error {
            AgentError::LlmRateLimit { .. } => "retry",
            AgentError::LlmContextOverflow { .. } => "retry",
            AgentError::LlmHallucination { .. } => "reframe",
            AgentError::LlmRefusal { .. } => "reframe",
            AgentError::LlmTimeout { .. } => "retry",
            AgentError::LlmProviderDown { .. } => "change_provider",
            AgentError::ToolNotFound { .. } => "swap_tool",
            AgentError::ToolExecutionFailed { .. } => "retry",
            AgentError::ToolTimeout { .. } => "retry",
            AgentError::ToolPermissionDenied { .. } => "retry",
            AgentError::ToolOutputInvalid { .. } => "reframe",
            AgentError::InfiniteLoop { .. } => "reduce_scope",
            AgentError::GoalUnachievable { .. } => "reduce_scope",
            AgentError::PlanInvalid { .. } => "decompose",
            AgentError::ContextCorrupted => "retry",
            AgentError::MemoryUnavailable { .. } => "retry",
            AgentError::McpConnectionFailed { .. } => "retry",
            AgentError::NetworkTimeout { .. } => "retry",
            AgentError::SubAgentFailed { .. } => "spawn_subagent",
            AgentError::OrchestratorFailed { .. } => "retry",
            AgentError::Config(_) => "retry",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn classify_recoverable_errors() {
        let cases = vec![
            AgentError::LlmRateLimit {
                retry_after: Duration::from_secs(1),
                provider: "test".into(),
            },
            AgentError::LlmContextOverflow {
                current: 100,
                limit: 50,
            },
            AgentError::ToolExecutionFailed {
                name: "bash".into(),
                reason: "fail".into(),
                exit_code: Some(1),
            },
            AgentError::ToolTimeout {
                name: "bash".into(),
                elapsed: Duration::from_secs(30),
            },
            AgentError::NetworkTimeout {
                url: "x".into(),
                elapsed: Duration::from_secs(10),
            },
        ];
        for error in cases {
            assert_eq!(
                ErrorClassifier::classify(&error),
                ErrorSeverity::Recoverable
            );
        }
    }

    #[test]
    fn classify_degradable_errors() {
        let cases = vec![
            AgentError::LlmRefusal {
                reason: "refused".into(),
                provider: "test".into(),
            },
            AgentError::LlmProviderDown {
                provider: "x".into(),
                reason: "down".into(),
            },
            AgentError::ToolNotFound {
                name: "bash".into(),
            },
            AgentError::InfiniteLoop {
                iteration_count: 20,
            },
            AgentError::MemoryUnavailable {
                store: "redis".into(),
                reason: "down".into(),
            },
        ];
        for error in cases {
            assert_eq!(ErrorClassifier::classify(&error), ErrorSeverity::Degradable);
        }
    }

    #[test]
    fn classify_fatal_errors() {
        let cases = vec![
            AgentError::ToolPermissionDenied {
                name: "bash".into(),
                resource: "/etc".into(),
            },
            AgentError::GoalUnachievable { reason: "x".into() },
            AgentError::ContextCorrupted,
            AgentError::Config("bad".into()),
            AgentError::OrchestratorFailed { reason: "x".into() },
        ];
        for error in cases {
            assert_eq!(ErrorClassifier::classify(&error), ErrorSeverity::Fatal);
        }
    }

    #[test]
    fn initial_strategy_matches_error_type() {
        let error = AgentError::LlmRateLimit {
            retry_after: Duration::from_secs(1),
            provider: "test".into(),
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "retry");

        let error = AgentError::ToolNotFound {
            name: "bash".into(),
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "swap_tool");

        let error = AgentError::LlmProviderDown {
            provider: "x".into(),
            reason: "down".into(),
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "change_provider");

        let error = AgentError::LlmHallucination {
            evidence: "bad".into(),
            confidence: 0.5,
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "reframe");

        let error = AgentError::InfiniteLoop {
            iteration_count: 20,
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "reduce_scope");

        let error = AgentError::PlanInvalid {
            plan: "x".into(),
            reason: "bad".into(),
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "decompose");

        let error = AgentError::SubAgentFailed {
            agent_id: "a".into(),
            error: Box::new(AgentError::ContextCorrupted),
        };
        assert_eq!(ErrorClassifier::initial_strategy(&error), "spawn_subagent");
    }
}
