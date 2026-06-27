use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MemoryError {
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("SurrealDB error: {0}")]
    Surreal(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}
