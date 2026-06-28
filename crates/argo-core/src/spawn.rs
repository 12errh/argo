use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::execution::execute_task;
use crate::llm::LlmProvider;
use argo_memory::handle::MemoryHandle;
use argo_tools::registry::ToolRegistry;

pub struct AgentSpawner {
    llm_providers: Vec<Arc<dyn LlmProvider>>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryHandle>,
}

impl AgentSpawner {
    pub fn new(
        llm_providers: Vec<Arc<dyn LlmProvider>>,
        tools: Arc<ToolRegistry>,
        memory: Arc<MemoryHandle>,
    ) -> Self {
        Self {
            llm_providers,
            tools,
            memory,
        }
    }

    pub async fn spawn_and_run(
        &self,
        parent_id: &str,
        goal: &str,
        config_override: Option<&AgentConfig>,
    ) -> Result<String, AgentError> {
        let child_id = Uuid::new_v4().to_string();

        info!(
            "Spawning child agent {} from parent {} for goal: {}",
            child_id, parent_id, goal
        );

        let config = config_override
            .cloned()
            .unwrap_or_else(|| self.default_child_config(&child_id));

        let llm = self
            .llm_providers
            .first()
            .ok_or_else(|| AgentError::OrchestratorFailed {
                reason: "No LLM providers available for child agent".to_string(),
            })?;

        let result = execute_task(
            goal,
            llm.as_ref(),
            self.tools.as_ref(),
            self.memory.as_ref(),
            &config,
        )
        .await?;

        match result {
            crate::message::TaskResult::Success { output } => Ok(output),
            crate::message::TaskResult::Partial { output, reason } => {
                Ok(format!("[partial: {}] {}", reason, output))
            }
            crate::message::TaskResult::Failed { error } => Err(AgentError::SubAgentFailed {
                agent_id: child_id,
                error: Box::new(error),
            }),
        }
    }

    fn default_child_config(&self, child_id: &str) -> AgentConfig {
        AgentConfig {
            agent: crate::config::AgentSection {
                name: child_id.to_string(),
                version: Some("0.1.0".to_string()),
                description: Some("Spawned child agent".to_string()),
            },
            model: crate::config::ModelSection {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                api_key: None,
                base_url: None,
                temperature: Some(0.2),
                max_tokens: Some(4096),
                context_strategy: None,
            },
            memory: None,
            heal: None,
            quality: None,
            tools: crate::config::ToolsSection {
                enabled: vec!["bash".to_string(), "files".to_string()],
            },
            permissions: crate::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: None,
            },
            observe: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_child_config_fields() {
        let config = AgentConfig {
            agent: crate::config::AgentSection {
                name: "test-child".to_string(),
                version: Some("0.1.0".to_string()),
                description: Some("Spawned child agent".to_string()),
            },
            model: crate::config::ModelSection {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                api_key: None,
                base_url: None,
                temperature: Some(0.2),
                max_tokens: Some(4096),
                context_strategy: None,
            },
            memory: None,
            heal: None,
            quality: None,
            tools: crate::config::ToolsSection {
                enabled: vec!["bash".to_string(), "files".to_string()],
            },
            permissions: crate::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: None,
            },
            observe: None,
        };
        assert_eq!(config.agent.name, "test-child");
        assert_eq!(config.model.provider, "anthropic");
    }
}
