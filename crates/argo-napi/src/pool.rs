use napi_derive::napi;
use serde_json::Value as JsonValue;

use crate::agent::Agent;
use crate::util;

#[napi]
pub struct AgentPool {
    configs: Vec<argo_core::config::AgentConfig>,
}

#[napi]
impl AgentPool {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    #[napi]
    pub fn add_agent(&mut self, agent: &Agent) {
        self.configs.push(agent.get_config().clone());
    }

    #[napi]
    pub async fn run(&self, goal: String) -> napi::Result<JsonValue> {
        if self.configs.is_empty() {
            return Err(napi::Error::from_reason("No agents in pool"));
        }

        let config = &self.configs[0];
        let llm =
            util::create_provider(config).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let tool_registry = util::create_tool_registry(&config.tools.enabled, config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let memory = util::create_memory(config)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let result = argo_core::execution::execute_task(
            &goal,
            llm.as_ref(),
            &tool_registry,
            &memory,
            config,
        )
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        match result {
            argo_core::message::TaskResult::Success { output } => {
                Ok(serde_json::json!({"success": true, "output": output}))
            }
            argo_core::message::TaskResult::Partial { output, reason } => {
                Ok(serde_json::json!({"success": false, "output": output, "reason": reason}))
            }
            argo_core::message::TaskResult::Failed { error } => {
                Ok(serde_json::json!({"success": false, "error": error.to_string()}))
            }
        }
    }

    #[napi(getter)]
    pub fn count(&self) -> u32 {
        self.configs.len() as u32
    }
}
