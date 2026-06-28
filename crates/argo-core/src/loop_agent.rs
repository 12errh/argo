use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::execution::execute_task;
use crate::llm::{CompletionRequest, LlmProvider, Message, MessageContent, Role};
use crate::message::{
    AgentTrace, CriterionScore, QualityRubric, QualityScore, TaskResult,
};
use argo_memory::handle::MemoryHandle;
use argo_tools::registry::ToolRegistry;

const DEFAULT_MAX_ITERATIONS: usize = 20;
const DEFAULT_THRESHOLD: f32 = 0.85;

pub struct LoopAgent {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    memory: Arc<MemoryHandle>,
    rubric: QualityRubric,
}

impl LoopAgent {
    pub fn new(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        memory: Arc<MemoryHandle>,
        rubric: QualityRubric,
    ) -> Self {
        Self {
            config,
            llm,
            tools,
            memory,
            rubric,
        }
    }

    pub fn from_config(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        memory: Arc<MemoryHandle>,
    ) -> Self {
        let rubric = config
            .quality
            .as_ref()
            .map(|q| QualityRubric {
                criteria: q
                    .criteria
                    .as_ref()
                    .map(|cs| {
                        cs.iter()
                            .map(|c| crate::message::QualityCriterion {
                                name: c.name.clone(),
                                weight: c.weight,
                                description: c.description.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                threshold: q.threshold.unwrap_or(DEFAULT_THRESHOLD),
                max_iterations: q.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS),
            })
            .unwrap_or_else(|| QualityRubric {
                criteria: vec![],
                threshold: DEFAULT_THRESHOLD,
                max_iterations: DEFAULT_MAX_ITERATIONS,
            });

        Self::new(config, llm, tools, memory, rubric)
    }

    pub async fn run(&self, goal: &str) -> Result<(TaskResult, AgentTrace), AgentError> {
        let max_iterations = self.rubric.max_iterations;
        let mut iteration = 0;

        info!(
            "LoopAgent starting: goal='{}', threshold={}, max_iter={}",
            goal, self.rubric.threshold, max_iterations
        );

        loop {
            iteration += 1;
            info!("Iteration {}/{}", iteration, max_iterations);

            let result = execute_task(
                goal,
                self.llm.as_ref(),
                self.tools.as_ref(),
                self.memory.as_ref(),
                &self.config,
            )
            .await;

            match result {
                Ok(task_result) => {
                    let output_text = task_result.output().map(String::from);

                    if !self.rubric.criteria.is_empty() {
                        let score = self.score_output(goal, &task_result).await?;
                        info!(
                            "Iteration {} scored: {:.2} (threshold: {:.2})",
                            iteration, score.overall, self.rubric.threshold
                        );

                        if score.meets_threshold || iteration >= max_iterations {
                            info!(
                                "{} at iteration {} with score {:.2}",
                                if score.meets_threshold { "Quality threshold met" } else { "Max iterations reached" },
                                iteration,
                                score.overall
                            );
                            return Ok((
                                task_result,
                                AgentTrace {
                                    run_id: Uuid::new_v4(),
                                    agent_name: self.config.agent.name.clone(),
                                    goal: goal.to_string(),
                                    started_at: chrono::Utc::now(),
                                    ended_at: Some(chrono::Utc::now()),
                                    duration_ms: None,
                                    success: score.meets_threshold,
                                    output: output_text,
                                    iterations: iteration,
                                    quality_score: Some(score.overall),
                                    tool_calls: vec![],
                                    llm_calls: vec![],
                                    heal_steps: vec![],
                                    lessons: vec![],
                                    memory_ops: vec![],
                                    errors: vec![],
                                },
                            ));
                        }
                    } else {
                        return Ok((
                            task_result,
                            AgentTrace {
                                run_id: Uuid::new_v4(),
                                agent_name: self.config.agent.name.clone(),
                                goal: goal.to_string(),
                                started_at: chrono::Utc::now(),
                                ended_at: Some(chrono::Utc::now()),
                                duration_ms: None,
                                success: true,
                                output: output_text,
                                iterations: iteration,
                                quality_score: None,
                                tool_calls: vec![],
                                llm_calls: vec![],
                                heal_steps: vec![],
                                lessons: vec![],
                                memory_ops: vec![],
                                errors: vec![],
                            },
                        ));
                    }
                }
                Err(e) => {
                    if iteration >= max_iterations {
                        return Err(e);
                    }
                }
            }
        }
    }

    async fn score_output(
        &self,
        goal: &str,
        result: &TaskResult,
    ) -> Result<QualityScore, AgentError> {
        let output = match result {
            TaskResult::Success { output } => output,
            TaskResult::Partial { output, .. } => output,
            TaskResult::Failed { error: _ } => {
                return Ok(QualityScore {
                    overall: 0.0,
                    criteria_scores: vec![],
                    meets_threshold: false,
                    iteration: 0,
                });
            }
        };

        let criteria_desc: String = self
            .rubric
            .criteria
            .iter()
            .map(|c| format!(
                "- {} (weight {:.2}): {}",
                c.name, c.weight, c.description
            ))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a quality evaluator. Score the following output against these criteria.\n\n\
             Criteria:\n{}\n\n\
             Goal: {}\n\n\
             Output:\n{}\n\n\
             Return ONLY a JSON object with this exact format:\n\
             {{\"scores\": [{{\"name\": \"...\", \"score\": 0.0-1.0, \"feedback\": \"...\"}}]}}",
            criteria_desc, goal, output
        );

        let request = CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text(prompt),
            }],
            system_prompt: Some(
                "You are a quality evaluator. Return ONLY valid JSON.".to_string(),
            ),
            temperature: Some(0.1),
            max_tokens: Some(1024),
            stop_sequences: None,
            tools: None,
        };

        let response = self.llm.complete(request).await.map_err(AgentError::from)?;

        let parsed: serde_json::Value = serde_json::from_str(&response.content)
            .unwrap_or_else(|_| serde_json::json!({"scores": []}));

        let scores_array = parsed
            .get("scores")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        let mut criteria_scores = Vec::new();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for criterion in &self.rubric.criteria {
            let score_val = scores_array
                .iter()
                .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(criterion.name.as_str()))
                .and_then(|s| s.get("score"))
                .and_then(|s| s.as_f64())
                .unwrap_or(0.5) as f32;

            let feedback = scores_array
                .iter()
                .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(criterion.name.as_str()))
                .and_then(|s| s.get("feedback"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            weighted_sum += score_val * criterion.weight;
            total_weight += criterion.weight;

            criteria_scores.push(CriterionScore {
                name: criterion.name.clone(),
                weight: criterion.weight,
                score: score_val,
                feedback,
            });
        }

        let overall = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        Ok(QualityScore {
            overall,
            criteria_scores,
            meets_threshold: overall >= self.rubric.threshold,
            iteration: 0,
        })
    }
}

impl TaskResult {
    fn output(&self) -> Option<&str> {
        match self {
            TaskResult::Success { output } => Some(output),
            TaskResult::Partial { output, .. } => Some(output),
            TaskResult::Failed { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_rubric_defaults() {
        let rubric = QualityRubric {
            criteria: vec![],
            threshold: DEFAULT_THRESHOLD,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        };
        assert_eq!(rubric.threshold, 0.85);
        assert_eq!(rubric.max_iterations, 20);
    }

    #[test]
    fn test_quality_score_threshold() {
        let score = QualityScore {
            overall: 0.9,
            criteria_scores: vec![],
            meets_threshold: true,
            iteration: 1,
        };
        assert!(score.meets_threshold);
        assert!(score.overall >= 0.85);
    }

    #[test]
    fn test_quality_score_below_threshold() {
        let score = QualityScore {
            overall: 0.5,
            criteria_scores: vec![],
            meets_threshold: false,
            iteration: 1,
        };
        assert!(!score.meets_threshold);
    }
}
