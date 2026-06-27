# Argo Agent Framework — Master Plan & Complete Specification

**Version:** 0.1.0-planning  
**Status:** Pre-development  
**License:** Apache 2.0  
**Repository:** github.com/argo-agents/argo  

---

## Table of Contents

1. [Vision & Mission](#1-vision--mission)
2. [What We Are Building](#2-what-we-are-building)
3. [End Product Description](#3-end-product-description)
4. [Architecture Overview](#4-architecture-overview)
5. [Core Engine — Rust](#5-core-engine--rust)
6. [Memory System](#6-memory-system)
7. [Self-Healing System](#7-self-healing-system)
8. [Self-Improvement & Evolution](#8-self-improvement--evolution)
9. [Multi-Agent System](#9-multi-agent-system)
10. [Tool & MCP System](#10-tool--mcp-system)
11. [Language SDKs](#11-language-sdks)
12. [Agent Configuration](#12-agent-configuration)
13. [CLI](#13-cli)
14. [Observability](#14-observability)
15. [Security Model](#15-security-model)
16. [Distribution & Packaging](#16-distribution--packaging)
17. [Tech Stack — Complete](#17-tech-stack--complete)
18. [Development Phases](#18-development-phases)
19. [Required Documentation List](#19-required-documentation-list)

---

## 1. Vision & Mission

### Vision

Argo is the first agent framework where agents genuinely get better over time. Every error makes them smarter. Every task builds experience. Every day they improve — without any human intervention.

### Mission

Make it trivially easy for any developer — regardless of their language preference — to build production-grade autonomous agents that are self-healing, self-improving, and capable of running indefinitely without human supervision.

### Core Principles

- **Rust at the center.** The core runtime is Rust: zero garbage collection, no memory leaks, blazing performance, fearless concurrency. Every language SDK calls into the same Rust core.
- **Invisible reliability.** Self-healing and recovery run silently in the background. The developer sees a result, not a stack trace.
- **Memory is a first-class citizen.** Agents do not forget. Every experience, every lesson, every error resolution is stored and retrieved intelligently.
- **Write once, run in any language.** The full feature set is available identically in Python, TypeScript, and Rust. No second-class SDKs.
- **Open source, forever.** Apache 2.0. No premium tiers. No feature locks. Community-owned.

---

## 2. What We Are Building

Argo is a **multi-language agent framework** with a Rust core. It enables developers to build:

- Single autonomous agents that run tasks without human supervision
- Multi-agent systems where specialized agents collaborate
- Loop agents that plan, execute, review, and iterate until a task is done to their own standard
- Long-running agents with persistent memory that improve with every run

The framework provides all the infrastructure an agent needs: a runtime, memory, tools, MCP connectivity, error handling, self-healing, self-improvement, and observability. The developer only writes the agent's goal, tools, and configuration.

### What Makes Argo Different from Every Existing Framework

| Capability | LangChain | CrewAI | AutoGen | **Argo** |
|---|---|---|---|---|
| Core language | Python | Python | Python | **Rust** |
| Self-healing | Manual | None | Partial | **Automatic, multi-strategy** |
| Self-improvement | None | None | None | **Built-in, persistent** |
| Memory layers | 1 | 1 | 1 | **3 (short / long / semantic)** |
| MCP native | Partial | No | No | **Full protocol** |
| Multi-language SDKs | Python only | Python only | Python only | **Python + TS + Rust** |
| Memory leaks | Possible | Possible | Possible | **Impossible (Rust)** |
| Loop agent (no human) | Partial | Partial | Partial | **Full, self-scoring** |

---

## 3. End Product Description

When Argo reaches v1.0, a developer should be able to:

### 3.1 Install in 30 seconds

```bash
# Rust
cargo add argo

# Python
pip install argo-agents

# TypeScript
npm install @argo-ai/sdk
```

### 3.2 Build a production coding agent in 10 lines

```python
from argo import Agent, tools

agent = Agent(
    name="coder",
    model="claude-sonnet-4-6",
    memory="persistent",
    heal=True,
    tools=[tools.bash, tools.git, tools.files]
)

agent.run("Build a REST API for a blog with full CRUD, tests, and a README")
```

The agent will:
- Plan the work
- Write the code
- Run the tests
- Fix failures automatically (self-heal)
- Iterate until tests pass
- Write the README
- Remember this project for future runs

If a tool fails, the agent tries an alternative. If the LLM produces bad code, it catches the error, reflects on it, tries a different approach. The developer never sees any of this — they see the finished API.

### 3.3 Build a multi-agent pipeline

```python
from argo import AgentPool

pool = AgentPool()
pool.add("researcher", tools=[tools.web, tools.browser])
pool.add("writer",     tools=[tools.files])
pool.add("reviewer",   tools=[tools.files])

pool.run("Research the top 5 AI frameworks of 2025, write a 2000-word report, review it for quality")
```

### 3.4 Build a fully autonomous loop agent

```python
from argo import LoopAgent

agent = LoopAgent(
    name="analyst",
    goal="Analyze Q3 sales data and produce an executive summary",
    quality_threshold=0.85,   # Agent scores its own output, loops until 85% score
    max_iterations=20,
    tools=[tools.files, tools.code]
)

agent.run()  # Runs until done, no human needed
```

### 3.5 Inspect the heal trace if desired

```python
result = agent.run("complex task")
trace = agent.inspect()

# trace.errors         — list of errors that occurred
# trace.heal_steps     — how each was resolved
# trace.memory_ops     — what was read/written
# trace.lessons        — new lessons learned this run
# trace.iterations     — how many times the loop ran
```

---

## 4. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Developer code                        │
│              Python SDK │ TypeScript SDK │ Rust SDK          │
└──────────────┬──────────┴──────┬─────────┴───────┬──────────┘
               │ PyO3            │ napi-rs          │ native
               ▼                 ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    Argo Core (Rust + Tokio)                   │
│                                                              │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────────┐ │
│  │ Actor Engine │  │ Memory Engine │  │   Heal Engine     │ │
│  │ (Actix)      │  │ Redis+SurrDB  │  │ Strategies + Loop │ │
│  └──────────────┘  └───────────────┘  └───────────────────┘ │
│                                                              │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────────┐ │
│  │ Tool Registry│  │ MCP Connector │  │  Observability    │ │
│  │ Hot reload   │  │ Full protocol │  │  (OpenTelemetry)  │ │
│  └──────────────┘  └───────────────┘  └───────────────────┘ │
└─────────────────────────────────────────────────────────────┘
               │                  │
               ▼                  ▼
┌──────────────────────┐  ┌──────────────────────────────────┐
│   Storage Layer      │  │        LLM Providers             │
│  Redis (hot)         │  │  Anthropic │ OpenAI │ Gemini     │
│  SurrealDB (long)    │  │  Ollama    │ any OpenAI-compat   │
│  Qdrant (vectors)    │  └──────────────────────────────────┘
└──────────────────────┘
```

---

## 5. Core Engine — Rust

### 5.1 Runtime: Tokio

The entire core is built on **Tokio**, Rust's async runtime. This gives:
- Non-blocking I/O across all agent operations
- Thousands of agents running concurrently on a single machine
- Structured concurrency — no orphaned tasks

### 5.2 Actor Model: Actix

Each agent is an **Actix actor**. An actor is an isolated unit with:
- Its own private state
- A mailbox (message queue)
- Message handlers that process one message at a time

This means:
- One agent crashing cannot corrupt another
- Agents communicate by sending typed messages, never by sharing memory
- The supervisor pattern allows automatic restart of failed actors

```rust
// Conceptual actor definition
pub struct AgentActor {
    config: AgentConfig,
    memory: MemoryHandle,
    heal: HealEngine,
    tools: ToolRegistry,
    llm: Box<dyn LlmProvider>,
}

impl Actor for AgentActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteTask> for AgentActor {
    type Result = ResponseFuture<TaskResult>;
    // ...
}
```

### 5.3 Message Types

All inter-agent and intra-agent communication uses typed Rust enums serialized with **MessagePack** (binary, fast, no schema compile step).

Core message types:
- `ExecuteTask` — run a goal
- `ToolCall` — invoke a tool
- `ToolResult` — result from a tool
- `MemoryRead` / `MemoryWrite` — memory operations
- `HealRequest` — trigger healing after error
- `SpawnAgent` — create a sub-agent
- `AgentDone` — task completion signal
- `AgentFailed` — unrecoverable failure signal
- `InspectRequest` — retrieve execution trace

### 5.4 Supervisor Tree

```
SupervisorActor
├── OrchestratorActor (for multi-agent)
│   ├── WorkerAgent_1
│   ├── WorkerAgent_2
│   └── WorkerAgent_N
└── SingleAgent (for standalone)
    └── HealEngine (as sub-actor)
```

If `WorkerAgent_2` panics, `OrchestratorActor` receives a notification and can restart it, reassign its task, or mark it failed and continue without it.

### 5.5 LLM Provider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;
    async fn stream(&self, request: CompletionRequest) -> Result<BoxStream<Token>, LlmError>;
    fn model_name(&self) -> &str;
    fn context_limit(&self) -> usize;
}
```

Built-in adapters: Anthropic Claude, OpenAI, Google Gemini, Ollama (local), any OpenAI-compatible endpoint.

---

## 6. Memory System

Argo implements a three-layer memory system that mimics how human experts retain knowledge.

### 6.1 Short-Term Memory (Redis)

**Purpose:** Active working context for the current task.

**What is stored:**
- Current task description and goal
- Conversation / turn history with the LLM
- Tool call outputs from the current run
- Agent scratchpad (intermediate reasoning)
- Current plan and progress

**Behavior:**
- Sub-millisecond access
- Automatically expires when the task ends (configurable TTL)
- Stored per-agent-run with a unique run ID

**Key structure:**
```
argo:agent:{agent_id}:run:{run_id}:context    → full context blob
argo:agent:{agent_id}:run:{run_id}:turns      → list of LLM turns
argo:agent:{agent_id}:run:{run_id}:scratch    → scratchpad
argo:agent:{agent_id}:run:{run_id}:plan       → current plan
```

### 6.2 Long-Term Memory (SurrealDB)

**Purpose:** Permanent record of what the agent has done, learned, and encountered.

**What is stored:**
- Completed task records (goal, outcome, duration, tools used)
- Key decisions made and why
- Entities encountered (files, APIs, people, projects)
- Relationships between entities (graph edges)
- Error records and their resolutions
- Agent-generated summaries of past work

**Why SurrealDB:** Multi-model — stores relational records AND graph relationships in the same database. An agent can query "what have I done in this project?" (relational) and also "what tools are connected to this API?" (graph) with the same system.

**Schema (excerpt):**

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

-- Entity record (anything the agent worked with)
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD type        ON entity TYPE string; -- file | api | repo | person | ...
DEFINE FIELD identifier  ON entity TYPE string;
DEFINE FIELD metadata    ON entity TYPE object;

-- Relationship (graph edge)
DEFINE TABLE interacted_with SCHEMAFULL;
-- task -> interacted_with -> entity
```

**Context window overflow handling:** When the LLM context window is nearly full, Argo automatically:
1. Takes the oldest N turns
2. Calls the LLM to summarize them into a dense paragraph
3. Stores the full turns in SurrealDB under `task:run_id:archived_turns`
4. Replaces the old turns in Redis with the summary
5. The agent continues with a compressed but complete understanding

### 6.3 Semantic Memory (Qdrant)

**Purpose:** Experience retrieval by meaning. The agent searches past experience before acting.

**What is stored:**
- Embeddings of past task summaries
- Embeddings of error resolutions ("error X was fixed by doing Y")
- Embeddings of tool usage patterns ("when doing task type Z, tool W was most effective")
- Embeddings of lessons learned from post-mortems

**How it is used:**

Before an agent starts executing:
1. The current task description is embedded
2. Qdrant is queried for the top-K most similar past experiences
3. Relevant lessons are injected into the agent's system prompt
4. The agent starts with the accumulated wisdom of its past runs

Before the agent calls a tool:
1. The current context is embedded
2. Qdrant checks if a similar situation led to an error before
3. If yes, the agent is warned and given the resolution strategy

**Collections:**

```
argo_experiences    — task summaries with outcome metadata
argo_errors         — error + resolution pairs
argo_lessons        — structured post-mortem lessons
argo_tool_patterns  — successful tool usage patterns
```

### 6.4 Memory Retrieval Pipeline

```
Agent receives task
        │
        ▼
Embed task description
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

---

## 7. Self-Healing System

The heal system is the most important differentiator in Argo. It runs entirely in the background. The developer never writes error handling code for their agent — the framework handles it.

### 7.1 Error Taxonomy

Every error that can occur during agent execution is classified into one of these types:

```rust
pub enum AgentError {
    // LLM errors
    LlmRateLimit { retry_after: Duration },
    LlmContextOverflow { current: usize, limit: usize },
    LlmHallucination { evidence: String },
    LlmRefusal { reason: String },
    LlmTimeout { elapsed: Duration },
    LlmProviderDown { provider: String },

    // Tool errors
    ToolNotFound { name: String },
    ToolExecutionFailed { name: String, reason: String },
    ToolTimeout { name: String, elapsed: Duration },
    ToolPermissionDenied { name: String, resource: String },
    ToolOutputInvalid { name: String, output: String },

    // Logic errors
    InfiniteLoop { iteration_count: usize },
    GoalUnachievable { reason: String },
    PlanInvalid { plan: String, reason: String },
    ContextCorrupted,

    // Infrastructure errors
    MemoryUnavailable { store: MemoryStore },
    McpConnectionFailed { server: String, reason: String },
    NetworkTimeout { url: String, elapsed: Duration },

    // Agent errors
    SubAgentFailed { agent_id: String, error: Box<AgentError> },
    OrchestratorFailed { reason: String },
}
```

### 7.2 Heal Strategy Chain

When an error occurs, the heal engine runs strategies in order until one succeeds:

```
Error occurs
    │
    ▼
Classify error type
    │
    ▼
Strategy 1: Retry with exponential backoff
    │ (failed? → next)
    ▼
Strategy 2: Reframe the prompt
  (rephrase the instruction to the LLM, add clarification)
    │ (failed? → next)
    ▼
Strategy 3: Swap tool
  (if tool A failed, try tool B that achieves the same result)
    │ (failed? → next)
    ▼
Strategy 4: Decompose
  (break the failing sub-task into smaller pieces)
    │ (failed? → next)
    ▼
Strategy 5: Spawn sub-agent
  (delegate the failing part to a fresh agent with fresh context)
    │ (failed? → next)
    ▼
Strategy 6: Change LLM provider
  (if Claude is down, try GPT-4; if rate limited, use a local model)
    │ (failed? → next)
    ▼
Strategy 7: Reduce scope
  (attempt a simpler version of the task)
    │ (all strategies exhausted)
    ▼
Log failure with full trace → Store in long-term memory → Return structured error to user
```

**Strategy selection is intelligent, not blind.** The heal engine checks semantic memory first: "has this exact error type in this context been seen before? What worked?" It starts from the historically successful strategy, not always from Strategy 1.

### 7.3 Heal Engine Architecture

```rust
pub struct HealEngine {
    strategies: Vec<Box<dyn HealStrategy>>,
    memory: MemoryHandle,
    telemetry: TelemetryHandle,
}

#[async_trait]
pub trait HealStrategy: Send + Sync {
    fn can_handle(&self, error: &AgentError) -> bool;
    async fn apply(&self, ctx: &HealContext) -> HealResult;
    fn name(&self) -> &str;
}

pub struct HealContext {
    pub error: AgentError,
    pub agent_config: AgentConfig,
    pub current_plan: Plan,
    pub iteration: usize,
    pub past_strategies: Vec<String>,  // what's been tried this run
    pub similar_resolutions: Vec<SemanticMatch>,  // from Qdrant
}
```

### 7.4 Post-Mortem Loop

After every error is resolved, Argo runs a post-mortem. This is how agents improve:

```
Error resolved successfully
        │
        ▼
LLM reflects on what happened:
  - What was the error?
  - Why did it occur?
  - Which strategy resolved it?
  - What could have prevented it?
        │
        ▼
Structured lesson written:
  {
    error_type: "ToolExecutionFailed",
    context_summary: "bash tool failed on Python script with import error",
    root_cause: "missing dependency in virtual environment",
    resolution: "added pip install step before execution",
    prevention: "always check imports before running Python scripts",
    confidence: 0.9
  }
        │
        ▼
Lesson embedded and stored in Qdrant (argo_lessons collection)
        │
        ▼
Lesson also stored in SurrealDB for queryable history
        │
        ▼
Next time a similar error occurs, this lesson is retrieved
and the resolution is attempted first
```

### 7.5 User Visibility

The heal system is **completely invisible by default**. The agent runs, heals silently, and delivers the result.

If the developer wants to see what happened:

```python
# Python
result = agent.run("task")
trace = agent.inspect()
print(trace.heal_steps)   # list of HealStep objects
print(trace.lessons)      # new lessons learned

# or pipe to observability backend
agent = Agent(name="x", observe=True)  # writes to OTel stream
```

```bash
# CLI — live view
argo inspect --live agent-run-id-123

# CLI — post-run report
argo report agent-run-id-123
```

---

## 8. Self-Improvement & Evolution

This is what makes Argo genuinely different from other frameworks. Agents get measurably better over time.

### 8.1 Daily Growth Cycle

Every 24 hours (configurable), Argo runs a **growth cycle** for each agent:

```
Growth Cycle (runs in background)
        │
        ▼
Pull all error records from last 24h (SurrealDB)
        │
        ▼
Detect patterns:
  - Same error 3+ times → high-confidence lesson
  - Same tool failing → tool config issue
  - Same task type succeeding → capture pattern
        │
        ▼
Generate improvement proposals:
  - Update agent system prompt with new context
  - Add a pre-check step to avoid known errors
  - Change default strategy order for specific error types
        │
        ▼
Auto-apply low-risk improvements (e.g. prompt additions)
Flag high-risk improvements for developer review
        │
        ▼
Update semantic memory with new patterns
        │
        ▼
Write growth report to SurrealDB
```

### 8.2 Self-Scoring for Loop Agents

Loop agents run until they meet their own quality threshold. The scoring system:

```rust
pub struct QualityRubric {
    pub criteria: Vec<Criterion>,
    pub threshold: f32,       // 0.0 to 1.0
    pub max_iterations: usize,
}

pub struct Criterion {
    pub name: String,
    pub weight: f32,
    pub description: String,  // LLM uses this to score
}
```

Example rubric for a coding agent:

```toml
[quality]
threshold = 0.85
max_iterations = 15

[[quality.criteria]]
name = "tests_pass"
weight = 0.40
description = "All unit tests run and pass with no errors"

[[quality.criteria]]
name = "code_quality"
weight = 0.30
description = "Code follows best practices, no obvious code smells"

[[quality.criteria]]
name = "documentation"
weight = 0.20
description = "README exists and covers setup, usage, and API"

[[quality.criteria]]
name = "error_handling"
weight = 0.10
description = "Functions handle edge cases and return meaningful errors"
```

The loop agent scores its own output against each criterion after every iteration. It only stops when the weighted average score meets or exceeds the threshold — or when `max_iterations` is hit.

### 8.3 Evolution Tracking

Argo tracks agent performance metrics over time, enabling developers to see how their agent has evolved:

```bash
argo stats my-coding-agent --range 30d

# Output:
# Period: last 30 days
# Tasks completed: 47
# Average quality score: 0.81 (+0.12 vs first week)
# Average iterations to completion: 3.2 (-1.8 vs first week)
# Errors per task: 0.9 (-2.1 vs first week)
# Most improved area: tool selection
# Top lesson learned: "Always verify file paths before writing"
```

---

## 9. Multi-Agent System

### 9.1 Topology: Supervisor + Workers

Argo's multi-agent model uses a strict **supervisor/worker hierarchy**. There is always one orchestrator agent that plans and delegates, and N worker agents that specialize and execute.

```
OrchestratorAgent
├── task: "Research AI frameworks and write a report"
│
├── spawns → ResearcherAgent
│   ├── tools: [web_search, browser, files]
│   └── goal: "Find top 5 AI frameworks with details"
│
├── spawns → WriterAgent  (after researcher completes)
│   ├── tools: [files]
│   └── goal: "Write 2000-word report using research"
│
└── spawns → ReviewerAgent  (after writer completes)
    ├── tools: [files]
    └── goal: "Review report for accuracy and quality"
```

### 9.2 Communication Protocol

Agents communicate via typed message passing through the actor mailbox system. No shared state. No direct function calls between agents.

```rust
// Orchestrator sends work to a worker
orchestrator.send(WorkerAgent_1, AssignTask {
    task_id: uuid(),
    goal: "Search for top 5 AI frameworks",
    context: relevant_memory,
    deadline: Some(Duration::from_secs(120)),
}).await?;

// Worker sends result back
worker.send(orchestrator_addr, TaskComplete {
    task_id,
    result: TaskResult::Success(output),
    duration_ms: elapsed,
    tools_used: vec!["web_search", "browser"],
}).await?;
```

### 9.3 Agent Pool

For parallel workloads, Argo provides an `AgentPool` that manages a set of agents and distributes tasks:

```python
from argo import AgentPool

pool = AgentPool(
    workers=4,
    agent_template=dict(
        model="claude-sonnet-4-6",
        tools=[tools.web, tools.files],
        memory="shared"  # workers share semantic memory
    )
)

results = pool.map([
    "Summarize paper 1",
    "Summarize paper 2",
    "Summarize paper 3",
    "Summarize paper 4",
])
```

### 9.4 Shared vs. Isolated Memory

| Mode | Short-term | Long-term | Semantic |
|---|---|---|---|
| `memory="isolated"` | Per-agent | Per-agent | Per-agent |
| `memory="shared"` | Per-agent | Shared pool | Shared pool |
| `memory="persistent"` | Per-agent | Agent-specific | Agent-specific |

In shared mode, all workers in a pool can read each other's long-term memory and semantic experience. A research agent's findings are immediately available to the writing agent.

### 9.5 Loop Agent (Fully Autonomous)

A loop agent runs with no human in the loop at all. It:

1. Plans the task
2. Executes steps
3. Reviews its own output
4. Scores against quality rubric
5. Identifies gaps and failures
6. Re-plans and re-executes
7. Loops until quality threshold is met or max iterations is hit

```python
from argo import LoopAgent

agent = LoopAgent(
    name="code-reviewer",
    model="claude-opus-4-6",
    goal="Review this codebase and produce a full security audit report",
    quality_threshold=0.90,
    max_iterations=25,
    tools=[tools.bash, tools.files, tools.web],
    memory="persistent"
)

# Runs completely autonomously. Heals itself. Loops until done.
report = agent.run()
```

---

## 10. Tool & MCP System

### 10.1 Tool Definition

Every tool in Argo is a typed Rust struct implementing the `Tool` trait, with bindings generated for Python and TypeScript.

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> JsonSchema;
    fn output_schema(&self) -> JsonSchema;
    fn permissions(&self) -> ToolPermissions;
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError>;
}
```

### 10.2 Built-in Tools

Argo ships with a standard library of production-ready tools:

**System tools:**
- `bash` — execute shell commands (sandboxed)
- `files` — read, write, list, delete files
- `http` — make HTTP requests with auth
- `python` — run Python code in a subprocess

**Development tools:**
- `git` — clone, commit, push, diff, branch
- `code` — write, edit, and run code files
- `test` — run test suites and parse results

**Web tools:**
- `web_search` — search the web and return results
- `browser` — control a headless browser (Playwright)
- `fetch` — fetch and parse web pages

**AI tools:**
- `embed` — generate embeddings for text
- `summarize` — summarize long documents
- `extract` — extract structured data from text

**Data tools:**
- `csv` — read and write CSV files
- `json` — parse and query JSON data
- `sql` — run SQL queries against a database

### 10.3 MCP (Model Context Protocol) Integration

Argo implements the full MCP specification. Agents can connect to any MCP server and use its tools as if they were native Argo tools.

```python
agent = Agent(
    name="asana-manager",
    mcp_servers=[
        "https://mcp.asana.com/sse",
        "https://mcp.linear.app/mcp",
    ],
    tools=[tools.web]
)
```

The MCP connector:
- Discovers available tools from the MCP server at startup
- Registers them in the tool registry under the server's namespace
- Handles authentication (bearer token, OAuth)
- Converts between Argo's tool call format and MCP protocol
- Handles server disconnections and reconnects gracefully

### 10.4 Tool Registry

The tool registry is a runtime catalog of all tools available to an agent. It supports hot reload — tools can be added or updated without restarting the agent.

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    versions: HashMap<String, semver::Version>,
    fallbacks: HashMap<String, Vec<String>>,  // tool → alternative tools
}
```

The `fallbacks` map is used by the heal engine: if `bash` fails, try `python`; if `web_search` fails, try `browser`.

### 10.5 Tool Permissions

Every tool declares what system resources it can access. These permissions are enforced at runtime:

```toml
[tools.bash]
allow_filesystem = true
allow_network = false
allow_subprocess = true
working_directory = "./sandbox"
max_execution_time = 30  # seconds

[tools.http]
allow_filesystem = false
allow_network = true
allowed_domains = ["api.github.com", "pypi.org"]
```

---

## 11. Language SDKs

### 11.1 Architecture: All SDKs wrap the same Rust core

```
Rust SDK  ──────────────────────────── calls Rust core directly
Python SDK ── PyO3 (native bindings) → calls Rust core directly
TS SDK ────── napi-rs (native module) → calls Rust core directly
```

All three SDKs call the same Rust code. There is no REST API between SDKs and core. Performance is identical across all languages.

### 11.2 Python SDK (argo-agents)

```python
from argo import Agent, LoopAgent, AgentPool, tools
from argo.memory import Memory
from argo.config import AgentConfig

# Simple agent
agent = Agent(name="...", model="...", tools=[...])
result = await agent.run("goal")

# With full config
config = AgentConfig.from_file("my-agent.toml")
agent = Agent.from_config(config)

# Memory access
memory = agent.memory
past_tasks = memory.query("tasks involving Python", limit=5)

# Async support
import asyncio
result = asyncio.run(agent.run("goal"))

# Sync support (auto-wraps async internally)
result = agent.run_sync("goal")
```

### 11.3 TypeScript SDK (@argo-ai/sdk)

```typescript
import { Agent, LoopAgent, AgentPool, tools } from '@argo-ai/sdk'

// Simple agent
const agent = new Agent({
  name: 'coder',
  model: 'claude-sonnet-4-6',
  tools: [tools.bash, tools.git, tools.files],
  memory: 'persistent',
  heal: true,
})

const result = await agent.run('Build a REST API')

// Type-safe result
if (result.success) {
  console.log(result.output)
  const trace = await agent.inspect()
  console.log(`Healed ${trace.healSteps.length} errors`)
}
```

### 11.4 Rust SDK (argo crate)

```rust
use argo::{Agent, AgentConfig, tools};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::builder()
        .name("coder")
        .model("claude-sonnet-4-6")
        .tool(tools::Bash::default())
        .tool(tools::Git::default())
        .tool(tools::Files::default())
        .memory(MemoryMode::Persistent)
        .heal(true)
        .build()?;

    let result = agent.run("Build a REST API").await?;
    println!("{}", result.output);
    Ok(())
}
```

### 11.5 SDK Feature Parity

Every feature available in the Rust SDK is available identically in Python and TypeScript. This is enforced during CI: a feature parity test suite runs the same operations across all three SDKs and asserts identical behavior and output schemas.

---

## 12. Agent Configuration

Every agent is defined by a TOML configuration file. This file is:
- Version-controlled alongside code
- Validated at startup against the Argo schema
- Deployable across environments via environment variable substitution

### 12.1 Full Agent Config Schema

```toml
# my-agent.toml

[agent]
name = "coder"
version = "1.0.0"
description = "Production coding agent with self-healing"

[model]
provider = "anthropic"          # anthropic | openai | gemini | ollama | custom
model = "claude-sonnet-4-6"
api_key = "${ANTHROPIC_API_KEY}" # environment variable substitution
temperature = 0.2
max_tokens = 8192
context_strategy = "summarize"  # summarize | sliding_window | truncate

[memory]
mode = "persistent"             # persistent | ephemeral | shared
short_term_ttl = 3600           # seconds, for ephemeral mode
long_term_backend = "surrealdb"
vector_backend = "qdrant"
embedding_model = "text-embedding-3-small"

[heal]
enabled = true
max_attempts = 7
strategies = [
    "retry",
    "reframe",
    "swap_tool",
    "decompose",
    "spawn_subagent",
    "change_provider",
    "reduce_scope",
]
background = true               # never surface to user unless inspect() called

[quality]                       # only for LoopAgent
threshold = 0.85
max_iterations = 20
[[quality.criteria]]
name = "tests_pass"
weight = 0.40
description = "All unit tests run and pass with zero failures"

[tools]
enabled = ["bash", "git", "files", "web_search"]
[[tools.mcp]]
url = "https://mcp.asana.com/sse"
auth = { type = "bearer", token = "${ASANA_TOKEN}" }

[permissions]
allow_network = true
allow_filesystem = true
allowed_paths = ["./workspace", "/tmp"]
max_execution_time = 300        # seconds per task

[observe]
enabled = false                  # silent by default
backend = "otlp"                 # otlp | stdout | none
endpoint = "http://localhost:4317"
```

### 12.2 Environment Profiles

```bash
# Development
argo run my-agent.toml --env dev

# Production (uses prod API keys, stricter permissions)
argo run my-agent.toml --env prod

# Environment files: .argo.dev.env, .argo.prod.env
```

---

## 13. CLI

The `argo` CLI is the primary interface for managing agents outside of code.

### 13.1 Commands

```bash
# Initialize a new agent project
argo init my-agent
# Creates: my-agent.toml, .gitignore, README.md

# Run an agent
argo run my-agent.toml "Build a REST API for a todo app"

# Run with live heal trace visible
argo run my-agent.toml "task" --inspect

# Run a loop agent (no goal argument — reads from config)
argo loop my-loop-agent.toml

# Inspect a completed run
argo inspect <run-id>

# Live view of a running agent
argo inspect <run-id> --live

# View agent memory
argo memory list my-agent
argo memory search my-agent "Python projects"
argo memory clear my-agent --type short_term

# View agent stats and evolution
argo stats my-agent --range 30d
argo stats my-agent --compare week1 week4

# Evaluate an agent against scenarios
argo eval my-agent.toml --scenarios ./evals/

# Validate a config file
argo validate my-agent.toml

# List available tools
argo tools list
argo tools info bash

# Connect an MCP server and discover tools
argo mcp connect https://mcp.asana.com/sse
argo mcp tools list asana

# Build and package an agent for distribution
argo package my-agent.toml
```

### 13.2 CLI Output Format

The CLI produces clean, minimal output. Progress is shown with a spinner. The heal trace is hidden unless `--inspect` is passed. On success, only the result is printed.

```
$ argo run coder.toml "Write a Python fibonacci function with tests"

  Running agent: coder
  Goal: Write a Python fibonacci function with tests
  ·····················
  Done in 14.2s

  fibonacci.py   created (24 lines)
  test_fib.py    created (18 lines)
  README.md      created (12 lines)
  Tests: 5 passed
```

With `--inspect`:

```
$ argo run coder.toml "Write a Python fibonacci function" --inspect

  Running agent: coder
  [plan]     Decompose into: write function → write tests → run tests
  [tool]     files.write: fibonacci.py
  [tool]     files.write: test_fibonacci.py
  [tool]     bash: python -m pytest test_fibonacci.py
  [error]    ToolExecutionFailed: ImportError: no module named pytest
  [heal]     Strategy: swap_tool (bash → python subprocess)
  [tool]     python: subprocess.run(['pip', 'install', 'pytest'])
  [tool]     bash: python -m pytest test_fibonacci.py
  [result]   5 passed
  [lesson]   Stored: "check pytest installed before running tests"
  Done in 16.8s
```

---

## 14. Observability

### 14.1 OpenTelemetry Integration

Argo emits structured telemetry via OpenTelemetry. All traces, metrics, and logs use standard OTel conventions so they work with any backend.

**Traces:** Every agent run is a root span. Tool calls, LLM calls, memory operations, and heal steps are child spans.

**Metrics:**
- `argo.task.duration` — histogram of task completion times
- `argo.task.success_rate` — counter of successes vs failures
- `argo.heal.attempts` — counter per strategy type
- `argo.memory.queries` — counter per memory layer
- `argo.llm.tokens` — counter of tokens used per provider
- `argo.tool.latency` — histogram per tool

**Logs:** Structured JSON logs with run ID, agent name, timestamp, level, message.

### 14.2 Inspect API

The `inspect()` API returns a typed `AgentTrace` object:

```python
trace = agent.inspect()

trace.run_id           # str
trace.agent_name       # str
trace.goal             # str
trace.started_at       # datetime
trace.ended_at         # datetime
trace.duration_ms      # int
trace.success          # bool
trace.output           # str | None
trace.iterations       # int (loop agents)
trace.quality_score    # float | None (loop agents)

trace.tool_calls       # List[ToolCall]
trace.llm_calls        # List[LlmCall]
trace.memory_ops       # List[MemoryOp]
trace.heal_steps       # List[HealStep]
trace.lessons          # List[Lesson]
trace.errors           # List[AgentError]
```

---

## 15. Security Model

### 15.1 Tool Sandboxing

All tool execution runs in a restricted context:
- File system access limited to declared paths
- Network access limited to declared domains
- Process spawning requires explicit permission
- Maximum execution time enforced per tool

### 15.2 Secret Management

API keys and tokens are never stored in the agent config file as plain text. They are always read from environment variables:

```toml
api_key = "${ANTHROPIC_API_KEY}"
```

The CLI supports `.argo.env` files (gitignored by default) for local development.

### 15.3 MCP Server Authentication

Each MCP server connection specifies its authentication method:

```toml
[[tools.mcp]]
url = "https://mcp.linear.app/mcp"
auth = { type = "bearer", token = "${LINEAR_TOKEN}" }

[[tools.mcp]]
url = "https://mcp.asana.com/sse"
auth = { type = "oauth2", client_id = "${ASANA_CLIENT_ID}", ... }
```

### 15.4 Agent Isolation

Each agent runs in its own actor with its own memory namespace. Agents cannot read each other's memory unless explicitly configured with `memory = "shared"`. There is no global shared state between agents.

---

## 16. Distribution & Packaging

### 16.1 Package Names

| Registry | Package name | Install command |
|---|---|---|
| crates.io | `argo` | `cargo add argo` |
| PyPI | `argo-agents` | `pip install argo-agents` |
| npm | `@argo-ai/sdk` | `npm install @argo-ai/sdk` |

Sub-packages on cargo:
- `argo` — meta-crate, re-exports everything
- `argo-core` — actor engine, message types
- `argo-memory` — all three memory layers
- `argo-heal` — heal engine and strategies
- `argo-tools` — built-in tool library
- `argo-mcp` — MCP protocol connector
- `argo-observe` — OpenTelemetry integration
- `argo-cli` — CLI binary

### 16.2 Binary Distribution

The `argo` CLI binary is compiled for:
- Linux x86_64 (musl static, glibc dynamic)
- Linux aarch64 (Raspberry Pi, AWS Graviton)
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

Distributed via:
- GitHub Releases (direct binary download)
- Homebrew tap (`brew install argo-agents/tap/argo`)
- `cargo install argo-cli`
- Docker image (`ghcr.io/argo-agents/argo:latest`)

### 16.3 Infrastructure Requirements

For production use, the developer runs:
- Redis (or Redis-compatible, e.g. Valkey) — short-term memory
- SurrealDB — long-term memory
- Qdrant — semantic memory

For local development:
- `argo dev` starts all three services via Docker Compose automatically

---

## 17. Tech Stack — Complete

| Layer | Technology | Version | Reason |
|---|---|---|---|
| Core language | Rust | stable | Zero-cost abstractions, no GC, memory safety |
| Async runtime | Tokio | 1.x | Industry standard, mature, battle-tested |
| Actor framework | Actix | 0.13 | Proven actor model for Rust, excellent docs |
| HTTP server | Axum | 0.7 | Tokio-native, ergonomic, fast |
| Serialization | serde + serde_json | 1.x | Universal Rust serialization standard |
| Binary encoding | rmp-serde (MessagePack) | 1.x | Fast binary format for inter-actor messages |
| Async traits | async-trait | 0.1 | Ergonomic async in trait definitions |
| Error handling | thiserror + anyhow | 1.x | Typed errors (thiserror) + propagation (anyhow) |
| CLI | clap | 4.x | Best-in-class CLI argument parsing for Rust |
| Python bindings | PyO3 | 0.21 | Rust → Python native extensions |
| TS bindings | napi-rs | 2.x | Rust → Node.js native modules |
| Short-term memory | Redis | 7.x | Sub-ms access, TTL support, battle-tested |
| Long-term memory | SurrealDB | 2.x | Multi-model: relational + graph in one |
| Semantic memory | Qdrant | 1.x | Purpose-built vector DB, Rust-native |
| Embeddings | OpenAI text-embedding-3 / Ollama nomic-embed | latest | Swappable via trait |
| Observability | OpenTelemetry (opentelemetry-rs) | 0.23 | Vendor-neutral, works with every backend |
| Config parsing | toml | 0.8 | Native Rust TOML parser |
| Schema validation | schemars | 0.8 | JSON Schema generation from Rust types |
| UUID | uuid | 1.x | Run IDs, entity IDs |
| Time | chrono | 0.4 | Timestamps, durations |
| Logging | tracing + tracing-subscriber | 0.1 | Structured async-aware logging |
| Testing | cargo test + tokio-test | built-in | Standard Rust testing |
| CI | GitHub Actions | N/A | Free, fast, great Rust support |
| Container | Docker + Docker Compose | N/A | Dev environment setup |
| Package registry | crates.io / PyPI / npm | N/A | Standard registries |

---

## 18. Development Phases

### Phase 0 — Planning (Current, 4–6 weeks)

- [ ] Write all RFC documents (see section 19)
- [ ] Define all proto/schema contracts
- [ ] Set up GitHub org and repositories
- [ ] Write contribution guide and issue templates
- [ ] Set up CI/CD pipeline skeleton
- [ ] Set up Discord server for community

**Deliverable:** All RFC docs approved, repositories created, CI green on empty project.

### Phase 1 — Rust Core (8–10 weeks)

- [ ] `argo-core`: actor engine, message types, supervisor tree
- [ ] `argo-core`: LLM provider trait + Anthropic and OpenAI adapters
- [ ] `argo-core`: basic agent execution loop (no memory, no healing)
- [ ] `argo-memory`: Redis short-term memory
- [ ] `argo-memory`: SurrealDB long-term memory
- [ ] `argo-tools`: bash, files, http built-in tools
- [ ] `argo-observe`: basic tracing and logging
- [ ] Integration test: single agent completes a simple task end-to-end

**Deliverable:** A Rust agent that can run a goal, call tools, and store results in long-term memory.

### Phase 2 — Heal Loop + Full Memory (6–8 weeks)

- [ ] `argo-heal`: error taxonomy (all error types)
- [ ] `argo-heal`: all 7 heal strategies
- [ ] `argo-heal`: post-mortem loop
- [ ] `argo-memory`: Qdrant semantic memory
- [ ] `argo-memory`: embedding pipeline
- [ ] `argo-memory`: experience retrieval before task execution
- [ ] `argo-memory`: context window overflow handling
- [ ] Integration test: agent heals from 5 different error types

**Deliverable:** An agent that heals from errors, learns from them, and retrieves past experience.

### Phase 3 — SDKs + CLI (8 weeks)

- [ ] `argo-cli`: all commands (init, run, inspect, memory, stats, tools, mcp)
- [ ] Python SDK via PyO3: full API surface
- [ ] TypeScript SDK via napi-rs: full API surface
- [ ] Feature parity test suite across all three languages
- [ ] SDK documentation and examples

**Deliverable:** Developers can use Argo from Python, TypeScript, and Rust with identical features.

### Phase 4 — Multi-Agent + MCP (6 weeks)

- [ ] `argo-core`: orchestrator actor
- [ ] `argo-core`: agent spawning from within agents
- [ ] `argo-core`: AgentPool
- [ ] `argo-core`: LoopAgent with self-scoring
- [ ] `argo-mcp`: full MCP protocol connector
- [ ] `argo-mcp`: MCP tool auto-discovery
- [ ] `argo-tools`: web_search, browser (Playwright), git, python, code

**Deliverable:** Multi-agent pipelines work. Any MCP server's tools are available to agents.

### Phase 5 — Evolution + Polish (6 weeks)

- [ ] Daily growth cycle implementation
- [ ] `argo stats` command with evolution tracking
- [ ] `argo eval` command for scenario testing
- [ ] Web dashboard for live agent inspection (optional, community-driven)
- [ ] Full documentation site
- [ ] Example agent library: coding agent, research agent, data analyst agent
- [ ] Community tool registry

**Deliverable:** v1.0.0 release.

---

## 19. Required Documentation List

These are all the documents that must be written before development begins, during development, and for release. Each document has a clear purpose and owner.

---

### Group A — Architecture RFCs (write before any code)

These define what gets built. Every technical decision is locked here before a single line of Rust is written.

| # | Document | Purpose |
|---|---|---|
| A-01 | RFC: Actor Engine Design | Defines the Actix actor hierarchy, message types, supervisor tree, and restart policies |
| A-02 | RFC: Memory Architecture | Defines all three memory layers, key schemas, TTL policies, overflow handling |
| A-03 | RFC: Error Taxonomy | Complete enum of every error type with classification rules and metadata |
| A-04 | RFC: Heal Strategy Specification | Each of the 7 strategies: trigger condition, algorithm, success criteria, failure behavior |
| A-05 | RFC: LLM Provider Trait | The full `LlmProvider` trait definition, required methods, error types, streaming contract |
| A-06 | RFC: Tool Trait & Registry | The `Tool` trait, permission model, hot-reload protocol, fallback registration |
| A-07 | RFC: MCP Connector | How Argo implements the MCP client protocol, tool discovery, auth, error handling |
| A-08 | RFC: Multi-Agent Protocol | Message types between orchestrator and workers, agent spawning, result aggregation |
| A-09 | RFC: Self-Improvement System | Daily growth cycle algorithm, pattern detection, improvement proposal schema |
| A-10 | RFC: Loop Agent & Scoring | Quality rubric schema, scoring algorithm, iteration management, termination conditions |
| A-11 | RFC: Observability Contract | OTel span names, metric names, log schema, trace hierarchy |
| A-12 | RFC: Security Model | Tool sandboxing implementation, secret handling, permission enforcement |

---

### Group B — Data Schemas (write before Phase 1)

These define every data structure stored or transmitted.

| # | Document | Purpose |
|---|---|---|
| B-01 | SurrealDB Schema | Full SCHEMAFULL definitions for all tables: task, entity, lesson, error_record, agent |
| B-02 | Redis Key Schema | All key patterns, data types (string/list/hash/set), TTL policies per key type |
| B-03 | Qdrant Collection Schema | All collection definitions, vector dimensions, payload fields, index config |
| B-04 | Agent Config TOML Schema | Complete TOML schema with all fields, types, defaults, validation rules |
| B-05 | MessagePack Message Catalog | Every message type serialized over the actor bus, with field definitions and versioning |
| B-06 | OTel Semantic Conventions | All span names, attribute keys, metric names, units used by Argo |

---

### Group C — API Contracts (write before Phase 3)

These define the developer-facing surface of the SDKs.

| # | Document | Purpose |
|---|---|---|
| C-01 | Rust SDK Public API | All public types, traits, and functions with doc comments and examples |
| C-02 | Python SDK Reference | Every class, method, argument, and return type — matches Rust API 1:1 |
| C-03 | TypeScript SDK Reference | Every class, method, type definition — matches Rust API 1:1 |
| C-04 | CLI Command Reference | Every command, subcommand, flag, and argument with examples and output format |
| C-05 | Feature Parity Matrix | Table showing every feature and its status across Rust / Python / TS SDKs |
| C-06 | Error Reference | Every error that can be returned to the developer, with meaning and suggested fix |

---

### Group D — Developer Guides (write during Phase 3–5)

These help developers use Argo.

| # | Document | Purpose |
|---|---|---|
| D-01 | Getting Started Guide | Install, create first agent, run it, see results — in under 10 minutes |
| D-02 | Building a Coding Agent | Full walkthrough: create a production coding agent from config to first run |
| D-03 | Building a Research Agent | Web browsing agent with persistent memory — full walkthrough |
| D-04 | Building a Multi-Agent Pipeline | Orchestrator + specialists pattern — full walkthrough |
| D-05 | Building a Loop Agent | Autonomous loop with self-scoring — full walkthrough |
| D-06 | Memory Guide | How all three memory layers work, when to use each, how to query them |
| D-07 | Heal System Guide | Understanding the heal loop, reading inspect() output, customizing strategies |
| D-08 | Tool Development Guide | How to write a custom tool in Rust, Python, and TypeScript |
| D-09 | MCP Integration Guide | How to connect an MCP server, discover tools, handle auth |
| D-10 | Configuration Reference | Every field in agent.toml explained with examples |
| D-11 | Observability Guide | Setting up OTel backend, reading traces, writing custom metrics |
| D-12 | Self-Improvement Guide | How the growth cycle works, reading evolution stats, tuning the rubric |
| D-13 | Deployment Guide | Running Argo in production: infrastructure, Docker Compose, Kubernetes |
| D-14 | Migration Guide | For developers migrating from LangChain, CrewAI, or AutoGen |

---

### Group E — Contributor Documentation (write during Phase 0)

These enable open-source contributors to work on Argo.

| # | Document | Purpose |
|---|---|---|
| E-01 | CONTRIBUTING.md | How to contribute, PR process, commit conventions, code review rules |
| E-02 | Architecture Guide (internal) | Deep dive into the internals — how the actor engine works, how memory is accessed |
| E-03 | Development Setup Guide | How to set up the dev environment: Rust toolchain, services, env vars |
| E-04 | Testing Guide | How to write unit tests, integration tests, and scenario evals |
| E-05 | Crate Structure Guide | What each crate does, what belongs where, dependency rules |
| E-06 | Release Process | How releases are versioned, how to cut a release, changelog conventions |
| E-07 | Security Policy | How to report vulnerabilities, security review process |
| E-08 | Code Style Guide | Rustfmt config, clippy rules, naming conventions, doc comment style |

---

### Group F — Project & Planning Documents (write during Phase 0)

| # | Document | Purpose |
|---|---|---|
| F-01 | This Master Plan | The document you are reading |
| F-02 | Project Roadmap | Phase timeline with milestones, tracked in GitHub Projects |
| F-03 | Issue Templates | Bug report, feature request, RFC proposal templates for GitHub |
| F-04 | PR Template | Checklist for all pull requests |
| F-05 | Decision Log | Record of all major architectural decisions and the reasons behind them |
| F-06 | Competitor Analysis | Detailed comparison with LangChain, CrewAI, AutoGen, Semantic Kernel |
| F-07 | Community Charter | Values, governance model, how the project is managed |
| F-08 | README.md | The face of the project: what it is, quick start, links to docs |

---

*End of Argo Master Plan v0.1.0*

*Next step: Begin writing the Group A RFC documents, starting with A-01 (Actor Engine Design) and A-02 (Memory Architecture).*
