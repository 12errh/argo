# Argo Roadmap

## Current Phase: Phase 0 — Planning

**Duration:** 4–6 weeks
**Status:** In Progress

### Milestones

- [x] Master plan and specification
- [x] Implementation plan
- [x] 12 Architecture RFCs
- [x] 6 Data Schema documents
- [ ] GitHub repository setup
- [x] CI/CD pipeline
- [x] Contributing guidelines
- [x] Decision log

## Upcoming Phases

### Phase 1 — Rust Core Engine (8–10 weeks)
- Actor engine with Actix
- LLM provider trait with Anthropic and OpenAI adapters
- Basic agent execution loop
- Redis and SurrealDB memory layers
- Built-in tools (bash, files, http)
- Basic observability

### Phase 2 — Heal Loop & Full Memory (6–8 weeks)
- Error taxonomy and classification
- 7 heal strategies
- Post-mortem loop
- Qdrant semantic memory
- Embedding pipeline
- Experience retrieval

### Phase 3 — SDKs & CLI (8 weeks)
- CLI with all commands
- Python SDK via PyO3
- TypeScript SDK via napi-rs
- Feature parity test suite

### Phase 4 — Multi-Agent & MCP (6 weeks)
- Orchestrator actor
- AgentPool
- LoopAgent with self-scoring
- Full MCP protocol connector
- Additional tools (web, browser, git, python)

### Phase 5 — Evolution & Production Polish (6 weeks)
- Daily growth cycle
- Evolution tracking
- Eval system
- Full documentation
- v1.0.0 release

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to get involved.

## Discussion

Join the conversation in [GitHub Discussions](https://github.com/argo-agents/argo/discussions) or [Discord](https://discord.gg/argo).
