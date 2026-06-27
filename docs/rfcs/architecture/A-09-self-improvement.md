# A-09: Self-Improvement System

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define the daily growth cycle algorithm, pattern detection rules, improvement proposal schema, and auto-apply vs flag-for-review behavior.

## Motivation

Agents should get better over time without human intervention. The self-improvement system analyzes past errors and successes, detects patterns, and applies improvements automatically (for low-risk changes) or flags them for review (for high-risk changes).

## Detailed Design

### Growth Cycle

```rust
pub struct GrowthCycle {
    memory: MemoryHandle,
    llm: Box<dyn LlmProvider>,
    config: GrowthConfig,
}

#[derive(Debug, Clone)]
pub struct GrowthConfig {
    pub interval: Duration,
    pub auto_apply_threshold: f32,
    pub max_proposals_per_cycle: usize,
}

impl GrowthCycle {
    pub async fn run(&self, agent_id: &str) -> Result<GrowthReport, GrowthError> {
        let errors = self.memory.query_errors(agent_id, Duration::from_secs(86400)).await?;
        let patterns = self.detect_patterns(&errors).await?;
        let proposals = self.generate_proposals(&patterns).await?;
        let applied = self.apply_proposals(&proposals).await?;
        self.update_memory(&patterns, &applied).await?;

        let report = GrowthReport {
            agent_id: agent_id.to_string(),
            timestamp: chrono::Utc::now(),
            errors_analyzed: errors.len(),
            patterns_detected: patterns.len(),
            proposals_generated: proposals.len(),
            proposals_applied: applied.len(),
            patterns,
            proposals,
        };

        self.memory.store_growth_report(&report).await?;
        Ok(report)
    }
}
```

### Pattern Detection

```rust
impl GrowthCycle {
    async fn detect_patterns(&self, errors: &[ErrorRecord]) -> Result<Vec<Pattern>, GrowthError> {
        let mut patterns = Vec::new();

        // Pattern 1: Same error 3+ times
        let mut error_counts: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for error in errors {
            error_counts.entry(error.error_type.clone())
                .or_default()
                .push(error);
        }
        for (error_type, occurrences) in &error_counts {
            if occurrences.len() >= 3 {
                patterns.push(Pattern {
                    pattern_type: PatternType::RecurringError,
                    description: format!("Error '{}' occurred {} times", error_type, occurrences.len()),
                    confidence: 0.9,
                    evidence: occurrences.iter().map(|e| e.id.clone()).collect(),
                });
            }
        }

        // Pattern 2: Same tool failing
        let mut tool_failures: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for error in errors {
            if let ErrorContext::Tool { tool_name, .. } = &error.context {
                tool_failures.entry(tool_name.clone())
                    .or_default()
                    .push(error);
            }
        }
        for (tool, failures) in &tool_failures {
            if failures.len() >= 2 {
                patterns.push(Pattern {
                    pattern_type: PatternType::ToolFailure,
                    description: format!("Tool '{}' failed {} times", tool, failures.len()),
                    confidence: 0.85,
                    evidence: failures.iter().map(|e| e.id.clone()).collect(),
                });
            }
        }

        Ok(patterns)
    }
}
```

### Improvement Proposals

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub id: Uuid,
    pub proposal_type: ProposalType,
    pub target: String,
    pub content: String,
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    PromptUpdate,
    PreCheck,
    StrategyReorder,
    ConfigChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}
```

### Auto-Apply Logic

```rust
impl GrowthCycle {
    async fn apply_proposals(&self, proposals: &[ImprovementProposal]) -> Result<Vec<AppliedProposal>, GrowthError> {
        let mut applied = Vec::new();

        for proposal in proposals {
            match proposal.risk_level {
                RiskLevel::Low | RiskLevel::Medium => {
                    self.apply_proposal(proposal).await?;
                    applied.push(AppliedProposal {
                        proposal: proposal.clone(),
                        applied_at: chrono::Utc::now(),
                        auto_applied: true,
                    });
                }
                RiskLevel::High => {
                    self.flag_for_review(proposal).await?;
                }
            }
        }

        Ok(applied)
    }

    async fn apply_proposal(&self, proposal: &ImprovementProposal) -> Result<(), GrowthError> {
        match proposal.proposal_type {
            ProposalType::PromptUpdate => {
                self.memory.append_to_prompt(&proposal.target, &proposal.content).await?;
            }
            ProposalType::PreCheck => {
                self.memory.add_pre_check(&proposal.target, &proposal.content).await?;
            }
            ProposalType::StrategyReorder => {
                self.memory.update_strategy_order(&proposal.target, &proposal.content).await?;
            }
            ProposalType::ConfigChange => {
                self.memory.update_config(&proposal.target, &proposal.content).await?;
            }
        }
        Ok(())
    }
}
```

### Growth Report

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthReport {
    pub agent_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub errors_analyzed: usize,
    pub patterns_detected: usize,
    pub proposals_generated: usize,
    pub proposals_applied: usize,
    pub patterns: Vec<Pattern>,
    pub proposals: Vec<ImprovementProposal>,
}
```

## Alternatives Considered

1. **Manual improvement only**: Simpler, but defeats the purpose of self-improvement.
2. **Reinforcement learning**: More adaptive, but requires reward signals and training data.
3. **Rule-based improvements**: Deterministic, but less flexible than LLM-driven analysis.

## Drawbacks

- Growth cycle consumes LLM tokens
- Auto-applied changes may degrade performance
- Pattern detection may produce false positives

## Unresolved Questions

- How to measure if improvements actually help (A/B testing)?
- Should growth cycle run more frequently for high-traffic agents?
- How to handle conflicting improvements from different growth cycles?
