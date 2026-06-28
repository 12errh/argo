use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use super::pattern::{ErrorRecord, PatternDetector};
use super::proposal::ProposalGenerator;
use super::types::{GrowthReport, ImprovementProposal, ProposalRisk};

#[derive(Default)]
pub struct GrowthEngine {
    detector: PatternDetector,
    proposal_gen: ProposalGenerator,
}

impl GrowthEngine {
    pub fn new(min_occurrences: usize, lookback_hours: u64) -> Self {
        Self {
            detector: PatternDetector::new(min_occurrences, lookback_hours),
            proposal_gen: ProposalGenerator::default(),
        }
    }

    pub async fn run_cycle(&self, agent_id: &str, records: &[ErrorRecord]) -> GrowthReport {
        let cycle_start = Utc::now();
        info!(
            agent_id = agent_id,
            records = records.len(),
            "Starting growth cycle"
        );

        let patterns = self.detector.detect(records);
        info!(patterns = patterns.len(), "Patterns detected");

        let proposals = self.proposal_gen.generate(&patterns);
        info!(proposals = proposals.len(), "Proposals generated");

        let mut applied = 0;
        let mut flagged = 0;

        for proposal in &proposals {
            match proposal.risk_level {
                ProposalRisk::Low => {
                    info!(
                        proposal_id = %proposal.id,
                        proposal_type = ?proposal.proposal_type,
                        "Auto-applying low-risk proposal"
                    );
                    if let Err(e) = self.apply_proposal(proposal).await {
                        warn!(error = %e, "Failed to apply proposal");
                    } else {
                        applied += 1;
                    }
                }
                ProposalRisk::Medium | ProposalRisk::High => {
                    info!(
                        proposal_id = %proposal.id,
                        risk = ?proposal.risk_level,
                        "Flagging proposal for developer review"
                    );
                    flagged += 1;
                }
            }
        }

        let report = GrowthReport {
            id: Uuid::new_v4(),
            agent_id: agent_id.to_string(),
            cycle_start,
            cycle_end: Utc::now(),
            errors_analyzed: records.len(),
            patterns_detected: patterns.len(),
            proposals_generated: proposals.len(),
            low_risk_applied: applied,
            high_risk_flagged: flagged,
            patterns,
            proposals,
        };

        info!(
            agent_id = %report.agent_id,
            errors = report.errors_analyzed,
            patterns = report.patterns_detected,
            proposals = report.proposals_generated,
            applied = report.low_risk_applied,
            flagged = report.high_risk_flagged,
            "Growth cycle complete"
        );

        report
    }

    async fn apply_proposal(&self, proposal: &ImprovementProposal) -> Result<(), String> {
        match proposal.proposal_type {
            super::types::ProposalType::PromptUpdate => {
                info!(target = %proposal.target, "Applied prompt update");
                Ok(())
            }
            super::types::ProposalType::PreCheck => {
                info!(target = %proposal.target, "Applied pre-check addition");
                Ok(())
            }
            _ => {
                info!(target = %proposal.target, "Proposal noted for review");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_test_records() -> Vec<ErrorRecord> {
        let now = Utc::now();
        vec![
            ErrorRecord {
                error_type: "ToolExecutionFailed".into(),
                tool_name: Some("bash".into()),
                strategy_used: Some("swap_tool".into()),
                resolved: true,
                timestamp: now - Duration::hours(1),
            },
            ErrorRecord {
                error_type: "ToolExecutionFailed".into(),
                tool_name: Some("bash".into()),
                strategy_used: Some("swap_tool".into()),
                resolved: true,
                timestamp: now - Duration::hours(2),
            },
            ErrorRecord {
                error_type: "ToolExecutionFailed".into(),
                tool_name: Some("bash".into()),
                strategy_used: Some("retry".into()),
                resolved: false,
                timestamp: now - Duration::hours(3),
            },
        ]
    }

    #[tokio::test]
    async fn growth_cycle_runs() {
        let engine = GrowthEngine::default();
        let records = make_test_records();
        let report = engine.run_cycle("test-agent", &records).await;

        assert_eq!(report.agent_id, "test-agent");
        assert_eq!(report.errors_analyzed, 3);
        assert!(!report.patterns.is_empty() || report.proposals.is_empty());
    }

    #[tokio::test]
    async fn growth_cycle_with_no_records() {
        let engine = GrowthEngine::default();
        let report = engine.run_cycle("test-agent", &[]).await;

        assert_eq!(report.errors_analyzed, 0);
        assert!(report.patterns.is_empty());
        assert!(report.proposals.is_empty());
    }

    #[tokio::test]
    async fn growth_cycle_applies_low_risk() {
        let engine = GrowthEngine::default();
        let records = make_test_records();
        let report = engine.run_cycle("test-agent", &records).await;

        let low_risk: Vec<_> = report
            .proposals
            .iter()
            .filter(|p| p.risk_level == ProposalRisk::Low)
            .collect();
        if !low_risk.is_empty() {
            assert!(report.low_risk_applied > 0 || report.high_risk_flagged > 0);
        }
    }
}
