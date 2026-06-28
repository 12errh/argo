use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

use commands::{init, inspect, loop_cmd, mcp, memory, package, run, stats, tools, validate};

#[derive(Parser)]
#[command(
    name = "argo",
    about = "Argo Agent Framework CLI — build self-healing, self-improving agents",
    version,
    long_about = "Argo is the first agent framework where agents genuinely get better over time.\nEvery error makes them smarter. Every task builds experience."
)]
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
        /// Directory to initialize in (default: current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Run an agent with a goal
    Run {
        /// Path to agent config file
        #[arg(short, long, default_value = "agent.toml")]
        config: PathBuf,
        /// Task goal for the agent
        goal: String,
        /// Show live heal trace
        #[arg(long)]
        inspect: bool,
        /// Environment profile to use
        #[arg(short, long)]
        env: Option<String>,
    },

    /// Run a loop agent until quality threshold is met
    Loop {
        /// Path to agent config file (must include [quality] section)
        #[arg(short, long, default_value = "agent.toml")]
        config: PathBuf,
        /// Show live heal trace
        #[arg(long)]
        inspect: bool,
    },

    /// Inspect a completed or running agent run
    Inspect {
        /// Run ID to inspect
        run_id: String,
        /// Show full trace details
        #[arg(long)]
        trace: bool,
        /// Show heal steps
        #[arg(long)]
        heal: bool,
        /// Show lessons learned
        #[arg(long)]
        lessons: bool,
    },

    /// Manage agent memory
    #[command(subcommand)]
    Memory(memory::MemoryCommands),

    /// View agent performance statistics
    Stats {
        /// Agent name to filter by
        #[arg(short, long)]
        agent: Option<String>,
        /// Time range (e.g., "24h", "7d", "30d")
        #[arg(short, long, default_value = "24h")]
        range: String,
        /// Compare with another agent
        #[arg(long)]
        compare: Option<String>,
    },

    /// Evaluate agent against scenario files
    Eval {
        /// Path to scenario file or directory
        scenario: PathBuf,
        /// Path to agent config
        #[arg(short, long, default_value = "agent.toml")]
        config: PathBuf,
        /// Output report format (json, text)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate an agent config file
    Validate {
        /// Path to agent config file
        #[arg(default_value = "agent.toml")]
        config: PathBuf,
    },

    /// List or inspect available tools
    #[command(subcommand)]
    Tools(tools::ToolsCommands),

    /// Manage MCP server connections
    #[command(subcommand)]
    Mcp(mcp::McpCommands),

    /// Build and package agent for distribution
    Package {
        /// Path to agent config
        #[arg(short, long, default_value = "agent.toml")]
        config: PathBuf,
        /// Output directory for packaged agent
        #[arg(short, long, default_value = "./dist")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, dir } => init::execute(&name, dir.as_deref()),
        Commands::Run {
            config,
            goal,
            inspect,
            env,
        } => run::execute(&config, &goal, inspect, env.as_deref()).await,
        Commands::Loop { config, inspect } => loop_cmd::execute(&config, inspect).await,
        Commands::Inspect {
            run_id,
            trace,
            heal,
            lessons,
        } => inspect::execute(&run_id, trace, heal, lessons).await,
        Commands::Memory(cmd) => memory::execute(cmd).await,
        Commands::Stats {
            agent,
            range,
            compare,
        } => stats::execute(agent.as_deref(), &range, compare.as_deref()).await,
        Commands::Eval {
            scenario,
            config,
            format,
        } => {
            println!(
                "Evaluating agent '{}' against scenario: {}",
                config.display(),
                scenario.display()
            );
            println!("Output format: {}", format);
            println!("Eval system will be available in Phase 5.");
            Ok(())
        }
        Commands::Validate { config } => validate::execute(&config),
        Commands::Tools(cmd) => tools::execute(cmd),
        Commands::Mcp(cmd) => mcp::execute(cmd).await,
        Commands::Package { config, output } => package::execute(&config, &output),
    }
}
