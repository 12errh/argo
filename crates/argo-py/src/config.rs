use pyo3::prelude::*;

/// Configuration for an Argo agent.
#[pyclass]
pub struct AgentConfig {
    inner: argo_core::config::AgentConfig,
}

#[pymethods]
impl AgentConfig {
    /// Create a config from a TOML file.
    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Self> {
        let config_path = std::path::Path::new(path);
        let inner = argo_core::config::AgentConfig::from_file(config_path)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Get the agent name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.agent.name
    }

    /// Get the model provider.
    #[getter]
    fn provider(&self) -> &str {
        &self.inner.model.provider
    }

    /// Get the model name.
    #[getter]
    fn model_name(&self) -> &str {
        &self.inner.model.model
    }
}

impl AgentConfig {
    pub fn inner(&self) -> &argo_core::config::AgentConfig {
        &self.inner
    }
}
