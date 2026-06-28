use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tracing::info;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::execution::execute_task;
use crate::llm::LlmProvider;
use crate::message::TaskResult;
use argo_memory::handle::MemoryHandle;
use argo_tools::registry::ToolRegistry;

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryMode {
    Shared,
    Isolated,
    Persistent,
}

impl MemoryMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "shared" => MemoryMode::Shared,
            "isolated" => MemoryMode::Isolated,
            "persistent" => MemoryMode::Persistent,
            _ => MemoryMode::Persistent,
        }
    }
}

pub struct AgentPool {
    workers: usize,
    agent_template: AgentConfig,
    llm_providers: Vec<Arc<dyn LlmProvider>>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryHandle>,
    memory_mode: MemoryMode,
    next_worker: AtomicUsize,
    task_queue: VecDeque<(String, tokio::sync::oneshot::Sender<TaskResult>)>,
}

impl AgentPool {
    pub fn new(
        workers: usize,
        agent_template: AgentConfig,
        llm_providers: Vec<Arc<dyn LlmProvider>>,
        tools: Arc<ToolRegistry>,
        memory: Arc<MemoryHandle>,
        memory_mode: MemoryMode,
    ) -> Self {
        Self {
            workers,
            agent_template,
            llm_providers,
            tools,
            memory,
            memory_mode,
            next_worker: AtomicUsize::new(0),
            task_queue: VecDeque::new(),
        }
    }

    fn next_llm_provider(&self) -> Arc<dyn LlmProvider> {
        let idx = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.llm_providers.len();
        self.llm_providers[idx].clone()
    }

    pub async fn run_task(&self, goal: &str) -> Result<TaskResult, AgentError> {
        let llm = self.next_llm_provider();
        let config = self.agent_template.clone();

        info!(
            "Pool dispatching task '{}' to worker (mode: {:?})",
            goal, self.memory_mode
        );

        execute_task(goal, llm.as_ref(), self.tools.as_ref(), self.memory.as_ref(), &config).await
    }

    pub async fn map(&self, goals: Vec<String>) -> Vec<Result<TaskResult, AgentError>> {
        let mut handles = Vec::new();

        for goal in goals {
            let llm = self.next_llm_provider();
            let tools = self.tools.clone();
            let memory = self.memory.clone();
            let config = self.agent_template.clone();
            let goal = goal.clone();

            let handle = tokio::spawn(async move {
                execute_task(
                    &goal,
                    llm.as_ref(),
                    tools.as_ref(),
                    memory.as_ref(),
                    &config,
                )
                .await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(AgentError::OrchestratorFailed {
                    reason: format!("Worker task panicked: {}", e),
                })),
            }
        }
        results
    }

    pub fn worker_count(&self) -> usize {
        self.workers
    }

    pub fn memory_mode(&self) -> &MemoryMode {
        &self.memory_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_mode_from_str() {
        assert_eq!(MemoryMode::from_str("shared"), MemoryMode::Shared);
        assert_eq!(MemoryMode::from_str("isolated"), MemoryMode::Isolated);
        assert_eq!(MemoryMode::from_str("persistent"), MemoryMode::Persistent);
        assert_eq!(MemoryMode::from_str("unknown"), MemoryMode::Persistent);
    }

    #[test]
    fn test_memory_mode_case_insensitive() {
        assert_eq!(MemoryMode::from_str("SHARED"), MemoryMode::Shared);
        assert_eq!(MemoryMode::from_str("Isolated"), MemoryMode::Isolated);
    }
}
