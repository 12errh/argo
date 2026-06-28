use napi_derive::napi;

#[napi]
pub struct AgentConfig {
    inner: argo_core::config::AgentConfig,
}

#[napi]
impl AgentConfig {
    #[napi(factory)]
    pub fn from_file(path: String) -> napi::Result<Self> {
        let config_path = std::path::Path::new(&path);
        let inner = argo_core::config::AgentConfig::from_file(config_path)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.agent.name.clone()
    }

    #[napi(getter)]
    pub fn provider(&self) -> String {
        self.inner.model.provider.clone()
    }

    #[napi(getter)]
    pub fn model_name(&self) -> String {
        self.inner.model.model.clone()
    }
}
