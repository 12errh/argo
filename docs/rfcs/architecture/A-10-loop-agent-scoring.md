# A-10: Loop Agent & Scoring

**Status:** Implemented
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define the quality rubric schema, scoring algorithm, iteration management, and termination conditions for loop agents.

## Motivation

Loop agents run autonomously until they meet their own quality standard. The scoring system must be transparent, configurable, and produce consistent results.

## Detailed Design

### Quality Rubric

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRubric {
    pub criteria: Vec<Criterion>,
    pub threshold: f32,
    pub max_iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

impl QualityRubric {
    pub fn validate(&self) -> Result<(), RubricError> {
        let total_weight: f32 = self.criteria.iter().map(|c| c.weight).sum();
        if (total_weight - 1.0).abs() > 0.001 {
            return Err(RubricError::InvalidWeights { total: total_weight });
        }
        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err(RubricError::InvalidThreshold { threshold: self.threshold });
        }
        Ok(())
    }
}
```

### Scoring Algorithm

```rust
pub struct LoopAgent {
    config: LoopAgentConfig,
    rubric: QualityRubric,
    llm: Box<dyn LlmProvider>,
    trace: AgentTrace,
}

impl LoopAgent {
    pub async fn run(&self, goal: &str) -> Result<LoopResult, LoopError> {
        let mut iteration = 0;
        let mut best_score = 0.0;
        let mut best_output = String::new();

        loop {
            iteration += 1;

            let output = self.execute_iteration(goal, iteration).await?;
            let scores = self.score_output(&output, &self.rubric).await?;
            let weighted_score = self.calculate_weighted_score(&scores);

            if weighted_score > best_score {
                best_score = weighted_score;
                best_output = output.clone();
            }

            if weighted_score >= self.rubric.threshold {
                return Ok(LoopResult {
                    output: best_output,
                    score: best_score,
                    iterations: iteration,
                    scores,
                });
            }

            if iteration >= self.rubric.max_iterations {
                return Ok(LoopResult {
                    output: best_output,
                    score: best_score,
                    iterations: iteration,
                    scores,
                });
            }

            let gaps = self.analyze_gaps(&scores).await?;
            self.replan(&gaps).await?;
        }
    }

    async fn score_output(&self, output: &str, rubric: &QualityRubric) -> Result<Vec<ScoredCriterion>, LoopError> {
        let mut scores = Vec::new();

        for criterion in &rubric.criteria {
            let prompt = format!(
                "Score the following output on a scale of 0.0 to 1.0 for the criterion: {}\n\nCriterion description: {}\n\nOutput:\n{}",
                criterion.name, criterion.description, output
            );

            let response = self.llm.complete(CompletionRequest {
                messages: vec![Message {
                    role: Role::User,
                    content: MessageContent::Text(prompt),
                }],
                system_prompt: Some("You are a strict quality evaluator. Return only a JSON object with a 'score' field containing a float between 0.0 and 1.0.".into()),
                temperature: Some(0.0),
                max_tokens: Some(100),
                stop_sequences: None,
                tools: None,
            }).await?;

            let score: f32 = serde_json::from_str(&response.content)
                .map_err(|_| LoopError::InvalidScore { criterion: criterion.name.clone() })?;

            scores.push(ScoredCriterion {
                criterion: criterion.clone(),
                score,
            });
        }

        Ok(scores)
    }

    fn calculate_weighted_score(&self, scores: &[ScoredCriterion]) -> f32 {
        scores.iter()
            .map(|s| s.criterion.weight * s.score)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct LoopResult {
    pub output: String,
    pub score: f32,
    pub iterations: usize,
    pub scores: Vec<ScoredCriterion>,
}

#[derive(Debug, Clone)]
pub struct ScoredCriterion {
    pub criterion: Criterion,
    pub score: f32,
}
```

### Gap Analysis

```rust
impl LoopAgent {
    async fn analyze_gaps(&self, scores: &[ScoredCriterion]) -> Result<Vec<Gap>, LoopError> {
        let prompt = format!(
            "Analyze these quality scores and identify the main gaps:\n\n{}\n\nWhat specific improvements would increase the score the most?",
            scores.iter()
                .map(|s| format!("{}: {:.2} (weight: {:.2})", s.criterion.name, s.score, s.criterion.weight))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let response = self.llm.complete(CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text(prompt),
            }],
            system_prompt: Some("You are a quality improvement analyst. Identify specific, actionable gaps.".into()),
            temperature: Some(0.3),
            max_tokens: Some(500),
            stop_sequences: None,
            tools: None,
        }).await?;

        todo!()
    }
}
```

### Termination Conditions

| Condition | Description |
|---|---|
| Score ≥ threshold | Quality standard met, stop |
| Iterations ≥ max_iterations | Budget exhausted, return best result |
| Score improvement < 0.01 for 3 iterations | Diminishing returns, stop |
| All criteria scored ≥ 0.9 | Excellent quality, stop early |

## Alternatives Considered

1. **External scoring (human or separate LLM)**: More accurate, but slower and more expensive.
2. **Rule-based scoring**: Deterministic, but less flexible than LLM-based scoring.
3. **Statistical scoring**: Uses metrics like test coverage, but limited to measurable criteria.

## Drawbacks

- LLM-based scoring adds cost and latency per iteration
- Scoring may be inconsistent across runs
- Gap analysis quality depends on LLM capability

## Unresolved Questions

- Should scoring use a separate, cheaper LLM model?
- How to handle criteria that are hard for LLMs to evaluate (e.g., code performance)?
- Should we support user-provided scoring functions?
