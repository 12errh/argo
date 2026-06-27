use actix::{Actor, Context, Handler, ResponseFuture};
use std::sync::Arc;

use crate::config::AgentConfig;
use crate::execution::execute_task;
use crate::llm::LlmProvider;
use crate::message::{ExecuteTask, TaskResult};
use argo_memory::handle::MemoryHandle;
use argo_tools::registry::ToolRegistry;

pub struct AgentActor {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryHandle>,
}

impl AgentActor {
    pub fn new(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        memory: Arc<MemoryHandle>,
    ) -> Self {
        Self {
            config,
            llm,
            tools,
            memory,
        }
    }
}

impl Actor for AgentActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteTask> for AgentActor {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: ExecuteTask, _ctx: &mut Self::Context) -> Self::Result {
        let llm = self.llm.clone();
        let tools = self.tools.clone();
        let memory = self.memory.clone();
        let config = self.config.clone();
        let goal = msg.goal;

        Box::pin(async move {
            match execute_task(
                &goal,
                llm.as_ref(),
                tools.as_ref(),
                memory.as_ref(),
                &config,
            )
            .await
            {
                Ok(result) => result,
                Err(e) => TaskResult::Failed { error: e },
            }
        })
    }
}
