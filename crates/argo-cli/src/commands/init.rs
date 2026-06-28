use std::path::Path;

pub fn execute(name: &str, dir: Option<&Path>) -> anyhow::Result<()> {
    let base_dir = dir.unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(base_dir)?;

    let config_content = format!(
        r#"[agent]
name = "{name}"
version = "0.1.0"
description = "Agent: {name}"

[model]
provider = "anthropic"
model = "claude-sonnet-4-6"
api_key = "${{ANTHROPIC_API_KEY}}"
temperature = 0.7
max_tokens = 8192

[memory]
mode = "persistent"
short_term_ttl = 3600

[heal]
enabled = true
max_attempts = 7
strategies = [
    "retry",
    "reframe",
    "swap_tool",
    "decompose",
    "spawn_subagent",
    "change_provider",
    "reduce_scope",
]

[tools]
enabled = ["bash", "files"]

[permissions]
allow_network = false
allow_filesystem = true
allowed_paths = ["./workspace", "/tmp"]
max_execution_time = 300

[observe]
enabled = false
backend = "none"
"#
    );

    let config_path = base_dir.join("agent.toml");
    std::fs::write(&config_path, &config_content)?;
    println!("Created {}", config_path.display());

    let gitignore = r#"# Argo
/dist/
*.log
.env
.env.*
!.env.example
/target/
"#;
    let gitignore_path = base_dir.join(".gitignore");
    std::fs::write(&gitignore_path, gitignore)?;
    println!("Created {}", gitignore_path.display());

    let readme = format!(
        r#"# {name}

An Argo agent.

## Quick Start

```bash
# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Run the agent
argo run --config agent.toml "Your task goal here"
```

## Configuration

Edit `agent.toml` to configure the agent's model, tools, memory, and healing behavior.

## Learn More

- [Argo Documentation](https://github.com/argo-agents/argo)
- [Master Plan](docs/argo-master-plan.md)
"#
    );
    let readme_path = base_dir.join("README.md");
    std::fs::write(&readme_path, readme)?;
    println!("Created {}", readme_path.display());

    println!("\nAgent '{}' initialized successfully!", name);
    println!("Next steps:");
    println!("  1. Set your API key: export ANTHROPIC_API_KEY=your-key");
    println!("  2. Run the agent: argo run --config agent.toml \"your goal\"");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init_creates_files() {
        let dir = std::env::temp_dir().join("argo_test_init");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        execute("test-agent", Some(&dir)).unwrap();

        assert!(dir.join("agent.toml").exists());
        assert!(dir.join(".gitignore").exists());
        assert!(dir.join("README.md").exists());

        let config = fs::read_to_string(dir.join("agent.toml")).unwrap();
        assert!(config.contains("name = \"test-agent\""));
        assert!(config.contains("provider = \"anthropic\""));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_init_default_dir() {
        let dir = std::env::temp_dir().join("argo_test_init_default");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        execute("default-agent", None).unwrap();

        assert!(Path::new("agent.toml").exists());

        let _ = fs::remove_dir_all(&dir);
        std::env::set_current_dir(original_dir).unwrap();
    }
}
