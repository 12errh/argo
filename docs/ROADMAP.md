# Argo Roadmap

## Completed Phases

### Phase 0 — Planning (Completed)
**Duration:** 4–6 weeks

- [x] Master plan and specification
- [x] Implementation plan
- [x] 12 Architecture RFCs
- [x] 6 Data Schema documents
- [x] CI/CD pipeline
- [x] Contributing guidelines
- [x] Decision log

### Phase 1 — Rust Core Engine (Completed)
**Duration:** 8–10 weeks

- [x] Actor engine with Actix
- [x] LLM provider trait with Anthropic and OpenAI adapters
- [x] Basic agent execution loop
- [x] Redis and SurrealDB memory layers
- [x] Built-in tools (bash, files, http)
- [x] Basic observability
- [x] TOML config parser
- [x] Tool registry with fallbacks

### Phase 2 — Heal Loop & Full Memory (Completed)
**Duration:** 6–8 weeks

- [x] Error taxonomy and classification (20 variants, severity mapping)
- [x] 7 heal strategies (retry, reframe, swap_tool, decompose, spawn_subagent, change_provider, reduce_scope)
- [x] HealEngine with strategy chain and ordering
- [x] Post-mortem loop for lesson generation
- [x] Qdrant semantic memory (4 collections: experiences, errors, lessons, tool_patterns)
- [x] Embedding pipeline (OpenAI, Ollama, Mock adapters)
- [x] Experience retrieval pipeline
- [x] Context overflow handler
- [x] Typed AgentTrace (HealStepRecord, LessonRecord, MemoryOpRecord)
- [x] 49 unit tests passing

## Upcoming Phases

### Phase 3 — SDKs & CLI (8 weeks)
- CLI with all commands (init, run, loop, inspect, memory, stats, eval, validate, tools, mcp, package)
- Python SDK via PyO3
- TypeScript SDK via napi-rs
- Feature parity test suite

### Phase 4 — Multi-Agent & MCP (6 weeks)
- Orchestrator actor
- AgentPool with task distribution
- LoopAgent with self-scoring
- Full MCP protocol connector
- Additional tools (web, browser, git, python, code)

### Phase 5 — Evolution & Production Polish (6 weeks)
- Daily growth cycle
- Evolution tracking (argo stats)
- Eval system (argo eval)
- Full documentation (14 guides)
- Example agents
- Docker Compose and Kubernetes deployment
- v1.0.0 release

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to get involved.

## Discussion

Join the conversation in [GitHub Discussions](https://github.com/argo-agents/argo/discussions) or [Discord](https://discord.gg/argo).
