use std::sync::Arc;
use std::time::Duration;

pub(crate) fn create_provider(
    config: &argo_core::config::AgentConfig,
) -> anyhow::Result<Arc<dyn argo_core::llm::LlmProvider>> {
    let api_key = config.model.api_key.as_deref().unwrap_or("");
    match config.model.provider.as_str() {
        "anthropic" => Ok(Arc::new(argo_core::llm::anthropic::AnthropicProvider::new(
            api_key.to_string(),
            config.model.model.clone(),
        ))),
        "openai" => Ok(Arc::new(argo_core::llm::openai::OpenAiProvider::new(
            api_key.to_string(),
            config.model.model.clone(),
        ))),
        _ => anyhow::bail!("Provider '{}' not supported", config.model.provider),
    }
}

pub(crate) fn create_tool_registry(
    tool_names: &[String],
    config: &argo_core::config::AgentConfig,
) -> anyhow::Result<argo_tools::registry::ToolRegistry> {
    let mut registry = argo_tools::registry::ToolRegistry::new();
    let max_exec_time = Duration::from_secs(config.permissions.max_execution_time.unwrap_or(300));
    let allowed_paths = config
        .permissions
        .allowed_paths
        .clone()
        .unwrap_or_else(|| vec![".".to_string()]);

    for tool_name in tool_names {
        match tool_name.as_str() {
            "bash" => {
                let working_dir = std::env::current_dir()
                    .unwrap_or_default()
                    .display()
                    .to_string();
                registry.register(Arc::new(argo_tools::bash::BashTool::new(
                    working_dir,
                    max_exec_time,
                )));
            }
            "files" => {
                registry.register(Arc::new(argo_tools::files::FilesTool::new(
                    allowed_paths.clone(),
                    max_exec_time,
                )));
            }
            "http" => {
                registry.register(Arc::new(argo_tools::http::HttpTool::new(
                    Vec::new(),
                    max_exec_time,
                )));
            }
            _ => {}
        }
    }
    Ok(registry)
}

pub(crate) async fn create_memory(
    _config: &argo_core::config::AgentConfig,
) -> anyhow::Result<argo_memory::handle::MemoryHandle> {
    let redis = argo_memory::redis::RedisMemory::new("redis://localhost:6379").await?;
    let surreal = argo_memory::surreal::SurrealMemory::new("ws://localhost:8000", "argo", "memory");
    Ok(argo_memory::handle::MemoryHandle::new(redis, surreal))
}

pub(crate) fn estimate_quality(output: &str) -> f32 {
    let len = output.len();
    if len == 0 {
        return 0.0;
    }
    let mut score: f32 = 0.5;
    if len > 100 {
        score += 0.1;
    }
    if len > 500 {
        score += 0.1;
    }
    if output.contains("test") || output.contains("assert") {
        score += 0.1;
    }
    if output.contains("fn ") || output.contains("def ") || output.contains("function ") {
        score += 0.1;
    }
    score.min(1.0)
}

pub(crate) fn build_config(
    name: &str,
    model: &str,
    provider: &str,
    api_key: &str,
    tools: Vec<String>,
) -> argo_core::config::AgentConfig {
    argo_core::config::AgentConfig {
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
            mode: Some("persistent".to_string()),
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
        quality: None,
        tools: argo_core::config::ToolsSection { enabled: tools },
        permissions: argo_core::config::PermissionsSection {
            allow_network: false,
            allow_filesystem: true,
            allowed_paths: None,
            max_execution_time: Some(300),
        },
        observe: None,
    }
}
