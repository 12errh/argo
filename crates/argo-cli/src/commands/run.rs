use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub async fn execute(
    config_path: &Path,
    goal: &str,
    inspect: bool,
    _env: Option<&str>,
) -> anyhow::Result<()> {
    if !config_path.exists() {
        anyhow::bail!("Config file not found: {}", config_path.display());
    }

    let config = argo_core::config::AgentConfig::from_file(config_path)?;

    println!(
        "Running agent '{}' (model: {})",
        config.agent.name, config.model.model
    );
    println!("Goal: {}", goal);
    println!();

    argo_observe::tracing::init_tracing(
        config
            .observe
            .as_ref()
            .and_then(|o| o.enabled)
            .unwrap_or(false),
        config
            .observe
            .as_ref()
            .and_then(|o| o.backend.clone())
            .unwrap_or_else(|| "none".to_string())
            .as_str(),
        config
            .observe
            .as_ref()
            .and_then(|o| o.endpoint.clone())
            .unwrap_or_default()
            .as_str(),
    );

    let llm = create_provider(&config)?;

    let mut tool_registry = argo_tools::registry::ToolRegistry::new();
    let max_exec_time = Duration::from_secs(config.permissions.max_execution_time.unwrap_or(300));
    let allowed_paths = config
        .permissions
        .allowed_paths
        .clone()
        .unwrap_or_else(|| vec![".".to_string()]);

    for tool_name in &config.tools.enabled {
        match tool_name.as_str() {
            "bash" => {
                let working_dir = std::env::current_dir()
                    .unwrap_or_default()
                    .display()
                    .to_string();
                tool_registry.register(Arc::new(argo_tools::bash::BashTool::new(
                    working_dir,
                    max_exec_time,
                )));
            }
            "files" => {
                tool_registry.register(Arc::new(argo_tools::files::FilesTool::new(
                    allowed_paths.clone(),
                    max_exec_time,
                )));
            }
            "http" => {
                tool_registry.register(Arc::new(argo_tools::http::HttpTool::new(
                    Vec::new(),
                    max_exec_time,
                )));
            }
            _ => {
                eprintln!("Warning: unknown tool '{}', skipping", tool_name);
            }
        }
    }

    let redis_url = config
        .memory
        .as_ref()
        .and(None::<String>)
        .unwrap_or_else(|| "redis://localhost:6379".to_string());

    let surreal_endpoint = "ws://localhost:8000";
    let surreal_namespace = "argo";
    let surreal_database = "memory";

    let redis = argo_memory::redis::RedisMemory::new(&redis_url).await?;
    let surreal = argo_memory::surreal::SurrealMemory::new(
        surreal_endpoint,
        surreal_namespace,
        surreal_database,
    );
    let memory = argo_memory::handle::MemoryHandle::new(redis, surreal);

    if inspect {
        println!("--- Live Trace ---");
    }

    let result =
        argo_core::execution::execute_task(goal, llm.as_ref(), &tool_registry, &memory, &config)
            .await;

    match result {
        Ok(argo_core::message::TaskResult::Success { output }) => {
            println!("\n--- Result ---");
            println!("{}", output);
        }
        Ok(argo_core::message::TaskResult::Partial { output, reason }) => {
            println!("\n--- Partial Result ---");
            println!("{}", output);
            println!("\nReason: {}", reason);
        }
        Ok(argo_core::message::TaskResult::Failed { error }) => {
            eprintln!("\n--- Failed ---");
            eprintln!("{}", error);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("\n--- Error ---");
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn create_provider(
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
        _ => anyhow::bail!(
            "Provider '{}' is not yet supported for CLI run",
            config.model.provider
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_anthropic() {
        let config = argo_core::config::AgentConfig {
            agent: argo_core::config::AgentSection {
                name: "test".to_string(),
                version: Some("0.1.0".to_string()),
                description: None,
            },
            model: argo_core::config::ModelSection {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                api_key: Some("${TEST_KEY}".to_string()),
                temperature: None,
                max_tokens: None,
                context_strategy: None,
            },
            memory: None,
            heal: None,
            quality: None,
            tools: argo_core::config::ToolsSection {
                enabled: vec!["bash".to_string()],
            },
            permissions: argo_core::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: None,
            },
            observe: None,
        };

        let provider = create_provider(&config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_provider_unsupported() {
        let config = argo_core::config::AgentConfig {
            agent: argo_core::config::AgentSection {
                name: "test".to_string(),
                version: Some("0.1.0".to_string()),
                description: None,
            },
            model: argo_core::config::ModelSection {
                provider: "gemini".to_string(),
                model: "gemini-pro".to_string(),
                api_key: Some("key".to_string()),
                temperature: None,
                max_tokens: None,
                context_strategy: None,
            },
            memory: None,
            heal: None,
            quality: None,
            tools: argo_core::config::ToolsSection {
                enabled: vec!["bash".to_string()],
            },
            permissions: argo_core::config::PermissionsSection {
                allow_network: false,
                allow_filesystem: true,
                allowed_paths: None,
                max_execution_time: None,
            },
            observe: None,
        };

        let provider = create_provider(&config);
        assert!(provider.is_err());
    }
}
