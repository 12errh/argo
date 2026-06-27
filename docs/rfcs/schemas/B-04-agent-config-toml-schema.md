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
