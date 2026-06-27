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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTask {
    pub task_id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub deadline: Option<Duration>,
    pub memory_mode: MemoryMode,
}

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
    Isolated,
    Shared,
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
