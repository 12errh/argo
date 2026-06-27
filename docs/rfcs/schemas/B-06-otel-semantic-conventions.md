# B-06: OTel Semantic Conventions

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Overview

OpenTelemetry span names, attribute keys, metric names, and units used by Argo.

## Span Names

| Span Name | Parent | Description |
|---|---|---|
| `argo.agent.run` | (root) | Complete agent execution |
| `argo.memory.retrieval` | `argo.agent.run` | Memory retrieval before task |
| `argo.memory.qdrant.query` | `argo.memory.retrieval` | Qdrant vector search |
| `argo.memory.surreal.query` | `argo.memory.retrieval` | SurrealDB relational query |
| `argo.llm.complete` | `argo.agent.run` | LLM completion call |
| `argo.llm.stream` | `argo.agent.run` | LLM streaming call |
| `argo.tool.execute` | `argo.agent.run` | Tool execution (generic) |
| `argo.tool.{name}` | `argo.tool.execute` | Specific tool execution |
| `argo.heal.attempt` | `argo.agent.run` | Heal strategy attempt |
| `argo.heal.strategy.{name}` | `argo.heal.attempt` | Specific strategy execution |
| `argo.memory.store` | `argo.agent.run` | Memory write operation |
| `argo.agent.complete` | `argo.agent.run` | Agent completion processing |

## Span Attributes

### Agent Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.agent.name` | string | Agent name |
| `argo.agent.id` | string | Agent UUID |
| `argo.agent.model` | string | LLM model name |
| `argo.agent.provider` | string | LLM provider name |

### Run Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.run.id` | string | Run UUID |
| `argo.run.goal` | string | Task goal (truncated to 1000 chars) |
| `argo.run.iteration` | int | Current iteration number |
| `argo.run.success` | bool | Whether run succeeded |
| `argo.run.duration_ms` | int | Total run duration |

### LLM Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.llm.provider` | string | Provider name |
| `argo.llm.model` | string | Model name |
| `argo.llm.input_tokens` | int | Input token count |
| `argo.llm.output_tokens` | int | Output token count |
| `argo.llm.total_tokens` | int | Total token count |
| `argo.llm.temperature` | float | Temperature setting |
| `argo.llm.max_tokens` | int | Max tokens setting |
| `argo.llm.stop_reason` | string | Stop reason |

### Tool Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.tool.name` | string | Tool name |
| `argo.tool.call_id` | string | Tool call UUID |
| `argo.tool.success` | bool | Whether tool call succeeded |
| `argo.tool.input_size` | int | Input size in bytes |
| `argo.tool.output_size` | int | Output size in bytes |
| `argo.tool.error` | string | Error message if failed |

### Heal Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.heal.error_type` | string | Error type classification |
| `argo.heal.strategy` | string | Strategy name |
| `argo.heal.success` | bool | Whether strategy succeeded |
| `argo.heal.attempt_number` | int | Attempt number in heal chain |
| `argo.heal.total_attempts` | int | Total strategies attempted |

### Memory Attributes

| Attribute | Type | Description |
|---|---|---|
| `argo.memory.store` | string | Storage backend (redis, surreal, qdrant) |
| `argo.memory.operation` | string | Operation type (read, write, delete) |
| `argo.memory.key` | string | Memory key |
| `argo.memory.namespace` | string | Memory namespace |
| `argo.memory.result_count` | int | Number of results returned |

## Metrics

### Counters

| Metric Name | Unit | Labels | Description |
|---|---|---|---|
| `argo.task.total` | 1 | agent_name, provider | Total tasks executed |
| `argo.task.success` | 1 | agent_name, provider | Successful tasks |
| `argo.task.failure` | 1 | agent_name, provider, error_type | Failed tasks |
| `argo.heal.attempts` | 1 | strategy, error_type | Heal strategy attempts |
| `argo.heal.success` | 1 | strategy, error_type | Successful heal attempts |
| `argo.memory.queries` | 1 | store, operation | Memory operations |
| `argo.llm.tokens` | 1 | provider, model, type | Tokens used (input/output) |
| `argo.tool.calls` | 1 | tool_name, success | Tool calls |
| `argo.agent.spawns` | 1 | agent_name | Sub-agents spawned |

### Histograms

| Metric Name | Unit | Labels | Description |
|---|---|---|---|
| `argo.task.duration` | ms | agent_name, provider | Task completion time |
| `argo.tool.latency` | ms | tool_name | Tool execution time |
| `argo.llm.latency` | ms | provider, model | LLM call time |
| `argo.memory.query_latency` | ms | store | Memory query time |
| `argo.heal.latency` | ms | strategy | Heal strategy execution time |

### Gauges

| Metric Name | Unit | Labels | Description |
|---|---|---|---|
| `argo.agent.active_agents` | 1 | - | Currently running agents |
| `argo.memory.short_term_keys` | 1 | agent_name | Active short-term memory keys |
| `argo.memory.long_term_records` | 1 | agent_name | Long-term memory records |
| `argo.memory.vector_count` | 1 | collection | Vectors in Qdrant collection |

## Log Schema

```json
{
  "timestamp": "2026-06-27T19:00:00.000Z",
  "level": "INFO",
  "message": "Tool execution completed",
  "target": "argo_tools::bash",
  "span_id": "abc123",
  "trace_id": "def456",
  "agent_name": "coder",
  "run_id": "550e8400-e29b-41d4-a716-446655440000",
  "fields": {
    "tool": "bash",
    "success": true,
    "duration_ms": 150
  }
}
```
