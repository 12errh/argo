pub async fn execute(run_id: &str, trace: bool, heal: bool, lessons: bool) -> anyhow::Result<()> {
    println!("Inspecting run: {}", run_id);
    println!();

    let run_uuid = uuid::Uuid::parse_str(run_id)
        .map_err(|e| anyhow::anyhow!("Invalid run ID '{}': {}", run_id, e))?;

    println!("Run ID: {}", run_uuid);
    println!("Status: (run inspection requires runtime data store)");
    println!();

    if trace {
        println!("--- Full Trace ---");
        println!("Trace data is available when the agent is run with --inspect flag.");
        println!("To see live traces, run: argo run --config agent.toml --inspect \"your goal\"");
    }

    if heal {
        println!("--- Heal Steps ---");
        println!("Heal step data is recorded during agent execution.");
        println!("Run the agent with --inspect to see heal steps in real-time.");
    }

    if lessons {
        println!("--- Lessons Learned ---");
        println!("Lessons are stored in Qdrant semantic memory after error resolution.");
        println!("Query lessons with: argo memory search --query \"error type\"");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inspect_invalid_id() {
        let result = execute("not-a-uuid", false, false, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inspect_valid_id() {
        let result = execute("550e8400-e29b-41d4-a716-446655440000", true, true, true).await;
        assert!(result.is_ok());
    }
}
