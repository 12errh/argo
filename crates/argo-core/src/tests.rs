use crate::error::AgentError;
use std::time::Duration;

#[test]
fn test_agent_error_display() {
    let err = AgentError::LlmRateLimit {
        retry_after: Duration::from_secs(5),
        provider: "anthropic".to_string(),
    };
    assert!(err.to_string().contains("rate limited"));
}

#[test]
fn test_llm_error_to_agent_error() {
    let llm_err = crate::error::LlmError::RateLimited {
        retry_after_ms: 5000,
    };
    let agent_err: AgentError = llm_err.into();
    match agent_err {
        AgentError::LlmRateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Duration::from_millis(5000));
        }
        _ => panic!("Expected LlmRateLimit"),
    }
}

#[test]
fn test_completion_request_serialization() {
    use crate::llm::{CompletionRequest, Message, MessageContent, Role};

    let req = CompletionRequest {
        messages: vec![Message {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
        }],
        system_prompt: Some("You are helpful".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1024),
        stop_sequences: None,
        tools: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("Hello"));
    assert!(json.contains("You are helpful"));
}

#[test]
fn test_config_validation() {
    use crate::config::*;
    let config = AgentConfig {
        agent: AgentSection {
            name: "test".to_string(),
            version: None,
            description: None,
        },
        model: ModelSection {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            api_key: None,
            temperature: Some(0.5),
            max_tokens: Some(4096),
            context_strategy: None,
        },
        memory: None,
        heal: None,
        quality: None,
        tools: ToolsSection {
            enabled: vec!["bash".to_string()],
        },
        permissions: PermissionsSection {
            allow_network: false,
            allow_filesystem: true,
            allowed_paths: None,
            max_execution_time: None,
        },
        observe: None,
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_bad_provider() {
    use crate::config::*;
    let config = AgentConfig {
        agent: AgentSection {
            name: "test".to_string(),
            version: None,
            description: None,
        },
        model: ModelSection {
            provider: "invalid".to_string(),
            model: "model".to_string(),
            api_key: None,
            temperature: None,
            max_tokens: None,
            context_strategy: None,
        },
        memory: None,
        heal: None,
        quality: None,
        tools: ToolsSection {
            enabled: vec!["bash".to_string()],
        },
        permissions: PermissionsSection {
            allow_network: false,
            allow_filesystem: true,
            allowed_paths: None,
            max_execution_time: None,
        },
        observe: None,
    };
    assert!(config.validate().is_err());
}
