//! Argo CLI — Command-line interface for Argo agent framework.

use clap::Parser;

#[derive(Parser)]
#[command(name = "argo", about = "Argo Agent Framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            println!("Initializing agent project: {}", name);
            // TODO: Create my-agent.toml, .gitignore, README.md
        }
        Commands::Run { config, goal, inspect } => {
            println!("Running agent from {} with goal: {}", config, goal);
            if inspect {
                println!("Live inspection enabled");
            }
            // TODO: Load config, create agent, run task
        }
    }

    Ok(())
}
