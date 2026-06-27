# A-02: Memory Architecture

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Summary

Define the three-layer memory system (Redis short-term, SurrealDB long-term, Qdrant semantic), key schemas, TTL policies, context window overflow handling, and the memory retrieval pipeline.

## Motivation

Agents need memory at different timescales: working context for the current task (fast, ephemeral), permanent records of past work (durable, queryable), and experience retrieval by meaning (semantic search). A single storage system cannot optimally serve all three needs.

## Detailed Design

### Layer 1: Short-Term Memory (Redis)

**Purpose:** Active working context for the current task.

**Key Patterns:**

```
argo:agent:{agent_id}:run:{run_id}:context    → String (JSON blob)
argo:agent:{agent_id}:run:{run_id}:turns      → List (LLM conversation turns)
argo:agent:{agent_id}:run:{run_id}:scratch    → String (agent scratchpad)
argo:agent:{agent_id}:run:{run_id}:plan       → String (current plan JSON)
```

**TTL Policies:**

| Key Pattern | TTL | Reason |
|---|---|---|
| `:context` | Task duration + 1h | Keep context available briefly after task ends for inspection |
| `:turns` | Task duration + 1h | Same as context |
| `:scratch` | Task duration + 1h | Same as context |
| `:plan` | Task duration + 1h | Same as context |

**Operations:**

```rust
#[async_trait]
pub trait ShortTermMemory: Send + Sync {
    async fn store_context(&self, agent_id: &str, run_id: &str, context: &str) -> Result<()>;
    async fn get_context(&self, agent_id: &str, run_id: &str) -> Result<Option<String>>;
    async fn store_turns(&self, agent_id: &str, run_id: &str, turns: &[Turn]) -> Result<()>;
    async fn get_turns(&self, agent_id: &str, run_id: &str) -> Result<Vec<Turn>>;
    async fn store_scratch(&self, agent_id: &str, run_id: &str, data: &str) -> Result<()>;
    async fn get_scratch(&self, agent_id: &str, run_id: &str) -> Result<Option<String>>;
    async fn store_plan(&self, agent_id: &str, run_id: &str, plan: &str) -> Result<()>;
    async fn get_plan(&self, agent_id: &str, run_id: &str) -> Result<Option<String>>;
    async fn cleanup(&self, agent_id: &str, run_id: &str) -> Result<()>;
}
```

### Layer 2: Long-Term Memory (SurrealDB)

**Purpose:** Permanent record of what the agent has done, learned, and encountered.

**Tables:**

```sql
-- Task record
DEFINE TABLE task SCHEMAFULL;
DEFINE FIELD agent_id    ON task TYPE string;
DEFINE FIELD goal        ON task TYPE string;
DEFINE FIELD outcome     ON task TYPE string; -- success | partial | failed
DEFINE FIELD summary     ON task TYPE string;
DEFINE FIELD tools_used  ON task TYPE array;
DEFINE FIELD duration_ms ON task TYPE int;
DEFINE FIELD started_at  ON task TYPE datetime;
DEFINE FIELD ended_at    ON task TYPE datetime;
DEFINE FIELD run_id      ON task TYPE string;

-- Entity record
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD type        ON entity TYPE string; -- file | api | repo | person
DEFINE FIELD identifier  ON entity TYPE string;
DEFINE FIELD metadata    ON entity TYPE object;

-- Relationship (graph edge)
DEFINE TABLE interacted_with SCHEMAFULL;

-- Error record
DEFINE TABLE error_record SCHEMAFULL;
DEFINE FIELD task_id     ON error_record TYPE string;
DEFINE FIELD error_type  ON error_record TYPE string;
DEFINE FIELD message     ON error_record TYPE string;
DEFINE FIELD resolution  ON option<string>;
DEFINE FIELD strategy    ON option<string>;
DEFINE FIELD occurred_at ON error_record TYPE datetime;

-- Lesson record
DEFINE TABLE lesson SCHEMAFULL;
DEFINE FIELD error_type      ON lesson TYPE string;
DEFINE FIELD context_summary ON lesson TYPE string;
DEFINE FIELD root_cause      ON lesson TYPE string;
DEFINE FIELD resolution      ON lesson TYPE string;
DEFINE FIELD prevention      ON lesson TYPE string;
DEFINE FIELD confidence      ON lesson TYPE float;
DEFINE FIELD created_at      ON lesson TYPE datetime;

-- Agent record
DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD name        ON agent TYPE string;
DEFINE FIELD model       ON agent TYPE string;
DEFINE FIELD config      ON agent TYPE object;
DEFINE FIELD created_at  ON agent TYPE datetime;
```

**Operations:**

