# Phase 2 — Heal Loop & Full Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task.

**Goal:** Build a self-healing agent that recovers from errors, learns from post-mortems, retrieves past experience from semantic memory, and handles context overflow.

**Architecture:** `argo-heal` gets error classifier, 7 strategies, and post-mortem loop. `argo-memory` gets Qdrant semantic memory, embedding pipeline, experience retrieval, and context overflow handling. `argo-core` execution loop integrates heal engine with proper AgentTrace types.

**Tech Stack:** Rust stable 1.75+, Tokio, async-trait, serde, thiserror, qdrant-client, reqwest, rand

## Global Constraints

- Rust stable 1.75+, edition 2021
- `thiserror` for library errors, `anyhow` for app-level propagation
- `async-trait` for async trait definitions
- Mock external services in unit tests; Docker Compose for integration tests
- `cargo fmt --all` and `cargo clippy` must pass before each commit
- Follow schemas in `docs/rfcs/schemas/` and RFCs in `docs/rfcs/architecture/`

---

## Task 1: Heal Types & Error Classifier

**Files:** `crates/argo-heal/src/types.rs` (create), `crates/argo-heal/src/classifier.rs` (create), `crates/argo-heal/src/lib.rs` (modify)

**Produces:** `ErrorSeverity`, `HealContext`, `HealResult`, `HealStep`, `Lesson`, `ErrorClassifier`

- [ ] Create `types.rs` with: `ErrorSeverity` (Recoverable/Degradable/Fatal), `HealStep`, `Lesson`, `HealContext`, `HealResult` — all derive Serialize, Deserialize, Clone, Debug
- [ ] Create `classifier.rs` with `ErrorClassifier::classify(AgentError) -> ErrorSeverity` and `initial_strategy(AgentError) -> &str` — match all 20 AgentError variants per A-03 RFC classification table
- [ ] Update `lib.rs`: add `pub mod classifier; pub mod types;` and re-exports
- [ ] Add classifier unit tests in `classifier/tests.rs`: test recoverable, degradable, fatal classification; test initial_strategy per error type
- [ ] Run `cargo check -p argo-heal && cargo test -p argo-heal`
- [ ] Commit: `feat(heal): add error classifier, types, and severity mapping`

---

## Task 2: HealStrategy Trait & All 7 Strategies

**Files:** `crates/argo-heal/src/strategy/mod.rs` (create), `crates/argo-heal/src/strategy/{retry,reframe,swap_tool,decompose,spawn_subagent,change_provider,reduce_scope}.rs` (create)

**Produces:** `HealStrategy` trait, 7 strategy implementations

- [ ] Create `strategy/mod.rs`: `#[async_trait] pub trait HealStrategy { fn can_handle(&self, error: &AgentError) -> bool; async fn apply(&self, ctx: &HealContext) -> HealResult; fn name(&self) -> &str; }`
- [ ] Create `strategy/retry.rs`: `RetryStrategy { max_retries: 5, base_delay_ms: 1000 }` — exponential backoff with jitter, handles LlmRateLimit/Timeout/ToolTimeout/NetworkTimeout
- [ ] Create `strategy/reframe.rs`: `ReframeStrategy` — generates modified prompt with constraints, handles LlmHallucination/LlmRefusal/ToolOutputInvalid/PlanInvalid
- [ ] Create `strategy/swap_tool.rs`: `SwapToolStrategy` — looks up fallback tools from ToolRegistry, handles ToolNotFound/ToolExecutionFailed/ToolTimeout
- [ ] Create `strategy/decompose.rs`: `DecomposeStrategy` — breaks task into sub-tasks, handles GoalUnachievable/InfiniteLoop/PlanInvalid
- [ ] Create `strategy/spawn_subagent.rs`: `SpawnSubagentStrategy` — delegates to fresh agent context, handles SubAgentFailed
- [ ] Create `strategy/change_provider.rs`: `ChangeProviderStrategy` — switches to next provider from config list, handles LlmProviderDown/LlmRateLimit
- [ ] Create `strategy/reduce_scope.rs`: `ReduceScopeStrategy` — attempts simpler version of goal, handles InfiniteLoop/GoalUnachievable
- [ ] Run `cargo check -p argo-heal`
- [ ] Commit: `feat(heal): add HealStrategy trait and all 7 strategies`

---

## Task 3: HealEngine

**Files:** `crates/argo-heal/src/engine.rs` (create)

**Produces:** `HealEngine` struct that runs strategies in order, queries semantic memory for past resolutions

- [ ] Create `engine.rs`: `HealEngine { strategies: Vec<Box<dyn HealStrategy>>, max_attempts: usize }`
- [ ] Implement `heal(ctx: &HealContext) -> HealResult` — iterates strategies, calls `can_handle` then `apply`, stops at first success
- [ ] Implement `select_strategies_for_error(error: &AgentError) -> Vec<Box<dyn HealStrategy>>` — reorders strategies based on initial_strategy from classifier
- [ ] Add engine unit tests: test strategy ordering, test exhaustion returns failure, test stops at first success
- [ ] Run `cargo check -p argo-heal && cargo test -p argo-heal`
- [ ] Commit: `feat(heal): add HealEngine with strategy chain`

