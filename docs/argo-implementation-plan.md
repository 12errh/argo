# Argo Agent Framework — Detailed Implementation Plan

**Version:** 0.1.0  
**Status:** Pre-development  
**Based on:** argo-master-plan.md  
**Total Duration Estimate:** 38–44 weeks  

---

## Table of Contents

1. [Implementation Overview](#1-implementation-overview)
2. [Phase 0 — Planning & RFCs](#2-phase-0--planning--rfcs)
3. [Phase 1 — Rust Core Engine](#3-phase-1--rust-core-engine)
4. [Phase 2 — Heal Loop & Full Memory](#4-phase-2--heal-loop--full-memory)
5. [Phase 3 — SDKs & CLI](#5-phase-3--sdks--cli)
6. [Phase 4 — Multi-Agent & MCP](#6-phase-4--multi-agent--mcp)
7. [Phase 5 — Evolution & Production Polish](#7-phase-5--evolution--production-polish)
8. [Testing Strategy](#8-testing-strategy)
9. [Additional Documentation Required](#9-additional-documentation-required)
10. [Risk Register](#10-risk-register)

---

## 1. Implementation Overview

### 1.1 Phase Dependency Chain

```
Phase 0 (Planning)
    │
    ▼
Phase 1 (Rust Core) ─────────────────────────────────┐
    │                                                  │
    ├── Phase 2 (Heal + Memory) ──────────────────────┤
    │       │                                          │
    │       ├── Phase 3 (SDKs + CLI) ─────────────────┤
    │       │       │                                  │
    │       │       ├── Phase 4 (Multi-Agent + MCP) ──┤
    │       │       │       │                          │
    │       │       │       └── Phase 5 (Production) ──┘
    │       │       │
    │       │       └── Phase 2.5 (MCP Foundation) ──┘ (parallel with Phase 3)
    │       │
    │       └── Phase 2.5 (Vector Memory) ────────────┘ (parallel with Phase 3)
    │
    └── Infrastructure Setup ──────────────────────────┘ (parallel throughout)
```

### 1.2 Crate Dependency Graph

```
argo (meta-crate, re-exports all)
├── argo-core        ← actor engine, message types, supervisor tree
├── argo-memory      ← Redis, SurrealDB, Qdrant layers
├── argo-heal        ← error taxonomy, strategies, post-mortem
├── argo-tools       ← built-in tool library
├── argo-mcp         ← MCP protocol connector
├── argo-observe     ← OpenTelemetry integration
└── argo-cli         ← CLI binary
```

---

## 2. Phase 0 — Planning & RFCs

**Duration:** 4–6 weeks  
**Goal:** All architecture decisions locked in writing before any code is written.

### 2.1 Tasks

| Task ID | Task | Details | Owner |
|---------|------|---------|-------|
| P0-T01 | Write A-01: Actor Engine Design RFC | Actix actor hierarchy, message types (ExecuteTask, ToolCall, ToolResult, MemoryRead, MemoryWrite, HealRequest, SpawnAgent, AgentDone, AgentFailed, InspectRequest), supervisor tree (SupervisorActor → OrchestratorActor → WorkerAgent_N), restart policies | Architect |
| P0-T02 | Write A-02: Memory Architecture RFC | Three-layer design (Redis short-term, SurrealDB long-term, Qdrant semantic), key schemas per layer, TTL policies, context window overflow handling algorithm, memory retrieval pipeline | Architect |
| P0-T03 | Write A-03: Error Taxonomy RFC | Complete AgentError enum (LlmRateLimit, LlmContextOverflow, LlmHallucination, LlmRefusal, LlmTimeout, LlmProviderDown, ToolNotFound, ToolExecutionFailed, ToolTimeout, ToolPermissionDenied, ToolOutputInvalid, InfiniteLoop, GoalUnachievable, PlanInvalid, ContextCorrupted, MemoryUnavailable, McpConnectionFailed, NetworkTimeout, SubAgentFailed, OrchestratorFailed), classification rules, metadata requirements | Architect |
| P0-T04 | Write A-04: Heal Strategy Specification RFC | 7 strategies (retry with backoff, reframe prompt, swap tool, decompose task, spawn sub-agent, change LLM provider, reduce scope), trigger conditions, algorithm per strategy, success criteria, failure escalation behavior | Architect |
| P0-T05 | Write A-05: LLM Provider Trait RFC | Full LlmProvider trait (complete, stream methods), CompletionRequest/CompletionResponse types, LlmError variants, streaming contract (BoxStream<Token>), adapter requirements for Anthropic, OpenAI, Gemini, Ollama, custom OpenAI-compat | Architect |
| P0-T06 | Write A-06: Tool Trait & Registry RFC | Tool trait (name, description, input_schema, output_schema, permissions, execute), ToolPermissions model (allow_filesystem, allow_network, allow_subprocess, working_directory, max_execution_time, allowed_domains), hot-reload protocol, fallback registration | Architect |
| P0-T07 | Write A-07: MCP Connector RFC | MCP client protocol implementation, tool discovery at startup, tool registration under server namespace, auth handling (bearer, OAuth2), format conversion between Argo and MCP, reconnection on server disconnect | Architect |
| P0-T08 | Write A-08: Multi-Agent Protocol RFC | Orchestrator/worker message types (AssignTask, TaskComplete, TaskFailed), agent spawning protocol, result aggregation, AgentPool task distribution, shared vs isolated memory modes | Architect |
| P0-T09 | Write A-09: Self-Improvement System RFC | Daily growth cycle algorithm (pull errors → detect patterns → generate proposals → auto-apply low-risk / flag high-risk), pattern detection rules (same error 3+ times, same tool failing, same task type succeeding), improvement proposal schema | Architect |
| P0-T10 | Write A-10: Loop Agent & Scoring RFC | QualityRubric schema (criteria with name/weight/description, threshold, max_iterations), scoring algorithm, iteration management, termination conditions (threshold met or max_iterations hit) | Architect |
| P0-T11 | Write A-11: Observability Contract RFC | OTel span naming conventions, metric definitions (argo.task.duration, argo.task.success_rate, argo.heal.attempts, argo.memory.queries, argo.llm.tokens, argo.tool.latency), log schema (structured JSON with run_id, agent_name, timestamp, level, message), trace hierarchy | Architect |
| P0-T12 | Write A-12: Security Model RFC | Tool sandboxing implementation, secret management (${ENV_VAR} pattern), MCP auth handling, agent isolation rules, permission enforcement at runtime | Architect |
| P0-T13 | Write B-01: SurrealDB Schema | SCHEMAFULL definitions: task (agent_id, goal, outcome, summary, tools_used, duration_ms, started_at, ended_at, run_id), entity (type, identifier, metadata), interacted_with (graph edge), lesson, error_record, agent | Data |
| P0-T14 | Write B-02: Redis Key Schema | All key patterns: argo:agent:{agent_id}:run:{run_id}:context, :turns, :scratch, :plan, TTL policies per key type | Data |
| P0-T15 | Write B-03: Qdrant Collection Schema | Collections: argo_experiences (task summaries), argo_errors (error+resolution pairs), argo_lessons (post-mortem lessons), argo_tool_patterns (successful tool usage), vector dimensions, payload fields | Data |
| P0-T16 | Write B-04: Agent Config TOML Schema | Complete schema: [agent], [model], [memory], [heal], [quality], [tools], [permissions], [observe] sections with all fields, types, defaults, validation rules | Data |
| P0-T17 | Write B-05: MessagePack Message Catalog | Every message type (ExecuteTask, ToolCall, ToolResult, MemoryRead, MemoryWrite, HealRequest, SpawnAgent, AgentDone, AgentFailed, InspectRequest) with field definitions and versioning | Data |
| P0-T18 | Write B-06: OTel Semantic Conventions | All span names, attribute keys, metric names, units used by Argo | Data |
| P0-T19 | Set up GitHub org and repositories | Create argo-agents org, argo repo, branch protection, CODEOWNERS | DevOps |
| P0-T20 | Set up CI/CD pipeline skeleton | GitHub Actions: cargo fmt check, cargo clippy, cargo test, build matrix (Linux/macOS/Windows) | DevOps |
| P0-T21 | Write CONTRIBUTING.md | PR process, commit conventions, code review rules, issue templates | Community |
| P0-T22 | Set up Discord server | Channels: #general, #dev, #rfcs, #help, #announcements | Community |
| P0-T23 | Write Decision Log (F-05) | Record of all architectural decisions and rationale | Architect |

### 2.2 Phase 0 Tests

| Test ID | Test Type | Description |
|---------|-----------|-------------|
| P0-TEST-01 | Schema Validation | Verify all B-series schema documents are syntactically valid (SurrealDB SQL, TOML schema, Qdrant config) |
| P0-TEST-02 | RFC Consistency Check | Cross-reference all A-series RFCs for contradictions (e.g., message types in A-01 match A-05 provider trait error types) |
| P0-TEST-03 | Config Schema Test | Parse the TOML schema document and verify all fields have types, defaults, and validation rules |

### 2.3 Phase 0 Outcome

- **12 Architecture RFCs** (A-01 through A-12) fully written and reviewed
- **6 Data Schema documents** (B-01 through B-06) fully written and validated
- GitHub org created with repo, branch protection, CI skeleton green
- CONTRIBUTING.md and issue/PR templates published
- Discord server operational
- Decision Log started

---

## 3. Phase 1 — Rust Core Engine

**Duration:** 8–10 weeks  
**Goal:** A Rust agent that can run a goal, call tools, and store results in long-term memory. No healing, no semantic memory yet.

### 3.1 Tasks

| Task ID | Task | Crate | Details |
|---------|------|-------|---------|
| P1-T01 | Project scaffolding | argo (workspace) | Create Cargo workspace with all 7 crates (argo, argo-core, argo-memory, argo-heal, argo-tools, argo-mcp, argo-observe, argo-cli), set up dependencies per Tech Stack section (tokio, actix, axum, serde, rmp-serde, async-trait, thiserror, anyhow, clap, uuid, chrono, tracing) |
| P1-T02 | Actor engine | argo-core | Implement AgentActor with Actix, typed message enums (ExecuteTask, ToolCall, ToolResult, MemoryRead, MemoryWrite, HealRequest, SpawnAgent, AgentDone, AgentFailed, InspectRequest), Context<Self> setup |
| P1-T03 | Supervisor tree | argo-core | SupervisorActor, OrchestratorActor (stub for now), restart policies (one_for_one, rest_for_one, one_for_all), actor failure notification handling |
| P1-T04 | LLM Provider trait | argo-core | Full trait: complete(CompletionRequest) → Result<CompletionResponse>, stream(CompletionRequest) → Result<BoxStream<Token>>, model_name(), context_limit(). CompletionRequest (messages, model, temperature, max_tokens), CompletionResponse (content, usage, stop_reason), LlmError variants |
| P1-T05 | Anthropic adapter | argo-core | Implement LlmProvider for Anthropic Claude (claude-sonnet-4-6, claude-opus-4-6, etc.), API key from env, streaming support, rate limit handling |
| P1-T06 | OpenAI adapter | argo-core | Implement LlmProvider for OpenAI (gpt-4o, gpt-4-turbo, etc.), API key from env, streaming support |
| P1-T07 | Basic agent execution loop | argo-core | Agent receives goal → builds prompt → calls LLM → parses tool calls → executes tools → feeds results back → loops until LLM returns final answer. No memory, no healing. |
| P1-T08 | Message serialization | argo-core | MessagePack (rmp-serde) serialization for all message types, binary encoding over actor mailbox |
| P1-T09 | Redis short-term memory | argo-memory | Redis connection pool, key patterns per B-02, operations: store_context, get_context, store_turns, get_turns, store_scratch, get_scratch, store_plan, get_plan. TTL enforcement. |
| P1-T10 | SurrealDB long-term memory | argo-memory | SurrealDB connection, schema initialization per B-01, operations: store_task_record, get_task_record, store_entity, get_entity, create_relationship, query_relationships. |
| P1-T11 | Memory handle abstraction | argo-memory | MemoryHandle trait that wraps Redis + SurrealDB, provides unified interface for agent actors |
| P1-T12 | Tool trait | argo-tools | Full Tool trait: name(), description(), input_schema(), output_schema(), permissions(), execute(input, ctx). ToolContext (agent_id, run_id, working_dir). ToolPermissions struct. ToolError type. |
| P1-T13 | Bash tool | argo-tools | Shell command execution with sandboxing (working_directory restriction, max_execution_time), stdout/stderr capture |
| P1-T14 | Files tool | argo-tools | Read, write, list, delete files. Permission: allowed_paths restriction. |
| P1-T15 | HTTP tool | argo-tools | HTTP requests with method/url/headers/body, response capture. Permission: allowed_domains restriction. |
| P1-T16 | Tool registry | argo-tools | ToolRegistry: HashMap<String, Arc<dyn Tool>>, register/unregister/lookup, fallbacks map |
| P1-T17 | Basic tracing | argo-observe | OpenTelemetry setup: TracerProvider, MeterProvider, LoggerProvider. Root span for agent run, child spans for tool calls and LLM calls. |
| P1-T18 | Structured logging | argo-observe | tracing-subscriber with JSON output, run_id and agent_name in span context |
| P1-T19 | Config parser | argo-core | TOML config parsing per B-04 schema, environment variable substitution (${VAR} → env value), validation at startup |
| P1-T20 | Integration test: single agent | tests/ | Agent runs goal "write a function that adds two numbers", calls files tool to write file, verifies output. End-to-end with real Redis and SurrealDB (Docker Compose for test). |

### 3.2 Phase 1 Unit Tests

| Test ID | Test File | Description |
|---------|-----------|-------------|
| P1-UT-01 | argo-core/src/actor/tests.rs | AgentActor processes ExecuteTask message, returns TaskResult |
| P1-UT-02 | argo-core/src/actor/tests.rs | SupervisorActor restarts crashed worker actor |
| P1-UT-03 | argo-core/src/llm/tests.rs | LlmProvider trait contract: mock provider returns CompletionResponse |
| P1-UT-04 | argo-core/src/llm/tests.rs | Anthropic adapter serializes request correctly (verify JSON structure) |
| P1-UT-05 | argo-core/src/llm/tests.rs | OpenAI adapter serializes request correctly |
| P1-UT-06 | argo-core/src/execution/tests.rs | Basic execution loop: goal → tool call → result → final answer |
| P1-UT-07 | argo-core/src/execution/tests.rs | Execution loop handles LLM returning no tool calls (direct answer) |
| P1-UT-08 | argo-core/src/config/tests.rs | TOML config parsing with all fields |
| P1-UT-09 | argo-core/src/config/tests.rs | Environment variable substitution works |
| P1-UT-10 | argo-core/src/config/tests.rs | Missing required field produces validation error |
| P1-UT-11 | argo-memory/src/redis/tests.rs | store_context / get_context round-trip |
| P1-UT-12 | argo-memory/src/redis/tests.rs | TTL expiration: key disappears after TTL |
| P1-UT-13 | argo-memory/src/redis/tests.rs | store_turns / get_turns round-trip |
| P1-UT-14 | argo-memory/src/surreal/tests.rs | store_task_record / get_task_record round-trip |
| P1-UT-15 | argo-memory/src/surreal/tests.rs | store_entity / get_entity round-trip |
| P1-UT-16 | argo-memory/src/surreal/tests.rs | create_relationship / query_relationships returns connected entities |
| P1-UT-17 | argo-tools/src/bash/tests.rs | Bash tool executes command, returns stdout |
| P1-UT-18 | argo-tools/src/bash/tests.rs | Bash tool respects max_execution_time (kills long command) |
| P1-UT-19 | argo-tools/src/files/tests.rs | Files tool reads existing file |
| P1-UT-20 | argo-tools/src/files/tests.rs | Files tool writes file to allowed path |
| P1-UT-21 | argo-tools/src/files/tests.rs | Files tool rejects write to disallowed path |
| P1-UT-22 | argo-tools/src/http/tests.rs | HTTP tool makes GET request |
| P1-UT-23 | argo-tools/src/http/tests.rs | HTTP tool rejects request to disallowed domain |
| P1-UT-24 | argo-tools/src/registry/tests.rs | ToolRegistry register + lookup |
| P1-UT-25 | argo-tools/src/registry/tests.rs | ToolRegistry fallback lookup |
| P1-UT-26 | argo-observe/src/tests.rs | Root span created for agent run |
| P1-UT-27 | argo-observe/src/tests.rs | Child spans created for tool calls |

### 3.3 Phase 1 Integration Tests

| Test ID | Test Description |
|---------|------------------|
| P1-IT-01 | Single agent receives goal, calls LLM, parses tool calls, executes bash/files tools, returns result. Uses Docker Compose for Redis + SurrealDB. |
| P1-IT-02 | Agent stores task record in SurrealDB after completion, can retrieve it |
| P1-IT-03 | Agent context stored in Redis with correct TTL, expires after task ends |
| P1-IT-04 | Agent handles LLM API error gracefully (returns error, does not crash) |
| P1-IT-05 | Tool execution timeout triggers error, agent continues (no crash) |
| P1-IT-06 | Agent runs with Anthropic provider end-to-end (requires API key in CI secrets) |
| P1-IT-07 | Agent runs with OpenAI provider end-to-end (requires API key in CI secrets) |
| P1-IT-08 | Supervisor actor restarts crashed agent actor, task continues |

### 3.4 Phase 1 Stress Tests

| Test ID | Test Description |
|---------|------------------|
| P1-ST-01 | Spawn 50 agents concurrently, each executing a simple task. Verify all complete without OOM or deadlocks. Measures memory usage and latency. |
| P1-ST-02 | Run single agent for 100 consecutive tasks. Verify no memory leaks (Redis keys cleaned up, SurrealDB records created correctly). |
| P1-ST-03 | Tool registry with 1000 registered tools, verify lookup performance < 1ms. |
| P1-ST-04 | Redis connection pool: 100 concurrent reads/writes, verify no connection timeouts. |
| P1-ST-05 | SurrealDB: 100 concurrent task record writes, verify no data corruption. |

### 3.5 Phase 1 Outcome

- **argo-core**: Actor engine, message types, supervisor tree, LLM provider trait with Anthropic + OpenAI adapters, basic agent execution loop, config parser
- **argo-memory**: Redis short-term memory, SurrealDB long-term memory, MemoryHandle abstraction
- **argo-tools**: Tool trait, ToolRegistry with fallbacks, bash/files/http tools
- **argo-observe**: Basic OTel tracing and structured logging
- **Single agent can**: receive goal → call LLM → execute tools → store results in long-term memory
- **All tests pass**: 27 unit tests, 8 integration tests, 5 stress tests
- **CI green**: cargo fmt, clippy, test suite all passing

---

## 4. Phase 2 — Heal Loop & Full Memory

**Duration:** 6–8 weeks  
**Goal:** An agent that heals from errors, learns from them, and retrieves past experience before acting.

### 4.1 Tasks

| Task ID | Task | Crate | Details |
|---------|------|-------|---------|
| P2-T01 | Error taxonomy | argo-heal | Full AgentError enum with all variants from A-03 (LlmRateLimit, LlmContextOverflow, LlmHallucination, LlmRefusal, LlmTimeout, LlmProviderDown, ToolNotFound, ToolExecutionFailed, ToolTimeout, ToolPermissionDenied, ToolOutputInvalid, InfiniteLoop, GoalUnachievable, PlanInvalid, ContextCorrupted, MemoryUnavailable, McpConnectionFailed, NetworkTimeout, SubAgentFailed, OrchestratorFailed) |
| P2-T02 | Error classifier | argo-heal | Classifies raw errors into AgentError variants, attaches metadata (retry_after, evidence, provider name, etc.) |
| P2-T03 | HealEngine | argo-heal | HealEngine struct: strategies Vec, memory handle, telemetry handle. Runs strategies in order until one succeeds. Strategy selection informed by semantic memory (query Qdrant for similar past errors). |
| P2-T04 | Strategy 1: Retry with backoff | argo-heal | Exponential backoff (1s, 2s, 4s, 8s, ...), max retries configurable, jitter |
| P2-T05 | Strategy 2: Reframe prompt | argo-heal | Rephrase the instruction to the LLM, add clarification, change system prompt, retry |
| P2-T06 | Strategy 3: Swap tool | argo-heal | If tool A fails, try tool B from fallbacks map in ToolRegistry |
| P2-T07 | Strategy 4: Decompose | argo-heal | Break failing sub-task into smaller pieces, re-plan, execute pieces individually |
| P2-T08 | Strategy 5: Spawn sub-agent | argo-heal | Delegate failing part to a fresh agent with fresh context (uses ActorEngine to spawn) |
| P2-T09 | Strategy 6: Change LLM provider | argo-heal | If current provider fails, try next available provider from config (e.g., Claude → GPT-4 → local Ollama) |
| P2-T10 | Strategy 7: Reduce scope | argo-heal | Attempt simpler version of the task, partial result if full task fails |
| P2-T11 | Post-mortem loop | argo-heal | After error resolution: LLM reflects (what error, why, what resolved it, prevention), structured lesson written, lesson embedded and stored in Qdrant (argo_lessons), lesson stored in SurrealDB |
| P2-T12 | Qdrant semantic memory | argo-memory | Qdrant connection, collection creation per B-03, operations: store_experience, query_similar_experiences, store_error_resolution, query_similar_errors, store_lesson, query_lessons, store_tool_pattern, query_tool_patterns |
| P2-T13 | Embedding pipeline | argo-memory | Embedding trait (text → vector), OpenAI text-embedding-3 adapter, Ollama nomic-embed adapter, batch embedding support |
| P2-T14 | Experience retrieval pipeline | argo-memory | Before task execution: embed task description → query Qdrant top-5 similar tasks → query SurrealDB related entities/decisions → inject into system prompt |
| P2-T15 | Context window overflow handling | argo-memory | Detect near-full context → summarize oldest N turns → store full turns in SurrealDB (task:run_id:archived_turns) → replace old turns with summary in Redis → agent continues with compressed context |
| P2-T16 | Integrate heal into execution loop | argo-core | Wrap LLM calls and tool calls with heal engine. On error → classify → heal → retry. Track heal steps in AgentTrace. |
| P2-T17 | AgentTrace struct | argo-core | Trace: run_id, agent_name, goal, started_at, ended_at, duration_ms, success, output, iterations, quality_score, tool_calls, llm_calls, memory_ops, heal_steps, lessons, errors |
| P2-T18 | Integration test: heal from errors | tests/ | Agent encounters 5 different error types, heals from each, verify correct strategy used, verify lessons stored |
| P2-T19 | Integration test: experience retrieval | tests/ | Agent runs task similar to past task, verify semantic memory retrieval injected into prompt |

### 4.2 Phase 2 Unit Tests

| Test ID | Test File | Description |
|---------|-----------|-------------|
| P2-UT-01 | argo-heal/src/error/tests.rs | Error classifier correctly maps raw errors to AgentError variants |
| P2-UT-02 | argo-heal/src/strategy/tests.rs | Retry strategy: retries with correct backoff timing |
| P2-UT-03 | argo-heal/src/strategy/tests.rs | Reframe strategy: generates new prompt from original |
| P2-UT-04 | argo-heal/src/strategy/tests.rs | Swap tool strategy: selects fallback tool from registry |
| P2-UT-05 | argo-heal/src/strategy/tests.rs | Decompose strategy: breaks task into sub-tasks |
| P2-UT-06 | argo-heal/src/strategy/tests.rs | Change provider strategy: switches to next available provider |
| P2-UT-07 | argo-heal/src/strategy/tests.rs | Reduce scope strategy: generates simpler version of goal |
| P2-UT-08 | argo-heal/src/engine/tests.rs | HealEngine runs strategies in order, stops at first success |
| P2-UT-09 | argo-heal/src/engine/tests.rs | HealEngine queries semantic memory for similar past errors |
| P2-UT-10 | argo-heal/src/engine/tests.rs | HealEngine exhausts all strategies, returns structured failure |
| P2-UT-11 | argo-heal/src/postmortem/tests.rs | Post-mortem generates structured lesson with all fields |
| P2-UT-12 | argo-memory/src/qdrant/tests.rs | Qdrant store_experience / query_similar round-trip |
| P2-UT-13 | argo-memory/src/qdrant/tests.rs | Qdrant store_error_resolution / query_similar round-trip |
| P2-UT-14 | argo-memory/src/qdrant/tests.rs | Qdrant store_lesson / query_lessons round-trip |
| P2-UT-15 | argo-memory/src/qdrant/tests.rs | Qdrant store_tool_pattern / query_tool_patterns round-trip |
| P2-UT-16 | argo-memory/src/embedding/tests.rs | OpenAI embedding produces correct dimension vector |
| P2-UT-17 | argo-memory/src/embedding/tests.rs | Ollama embedding produces correct dimension vector |
| P2-UT-18 | argo-memory/src/retrieval/tests.rs | Experience retrieval pipeline returns top-K results |
| P2-UT-19 | argo-memory/src/overflow/tests.rs | Context overflow detection triggers at correct threshold |
| P2-UT-20 | argo-memory/src/overflow/tests.rs | Overflow handling: summarization replaces old turns, stores archive |
| P2-UT-21 | argo-core/src/trace/tests.rs | AgentTrace records all heal steps correctly |
| P2-UT-22 | argo-core/src/trace/tests.rs | AgentTrace records lessons learned |

### 4.3 Phase 2 Integration Tests

| Test ID | Test Description |
|---------|------------------|
| P2-IT-01 | Agent encounters ToolExecutionFailed (missing dependency), heal engine retries → swaps tool → succeeds. Verify lesson stored in Qdrant. |
| P2-IT-02 | Agent encounters LlmRateLimit, heal engine retries with backoff → succeeds. Verify retry timing correct. |
| P2-IT-03 | Agent encounters LlmContextOverflow, heal engine triggers overflow handling → summarizes → continues → succeeds. |
| P2-IT-04 | Agent encounters ToolNotFound, heal engine swaps to fallback tool → succeeds. |
| P2-IT-05 | Agent encounters LlmProviderDown, heal engine changes provider → succeeds with backup provider. |
| P2-IT-06 | Agent runs task similar to previously completed task, verify experience retrieval injects relevant past lessons into prompt. |
| P2-IT-07 | Agent heals from error, next run encounters similar error, verify resolution is attempted first (from semantic memory). |
| P2-IT-08 | Agent runs 10 tasks with intentional errors, verify all heal correctly and all lessons stored. |

### 4.4 Phase 2 Stress Tests

| Test ID | Test Description |
|---------|------------------|
| P2-ST-01 | 20 agents concurrently healing from errors, verify no deadlocks, all complete within timeout. |
| P2-ST-02 | Single agent encounters 50 consecutive errors, verify heal engine doesn't loop infinitely (max_attempts enforced). |
| P2-ST-03 | Qdrant: insert 10,000 lessons, query similarity, verify response time < 100ms. |
| P2-ST-04 | Context overflow handling: agent with 500 turns, verify summarization completes in < 30s. |
| P2-ST-05 | Concurrent embedding generation: 100 texts embedded in parallel, verify no rate limit errors (with batching). |

### 4.5 Phase 2 Outcome

- **argo-heal**: Full error taxonomy, 7 heal strategies, post-mortem loop, HealEngine with semantic-memory-informed strategy selection
- **argo-memory**: Qdrant semantic memory (4 collections), embedding pipeline, experience retrieval pipeline, context window overflow handling
- **Agent can**: heal from all error types, learn from errors, retrieve past experience before acting
- **All tests pass**: 22 unit tests, 8 integration tests, 5 stress tests
- **Integration verified**: 5 different error types healed, lessons stored and retrieved

---

## 5. Phase 3 — SDKs & CLI

**Duration:** 8 weeks  
**Goal:** Developers can use Argo from Python, TypeScript, and Rust with identical features.

### 5.1 Tasks

| Task ID | Task | Crate | Details |
|---------|------|-------|---------|
| P3-T01 | CLI scaffold | argo-cli | clap-based CLI with all commands: init, run, loop, inspect, memory (list/search/clear), stats, eval, validate, tools (list/info), mcp (connect/tools list), package |
| P3-T02 | argo init command | argo-cli | Creates: my-agent.toml (template per B-04), .gitignore, README.md |
| P3-T03 | argo run command | argo-cli | Runs agent from TOML config with goal argument, --inspect flag for live trace, --env flag for environment profiles |
| P3-T04 | argo loop command | argo-cli | Runs LoopAgent from TOML config (reads goal from config, no goal argument) |
| P3-T05 | argo inspect command | argo-cli | Inspect completed run by run-id, --live flag for running agents, displays AgentTrace |
| P3-T06 | argo memory command | argo-cli | list/search/clear subcommands for agent memory |
| P3-T07 | argo stats command | argo-cli | View agent performance metrics, --range flag, --compare flag |
| P3-T08 | argo eval command | argo-cli | Evaluate agent against scenario files |
| P3-T09 | argo validate command | argo-cli | Validate TOML config file against schema |
| P3-T10 | argo tools command | argo-cli | list/info subcommands for available tools |
| P3-T11 | argo mcp command | argo-cli | connect/tools list subcommands for MCP servers |
| P3-T12 | argo package command | argo-cli | Build and package agent for distribution |
| P3-T13 | Python SDK scaffold | argo-agents (PyO3) | Set up PyO3 bindings, maturin build, Python package structure |
| P3-T14 | Python SDK: Agent class | argo-agents | Agent(name, model, memory, heal, tools), run(goal), run_sync(goal), inspect(), memory property |
| P3-T15 | Python SDK: LoopAgent class | argo-agents | LoopAgent(name, goal, quality_threshold, max_iterations, tools, memory), run() |
| P3-T16 | Python SDK: AgentPool class | argo-agents | AgentPool(workers, agent_template), map(tasks), add(name, tools), run(goal) |
| P3-T17 | Python SDK: Memory access | argo-agents | memory.query(text, limit), memory.list(), memory.clear(type) |
| P3-T18 | Python SDK: Config loading | argo-agents | AgentConfig.from_file(path), Agent.from_config(config) |
| P3-T19 | Python SDK: Async support | argo-agents | asyncio.run(agent.run(goal)), async context managers |
| P3-T20 | TypeScript SDK scaffold | @argo-ai/sdk (napi-rs) | Set up napi-rs bindings, npm package structure, TypeScript type definitions |
| P3-T21 | TypeScript SDK: Agent class | @argo-ai/sdk | new Agent({name, model, tools, memory, heal}), run(goal), inspect() |
| P3-T22 | TypeScript SDK: LoopAgent class | @argo-ai/sdk | new LoopAgent({name, goal, quality_threshold, max_iterations, tools, memory}), run() |
| P3-T23 | TypeScript SDK: AgentPool class | @argo-ai/sdk | new AgentPool({workers, agent_template}), map(tasks), run(goal) |
| P3-T24 | TypeScript SDK: Memory access | @argo-ai/sdk | agent.memory.query(text, limit), agent.memory.list(), agent.memory.clear(type) |
| P3-T25 | TypeScript SDK: Config loading | @argo-ai/sdk | AgentConfig.fromFile(path), Agent.fromConfig(config) |
| P3-T26 | Feature parity test suite | tests/ | Same operations run across Rust, Python, TS SDKs, assert identical behavior and output schemas |
| P3-T27 | SDK documentation | docs/ | Python SDK reference (C-02), TypeScript SDK reference (C-03), CLI command reference (C-04) |
| P3-T28 | Error reference | docs/ | C-06: Every error returned to developer with meaning and suggested fix |

### 5.2 Phase 3 Unit Tests

| Test ID | Test File | Description |
|---------|-----------|-------------|
| P3-UT-01 | argo-cli/src/init/tests.rs | init command creates correct files |
| P3-UT-02 | argo-cli/src/run/tests.rs | run command parses args correctly |
| P3-UT-03 | argo-cli/src/inspect/tests.rs | inspect command formats AgentTrace correctly |
| P3-UT-04 | argo-cli/src/validate/tests.rs | validate command catches invalid config |
| P3-UT-05 | argo-agents/src/agent/tests.rs | Python Agent class instantiation |
| P3-UT-06 | argo-agents/src/agent/tests.rs | Python Agent.run() returns correct result type |
| P3-UT-07 | argo-agents/src/loop_agent/tests.rs | Python LoopAgent runs until quality threshold |
| P3-UT-08 | argo-agents/src/pool/tests.py | Python AgentPool distributes tasks |
| P3-UT-09 | argo-agents/src/memory/tests.py | Python memory.query() returns results |
| P3-UT-10 | argo-agents/src/config/tests.py | Python AgentConfig.from_file() parses TOML |
| P3-UT-11 | @argo-ai/sdk/src/agent/tests.ts | TypeScript Agent instantiation |
| P3-UT-12 | @argo-ai/sdk/src/agent/tests.ts | TypeScript Agent.run() returns typed result |
| P3-UT-13 | @argo-ai/sdk/src/loop_agent/tests.ts | TypeScript LoopAgent runs until threshold |
| P3-UT-14 | @argo-ai/sdk/src/pool/tests.ts | TypeScript AgentPool distributes tasks |
| P3-UT-15 | @argo-ai/sdk/src/memory/tests.ts | TypeScript memory.query() returns results |
| P3-UT-16 | @argo-ai/sdk/src/config/tests.ts | TypeScript AgentConfig.fromFile() parses TOML |

### 5.3 Phase 3 Integration Tests

| Test ID | Test Description |
|---------|------------------|
| P3-IT-01 | CLI: argo init → argo validate → argo run (end-to-end CLI workflow) |
| P3-IT-02 | CLI: argo run with --inspect shows live heal trace |
| P3-IT-03 | CLI: argo memory list/search/clear after agent run |
| P3-IT-04 | Python SDK: Agent runs task, heals from error, inspects trace |
| P3-IT-05 | TypeScript SDK: Agent runs task, heals from error, inspects trace |
| P3-IT-06 | Feature parity: identical task run in Rust, Python, TS → identical output schema |
| P3-IT-07 | Feature parity: identical heal trace in all three SDKs |
| P3-IT-08 | Feature parity: identical memory query results across SDKs |

### 5.4 Phase 3 Stress Tests

| Test ID | Test Description |
|---------|------------------|
| P3-ST-01 | CLI: 100 concurrent argo run commands, verify all complete without crash. |
| P3-ST-02 | Python SDK: spawn 50 agents from Python, verify GIL doesn't cause deadlock. |
| P3-ST-03 | TypeScript SDK: spawn 50 agents from Node.js, verify event loop doesn't block. |
| P3-ST-04 | Feature parity suite: run 20 scenarios across all SDKs, verify all pass. |

### 5.5 Phase 3 Outcome

- **argo-cli**: Full CLI with all commands (init, run, loop, inspect, memory, stats, eval, validate, tools, mcp, package)
- **argo-agents (Python)**: Agent, LoopAgent, AgentPool, Memory, Config — full API surface via PyO3
- **@argo-ai/sdk (TypeScript)**: Agent, LoopAgent, AgentPool, Memory, Config — full API surface via napi-rs
- **Feature parity**: All features identical across Rust, Python, TypeScript
- **All tests pass**: 16 unit tests, 8 integration tests, 4 stress tests
- **SDK docs published**: Python reference, TypeScript reference, CLI reference, Error reference

---

## 6. Phase 4 — Multi-Agent & MCP

**Duration:** 6 weeks  
**Goal:** Multi-agent pipelines work. Any MCP server's tools are available to agents.

### 6.1 Tasks

| Task ID | Task | Crate | Details |
|---------|------|-------|---------|
| P4-T01 | Orchestrator actor | argo-core | OrchestratorActor that receives top-level task, plans decomposition, assigns sub-tasks to workers |
| P4-T02 | Agent spawning | argo-core | SpawnAgent message type, agent creation from within running agent, child actor lifecycle management |
| P4-T03 | Task assignment protocol | argo-core | AssignTask message (task_id, goal, context, deadline), TaskComplete message (task_id, result, duration_ms, tools_used), TaskFailed message |
| P4-T04 | Result aggregation | argo-core | Orchestrator collects results from workers, merges outputs, handles partial failures |
| P4-T05 | AgentPool | argo-core | AgentPool implementation: worker count, task distribution (round-robin or least-busy), shared vs isolated memory modes |
| P4-T06 | Shared memory mode | argo-memory | memory="shared": workers share SurrealDB long-term and Qdrant semantic memory, isolated Redis short-term |
| P4-T07 | Isolated memory mode | argo-memory | memory="isolated": all three layers per-agent, no cross-agent access |
| P4-T08 | Persistent memory mode | argo-memory | memory="persistent": per-agent long-term and semantic, isolated short-term |
| P4-T09 | LoopAgent | argo-core | LoopAgent implementation: plan → execute → review → score → re-plan loop, QualityRubric integration |
| P4-T10 | Self-scoring | argo-core | Score output against QualityRubric criteria using LLM, weighted average calculation, threshold comparison |
| P4-T11 | MCP connector | argo-mcp | MCP client protocol implementation: SSE transport, tool discovery (tools/list), tool invocation (tools/call), auth handling |
| P4-T12 | MCP tool registration | argo-mcp | Discovered MCP tools registered in ToolRegistry under server namespace |
| P4-T13 | MCP auth: bearer token | argo-mcp | Bearer token auth for MCP servers |
| P4-T14 | MCP auth: OAuth2 | argo-mcp | OAuth2 flow for MCP servers (client_id, client_secret, token exchange) |
| P4-T15 | MCP reconnection | argo-mcp | Handle server disconnects, auto-reconnect with exponential backoff |
| P4-T16 | Web search tool | argo-tools | Web search via search API, result parsing |
| P4-T17 | Browser tool | argo-tools | Headless browser via Playwright, page navigation, element interaction, screenshot |
| P4-T18 | Git tool | argo-tools | Clone, commit, push, diff, branch operations |
| P4-T19 | Python tool | argo-tools | Python code execution in subprocess |
| P4-T20 | Code tool | argo-tools | Write, edit, run code files |
| P4-T21 | Integration test: multi-agent pipeline | tests/ | Orchestrator spawns researcher → writer → reviewer pipeline, verify end-to-end |
| P4-T22 | Integration test: MCP tool discovery | tests/ | Connect to mock MCP server, discover tools, invoke tool, verify result |

### 6.2 Phase 4 Unit Tests

| Test ID | Test File | Description |
|---------|-----------|-------------|
| P4-UT-01 | argo-core/src/orchestrator/tests.rs | Orchestrator decomposes task into sub-tasks |
| P4-UT-02 | argo-core/src/orchestrator/tests.rs | Orchestrator assigns tasks to workers |
| P4-UT-03 | argo-core/src/orchestrator/tests.rs | Orchestrator aggregates results from workers |
| P4-UT-04 | argo-core/src/orchestrator/tests.rs | Orchestrator handles worker failure (reassigns or continues) |
| P4-UT-05 | argo-core/src/spawn/tests.rs | Agent spawning creates child actor |
| P4-UT-06 | argo-core/src/spawn/tests.rs | Child actor lifecycle managed correctly |
| P4-UT-07 | argo-core/src/pool/tests.rs | AgentPool distributes tasks evenly |
| P4-UT-08 | argo-core/src/pool/tests.rs | AgentPool shared memory mode: workers see each other's data |
| P4-UT-09 | argo-core/src/pool/tests.rs | AgentPool isolated memory mode: workers isolated |
| P4-UT-10 | argo-core/src/loop/tests.rs | LoopAgent iterates until quality threshold met |
| P4-UT-11 | argo-core/src/loop/tests.rs | LoopAgent stops at max_iterations |
| P4-UT-12 | argo-core/src/scoring/tests.rs | Self-scoring produces correct weighted average |
| P4-UT-13 | argo-core/src/scoring/tests.rs | Scoring with all criteria at threshold returns pass |
| P4-UT-14 | argo-mcp/src/connector/tests.rs | MCP tool discovery parses tools/list response |
| P4-UT-15 | argo-mcp/src/connector/tests.rs | MCP tool invocation calls tools/call correctly |
| P4-UT-16 | argo-mcp/src/connector/tests.rs | MCP bearer auth header attached correctly |
| P4-UT-17 | argo-mcp/src/connector/tests.rs | MCP reconnection after disconnect |
| P4-UT-18 | argo-tools/src/web_search/tests.rs | Web search returns results |
| P4-UT-19 | argo-tools/src/browser/tests.rs | Browser navigates to page |
| P4-UT-20 | argo-tools/src/git/tests.rs | Git clone/commit/push operations |
| P4-UT-21 | argo-tools/src/python/tests.rs | Python tool executes code |
| P4-UT-22 | argo-tools/src/code/tests.rs | Code tool writes and runs file |

### 6.3 Phase 4 Integration Tests

| Test ID | Test Description |
|---------|------------------|
| P4-IT-01 | Multi-agent pipeline: orchestrator → researcher → writer → reviewer, verify final output |
| P4-IT-02 | AgentPool with 4 workers processes 8 tasks in parallel, verify all complete |
| P4-IT-03 | LoopAgent runs until quality threshold, verify score meets threshold |
| P4-IT-04 | MCP: connect to mock server, discover 5 tools, invoke each, verify results |
| P4-IT-05 | MCP: server disconnects mid-task, agent reconnects and continues |
| P4-IT-06 | Multi-agent with shared memory: researcher's findings available to writer |
| P4-IT-07 | Multi-agent: one worker fails, orchestrator reassigns task, pipeline completes |
| P4-IT-08 | LoopAgent: fails first iteration, heals, improves score on second iteration |

### 6.4 Phase 4 Stress Tests

| Test ID | Test Description |
|---------|------------------|
| P4-ST-01 | Orchestrator spawns 20 workers concurrently, all execute tasks, verify no deadlocks |
| P4-ST-02 | AgentPool with 10 workers processes 100 tasks, verify throughput > 10 tasks/minute |
| P4-ST-03 | MCP: 10 agents connect to same MCP server simultaneously, verify no conflicts |
| P4-ST-04 | LoopAgent: 20 iterations with 5 criteria each, verify scoring completes in < 60s |
| P4-ST-05 | Multi-agent pipeline: 50 sequential pipeline runs, verify no memory leaks |

### 6.5 Phase 4 Outcome

- **argo-core**: OrchestratorActor, agent spawning, AgentPool, LoopAgent with self-scoring
- **argo-mcp**: Full MCP protocol connector, tool discovery, auth (bearer + OAuth2), reconnection
- **argo-tools**: web_search, browser (Playwright), git, python, code tools
- **Multi-agent pipelines**: Orchestrator + specialists pattern working end-to-end
- **Loop agents**: Autonomous execution with quality threshold
- **All tests pass**: 22 unit tests, 8 integration tests, 5 stress tests

---

## 7. Phase 5 — Evolution & Production Polish

**Duration:** 6 weeks  
**Goal:** v1.0.0 production-ready release.

### 7.1 Tasks

| Task ID | Task | Crate | Details |
|---------|------|-------|---------|
| P5-T01 | Daily growth cycle | argo-heal | Background task: pull errors from last 24h (SurrealDB), detect patterns (same error 3+ times, same tool failing, same task type succeeding), generate improvement proposals, auto-apply low-risk, flag high-risk |
| P5-T02 | Pattern detection | argo-heal | Statistical analysis of error records: frequency, recency, context clustering |
| P5-T03 | Improvement proposal schema | argo-heal | Proposal: type (prompt_update, pre_check, strategy_reorder), target, content, risk_level, confidence |
| P5-T04 | Auto-apply low-risk | argo-heal | Prompt additions, pre-check steps applied automatically |
| P5-T05 | Flag high-risk | argo-heal | Strategy reordering, scope changes flagged for developer review |
| P5-T06 | Growth report | argo-heal | Structured report written to SurrealDB after each growth cycle |
| P5-T07 | argo stats command | argo-cli | Display evolution metrics: tasks completed, avg quality score, avg iterations, errors per task, most improved area, top lesson |
| P5-T08 | argo eval command | argo-cli | Load scenario files, run agent against each, score results, produce eval report |
| P5-T09 | Eval scenario format | docs/ | Define eval scenario schema (goal, expected outcome, scoring criteria, tools allowed) |
| P5-T10 | Getting Started Guide | docs/ | D-01: Install, create first agent, run it, see results — under 10 minutes |
| P5-T11 | Building a Coding Agent guide | docs/ | D-02: Full walkthrough from config to first run |
| P5-T12 | Building a Research Agent guide | docs/ | D-03: Web browsing agent with persistent memory walkthrough |
| P5-T13 | Building a Multi-Agent Pipeline guide | docs/ | D-04: Orchestrator + specialists pattern walkthrough |
| P5-T14 | Building a Loop Agent guide | docs/ | D-05: Autonomous loop with self-scoring walkthrough |
| P5-T15 | Memory Guide | docs/ | D-06: Three memory layers, when to use each, how to query |
| P5-T16 | Heal System Guide | docs/ | D-07: Heal loop, reading inspect() output, customizing strategies |
| P5-T17 | Tool Development Guide | docs/ | D-08: Write custom tool in Rust, Python, TypeScript |
| P5-T18 | MCP Integration Guide | docs/ | D-09: Connect MCP server, discover tools, handle auth |
| P5-T19 | Configuration Reference | docs/ | D-10: Every field in agent.toml explained |
| P5-T20 | Observability Guide | docs/ | D-11: OTel backend setup, reading traces, custom metrics |
| P5-T21 | Self-Improvement Guide | docs/ | D-12: Growth cycle, evolution stats, tuning rubric |
| P5-T22 | Deployment Guide | docs/ | D-13: Production setup, Docker Compose, Kubernetes |
| P5-T23 | Migration Guide | docs/ | D-14: Migrating from LangChain, CrewAI, AutoGen |
| P5-T24 | Example agents | examples/ | Coding agent, research agent, data analyst agent |
| P5-T25 | Community tool registry | docs/ | E-08: How to publish custom tools |
| P5-T26 | Security audit | security/ | Full security review: tool sandboxing, secret handling, permission enforcement |
| P5-T27 | Performance benchmark | tests/ | Benchmark suite: agent startup time, tool execution latency, memory query latency, heal strategy execution time |
| P5-T28 | Docker Compose production config | deploy/ | Production Docker Compose with Redis, SurrealDB, Qdrant, agent |
| P5-T29 | Kubernetes manifests | deploy/ | K8s deployment, service, configmap, secret manifests |
| P5-T30 | GitHub Release setup | release/ | Release automation: version bumping, changelog generation, binary builds for all platforms, crates.io/PyPI/npm publish |

### 7.2 Phase 5 Unit Tests

| Test ID | Test File | Description |
|---------|-----------|-------------|
| P5-UT-01 | argo-heal/src/growth/tests.rs | Growth cycle pulls errors from last 24h |
| P5-UT-02 | argo-heal/src/growth/tests.rs | Pattern detection identifies recurring errors |
| P5-UT-03 | argo-heal/src/growth/tests.rs | Improvement proposal generation produces valid proposals |
| P5-UT-04 | argo-heal/src/growth/tests.rs | Low-risk proposals auto-applied |
| P5-UT-05 | argo-heal/src/growth/tests.rs | High-risk proposals flagged for review |
| P5-UT-06 | argo-cli/src/stats/tests.rs | Stats command formats output correctly |
| P5-UT-07 | argo-cli/src/eval/tests.rs | Eval command loads scenarios correctly |
| P5-UT-08 | argo-cli/src/eval/tests.rs | Eval command scores agent output |

### 7.3 Phase 5 Integration Tests

| Test ID | Test Description |
|---------|------------------|
| P5-IT-01 | Growth cycle runs after 24h of agent operation, detects patterns, generates proposals |
| P5-IT-02 | Eval: run agent against 5 scenarios, produce eval report with scores |
| P5-IT-03 | Deployment: docker-compose up → agent runs → heals → learns → evolution tracked |
| P5-IT-04 | Getting Started Guide: follow guide verbatim, agent runs successfully |
| P5-IT-05 | Coding Agent Guide: follow guide, agent produces working code with tests |
| P5-IT-06 | Multi-Agent Guide: follow guide, pipeline completes end-to-end |

### 7.4 Phase 5 Stress Tests

| Test ID | Test Description |
|---------|------------------|
| P5-ST-01 | Agent runs 1000 tasks over 24h, growth cycle processes all, no memory leaks |
| P5-ST-02 | Eval suite: 50 scenarios run sequentially, complete in < 1 hour |
| P5-ST-03 | Production deployment: 10 agents running for 1 hour, verify stability, no crashes |
| P5-ST-04 | Performance benchmarks: all metrics within acceptable thresholds (agent startup < 1s, tool execution < 5s, memory query < 100ms) |
| P5-ST-05 | Binary builds: CLI binary runs on Linux x86_64, macOS aarch64, Windows x86_64 |

### 7.5 Phase 5 Outcome

- **Daily growth cycle**: Agents improve automatically over time
- **Evolution tracking**: argo stats shows measurable improvement
- **Eval system**: argo eval for scenario-based agent evaluation
- **Full documentation site**: 14 developer guides (D-01 through D-14)
- **Example agents**: Coding, research, data analyst templates
- **Production deployment**: Docker Compose and Kubernetes configs
- **v1.0.0 release**: All platforms, all registries (crates.io, PyPI, npm), GitHub Releases
- **Security audit passed**
- **All tests pass**: 8 unit tests, 6 integration tests, 5 stress tests

---

## 8. Testing Strategy

### 8.1 Test Pyramid

```
           ┌─────────┐
           │ Stress  │  ← Phase-specific stress tests
          ┌┴─────────┴┐
          │Integration │  ← End-to-end with real services
         ┌┴───────────┴┐
         │    Unit     │  ← Fast, isolated, mocked external deps
         └─────────────┘
```

### 8.2 Test Execution Rules

| Rule | Description |
|------|-------------|
| Unit tests | Run on every commit, must pass before merge |
| Integration tests | Run on every PR to main, require Docker Compose services |
| Stress tests | Run nightly on main, results tracked over time |
| Feature parity | Run on every SDK change, assert identical output schemas |
| Security scan | Run on every release, cargo-audit + secret scanning |

### 8.3 Test Infrastructure

- **Docker Compose**: Redis, SurrealDB, Qdrant for integration/stress tests
- **Mock LLM server**: Returns predetermined responses for deterministic testing
- **Mock MCP server**: Returns predetermined tool lists for MCP testing
- **CI matrix**: Linux (ubuntu-latest), macOS (macos-latest), Windows (windows-latest)
- **Code coverage**: cargo-tarpaulin, target > 80% for argo-core, > 70% for all crates

### 8.4 Test Count Summary

| Phase | Unit | Integration | Stress | Total |
|-------|------|-------------|--------|-------|
| Phase 1 | 27 | 8 | 5 | 40 |
| Phase 2 | 22 | 8 | 5 | 35 |
| Phase 3 | 16 | 8 | 4 | 28 |
| Phase 4 | 22 | 8 | 5 | 35 |
| Phase 5 | 8 | 6 | 5 | 19 |
| **Total** | **95** | **38** | **24** | **157** |

---

## 9. Additional Documentation Required

Beyond what the master plan specifies, the following additional documents are needed for a production-ready framework:

| # | Document | Purpose | Phase |
|---|----------|---------|-------|
| ADD-01 | API Rate Limiting Guide | How Argo handles rate limits per provider, configurable backoff, provider-specific limits | Phase 2 |
| ADD-02 | Custom Tool Development Tutorial | Step-by-step tutorial for building a custom tool in Rust with PyO3 and napi-rs bindings | Phase 3 |
| ADD-03 | Agent Design Patterns | Common patterns: plan-and-execute, reflection, tool-use, multi-step reasoning | Phase 4 |
| ADD-04 | Benchmarking Guide | How to run and interpret Argo performance benchmarks | Phase 5 |
| ADD-05 | Troubleshooting Guide | Common issues and solutions: connection failures, memory issues, heal loops | Phase 5 |
| ADD-06 | Changelog Template | Structured changelog format following Keep a Changelog convention | Phase 0 |
| ADD-07 | Roadmap Visualization | Public roadmap page showing phases, milestones, and progress | Phase 0 |
| ADD-08 | RFC Review Process | How RFCs are proposed, reviewed, approved, and implemented | Phase 0 |
| ADD-09 | Crate Dependency Rules | Which crates can depend on which, no circular dependencies, dependency audit | Phase 1 |
| ADD-10 | Performance Budget | Maximum acceptable latency/throughput for each operation | Phase 1 |

---

## 10. Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Actix actor model too complex for agent orchestration | High | Medium | Prototype actor engine in Phase 1 Week 2, validate with 3-agent scenario before committing |
| SurrealDB 2.x API instability | High | Medium | Pin to specific version, abstract behind trait, prepare SQLite fallback |
| PyO3/napi-rs binding complexity | Medium | High | Start with minimal API surface in Phase 3, expand incrementally |
| MCP protocol specification changes | Medium | Medium | Pin to specific MCP version, isolate behind connector trait |
| LLM API changes breaking adapters | High | Medium | LLM Provider trait isolates changes, adapter updates are localized |
| Memory growth over long-running agents | High | Low | Implement TTL enforcement early, monitor in stress tests |
| Community adoption slower than expected | Medium | High | Focus on documentation quality, example agents, migration guides from popular frameworks |
| Cross-platform binary compatibility | Medium | Medium | Test on all 3 platforms in CI from Phase 1, use cross-compilation |

---

*End of Argo Implementation Plan v0.1.0*
