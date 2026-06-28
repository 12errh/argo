use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct PythonTool;

impl PythonTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for PythonTool {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Execute Python code in a subprocess. Returns stdout, stderr, and exit code."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Python code to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 30)",
                    "default": 30
                }
            },
            "required": ["code"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "stdout": { "type": "string" },
                "stderr": { "type": "string" },
                "exit_code": { "type": "integer" }
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
        let code =
            input
                .get("code")
                .and_then(|c| c.as_str())
                .ok_or_else(|| ToolError::InvalidInput {
                    reason: "Missing 'code' parameter".to_string(),
                })?;

        let timeout_secs = input.get("timeout").and_then(|t| t.as_u64()).unwrap_or(30);

        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            tokio::process::Command::new("python3")
                .arg("-c")
                .arg(code)
                .current_dir(&ctx.working_dir)
                .output(),
        )
        .await
        .map_err(|_| ToolError::Timeout {
            elapsed: Duration::from_secs(timeout_secs),
        })?
        .map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to run Python: {}", e),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        if exit_code != 0 && stderr.contains("ModuleNotFoundError") {
            return Err(ToolError::ExecutionFailed {
                reason: format!("Python dependency error: {}", stderr),
            });
        }

        Ok(json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_name() {
        let tool = PythonTool::new();
        assert_eq!(tool.name(), "python");
    }

    #[test]
    fn test_python_permissions() {
        let tool = PythonTool::new();
        let perms = tool.permissions();
        assert!(perms.allow_subprocess);
        assert!(!perms.allow_network);
        assert!(perms.allow_filesystem);
    }

    #[test]
    fn test_python_input_schema() {
        let tool = PythonTool::new();
        let schema = tool.input_schema();
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&json!("code")));
    }
}