---

## Task 4: Post-Mortem Loop

**Files:** `crates/argo-heal/src/postmortem.rs` (create)

**Produces:** `PostMortem` — generates structured lessons after error resolution

- [ ] Create `postmortem.rs`: `PostMortem { llm: Box<dyn LlmProvider> }`
- [ ] Implement `generate_lesson(error, resolution, context) -> Lesson` — uses LLM to reflect on error cause, resolution, prevention
- [ ] Store lesson in SurrealDB via MemoryHandle and embed + store in Qdrant via QdrantMemory
- [ ] Add post-mortem unit tests with mock LLM
- [ ] Run `cargo check -p argo-heal && cargo test -p argo-heal`
- [ ] Commit: `feat(heal): add post-mortem loop for lesson generation`

---

## Task 5: Qdrant Semantic Memory

**Files:** `crates/argo-memory/src/qdrant.rs` (create), `crates/argo-memory/src/error.rs` (modify), `crates/argo-memory/src/lib.rs` (modify)

**Produces:** `QdrantMemory` — 4 collections per B-03 schema, CRUD operations

- [ ] Add `Qdrant(String)` variant to `MemoryError`
- [ ] Create `qdrant.rs`: `QdrantMemory { client: QdrantClient }` with `new(url)` constructor
- [ ] Implement `ensure_collections()` — create 4 collections (argo_experiences, argo_errors, argo_lessons, argo_tool_patterns) with 1536-dim vectors, cosine distance
- [ ] Implement per-collection operations: `store_experience`, `query_similar_experiences`, `store_error_resolution`, `query_similar_errors`, `store_lesson`, `query_lessons`, `store_tool_pattern`, `query_tool_patterns`
- [ ] Each store operation: generate UUID point, insert with payload. Each query: embed query text, search with top-K, filter by agent_id
- [ ] Update `lib.rs`: add `pub mod qdrant;`
- [ ] Add unit tests (mock Qdrant client): store/get round-trip, query returns sorted results
- [ ] Run `cargo check -p argo-memory && cargo test -p argo-memory`
- [ ] Commit: `feat(memory): add Qdrant semantic memory with 4 collections`

---

## Task 6: Embedding Pipeline

**Files:** `crates/argo-memory/src/embedding.rs` (create), `crates/argo-memory/src/error.rs` (modify), `crates/argo-memory/src/lib.rs` (modify)

**Produces:** `EmbeddingProvider` trait, `OpenAIEmbedding`, `OllamaEmbedding`

- [ ] Add `Embedding(String)` variant to `MemoryError`
- [ ] Create `embedding.rs`: `#[async_trait] pub trait EmbeddingProvider: Send + Sync { async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>; fn dimension(&self) -> usize; fn name(&self) -> &str; }`
- [ ] Implement `OpenAIEmbedding { api_key, model: "text-embedding-3-small" }` — POST to api.openai.com/v1/embeddings
- [ ] Implement `OllamaEmbedding { base_url, model: "nomic-embed-text" }` — POST to {base_url}/api/embeddings
- [ ] Update `lib.rs`: add `pub mod embedding;`
- [ ] Add unit tests: verify request format, dimension = 1536 for OpenAI
- [ ] Run `cargo check -p argo-memory && cargo test -p argo-memory`
- [ ] Commit: `feat(memory): add embedding pipeline with OpenAI and Ollama adapters`

---

## Task 7: Experience Retrieval Pipeline

**Files:** `crates/argo-memory/src/retrieval.rs` (create), `crates/argo-memory/src/lib.rs` (modify)

**Produces:** `ExperienceRetrieval` — embeds task, queries Qdrant + SurrealDB, returns context for system prompt

- [ ] Create `retrieval.rs`: `ExperienceRetrieval { qdrant: QdrantMemory, surreal: SurrealMemory, embedding: Box<dyn EmbeddingProvider> }`
- [ ] Implement `retrieve_context(agent_id, task_description, limit) -> Result<String>` — embed task, query top-5 similar experiences from Qdrant, query related entities from SurrealDB, format into context string
- [ ] Implement `inject_into_prompt(base_prompt, context) -> String` — appends retrieved context under "Past Experience" heading
- [ ] Update `lib.rs`: add `pub mod retrieval;`
- [ ] Add unit tests with mocks: retrieval returns formatted context, empty results returns base prompt
- [ ] Run `cargo check -p argo-memory && cargo test -p argo-memory`
- [ ] Commit: `feat(memory): add experience retrieval pipeline`

---

## Task 8: Context Overflow Handler

