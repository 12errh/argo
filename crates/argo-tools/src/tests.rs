use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::ToolError;
use crate::registry::ToolRegistry;
use crate::trait_def::{Tool, ToolContext, ToolPermissions};

struct MockTool;

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock"
    }
    fn description(&self) -> &str {
        "Mock tool"
    }
    fn input_schema(&self) -> Value {
        json!({})
    }
    fn output_schema(&self) -> Value {
        json!({})
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: false,
            allow_network: false,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: Vec::new(),
            allowed_domains: Vec::new(),
            max_execution_time: Duration::from_secs(30),
        }
    }
    async fn execute(&self, _input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        Ok(json!({"result": "ok"}))
    }
}

fn mock_ctx() -> ToolContext {
    ToolContext {
        agent_id: "test-agent".to_string(),
        run_id: "test-run".to_string(),
        working_dir: ".".to_string(),
        environment: HashMap::new(),
    }
}

#[test]
fn test_registry_register_and_get() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool));
    assert!(registry.get("mock").is_some());
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_registry_list() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool));
    let list = registry.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "mock");
}

#[test]
fn test_registry_unregister() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool));
    registry.unregister("mock");
    assert!(registry.get("mock").is_none());
}

#[test]
fn test_registry_hot_reload() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool));
    registry.hot_reload(Arc::new(MockTool));
    let list = registry.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].version, semver::Version::new(0, 1, 1));
}

#[test]
fn test_registry_fallbacks() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool));
    registry.register_fallbacks("mock", vec!["fallback_a".to_string(), "mock".to_string()]);
    let fallbacks = registry.get_fallbacks("mock");
    assert_eq!(fallbacks.len(), 1);
    assert_eq!(fallbacks[0].name(), "mock");
}

#[tokio::test]
async fn test_mock_tool_execute() {
    let tool = MockTool;
    let ctx = mock_ctx();
    let result = tool.execute(json!({}), &ctx).await.unwrap();
    assert_eq!(result, json!({"result": "ok"}));
}
