use clap::Subcommand;

#[derive(Subcommand)]
pub enum ToolsCommands {
    /// List all available tools
    List,
    /// Show detailed info about a specific tool
    Info {
        /// Tool name
        name: String,
    },
}

pub fn execute(cmd: ToolsCommands) -> anyhow::Result<()> {
    match cmd {
        ToolsCommands::List => {
            println!("Available Tools");
            println!();
            println!("{:<15} {:<50} {}", "Name", "Description", "Version");
            println!("{:<15} {:<50} {}", "----", "-----------", "-------");

            let tools = vec![
                ("bash", "Execute shell commands with sandboxing", "0.1.0"),
                ("files", "Read, write, list, and delete files", "0.1.0"),
                (
                    "http",
                    "Make HTTP requests with domain restrictions",
                    "0.1.0",
                ),
            ];

            for (name, desc, version) in &tools {
                println!("{:<15} {:<50} {}", name, desc, version);
            }

            println!();
            println!("To inspect a tool: argo tools info <name>");
        }
        ToolsCommands::Info { name } => {
            println!("Tool: {}", name);
            println!();

            match name.as_str() {
                "bash" => {
                    println!("Description: Execute shell commands with sandboxing");
                    println!("Version: 0.1.0");
                    println!();
                    println!("Permissions:");
                    println!("  allow_filesystem: true");
                    println!("  allow_network: false");
                    println!("  allow_subprocess: true");
                    println!("  max_execution_time: 30s");
                    println!();
                    println!("Input Schema:");
                    println!("  command: string (required) — Shell command to execute");
                    println!("  timeout: integer (optional) — Timeout in seconds");
                    println!();
                    println!("Output Schema:");
                    println!("  stdout: string — Standard output");
                    println!("  stderr: string — Standard error");
                    println!("  exit_code: integer — Process exit code");
                }
                "files" => {
                    println!("Description: Read, write, list, and delete files");
                    println!("Version: 0.1.0");
                    println!();
                    println!("Permissions:");
                    println!("  allow_filesystem: true");
                    println!("  allow_network: false");
                    println!("  allow_subprocess: false");
                    println!();
                    println!("Input Schema:");
                    println!("  action: string (required) — read|write|list|delete");
                    println!("  path: string (required) — File path");
                    println!("  content: string (optional) — Content for write");
                }
                "http" => {
                    println!("Description: Make HTTP requests with domain restrictions");
                    println!("Version: 0.1.0");
                    println!();
                    println!("Permissions:");
                    println!("  allow_filesystem: false");
                    println!("  allow_network: true");
                    println!("  allow_subprocess: false");
                    println!();
                    println!("Input Schema:");
                    println!("  method: string (required) — GET|POST|PUT|DELETE");
                    println!("  url: string (required) — Target URL");
                    println!("  headers: object (optional) — Request headers");
                    println!("  body: string (optional) — Request body");
                }
                _ => {
                    eprintln!("Unknown tool: {}", name);
                    eprintln!("Available tools: bash, files, http");
                    anyhow::bail!("Unknown tool: {}", name);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list() {
        let result = execute(ToolsCommands::List);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tools_info_bash() {
        let result = execute(ToolsCommands::Info {
            name: "bash".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_tools_info_files() {
        let result = execute(ToolsCommands::Info {
            name: "files".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_tools_info_http() {
        let result = execute(ToolsCommands::Info {
            name: "http".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_tools_info_unknown() {
        let result = execute(ToolsCommands::Info {
            name: "unknown".to_string(),
        });
        assert!(result.is_err());
    }
}
