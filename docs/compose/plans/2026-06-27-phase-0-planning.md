# Argo Phase 0 — Planning & RFCs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete all Phase 0 deliverables — 12 Architecture RFCs, 6 Data Schema docs, GitHub repo setup, CI/CD skeleton, contributor docs, and decision log.

**Architecture:** Documentation-first approach. All technical decisions are locked in writing before any code. Each RFC is a standalone markdown document in `docs/rfcs/architecture/`. Schema docs go in `docs/rfcs/schemas/`. GitHub setup uses the `gh` CLI and GitHub API via MCP tools.

**Tech Stack:** Markdown (docs), GitHub Actions (CI), TOML/YAML (config), Rust workspace (Cargo.toml scaffold)

## Global Constraints

- All RFCs must reference the master plan (`docs/argo-master-plan.md`) as the source of truth
- Every RFC must follow the template: Title, Status, Summary, Motivation, Detailed Design, Alternatives, Drawbacks, Unresolved Questions
- Schema docs must be syntactically valid in their target format (SurrealDB SQL, TOML, Qdrant config, MessagePack)
- GitHub repo: `argo-agents/argo` (organization: `argo-agents`)
- CI must pass on empty project (cargo fmt, clippy, test)
- Apache 2.0 license
- All docs use consistent terminology from the master plan

---

## Task Group 1: Project Scaffolding & GitHub Setup

### Task 1: Create directory structure and workspace files

**Covers:** P0-T19, P0-T20

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `.gitignore`
- Create: `README.md`
- Create: `LICENSE`
- Create: `docs/rfcs/architecture/` (directory)
- Create: `docs/rfcs/schemas/` (directory)
- Create: `docs/rfcs/guides/` (directory)
- Create: `.github/workflows/ci.yml`
- Create: `.github/ISSUE_TEMPLATE/bug_report.md`
- Create: `.github/ISSUE_TEMPLATE/feature_request.md`
- Create: `.github/PULL_REQUEST_TEMPLATE.md`
- Create: `.github/CODEOWNERS`

- [ ] **Step 1: Create Cargo workspace**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/argo-core",
    "crates/argo-memory",
    "crates/argo-heal",
    "crates/argo-tools",
    "crates/argo-mcp",
    "crates/argo-observe",
    "crates/argo-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/argo-agents/argo"
description = "The first agent framework where agents genuinely get better over time"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Create .gitignore**

```gitignore
/target
*.swp
*.swo
*~
.DS_Store
.env
.env.*
!.env.example
*.log
.idea/
.vscode/
```

- [ ] **Step 3: Create LICENSE (Apache 2.0)**

```text
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to the Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by the Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding any notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

APPENDIX: How to apply the Apache License to your work.

   To apply the Apache License to your work, attach the following
   boilerplate notice, with the fields enclosed by brackets "[]"
   replaced with your own identifying information. (Don't include
   the brackets!)  The text should be enclosed in the appropriate
   comment syntax for the file format. Please also get in touch with
   the Apache Software Foundation to determine if it is appropriate
   to include your project in the list of projects that have made use
   of this license.

   Copyright 2026 Argo Agents Contributors

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```

- [ ] **Step 4: Create README.md**

```markdown
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
```

