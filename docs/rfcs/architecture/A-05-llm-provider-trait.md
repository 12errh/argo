# A-05: LLM Provider Trait

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define the `LlmProvider` trait, request/response types, error types, and streaming contract for all LLM adapters.

## Motivation

Argo must support multiple LLM providers (Anthropic, OpenAI, Gemini, Ollama) with a unified interface. The trait isolates provider-specific logic and enables the heal engine to switch providers transparently.

## Detailed Design

### Provider Trait

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request and get a response
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Send a completion request and stream tokens
    async fn stream(&self, request: CompletionRequest) -> Result<BoxStream<'static, Token>, LlmError>;

    /// Provider name (e.g., "anthropic", "openai")
    fn provider_name(&self) -> &str;

    /// Model name (e.g., "claude-sonnet-4-6")
    fn model_name(&self) -> &str;

    /// Context window limit in tokens
    fn context_limit(&self) -> usize;

    /// Maximum output tokens
    fn max_output_tokens(&self) -> usize;
}
```

### Request Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    MultiPart(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text { text: String },
    Image { url: String, media_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
```

### Response Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCallRequest>,
    pub usage: TokenUsage,
    pub stop_reason: StopReason,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Text,
    ToolCallStart,
    ToolCallInput,
}
```

### Error Types

```rust
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum LlmError {
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Context overflow: {current} tokens, limit is {limit}")]
    ContextOverflow { current: usize, limit: usize },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },

    #[error("Request timed out after {elapsed_ms}ms")]
    Timeout { elapsed_ms: u64 },

    #[error("Provider error: {status} {message}")]
    ProviderError { status: u16, message: String },

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("Invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("Streaming error: {reason}")]
    StreamingError { reason: String },
}
```

### Adapter Implementations

**Anthropic Claude:**

```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": request.max_tokens.unwrap_or(8192),
            "system": request.system_prompt,
            "messages": serialize_messages(&request.messages),
            "tools": serialize_tools(&request.tools.unwrap_or_default()),
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError { reason: e.to_string() })?;

        parse_anthropic_response(response).await
    }

    fn provider_name(&self) -> &str { "anthropic" }
    fn model_name(&self) -> &str { &self.model }
    fn context_limit(&self) -> usize { 200_000 }
    fn max_output_tokens(&self) -> usize { 8192 }
}
```

**OpenAI:**

```rust
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": serialize_messages(&request.messages),
            "max_tokens": request.max_tokens,
            "tools": serialize_tools(&request.tools.unwrap_or_default()),
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError { reason: e.to_string() })?;

        parse_openai_response(response).await
    }

    fn provider_name(&self) -> &str { "openai" }
    fn model_name(&self) -> &str { &self.model }
    fn context_limit(&self) -> usize { 128_000 }
    fn max_output_tokens(&self) -> usize { 16384 }
}
```

### Provider Factory

```rust
pub fn create_provider(config: &ModelConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider.as_str() {
        "anthropic" => Ok(Box::new(AnthropicProvider::new(
            std::env::var("ANTHROPIC_API_KEY").map_err(|_| LlmError::AuthenticationFailed { reason: "ANTHROPIC_API_KEY not set".into() })?,
            config.model.clone(),
        ))),
        "openai" => Ok(Box::new(OpenAiProvider::new(
            std::env::var("OPENAI_API_KEY").map_err(|_| LlmError::AuthenticationFailed { reason: "OPENAI_API_KEY not set".into() })?,
            config.model.clone(),
        ))),
        "ollama" => Ok(Box::new(OllamaProvider::new(
            config.ollama_url.clone().unwrap_or_else(|| "http://localhost:11434".into()),
            config.model.clone(),
        ))),
        _ => Err(LlmError::ProviderError { status: 0, message: format!("Unknown provider: {}", config.provider) }),
    }
}
```

## Alternatives Considered

1. **REST API between SDKs and core**: Simpler, but adds latency and loses type safety.
2. **Generic HTTP client**: Maximum flexibility, but loses provider-specific optimizations.
3. **Provider-specific enums instead of trait**: Simpler for single provider, but doesn't scale.

## Drawbacks

- Each provider adapter must handle provider-specific quirks
- Streaming contract adds complexity
- Provider API changes require adapter updates

## Unresolved Questions

- Should providers support structured output (JSON mode) natively?
- How to handle provider-specific features (e.g., Anthropic's tool use vs OpenAI's function calling)?
- Should we support batch completion requests for cost optimization?
