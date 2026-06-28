use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::util;

#[pyclass]
#[derive(Clone)]
pub struct Agent {
    config: argo_core::config::AgentConfig,
}

#[pymethods]
impl Agent {
    #[new]
    #[pyo3(signature = (name, model, provider, api_key, tools=None, memory_mode="persistent", heal_enabled=true))]
    fn new(
        name: &str,
        model: &str,
        provider: &str,
        api_key: &str,
        tools: Option<Vec<String>>,
        memory_mode: &str,
        heal_enabled: bool,
    ) -> Self {
        let tool_names = tools.unwrap_or_else(|| vec!["bash".to_string(), "files".to_string()]);

        let config = argo_core::config::AgentConfig {
            agent: argo_core::config::AgentSection {
                name: name.to_string(),
                version: Some("0.1.0".to_string()),
                description: None,
            },
            model: argo_core::config::ModelSection {
                provider: provider.to_string(),
                model: model.to_string(),
                api_key: Some(api_key.to_string()),
                temperature: None,
                max_tokens: None,
                context_strategy: None,
            },
            memory: Some(argo_core::config::MemorySection {
                mode: Some(memory_mode.to_string()),
                short_term_ttl: Some(3600),
                long_term_backend: None,
                vector_backend: None,
                embedding_model: None,
            }),
            heal: Some(argo_core::config::HealSection {
                enabled: Some(heal_enabled),
                max_attempts: Some(7),
                strategies: None,
                background: None,
            }),
            quality: None,
            tools: argo_core::config::ToolsSection {
                enabled: tool_names,
            },
            permissions: argo_core::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: Some(300),
            },
            observe: None,
        };

        Self { config }
    }

    #[staticmethod]
    #[allow(clippy::useless_conversion)]
    fn from_config(path: &str) -> PyResult<Self> {
        let config_path = std::path::Path::new(path);
        let config = argo_core::config::AgentConfig::from_file(config_path)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { config })
    }

    #[allow(clippy::useless_conversion)]
    fn run<'py>(&self, py: Python<'py>, goal: &str) -> PyResult<Bound<'py, PyDict>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let result = rt.block_on(async {
            let llm = util::create_provider(&self.config)?;
            let tool_registry =
                util::create_tool_registry(&self.config.tools.enabled, &self.config)?;
            let memory = util::create_memory(&self.config).await?;
            argo_core::execution::execute_task(
                goal,
                llm.as_ref(),
                &tool_registry,
                &memory,
                &self.config,
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))
        });

        let dict = PyDict::new(py);
        match result {
            Ok(argo_core::message::TaskResult::Success { output }) => {
                dict.set_item("success", true)?;
                dict.set_item("output", output)?;
            }
            Ok(argo_core::message::TaskResult::Partial { output, reason }) => {
                dict.set_item("success", false)?;
                dict.set_item("output", output)?;
                dict.set_item("reason", reason)?;
            }
            Ok(argo_core::message::TaskResult::Failed { error }) => {
                dict.set_item("success", false)?;
                dict.set_item("error", error.to_string())?;
            }
            Err(e) => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
            }
        }
        Ok(dict)
    }

    #[getter]
    fn name(&self) -> &str {
        &self.config.agent.name
    }

    #[getter]
    fn model(&self) -> &str {
        &self.config.model.model
    }

    #[getter]
    fn provider(&self) -> &str {
        &self.config.model.provider
    }

    #[getter]
    fn tools(&self) -> Vec<String> {
        self.config.tools.enabled.clone()
    }
}

impl Agent {
    pub(crate) fn get_config(&self) -> &argo_core::config::AgentConfig {
        &self.config
    }
}

#[pyclass]
pub struct LoopAgent {
    config: argo_core::config::AgentConfig,
    threshold: f32,
    max_iterations: usize,
}

