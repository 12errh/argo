# Phase 1 - Rust Core Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task.

**Goal:** Build a working Rust agent that can receive a goal, call an LLM, execute tools, and store results in memory.

**Architecture:** `argo-core` contains actor engine, LLM trait, execution loop. `argo-memory` wraps Redis + SurrealDB. `argo-tools` provides tool trait and built-in tools. `argo-observe` handles tracing. All crates in a single Cargo workspace.

**Tech Stack:** Rust stable, Tokio, Actix, serde/rmp-serde, reqwest, redis, surrealdb, async-trait, thiserror/anyhow, clap, tracing, uuid, chrono

## Global Constraints

- Rust stable 1.75+, edition 2021
- Use `thiserror` for library errors, `anyhow` for app-level propagation
- Use `async-trait` for async trait definitions
- Use `uuid::Uuid` for identifiers, `chrono` for timestamps
- Mock external services in unit tests; Docker Compose for integration tests
- `cargo fmt --all` and `cargo clippy` must pass before each commit
- Follow schemas in `docs/rfcs/schemas/` and RFCs in `docs/rfcs/architecture/`

---

## Task 1: Add Dependencies to Workspace

**Files:** Modify `Cargo.toml`, `crates/argo-{core,memory,tools,observe,cli}/Cargo.toml`

- [ ] Update workspace Cargo.toml with all deps: tokio (full), serde (derive), serde_json, anyhow, thiserror, tracing, tracing-subscriber (json, env-filter), uuid (v4, serde), chrono (serde), async-trait, rmp-serde, reqwest (json, stream), futures, toml, semver, redis (tokio-comp, connection-manager), surrealdb, opentelemetry, opentelemetry_sdk, opentelemetry-otlp
- [ ] Update each crate Cargo.toml with its specific deps
- [ ] Run `cargo check --workspace`
- [ ] Commit: `chore: add all Phase 1 dependencies to workspace`

---

## Task 2: Core Error Types

**Files:** Create `crates/argo-core/src/error.rs`, modify `crates/argo-core/src/lib.rs`

**Produces:** `AgentError`, `LlmError`, `ToolError`, `From<LlmError> for AgentError`

- [ ] Create error.rs with all AgentError variants from A-03 RFC
- [ ] Create LlmError enum (RateLimited, ContextOverflow, AuthenticationFailed, ModelNotAvailable, Timeout, ProviderError, NetworkError, InvalidResponse, StreamingError)
- [ ] Create ToolError enum (PermissionDenied, ExecutionFailed, Timeout, InvalidInput, OutputTooLarge)
- [ ] Implement From<LlmError> for AgentError
- [ ] Update lib.rs: `pub mod error;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add AgentError, LlmError, ToolError types`

---

## Task 3: LLM Provider Trait

**Files:** Create `crates/argo-core/src/llm.rs`, modify `crates/argo-core/src/lib.rs`

**Produces:** `LlmProvider` trait, `CompletionRequest`, `CompletionResponse`, `Message`, `Role`, `MessageContent`, `ToolDefinition`, `ToolCallRequest`, `TokenUsage`, `StopReason`

- [ ] Create llm.rs with all types from A-05 RFC
- [ ] Define `#[async_trait] pub trait LlmProvider` with: complete(), stream(), provider_name(), model_name(), context_limit(), max_output_tokens()
- [ ] Update lib.rs: `pub mod llm;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add LlmProvider trait and message types`

---

## Task 4: Anthropic Adapter

**Files:** Create `crates/argo-core/src/llm/anthropic.rs`

**Produces:** `AnthropicProvider` implementing `LlmProvider`

- [ ] Create AnthropicProvider struct (api_key, model, client)
- [ ] Implement complete(): POST to api.anthropic.com/v1/messages with x-api-key, anthropic-version 2023-06-01
- [ ] Parse Anthropic response format into CompletionResponse
- [ ] Handle rate limits (429), auth errors (401), timeouts
- [ ] Convert llm.rs to module directory: llm/mod.rs, llm/anthropic.rs
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add Anthropic Claude adapter`

---

## Task 5: OpenAI Adapter

**Files:** Create `crates/argo-core/src/llm/openai.rs`

**Produces:** `OpenAiProvider` implementing `LlmProvider`

- [ ] Create OpenAiProvider struct (api_key, model, client, base_url)
- [ ] Implement complete(): POST to api.openai.com/v1/chat/completions with Bearer auth
- [ ] Parse OpenAI response format into CompletionResponse
- [ ] Handle rate limits, auth errors, timeouts
- [ ] Register in llm/mod.rs
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add OpenAI adapter`

