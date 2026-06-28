pub mod engine;
pub mod pattern;
pub mod proposal;
pub mod types;

pub use engine::GrowthEngine;
pub use pattern::{ErrorRecord, PatternDetector};
pub use proposal::ProposalGenerator;
pub use types::{DetectedPattern, GrowthReport, ImprovementProposal, ProposalRisk, ProposalType};