- [ ] **Step 5: Create CI/CD workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all

  build-matrix:
    name: Build (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --all
```

- [ ] **Step 6: Create GitHub templates**

```markdown
<!-- .github/ISSUE_TEMPLATE/bug_report.md -->
---
name: Bug Report
about: Report a bug in Argo
title: "[BUG] "
labels: bug
assignees: ''
---

## Describe the Bug

A clear and concise description of what the bug is.

## To Reproduce

Steps to reproduce the behavior:

1. ...
2. ...

## Expected Behavior

A clear and concise description of what you expected to happen.

## Environment

- OS: [e.g. Ubuntu 22.04, macOS 14, Windows 11]
- Rust version: [e.g. 1.75.0]
- Argo version: [e.g. 0.1.0]
- LLM Provider: [e.g. Anthropic Claude, OpenAI GPT-4]

## Additional Context

Add any other context about the problem here.
```

```markdown
<!-- .github/ISSUE_TEMPLATE/feature_request.md -->
---
name: Feature Request
about: Suggest a new feature for Argo
title: "[FEATURE] "
labels: enhancement
assignees: ''
---

## Problem Statement

A clear and concise description of what problem this feature would solve.

## Proposed Solution

A clear and concise description of what you want to happen.

## Alternatives Considered

A clear and concise description of any alternative solutions or features you've considered.

## Additional Context

Add any other context or screenshots about the feature request here.
```

```markdown
<!-- .github/PULL_REQUEST_TEMPLATE.md -->
## Summary

<!-- What does this PR do? -->

## Changes

<!-- List the main changes -->

## Testing

<!-- How was this tested? -->

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
- [ ] Documentation updated (if applicable)
```

```
# .github/CODEOWNERS
# Default owners for everything in the repo
* @argo-agents/core-team

# RFCs require architect review
docs/rfcs/ @argo-agents/architects

# CI/CD changes require DevOps review
.github/ @argo-agents/devops
```

- [ ] **Step 7: Create crate scaffolds**

Create minimal `Cargo.toml` and `src/lib.rs` for each crate:

```toml
# crates/argo-core/Cargo.toml
[package]
name = "argo-core"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core actor engine, message types, and agent execution for Argo"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
uuid.workspace = true
chrono.workspace = true
async-trait = "0.1"
```

```toml
# crates/argo-memory/Cargo.toml
[package]
name = "argo-memory"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Memory system for Argo: Redis, SurrealDB, and Qdrant layers"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
uuid.workspace = true
async-trait = "0.1"
```

```toml
# crates/argo-heal/Cargo.toml
[package]
name = "argo-heal"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Self-healing engine with error taxonomy and recovery strategies"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
async-trait = "0.1"
```

```toml
# crates/argo-tools/Cargo.toml
[package]
name = "argo-tools"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Built-in tool library for Argo agents"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
async-trait = "0.1"
```

```toml
# crates/argo-mcp/Cargo.toml
[package]
name = "argo-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "MCP protocol connector for Argo"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
async-trait = "0.1"
reqwest = { version = "0.12", features = ["json"] }
```

```toml
# crates/argo-observe/Cargo.toml
[package]
name = "argo-observe"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "OpenTelemetry observability for Argo"

[dependencies]
tokio.workspace = true
serde.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
opentelemetry = "0.23"
opentelemetry_sdk = { version = "0.23", features = ["rt-tokio"] }
opentelemetry-otlp = "0.16"
```

```toml
# crates/argo-cli/Cargo.toml
[package]
name = "argo-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "CLI for Argo agent framework"

[[bin]]
name = "argo"
path = "src/main.rs"

[dependencies]
argo-core = { path = "../argo-core" }
argo-memory = { path = "../argo-memory" }
argo-heal = { path = "../argo-heal" }
argo-tools = { path = "../argo-tools" }
argo-mcp = { path = "../argo-mcp" }
argo-observe = { path = "../argo-observe" }
tokio.workspace = true
anyhow.workspace = true
clap = { version = "4", features = ["derive"] }
tracing.workspace = true
tracing-subscriber.workspace = true
```

Create placeholder `src/lib.rs` files for each crate with a minimal test:

```rust
// crates/argo-core/src/lib.rs
//! Argo Core — Actor engine, message types, and agent execution.

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```

(Same pattern for all other crates with appropriate doc comments)

- [ ] **Step 8: Verify workspace builds**

Run: `cargo check --workspace`
Expected: Compiles successfully (empty crates, no errors)

- [ ] **Step 9: Commit**

```bash
git init
git add .
git commit -m "chore: initialize Argo workspace with crate scaffolds, CI, and docs structure"
```

---

## Task Group 2: Architecture RFCs (A-01 to A-12)

These tasks are independent and can be executed in parallel. Each RFC follows the template: Title, Status, Summary, Motivation, Detailed Design, Alternatives, Drawbacks, Unresolved Questions.

### Task 2: Write A-01 — Actor Engine Design RFC

**Covers:** P0-T01

**Files:**
- Create: `docs/rfcs/architecture/A-01-actor-engine.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-01: Actor Engine Design

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the Actix-based actor hierarchy, typed message system, supervisor tree, and restart policies for Argo's core runtime.

## Motivation

Argo agents must run concurrently, be isolated from each other, and recover from failures automatically. The actor model provides all three guarantees natively. Each agent is an actor with private state, a mailbox, and message handlers. Actors communicate via typed messages, never shared memory.

## Detailed Design

### Actor Hierarchy

```
SupervisorActor
├── OrchestratorActor (for multi-agent pipelines)
│   ├── WorkerAgent_1
│   ├── WorkerAgent_2
│   └── WorkerAgent_N
└── SingleAgent (for standalone agents)
    └── HealEngine (as sub-actor)
```

### Core Actor: AgentActor

```rust
use actix::{Actor, Context, Handler, ResponseFuture};
use async_trait::async_trait;

pub struct AgentActor {
    config: AgentConfig,
    memory: MemoryHandle,
    heal: HealEngine,
    tools: ToolRegistry,
    llm: Box<dyn LlmProvider>,
    trace: AgentTrace,
}

impl Actor for AgentActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteTask> for AgentActor {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: ExecuteTask, _ctx: &mut Self::Context) -> Self::Result {
        let config = self.config.clone();
        let memory = self.memory.clone();
        let tools = self.tools.clone();
        let llm = self.llm.clone();
        let heal = self.heal.clone();
        let mut trace = self.trace.clone();

        Box::pin(async move {
            execute_task_loop(&config, &memory, &tools, &*llm, &heal, &mut trace, msg.goal).await
        })
    }
}
```

### Message Types

All messages are serializable with MessagePack via `rmp-serde`.

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTask {
    pub task_id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOp {
    Read { key: String },
    Write { key: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealRequest {
    pub error: AgentError,
    pub context: HealContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgent {
    pub agent_id: Uuid,
    pub config: AgentConfig,
    pub goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDone {
    pub task_id: Uuid,
    pub result: TaskResult,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailed {
    pub task_id: Uuid,
    pub error: AgentError,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectRequest {
    pub run_id: Uuid,
}
```

### Supervisor Tree

The `SupervisorActor` monitors child actors and applies restart strategies:

```rust
pub enum RestartStrategy {
    /// Restart only the failed actor
    OneForOne,
    /// Restart the failed actor and all actors started after it
    RestForOne,
    /// Restart all child actors
    OneForAll,
}

pub struct SupervisorActor {
    children: Vec<Addr<AgentActor>>,
    strategy: RestartStrategy,
}
```

When an actor panics or returns an error, the supervisor:
1. Receives the `Actix::SupervisionEvent`
2. Logs the failure with full context
3. Applies the restart strategy
4. Optionally reassigns the failed actor's task to another worker

### Message Serialization

Messages are serialized with MessagePack (rmp-serde) for binary efficiency over the actor mailbox. This avoids JSON parsing overhead for inter-actor communication.

```rust
use rmp_serde::{encode, decode};

pub fn serialize_message<T: Serialize>(msg: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    encode::to_vec(msg)
}

pub fn deserialize_message<T: DeserializeOwned>(data: &[u8]) -> Result<T, rmp_serde::decode::Error> {
    decode::from_slice(data)
}
```

### Actor Lifecycle

1. **Spawn**: `SupervisorActor` creates `AgentActor` with config, registers in child list
2. **Running**: Actor processes messages from mailbox one at a time
3. **Failure**: Actor panics or returns unrecoverable error → supervisor notified
4. **Restart**: Supervisor applies strategy, creates new actor instance
5. **Shutdown**: Graceful stop via `Context::stop()`, actor flushes state to memory

## Alternatives Considered

1. **Tokio tasks without Actix**: Simpler, but no built-in supervision, restart, or mailbox isolation. Would require manual implementation.
2. **Ractor**: Newer actor framework for Rust. Less mature than Actix, smaller community.
3. **Custom actor implementation**: Full control, but reinvents well-solved problems.

## Drawbacks

- Actix adds a dependency, though it's battle-tested and well-maintained
- Actor model adds complexity for simple single-agent use cases
- Message serialization overhead (mitigated by MessagePack binary format)

## Unresolved Questions

- Should actors support `SupervisionStrategy::OneForRestart` with configurable max restart count?
- How to handle actor state migration during restart (transfer state to new instance)?
- Should the supervisor be aware of agent health (heartbeat) or only react to failures?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-01-actor-engine.md
git commit -m "docs(rfc): add A-01 actor engine design"
```

---

### Task 3: Write A-02 — Memory Architecture RFC

**Covers:** P0-T02

**Files:**
- Create: `docs/rfcs/architecture/A-02-memory-architecture.md`

- [ ] **Step 1: Write the RFC**

```markdown
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

    // Find how many oldest turns to summarize
    let mut tokens_to_remove = 0;
    let mut cutoff = 0;
    for (i, turn) in turns.iter().enumerate() {
        tokens_to_remove += estimate_tokens_single(turn);
        cutoff = i;
        if current_tokens - tokens_to_remove < context_limit as f64 * 0.6 {
            break;
        }
    }

    // Archive full turns
    let archived = turns[..=cutoff].to_vec();
    memory.archive_turns(agent_id, run_id, &archived).await?;

    // Summarize
    let summary = llm.summarize(&archived).await?;

    // Replace with summary
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
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-02-memory-architecture.md
git commit -m "docs(rfc): add A-02 memory architecture"
```

---

### Task 4: Write A-03 — Error Taxonomy RFC

**Covers:** P0-T03

**Files:**
- Create: `docs/rfcs/architecture/A-03-error-taxonomy.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-03: Error Taxonomy

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Complete classification of every error type in Argo, with metadata requirements and classification rules for the self-healing system.

## Motivation

The self-healing system needs a structured error taxonomy to select appropriate recovery strategies. Every error must be classified into a known type with enough metadata for the heal engine to act on it.

## Detailed Design

### Error Enum

```rust
use thiserror::Error;
use std::time::Duration;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum AgentError {
    // === LLM Errors ===

    #[error("Rate limited by LLM provider, retry after {retry_after:?}")]
    LlmRateLimit {
        retry_after: Duration,
        provider: String,
    },

    #[error("LLM context overflow: {current} tokens exceeds limit {limit}")]
    LlmContextOverflow {
        current: usize,
        limit: usize,
    },

    #[error("LLM hallucination detected: {evidence}")]
    LlmHallucination {
        evidence: String,
        confidence: f32,
    },

    #[error("LLM refused to complete: {reason}")]
    LlmRefusal {
        reason: String,
        provider: String,
    },

    #[error("LLM timeout after {elapsed:?}")]
    LlmTimeout {
        elapsed: Duration,
        provider: String,
    },

    #[error("LLM provider {provider} is down: {reason}")]
    LlmProviderDown {
        provider: String,
        reason: String,
    },

    // === Tool Errors ===

    #[error("Tool not found: {name}")]
    ToolNotFound {
        name: String,
    },

    #[error("Tool {name} execution failed: {reason}")]
    ToolExecutionFailed {
        name: String,
        reason: String,
        exit_code: Option<i32>,
    },

    #[error("Tool {name} timed out after {elapsed:?}")]
    ToolTimeout {
        name: String,
        elapsed: Duration,
    },

    #[error("Tool {name} permission denied for {resource}")]
    ToolPermissionDenied {
        name: String,
        resource: String,
    },

    #[error("Tool {name} produced invalid output: {output}")]
    ToolOutputInvalid {
        name: String,
        output: String,
    },

    // === Logic Errors ===

    #[error("Infinite loop detected after {iteration_count} iterations")]
    InfiniteLoop {
        iteration_count: usize,
    },

    #[error("Goal appears unachievable: {reason}")]
    GoalUnachievable {
        reason: String,
    },

    #[error("Plan is invalid: {reason}")]
    PlanInvalid {
        plan: String,
        reason: String,
    },

    #[error("Agent context is corrupted")]
    ContextCorrupted,

    // === Infrastructure Errors ===

    #[error("Memory store {store} is unavailable: {reason}")]
    MemoryUnavailable {
        store: MemoryStore,
        reason: String,
    },

    #[error("MCP connection to {server} failed: {reason}")]
    McpConnectionFailed {
        server: String,
        reason: String,
    },

    #[error("Network timeout for {url} after {elapsed:?}")]
    NetworkTimeout {
        url: String,
        elapsed: Duration,
    },

    // === Agent Errors ===

    #[error("Sub-agent {agent_id} failed: {error}")]
    SubAgentFailed {
        agent_id: String,
        error: Box<AgentError>,
    },

    #[error("Orchestrator failed: {reason}")]
    OrchestratorFailed {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStore {
    Redis,
    SurrealDB,
    Qdrant,
}
```

### Classification Rules

When an error occurs, the classifier:

1. Inspects the error type (Rust enum variant)
2. Extracts metadata fields
3. Classifies severity: `recoverable`, `degradable`, `fatal`
4. Selects initial heal strategy based on error type

| Error Type | Severity | Initial Strategy |
|---|---|---|
| `LlmRateLimit` | Recoverable | Retry with backoff |
| `LlmContextOverflow` | Recoverable | Context overflow handling |
| `LlmHallucination` | Recoverable | Reframe prompt |
| `LlmRefusal` | Degradable | Reframe prompt |
| `LlmTimeout` | Recoverable | Retry with backoff |
| `LlmProviderDown` | Degradable | Change provider |
| `ToolNotFound` | Degradable | Swap tool |
| `ToolExecutionFailed` | Recoverable | Retry, then swap tool |
| `ToolTimeout` | Recoverable | Retry with backoff |
| `ToolPermissionDenied` | Fatal | Report to user |
| `ToolOutputInvalid` | Recoverable | Reframe prompt |
| `InfiniteLoop` | Degradable | Reduce scope |
| `GoalUnachievable` | Fatal | Report to user |
| `PlanInvalid` | Recoverable | Decompose |
| `ContextCorrupted` | Fatal | Report to user |
| `MemoryUnavailable` | Degradable | Retry, then continue without memory |
| `McpConnectionFailed` | Recoverable | Retry with backoff |
| `NetworkTimeout` | Recoverable | Retry with backoff |
| `SubAgentFailed` | Degradable | Spawn new sub-agent |
| `OrchestratorFailed` | Fatal | Report to user |

### Error Metadata

Every error carries:

```rust
pub struct ErrorContext {
    pub error: AgentError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: String,
    pub run_id: String,
    pub task_id: uuid::Uuid,
    pub iteration: usize,
    pub previous_errors: Vec<AgentError>,  // errors in this run so far
    pub current_plan: Option<String>,
}
```

## Alternatives Considered

1. **Flat error codes (E001, E002)**: Simpler, but loses semantic meaning and makes heal strategy selection harder.
2. **String-based errors**: Maximum flexibility, but no type safety, no exhaustive matching.
3. **Error hierarchy (Error → LlmError → RateLimitError)**: More granular, but adds complexity without clear benefit for heal strategy selection.

## Drawbacks

- Large enum with many variants increases compile time slightly
- Adding new error types requires updating heal strategy mappings
- Some errors may be misclassified (e.g., a tool error that's actually an LLM error)

## Unresolved Questions

- Should errors carry a `retry_count` field to track how many times this specific error has been retried?
- How to handle compound errors (e.g., LLM timeout while a tool is running)?
- Should we support custom user-defined error types?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-03-error-taxonomy.md
git commit -m "docs(rfc): add A-03 error taxonomy"
```

---

### Task 5: Write A-04 — Heal Strategy Specification RFC

**Covers:** P0-T04

**Files:**
- Create: `docs/rfcs/architecture/A-04-heal-strategy.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-04: Heal Strategy Specification

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the 7 heal strategies: trigger conditions, algorithms, success criteria, and failure escalation behavior for each.

## Motivation

The self-healing system must handle every recoverable error automatically. Each strategy targets a specific class of errors and has a clear success/failure definition. The heal engine runs strategies in order until one succeeds.

## Detailed Design

### Strategy Trait

```rust
#[async_trait]
pub trait HealStrategy: Send + Sync {
    /// Whether this strategy can handle the given error
    fn can_handle(&self, error: &AgentError) -> bool;

    /// Execute the strategy
    async fn apply(&self, ctx: &HealContext) -> HealResult;

    /// Human-readable name for logging
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct HealContext {
    pub error: AgentError,
    pub agent_config: AgentConfig,
    pub current_plan: Plan,
    pub iteration: usize,
    pub past_strategies: Vec<String>,
    pub similar_resolutions: Vec<SemanticMatch>,
}

#[derive(Debug, Clone)]
pub enum HealResult {
    Success { output: String },
    Failed { reason: String },
}
```

### Strategy 1: Retry with Exponential Backoff

**Trigger:** `LlmRateLimit`, `LlmTimeout`, `ToolTimeout`, `NetworkTimeout`, `McpConnectionFailed`

**Algorithm:**
```
base_delay = 1 second
max_delay = 60 seconds
max_retries = 5

delay = min(base_delay * 2^attempt, max_delay)
jitter = random(0, delay * 0.1)
actual_delay = delay + jitter

sleep(actual_delay)
retry operation
```

**Success:** Operation completes without error
**Failure:** Max retries exhausted → escalate to next strategy

### Strategy 2: Reframe Prompt

**Trigger:** `LlmHallucination`, `LlmRefusal`, `ToolOutputInvalid`, `PlanInvalid`

**Algorithm:**
1. Analyze the error context
2. Modify the system prompt:
   - Add explicit constraints ("Do NOT assume X")
   - Add clarification ("The task is specifically about Y, not Z")
   - Change phrasing (simpler language, different structure)
3. Retry with modified prompt

**Success:** LLM produces valid output
**Failure:** Same error persists → escalate

### Strategy 3: Swap Tool

**Trigger:** `ToolNotFound`, `ToolExecutionFailed`, `ToolTimeout`

**Algorithm:**
1. Look up fallback tools in `ToolRegistry.fallbacks`
2. Select first available fallback
3. Rewrite the tool call for the new tool
4. Execute with fallback tool

**Fallback Map:**
```
bash → python (run command via Python subprocess)
files → bash (use cat/echo for file operations)
http → python (use requests library)
web_search → browser (search via headless browser)
```

**Success:** Fallback tool produces valid result
**Failure:** No fallback available or fallback also fails → escalate

### Strategy 4: Decompose

**Trigger:** `GoalUnachievable`, `InfiniteLoop`, `PlanInvalid`

**Algorithm:**
1. Analyze the current plan and failure point
2. Break the failing sub-task into 2-5 smaller pieces
3. Create a new plan with the smaller pieces
4. Execute each piece individually
5. Combine results

**Success:** All sub-tasks complete successfully
**Failure:** Sub-tasks also fail → escalate

### Strategy 5: Spawn Sub-Agent

**Trigger:** Any error after strategies 1-4 fail

**Algorithm:**
1. Create a fresh agent with clean context
2. Assign the failing sub-task to the new agent
3. Give the new agent relevant context from semantic memory
4. Set a tight deadline (60 seconds)
5. Collect result

**Success:** Sub-agent completes the task
**Failure:** Sub-agent also fails → escalate

### Strategy 6: Change LLM Provider

**Trigger:** `LlmProviderDown`, `LlmRateLimit` (after retries), `LlmRefusal`

**Algorithm:**
1. Check available providers in config
2. Switch to next provider in priority list
3. Retry the operation with new provider

**Provider Priority:** Configurable per agent, default: `[anthropic, openai, gemini, ollama]`

**Success:** New provider completes the operation
**Failure:** All providers exhausted → escalate

### Strategy 7: Reduce Scope

**Trigger:** Any error after strategies 1-6 fail

**Algorithm:**
1. Analyze the original goal
2. Identify a simpler version that's still valuable
3. Generate a reduced-scope plan
4. Execute the simplified version
5. Report partial success

**Success:** Simplified goal achieved
**Failure:** Even simplified version fails → all strategies exhausted

### Strategy Selection (Memory-Informed)

Before running strategies, the heal engine queries semantic memory:

```rust
pub async fn select_strategies(
    error: &AgentError,
    memory: &SemanticMemory,
) -> Vec<Box<dyn HealStrategy>> {
    // Query for similar past errors
    let embedding = embed_error(error);
    let similar = memory.query_similar_errors(&embedding, 5).await;

    // If we have a history of what worked, start from that
    if let Some(best_resolution) = similar.iter().max_by_key(|r| r.confidence) {
        return reorder_strategies(best_resolution.strategy);
    }

    // Default order from error taxonomy
    default_strategies_for_error(error)
}
```

### Post-Mortem Loop

After successful healing:

```
Error resolved
        │
        ▼
LLM reflects:
  - What was the error?
  - Why did it occur?
  - Which strategy resolved it?
  - What could have prevented it?
        │
        ▼
Structured lesson written
        │
        ▼
Lesson embedded and stored in Qdrant (argo_lessons)
        │
        ▼
Lesson stored in SurrealDB for queryable history
```

## Alternatives Considered

1. **Fixed strategy order**: Simpler, but doesn't leverage past experience.
2. **Machine learning for strategy selection**: More adaptive, but adds complexity and training data requirements.
3. **User-configurable strategy order**: Maximum flexibility, but most users won't configure it.

## Drawbacks

- 7 strategies add code complexity
- Post-mortem loop adds latency after error resolution
- Strategy effectiveness depends on error classification accuracy

## Unresolved Questions

- Should strategies have configurable retry counts?
- How to handle cascading failures (error in strategy triggers new error)?
- Should users be able to register custom heal strategies?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-04-heal-strategy.md
git commit -m "docs(rfc): add A-04 heal strategy specification"
```

---

### Task 6: Write A-05 — LLM Provider Trait RFC

**Covers:** P0-T05

**Files:**
- Create: `docs/rfcs/architecture/A-05-llm-provider-trait.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-05: LLM Provider Trait

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the `LlmProvider` trait, request/response types, error types, and streaming contract for all LLM adapters.

## Motivation

Argo must support multiple LLM providers (Anthropic, OpenAI, Gemini, Ollama) with a unified interface. The trait isolates provider-specific logic and enables the heal engine to switch providers transparently.

## Detailed Design

### Provider Trait

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request and get a response
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Send a completion request and stream tokens
    async fn stream(&self, request: CompletionRequest) -> Result<BoxStream<'static, Token>, LlmError>;

    /// Provider name (e.g., "anthropic", "openai")
    fn provider_name(&self) -> &str;

    /// Model name (e.g., "claude-sonnet-4-6")
    fn model_name(&self) -> &str;

    /// Context window limit in tokens
    fn context_limit(&self) -> usize;

    /// Maximum output tokens
    fn max_output_tokens(&self) -> usize;
}
```

### Request Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    MultiPart(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text { text: String },
    Image { url: String, media_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
```

### Response Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCallRequest>,
    pub usage: TokenUsage,
    pub stop_reason: StopReason,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Text,
    ToolCallStart,
    ToolCallInput,
}
```

### Error Types

```rust
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum LlmError {
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Context overflow: {current} tokens, limit is {limit}")]
    ContextOverflow { current: usize, limit: usize },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },

    #[error("Request timed out after {elapsed_ms}ms")]
    Timeout { elapsed_ms: u64 },

    #[error("Provider error: {status} {message}")]
    ProviderError { status: u16, message: String },

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("Invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("Streaming error: {reason}")]
    StreamingError { reason: String },
}
```

### Adapter Implementations

**Anthropic Claude:**

```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": request.max_tokens.unwrap_or(8192),
            "system": request.system_prompt,
            "messages": serialize_messages(&request.messages),
            "tools": serialize_tools(&request.tools.unwrap_or_default()),
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError { reason: e.to_string() })?;

        parse_anthropic_response(response).await
    }

    fn provider_name(&self) -> &str { "anthropic" }
    fn model_name(&self) -> &str { &self.model }
    fn context_limit(&self) -> usize { 200_000 }
    fn max_output_tokens(&self) -> usize { 8192 }
}
```

**OpenAI:**

```rust
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": serialize_messages(&request.messages),
            "max_tokens": request.max_tokens,
            "tools": serialize_tools(&request.tools.unwrap_or_default()),
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError { reason: e.to_string() })?;

        parse_openai_response(response).await
    }

    fn provider_name(&self) -> &str { "openai" }
    fn model_name(&self) -> &str { &self.model }
    fn context_limit(&self) -> usize { 128_000 }
    fn max_output_tokens(&self) -> usize { 16384 }
}
```

### Provider Factory

```rust
pub fn create_provider(config: &ModelConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider.as_str() {
        "anthropic" => Ok(Box::new(AnthropicProvider::new(
            std::env::var("ANTHROPIC_API_KEY").map_err(|_| LlmError::AuthenticationFailed { reason: "ANTHROPIC_API_KEY not set".into() })?,
            config.model.clone(),
        ))),
        "openai" => Ok(Box::new(OpenAiProvider::new(
            std::env::var("OPENAI_API_KEY").map_err(|_| LlmError::AuthenticationFailed { reason: "OPENAI_API_KEY not set".into() })?,
            config.model.clone(),
        ))),
        "ollama" => Ok(Box::new(OllamaProvider::new(
            config.ollama_url.clone().unwrap_or_else(|| "http://localhost:11434".into()),
            config.model.clone(),
        ))),
        _ => Err(LlmError::ProviderError { status: 0, message: format!("Unknown provider: {}", config.provider) }),
    }
}
```

## Alternatives Considered

1. **REST API between SDKs and core**: Simpler, but adds latency and loses type safety.
2. **Generic HTTP client**: Maximum flexibility, but loses provider-specific optimizations.
3. **Provider-specific enums instead of trait**: Simpler for single provider, but doesn't scale.

## Drawbacks

- Each provider adapter must handle provider-specific quirks
- Streaming contract adds complexity
- Provider API changes require adapter updates

## Unresolved Questions

- Should providers support structured output (JSON mode) natively?
- How to handle provider-specific features (e.g., Anthropic's tool use vs OpenAI's function calling)?
- Should we support batch completion requests for cost optimization?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-05-llm-provider-trait.md
git commit -m "docs(rfc): add A-05 LLM provider trait"
```

---

### Task 7: Write A-06 — Tool Trait & Registry RFC

**Covers:** P0-T06

**Files:**
- Create: `docs/rfcs/architecture/A-06-tool-trait-registry.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-06: Tool Trait & Registry

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the `Tool` trait, permission model, hot-reload protocol, and fallback registration for Argo's tool system.

## Motivation

Tools are the primary way agents interact with the outside world. Every tool must have a consistent interface, declared permissions, and support for hot-reload and fallbacks.

## Detailed Design

### Tool Trait

```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (unique identifier)
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// JSON Schema for input validation
    fn input_schema(&self) -> Value;

    /// JSON Schema for output validation
    fn output_schema(&self) -> Value;

    /// Required permissions
    fn permissions(&self) -> ToolPermissions;

    /// Execute the tool with given input
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError>;
}

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub agent_id: String,
    pub run_id: String,
    pub working_dir: String,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissions {
    pub allow_filesystem: bool,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub working_directory: Option<String>,
    pub allowed_paths: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub max_execution_time: Duration,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ToolError {
    #[error("Permission denied: {resource}")]
    PermissionDenied { resource: String },

    #[error("Execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Timeout after {elapsed:?}")]
    Timeout { elapsed: Duration },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Output too large: {size} bytes exceeds limit")]
    OutputTooLarge { size: usize },
}
```

### Tool Registry

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    versions: HashMap<String, semver::Version>,
    fallbacks: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            versions: HashMap::new(),
            fallbacks: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name.clone(), tool);
        self.versions.insert(name, semver::Version::new(0, 1, 0));
    }

    /// Unregister a tool
    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
        self.versions.remove(name);
    }

    /// Look up a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Register fallback tools for a given tool
    pub fn register_fallbacks(&mut self, tool_name: &str, fallbacks: Vec<String>) {
        self.fallbacks.insert(tool_name.to_string(), fallbacks);
    }

    /// Get fallback tools for a given tool
    pub fn get_fallbacks(&self, tool_name: &str) -> Vec<Arc<dyn Tool>> {
        self.fallbacks
            .get(tool_name)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.tools.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Hot-reload: replace a tool with a new version
    pub fn hot_reload(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        if let Some(version) = self.versions.get_mut(&name) {
            *version = version.clone().increment();
        }
        self.tools.insert(name, tool);
    }

    /// List all registered tools
    pub fn list(&self) -> Vec<ToolInfo> {
        self.tools.iter().map(|(name, tool)| ToolInfo {
            name: name.clone(),
            description: tool.description().to_string(),
            version: self.versions.get(name).cloned().unwrap_or_else(|| semver::Version::new(0, 0, 0)),
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: semver::Version,
}
```

### Built-in Tools

```rust
// Bash tool
pub struct BashTool {
    working_directory: String,
    max_execution_time: Duration,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Execute shell commands" }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "timeout": { "type": "integer", "description": "Timeout in seconds" }
            },
            "required": ["command"]
        })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: true,
            allow_network: false,
            allow_subprocess: true,
            working_directory: Some(self.working_directory.clone()),
            allowed_paths: vec![],
            allowed_domains: vec![],
            max_execution_time: self.max_execution_time,
        }
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let command = input["command"].as_str()
            .ok_or_else(|| ToolError::InvalidInput { reason: "missing 'command' field".into() })?;

        // Validate working directory
        // Execute command with timeout
        // Capture stdout/stderr
        // Return result
        todo!()
    }
}

// Files tool
pub struct FilesTool {
    allowed_paths: Vec<String>,
}

// HTTP tool
pub struct HttpTool {
    allowed_domains: Vec<String>,
}
```

### Permission Enforcement

```rust
pub fn check_permissions(tool: &dyn Tool, ctx: &ToolContext) -> Result<(), ToolError> {
    let perms = tool.permissions();

    if perms.allow_filesystem {
        // Verify working_directory and allowed_paths
    }

    if perms.allow_network {
        // Verify allowed_domains
    }

    if perms.allow_subprocess {
        // Verify subprocess is allowed
    }

    Ok(())
}
```

## Alternatives Considered

1. **Macro-based tool registration**: More ergonomic, but less flexible at runtime.
2. **Plugin system with dynamic loading**: Maximum extensibility, but adds complexity.
3. **Tool as a simple function**: Simpler, but loses permission model and hot-reload.

## Drawbacks

- Hot-reload requires careful state management
- Permission model adds overhead to every tool call
- Fallback selection logic can be complex

## Unresolved Questions

- Should tools support async streaming output?
- How to handle tool versioning across agents?
- Should we support tool composition (tool A calls tool B)?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-06-tool-trait-registry.md
git commit -m "docs(rfc): add A-06 tool trait and registry"
```

---

### Task 8: Write A-07 — MCP Connector RFC

**Covers:** P0-T07

**Files:**
- Create: `docs/rfcs/architecture/A-07-mcp-connector.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-07: MCP Connector

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define how Argo implements the MCP client protocol, tool discovery, authentication, format conversion, and reconnection handling.

## Motivation

MCP (Model Context Protocol) enables agents to connect to external tool servers and use their tools as if they were native. Argo must implement the full MCP client protocol to integrate with the growing ecosystem of MCP servers.

## Detailed Design

### MCP Client Protocol

```rust
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Connect to an MCP server
    async fn connect(&self, url: &str, auth: Option<AuthConfig>) -> Result<(), McpError>;

    /// Discover available tools
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;

    /// Invoke a tool
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError>;

    /// Disconnect from server
    async fn disconnect(&self) -> Result<(), McpError>;

    /// Check if connected
    fn is_connected(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Bearer { token: String },
    OAuth2 { client_id: String, client_secret: String },
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum McpError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("Tool execution failed: {reason}")]
    ToolExecutionFailed { reason: String },

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },

    #[error("Server disconnected")]
    Disconnected,
}
```

### SSE Transport

MCP uses Server-Sent Events (SSE) for communication:

```rust
pub struct SseMcpClient {
    client: reqwest::Client,
    endpoint: String,
    session_id: Option<String>,
    connected: bool,
}

#[async_trait]
impl McpClient for SseMcpClient {
    async fn connect(&self, url: &str, auth: Option<AuthConfig>) -> Result<(), McpError> {
        // 1. Send initialize request
        // 2. Receive server capabilities
        // 3. Store session ID from headers
        // 4. Start SSE listener for server messages
        todo!()
    }

    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        // Send tools/list request
        // Parse response
        todo!()
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        // Send tools/call request
        // Wait for response
        // Parse result
        todo!()
    }
}
```

### Tool Registration

When an MCP server is connected, its tools are registered in the `ToolRegistry`:

```rust
pub async fn register_mcp_tools(
    registry: &mut ToolRegistry,
    client: &dyn McpClient,
    server_name: &str,
) -> Result<(), McpError> {
    let tools = client.list_tools().await?;

    for tool in tools {
        let mcp_tool = McpToolAdapter {
            client: client.clone(),
            server_name: server_name.to_string(),
            tool: tool.clone(),
        };
        registry.register(Arc::new(mcp_tool));
    }

    Ok(())
}

/// Adapter that wraps an MCP tool as an Argo Tool
struct McpToolAdapter {
    client: Arc<dyn McpClient>,
    server_name: String,
    tool: McpTool,
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> &str {
        &self.tool.name
    }

    fn description(&self) -> &str {
        &self.tool.description
    }

    fn input_schema(&self) -> Value {
        self.tool.input_schema.clone()
    }

    fn output_schema(&self) -> Value {
        serde_json::json!({ "type": "object" })
    }

    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            allow_filesystem: false,
            allow_network: true,
            allow_subprocess: false,
            working_directory: None,
            allowed_paths: vec![],
            allowed_domains: vec![],
            max_execution_time: Duration::from_secs(30),
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<Value, ToolError> {
        self.client.call_tool(&self.tool.name, input).await
            .map_err(|e| ToolError::ExecutionFailed { reason: e.to_string() })
    }
}
```

### Reconnection

```rust
pub struct ResilientMcpClient {
    inner: SseMcpClient,
    max_reconnect_attempts: u32,
    reconnect_delay: Duration,
}

impl ResilientMcpClient {
    pub async fn ensure_connected(&self) -> Result<(), McpError> {
        if self.inner.is_connected() {
            return Ok(());
        }

        for attempt in 0..self.max_reconnect_attempts {
            match self.inner.connect(&self.inner.endpoint, self.inner.auth.clone()).await {
                Ok(()) => return Ok(()),
                Err(_) => {
                    let delay = self.reconnect_delay * 2u32.pow(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(McpError::ConnectionFailed { reason: "Max reconnect attempts exceeded".into() })
    }
}
```

## Alternatives Considered

1. **stdio transport**: Simpler, but requires spawning a subprocess for each MCP server.
2. **HTTP transport**: More standard, but MCP specification uses SSE.
3. **WebSocket transport**: More efficient, but not part of MCP specification.

## Drawbacks

- SSE requires maintaining a long-lived connection
- Reconnection logic adds complexity
- MCP server availability is external dependency

## Unresolved Questions

- Should MCP tools be namespaced (e.g., `asana.create_task`) or use bare names?
- How to handle MCP server rate limits?
- Should we support MCP resources (not just tools)?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-07-mcp-connector.md
git commit -m "docs(rfc): add A-07 MCP connector"
```

---

### Task 9: Write A-08 — Multi-Agent Protocol RFC

**Covers:** P0-T08

**Files:**
- Create: `docs/rfcs/architecture/A-08-multi-agent-protocol.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-08: Multi-Agent Protocol

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the orchestrator/worker message types, agent spawning protocol, result aggregation, AgentPool task distribution, and shared vs isolated memory modes.

## Motivation

Multi-agent systems enable specialization and parallelism. An orchestrator agent plans and delegates, while worker agents specialize and execute. The protocol must be type-safe, efficient, and support fault tolerance.

## Detailed Design

### Orchestrator Actor

```rust
pub struct OrchestratorActor {
    workers: Vec<Addr<WorkerAgent>>,
    config: OrchestratorConfig,
    memory: MemoryHandle,
    pending_tasks: HashMap<Uuid, TaskAssignment>,
}

impl Actor for OrchestratorActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteTask> for OrchestratorActor {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: ExecuteTask, _ctx: &mut Self::Context) -> Self::Result {
        // 1. Plan decomposition
        // 2. Assign sub-tasks to workers
        // 3. Collect results
        // 4. Aggregate and return
        todo!()
    }
}
```

### Message Types

```rust
// Orchestrator → Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTask {
    pub task_id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<Duration>,
    pub memory_mode: MemoryMode,
}

// Worker → Orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplete {
    pub task_id: Uuid,
    pub result: TaskResult,
    pub duration_ms: u64,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailed {
    pub task_id: Uuid,
    pub error: AgentError,
    pub partial_result: Option<String>,
}

// Orchestrator → System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnWorker {
    pub worker_id: Uuid,
    pub config: AgentConfig,
    pub memory_mode: MemoryMode,
}
```

### AgentPool

```rust
pub struct AgentPool {
    workers: Vec<Addr<WorkerAgent>>,
    config: AgentPoolConfig,
    task_queue: VecDeque<AssignTask>,
    busy_workers: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AgentPoolConfig {
    pub worker_count: usize,
    pub agent_template: AgentConfig,
    pub memory_mode: MemoryMode,
    pub max_concurrent_tasks: usize,
}

impl AgentPool {
    pub fn new(config: AgentPoolConfig) -> Self {
        Self {
            workers: Vec::new(),
            config,
            task_queue: VecDeque::new(),
            busy_workers: HashSet::new(),
        }
    }

    /// Distribute tasks to workers
    pub fn distribute(&mut self, tasks: Vec<String>) {
        for goal in tasks {
            let task = AssignTask {
                task_id: Uuid::new_v4(),
                goal,
                context: None,
                deadline: None,
                memory_mode: self.config.memory_mode.clone(),
            };
            self.task_queue.push_back(task);
        }
        self.assign_pending();
    }

    /// Assign pending tasks to idle workers
    fn assign_pending(&mut self) {
        while let Some(task) = self.task_queue.pop_front() {
            if let Some(worker) = self.find_idle_worker() {
                self.busy_workers.insert(worker.id);
                worker.addr.do_send(task);
            } else {
                self.task_queue.push_front(task);
                break;
            }
        }
    }

    /// Map: distribute tasks and collect results
    pub async fn map(&self, goals: Vec<String>) -> Vec<TaskResult> {
        // Distribute, wait for all to complete, return results
        todo!()
    }
}
```

### Memory Modes

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryMode {
    /// Each agent has its own memory namespace
    Isolated,
    /// Workers share long-term and semantic memory
    Shared,
    /// Per-agent long-term and semantic, isolated short-term
    Persistent,
}
```

| Mode | Short-term (Redis) | Long-term (SurrealDB) | Semantic (Qdrant) |
|---|---|---|---|
| `Isolated` | Per-agent | Per-agent | Per-agent |
| `Shared` | Per-agent | Shared pool | Shared pool |
| `Persistent` | Per-agent | Per-agent | Per-agent |

### Task Assignment Protocol

```rust
pub struct TaskAssignment {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub goal: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl OrchestratorActor {
    fn assign_task(&mut self, task: AssignTask, ctx: &mut Context<Self>) {
        // Find least-busy worker
        let worker = self.workers.iter()
            .filter(|w| !self.pending_tasks.values().any(|t| t.worker_id == w.id))
            .min_by_key(|w| self.count_worker_tasks(w.id))
            .expect("no available workers");

        self.pending_tasks.insert(task.task_id, TaskAssignment {
            task_id: task.task_id,
            worker_id: worker.id,
            goal: task.goal.clone(),
            assigned_at: chrono::Utc::now(),
            deadline: task.deadline,
        });

        worker.addr.do_send(task);
    }
}
```

## Alternatives Considered

1. **Peer-to-peer agents**: No orchestrator, agents communicate directly. Simpler, but harder to coordinate.
2. **Central message bus**: All agents publish/subscribe to a bus. More decoupled, but adds latency.
3. **Function call instead of message passing**: Simpler, but loses isolation and fault tolerance.

## Drawbacks

- Orchestrator is a single point of failure (mitigated by supervisor)
- Message passing adds serialization overhead
- Complex coordination for large agent pools

## Unresolved Questions

- Should workers be able to spawn their own sub-workers?
- How to handle worker load balancing (round-robin vs least-busy)?
- Should the orchestrator support work-stealing?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-08-multi-agent-protocol.md
git commit -m "docs(rfc): add A-08 multi-agent protocol"
```

---

### Task 10: Write A-09 — Self-Improvement System RFC

**Covers:** P0-T09

**Files:**
- Create: `docs/rfcs/architecture/A-09-self-improvement.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-09: Self-Improvement System

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the daily growth cycle algorithm, pattern detection rules, improvement proposal schema, and auto-apply vs flag-for-review behavior.

## Motivation

Agents should get better over time without human intervention. The self-improvement system analyzes past errors and successes, detects patterns, and applies improvements automatically (for low-risk changes) or flags them for review (for high-risk changes).

## Detailed Design

### Growth Cycle

Runs every 24 hours (configurable) in the background:

```rust
pub struct GrowthCycle {
    memory: MemoryHandle,
    llm: Box<dyn LlmProvider>,
    config: GrowthConfig,
}

#[derive(Debug, Clone)]
pub struct GrowthConfig {
    pub interval: Duration,
    pub auto_apply_threshold: f32,  // confidence threshold for auto-apply
    pub max_proposals_per_cycle: usize,
}

impl GrowthCycle {
    pub async fn run(&self, agent_id: &str) -> Result<GrowthReport, GrowthError> {
        // 1. Pull error records from last 24h
        let errors = self.memory.query_errors(agent_id, Duration::from_secs(86400)).await?;

        // 2. Detect patterns
        let patterns = self.detect_patterns(&errors).await?;

        // 3. Generate improvement proposals
        let proposals = self.generate_proposals(&patterns).await?;

        // 4. Auto-apply low-risk, flag high-risk
        let applied = self.apply_proposals(&proposals).await?;

        // 5. Update semantic memory
        self.update_memory(&patterns, &applied).await?;

        // 6. Write growth report
        let report = GrowthReport {
            agent_id: agent_id.to_string(),
            timestamp: chrono::Utc::now(),
            errors_analyzed: errors.len(),
            patterns_detected: patterns.len(),
            proposals_generated: proposals.len(),
            proposals_applied: applied.len(),
            patterns,
            proposals,
        };

        self.memory.store_growth_report(&report).await?;
        Ok(report)
    }
}
```

### Pattern Detection

```rust
impl GrowthCycle {
    async fn detect_patterns(&self, errors: &[ErrorRecord]) -> Result<Vec<Pattern>, GrowthError> {
        let mut patterns = Vec::new();

        // Pattern 1: Same error 3+ times
        let mut error_counts: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for error in errors {
            error_counts.entry(error.error_type.clone())
                .or_default()
                .push(error);
        }
        for (error_type, occurrences) in &error_counts {
            if occurrences.len() >= 3 {
                patterns.push(Pattern {
                    pattern_type: PatternType::RecurringError,
                    description: format!("Error '{}' occurred {} times", error_type, occurrences.len()),
                    confidence: 0.9,
                    evidence: occurrences.iter().map(|e| e.id.clone()).collect(),
                });
            }
        }

        // Pattern 2: Same tool failing
        let mut tool_failures: HashMap<String, Vec<&ErrorRecord>> = HashMap::new();
        for error in errors {
            if let ErrorContext::Tool { tool_name, .. } = &error.context {
                tool_failures.entry(tool_name.clone())
                    .or_default()
                    .push(error);
            }
        }
        for (tool, failures) in &tool_failures {
            if failures.len() >= 2 {
                patterns.push(Pattern {
                    pattern_type: PatternType::ToolFailure,
                    description: format!("Tool '{}' failed {} times", tool, failures.len()),
                    confidence: 0.85,
                    evidence: failures.iter().map(|e| e.id.clone()).collect(),
                });
            }
        }

        // Pattern 3: Same task type succeeding
        let mut task_successes: HashMap<String, Vec<&TaskRecord>> = HashMap::new();
        // ... similar analysis for successes

        Ok(patterns)
    }
}
```

### Improvement Proposals

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub id: Uuid,
    pub proposal_type: ProposalType,
    pub target: String,
    pub content: String,
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Add context to agent's system prompt
    PromptUpdate,
    /// Add a pre-check step before tool execution
    PreCheck,
    /// Reorder heal strategies for specific error types
    StrategyReorder,
    /// Change default configuration
    ConfigChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,      // Auto-apply
    Medium,   // Auto-apply with logging
    High,     // Flag for developer review
}
```

### Auto-Apply Logic

```rust
impl GrowthCycle {
    async fn apply_proposals(&self, proposals: &[ImprovementProposal]) -> Result<Vec<AppliedProposal>, GrowthError> {
        let mut applied = Vec::new();

        for proposal in proposals {
            match proposal.risk_level {
                RiskLevel::Low | RiskLevel::Medium => {
                    // Auto-apply
                    self.apply_proposal(proposal).await?;
                    applied.push(AppliedProposal {
                        proposal: proposal.clone(),
                        applied_at: chrono::Utc::now(),
                        auto_applied: true,
                    });
                }
                RiskLevel::High => {
                    // Flag for review
                    self.flag_for_review(proposal).await?;
                }
            }
        }

        Ok(applied)
    }

    async fn apply_proposal(&self, proposal: &ImprovementProposal) -> Result<(), GrowthError> {
        match proposal.proposal_type {
            ProposalType::PromptUpdate => {
                // Append to agent's system prompt
                self.memory.append_to_prompt(&proposal.target, &proposal.content).await?;
            }
            ProposalType::PreCheck => {
                // Add pre-check step to execution pipeline
                self.memory.add_pre_check(&proposal.target, &proposal.content).await?;
            }
            ProposalType::StrategyReorder => {
                // Update heal strategy order
                self.memory.update_strategy_order(&proposal.target, &proposal.content).await?;
            }
            ProposalType::ConfigChange => {
                // Update configuration
                self.memory.update_config(&proposal.target, &proposal.content).await?;
            }
        }
        Ok(())
    }
}
```

### Growth Report

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthReport {
    pub agent_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub errors_analyzed: usize,
    pub patterns_detected: usize,
    pub proposals_generated: usize,
    pub proposals_applied: usize,
    pub patterns: Vec<Pattern>,
    pub proposals: Vec<ImprovementProposal>,
}
```

## Alternatives Considered

1. **Manual improvement only**: Simpler, but defeats the purpose of self-improvement.
2. **Reinforcement learning**: More adaptive, but requires reward signals and training data.
3. **Rule-based improvements**: Deterministic, but less flexible than LLM-driven analysis.

## Drawbacks

- Growth cycle consumes LLM tokens
- Auto-applied changes may degrade performance
- Pattern detection may produce false positives

## Unresolved Questions

- How to measure if improvements actually help (A/B testing)?
- Should growth cycle run more frequently for high-traffic agents?
- How to handle conflicting improvements from different growth cycles?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-09-self-improvement.md
git commit -m "docs(rfc): add A-09 self-improvement system"
```

---

### Task 11: Write A-10 — Loop Agent & Scoring RFC

**Covers:** P0-T10

**Files:**
- Create: `docs/rfcs/architecture/A-10-loop-agent-scoring.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-10: Loop Agent & Scoring

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define the quality rubric schema, scoring algorithm, iteration management, and termination conditions for loop agents.

## Motivation

Loop agents run autonomously until they meet their own quality standard. The scoring system must be transparent, configurable, and produce consistent results.

## Detailed Design

### Quality Rubric

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRubric {
    pub criteria: Vec<Criterion>,
    pub threshold: f32,       // 0.0 to 1.0
    pub max_iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub name: String,
    pub weight: f32,          // 0.0 to 1.0, must sum to 1.0
    pub description: String,  // LLM uses this to score
}

impl QualityRubric {
    pub fn validate(&self) -> Result<(), RubricError> {
        let total_weight: f32 = self.criteria.iter().map(|c| c.weight).sum();
        if (total_weight - 1.0).abs() > 0.001 {
            return Err(RubricError::InvalidWeights { total: total_weight });
        }
        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err(RubricError::InvalidThreshold { threshold: self.threshold });
        }
        Ok(())
    }
}
```

### Scoring Algorithm

```rust
pub struct LoopAgent {
    config: LoopAgentConfig,
    rubric: QualityRubric,
    llm: Box<dyn LlmProvider>,
    trace: AgentTrace,
}

impl LoopAgent {
    pub async fn run(&self, goal: &str) -> Result<LoopResult, LoopError> {
        let mut iteration = 0;
        let mut best_score = 0.0;
        let mut best_output = String::new();

        loop {
            iteration += 1;

            // 1. Execute task
            let output = self.execute_iteration(goal, iteration).await?;

            // 2. Score output
            let scores = self.score_output(&output, &self.rubric).await?;
            let weighted_score = self.calculate_weighted_score(&scores);

            // 3. Track best result
            if weighted_score > best_score {
                best_score = weighted_score;
                best_output = output.clone();
            }

            // 4. Check termination conditions
            if weighted_score >= self.rubric.threshold {
                return Ok(LoopResult {
                    output: best_output,
                    score: best_score,
                    iterations: iteration,
                    scores,
                });
            }

            if iteration >= self.rubric.max_iterations {
                return Ok(LoopResult {
                    output: best_output,
                    score: best_score,
                    iterations: iteration,
                    scores,
                });
            }

            // 5. Analyze gaps and re-plan
            let gaps = self.analyze_gaps(&scores).await?;
            self.replan(&gaps).await?;
        }
    }

    async fn score_output(&self, output: &str, rubric: &QualityRubric) -> Result<Vec<ScoredCriterion>, LoopError> {
        let mut scores = Vec::new();

        for criterion in &rubric.criteria {
            let prompt = format!(
                "Score the following output on a scale of 0.0 to 1.0 for the criterion: {}\n\nCriterion description: {}\n\nOutput:\n{}",
                criterion.name, criterion.description, output
            );

            let response = self.llm.complete(CompletionRequest {
                messages: vec![Message {
                    role: Role::User,
                    content: MessageContent::Text(prompt),
                }],
                system_prompt: Some("You are a strict quality evaluator. Return only a JSON object with a 'score' field containing a float between 0.0 and 1.0.".into()),
                temperature: Some(0.0),
                max_tokens: Some(100),
                stop_sequences: None,
                tools: None,
            }).await?;

            let score: f32 = serde_json::from_str(&response.content)
                .map_err(|_| LoopError::InvalidScore { criterion: criterion.name.clone() })?;

            scores.push(ScoredCriterion {
                criterion: criterion.clone(),
                score,
            });
        }

        Ok(scores)
    }

    fn calculate_weighted_score(&self, scores: &[ScoredCriterion]) -> f32 {
        scores.iter()
            .map(|s| s.criterion.weight * s.score)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct LoopResult {
    pub output: String,
    pub score: f32,
    pub iterations: usize,
    pub scores: Vec<ScoredCriterion>,
}

#[derive(Debug, Clone)]
pub struct ScoredCriterion {
    pub criterion: Criterion,
    pub score: f32,
}
```

### Gap Analysis

```rust
impl LoopAgent {
    async fn analyze_gaps(&self, scores: &[ScoredCriterion]) -> Result<Vec<Gap>, LoopError> {
        let prompt = format!(
            "Analyze these quality scores and identify the main gaps:\n\n{}\n\nWhat specific improvements would increase the score the most?",
            scores.iter()
                .map(|s| format!("{}: {:.2} (weight: {:.2})", s.criterion.name, s.score, s.criterion.weight))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let response = self.llm.complete(CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text(prompt),
            }],
            system_prompt: Some("You are a quality improvement analyst. Identify specific, actionable gaps.".into()),
            temperature: Some(0.3),
            max_tokens: Some(500),
            stop_sequences: None,
            tools: None,
        }).await?;

        // Parse response into gaps
        todo!()
    }
}
```

### Termination Conditions

| Condition | Description |
|---|---|
| Score ≥ threshold | Quality standard met, stop |
| Iterations ≥ max_iterations | Budget exhausted, return best result |
| Score improvement < 0.01 for 3 iterations | Diminishing returns, stop |
| All criteria scored ≥ 0.9 | Excellent quality, stop early |

## Alternatives Considered

1. **External scoring (human or separate LLM)**: More accurate, but slower and more expensive.
2. **Rule-based scoring**: Deterministic, but less flexible than LLM-based scoring.
3. **Statistical scoring**: Uses metrics like test coverage, but limited to measurable criteria.

## Drawbacks

- LLM-based scoring adds cost and latency per iteration
- Scoring may be inconsistent across runs
- Gap analysis quality depends on LLM capability

## Unresolved Questions

- Should scoring use a separate, cheaper LLM model?
- How to handle criteria that are hard for LLMs to evaluate (e.g., code performance)?
- Should we support user-provided scoring functions?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-10-loop-agent-scoring.md
git commit -m "docs(rfc): add A-10 loop agent and scoring"
```

---

### Task 12: Write A-11 — Observability Contract RFC

**Covers:** P0-T11

**Files:**
- Create: `docs/rfcs/architecture/A-11-observability-contract.md`

- [ ] **Step 1: Write the RFC**

```markdown
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
span.set_attribute("argo.memory.store", store);  // redis, surreal, qdrant
span.set_attribute("argo.memory.operation", op);  // read, write
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

// Example log entry:
// {
//   "timestamp": "2026-06-27T19:00:00Z",
//   "level": "INFO",
//   "message": "Tool execution completed",
//   "run_id": "abc-123",
//   "agent_name": "coder",
//   "span_id": "def-456",
//   "fields": {
//     "tool": "bash",
//     "success": true,
//     "duration_ms": 150
//   }
// }
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
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-11-observability-contract.md
git commit -m "docs(rfc): add A-11 observability contract"
```

---

### Task 13: Write A-12 — Security Model RFC

**Covers:** P0-T12

**Files:**
- Create: `docs/rfcs/architecture/A-12-security-model.md`

- [ ] **Step 1: Write the RFC**

```markdown
# A-12: Security Model

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Summary

Define tool sandboxing implementation, secret handling, MCP authentication, agent isolation rules, and runtime permission enforcement.

## Motivation

Agents execute arbitrary code and access external services. The security model must prevent unauthorized access, protect secrets, and isolate agents from each other.

## Detailed Design

### Tool Sandboxing

```rust
pub struct SandboxConfig {
    pub working_directory: PathBuf,
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_domains: Vec<String>,
    pub allow_network: bool,
    pub allow_subprocess: bool,
    pub max_execution_time: Duration,
    pub max_memory: Option<usize>,  // bytes
}

pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn check_file_access(&self, path: &Path, operation: FileOperation) -> Result<(), SecurityError> {
        let canonical = path.canonicalize()
            .map_err(|_| SecurityError::PathResolutionFailed { path: path.to_path_buf() })?;

        // Check if path is within allowed paths
        if !self.config.allowed_paths.iter().any(|allowed| canonical.starts_with(allowed)) {
            return Err(SecurityError::PathDenied { path: path.to_path_buf() });
        }

        // Check operation-specific rules
        match operation {
            FileOperation::Read => Ok(()),
            FileOperation::Write => {
                if self.is_read_only(&canonical) {
                    Err(SecurityError::WriteDenied { path: path.to_path_buf() })
                } else {
                    Ok(())
                }
            }
            FileOperation::Delete => {
                if self.is_protected(&canonical) {
                    Err(SecurityError::DeleteDenied { path: path.to_path_buf() })
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn check_network_access(&self, url: &Url) -> Result<(), SecurityError> {
        if !self.config.allow_network {
            return Err(SecurityError::NetworkDenied { url: url.to_string() });
        }

        let host = url.host_str().ok_or_else(|| SecurityError::InvalidUrl { url: url.to_string() })?;

        if !self.config.allowed_domains.iter().any(|domain| host.ends_with(domain)) {
            return Err(SecurityError::DomainDenied { domain: host.to_string() });
        }

        Ok(())
    }

    pub fn check_execution_time(&self, elapsed: Duration) -> Result<(), SecurityError> {
        if elapsed > self.config.max_execution_time {
            Err(SecurityError::ExecutionTimeout { elapsed })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    #[error("Path access denied: {path}")]
    PathDenied { path: PathBuf },

    #[error("Write access denied: {path}")]
    WriteDenied { path: PathBuf },

    #[error("Delete access denied: {path}")]
    DeleteDenied { path: PathBuf },

    #[error("Network access denied: {url}")]
    NetworkDenied { url: String },

    #[error("Domain access denied: {domain}")]
    DomainDenied { domain: String },

    #[error("Execution timeout: {elapsed:?}")]
    ExecutionTimeout { elapsed: Duration },

    #[error("Path resolution failed: {path}")]
    PathResolutionFailed { path: PathBuf },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
}
```

### Secret Management

```rust
pub struct SecretManager {
    env_vars: HashMap<String, String>,
}

impl SecretManager {
    pub fn new() -> Self {
        Self {
            env_vars: std::env::vars().collect(),
        }
    }

    /// Resolve ${VAR_NAME} patterns in config strings
    pub fn resolve(&self, input: &str) -> Result<String, SecretError> {
        let re = regex::Regex::new(r"\$\{(\w+)\}").unwrap();

        let resolved = re.replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.env_vars.get(var_name)
                .map(|v| v.as_str())
                .unwrap_or("")
        }).to_string();

        Ok(resolved)
    }

    /// Validate that all required secrets are present
    pub fn validate(&self, required: &[&str]) -> Result<(), SecretError> {
        for var_name in required {
            if !self.env_vars.contains_key(*var_name) {
                return Err(SecretError::MissingSecret { name: var_name.to_string() });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Error)]
pub enum SecretError {
    #[error("Missing required secret: {name}")]
    MissingSecret { name: String },

    #[error("Secret resolution failed: {reason}")]
    ResolutionFailed { reason: String },
}
```

### MCP Authentication

```rust
pub struct McpAuth {
    config: AuthConfig,
}

impl McpAuth {
    pub fn apply_headers(&self, request: &mut reqwest::RequestBuilder) -> Result<(), SecurityError> {
        match &self.config.auth_type {
            AuthType::Bearer { token } => {
                let resolved = SecretManager::new().resolve(token)?;
                request.header("Authorization", format!("Bearer {}", resolved));
            }
            AuthType::OAuth2 { client_id, client_secret } => {
                // Token exchange flow
                let resolved_id = SecretManager::new().resolve(client_id)?;
                let resolved_secret = SecretManager::new().resolve(client_secret)?;
                // ... OAuth2 token exchange
            }
        }
        Ok(())
    }
}
```

### Agent Isolation

```rust
pub struct AgentIsolation {
    agents: HashMap<String, AgentNamespace>,
}

pub struct AgentNamespace {
    pub short_term_prefix: String,   // redis key prefix
    pub long_term_namespace: String, // surreal namespace
    pub semantic_namespace: String,  // qdrant collection prefix
}

impl AgentIsolation {
    pub fn create_namespace(&mut self, agent_id: &str, mode: &MemoryMode) -> AgentNamespace {
        match mode {
            MemoryMode::Isolated => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: format!("agent_{}", agent_id),
                semantic_namespace: format!("argo_{}_", agent_id),
            },
            MemoryMode::Shared => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: "shared".to_string(),
                semantic_namespace: "argo_shared_".to_string(),
            },
            MemoryMode::Persistent => AgentNamespace {
                short_term_prefix: format!("argo:agent:{}:", agent_id),
                long_term_namespace: format!("agent_{}", agent_id),
                semantic_namespace: format!("argo_{}_", agent_id),
            },
        }
    }

    pub fn can_access(&self, agent_id: &str, target: &str, mode: &MemoryMode) -> bool {
        match mode {
            MemoryMode::Isolated | MemoryMode::Persistent => agent_id == target,
            MemoryMode::Shared => true,
        }
    }
}
```

### Permission Enforcement

```rust
pub fn enforce_permissions(tool: &dyn Tool, ctx: &ToolContext, sandbox: &Sandbox) -> Result<(), SecurityError> {
    let perms = tool.permissions();

    // File system permissions
    if perms.allow_filesystem {
        if let Some(ref working_dir) = perms.working_directory {
            sandbox.check_file_access(working_dir, FileOperation::Read)?;
        }
    }

    // Network permissions
    if perms.allow_network {
        for domain in &perms.allowed_domains {
            // Validate domain format
        }
    }

    // Subprocess permissions
    if perms.allow_subprocess {
        // Verify subprocess is allowed in sandbox
    }

    // Execution time
    // (checked at runtime, not at permission check time)

    Ok(())
}
```

## Alternatives Considered

1. **OS-level sandboxing (containers, seccomp)**: Strongest isolation, but adds deployment complexity.
2. **WASM sandboxing**: Portable, but limited system access.
3. **No sandboxing**: Simplest, but unacceptable for production use.

## Drawbacks

- Path resolution adds overhead to file operations
- Secret management relies on environment variables (not vault integration)
- Agent isolation depends on correct namespace configuration

## Unresolved Questions

- Should we support vault integration (HashiCorp Vault, AWS Secrets Manager)?
- How to handle secrets in multi-tenant deployments?
- Should we support MAC-based mandatory access control?
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/architecture/A-12-security-model.md
git commit -m "docs(rfc): add A-12 security model"
```

---

## Task Group 3: Data Schema Documents (B-01 to B-06)

These tasks are independent and can be executed in parallel.

### Task 14: Write B-01 — SurrealDB Schema

**Covers:** P0-T13

**Files:**
- Create: `docs/rfcs/schemas/B-01-surrealdb-schema.md`

- [ ] **Step 1: Write the schema**

```markdown
# B-01: SurrealDB Schema

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Overview

Complete SurrealDB SCHEMAFULL definitions for all Argo tables.

## Schema

```sql
-- Use memory database
USE NS argo DB memory;

-- Task record: tracks completed agent tasks
DEFINE TABLE task SCHEMAFULL;
DEFINE FIELD agent_id    ON task TYPE string;
DEFINE FIELD goal        ON task TYPE string;
DEFINE FIELD outcome     ON task TYPE string ASSERT $value INSIDE ['success', 'partial', 'failed'];
DEFINE FIELD summary     ON task TYPE string;
DEFINE FIELD tools_used  ON task TYPE array;
DEFINE FIELD duration_ms ON task TYPE int;
DEFINE FIELD started_at  ON task TYPE datetime;
DEFINE FIELD ended_at    ON task TYPE datetime;
DEFINE FIELD run_id      ON task TYPE string;
DEFINE FIELD metadata    ON task TYPE option<object>;
DEFINE INDEX task_agent_idx ON task FIELDS agent_id;
DEFINE INDEX task_run_idx ON task FIELDS run_id;
DEFINE INDEX task_started_idx ON task FIELDS started_at;

-- Entity record: anything the agent worked with
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD type        ON entity TYPE string ASSERT $value INSIDE ['file', 'api', 'repo', 'person', 'project', 'tool', 'other'];
DEFINE FIELD identifier  ON entity TYPE string;
DEFINE FIELD metadata    ON entity TYPE option<object>;
DEFINE FIELD created_at  ON entity TYPE datetime;
DEFINE FIELD updated_at  ON entity TYPE datetime;
DEFINE INDEX entity_type_idx ON entity FIELDS type;
DEFINE INDEX entity_identifier_idx ON entity FIELDS identifier;

-- Graph relationships
DEFINE TABLE interacted_with SCHEMAFULL;
DEFINE FIELD from_task    ON interacted_with TYPE string;
DEFINE FIELD to_entity    ON interacted_with TYPE string;
DEFINE FIELD relation     ON interacted_with TYPE string;
DEFINE FIELD metadata     ON interacted_with TYPE option<object>;
DEFINE FIELD created_at   ON interacted_with TYPE datetime;

DEFINE TABLE depends_on SCHEMAFULL;
DEFINE FIELD from_entity  ON depends_on TYPE string;
DEFINE FIELD to_entity    ON depends_on TYPE string;
DEFINE FIELD relation     ON depends_on TYPE string;
DEFINE FIELD metadata     ON depends_on TYPE option<object>;

-- Error record: tracks errors encountered during execution
DEFINE TABLE error_record SCHEMAFULL;
DEFINE FIELD task_id      ON error_record TYPE string;
DEFINE FIELD run_id       ON error_record TYPE string;
DEFINE FIELD error_type   ON error_record TYPE string;
DEFINE FIELD message      ON error_record TYPE string;
DEFINE FIELD context      ON error_record TYPE option<object>;
DEFINE FIELD resolution   ON error_record TYPE option<string>;
DEFINE FIELD strategy     ON error_record TYPE option<string>;
DEFINE FIELD occurred_at  ON error_record TYPE datetime;
DEFINE FIELD resolved_at  ON error_record TYPE option<datetime>;
DEFINE INDEX error_task_idx ON error_record FIELDS task_id;
DEFINE INDEX error_type_idx ON error_record FIELDS error_type;

-- Lesson record: structured knowledge from post-mortems
DEFINE TABLE lesson SCHEMAFULL;
DEFINE FIELD error_type      ON lesson TYPE string;
DEFINE FIELD context_summary ON lesson TYPE string;
DEFINE FIELD root_cause      ON lesson TYPE string;
DEFINE FIELD resolution      ON lesson TYPE string;
DEFINE FIELD prevention      ON lesson TYPE string;
DEFINE FIELD confidence      ON lesson TYPE float ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD source_error    ON lesson TYPE string;
DEFINE FIELD created_at      ON lesson TYPE datetime;
DEFINE INDEX lesson_error_idx ON lesson FIELDS error_type;

-- Agent record: agent configuration and metadata
DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD name        ON agent TYPE string;
DEFINE FIELD model       ON agent TYPE string;
DEFINE FIELD config      ON agent TYPE object;
DEFINE FIELD created_at  ON agent TYPE datetime;
DEFINE FIELD updated_at  ON agent TYPE datetime;
DEFINE INDEX agent_name_idx ON agent FIELDS name;

-- Growth report: records from daily growth cycles
DEFINE TABLE growth_report SCHEMAFULL;
DEFINE FIELD agent_id          ON growth_report TYPE string;
DEFINE FIELD timestamp         ON growth_report TYPE datetime;
DEFINE FIELD errors_analyzed   ON growth_report TYPE int;
DEFINE FIELD patterns_detected ON growth_report TYPE int;
DEFINE FIELD proposals_generated ON growth_report TYPE int;
DEFINE FIELD proposals_applied ON growth_report TYPE int;
DEFINE FIELD patterns          ON growth_report TYPE array;
DEFINE FIELD proposals         ON growth_report TYPE array;
DEFINE INDEX growth_agent_idx ON growth_report FIELDS agent_id;

-- Archived turns: for context window overflow handling
DEFINE TABLE archived_turns SCHEMAFULL;
DEFINE FIELD run_id      ON archived_turns TYPE string;
DEFINE FIELD agent_id    ON archived_turns TYPE string;
DEFINE FIELD turns       ON archived_turns TYPE array;
DEFINE FIELD archived_at ON archived_turns TYPE datetime;
DEFINE INDEX archived_run_idx ON archived_turns FIELDS run_id;
```
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-01-surrealdb-schema.md
git commit -m "docs(schema): add B-01 SurrealDB schema"
```

---

### Task 15: Write B-02 — Redis Key Schema

**Covers:** P0-T14

**Files:**
- Create: `docs/rfcs/schemas/B-02-redis-key-schema.md`

- [ ] **Step 1: Write the schema**

```markdown
# B-02: Redis Key Schema

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Overview

All Redis key patterns, data types, and TTL policies for Argo's short-term memory.

## Key Patterns

| Key Pattern | Data Type | TTL | Description |
|---|---|---|---|
| `argo:agent:{agent_id}:run:{run_id}:context` | String | Task duration + 1h | Full context blob (JSON) |
| `argo:agent:{agent_id}:run:{run_id}:turns` | List | Task duration + 1h | LLM conversation turns |
| `argo:agent:{agent_id}:run:{run_id}:scratch` | String | Task duration + 1h | Agent scratchpad |
| `argo:agent:{agent_id}:run:{run_id}:plan` | String | Task duration + 1h | Current plan JSON |
| `argo:agent:{agent_id}:run:{run_id}:tools` | Hash | Task duration + 1h | Tool call history |
| `argo:agent:{agent_id}:active_runs` | Set | No expiry | Set of active run IDs |
| `argo:agent:{agent_id}:config` | String | No expiry | Cached agent config |

## Key Format

```
argo:agent:{agent_id}:run:{run_id}:{suffix}
```

- `agent_id`: UUID of the agent
- `run_id`: UUID of the current run
- `suffix`: One of `context`, `turns`, `scratch`, `plan`, `tools`

## Data Formats

### Context (`:context`)

```json
{
  "goal": "Build a REST API",
  "system_prompt": "You are a coding agent...",
  "created_at": "2026-06-27T19:00:00Z",
  "token_count": 1234
}
```

### Turns (`:turns`)

List of messages (LPUSH to add, LRANGE to read):

```json
[
  {
    "role": "user",
    "content": "Build a REST API for a blog"
  },
  {
    "role": "assistant",
    "content": "I'll plan the implementation...",
    "tool_calls": [...]
  },
  {
    "role": "tool",
    "content": "{\"success\": true, \"output\": ...}"
  }
]
```

### Scratch (`:scratch`)

Free-form text for agent's intermediate reasoning:

```
Current plan:
1. Create project structure
2. Implement models
3. Add routes
4. Write tests

Progress: Step 2 complete, working on step 3
```

### Plan (`:plan`)

```json
{
  "steps": [
    {"id": 1, "description": "Create project structure", "status": "completed"},
    {"id": 2, "description": "Implement models", "status": "completed"},
    {"id": 3, "description": "Add routes", "status": "in_progress"},
    {"id": 4, "description": "Write tests", "status": "pending"}
  ],
  "current_step": 3,
  "started_at": "2026-06-27T19:00:00Z"
}
```

### Tool History (`:tools`)

Hash with tool call ID as field:

```
HSET argo:agent:abc:run:def:tools call-1 '{"tool":"bash","command":"ls","success":true}'
HSET argo:agent:abc:run:def:tools call-2 '{"tool":"files","action":"write","success":true}'
```

## TTL Calculation

```
TTL = task_duration + 3600 seconds
```

Where `task_duration` is the time from task start to task end. For tasks that haven't completed yet, TTL is set to `max_expected_duration + 3600`.

## Cleanup

When a task completes:

1. Set final TTL on all keys for this run
2. Remove run_id from `argo:agent:{agent_id}:active_runs`

When TTL expires, Redis automatically removes the keys.

## Operations

```rust
pub struct RedisKeySchema;

impl RedisKeySchema {
    pub fn context(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:context", agent_id, run_id)
    }

    pub fn turns(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:turns", agent_id, run_id)
    }

    pub fn scratch(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:scratch", agent_id, run_id)
    }

    pub fn plan(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:plan", agent_id, run_id)
    }

    pub fn tools(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:tools", agent_id, run_id)
    }

    pub fn active_runs(agent_id: &str) -> String {
        format!("argo:agent:{}:active_runs", agent_id)
    }
}
```
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-02-redis-key-schema.md
git commit -m "docs(schema): add B-02 Redis key schema"
```

---

### Task 16: Write B-03 — Qdrant Collection Schema

**Covers:** P0-T15

**Files:**
- Create: `docs/rfcs/schemas/B-03-qdrant-collection-schema.md`

- [ ] **Step 1: Write the schema**

```markdown
# B-03: Qdrant Collection Schema

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Overview

Qdrant collection definitions for Argo's semantic memory layer.

## Collections

### argo_experiences

Stores embeddings of past task summaries for experience retrieval.

```json
{
  "vectors": {
    "size": 1536,
    "distance": "Cosine"
  },
  "payload_schema": {
    "task_summary": { "type": "text" },
    "outcome": { "type": "keyword" },
    "tools_used": { "type": "keyword", "array": true },
    "duration_ms": { "type": "integer" },
    "agent_id": { "type": "keyword" },
    "run_id": { "type": "keyword" },
    "created_at": { "type": "keyword" }
  }
}
```

**Use case:** Before executing a task, embed the task description and query this collection for similar past experiences.

### argo_errors

Stores embeddings of error + resolution pairs for heal strategy selection.

```json
{
  "vectors": {
    "size": 1536,
    "distance": "Cosine"
  },
  "payload_schema": {
    "error_type": { "type": "keyword" },
    "context_summary": { "type": "text" },
    "resolution": { "type": "text" },
    "strategy": { "type": "keyword" },
    "confidence": { "type": "float" },
    "agent_id": { "type": "keyword" },
    "created_at": { "type": "keyword" }
  }
}
```

**Use case:** When an error occurs, embed the error context and query for similar past errors to find what strategy worked.

### argo_lessons

Stores embeddings of structured post-mortem lessons.

```json
{
  "vectors": {
    "size": 1536,
    "distance": "Cosine"
  },
  "payload_schema": {
    "error_type": { "type": "keyword" },
    "root_cause": { "type": "text" },
    "prevention": { "type": "text" },
    "confidence": { "type": "float" },
    "agent_id": { "type": "keyword" },
    "created_at": { "type": "keyword" }
  }
}
```

**Use case:** Before executing a potentially risky operation, check if a lesson exists for similar situations.

### argo_tool_patterns

Stores embeddings of successful tool usage patterns.

```json
{
  "vectors": {
    "size": 1536,
    "distance": "Cosine"
  },
  "payload_schema": {
    "tool_name": { "type": "keyword" },
    "task_type": { "type": "keyword" },
    "success_rate": { "type": "float" },
    "avg_duration_ms": { "type": "integer" },
    "agent_id": { "type": "keyword" },
    "created_at": { "type": "keyword" }
  }
}
```

**Use case:** When selecting between tools for a task, query for successful patterns with similar task types.

## Vector Dimensions

All collections use 1536-dimensional vectors, matching OpenAI's `text-embedding-3-small` model. If using a different embedding model, the vector dimension must be updated.

## Index Configuration

```json
{
  "optimizer_config": {
    "deleted_threshold": 0.2,
  "indexed_vectors_threshold": 20000
  },
  "search_optimized_config": {
    "search_optimization": {
      "disable_on_disk": false
    }
  }
}
```

## Payload Indexing

For efficient filtering, create payload indexes:

```rust
// Create index for keyword fields
client.create_payload_index("argo_experiences", "outcome", KeywordIndexParams {
    r#type: KeywordIndexType::Keyword,
}).await?;

client.create_payload_index("argo_experiences", "agent_id", KeywordIndexParams {
    r#type: KeywordIndexType::Keyword,
}).await?;
```

## Operations

```rust
pub struct QdrantCollectionSchema;

impl QdrantCollectionSchema {
    pub const EXPERIENCES: &'static str = "argo_experiences";
    pub const ERRORS: &'static str = "argo_errors";
    pub const LESSONS: &'static str = "argo_lessons";
    pub const TOOL_PATTERNS: &'static str = "argo_tool_patterns";

    pub const VECTOR_DIMENSION: usize = 1536;
}
```
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-03-qdrant-collection-schema.md
git commit -m "docs(schema): add B-03 Qdrant collection schema"
```

---

### Task 17: Write B-04 — Agent Config TOML Schema

**Covers:** P0-T16

**Files:**
- Create: `docs/rfcs/schemas/B-04-agent-config-toml-schema.md`

- [ ] **Step 1: Write the schema**

```markdown
# B-04: Agent Config TOML Schema

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Overview

Complete TOML schema for Argo agent configuration files.

## Schema

```toml
# my-agent.toml

[agent]
name = "coder"                              # Required: string
version = "1.0.0"                           # Required: semver
description = "Production coding agent"     # Optional: string

[model]
provider = "anthropic"                      # Required: "anthropic" | "openai" | "gemini" | "ollama" | "custom"
model = "claude-sonnet-4-6"                 # Required: string
api_key = "${ANTHROPIC_API_KEY}"            # Required: env var reference
temperature = 0.2                           # Optional: float, 0.0-2.0, default 0.7
max_tokens = 8192                           # Optional: int, default 4096
context_strategy = "summarize"              # Optional: "summarize" | "sliding_window" | "truncate", default "summarize"

# Optional: fallback providers (used by heal engine)
[[model.fallbacks]]
provider = "openai"
model = "gpt-4o"
api_key = "${OPENAI_API_KEY}"

[memory]
mode = "persistent"                         # Required: "persistent" | "ephemeral" | "shared"
short_term_ttl = 3600                       # Optional: int (seconds), default 3600
long_term_backend = "surrealdb"             # Optional: "surrealdb", default "surrealdb"
vector_backend = "qdrant"                   # Optional: "qdrant", default "qdrant"
embedding_model = "text-embedding-3-small"  # Optional: string, default "text-embedding-3-small"

# Optional: memory backend URLs
[memory.redis]
url = "redis://localhost:6379"              # Optional: string, default "redis://localhost:6379"

[memory.surrealdb]
url = "ws://localhost:8000"                 # Optional: string, default "ws://localhost:8000"
namespace = "argo"                          # Optional: string, default "argo"
database = "memory"                         # Optional: string, default "memory"
username = "root"                           # Optional: string, default "root"
password = "${SURREALDB_PASSWORD}"          # Optional: env var reference

[memory.qdrant]
url = "http://localhost:6333"               # Optional: string, default "http://localhost:6333"
api_key = "${QDRANT_API_KEY}"              # Optional: env var reference

[heal]
enabled = true                              # Required: bool
max_attempts = 7                            # Optional: int, default 7
strategies = [                              # Optional: list of strategy names
    "retry",
    "reframe",
    "swap_tool",
    "decompose",
    "spawn_subagent",
    "change_provider",
    "reduce_scope",
]
background = true                           # Optional: bool, default true

# Only for LoopAgent
[quality]
threshold = 0.85                            # Required for LoopAgent: float, 0.0-1.0
max_iterations = 20                         # Required for LoopAgent: int

[[quality.criteria]]
name = "tests_pass"                         # Required: string
weight = 0.40                               # Required: float, 0.0-1.0
description = "All unit tests pass"         # Required: string

[[quality.criteria]]
name = "code_quality"
weight = 0.30
description = "Code follows best practices"

[tools]
enabled = ["bash", "git", "files", "web_search"]  # Required: list of tool names

# Optional: MCP server connections
[[tools.mcp]]
url = "https://mcp.asana.com/sse"          # Required: string
[tools.mcp.auth]
type = "bearer"                             # Required: "bearer" | "oauth2"
token = "${ASANA_TOKEN}"                    # Required for bearer: env var reference

[[tools.mcp]]
url = "https://mcp.linear.app/mcp"
[tools.mcp.auth]
type = "oauth2"
client_id = "${LINEAR_CLIENT_ID}"
client_secret = "${LINEAR_CLIENT_SECRET}"

# Optional: tool-specific configuration
[tools.bash]
working_directory = "./sandbox"             # Optional: string
max_execution_time = 30                     # Optional: int (seconds), default 30

[tools.http]
allowed_domains = ["api.github.com", "pypi.org"]  # Optional: list of domains

[permissions]
allow_network = true                        # Required: bool
allow_filesystem = true                     # Required: bool
allowed_paths = ["./workspace", "/tmp"]     # Optional: list of paths
max_execution_time = 300                    # Optional: int (seconds), default 300

[observe]
enabled = false                             # Required: bool
backend = "otlp"                            # Optional: "otlp" | "stdout" | "none", default "none"
endpoint = "http://localhost:4317"          # Optional: string
```

## Validation Rules

| Field | Type | Required | Default | Validation |
|---|---|---|---|---|
| `agent.name` | string | Yes | - | Non-empty, alphanumeric + hyphens |
| `agent.version` | string | Yes | - | Valid semver |
| `model.provider` | string | Yes | - | One of: anthropic, openai, gemini, ollama, custom |
| `model.model` | string | Yes | - | Non-empty |
| `model.api_key` | string | Yes | - | Starts with `$` |
| `model.temperature` | float | No | 0.7 | 0.0 - 2.0 |
| `model.max_tokens` | int | No | 4096 | > 0 |
| `memory.mode` | string | Yes | - | One of: persistent, ephemeral, shared |
| `heal.enabled` | bool | Yes | - | - |
| `heal.max_attempts` | int | No | 7 | > 0 |
| `quality.threshold` | float | Conditional | - | 0.0 - 1.0 (required for LoopAgent) |
| `quality.max_iterations` | int | Conditional | - | > 0 (required for LoopAgent) |
| `tools.enabled` | list | Yes | - | Non-empty |
| `permissions.allow_network` | bool | Yes | - | - |
| `permissions.allow_filesystem` | bool | Yes | - | - |

## Environment Variable Substitution

Any string value can contain `${VAR_NAME}` patterns. At startup, Argo resolves these from environment variables:

```
api_key = "${ANTHROPIC_API_KEY}"  →  api_key = "sk-ant-..."
```

If the environment variable is not set, Argo fails at startup with a clear error message.
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-04-agent-config-toml-schema.md
git commit -m "docs(schema): add B-04 agent config TOML schema"
```

---

### Task 18: Write B-05 — MessagePack Message Catalog

**Covers:** P0-T17

**Files:**
- Create: `docs/rfcs/schemas/B-05-messagepack-message-catalog.md`

- [ ] **Step 1: Write the schema**

```markdown
# B-05: MessagePack Message Catalog

**Status:** Proposed  
**Author:** Argo Core Team  
**Created:** 2026-06-27

---

## Overview

Every message type serialized over the actor bus, with field definitions and versioning.

## Message Format

All messages are serialized with MessagePack (rmp-serde). Each message includes a version header for forward compatibility.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub version: u32,      // Message format version
    pub message_type: String,
    pub payload: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<uuid::Uuid>,
}
```

## Message Types

### ExecuteTask

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTask {
    pub task_id: uuid::Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_mode: MemoryMode,
}

// MessagePack layout:
// {
//   "task_id": "550e8400-e29b-41d4-a716-446655440000",
//   "goal": "Build a REST API",
//   "context": null,
//   "deadline": null,
//   "memory_mode": "persistent"
// }
```

### ToolCall

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: uuid::Uuid,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub timeout_ms: Option<u64>,
}
```

### ToolResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: uuid::Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

### MemoryRead / MemoryWrite

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRead {
    pub op_id: uuid::Uuid,
    pub store: MemoryStore,
    pub key: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWrite {
    pub op_id: uuid::Uuid,
    pub store: MemoryStore,
    pub key: String,
    pub value: serde_json::Value,
    pub namespace: Option<String>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStore {
    Redis,
    SurrealDB,
    Qdrant,
}
```

### HealRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealRequest {
    pub error: AgentError,
    pub context: HealContext,
    pub max_attempts: Option<usize>,
}
```

### SpawnAgent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgent {
    pub agent_id: uuid::Uuid,
    pub config: AgentConfig,
    pub goal: String,
    pub parent_id: Option<uuid::Uuid>,
    pub memory_mode: MemoryMode,
}
```

### AgentDone / AgentFailed

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDone {
    pub task_id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub run_id: uuid::Uuid,
    pub result: TaskResult,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFailed {
    pub task_id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub run_id: uuid::Uuid,
    pub error: AgentError,
    pub trace: AgentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success { output: String },
    Partial { output: String, reason: String },
    Failed { error: AgentError },
}
```

### InspectRequest / InspectResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectRequest {
    pub run_id: uuid::Uuid,
    pub include_trace: bool,
    pub include_heal_steps: bool,
    pub include_lessons: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResponse {
    pub trace: Option<AgentTrace>,
    pub heal_steps: Option<Vec<HealStepRecord>>,
    pub lessons: Option<Vec<LessonRecord>>,
}
```

### AssignTask / TaskComplete / TaskFailed

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTask {
    pub task_id: uuid::Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub memory_mode: MemoryMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplete {
    pub task_id: uuid::Uuid,
    pub result: TaskResult,
    pub duration_ms: u64,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailed {
    pub task_id: uuid::Uuid,
    pub error: AgentError,
    pub partial_result: Option<String>,
}
```

## Versioning

Message format version is tracked in the `MessageEnvelope.version` field. When the message format changes:

1. Increment the version number
2. Add backward compatibility for the previous version
3. Deprecate the old version after 2 releases

Current version: `1`
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-05-messagepack-message-catalog.md
git commit -m "docs(schema): add B-05 MessagePack message catalog"
```

---

### Task 19: Write B-06 — OTel Semantic Conventions

**Covers:** P0-T18

**Files:**
- Create: `docs/rfcs/schemas/B-06-otel-semantic-conventions.md`

- [ ] **Step 1: Write the schema**

```markdown
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
```

- [ ] **Step 2: Commit**

```bash
git add docs/rfcs/schemas/B-06-otel-semantic-conventions.md
git commit -m "docs(schema): add B-06 OTel semantic conventions"
```

---

## Task Group 4: Contributor Documentation

### Task 20: Write CONTRIBUTING.md

**Covers:** P0-T21

**Files:**
- Create: `CONTRIBUTING.md`

- [ ] **Step 1: Write CONTRIBUTING.md**

```markdown
# Contributing to Argo

Thank you for your interest in contributing to Argo! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive. We are building something important together.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Open a new issue using the Bug Report template
3. Include: description, reproduction steps, environment, expected vs actual behavior

### Suggesting Features

1. Check existing issues and RFCs
2. Open a new issue using the Feature Request template
3. For major features, consider writing an RFC first

### Submitting Code

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes following the coding standards
4. Add or update tests
5. Ensure all tests pass
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75+ (stable)
- Docker and Docker Compose
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/argo-agents/argo.git
cd argo

# Start development services (Redis, SurrealDB, Qdrant)
docker compose up -d

# Build the project
cargo build

# Run tests
cargo test

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

## Coding Standards

### Rust

- Follow `rustfmt` formatting (run `cargo fmt`)
- No `clippy` warnings (run `cargo clippy`)
- Use `thiserror` for error types, `anyhow` for error propagation
- Use `async-trait` for async trait definitions
- Prefer `serde` for serialization
- Use `uuid::Uuid` for identifiers
- Use `chrono` for timestamps

### Documentation

- All public APIs must have doc comments
- RFCs must follow the template in `docs/rfcs/`
- Use clear, concise language
- Include examples where helpful

### Testing

- Unit tests in the same file or `tests/` module
- Integration tests in `tests/` directory
- Use `tokio-test` for async tests
- Mock external services (LLM providers, databases) in unit tests

## Commit Messages

Follow Conventional Commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code restructuring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(core): add agent actor implementation
fix(memory): handle Redis connection timeout
docs(rfc): add A-01 actor engine design
```

## Pull Request Process

1. Fill out the PR template completely
2. Link related issues
3. Ensure CI passes
4. Request review from relevant CODEOWNERS
5. Address review feedback
6. Squash and merge when approved

## RFC Process

For major changes:

1. Copy the RFC template
2. Fill out all sections
3. Submit as a draft PR
4. Discuss with the team
5. Iterate until approved
6. Merge the RFC
7. Implement the approved design

## Questions?

Open a discussion in GitHub Discussions or ask in Discord.
```

- [ ] **Step 2: Commit**

```bash
git add CONTRIBUTING.md
git commit -m "docs: add CONTRIBUTING.md"
```

---

### Task 21: Write Decision Log and Additional Docs

**Covers:** P0-T23, ADD-06, ADD-07, ADD-08

**Files:**
- Create: `docs/rfcs/guides/F-05-decision-log.md`
- Create: `CHANGELOG.md`
- Create: `docs/ROADMAP.md`
- Create: `docs/rfcs/guides/RFC-REVIEW-PROCESS.md`

- [ ] **Step 1: Write Decision Log**

```markdown
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
```

- [ ] **Step 2: Write CHANGELOG.md**

```markdown
# Changelog

All notable changes to Argo will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project initialization with workspace structure
- 12 Architecture RFCs (A-01 through A-12)
- 6 Data Schema documents (B-01 through B-06)
- GitHub Actions CI/CD pipeline
- Contributing guidelines
- Decision log

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2026-06-27

### Added
- Initial project structure
- Master plan and implementation plan documentation
```

- [ ] **Step 3: Write ROADMAP.md**

```markdown
# Argo Roadmap

## Current Phase: Phase 0 — Planning

**Duration:** 4–6 weeks  
**Status:** In Progress

### Milestones

- [x] Master plan and specification
- [x] Implementation plan
- [ ] 12 Architecture RFCs
- [ ] 6 Data Schema documents
- [ ] GitHub repository setup
- [ ] CI/CD pipeline
- [ ] Contributing guidelines
- [ ] Decision log

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to get involved.

## Discussion

Join the conversation in [GitHub Discussions](https://github.com/argo-agents/argo/discussions) or [Discord](https://discord.gg/argo).
```

- [ ] **Step 4: Write RFC Review Process**

```markdown
# RFC Review Process

## How RFCs Work

1. **Proposal**: Author creates a draft PR with the RFC
2. **Discussion**: Team reviews, asks questions, suggests changes
3. **Revision**: Author addresses feedback
4. **Approval**: At least 2 core team members approve
5. **Merge**: RFC is merged to `main`
6. **Implementation**: Development follows the approved design

## RFC Template

```markdown
# A-XX: Title

**Status:** Proposed | Accepted | Rejected | Superseded
**Author:** Name
**Created:** YYYY-MM-DD
**Supersedes:** A-XX (if applicable)

---

## Summary
One paragraph description.

## Motivation
Why this is needed.

## Detailed Design
The technical design.

## Alternatives Considered
Other approaches and why they were rejected.

## Drawbacks
Known limitations.

## Unresolved Questions
Open questions for discussion.
```

## Review Criteria

1. **Completeness**: All sections filled out
2. **Consistency**: Aligns with other RFCs and master plan
3. **Feasibility**: Can be implemented in the target phase
4. **Trade-offs**: Alternatives considered, drawbacks acknowledged
5. **Clarity**: Design is unambiguous

## Approval Requirements

- 2 approvals from core team
- No unresolved objections
- All questions answered or deferred with rationale
```

- [ ] **Step 5: Commit**

```bash
git add docs/rfcs/guides/F-05-decision-log.md CHANGELOG.md docs/ROADMAP.md docs/rfcs/guides/RFC-REVIEW-PROCESS.md
git commit -m "docs: add decision log, changelog, roadmap, and RFC review process"
```

---

## Task Group 5: Verification

### Task 22: Verify all Phase 0 deliverables

**Covers:** P0-TEST-01, P0-TEST-02, P0-TEST-03

- [ ] **Step 1: Verify workspace builds**

Run: `cargo check --workspace`
Expected: Compiles successfully

- [ ] **Step 2: Verify directory structure**

Check that all expected files exist:
- `docs/rfcs/architecture/A-01` through `A-12`
- `docs/rfcs/schemas/B-01` through `B-06`
- `docs/rfcs/guides/F-05-decision-log.md`
- `docs/rfcs/guides/RFC-REVIEW-PROCESS.md`
- `CONTRIBUTING.md`
- `CHANGELOG.md`
- `docs/ROADMAP.md`
- `.github/workflows/ci.yml`
- `.github/ISSUE_TEMPLATE/`
- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/CODEOWNERS`
- `Cargo.toml`
- `.gitignore`
- `README.md`
- `LICENSE`

- [ ] **Step 3: Verify RFC cross-references**

Check that all RFCs reference the master plan and use consistent terminology.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: complete Phase 0 planning deliverables"
```

---

## Summary

| Task Group | Tasks | Deliverables |
|---|---|---|
| 1: Scaffolding & GitHub | 1 | Directory structure, Cargo workspace, CI/CD, templates |
| 2: Architecture RFCs | 12 | A-01 through A-12 |
| 3: Data Schemas | 6 | B-01 through B-06 |
| 4: Contributor Docs | 2 | CONTRIBUTING.md, Decision Log, CHANGELOG, ROADMAP, RFC Process |
| 5: Verification | 1 | Build check, structure verification |

**Total:** 22 tasks, 38+ files created
