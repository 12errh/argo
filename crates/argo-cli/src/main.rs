use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "argo", about = "Argo Agent Framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new agent project
    Init {
        /// Agent name
        name: String,
    },
    /// Run an agent
    Run {
        /// Path to agent config file
        config: String,
        /// Task goal
        goal: String,
        /// Show live heal trace
        #[arg(long)]
        inspect: bool,
    },
    /// Validate a config file
    Validate {
        /// Path to agent config file
        config: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            let config_content = format!(
                r#"[agent]
name = "{}"
version = "0.1.0"
description = "Agent: {}"

[model]
provider = "anthropic"
model = "claude-sonnet-4-6"
api_key = "${{ANTHROPIC_API_KEY}}"

[tools]
enabled = ["bash", "files"]

[permissions]
allow_network = false
allow_filesystem = true
"#,
                name, name
            );
            std::fs::write("agent.toml", &config_content)?;
            println!("Created agent.toml for '{}'", name);
        }
        Commands::Run {
            config,
            goal,
            inspect,
        } => {
            argo_observe::tracing::init_tracing(false, "none", "");
            let config_path = std::path::Path::new(&config);
            let agent_config = argo_core::config::AgentConfig::from_file(config_path)?;
            println!(
                "Running agent '{}' with goal: {}",
                agent_config.agent.name, goal
            );
            println!("Config loaded from: {}", config);
            if inspect {
                println!("Inspect mode enabled");
            }
        }
        Commands::Validate { config } => {
            let config_path = std::path::Path::new(&config);
            match argo_core::config::AgentConfig::from_file(config_path) {
                Ok(_) => println!("Config '{}' is valid", config),
                Err(e) => {
                    eprintln!("Config validation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
