# F-05: Decision Log

**Status:** Active
**Owner:** Argo Core Team

---

## Purpose

Record of all architectural decisions and their rationale. Each entry captures what was decided, why, and what alternatives were considered.

## Decisions

### D-001: Rust as Core Language

**Date:** 2026-06-27
**Status:** Accepted
**RFC:** Master Plan §1

**Decision:** Argo's core runtime will be written in Rust.

**Rationale:**
- Zero garbage collection eliminates memory leaks
- Fearless concurrency via ownership model
- Blazing performance for concurrent agent execution
- Memory safety without runtime overhead
- Strong ecosystem for async (Tokio) and actors (Actix)

**Alternatives Considered:**
- Go: Good concurrency, but GC pauses, less control over memory
- C++: Performance, but memory safety risks, more complex
- Erlang/Elixir: Great actor model, but limited ecosystem

**Consequences:**
- SDKs for Python and TypeScript must use FFI bindings (PyO3, napi-rs)
- Steeper learning curve for contributors
- Better long-term performance and reliability

### D-002: Three-Layer Memory Architecture

**Date:** 2026-06-27
**Status:** Accepted
**RFC:** A-02

**Decision:** Argo uses three separate storage layers: Redis (short-term), SurrealDB (long-term), Qdrant (semantic).

**Rationale:**
- Each layer optimized for its access pattern
- Redis: sub-millisecond for working context
- SurrealDB: relational + graph for entity relationships
- Qdrant: vector search for experience retrieval

**Alternatives Considered:**
- Single PostgreSQL: Simpler, but no vector search, worse performance for short-term
- SQLite: Embedded, no server needed, but no graph or vector capabilities
- Redis for all: Fast, but no persistence, no vector search

**Consequences:**
- Three services to run in production
- More complex deployment
- Better performance and capabilities per use case

### D-003: Actix for Actor Model

**Date:** 2026-06-27
**Status:** Accepted
**RFC:** A-01

**Decision:** Use Actix as the actor framework for Argo's core runtime.

**Rationale:**
- Mature and battle-tested in production
- Excellent documentation and community
- Built-in supervision tree support
- Efficient message passing

**Alternatives Considered:**
- Ractor: Newer, less mature
- Custom actor implementation: Full control, but reinvents solved problems
- Tokio tasks: Simpler, but no built-in supervision

**Consequences:**
- Dependency on Actix's release cycle
- Need to learn Actix's patterns
- Good foundation for fault tolerance

### D-004: MessagePack for Inter-Actor Communication

**Date:** 2026-06-27
**Status:** Accepted
**RFC:** A-01

**Decision:** Use MessagePack (rmp-serde) for serializing messages between actors.

**Rationale:**
- Binary format: smaller than JSON, faster to parse
- Schema-less: no compile step required
- Good Rust support via rmp-serde
- Efficient for high-throughput message passing

**Alternatives Considered:**
- JSON: Human-readable, but larger and slower
- Protocol Buffers: Efficient, but requires schema compilation
- Bincode: Fast, but not self-describing

**Consequences:**
- Messages not human-readable in logs (mitigated by structured logging)
- Need to handle versioning manually
- Good performance for actor communication

### D-005: Apache 2.0 License

**Date:** 2026-06-27
**Status:** Accepted
**RFC:** Master Plan §1

**Decision:** Argo is licensed under Apache 2.0.

**Rationale:**
- Permissive: allows commercial use
- Patent grant: protects users
- Strong community adoption
- Compatible with most open-source licenses

**Alternatives Considered:**
- MIT: Simpler, but no patent grant
- GPL: Copyleft, but may discourage commercial adoption
- MPL: Weak copyleft, but less familiar

**Consequences:**
- Must include LICENSE file in all distributions
- Must include copyright notice
- Contributors retain copyright, grant license
