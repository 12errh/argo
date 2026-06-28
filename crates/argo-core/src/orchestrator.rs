use std::collections::HashMap;
use std::sync::Arc;

use actix::{Actor, ActorFutureExt, AsyncContext, Context, Handler, ResponseFuture, WrapFuture};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::execution::execute_task;
use crate::llm::{CompletionRequest, LlmProvider, Message, MessageContent, Role};
use crate::message::{AssignTask, TaskResult};
use argo_memory::handle::MemoryHandle;
use argo_tools::registry::ToolRegistry;

#[derive(Debug, Clone)]
pub struct SubTask {
    pub task_id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub status: SubTaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubTaskStatus {
    Pending,
    Assigned,
    Completed,
    Failed,
}

pub struct OrchestratorActor {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryHandle>,
    sub_tasks: HashMap<Uuid, SubTask>,
    results: HashMap<Uuid, TaskResult>,
}

impl OrchestratorActor {
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
            sub_tasks: HashMap::new(),
            results: HashMap::new(),
        }
    }

    async fn decompose_goal(llm: &dyn LlmProvider, goal: &str) -> Result<Vec<String>, AgentError> {
        let prompt = format!(
            "You are a task decomposer. Break the following goal into 2-5 independent sub-tasks. \
             Return ONLY a JSON array of strings, each being a sub-task description. \
             Goal: {}",
            goal
        );

        let request = CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text(prompt),
            }],
            system_prompt: Some(
                "You are a task planner. Return ONLY valid JSON arrays.".to_string(),
            ),
            temperature: Some(0.3),
            max_tokens: Some(1024),
            stop_sequences: None,
            tools: None,
        };

        let response = llm.complete(request).await.map_err(AgentError::from)?;
        let sub_tasks: Vec<String> =
            serde_json::from_str(&response.content).unwrap_or_else(|_| vec![goal.to_string()]);
        Ok(sub_tasks)
    }

    #[allow(dead_code)]
    fn aggregate_results(&self) -> TaskResult {
        if self.results.is_empty() {
            return TaskResult::Failed {
                error: AgentError::OrchestratorFailed {
                    reason: "No results collected".to_string(),
                },
            };
        }

        let mut outputs = Vec::new();
        let mut all_success = true;
        let mut any_partial = false;

        for result in self.results.values() {
            match result {
                TaskResult::Success { output } => outputs.push(output.clone()),
                TaskResult::Partial { output, reason } => {
                    outputs.push(format!("[partial: {}] {}", reason, output));
                    any_partial = true;
                }
                TaskResult::Failed { error } => {
                    outputs.push(format!("[failed: {}]", error));
                    all_success = false;
                }
            }
        }

        let combined = outputs.join("\n\n");

        if all_success {
            TaskResult::Success { output: combined }
        } else if any_partial || !outputs.is_empty() {
            TaskResult::Partial {
                output: combined,
                reason: "Some sub-tasks failed or returned partial results".to_string(),
            }
        } else {
            TaskResult::Failed {
                error: AgentError::OrchestratorFailed {
                    reason: "All sub-tasks failed".to_string(),
                },
            }
        }
    }
}

impl Actor for OrchestratorActor {
    type Context = Context<Self>;
}

impl Handler<AssignTask> for OrchestratorActor {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: AssignTask, _ctx: &mut Self::Context) -> Self::Result {
        let llm = self.llm.clone();
        let tools = self.tools.clone();
        let memory = self.memory.clone();
        let config = self.config.clone();
        let goal = msg.goal.clone();

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

impl OrchestratorActor {
    pub fn decompose_and_assign(&mut self, goal: String, ctx: &mut Context<Self>) {
        let llm = self.llm.clone();
        let goal_clone = goal.clone();
        let fut =
            async move { Self::decompose_goal(llm.as_ref(), &goal_clone).await }
                .into_actor(self)
                .map(|result, actor, ctx| match result {
                    Ok(sub_goals) => {
                        let mut sub_tasks = Vec::new();
                        for sg in &sub_goals {
                            let task_id = Uuid::new_v4();
                            actor.sub_tasks.insert(
                                task_id,
                                SubTask {
                                    task_id,
                                    goal: sg.clone(),
                                    context: None,
                                    status: SubTaskStatus::Pending,
                                },
                            );
                            sub_tasks.push((task_id, sg.clone()));
                        }

                        info!(
                            "Decomposed into {} sub-tasks, spawning workers",
                            sub_tasks.len()
                        );

                        for (task_id, sub_goal) in sub_tasks {
                            let llm = actor.llm.clone();
                            let tools = actor.tools.clone();
                            let memory = actor.memory.clone();
                            let config = actor.config.clone();

                            let worker_fut = async move {
                                execute_task(
                                    &sub_goal,
                                    llm.as_ref(),
                                    tools.as_ref(),
                                    memory.as_ref(),
                                    &config,
                                )
                                .await
                            };

                            ctx.spawn(worker_fut.into_actor(actor).map(
                                move |result, actor, _ctx| {
                                    if let Some(sub_task) = actor.sub_tasks.get_mut(&task_id) {
                                        match &result {
                                            Ok(_) => sub_task.status = SubTaskStatus::Completed,
                                            Err(_) => sub_task.status = SubTaskStatus::Failed,
                                        }
                                    }
                                    actor.results.insert(
                                        task_id,
                                        result.unwrap_or_else(|e| TaskResult::Failed { error: e }),
                                    );
                                },
                            ));
                        }
                    }
                    Err(e) => {
                        warn!("Task decomposition failed: {}", e);
                    }
                });

        ctx.spawn(fut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_task_status_default() {
        let st = SubTask {
            task_id: Uuid::new_v4(),
            goal: "test".to_string(),
            context: None,
            status: SubTaskStatus::Pending,
        };
        assert_eq!(st.status, SubTaskStatus::Pending);
    }

    #[test]
    fn test_sub_task_transitions() {
        let mut st = SubTask {
            task_id: Uuid::new_v4(),
            goal: "test".to_string(),
            context: None,
            status: SubTaskStatus::Pending,
        };
        st.status = SubTaskStatus::Assigned;
        assert_eq!(st.status, SubTaskStatus::Assigned);
        st.status = SubTaskStatus::Completed;
        assert_eq!(st.status, SubTaskStatus::Completed);
    }
}
