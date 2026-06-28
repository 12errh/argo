use std::sync::Arc;
use std::time::Duration;

use argo_core::config::*;
use argo_core::llm::openai::OpenAiProvider;
use argo_core::loop_agent::LoopAgent;
use argo_core::message::TaskResult;
use argo_memory::handle::MemoryHandle;
use argo_memory::redis::RedisMemory;
use argo_memory::surreal::SurrealMemory;
use argo_tools::registry::ToolRegistry;

fn load_env() {
    let _ = dotenvy::from_filename("../../.env");
}

fn test_config() -> AgentConfig {
    load_env();
    let api_key =
        std::env::var("OPENCODE_API_KEY").expect("Set OPENCODE_API_KEY in .env or environment");

    AgentConfig {
        agent: AgentSection {
            name: "full-test-agent".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Full autonomous agent testing all features".to_string()),
        },
        model: ModelSection {
            provider: "openai".to_string(),
            model: "mimo-v2.5-free".to_string(),
            api_key: Some(api_key),
            base_url: Some("https://opencode.ai/zen/v1".to_string()),
            temperature: Some(0.2),
            max_tokens: Some(4096),
            context_strategy: None,
        },
        memory: Some(MemorySection {
            mode: Some("persistent".to_string()),
            short_term_ttl: Some(3600),
            long_term_backend: Some("surrealdb".to_string()),
            vector_backend: Some("qdrant".to_string()),
            embedding_model: None,
        }),
        heal: Some(HealSection {
            enabled: Some(true),
            max_attempts: Some(3),
            strategies: Some(vec![
                "retry".to_string(),
                "reframe".to_string(),
                "swap_tool".to_string(),
            ]),
            background: Some(false),
        }),
        quality: Some(QualitySection {
            threshold: Some(0.7),
            max_iterations: Some(5),
            criteria: Some(vec![
                QualityCriterion {
                    name: "correctness".to_string(),
                    weight: 0.5,
                    description: "Output is correct and complete".to_string(),
                },
                QualityCriterion {
                    name: "uses_tools".to_string(),
                    weight: 0.3,
                    description: "Agent actually used tools to complete the task".to_string(),
                },
                QualityCriterion {
                    name: "well_structured".to_string(),
                    weight: 0.2,
                    description: "Output is well organized and clear".to_string(),
                },
            ]),
        }),
        tools: ToolsSection {
            enabled: vec![
                "bash".to_string(),
                "files".to_string(),
                "git".to_string(),
                "code".to_string(),
                "python".to_string(),
            ],
        },
        permissions: PermissionsSection {
            allow_network: true,
            allow_filesystem: true,
            allowed_paths: Some(vec![".".to_string(), "/tmp".to_string()]),
            max_execution_time: Some(120),
        },
        observe: Some(ObserveSection {
            enabled: Some(true),
            backend: Some("tracing".to_string()),
            endpoint: None,
        }),
    }
}

async fn setup_memory() -> Option<MemoryHandle> {
    let redis_url = "redis://127.0.0.1:6379";
    let redis = match RedisMemory::new(redis_url).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Redis not available: {}", e);
            return None;
        }
    };

    let surreal = SurrealMemory::new("http://127.0.0.1:8000", "argo_test", "full_test");
    Some(MemoryHandle::persistent(redis, surreal, "full-test-agent"))
}

fn make_llm(config: &AgentConfig) -> Arc<OpenAiProvider> {
    Arc::new(
        OpenAiProvider::new(
            config.model.api_key.clone().unwrap(),
            config.model.model.clone(),
        )
        .with_base_url(config.model.base_url.clone().unwrap()),
    )
}

fn make_tools() -> Arc<ToolRegistry> {
    let mut tools = ToolRegistry::new();
    let allowed_paths = vec![".".to_string(), "/tmp".to_string()];
    let max_exec = Duration::from_secs(120);

    tools.register(Arc::new(argo_tools::bash::BashTool::new(
        "/tmp".to_string(),
        max_exec,
    )));
    tools.register(Arc::new(argo_tools::files::FilesTool::new(
        allowed_paths.clone(),
        max_exec,
    )));
    tools.register(Arc::new(argo_tools::git_tool::GitTool::new()));
    tools.register(Arc::new(argo_tools::code::CodeTool::new()));
    tools.register(Arc::new(argo_tools::python_tool::PythonTool::new()));
    Arc::new(tools)
}

// ─── Test 1: Basic bash tool execution ──────────────────────────────────────

#[tokio::test]
async fn test_1_basic_execution_with_tools() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    println!("--- Test 1: Basic execution with bash tool ---");

    let result = argo_core::execution::execute_task(
        "Use the bash tool to run 'echo hello-from-argo' and return the output",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Task should succeed");

    match result.unwrap() {
        TaskResult::Success { output } => {
            println!("Output: {}", output);
        }
        other => panic!("Expected Success, got {:?}", other),
    }
}

// ─── Test 2: File write + read ──────────────────────────────────────────────

