use async_trait::async_trait;

use crate::types::{HealContext, HealResult};
use argo_core::error::AgentError;

pub mod change_provider;
pub mod decompose;
pub mod reduce_scope;
pub mod reframe;
pub mod retry;
pub mod spawn_subagent;
pub mod swap_tool;

#[async_trait]
pub trait HealStrategy: Send + Sync {
    fn can_handle(&self, error: &AgentError) -> bool;
    async fn apply(&self, ctx: &HealContext) -> HealResult;
    fn name(&self) -> &str;
}

pub fn default_strategies() -> Vec<Box<dyn HealStrategy>> {
    vec![
        Box::new(retry::RetryStrategy::default()),
        Box::new(reframe::ReframeStrategy),
        Box::new(swap_tool::SwapToolStrategy),
        Box::new(decompose::DecomposeStrategy),
        Box::new(spawn_subagent::SpawnSubagentStrategy),
        Box::new(change_provider::ChangeProviderStrategy { providers: vec![] }),
        Box::new(reduce_scope::ReduceScopeStrategy),
    ]
}
