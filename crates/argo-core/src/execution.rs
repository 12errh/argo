use std::time::Instant;
use uuid::Uuid;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::llm::{
    CompletionRequest, LlmProvider, Message, MessageContent, Role, ToolDefinition,
};
use crate::message::{AgentTrace, LlmCallRecord, TaskResult, ToolCallRecord};
use argo_memory::handle::MemoryHandle;
use argo_memory::surreal::TaskRecord;
use argo_tools::registry::ToolRegistry;
use argo_tools::trait_def::ToolContext;

const MAX_ITERATIONS: usize = 20;

pub async fn execute_task(
    goal: &str,
    llm: &dyn LlmProvider,
    tools: &ToolRegistry,
    memory: &MemoryHandle,
    config: &AgentConfig,
) -> Result<TaskResult, AgentError> {
    let run_id = Uuid::new_v4();
    let agent_id = &config.agent.name;
    let start = Instant::now();
    let mut trace = AgentTrace {
        run_id,
        agent_name: agent_id.clone(),
        goal: goal.to_string(),
        started_at: chrono::Utc::now(),
        ended_at: None,
        duration_ms: None,
        success: false,
        output: None,
        iterations: 0,
        quality_score: None,
        tool_calls: Vec::new(),
        llm_calls: Vec::new(),
        heal_steps: Vec::new(),
        lessons: Vec::new(),
        errors: Vec::new(),
    };

    let system_prompt = format!(
        "You are an AI agent named {}. Your goal: {}\n\nYou have access to tools. When you need to use a tool, respond with a tool call. When you are done, respond with your final answer.",
        agent_id, goal
    );

    let mut messages = vec![Message {
        role: Role::User,
        content: MessageContent::Text(goal.to_string()),
    }];

    let tool_defs: Vec<ToolDefinition> = tools
        .list()
        .iter()
        .map(|info| ToolDefinition {
            name: info.name.clone(),
            description: info.description.clone(),
            input_schema: serde_json::json!({}),
        })
        .collect();

    let ctx = ToolContext {
        agent_id: agent_id.clone(),
        run_id: run_id.to_string(),
        working_dir: std::env::current_dir()
            .unwrap_or_default()
            .display()
            .to_string(),
        environment: std::env::vars().collect(),
    };

    for iteration in 0..MAX_ITERATIONS {
        trace.iterations = iteration + 1;

        let request = CompletionRequest {
            messages: messages.clone(),
            system_prompt: Some(system_prompt.clone()),
            temperature: config.model.temperature,
            max_tokens: config.model.max_tokens,
            stop_sequences: None,
            tools: if tool_defs.is_empty() {
                None
            } else {
                Some(tool_defs.clone())
            },
        };

        let llm_start = Instant::now();
        let response = llm.complete(request).await.map_err(|e| {
            trace.errors.push(AgentError::from(e.clone()));
            AgentError::from(e)
        })?;
        let llm_duration = llm_start.elapsed().as_millis() as u64;

        trace.llm_calls.push(LlmCallRecord {
            provider: llm.provider_name().to_string(),
            model: llm.model_name().to_string(),
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
            duration_ms: llm_duration,
            timestamp: chrono::Utc::now(),
        });

        if !response.tool_calls.is_empty() {
            messages.push(Message {
                role: Role::Assistant,
                content: MessageContent::Text(response.content.clone()),
            });

            for tool_call in &response.tool_calls {
                let tool_start = Instant::now();
                let result = match tools.get(&tool_call.name) {
                    Some(tool) => tool.execute(tool_call.input.clone(), &ctx).await,
                    None => Err(argo_tools::error::ToolError::ExecutionFailed {
                        reason: format!("Tool '{}' not found", tool_call.name),
                    }),
                };
                let tool_duration = tool_start.elapsed().as_millis() as u64;

                let (success, output) = match result {
                    Ok(val) => (true, val),
                    Err(e) => (false, serde_json::json!({"error": e.to_string()})),
                };

                trace.tool_calls.push(ToolCallRecord {
                    call_id: Uuid::new_v4(),
                    tool_name: tool_call.name.clone(),
                    input: tool_call.input.clone(),
                    output: Some(output.clone()),
                    success,
                    duration_ms: tool_duration,
                    timestamp: chrono::Utc::now(),
                });

                messages.push(Message {
                    role: Role::Tool,
                    content: MessageContent::Text(
                        serde_json::to_string(&output).unwrap_or_default(),
                    ),
                });
            }
        } else {
            trace.success = true;
            trace.output = Some(response.content.clone());
            trace.ended_at = Some(chrono::Utc::now());
            trace.duration_ms = Some(start.elapsed().as_millis() as u64);

            let task_record = TaskRecord {
                agent_id: agent_id.clone(),
                goal: goal.to_string(),
                outcome: "success".to_string(),
                summary: response.content.clone(),
                tools_used: trace.tool_calls.iter().map(|tc| tc.tool_name.clone()).collect(),
                duration_ms: start.elapsed().as_millis() as i64,
                run_id: run_id.to_string(),
                started_at: trace.started_at.to_rfc3339(),
                ended_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = memory.store_task_record(&task_record).await;

            return Ok(TaskResult::Success {
                output: response.content,
            });
        }
    }

    trace.ended_at = Some(chrono::Utc::now());
    trace.duration_ms = Some(start.elapsed().as_millis() as u64);

    Ok(TaskResult::Failed {
        error: AgentError::InfiniteLoop {
            iteration_count: MAX_ITERATIONS,
        },
    })
}