#[tokio::test]
async fn test_2_file_write_and_read() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    println!("--- Test 2: File write and read ---");

    let result = argo_core::execution::execute_task(
        "Write a file at /tmp/argo_test_file.txt with content 'Hello from Argo agent'. Then read it back and confirm the content.",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Task should succeed");

    let content = tokio::fs::read_to_string("/tmp/argo_test_file.txt")
        .await
        .expect("File should exist");
    assert_eq!(content, "Hello from Argo agent");
    println!("File verified: {}", content);
}

// ─── Test 3: LoopAgent with quality scoring ─────────────────────────────────

#[tokio::test]
async fn test_3_loop_agent_with_quality_scoring() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    let loop_agent = LoopAgent::from_config(config, llm, tools, memory);

    println!("--- Test 3: LoopAgent with quality scoring ---");

    let (result, trace) = loop_agent
        .run("Use the bash tool to run 'uname -a' and return the system information")
        .await
        .expect("LoopAgent should not error");

    println!("Iterations: {}", trace.iterations);
    println!("Quality score: {:?}", trace.quality_score);

    match result {
        TaskResult::Success { output } => {
            println!("Output: {}", output);
        }
        other => panic!("Expected Success, got {:?}", other),
    }
}

// ─── Test 4: Memory persistence (Redis + SurrealDB) ─────────────────────────

#[tokio::test]
async fn test_4_memory_persistence() {
    let memory = setup_memory().await.expect("Redis must be running");

    println!("--- Test 4: Memory persistence (Redis + SurrealDB) ---");

    // Test Redis short-term memory
    memory
        .store_context("test-agent", "run-001", "test context data")
        .await
        .expect("Redis store should work");

    let retrieved = memory
        .get_context("test-agent", "run-001")
        .await
        .expect("Redis get should work");

    assert_eq!(retrieved, Some("test context data".to_string()));
    println!("Redis: context stored and retrieved OK");

    // Test SurrealDB long-term memory
    let record = argo_memory::surreal::TaskRecord {
        agent_id: "test-agent".to_string(),
        goal: "test goal".to_string(),
        outcome: "success".to_string(),
        summary: "test summary".to_string(),
        tools_used: vec!["bash".to_string()],
        duration_ms: 1000,
        run_id: "run-002".to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        ended_at: chrono::Utc::now().to_rfc3339(),
    };

    memory
        .store_task_record(&record)
        .await
        .expect("SurrealDB store should work");

    let retrieved = memory
        .get_task_record("run-002")
        .await
        .expect("SurrealDB get should work");

    assert!(retrieved.is_some(), "Task record should exist");
    let rec = retrieved.unwrap();
    assert_eq!(rec.agent_id, "test-agent");
    assert_eq!(rec.outcome, "success");
    println!("SurrealDB: task record stored and retrieved OK");
}

// ─── Test 5: Code tool (write + run Python) ─────────────────────────────────

#[tokio::test]
async fn test_5_code_tool_execution() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    println!("--- Test 5: Code tool (write + run) ---");

    let result = argo_core::execution::execute_task(
        "Use the bash tool to run: python3 -c \"print(2+2)\" and return the output",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Task should succeed");
}

// ─── Test 6: Multi-step with git ────────────────────────────────────────────

#[tokio::test]
async fn test_6_multi_step_with_git() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    println!("--- Test 6: Multi-step with git ---");

    let result = argo_core::execution::execute_task(
        "Create directory /tmp/argo_git_test, init git repo there, write a README.md with '# Test', and run git status",
        llm.as_ref(),
        tools.as_ref(),
        memory.as_ref(),
        &config,
    )
    .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Task should succeed");
}

// ─── Test 7: Full autonomous loop (all features combined) ───────────────────

#[tokio::test]
async fn test_7_full_autonomous_loop() {
    let config = test_config();
    let llm = make_llm(&config);
    let tools = make_tools();
    let memory = Arc::new(setup_memory().await.expect("Redis must be running"));

    let loop_agent = LoopAgent::from_config(config, llm, tools, memory);

    println!("--- Test 7: Full autonomous loop (create project + tests + git) ---");

    let (result, trace) = loop_agent
        .run(
            "Create a small Rust project at /tmp/argo_demo_project:\n\
             1. mkdir -p /tmp/argo_demo_project/src\n\
             2. Write src/lib.rs with a function `pub fn add(a: i32, b: i32) -> i32 { a + b }`\n\
             3. Write Cargo.toml with name 'argo_demo', edition 2021\n\
             4. Write tests/add_test.rs with a test that asserts add(2,3) == 5\n\
             5. Run 'cargo test' in the project directory\n\
             Return the test output.",
        )
        .await
        .expect("LoopAgent should not error");

    println!("Iterations: {}", trace.iterations);
    println!("Quality score: {:?}", trace.quality_score);

    match result {
        TaskResult::Success { output } => {
            println!("Output: {}", output);
        }
        other => panic!("Expected Success, got {:?}", other),
    }
}
