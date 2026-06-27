//! Argo Core — Actor engine, message types, and agent execution.

pub mod actor;
pub mod config;
pub mod error;
pub mod execution;
pub mod llm;
pub mod message;

#[cfg(test)]
pub mod tests;
