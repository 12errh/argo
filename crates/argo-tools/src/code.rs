use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct CodeTool;

impl CodeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeTool {
    fn name(&self) -> &str {
        "code"
    }

    fn description(&self) -> &str {
        "Write, read, and execute code files. Supports creating new files, reading existing ones, and running code."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["write", "read", "run"],
                    "description": "Action to perform"
                },
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "content": {
                    "type": "string",
                    "description": "File content (for write action)"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (for run action): python, rust, node, bash"
                }
            },
            "required": ["action", "path"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "content": { "type": "string" },
                "path": { "type": "string" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: false,
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

        let path =
            input
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ToolError::InvalidInput {
                    reason: "Missing 'path' parameter".to_string(),
                })?;

        let full_path = if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            format!("{}/{}", ctx.working_dir, path)
        };

        match action {
            "write" => {
                let content = input
                    .get("content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "Missing 'content' for write action".to_string(),
                    })?;

                if let Some(parent) = Path::new(&full_path).parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        ToolError::ExecutionFailed {
                            reason: format!("Failed to create directory: {}", e),
                        }
                    })?;
                }

                tokio::fs::write(&full_path, content).await.map_err(|e| {
                    ToolError::ExecutionFailed {
                        reason: format!("Failed to write file: {}", e),
                    }
                })?;

                Ok(json!({
                    "success": true,
                    "content": format!("File written: {} ({} bytes)", full_path, content.len()),
                    "path": full_path
                }))
            }
            "read" => {
                let content = tokio::fs::read_to_string(&full_path).await.map_err(|e| {
                    ToolError::ExecutionFailed {
                        reason: format!("Failed to read file: {}", e),
                    }
                })?;

                Ok(json!({
                    "success": true,
                    "content": content,
                    "path": full_path
                }))
            }
            "run" => {
                let language = input
                    .get("language")
                    .and_then(|l| l.as_str())
                    .unwrap_or("bash");

                let output = match language {
                    "python" => {
                        tokio::process::Command::new("python3")
                            .arg(&full_path)
                            .current_dir(&ctx.working_dir)
                            .output()
                            .await
                    }
                    "node" => {
                        tokio::process::Command::new("node")
                            .arg(&full_path)
                            .current_dir(&ctx.working_dir)
                            .output()
                            .await
                    }
                    "rust" => {
                        let binary = full_path.trim_end_matches(".rs");
                        tokio::process::Command::new("rustc")
                            .args([full_path.as_str(), "-o", binary])
                            .current_dir(&ctx.working_dir)
                            .output()
                            .await
                            .map_err(|e| ToolError::ExecutionFailed {
                                reason: format!("Failed to compile Rust: {}", e),
                            })?;
                        tokio::process::Command::new(binary)
                            .current_dir(&ctx.working_dir)
                            .output()
                            .await
                    }
                    "bash" | "sh" => {
                        tokio::process::Command::new("bash")
                            .arg(&full_path)
                            .current_dir(&ctx.working_dir)
                            .output()
                            .await
                    }
                    _ => {
                        return Err(ToolError::InvalidInput {
                            reason: format!("Unsupported language: {}", language),
                        });
                    }
                };

                let output = output.map_err(|e| ToolError::ExecutionFailed {
                    reason: format!("Failed to run code: {}", e),
                })?;

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(json!({
                    "success": exit_code == 0,
                    "content": if exit_code == 0 { stdout } else { stderr },
                    "path": full_path
                }))
            }
            _ => Err(ToolError::InvalidInput {
                reason: format!("Unknown code action: {}", action),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_name() {
        let tool = CodeTool::new();
        assert_eq!(tool.name(), "code");
    }

    #[test]
    fn test_code_permissions() {
        let tool = CodeTool::new();
        let perms = tool.permissions();
        assert!(perms.allow_filesystem);
        assert!(perms.allow_subprocess);
        assert!(!perms.allow_network);
    }

    #[test]
    fn test_code_input_schema() {
        let tool = CodeTool::new();
        let schema = tool.input_schema();
        let props = schema.get("properties").unwrap();
        assert!(props.get("action").is_some());
        assert!(props.get("path").is_some());
        assert!(props.get("content").is_some());
    }
}
