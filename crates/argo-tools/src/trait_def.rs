use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::ToolError;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub run_id: String,
    pub working_dir: String,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolPermissions {
    pub allow_filesystem: bool,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub working_directory: Option<String>,
    pub allowed_paths: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub max_execution_time: Duration,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn output_schema(&self) -> Value;
    fn permissions(&self) -> ToolPermissions;
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError>;
}