---

## Task 6: Agent Message Types

**Files:** Create `crates/argo-core/src/message.rs`

**Produces:** `ExecuteTask`, `ToolCall`, `ToolResult`, `TaskResult`, `AgentTrace`, and all inter-agent message types from A-01 RFC

- [ ] Create message.rs with all types from A-01 RFC
- [ ] All types derive Debug, Clone, Serialize, Deserialize
- [ ] TaskResult enum: Success, Partial, Failed
- [ ] AgentTrace struct with all fields
- [ ] Update lib.rs: `pub mod message;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add agent message types`

---

## Task 7: TOML Config Parser

**Files:** Create `crates/argo-core/src/config.rs`

**Produces:** `AgentConfig` and all nested config structs matching B-04 schema

- [ ] Create config structs matching B-04 schema
- [ ] Implement AgentConfig::from_file(path) with TOML parsing
- [ ] Implement env var substitution: ${VAR} to env value
- [ ] Validate required fields, types, ranges
- [ ] Update lib.rs: `pub mod config;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add TOML config parser`

---

## Task 8: Tool Trait and Context

**Files:** Create `crates/argo-tools/src/trait_def.rs`, `crates/argo-tools/src/error.rs`, modify `crates/argo-tools/src/lib.rs`

**Produces:** `Tool` trait, `ToolContext`, `ToolPermissions`

- [ ] Create error.rs with ToolError variants
- [ ] Create trait_def.rs with Tool trait: name(), description(), input_schema(), output_schema(), permissions(), execute()
- [ ] ToolContext: agent_id, run_id, working_dir, environment
- [ ] ToolPermissions: allow_filesystem, allow_network, allow_subprocess, working_directory, allowed_paths, allowed_domains, max_execution_time
- [ ] Update lib.rs
- [ ] Run `cargo check -p argo-tools`
- [ ] Commit: `feat(tools): add Tool trait, ToolContext, ToolPermissions`

---

## Task 9: Bash Tool

**Files:** Create `crates/argo-tools/src/bash.rs`

- [ ] Implement BashTool with working_directory and max_execution_time
- [ ] execute(): validate working dir, spawn sh, set timeout, capture stdout/stderr, return JSON
- [ ] Register in lib.rs
- [ ] Run `cargo check -p argo-tools`
- [ ] Commit: `feat(tools): add Bash tool`

---

## Task 10: Files Tool

**Files:** Create `crates/argo-tools/src/files.rs`

- [ ] Implement FilesTool with allowed_paths
- [ ] execute(): match on action (read/write/list/delete), validate path, perform operation
- [ ] Register in lib.rs
- [ ] Run `cargo check -p argo-tools`
- [ ] Commit: `feat(tools): add Files tool`

---

## Task 11: HTTP Tool

**Files:** Create `crates/argo-tools/src/http.rs`

- [ ] Implement HttpTool with allowed_domains
- [ ] execute(): validate domain, make request, return status/headers/body
- [ ] Register in lib.rs
- [ ] Run `cargo check -p argo-tools`
- [ ] Commit: `feat(tools): add HTTP tool`

---

## Task 12: Tool Registry

**Files:** Create `crates/argo-tools/src/registry.rs`

**Produces:** `ToolRegistry`, `ToolInfo`

- [ ] Create ToolRegistry with HashMap of tools, versions, fallbacks
- [ ] Implement register(), unregister(), get(), list()
- [ ] Implement register_fallbacks(), get_fallbacks()
- [ ] Implement hot_reload()
- [ ] Register in lib.rs
- [ ] Run `cargo check -p argo-tools`
- [ ] Commit: `feat(tools): add ToolRegistry with fallbacks`

---

## Task 13: Structured Logging

**Files:** Create `crates/argo-observe/src/tracing.rs`, modify `crates/argo-observe/src/lib.rs`

**Produces:** `init_tracing()` function

- [ ] Create tracing.rs with init_tracing(enabled, backend, endpoint)
- [ ] Setup tracing-subscriber with JSON format, env-filter
- [ ] If OTLP enabled, configure OpenTelemetry TracerProvider
- [ ] Update lib.rs
- [ ] Run `cargo check -p argo-observe`
- [ ] Commit: `feat(observe): add structured logging with optional OTel`

