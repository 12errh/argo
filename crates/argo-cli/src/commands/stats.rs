pub async fn execute(
    agent: Option<&str>,
    range: &str,
    compare: Option<&str>,
) -> anyhow::Result<()> {
    println!("Agent Performance Statistics");
    println!("Time range: {}", range);

    if let Some(a) = agent {
        println!("Agent: {}", a);
    }

    if let Some(c) = compare {
        println!("Comparing with: {}", c);
    }

    println!();
    println!("Stats require runtime data from SurrealDB.");
    println!("Start services with: docker compose up -d");
    println!();
    println!("Available metrics once connected:");
    println!("  - Tasks completed: total, success, partial, failed");
    println!("  - Average quality score");
    println!("  - Average iterations per task");
    println!("  - Errors per task");
    println!("  - Most improved area");
    println!("  - Top lessons learned");
    println!("  - Tool usage breakdown");
    println!("  - LLM token usage");

    Ok(())
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
}
