use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalScenario {
    pub name: String,
    pub goal: String,
    pub expected_outcome: Option<String>,
    pub scoring_criteria: Vec<ScoringCriterion>,
    pub tools_allowed: Option<Vec<String>>,
    pub max_iterations: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringCriterion {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub scenario_name: String,
    pub success: bool,
    pub score: f32,
    pub criteria_scores: Vec<CriterionResult>,
    pub output: String,
    pub duration_ms: u64,
    pub iterations: usize,
    pub heal_steps: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub name: String,
    pub weight: f32,
    pub score: f32,
    pub feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub total_scenarios: usize,
    pub passed: usize,
    pub failed: usize,
    pub avg_score: f32,
    pub results: Vec<EvalResult>,
}

fn load_scenario(path: &Path) -> anyhow::Result<EvalScenario> {
    let content = std::fs::read_to_string(path)?;
    let scenario: EvalScenario = toml::from_str(&content)?;
    Ok(scenario)
}

fn load_scenarios_from_dir(dir: &Path) -> anyhow::Result<Vec<EvalScenario>> {
    let mut scenarios = Vec::new();

    if dir.is_file() {
        scenarios.push(load_scenario(dir)?);
        return Ok(scenarios);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            match load_scenario(&path) {
                Ok(scenario) => scenarios.push(scenario),
                Err(e) => {
                    tracing::warn!("Failed to load scenario {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(scenarios)
}

pub async fn execute(scenario_path: &Path, config_path: &Path, format: &str) -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           Argo Agent Evaluation                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Agent config: {}", config_path.display());
    println!("  Scenarios:    {}", scenario_path.display());
    println!();

    let scenarios = load_scenarios_from_dir(scenario_path)?;

    if scenarios.is_empty() {
        println!("No valid scenarios found at {}", scenario_path.display());
        println!();
        println!("Create a scenario file (.toml) with this format:");
        println!();
        println!("  name = \"coding-basics\"");
        println!("  goal = \"Write a Python function that adds two numbers\"");
        println!("  expected_outcome = \"Function defined and tested\"");
        println!();
        println!("  [[scoring_criteria]]");
        println!("  name = \"correctness\"");
        println!("  weight = 0.5");
        println!("  description = \"Function works correctly\"");
        println!();
        println!("  [[scoring_criteria]]");
        println!("  name = \"tests\"");
        println!("  weight = 0.3");
        println!("  description = \"Tests are present and pass\"");
        println!();
        println!("  [[scoring_criteria]]");
        println!("  name = \"style\"");
        println!("  weight = 0.2");
        println!("  description = \"Code follows Python conventions\"");
        return Ok(());
    }

    println!("Found {} scenario(s) to evaluate", scenarios.len());
    println!();

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for scenario in &scenarios {
        print!("  Evaluating: {} ... ", scenario.name);

        let result = run_scenario(scenario, config_path).await;

        match &result {
            Ok(eval_result) => {
                if eval_result.success {
                    println!("PASS (score: {:.2})", eval_result.score);
                    passed += 1;
                } else {
                    println!("FAIL (score: {:.2})", eval_result.score);
                    failed += 1;
                }
                results.push(eval_result.clone());
            }
            Err(e) => {
                println!("ERROR: {}", e);
                failed += 1;
                results.push(EvalResult {
                    scenario_name: scenario.name.clone(),
                    success: false,
                    score: 0.0,
                    criteria_scores: vec![],
                    output: String::new(),
                    duration_ms: 0,
                    iterations: 0,
                    heal_steps: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let total = results.len();
    let avg_score = if total > 0 {
        results.iter().map(|r| r.score).sum::<f32>() / total as f32
    } else {
        0.0
    };

    let report = EvalReport {
        total_scenarios: total,
        passed,
        failed,
        avg_score,
        results,
    };

    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  Evaluation Summary                                    │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│  Scenarios:  {:<41}│", report.total_scenarios);
    println!("│  Passed:     {:<41}│", report.passed);
    println!("│  Failed:     {:<41}│", report.failed);
    println!("│  Avg score:  {:<41}│", format!("{:.2}", report.avg_score));
    println!("└─────────────────────────────────────────────────────────┘");

    if format == "json" {
        let json = serde_json::to_string_pretty(&report)?;
        println!();
        println!("{}", json);
    }

    Ok(())
}

async fn run_scenario(scenario: &EvalScenario, _config_path: &Path) -> anyhow::Result<EvalResult> {
    let start = std::time::Instant::now();

    tracing::info!(
        scenario = %scenario.name,
        goal = %scenario.goal,
        "Running eval scenario"
    );

    let duration = start.elapsed().as_millis() as u64;

    Ok(EvalResult {
        scenario_name: scenario.name.clone(),
        success: false,
        score: 0.0,
        criteria_scores: scenario
            .scoring_criteria
            .iter()
            .map(|c| CriterionResult {
                name: c.name.clone(),
                weight: c.weight,
                score: 0.0,
                feedback: "Eval requires connected agent runtime".into(),
            })
            .collect(),
        output: "Scenario evaluation requires agent runtime connection".into(),
        duration_ms: duration,
        iterations: 0,
        heal_steps: 0,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_scenario_from_file() {
        let dir = std::env::temp_dir().join("argo_eval_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let scenario_content = r#"
name = "test-scenario"
goal = "Write a function that adds two numbers"
expected_outcome = "Function works correctly"

[[scoring_criteria]]
name = "correctness"
weight = 0.5
description = "Function produces correct output"

[[scoring_criteria]]
name = "tests"
weight = 0.3
description = "Tests are present and pass"
"#;

        let path = dir.join("test.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(scenario_content.as_bytes()).unwrap();

        let scenario = load_scenario(&path).unwrap();
        assert_eq!(scenario.name, "test-scenario");
        assert_eq!(scenario.scoring_criteria.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_scenarios_from_dir() {
        let dir = std::env::temp_dir().join("argo_eval_dir_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let scenario1 = r#"
name = "scenario-1"
goal = "Task 1"

[[scoring_criteria]]
name = "quality"
weight = 1.0
description = "Output quality"
"#;

        let scenario2 = r#"
name = "scenario-2"
goal = "Task 2"

[[scoring_criteria]]
name = "quality"
weight = 1.0
description = "Output quality"
"#;

        std::fs::write(dir.join("s1.toml"), scenario1).unwrap();
        std::fs::write(dir.join("s2.toml"), scenario2).unwrap();
        std::fs::write(dir.join("readme.txt"), "not a scenario").unwrap();

        let scenarios = load_scenarios_from_dir(&dir).unwrap();
        assert_eq!(scenarios.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_execute_no_scenarios() {
        let dir = std::env::temp_dir().join("argo_eval_empty_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let config = dir.join("agent.toml");
        std::fs::write(&config, "[agent]\nname = \"test\"\n").unwrap();

        let result = execute(&dir, &config, "text").await;
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
