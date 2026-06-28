use napi_derive::napi;
use serde_json::Value as JsonValue;

use crate::util;

#[napi]
pub struct Agent {
    config: argo_core::config::AgentConfig,
}

#[napi]
impl Agent {
    #[napi(factory)]
    pub fn new(
        name: String,
        model: String,
        provider: String,
        api_key: String,
        tools: Option<Vec<String>>,
    ) -> Self {
        let tool_names = tools.unwrap_or_else(|| vec!["bash".to_string(), "files".to_string()]);
        let config = util::build_config(&name, &model, &provider, &api_key, tool_names);
        Self { config }
    }

    #[napi(factory)]
    pub fn from_config(path: String) -> napi::Result<Self> {
        let config_path = std::path::Path::new(&path);
        let config = argo_core::config::AgentConfig::from_file(config_path)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(Self { config })
    }

    #[napi]
    pub async fn run(&self, goal: String) -> napi::Result<JsonValue> {
        let llm = util::create_provider(&self.config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let tool_registry = util::create_tool_registry(&self.config.tools.enabled, &self.config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let memory = util::create_memory(&self.config)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let result = argo_core::execution::execute_task(
            &goal,
            llm.as_ref(),
            &tool_registry,
            &memory,
            &self.config,
        )
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        match result {
            argo_core::message::TaskResult::Success { output } => {
                Ok(serde_json::json!({"success": true, "output": output}))
            }
            argo_core::message::TaskResult::Partial { output, reason } => {
                Ok(serde_json::json!({"success": false, "output": output, "reason": reason}))
            }
            argo_core::message::TaskResult::Failed { error } => {
                Ok(serde_json::json!({"success": false, "error": error.to_string()}))
            }
        }
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.config.agent.name.clone()
    }

    #[napi(getter)]
    pub fn model(&self) -> String {
        self.config.model.model.clone()
    }

    #[napi(getter)]
    pub fn provider(&self) -> String {
        self.config.model.provider.clone()
    }

    #[napi(getter)]
    pub fn tools(&self) -> Vec<String> {
        self.config.tools.enabled.clone()
    }
}

impl Agent {
    pub(crate) fn get_config(&self) -> &argo_core::config::AgentConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new(
            "test-agent".to_string(),
            "claude-sonnet-4-6".to_string(),
            "anthropic".to_string(),
            "test-key".to_string(),
            Some(vec!["bash".to_string(), "files".to_string()]),
        );
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.model(), "claude-sonnet-4-6");
        assert_eq!(agent.provider(), "anthropic");
        assert_eq!(agent.tools(), vec!["bash", "files"]);
    }

    #[test]
    fn test_agent_default_tools() {
        let agent = Agent::new(
            "test-agent".to_string(),
            "gpt-4".to_string(),
            "openai".to_string(),
            "test-key".to_string(),
            None,
        );
        assert_eq!(agent.tools(), vec!["bash", "files"]);
    }

    #[test]
    fn test_loop_agent_new() {
        let agent = LoopAgent::new(
            "loop-agent".to_string(),
            "claude-sonnet-4-6".to_string(),
            "anthropic".to_string(),
            "test-key".to_string(),
            Some(0.9),
            Some(10),
            None,
        );
        assert_eq!(agent.name(), "loop-agent");
        assert!((agent.threshold() - 0.9).abs() < 0.001);
        assert_eq!(agent.max_iterations(), 10);
    }

    #[test]
    fn test_loop_agent_defaults() {
        let agent = LoopAgent::new(
            "loop-agent".to_string(),
            "gpt-4".to_string(),
            "openai".to_string(),
            "test-key".to_string(),
            None,
            None,
            None,
        );
        assert!((agent.threshold() - 0.85).abs() < 0.001);
        assert_eq!(agent.max_iterations(), 20);
    }
}

#[napi]
pub struct LoopAgent {
    config: argo_core::config::AgentConfig,
    threshold: f32,
    max_iterations: usize,
}

#[napi]
impl LoopAgent {
    #[napi(factory)]
    pub fn new(
        name: String,
        model: String,
        provider: String,
        api_key: String,
        threshold: Option<f64>,
        max_iterations: Option<u32>,
        tools: Option<Vec<String>>,
    ) -> Self {
        let tool_names = tools.unwrap_or_else(|| vec!["bash".to_string(), "files".to_string()]);
        let mut config = util::build_config(&name, &model, &provider, &api_key, tool_names);

        let threshold_val = threshold.unwrap_or(0.85) as f32;
        let max_iter_val = max_iterations.unwrap_or(20) as usize;

        config.quality = Some(argo_core::config::QualitySection {
            threshold: Some(threshold_val),
            max_iterations: Some(max_iter_val),
            criteria: None,
        });

        Self {
            config,
            threshold: threshold_val,
            max_iterations: max_iter_val,
        }
    }

    #[napi]
    pub async fn run(&self, goal: String) -> napi::Result<JsonValue> {
        let llm = util::create_provider(&self.config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let tool_registry = util::create_tool_registry(&self.config.tools.enabled, &self.config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let memory = util::create_memory(&self.config)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let mut best_score: f32 = 0.0;
        let mut best_output = String::new();
        let mut final_iteration = self.max_iterations;

        for iteration in 1..=self.max_iterations {
            let task_result = argo_core::execution::execute_task(
                &goal,
                llm.as_ref(),
                &tool_registry,
                &memory,
                &self.config,
            )
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            if let argo_core::message::TaskResult::Success { output } = task_result {
                    let score = util::estimate_quality(&output);
                    if score > best_score {
                        best_score = score;
                        best_output = output;
                    }
                    if score >= self.threshold {
                        final_iteration = iteration;
                        break;
                    }
                }
        }

        Ok(serde_json::json!({
            "output": best_output,
            "score": best_score,
            "iterations": final_iteration,
            "thresholdReached": best_score >= self.threshold,
        }))
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.config.agent.name.clone()
    }

    #[napi(getter)]
    pub fn threshold(&self) -> f64 {
        self.threshold as f64
    }

    #[napi(getter)]
    pub fn max_iterations(&self) -> u32 {
        self.max_iterations as u32
    }
}
