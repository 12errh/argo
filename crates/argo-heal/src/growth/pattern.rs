use std::collections::HashMap;

use chrono::{Duration, Utc};

use super::types::{DetectedPattern, PatternType};

#[derive(Debug, Clone)]
pub struct ErrorRecord {
    pub error_type: String,
    pub tool_name: Option<String>,
    pub strategy_used: Option<String>,
    pub resolved: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct PatternDetector {
    pub min_occurrences: usize,
    pub lookback_hours: u64,
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self {
            min_occurrences: 3,
            lookback_hours: 24,
        }
    }
}

impl PatternDetector {
    pub fn new(min_occurrences: usize, lookback_hours: u64) -> Self {
        Self {
            min_occurrences,
            lookback_hours,
        }
    }

    pub fn detect(&self, records: &[ErrorRecord]) -> Vec<DetectedPattern> {
        let cutoff = Utc::now() - Duration::hours(self.lookback_hours as i64);
        let recent: Vec<&ErrorRecord> = records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .collect();

        let mut patterns = Vec::new();

        patterns.extend(self.detect_recurring_errors(&recent));
        patterns.extend(self.detect_tool_failures(&recent));
        patterns.extend(self.detect_strategy_effectiveness(&recent));

        patterns
    }

    fn detect_recurring_errors(&self, records: &[&ErrorRecord]) -> Vec<DetectedPattern> {
        let mut error_counts: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for record in records {
            error_counts
                .entry(record.error_type.clone())
                .or_default()
                .push(record);
        }

        let mut patterns = Vec::new();
        for (error_type, occurrences) in &error_counts {
            if occurrences.len() >= self.min_occurrences {
                let resolved_count = occurrences.iter().filter(|r| r.resolved).count();
                let confidence = resolved_count as f32 / occurrences.len() as f32;

                patterns.push(DetectedPattern {
                    pattern_type: PatternType::RecurringError,
                    description: format!(
                        "Error '{}' occurred {} times in the last {}h",
                        error_type,
                        occurrences.len(),
                        self.lookback_hours
                    ),
                    occurrences: occurrences.len(),
                    error_types: vec![error_type.clone()],
                    tool_names: occurrences
                        .iter()
                        .filter_map(|r| r.tool_name.clone())
                        .collect(),
                    time_range_hours: self.lookback_hours,
                    confidence,
                });
            }
        }

        patterns
    }

    fn detect_tool_failures(&self, records: &[&ErrorRecord]) -> Vec<DetectedPattern> {
        let mut tool_errors: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for record in records {
            if let Some(tool) = &record.tool_name {
                tool_errors
                    .entry(tool.clone())
                    .or_default()
                    .push(record);
            }
        }

        let mut patterns = Vec::new();
        for (tool_name, failures) in &tool_errors {
            if failures.len() >= self.min_occurrences {
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::ToolFailure,
                    description: format!(
                        "Tool '{}' failed {} times in the last {}h",
                        tool_name,
                        failures.len(),
                        self.lookback_hours
                    ),
                    occurrences: failures.len(),
                    error_types: failures
                        .iter()
                        .map(|r| r.error_type.clone())
                        .collect(),
                    tool_names: vec![tool_name.clone()],
                    time_range_hours: self.lookback_hours,
                    confidence: 0.8,
                });
            }
        }

        patterns
    }

    fn detect_strategy_effectiveness(&self, records: &[&ErrorRecord]) -> Vec<DetectedPattern> {
        let mut strategy_stats: HashMap<String, (usize, usize)> = HashMap::new();
        for record in records {
            if let Some(strategy) = &record.strategy_used {
                let entry = strategy_stats.entry(strategy.clone()).or_insert((0, 0));
                entry.0 += 1;
                if record.resolved {
                    entry.1 += 1;
                }
            }
        }

        let mut patterns = Vec::new();
        for (strategy, (total, resolved)) in &strategy_stats {
            if *total >= self.min_occurrences {
                let effectiveness = *resolved as f32 / *total as f32;
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::StrategyEffectiveness,
                    description: format!(
                        "Strategy '{}' resolved {}/{} errors ({:.0}%)",
                        strategy,
                        resolved,
                        total,
                        effectiveness * 100.0
                    ),
                    occurrences: *total,
                    error_types: vec![],
                    tool_names: vec![],
                    time_range_hours: self.lookback_hours,
                    confidence: effectiveness,
                });
            }
        }

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_records() -> Vec<ErrorRecord> {
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
            ErrorRecord {
                error_type: "LlmRateLimit".into(),
                tool_name: None,
                strategy_used: Some("retry".into()),
                resolved: true,
                timestamp: now - Duration::hours(4),
            },
            ErrorRecord {
                error_type: "LlmRateLimit".into(),
                tool_name: None,
                strategy_used: Some("retry".into()),
                resolved: true,
                timestamp: now - Duration::hours(5),
            },
            ErrorRecord {
                error_type: "LlmRateLimit".into(),
                tool_name: None,
                strategy_used: Some("retry".into()),
                resolved: true,
                timestamp: now - Duration::hours(6),
            },
        ]
    }

    #[test]
    fn detects_recurring_errors() {
        let detector = PatternDetector::default();
        let records = make_records();
        let patterns = detector.detect(&records);

        let recurring: Vec<_> = patterns
            .iter()
            .filter(|p| p.pattern_type == PatternType::RecurringError)
            .collect();
        assert!(recurring.len() >= 2);
    }

    #[test]
    fn detects_tool_failures() {
        let detector = PatternDetector::default();
        let records = make_records();
        let patterns = detector.detect(&records);

        let tool_fails: Vec<_> = patterns
            .iter()
            .filter(|p| p.pattern_type == PatternType::ToolFailure)
            .collect();
        assert!(!tool_fails.is_empty());
        assert!(tool_fails[0].tool_names.contains(&"bash".to_string()));
    }

    #[test]
    fn detects_strategy_effectiveness() {
        let detector = PatternDetector::default();
        let records = make_records();
        let patterns = detector.detect(&records);

        let strategy_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| p.pattern_type == PatternType::StrategyEffectiveness)
            .collect();
        assert!(!strategy_patterns.is_empty());
    }

    #[test]
    fn respects_min_occurrences() {
        let detector = PatternDetector::new(10, 24);
        let records = make_records();
        let patterns = detector.detect(&records);

        for pattern in &patterns {
            assert!(pattern.occurrences >= 10);
        }
    }
}