```rust
#[async_trait]
pub trait LongTermMemory: Send + Sync {
    async fn store_task_record(&self, record: &TaskRecord) -> Result<()>;
    async fn get_task_record(&self, run_id: &str) -> Result<Option<TaskRecord>>;
    async fn store_entity(&self, entity: &Entity) -> Result<()>;
    async fn get_entity(&self, entity_type: &str, identifier: &str) -> Result<Option<Entity>>;
    async fn create_relationship(&self, from: &str, to: &str, rel_type: &str) -> Result<()>;
    async fn query_relationships(&self, entity_id: &str, rel_type: &str) -> Result<Vec<Entity>>;
    async fn store_error_record(&self, record: &ErrorRecord) -> Result<()>;
    async fn store_lesson(&self, lesson: &Lesson) -> Result<()>;
    async fn query_lessons(&self, error_type: &str, limit: usize) -> Result<Vec<Lesson>>;
}
```

### Layer 3: Semantic Memory (Qdrant)

**Purpose:** Experience retrieval by meaning. Search past experience before acting.

**Collections:**

| Collection | Vector Dimension | Payload Fields |
|---|---|---|
| `argo_experiences` | 1536 | task_summary, outcome, tools_used, duration_ms |
| `argo_errors` | 1536 | error_type, context_summary, resolution, strategy |
| `argo_lessons` | 1536 | error_type, root_cause, prevention, confidence |
| `argo_tool_patterns` | 1536 | tool_name, task_type, success_rate, avg_duration_ms |

**Operations:**

```rust
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    async fn store_experience(&self, embedding: &[f32], metadata: &ExperienceMetadata) -> Result<()>;
    async fn query_similar_experiences(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<Experience>>;
    async fn store_error_resolution(&self, embedding: &[f32], metadata: &ErrorMetadata) -> Result<()>;
    async fn query_similar_errors(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<ErrorResolution>>;
    async fn store_lesson(&self, embedding: &[f32], metadata: &LessonMetadata) -> Result<()>;
    async fn query_lessons(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<Lesson>>;
}
```

### Memory Retrieval Pipeline

```
Agent receives task
        │
        ▼
Embed task description (text-embedding-3-small)
        │
        ▼
Query Qdrant: top-5 similar past tasks
        │
        ▼
Query SurrealDB: related entities and decisions
        │
        ▼
Inject retrieved context into system prompt
        │
        ▼
Agent executes with full historical context
```

### Context Window Overflow Handling

When context window is nearly full (≥80% of limit):

1. Detect via token count estimation
2. Take oldest N turns (N = enough to bring usage below 60%)
3. Call LLM to summarize those turns into a dense paragraph
4. Store full turns in SurrealDB: `task:{run_id}:archived_turns`
5. Replace old turns in Redis with the summary
6. Agent continues with compressed but complete understanding

```rust
pub async fn handle_context_overflow(
    memory: &MemoryHandle,
    llm: &dyn LlmProvider,
    agent_id: &str,
    run_id: &str,
    context_limit: usize,
) -> Result<()> {
    let turns = memory.get_turns(agent_id, run_id).await?;
    let current_tokens = estimate_tokens(&turns);

    if current_tokens < context_limit as f64 * 0.8 {
        return Ok(());
    }

    let mut tokens_to_remove = 0;
    let mut cutoff = 0;
    for (i, turn) in turns.iter().enumerate() {
        tokens_to_remove += estimate_tokens_single(turn);
        cutoff = i;
        if current_tokens - tokens_to_remove < context_limit as f64 * 0.6 {
            break;
        }
    }

    let archived = turns[..=cutoff].to_vec();
    memory.archive_turns(agent_id, run_id, &archived).await?;

    let summary = llm.summarize(&archived).await?;

    let remaining = &turns[cutoff + 1..];
    let mut new_turns = vec![Turn::Summary(summary)];
    new_turns.extend_from_slice(remaining);
    memory.store_turns(agent_id, run_id, &new_turns).await?;

    Ok(())
}
```

## Alternatives Considered

1. **Single PostgreSQL database**: Simpler, but no graph relationships, no vector search, worse performance for short-term memory.
2. **SQLite for long-term**: Embedded, no server needed, but no graph capabilities.
3. **Redis for all layers**: Fast, but no persistent storage, no vector search, limited query capabilities.

## Drawbacks

- Three external services to run (Redis, SurrealDB, Qdrant) increases deployment complexity
- SurrealDB 2.x is relatively new, API may change
- Embedding pipeline adds latency to task startup

## Unresolved Questions

- Should short-term memory support pub/sub for real-time agent communication?
- How to handle SurrealDB schema migrations across versions?
- Should we provide an embedded alternative (SQLite + sqlite-vss) for single-agent deployments?
