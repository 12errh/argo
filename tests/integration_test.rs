use std::sync::Arc;
use std::time::Duration;

use argo_core::config::AgentConfig;
use argo_core::llm::{
    CompletionRequest, CompletionResponse, LlmProvider, StopReason, TokenUsage,
};
use argo_core::message::TaskResult;
use argo_memory::handle::MemoryHandle;
use argo_memory::redis::RedisMemory;
use argo_memory::surreal::SurrealMemory;
use argo_tools::registry::ToolRegistry;
use argo_tools::trait_def::Tool;

struct MockLlmProvider {
    response: String,
}

#[async_trait::async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, argo_core::error::LlmError> {
        Ok(CompletionResponse {
            content: self.response.clone(),
            tool_calls: Vec::new(),
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            },
            stop_reason: StopReason::EndTurn,
            model: "mock-model".to_string(),
        })
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<futures::stream::BoxStream<'static, argo_core::llm::Token>, argo_core::error::LlmError>
    {
        unimplemented!()
    }

    fn provider_name(&self) -> &str { "mock" }
    fn model_name(&self) -> &str { "mock-model" }
    fn context_limit(&self) -> usize { 100_000 }
    fn max_output_tokens(&self) -> usize { 4096 }
}

struct MockTool;

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str { "mock_tool" }
    fn description(&self) -> &str { "A mock tool for testing" }
    fn input_schema(&self) -> serde_json::Value { serde_json::json!({}) }
    fn output_schema(&self) -> serde_json::Value { serde_json::json!({}) }
    fn permissions(&self) -> argo_tools::trait_def::ToolPermissions {
        argo_tools::trait_def::ToolPermissions {
            allow_filesystem: false,
            allow_network: false,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: Vec::new(),
            allowed_domains: Vec::new(),
            max_execution_time: Duration::from_secs(30),
        }
    }
    async fn execute(&self, _input: serde_json::Value, _ctx: &argo_tools::trait_def::ToolContext) -> Result<serde_json::Value, argo_tools::error::ToolError> {
        Ok(serde_json::json!({"result": "mock output"}))
    }
}

#[tokio::test]
async fn test_agent_with_mock_llm() {
    let llm = Arc::new(MockLlmProvider {
        response: "Hello from mock agent".to_string(),
    });

    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(MockTool));
    let tools = Arc::new(tools);

    let redis = RedisMemory::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");
    let surreal = SurrealMemory::new("http://127.0.0.1:8000", "argo", "test");
    let memory = Arc::new(MemoryHandle::new(redis, surreal));

    let config = AgentConfig {
        agent: argo_core::config::AgentSection {
            name: "test-agent".to_string(),
            version: Some("0.1.0".to_string()),
            description: None,
        },
        model: argo_core::config::ModelSection {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            api_key: None,
            temperature: None,
            max_tokens: None,
            context_strategy: None,
        },
        memory: None,
        heal: None,
        quality: None,
        tools: argo_core::config::ToolsSection {
            enabled: vec!["mock_tool".to_string()],
        },
        permissions: argo_core::config::PermissionsSection {
            allow_network: false,
            allow_filesystem: false,
            allowed_paths: None,
            max_execution_time: None,
        },
        observe: None,
    };

    let result = argo_core::execution::execute_task(
        "Test goal",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    assert!(result.is_ok());
    match result.unwrap() {
        TaskResult::Success { output } => {
            assert_eq!(output, "Hello from mock agent");
        }
        _ => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_agent_handles_llm_error() {
    struct FailingLlm;

    #[async_trait::async_trait]
    impl LlmProvider for FailingLlm {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, argo_core::error::LlmError> {
            Err(argo_core::error::LlmError::NetworkError {
                reason: "Simulated network failure".to_string(),
            })
        }
        async fn stream(&self, _request: CompletionRequest) -> Result<futures::stream::BoxStream<'static, argo_core::llm::Token>, argo_core::error::LlmError> {
            unimplemented!()
        }
        fn provider_name(&self) -> &str { "failing" }
        fn model_name(&self) -> &str { "failing-model" }
        fn context_limit(&self) -> usize { 100_000 }
        fn max_output_tokens(&self) -> usize { 4096 }
    }

    let llm = Arc::new(FailingLlm);
    let tools = Arc::new(ToolRegistry::new());
    let redis = RedisMemory::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");
    let surreal = SurrealMemory::new("http://127.0.0.1:8000", "argo", "test");
    let memory = Arc::new(MemoryHandle::new(redis, surreal));
    let config = AgentConfig {
        agent: argo_core::config::AgentSection {
            name: "test-agent".to_string(),
            version: None,
            description: None,
        },
        model: argo_core::config::ModelSection {
            provider: "failing".to_string(),
            model: "failing-model".to_string(),
            api_key: None,
            temperature: None,
            max_tokens: None,
            context_strategy: None,
        },
        memory: None,
        heal: None,
        quality: None,
        tools: argo_core::config::ToolsSection {
            enabled: Vec::new(),
        },
        permissions: argo_core::config::PermissionsSection {
            allow_network: false,
            allow_filesystem: false,
            allowed_paths: None,
            max_execution_time: None,
        },
        observe: None,
    };

    let result = argo_core::execution::execute_task(
        "Test goal",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    assert!(result.is_err());
}
