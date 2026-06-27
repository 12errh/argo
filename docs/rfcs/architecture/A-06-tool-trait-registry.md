# A-06: Tool Trait & Registry

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define the `Tool` trait, permission model, hot-reload protocol, and fallback registration for Argo's tool system.

## Motivation

Tools are the primary way agents interact with the outside world. Every tool must have a consistent interface, declared permissions, and support for hot-reload and fallbacks.

## Detailed Design

### Tool Trait

```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn output_schema(&self) -> Value;
    fn permissions(&self) -> ToolPermissions;
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError>;
}

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub run_id: String,
    pub working_dir: String,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissions {
    pub allow_filesystem: bool,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub working_directory: Option<String>,
    pub allowed_paths: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub max_execution_time: Duration,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ToolError {
    #[error("Permission denied: {resource}")]
    PermissionDenied { resource: String },

    #[error("Execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Timeout after {elapsed:?}")]
    Timeout { elapsed: Duration },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Output too large: {size} bytes exceeds limit")]
    OutputTooLarge { size: usize },
}
```

### Tool Registry

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    versions: HashMap<String, semver::Version>,
    fallbacks: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            versions: HashMap::new(),
            fallbacks: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name.clone(), tool);
        self.versions.insert(name, semver::Version::new(0, 1, 0));
    }

    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
        self.versions.remove(name);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn register_fallbacks(&mut self, tool_name: &str, fallbacks: Vec<String>) {
        self.fallbacks.insert(tool_name.to_string(), fallbacks);
    }

    pub fn get_fallbacks(&self, tool_name: &str) -> Vec<Arc<dyn Tool>> {
        self.fallbacks
            .get(tool_name)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.tools.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn hot_reload(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        if let Some(version) = self.versions.get_mut(&name) {
            *version = version.clone().increment();
        }
        self.tools.insert(name, tool);
    }

    pub fn list(&self) -> Vec<ToolInfo> {
        self.tools.iter().map(|(name, tool)| ToolInfo {
            name: name.clone(),
            description: tool.description().to_string(),
            version: self.versions.get(name).cloned().unwrap_or_else(|| semver::Version::new(0, 0, 0)),
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: semver::Version,
}
```

### Built-in Tools

```rust
pub struct BashTool {
    working_directory: String,
    max_execution_time: Duration,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Execute shell commands" }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "timeout": { "type": "integer", "description": "Timeout in seconds" }
            },
            "required": ["command"]
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: false,
            allow_subprocess: true,
            working_directory: Some(self.working_directory.clone()),
            allowed_paths: vec![],
            allowed_domains: vec![],
            max_execution_time: self.max_execution_time,
        }
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let command = input["command"].as_str()
            .ok_or_else(|| ToolError::InvalidInput { reason: "missing 'command' field".into() })?;
        // Validate working directory, execute command, capture output
        todo!()
    }
}

pub struct FilesTool {
    allowed_paths: Vec<String>,
}

pub struct HttpTool {
    allowed_domains: Vec<String>,
}
```

## Alternatives Considered

1. **Macro-based tool registration**: More ergonomic, but less flexible at runtime.
2. **Plugin system with dynamic loading**: Maximum extensibility, but adds complexity.
3. **Tool as a simple function**: Simpler, but loses permission model and hot-reload.

## Drawbacks

- Hot-reload requires careful state management
- Permission model adds overhead to every tool call
- Fallback selection logic can be complex

## Unresolved Questions

- Should tools support async streaming output?
- How to handle tool versioning across agents?
- Should we support tool composition (tool A calls tool B)?
