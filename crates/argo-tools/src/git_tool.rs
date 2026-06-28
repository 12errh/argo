use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct GitTool;

impl GitTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Execute git operations: clone, commit, push, diff, branch, status, log."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["clone", "commit", "push", "diff", "branch", "status", "log", "add"],
                    "description": "Git action to perform"
                },
                "repo_url": {
                    "type": "string",
                    "description": "Repository URL (for clone)"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message (for commit)"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (for branch)"
                },
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Files to add (for add/commit)"
                }
            },
            "required": ["action"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "output": { "type": "string" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: true,
            allow_subprocess: true,
            working_directory: None,
            allowed_paths: vec![],
            allowed_domains: vec![],
            max_execution_time: Duration::from_secs(30),
        }
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let action = input
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "Missing 'action' parameter".to_string(),
            })?;

        let output = match action {
            "clone" => {
                let url = input
                    .get("repo_url")
                    .and_then(|u| u.as_str())
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "Missing 'repo_url' for clone".to_string(),
                    })?;

                run_git_command(&["clone", url], &ctx.working_dir).await?
            }
            "commit" => {
                let message = input
                    .get("message")
                    .and_then(|m| m.as_str())
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "Missing 'message' for commit".to_string(),
                    })?;

                if let Some(files) = input.get("files").and_then(|f| f.as_array()) {
                    for file in files {
                        if let Some(path) = file.as_str() {
                            run_git_command(&["add", path], &ctx.working_dir).await?;
                        }
                    }
                }

                run_git_command(&["commit", "-m", message], &ctx.working_dir).await?
            }
            "push" => {
                let branch = input
                    .get("branch")
                    .and_then(|b| b.as_str())
                    .unwrap_or("main");
                run_git_command(&["push", "origin", branch], &ctx.working_dir).await?
            }
            "diff" => run_git_command(&["diff"], &ctx.working_dir).await?,
            "branch" => {
                let branch_name = input.get("branch").and_then(|b| b.as_str());
                match branch_name {
                    Some(name) => run_git_command(&["branch", name], &ctx.working_dir).await?,
                    None => run_git_command(&["branch", "-a"], &ctx.working_dir).await?,
                }
            }
            "status" => run_git_command(&["status"], &ctx.working_dir).await?,
            "log" => run_git_command(&["log", "--oneline", "-20"], &ctx.working_dir).await?,
            "add" => {
                let files = input
                    .get("files")
                    .and_then(|f| f.as_array())
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "Missing 'files' for add".to_string(),
                    })?;

                let mut all_output = String::new();
                for file in files {
                    if let Some(path) = file.as_str() {
                        let out = run_git_command(&["add", path], &ctx.working_dir).await?;
                        all_output.push_str(&out);
                        all_output.push('\n');
                    }
                }
                all_output
            }
            _ => {
                return Err(ToolError::InvalidInput {
                    reason: format!("Unknown git action: {}", action),
                });
            }
        };

        Ok(json!({
            "success": true,
            "output": output
        }))
    }
}

async fn run_git_command(args: &[&str], working_dir: &str) -> Result<String, ToolError> {
    let output = tokio::process::Command::new("git")
        .args(args)
        .current_dir(working_dir)
        .output()
        .await
        .map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to run git: {}", e),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ToolError::ExecutionFailed {
            reason: format!("Git command failed: {}", stderr),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_name() {
        let tool = GitTool::new();
        assert_eq!(tool.name(), "git");
    }

    #[test]
    fn test_git_permissions() {
        let tool = GitTool::new();
        let perms = tool.permissions();
        assert!(perms.allow_network);
        assert!(perms.allow_filesystem);
        assert!(perms.allow_subprocess);
    }

    #[test]
    fn test_git_input_schema() {
        let tool = GitTool::new();
        let schema = tool.input_schema();
        let props = schema.get("properties").unwrap();
        assert!(props.get("action").is_some());
        assert!(props.get("repo_url").is_some());
    }
}
