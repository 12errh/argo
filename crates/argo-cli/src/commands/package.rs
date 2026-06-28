use std::path::Path;

pub fn execute(config_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    if !config_path.exists() {
        anyhow::bail!("Config file not found: {}", config_path.display());
    }

    let config = argo_core::config::AgentConfig::from_file(config_path)?;

    println!("Packaging agent '{}'", config.agent.name);
    println!("Config: {}", config_path.display());
    println!("Output: {}", output_dir.display());
    println!();

    std::fs::create_dir_all(output_dir)?;

    let package_name = format!("argo-agent-{}", config.agent.name);

    let manifest = serde_json::json!({
        "name": config.agent.name,
        "version": config.agent.version.unwrap_or_else(|| "0.1.0".to_string()),
        "description": config.agent.description,
        "model": {
            "provider": config.model.provider,
            "model": config.model.model,
        },
        "tools": config.tools.enabled,
        "packaged_at": chrono::Utc::now().to_rfc3339(),
        "argo_version": env!("CARGO_PKG_VERSION"),
    });

    let manifest_path = output_dir.join("agent-manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!("Created {}", manifest_path.display());

    let config_out = output_dir.join("agent.toml");
    std::fs::copy(config_path, &config_out)?;
    println!("Copied config to {}", config_out.display());

    let readme = format!(
        r#"# {}

Packaged Argo agent.

## Run

```bash
argo run --config agent.toml "Your goal here"
```

## Configuration

Edit `agent.toml` to configure the agent.

## Package Info

- Packaged at: {}
- Argo version: {}
- Model: {} ({})
"#,
        config.agent.name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        env!("CARGO_PKG_VERSION"),
        config.model.model,
        config.model.provider,
    );

    let readme_path = output_dir.join("README.md");
    std::fs::write(readme_path, readme)?;
    println!("Created README.md");

    println!();
    println!(
        "Agent '{}' packaged successfully to {}",
        config.agent.name,
        output_dir.display()
    );
    println!("Package name: {}", package_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_package_creates_files() {
        let dir = std::env::temp_dir().join("argo_test_package");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_content = r#"[agent]
name = "test-agent"
version = "0.1.0"
description = "Test agent"

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

        let output_dir = dir.join("dist");
        let result = execute(&config_path, &output_dir);
        assert!(result.is_ok());

        assert!(output_dir.join("agent-manifest.json").exists());
        assert!(output_dir.join("agent.toml").exists());
        assert!(output_dir.join("README.md").exists());

        let manifest_content = fs::read_to_string(output_dir.join("agent-manifest.json")).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();
        assert_eq!(manifest["name"], "test-agent");

        let _ = fs::remove_dir_all(&dir);
    }
}
