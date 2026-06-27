use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{
    CompletionRequest, CompletionResponse, LlmProvider, MessageContent, Role, StopReason,
    TokenUsage,
};
use crate::error::LlmError;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: Client,
    base_url: String,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let mut messages: Vec<OpenAiMessage> = Vec::new();

        if let Some(system) = &request.system_prompt {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: Some(system.clone()),
            });
        }

        for m in request.messages {
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "assistant",
            };
            let content = match m.content {
                MessageContent::Text(t) => Some(t),
            };
            messages.push(OpenAiMessage {
                role: role.to_string(),
                content,
            });
        }

        let body = OpenAiRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse {
                reason: e.to_string(),
            })?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::InvalidResponse {
                reason: "No choices in response".to_string(),
            })?;

        let content = choice.message.content.unwrap_or_default();

        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("tool_calls") => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        };

        Ok(CompletionResponse {
            content,
            tool_calls: Vec::new(),
            usage: TokenUsage {
                input_tokens: openai_response.usage.prompt_tokens,
                output_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
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
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn context_limit(&self) -> usize {
        128_000
    }

    fn max_output_tokens(&self) -> usize {
        16384
    }
}
