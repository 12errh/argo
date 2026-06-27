# A-11: Observability Contract

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define OTel span naming conventions, metric definitions, log schema, and trace hierarchy for Argo's observability system.

## Motivation

Production agents need observability for debugging, performance monitoring, and understanding agent behavior. OpenTelemetry provides vendor-neutral instrumentation.

## Detailed Design

### Span Hierarchy

```
argo.agent.run (root span)
├── argo.memory.retrieval
│   ├── argo.memory.qdrant.query
│   └── argo.memory.surreal.query
├── argo.llm.complete
├── argo.tool.execute (per tool call)
│   └── argo.tool.{tool_name}
├── argo.heal.attempt (if healing triggered)
│   └── argo.heal.strategy.{strategy_name}
├── argo.memory.store
└── argo.agent.complete
```

### Span Attributes

```rust
// Root span
span.set_attribute("argo.agent.name", agent_name);
span.set_attribute("argo.agent.id", agent_id);
span.set_attribute("argo.run.id", run_id);
span.set_attribute("argo.task.goal", goal);
span.set_attribute("argo.task.iteration", iteration);

// LLM span
span.set_attribute("argo.llm.provider", provider);
span.set_attribute("argo.llm.model", model);
span.set_attribute("argo.llm.input_tokens", input_tokens);
span.set_attribute("argo.llm.output_tokens", output_tokens);
span.set_attribute("argo.llm.duration_ms", duration_ms);

// Tool span
span.set_attribute("argo.tool.name", tool_name);
span.set_attribute("argo.tool.input_size", input_size);
span.set_attribute("argo.tool.output_size", output_size);
span.set_attribute("argo.tool.success", success);

// Heal span
span.set_attribute("argo.heal.error_type", error_type);
span.set_attribute("argo.heal.strategy", strategy);
span.set_attribute("argo.heal.success", success);
span.set_attribute("argo.heal.attempt_number", attempt);

// Memory span
span.set_attribute("argo.memory.store", store);
span.set_attribute("argo.memory.operation", op);
span.set_attribute("argo.memory.key", key);
```

### Metrics

```rust
// Histograms
metrics::histogram!("argo.task.duration", "ms");
metrics::histogram!("argo.tool.latency", "ms");
metrics::histogram!("argo.llm.latency", "ms");
metrics::histogram!("argo.memory.query_latency", "ms");

// Counters
metrics::counter!("argo.task.total");
metrics::counter!("argo.task.success");
metrics::counter!("argo.task.failure");
metrics::counter!("argo.heal.attempts", "strategy" => strategy_name);
metrics::counter!("argo.heal.success", "strategy" => strategy_name);
metrics::counter!("argo.memory.queries", "store" => store_name);
metrics::counter!("argo.llm.tokens", "provider" => provider, "type" => token_type);
metrics::counter!("argo.tool.calls", "tool" => tool_name);

// Gauges
metrics::gauge!("argo.agent.active_agents");
metrics::gauge!("argo.memory.short_term_keys");
metrics::gauge!("argo.memory.long_term_records");
```

### Log Schema

```rust
#[derive(Debug, Serialize)]
pub struct StructuredLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: tracing::Level,
    pub message: String,
    pub run_id: Option<String>,
    pub agent_name: Option<String>,
    pub span_id: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
}
```

### AgentTrace

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrace {
    pub run_id: Uuid,
    pub agent_name: String,
    pub goal: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub output: Option<String>,
    pub iterations: usize,
    pub quality_score: Option<f32>,

    pub tool_calls: Vec<ToolCallRecord>,
    pub llm_calls: Vec<LlmCallRecord>,
    pub memory_ops: Vec<MemoryOpRecord>,
    pub heal_steps: Vec<HealStepRecord>,
    pub lessons: Vec<LessonRecord>,
    pub errors: Vec<AgentError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub call_id: Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallRecord {
    pub provider: String,
    pub model: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealStepRecord {
    pub error: AgentError,
    pub strategy: String,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

## Alternatives Considered

1. **Custom logging only**: Simpler, but no standard format or backend integration.
2. **Prometheus metrics only**: Good for metrics, but no distributed tracing.
3. **Jaeger-native**: Vendor-specific, not portable.

## Drawbacks

- OTel adds dependencies and configuration complexity
- Instrumentation adds some overhead
- Span hierarchy must be maintained manually

## Unresolved Questions

- Should we provide a built-in dashboard or rely on external tools?
- How to handle sensitive data in traces (PII, secrets)?
- Should metrics be exposed via Prometheus endpoint?
