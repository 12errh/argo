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
    "tool_calls": []
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
