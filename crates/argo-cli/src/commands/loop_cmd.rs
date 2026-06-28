use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub async fn execute(config_path: &Path, inspect: bool) -> anyhow::Result<()> {
    if !config_path.exists() {
        anyhow::bail!("Config file not found: {}", config_path.display());
    }

    let config = argo_core::config::AgentConfig::from_file(config_path)?;

    let quality = config.quality.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Config must include [quality] section for loop agent. See B-04 schema for details."
        )
    })?;

    let threshold = quality.threshold.unwrap_or(0.85);
    let max_iterations = quality.max_iterations.unwrap_or(20);

    println!(
        "Running loop agent '{}' (model: {})",
        config.agent.name, config.model.model
    );
    println!("Quality threshold: {:.2}", threshold);
    println!("Max iterations: {}", max_iterations);
    println!();

    argo_observe::tracing::init_tracing(false, "none", "");

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
            _ => {}
        }
    }

    let redis_url = "redis://localhost:6379";
    let surreal_endpoint = "ws://localhost:8000";
    let surreal_namespace = "argo";
    let surreal_database = "memory";

    let redis = argo_memory::redis::RedisMemory::new(redis_url).await?;
    let surreal = argo_memory::surreal::SurrealMemory::new(
        surreal_endpoint,
        surreal_namespace,
        surreal_database,
    );
    let memory = argo_memory::handle::MemoryHandle::new(redis, surreal);

    let goal = config
        .agent
        .description
        .clone()
        .unwrap_or_else(|| "Complete the assigned task".to_string());

    let mut best_score: f32 = 0.0;
    let mut best_output = String::new();

    for iteration in 1..=max_iterations {
        if inspect {
            println!("--- Iteration {} ---", iteration);
        }

        let result = argo_core::execution::execute_task(
            &goal,
            llm.as_ref(),
            &tool_registry,
            &memory,
            &config,
        )
        .await;

        match result {
            Ok(argo_core::message::TaskResult::Success { output }) => {
                let score = estimate_quality(&output);
                if inspect {
                    println!("Quality score: {:.2}", score);
                }

                if score > best_score {
                    best_score = score;
                    best_output = output;
                }

                if score >= threshold {
                    println!(
                        "\nLoop agent completed after {} iterations (score: {:.2})",
                        iteration, score
                    );
                    println!("--- Best Result ---");
                    println!("{}", best_output);
                    return Ok(());
                }
            }
            Ok(_) => {
                if inspect {
                    println!("Iteration {} produced partial/failed result", iteration);
                }
            }
            Err(e) => {
                if inspect {
                    eprintln!("Iteration {} error: {}", iteration, e);
                }
            }
        }
    }

    println!(
        "\nLoop agent reached max iterations ({}) with best score: {:.2}",
        max_iterations, best_score
    );
    if !best_output.is_empty() {
        println!("--- Best Result ---");
        println!("{}", best_output);
    }

    Ok(())
}

fn estimate_quality(output: &str) -> f32 {
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
            "Provider '{}' not supported for loop agent",
            config.model.provider
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_quality_empty() {
        assert_eq!(estimate_quality(""), 0.0);
    }

    #[test]
    fn test_estimate_quality_basic() {
        let score = estimate_quality("Hello world");
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_estimate_quality_with_code() {
        let score = estimate_quality("fn main() { assert!(true); }");
        assert!(score >= 0.7);
    }
}
