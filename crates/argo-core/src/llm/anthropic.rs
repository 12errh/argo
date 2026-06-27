use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{
    CompletionRequest, CompletionResponse, LlmProvider, MessageContent, Role, StopReason,
    ToolCallRequest, TokenUsage,
};
use crate::error::LlmError;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .map(|m| {
                let role = match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "user",
                    Role::System => "user",
                };
                let content = match m.content {
                    MessageContent::Text(t) => serde_json::Value::String(t),
                };
                AnthropicMessage {
                    role: role.to_string(),
                    content,
                }
            })
            .collect();

        let body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(8192),
            system: request.system_prompt,
            messages,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError {
                reason: e.to_string(),
            })?;

        let status = response.status();
        if status.as_u16() == 429 {
            return Err(LlmError::RateLimited {
                retry_after_ms: 60000,
            });
        }
        if status.as_u16() == 401 {
            return Err(LlmError::AuthenticationFailed {
                reason: "Invalid API key".to_string(),
            });
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError {
                status: status.as_u16(),
                message: text,
            });
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse {
                reason: e.to_string(),
            })?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for block in &anthropic_response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(t) = &block.text {
                        content.push_str(t);
                    }
                }
                "tool_use" => {
                    tool_calls.push(ToolCallRequest {
                        id: block.id.clone().unwrap_or_default(),
                        name: block.name.clone().unwrap_or_default(),
                        input: block.input.clone().unwrap_or(serde_json::Value::Null),
                    });
                }
                _ => {}
            }
        }

        let stop_reason = match anthropic_response.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("tool_use") => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        };

        Ok(CompletionResponse {
            content,
            tool_calls,
            usage: TokenUsage {
                input_tokens: anthropic_response.usage.input_tokens,
                output_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            },
            stop_reason,
            model: self.model.clone(),
        })
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<futures::stream::BoxStream<'static, super::Token>, LlmError> {
        Err(LlmError::StreamingError {
            reason: "Streaming not yet implemented".to_string(),
        })
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn context_limit(&self) -> usize {
        200_000
    }

    fn max_output_tokens(&self) -> usize {
        8192
    }
}
