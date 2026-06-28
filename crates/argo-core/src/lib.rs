//! Argo Core — Actor engine, message types, and agent execution.

pub mod actor;
pub mod config;
pub mod error;
pub mod execution;
pub mod llm;
pub mod loop_agent;
pub mod message;
pub mod orchestrator;
pub mod pool;
pub mod spawn;

#[cfg(test)]
pub mod tests;
