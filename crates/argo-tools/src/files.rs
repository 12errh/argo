use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::ToolError;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

pub struct FilesTool {
    allowed_paths: Vec<String>,
    max_execution_time: Duration,
}

#[derive(Serialize, Deserialize)]
struct FilesInput {
    action: String,
    path: Option<String>,
    content: Option<String>,
}

impl FilesTool {
    pub fn new(allowed_paths: Vec<String>, max_execution_time: Duration) -> Self {
        Self {
            allowed_paths,
            max_execution_time,
        }
    }

    fn is_path_allowed(&self, path: &Path) -> bool {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };
        self.allowed_paths.iter().any(|allowed| {
            if let Ok(allowed_canonical) = Path::new(allowed).canonicalize() {
                canonical.starts_with(&allowed_canonical)
            } else {
                false
            }
        })
    }
}

#[async_trait]
impl Tool for FilesTool {
    fn name(&self) -> &str {
        "files"
    }

    fn description(&self) -> &str {
        "Read, write, list, and delete files"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read", "write", "list", "delete"],
                    "description": "File operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (only for write action)"
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
                "error": { "type": "string" }
            }
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: false,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: self.allowed_paths.clone(),
            allowed_domains: Vec::new(),
            max_execution_time: self.max_execution_time,
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        let files_input: FilesInput =
            serde_json::from_value(input).map_err(|e| ToolError::InvalidInput {
                reason: e.to_string(),
            })?;

        let path = PathBuf::from(files_input.path.ok_or_else(|| ToolError::InvalidInput {
            reason: "missing 'path' field".to_string(),
        })?);

        if !self.is_path_allowed(&path) {
            return Err(ToolError::PermissionDenied {
                resource: path.display().to_string(),
            });
        }

        match files_input.action.as_str() {
            "read" => {
                let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
                    ToolError::ExecutionFailed {
                        reason: e.to_string(),
                    }
                })?;
                Ok(json!({
                    "success": true,
                    "content": content
                }))
            }
            "write" => {
                let content = files_input.content.unwrap_or_default();
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        ToolError::ExecutionFailed {
                            reason: e.to_string(),
                        }
                    })?;
                }
                tokio::fs::write(&path, &content).await.map_err(|e| {
                    ToolError::ExecutionFailed {
                        reason: e.to_string(),
                    }
                })?;
                Ok(json!({
                    "success": true,
                    "content": format!("Written {} bytes", content.len())
                }))
            }
            "list" => {
                let mut entries = Vec::new();
                let mut dir =
                    tokio::fs::read_dir(&path)
                        .await
                        .map_err(|e| ToolError::ExecutionFailed {
                            reason: e.to_string(),
                        })?;
                while let Some(entry) =
                    dir.next_entry()
                        .await
                        .map_err(|e| ToolError::ExecutionFailed {
                            reason: e.to_string(),
                        })?
                {
                    entries.push(entry.file_name().to_string_lossy().to_string());
                }
                Ok(json!({
                    "success": true,
                    "content": serde_json::to_string(&entries).unwrap_or_default()
                }))
            }
            "delete" => {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed {
                        reason: e.to_string(),
                    })?;
                Ok(json!({
                    "success": true,
                    "content": "Deleted".to_string()
                }))
            }
            other => Err(ToolError::InvalidInput {
                reason: format!("unknown action: {}", other),
            }),
        }
    }
}
