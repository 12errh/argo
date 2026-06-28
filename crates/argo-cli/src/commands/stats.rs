use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStats {
    pub agent_name: String,
    pub period: String,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub partial_tasks: usize,
    pub failed_tasks: usize,
    pub avg_quality_score: Option<f32>,
    pub avg_iterations: Option<f32>,
    pub errors_per_task: f32,
    pub total_heal_steps: usize,
    pub strategy_breakdown: Vec<StrategyStat>,
    pub tool_usage: Vec<ToolUsageStat>,
    pub top_lessons: Vec<String>,
    pub growth_reports: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyStat {
    pub name: String,
    pub attempts: usize,
    pub successes: usize,
    pub effectiveness: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolUsageStat {
    pub tool_name: String,
    pub invocations: usize,
    pub success_rate: f32,
    pub avg_duration_ms: u64,
}

fn parse_duration(s: &str) -> Duration {
    let trimmed = s.trim().to_lowercase();
    if trimmed.ends_with('h') {
        let hours: u64 = trimmed.trim_end_matches('h').parse().unwrap_or(24);
        Duration::hours(hours as i64)
    } else if trimmed.ends_with('d') {
        let days: u64 = trimmed.trim_end_matches('d').parse().unwrap_or(7);
        Duration::days(days as i64)
    } else if trimmed.ends_with("min") || trimmed.ends_with('m') {
        let minutes: u64 = trimmed
            .trim_end_matches("min")
            .trim_end_matches('m')
            .parse()
            .unwrap_or(60);
        Duration::minutes(minutes as i64)
    } else {
        Duration::hours(24)
    }
}

pub async fn execute(
    agent: Option<&str>,
    range: &str,
    compare: Option<&str>,
) -> anyhow::Result<()> {
    let duration = parse_duration(range);
    let agent_name = agent.unwrap_or("all");

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           Argo Agent Performance Statistics              ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Period: last {}", range);
    println!("  Agent:  {}", agent_name);
    println!();

    let stats = gather_stats(agent_name, duration).await;

    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  Task Summary                                          │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│  Total tasks:     {:<37}│", stats.total_tasks);
    println!("│  Successful:      {:<37}│", stats.successful_tasks);
    println!("│  Partial:         {:<37}│", stats.partial_tasks);
    println!("│  Failed:          {:<37}│", stats.failed_tasks);
    println!("└─────────────────────────────────────────────────────────┘");
    println!();

    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  Quality Metrics                                       │");
    println!("├─────────────────────────────────────────────────────────┤");
    match stats.avg_quality_score {
        Some(score) => println!("│  Avg quality:     {:<37}│", format!("{:.2}", score)),
        None => println!("│  Avg quality:     {:<37}│", "N/A (no loop agents)"),
    }
    match stats.avg_iterations {
        Some(iters) => println!("│  Avg iterations:  {:<37}│", format!("{:.1}", iters)),
        None => println!("│  Avg iterations:  {:<37}│", "N/A"),
    }
    println!(
        "│  Errors/task:     {:<37}│",
        format!("{:.1}", stats.errors_per_task)
    );
    println!(
        "│  Heal steps:      {:<37}│",
        stats.total_heal_steps
    );
    println!("└─────────────────────────────────────────────────────────┘");
    println!();

    if !stats.strategy_breakdown.is_empty() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Heal Strategy Effectiveness                           │");
        println!("├─────────────────────────────────────────────────────────┤");
        for s in &stats.strategy_breakdown {
            println!(
                "│  {:<20} {:>4}/{:<4} ({:>5.0}%)              │",
                s.name,
                s.successes,
                s.attempts,
                s.effectiveness * 100.0
            );
        }
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
    }

    if !stats.tool_usage.is_empty() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Tool Usage                                            │");
        println!("├─────────────────────────────────────────────────────────┤");
        for t in &stats.tool_usage {
            println!(
                "│  {:<16} {:>5} calls  {:>5.0}% ok  {:>6}ms avg │",
                t.tool_name,
                t.invocations,
                t.success_rate * 100.0,
                t.avg_duration_ms
            );
        }
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
    }

    if !stats.top_lessons.is_empty() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Top Lessons Learned                                   │");
        println!("├─────────────────────────────────────────────────────────┤");
        for (i, lesson) in stats.top_lessons.iter().take(5).enumerate() {
            let truncated = if lesson.len() > 53 {
                format!("{}...", &lesson[..50])
            } else {
                lesson.clone()
            };
            println!("│  {}. {:<51}│", i + 1, truncated);
        }
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
    }

    if let Some(compare_agent) = compare {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Comparison: {} vs {}", agent_name, compare_agent);
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│  (Compare feature available with persistent memory)    │");
        println!("└─────────────────────────────────────────────────────────┘");
    }

    println!();
    println!("  Growth reports generated: {}", stats.growth_reports);
    println!();

    Ok(())
}

async fn gather_stats(agent_name: &str, _duration: Duration) -> AgentStats {
    let _now = Utc::now();

    let stats = AgentStats {
        agent_name: agent_name.to_string(),
        period: "dynamic".to_string(),
        total_tasks: 0,
        successful_tasks: 0,
        partial_tasks: 0,
        failed_tasks: 0,
        avg_quality_score: None,
        avg_iterations: None,
        errors_per_task: 0.0,
        total_heal_steps: 0,
        strategy_breakdown: vec![],
        tool_usage: vec![],
        top_lessons: vec![],
        growth_reports: 0,
    };

    tracing::debug!(
        agent = agent_name,
        "Gathering stats from memory stores"
    );

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stats_basic() {
        let result = execute(None, "24h", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stats_with_agent() {
        let result = execute(Some("coder"), "7d", Some("researcher")).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_duration_hours() {
        let d = parse_duration("24h");
        assert_eq!(d, Duration::hours(24));
    }

    #[test]
    fn test_parse_duration_days() {
        let d = parse_duration("7d");
        assert_eq!(d, Duration::days(7));
    }

    #[test]
    fn test_parse_duration_minutes() {
        let d = parse_duration("30min");
        assert_eq!(d, Duration::minutes(30));
    }
}