**Files:** `crates/argo-memory/src/overflow.rs` (create), `crates/argo-memory/src/lib.rs` (modify)

**Produces:** `ContextOverflowHandler` — detects near-full context, summarizes, archives, compresses

- [ ] Create `overflow.rs`: `ContextOverflowHandler { llm: Box<dyn LlmProvider>, memory: MemoryHandle }`
- [ ] Implement `check_and_handle(agent_id, run_id, context_limit) -> Result<bool>` — estimate token count of turns, if >=80% of limit: find cutoff point, archive full turns to SurrealDB, summarize via LLM, replace old turns with summary in Redis
- [ ] Implement `estimate_tokens(text: &str) -> usize` — simple approximation (words * 1.3)
- [ ] Implement `archive_turns(agent_id, run_id, turns)` — store in SurrealDB under `task:{run_id}:archived_turns`
- [ ] Update `lib.rs`: add `pub mod overflow;`
- [ ] Add unit tests: detection triggers at correct threshold, summarization replaces old turns
- [ ] Run `cargo check -p argo-memory && cargo test -p argo-memory`
- [ ] Commit: `feat(memory): add context overflow handler`

---

## Task 9: Upgrade MemoryHandle with Qdrant + Embedding + Retrieval + Overflow

**Files:** `crates/argo-memory/src/handle.rs` (modify), `crates/argo-memory/src/lib.rs` (modify)

**Produces:** Extended `MemoryHandle` with semantic memory and overflow capabilities

- [ ] Add fields to `MemoryHandle`: `qdrant: Option<QdrantMemory>`, `embedding: Option<Box<dyn EmbeddingProvider>>`
- [ ] Add methods: `store_experience`, `query_similar_experiences`, `store_error_resolution`, `query_similar_errors`, `retrieve_experience_context`, `handle_context_overflow`
- [ ] Each method delegates to the appropriate backend, returns Ok(()) or MemoryError on failure
- [ ] Update `lib.rs` to ensure all modules are public
- [ ] Run `cargo check -p argo-memory && cargo test -p argo-memory`
- [ ] Commit: `feat(memory): extend MemoryHandle with semantic memory and overflow`

---

## Task 10: Upgrade AgentTrace Types

**Files:** `crates/argo-core/src/message.rs` (modify)

**Produces:** Proper `HealStepRecord` and `LessonRecord` types replacing `serde_json::Value`

- [ ] Create `HealStepRecord { id, error: AgentError, strategy_name, started_at, ended_at, success, output }`
- [ ] Create `LessonRecord { id, error_type, context_summary, root_cause, resolution, prevention, confidence, created_at }`
- [ ] Replace `heal_steps: Vec<serde_json::Value>` with `heal_steps: Vec<HealStepRecord>`
- [ ] Replace `lessons: Vec<serde_json::Value>` with `lessons: Vec<LessonRecord>`
- [ ] Add `memory_ops: Vec<MemoryOpRecord>` field to AgentTrace
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): upgrade AgentTrace with typed heal and lesson records`

---

## Task 11: Integrate Heal Engine into Execution Loop

**Files:** `crates/argo-core/src/execution.rs` (modify), `crates/argo-core/Cargo.toml` (modify)

**Produces:** Execution loop that catches errors, runs heal engine, retries, and records heal steps

- [ ] Add `argo-heal` dependency to `crates/argo-core/Cargo.toml`
- [ ] Modify `execute_task`: wrap LLM call in match — on error, classify severity, if Recoverable/Degradable: create HealContext, call HealEngine, on Success: record HealStep and continue loop, on Failure: record and return error
- [ ] Modify tool execution: on ToolError, create HealContext, attempt heal via swap_tool strategy
- [ ] Record all HealStep and Lesson results into AgentTrace
- [ ] Add experience retrieval: before first LLM call, call `memory.retrieve_experience_context()` and inject into system prompt
- [ ] Add context overflow check: before each LLM call, call `memory.handle_context_overflow()`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): integrate heal engine and experience retrieval into execution loop`

---

## Task 12: Unit Tests for Heal + Memory

**Files:** Add test modules to all modified files

- [ ] `argo-heal`: classifier tests (10 cases), strategy tests (each strategy can_handle + apply), engine tests (ordering, exhaustion), post-mortem tests (lesson generation)
- [ ] `argo-memory`: qdrant tests (store/get round-trip for each collection), embedding tests (request format), retrieval tests (context formatting), overflow tests (threshold detection)
- [ ] `argo-core`: execution loop tests (heal from ToolExecutionFailed, heal from LlmRateLimit, experience retrieval injection)
- [ ] Run `cargo test --workspace`
- [ ] Commit: `test: add comprehensive unit tests for Phase 2`

---

## Task 13: Final Verification

- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets --all-features`
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo build --workspace`
- [ ] Fix any issues found
- [ ] Final commit if fixes needed