---

## Task 14: Memory System

**Files:** Create `crates/argo-memory/src/{error.rs,handle.rs,redis.rs,surreal.rs}`, modify `crates/argo-memory/src/lib.rs`

**Produces:** `MemoryHandle`, `RedisMemory`, `SurrealMemory`

- [ ] Create error.rs with MemoryError variants
- [ ] Create redis.rs with RedisMemory: store_context, get_context, store_turns, get_turns, store_scratch, get_scratch, store_plan, get_plan, cleanup
- [ ] Redis key patterns per B-02 schema
- [ ] Create surreal.rs with SurrealMemory: store_task_record, get_task_record, store_entity, get_entity, create_relationship, query_relationships
- [ ] SurrealDB schema initialization per B-01
- [ ] Create handle.rs combining both stores
- [ ] Update lib.rs
- [ ] Run `cargo check -p argo-memory`
- [ ] Commit: `feat(memory): add Redis short-term and SurrealDB long-term memory`

---

## Task 15: Agent Execution Loop

**Files:** Create `crates/argo-core/src/execution.rs`

**Consumes:** LlmProvider (Task 3), ToolRegistry (Task 12), MemoryHandle (Task 14), AgentError (Task 2)

- [ ] Create execute_task(goal, llm, tools, memory, config) async function
- [ ] Build system prompt from config
- [ ] Loop: call LLM, parse response, if tool_calls execute via registry, append results, continue; if no tool_calls return final answer
- [ ] Max 20 iterations safety limit
- [ ] Store task record in SurrealDB on completion
- [ ] Return TaskResult
- [ ] Update lib.rs: `pub mod execution;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add agent execution loop`

---

## Task 16: AgentActor with Actix

**Files:** Create `crates/argo-core/src/actor.rs`

**Consumes:** execute_task (Task 15), message types (Task 6)

- [ ] Create AgentActor struct with config, memory, tools, llm
- [ ] Implement Actor for AgentActor
- [ ] Implement Handler<ExecuteTask> calling execute_task
- [ ] Create SupervisorActor with RestartStrategy enum
- [ ] Implement supervisor failure detection and restart
- [ ] Update lib.rs: `pub mod actor;`
- [ ] Run `cargo check -p argo-core`
- [ ] Commit: `feat(core): add AgentActor and SupervisorActor`

---

## Task 17: CLI Binary

**Files:** Modify `crates/argo-cli/src/main.rs`, `crates/argo-cli/Cargo.toml`

- [ ] Update Cargo.toml with deps on argo-core, argo-memory, argo-tools, argo-observe
- [ ] Implement argo init: create agent.toml template, .gitignore, README
- [ ] Implement argo run: load config, create providers, tools, memory, run agent, print result
- [ ] Implement argo validate: parse and validate TOML
- [ ] Add --inspect flag for trace output
- [ ] Run `cargo build -p argo-cli`
- [ ] Commit: `feat(cli): implement init, run, validate commands`

---

## Task 18: Integration Test

**Files:** Create `tests/integration_test.rs`, `docker-compose.yml`

- [ ] Create docker-compose.yml with Redis 7 and SurrealDB 2
- [ ] Write integration test with mock LLM provider
- [ ] Test: agent goal, tool execution, memory storage
- [ ] Test: LLM error handling
- [ ] Test: tool timeout
- [ ] Run `cargo test --test integration_test`
- [ ] Commit: `test: add integration tests for single agent`

---

## Task 19: Unit Tests

**Files:** Add tests modules to all crates

- [ ] argo-core: Actor processes ExecuteTask, Supervisor restarts, LLM trait contract, config parsing, env var substitution
- [ ] argo-memory: Redis store/get round-trip, TTL, SurrealDB store/get round-trip, relationships
- [ ] argo-tools: Bash executes, Files read/write, HTTP request, Registry register/lookup/fallbacks
- [ ] argo-observe: Root span created, child spans created
- [ ] Run `cargo test --workspace`
- [ ] Commit: `test: add unit tests for all crates`

---

## Task 20: Final Verification

- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets --all-features`
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo build --workspace`
- [ ] Push to GitHub, verify CI passes
- [ ] Commit if any fixes needed
