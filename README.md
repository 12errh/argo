# Argo

**The first agent framework where agents genuinely get better over time.**

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/argo-agents/argo/actions/workflows/ci.yml/badge.svg)](https://github.com/argo-agents/argo/actions/workflows/ci.yml)

Argo is a multi-language agent framework with a Rust core. It enables developers to build:

- Single autonomous agents that run tasks without human supervision
- Multi-agent systems where specialized agents collaborate
- Loop agents that plan, execute, review, and iterate until done
- Long-running agents with persistent memory that improve with every run

## Why Argo?

| Capability | LangChain | CrewAI | AutoGen | **Argo** |
|---|---|---|---|---|
| Core language | Python | Python | Python | **Rust** |
| Self-healing | Manual | None | Partial | **Automatic** |
| Self-improvement | None | None | None | **Built-in** |
| Memory layers | 1 | 1 | 1 | **3** |
| MCP native | Partial | No | No | **Full** |
| Multi-language SDKs | Python | Python | Python | **Python + TS + Rust** |

## Quick Start

```bash
# Rust
cargo add argo

# Python
pip install argo-agents

# TypeScript
npm install @argo-ai/sdk
```

## Documentation

- [Master Plan](docs/argo-master-plan.md) — Full vision and specification
- [Implementation Plan](docs/argo-implementation-plan.md) — Detailed development phases
- [Architecture RFCs](docs/rfcs/architecture/) — Technical design documents
- [Data Schemas](docs/rfcs/schemas/) — Storage and protocol schemas
- [Contributing Guide](CONTRIBUTING.md) — How to contribute

## License

Apache 2.0 — see [LICENSE](LICENSE) for details.