#[pymethods]
impl LoopAgent {
    #[new]
    #[pyo3(signature = (name, model, provider, api_key, threshold=0.85, max_iterations=20, tools=None, memory_mode="persistent"))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &str,
        model: &str,
        provider: &str,
        api_key: &str,
        threshold: f32,
        max_iterations: usize,
        tools: Option<Vec<String>>,
        memory_mode: &str,
    ) -> Self {
        let tool_names = tools.unwrap_or_else(|| vec!["bash".to_string(), "files".to_string()]);

        let config = argo_core::config::AgentConfig {
            agent: argo_core::config::AgentSection {
                name: name.to_string(),
                version: Some("0.1.0".to_string()),
                description: Some(format!("Loop agent: {}", name)),
            },
            model: argo_core::config::ModelSection {
                provider: provider.to_string(),
                model: model.to_string(),
                api_key: Some(api_key.to_string()),
                temperature: None,
                max_tokens: None,
                context_strategy: None,
            },
            memory: Some(argo_core::config::MemorySection {
                mode: Some(memory_mode.to_string()),
                short_term_ttl: Some(3600),
                long_term_backend: None,
                vector_backend: None,
                embedding_model: None,
            }),
            heal: Some(argo_core::config::HealSection {
                enabled: Some(true),
                max_attempts: Some(7),
                strategies: None,
                background: None,
            }),
            quality: Some(argo_core::config::QualitySection {
                threshold: Some(threshold),
                max_iterations: Some(max_iterations),
                criteria: None,
            }),
            tools: argo_core::config::ToolsSection {
                enabled: tool_names,
            },
            permissions: argo_core::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: Some(300),
            },
            observe: None,
        };

        Self {
            config,
            threshold,
            max_iterations,
        }
    }

    #[allow(clippy::useless_conversion)]
    fn run<'py>(&self, py: Python<'py>, goal: &str) -> PyResult<Bound<'py, PyDict>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let threshold = self.threshold;
        let max_iterations = self.max_iterations;

        let result = rt.block_on(async {
            let llm = util::create_provider(&self.config)?;
            let tool_registry =
                util::create_tool_registry(&self.config.tools.enabled, &self.config)?;
            let memory = util::create_memory(&self.config).await?;

            let mut best_score: f32 = 0.0;
            let mut best_output = String::new();
            let mut final_iteration = max_iterations;

            for iteration in 1..=max_iterations {
                let task_result = argo_core::execution::execute_task(
                    goal,
                    llm.as_ref(),
                    &tool_registry,
                    &memory,
                    &self.config,
                )
                .await;

                if let Ok(argo_core::message::TaskResult::Success { output }) = task_result {
                    let score = util::estimate_quality(&output);
                    if score > best_score {
                        best_score = score;
                        best_output = output;
                    }
                    if score >= threshold {
                        final_iteration = iteration;
                        break;
                    }
                }
            }

            Ok::<_, anyhow::Error>((best_output, best_score, final_iteration))
        });

        let (output, score, iterations) =
            result.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let dict = PyDict::new(py);
        dict.set_item("output", output)?;
        dict.set_item("score", score)?;
        dict.set_item("iterations", iterations)?;
        dict.set_item("threshold_reached", score >= threshold)?;
        Ok(dict)
    }

    #[getter]
    fn name(&self) -> &str {
        &self.config.agent.name
    }

    #[getter]
    fn threshold(&self) -> f32 {
        self.threshold
    }

    #[getter]
    fn max_iterations(&self) -> usize {
        self.max_iterations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_new() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            let agent = Agent::new(
                "test-agent",
                "claude-sonnet-4-6",
                "anthropic",
                "test-key",
                Some(vec!["bash".to_string(), "files".to_string()]),
                "persistent",
                true,
            );
            assert_eq!(agent.name(), "test-agent");
            assert_eq!(agent.model(), "claude-sonnet-4-6");
            assert_eq!(agent.provider(), "anthropic");
            assert_eq!(agent.tools(), vec!["bash", "files"]);
        });
    }

    #[test]
    fn test_agent_default_tools() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            let agent = Agent::new(
                "test-agent",
                "gpt-4",
                "openai",
                "test-key",
                None,
                "ephemeral",
                false,
            );
            assert_eq!(agent.tools(), vec!["bash", "files"]);
        });
    }

    #[test]
    fn test_loop_agent_new() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            let agent = LoopAgent::new(
                "loop-agent",
                "claude-sonnet-4-6",
                "anthropic",
                "test-key",
                0.9,
                10,
                None,
                "persistent",
            );
            assert_eq!(agent.name(), "loop-agent");
            assert!((agent.threshold() - 0.9).abs() < 0.001);
            assert_eq!(agent.max_iterations(), 10);
        });
    }

    #[test]
    fn test_loop_agent_defaults() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            let agent = LoopAgent::new(
                "loop-agent",
                "gpt-4",
                "openai",
                "test-key",
                0.85,
                20,
                None,
                "persistent",
            );
            assert!((agent.threshold() - 0.85).abs() < 0.001);
            assert_eq!(agent.max_iterations(), 20);
        });
    }
}
