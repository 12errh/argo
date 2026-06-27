use serde::Deserialize;
use std::path::Path;

use crate::error::AgentError;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub agent: AgentSection,
    pub model: ModelSection,
    pub memory: Option<MemorySection>,
    pub heal: Option<HealSection>,
    pub quality: Option<QualitySection>,
    pub tools: ToolsSection,
    pub permissions: PermissionsSection,
    pub observe: Option<ObserveSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentSection {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelSection {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub context_strategy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemorySection {
    pub mode: Option<String>,
    pub short_term_ttl: Option<u64>,
    pub long_term_backend: Option<String>,
    pub vector_backend: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealSection {
    pub enabled: Option<bool>,
    pub max_attempts: Option<usize>,
    pub strategies: Option<Vec<String>>,
    pub background: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QualitySection {
    pub threshold: Option<f32>,
    pub max_iterations: Option<usize>,
    pub criteria: Option<Vec<QualityCriterion>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QualityCriterion {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolsSection {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionsSection {
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allowed_paths: Option<Vec<String>>,
    pub max_execution_time: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObserveSection {
    pub enabled: Option<bool>,
    pub backend: Option<String>,
    pub endpoint: Option<String>,
}

fn resolve_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start + 2..].find('}') {
            let var_name = &result[start + 2..start + 2 + end];
            if let Ok(value) = std::env::var(var_name) {
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    value,
                    &result[start + 2 + end + 1..]
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

impl AgentConfig {
    pub fn from_file(path: &Path) -> Result<Self, AgentError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AgentError::Config(format!("Failed to read config file: {}", e)))?;

        let resolved = resolve_env_vars(&content);

        let config: AgentConfig = toml::from_str(&resolved)
            .map_err(|e| AgentError::Config(format!("Failed to parse TOML: {}", e)))?;

        config.validate()?;

        Ok(config)
    }

    pub(crate) fn validate(&self) -> Result<(), AgentError> {
        let valid_providers = ["anthropic", "openai", "gemini", "ollama", "custom"];
        if !valid_providers.contains(&self.model.provider.as_str()) {
            return Err(AgentError::Config(format!(
                "Invalid provider: {}. Must be one of: {:?}",
                self.model.provider, valid_providers
            )));
        }

        if let Some(temp) = self.model.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(AgentError::Config(format!(
                    "Temperature must be between 0.0 and 2.0, got {}",
                    temp
                )));
            }
        }

        if let Some(tokens) = self.model.max_tokens {
            if tokens == 0 {
                return Err(AgentError::Config(
                    "max_tokens must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(quality) = &self.quality {
            if let Some(threshold) = quality.threshold {
                if !(0.0..=1.0).contains(&threshold) {
                    return Err(AgentError::Config(format!(
                        "Quality threshold must be between 0.0 and 1.0, got {}",
                        threshold
                    )));
                }
            }
        }

        if self.tools.enabled.is_empty() {
            return Err(AgentError::Config(
                "tools.enabled must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}
