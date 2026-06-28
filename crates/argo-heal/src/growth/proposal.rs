use uuid::Uuid;

use super::types::{DetectedPattern, ImprovementProposal, ProposalRisk, ProposalType};

pub struct ProposalGenerator {
    pub auto_apply_threshold: ProposalRisk,
}

impl Default for ProposalGenerator {
    fn default() -> Self {
        Self {
            auto_apply_threshold: ProposalRisk::Low,
        }
    }
}

impl ProposalGenerator {
    pub fn generate(&self, patterns: &[DetectedPattern]) -> Vec<ImprovementProposal> {
        let mut proposals = Vec::new();

        for pattern in patterns {
            match pattern.pattern_type {
                super::types::PatternType::RecurringError => {
                    proposals.extend(self.generate_for_recurring_error(pattern));
                }
                super::types::PatternType::ToolFailure => {
                    proposals.extend(self.generate_for_tool_failure(pattern));
                }
                super::types::PatternType::StrategyEffectiveness => {
                    proposals.extend(self.generate_for_strategy(pattern));
                }
                _ => {}
            }
        }

        proposals
    }

    fn generate_for_recurring_error(&self, pattern: &DetectedPattern) -> Vec<ImprovementProposal> {
        let mut proposals = Vec::new();

        if let Some(error_type) = pattern.error_types.first() {
            let risk = if pattern.confidence > 0.8 {
                ProposalRisk::Low
            } else {
                ProposalRisk::Medium
            };

            proposals.push(ImprovementProposal {
                id: Uuid::new_v4(),
                proposal_type: ProposalType::PreCheck,
                risk_level: risk.clone(),
                target: error_type.clone(),
                content: format!(
                    "Add pre-check to prevent '{}' errors: validate inputs before execution",
                    error_type
                ),
                confidence: pattern.confidence,
                evidence: pattern.error_types.clone(),
                created_at: chrono::Utc::now(),
            });

            if !pattern.tool_names.is_empty() {
                proposals.push(ImprovementProposal {
                    id: Uuid::new_v4(),
                    proposal_type: ProposalType::ToolSwap,
                    risk_level: ProposalRisk::Low,
                    target: pattern.tool_names.join(", "),
                    content: format!(
                        "Consider alternative tools for tasks currently using: {}",
                        pattern.tool_names.join(", ")
                    ),
                    confidence: 0.7,
                    evidence: pattern.tool_names.clone(),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        proposals
    }

    fn generate_for_tool_failure(&self, pattern: &DetectedPattern) -> Vec<ImprovementProposal> {
        let mut proposals = Vec::new();

        if let Some(tool) = pattern.tool_names.first() {
            proposals.push(ImprovementProposal {
                id: Uuid::new_v4(),
                proposal_type: ProposalType::ToolSwap,
                risk_level: ProposalRisk::Medium,
                target: tool.clone(),
                content: format!(
                    "Tool '{}' has failed {} times. Consider adding fallback tool or updating configuration.",
                    tool, pattern.occurrences
                ),
                confidence: 0.75,
                evidence: pattern.error_types.clone(),
                created_at: chrono::Utc::now(),
            });
        }

        proposals
    }

    fn generate_for_strategy(&self, pattern: &DetectedPattern) -> Vec<ImprovementProposal> {
        let mut proposals = Vec::new();

        let risk = if pattern.confidence > 0.7 {
            ProposalRisk::Low
        } else {
            ProposalRisk::High
        };

        proposals.push(ImprovementProposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::StrategyReorder,
            risk_level: risk,
            target: "heal_strategy_order".to_string(),
            content: format!(
                "Strategy effectiveness: {:.0}% success rate. {}",
                pattern.confidence * 100.0,
                if pattern.confidence > 0.7 {
                    "Consider promoting this strategy in the ordering."
                } else {
                    "This strategy may need review or replacement."
                }
            ),
            confidence: pattern.confidence,
            evidence: vec![pattern.description.clone()],
            created_at: chrono::Utc::now(),
        });

        proposals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth::types::{DetectedPattern, PatternType};

    fn make_recurring_pattern() -> DetectedPattern {
        DetectedPattern {
            pattern_type: PatternType::RecurringError,
            description: "Test pattern".into(),
            occurrences: 5,
            error_types: vec!["ToolExecutionFailed".into()],
            tool_names: vec!["bash".into()],
            time_range_hours: 24,
            confidence: 0.9,
        }
    }

    fn make_tool_failure_pattern() -> DetectedPattern {
        DetectedPattern {
            pattern_type: PatternType::ToolFailure,
            description: "Tool failing".into(),
            occurrences: 4,
            error_types: vec!["ToolExecutionFailed".into()],
            tool_names: vec!["python".into()],
            time_range_hours: 24,
            confidence: 0.8,
        }
    }

    #[test]
    fn generates_proposals_for_recurring_error() {
        let gen = ProposalGenerator::default();
        let patterns = vec![make_recurring_pattern()];
        let proposals = gen.generate(&patterns);
        assert!(!proposals.is_empty());
        assert!(proposals
            .iter()
            .any(|p| p.proposal_type == ProposalType::PreCheck));
    }

    #[test]
    fn generates_proposals_for_tool_failure() {
        let gen = ProposalGenerator::default();
        let patterns = vec![make_tool_failure_pattern()];
        let proposals = gen.generate(&patterns);
        assert!(!proposals.is_empty());
        assert!(proposals
            .iter()
            .any(|p| p.proposal_type == ProposalType::ToolSwap));
    }

    #[test]
    fn high_confidence_is_low_risk() {
        let gen = ProposalGenerator::default();
        let patterns = vec![make_recurring_pattern()];
        let proposals = gen.generate(&patterns);
        let precheck = proposals
            .iter()
            .find(|p| p.proposal_type == ProposalType::PreCheck)
            .unwrap();
        assert_eq!(precheck.risk_level, ProposalRisk::Low);
    }
}
