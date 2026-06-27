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
