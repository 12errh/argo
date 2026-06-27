use std::collections::HashMap;
use std::sync::Arc;

use crate::trait_def::Tool;

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: semver::Version,
}

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
        self.versions
            .insert(name, semver::Version::new(0, 1, 0));
    }

    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
        self.versions.remove(name);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn register_fallbacks(&mut self, tool_name: &str, fallbacks: Vec<String>) {
        self.fallbacks
            .insert(tool_name.to_string(), fallbacks);
    }

    pub fn get_fallbacks(&self, tool_name: &str) -> Vec<Arc<dyn Tool>> {
        self.fallbacks
            .get(tool_name)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.tools.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn hot_reload(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        if let Some(version) = self.versions.get_mut(&name) {
            version.patch += 1;
        }
        self.tools.insert(name, tool);
    }

    pub fn list(&self) -> Vec<ToolInfo> {
        self.tools
            .iter()
            .map(|(name, tool)| ToolInfo {
                name: name.clone(),
                description: tool.description().to_string(),
                version: self
                    .versions
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| semver::Version::new(0, 0, 0)),
            })
            .collect()
    }
}
