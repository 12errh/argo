use std::path::Path;

pub fn execute(config_path: &Path) -> anyhow::Result<()> {
    if !config_path.exists() {
        anyhow::bail!("Config file not found: {}", config_path.display());
    }

    println!("Validating config: {}", config_path.display());

    match argo_core::config::AgentConfig::from_file(config_path) {
        Ok(config) => {
            println!();
            println!("Config is valid!");
            println!();
            println!("Agent: {}", config.agent.name);
            println!(
                "Version: {}",
                config.agent.version.as_deref().unwrap_or("unspecified")
            );
            println!("Provider: {}", config.model.provider);
            println!("Model: {}", config.model.model);
            println!(
                "Memory mode: {}",
                config
                    .memory
                    .as_ref()
                    .and_then(|m| m.mode.as_deref())
                    .unwrap_or("persistent")
            );
            println!(
                "Heal enabled: {}",
                config.heal.as_ref().and_then(|h| h.enabled).unwrap_or(true)
            );
            println!("Tools: {:?}", config.tools.enabled);
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Validation failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_validate_valid_config() {
        let dir = std::env::temp_dir().join("argo_test_validate");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_content = r#"[agent]
name = "test-agent"
version = "0.1.0"

[model]
provider = "anthropic"
model = "claude-sonnet-4-6"

[tools]
enabled = ["bash"]

[permissions]
allow_network = false
allow_filesystem = true
"#;

        let config_path = dir.join("agent.toml");
        fs::write(&config_path, config_content).unwrap();

        let result = execute(&config_path);
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_missing_file() {
        let result = execute(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_config() {
        let dir = std::env::temp_dir().join("argo_test_validate_invalid");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_content = r#"[agent]
name = "test"
version = "0.1.0"

[model]
provider = "invalid_provider"
model = "test"

[tools]
enabled = []

[permissions]
allow_network = false
allow_filesystem = true
"#;

        let config_path = dir.join("agent.toml");
        fs::write(&config_path, config_content).unwrap();

        let result = execute(&config_path);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
