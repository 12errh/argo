use pyo3::prelude::*;

use crate::agent::Agent;
use crate::util;

/// A pool of agents that can run tasks concurrently.
#[pyclass]
pub struct AgentPool {
    configs: Vec<argo_core::config::AgentConfig>,
}

#[pymethods]
impl AgentPool {
    #[new]
    fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Add an agent to the pool.
    fn add_agent(&mut self, agent: &Agent) {
        self.configs.push(agent.get_config().clone());
    }

    /// Run a task with the first agent and return the result.
    #[allow(clippy::useless_conversion)]
    fn run(&self, goal: &str) -> PyResult<String> {
        if self.configs.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No agents in pool",
            ));
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let result = rt.block_on(async {
            let config = &self.configs[0];
            let llm = util::create_provider(config)?;
            let tool_registry = util::create_tool_registry(&config.tools.enabled, config)?;
            let memory = util::create_memory(config).await?;

            argo_core::execution::execute_task(
                goal,
                llm.as_ref(),
                &tool_registry,
                &memory,
                config,
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))
        });

        match result {
            Ok(argo_core::message::TaskResult::Success { output }) => {
                Ok(serde_json::json!({"success": true, "output": output}).to_string())
            }
            Ok(argo_core::message::TaskResult::Partial { output, reason }) => {
                Ok(serde_json::json!({"success": false, "output": output, "reason": reason}).to_string())
            }
            Ok(argo_core::message::TaskResult::Failed { error }) => {
                Ok(serde_json::json!({"success": false, "error": error.to_string()}).to_string())
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    #[getter]
    fn count(&self) -> usize {
        self.configs.len()
    }
}
