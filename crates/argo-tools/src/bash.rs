use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct BashTool {
    working_directory: String,
    max_execution_time: Duration,
}

impl BashTool {
    pub fn new(working_directory: String, max_execution_time: Duration) -> Self {
        Self {
            working_directory,
            max_execution_time,
        }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute shell commands"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "stdout": { "type": "string" },
                "stderr": { "type": "string" },
                "exit_code": { "type": "integer" },
                "success": { "type": "boolean" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: false,
            allow_subprocess: true,
            working_directory: Some(self.working_directory.clone()),
            allowed_paths: Vec::new(),
            allowed_domains: Vec::new(),
            max_execution_time: self.max_execution_time,
        }
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "missing 'command' field".to_string(),
            })?;

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd.current_dir(&ctx.working_dir);

        let output = tokio::time::timeout(self.max_execution_time, cmd.output())
            .await
            .map_err(|_| ToolError::Timeout {
                elapsed: self.max_execution_time,
            })?
            .map_err(|e| ToolError::ExecutionFailed {
                reason: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "success": output.status.success()
        }))
    }
}
