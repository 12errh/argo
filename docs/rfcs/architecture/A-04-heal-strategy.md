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
    let embedding = embed_error(error);
    let similar = memory.query_similar_errors(&embedding, 5).await;

    if let Some(best_resolution) = similar.iter().max_by_key(|r| r.confidence) {
        return reorder_strategies(best_resolution.strategy);
    }

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
