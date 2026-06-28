use clap::Subcommand;

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// List all memory entries for an agent
    List {
        /// Agent name to list memory for
        #[arg(short, long)]
        agent: String,
        /// Memory type filter (short-term, long-term, semantic)
        #[arg(short, long)]
        r#type: Option<String>,
        /// Maximum number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Search semantic memory
    Search {
        /// Search query text
        query: String,
        /// Agent name to search within
        #[arg(short, long)]
        agent: Option<String>,
        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    /// Clear memory entries
    Clear {
        /// Agent name to clear memory for
        #[arg(short, long)]
        agent: String,
        /// Memory type to clear (short-term, long-term, semantic, all)
        #[arg(short, long, default_value = "all")]
        r#type: String,
        /// Confirm without prompting
        #[arg(long)]
        force: bool,
    },
}

pub async fn execute(cmd: MemoryCommands) -> anyhow::Result<()> {
    match cmd {
        MemoryCommands::List {
            agent,
            r#type,
            limit,
        } => {
            println!("Memory entries for agent '{}':", agent);
            if let Some(t) = &r#type {
                println!("Filter: {}", t);
            }
            println!("Limit: {}", limit);
            println!();
            println!("Memory listing requires a running Redis/SurrealDB/Qdrant instance.");
            println!("Start services with: docker compose up -d");
            println!();
            println!("Once connected, memory entries will be displayed here.");
        }
        MemoryCommands::Search {
            query,
            agent,
            limit,
        } => {
            println!("Searching semantic memory: \"{}\"", query);
            if let Some(a) = &agent {
                println!("Agent: {}", a);
            }
            println!("Limit: {}", limit);
            println!();
            println!("Semantic search requires Qdrant to be running.");
            println!("Start services with: docker compose up -d");
        }
        MemoryCommands::Clear {
            agent,
            r#type,
            force,
        } => {
            if !force {
                println!(
                    "WARNING: This will clear {} memory for agent '{}'.",
                    r#type, agent
                );
                println!("Use --force to confirm.");
                return Ok(());
            }

            println!("Clearing {} memory for agent '{}'...", r#type, agent);
            println!("Memory clearing requires a running Redis/SurrealDB/Qdrant instance.");
            println!("Start services with: docker compose up -d");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_commands_exist() {
        let cmd = MemoryCommands::List {
            agent: "test".to_string(),
            r#type: None,
            limit: 10,
        };
        match cmd {
            MemoryCommands::List { agent, .. } => assert_eq!(agent, "test"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_memory_search_command() {
        let cmd = MemoryCommands::Search {
            query: "test query".to_string(),
            agent: Some("agent1".to_string()),
            limit: 5,
        };
        match cmd {
            MemoryCommands::Search { query, .. } => assert_eq!(query, "test query"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_memory_clear_command() {
        let cmd = MemoryCommands::Clear {
            agent: "test".to_string(),
            r#type: "all".to_string(),
            force: false,
        };
        match cmd {
            MemoryCommands::Clear { force, .. } => assert!(!force),
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_memory_list_no_services() {
        let cmd = MemoryCommands::List {
            agent: "test".to_string(),
            r#type: None,
            limit: 10,
        };
        let result = execute(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_search_no_services() {
        let cmd = MemoryCommands::Search {
            query: "test".to_string(),
            agent: None,
            limit: 5,
        };
        let result = execute(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_clear_no_force() {
        let cmd = MemoryCommands::Clear {
            agent: "test".to_string(),
            r#type: "all".to_string(),
            force: false,
        };
        let result = execute(cmd).await;
        assert!(result.is_ok());
    }
}
