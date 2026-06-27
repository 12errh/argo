use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::MemoryError;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn dimension(&self) -> usize;
    fn name(&self) -> &str;
}

pub struct OpenAIEmbedding {
    pub api_key: String,
    pub model: String,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "text-embedding-3-small".to_string(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIEmbeddingRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError> {
        let client = reqwest::Client::new();
        let request = OpenAIEmbeddingRequest {
            input: vec![text.to_string()],
            model: self.model.clone(),
        };

        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| MemoryError::ConnectionFailed(e.to_string()))?;

        let embedding_response: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;

        embedding_response
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| MemoryError::Serialization("No embedding data returned".to_string()))
    }

    fn dimension(&self) -> usize {
        1536
    }

    fn name(&self) -> &str {
        "openai"
    }
}

pub struct OllamaEmbedding {
    pub base_url: String,
    pub model: String,
}

impl OllamaEmbedding {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            model: "nomic-embed-text".to_string(),
        }
    }
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError> {
        let client = reqwest::Client::new();
        let request = OllamaEmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| MemoryError::ConnectionFailed(e.to_string()))?;

        let embedding_response: OllamaEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;

        Ok(embedding_response.embedding)
    }

    fn dimension(&self) -> usize {
        768
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

pub struct MockEmbedding {
    pub dimension: usize,
}

impl MockEmbedding {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError> {
        let mut embedding = vec![0.0; self.dimension];
        let bytes = text.as_bytes();
        for (i, val) in embedding.iter_mut().enumerate() {
            if i < bytes.len() {
                *val = bytes[i] as f32 / 255.0;
            } else {
                *val = (i as f32).sin();
            }
        }
        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_embedding_produces_correct_dimension() {
        let embedder = MockEmbedding::new(1536);
        let result = embedder.embed("test text").await.unwrap();
        assert_eq!(result.len(), 1536);
    }

    #[test]
    fn mock_embedding_name() {
        let embedder = MockEmbedding::new(1536);
        assert_eq!(embedder.name(), "mock");
    }

    #[test]
    fn mock_embedding_dimension() {
        let embedder = MockEmbedding::new(768);
        assert_eq!(embedder.dimension(), 768);
    }
}
