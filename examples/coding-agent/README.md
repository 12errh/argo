# Coding Agent Example

A production-ready coding agent with self-healing and test execution capabilities.

## Features

- Plans and executes coding tasks
- Writes code with proper error handling
- Runs tests automatically
- Self-heals from tool failures and code errors
- Scores output quality against rubric

## Quick Start

```bash
# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Run the agent
argo run --config agent.toml "Build a REST API for a todo app with CRUD operations, tests, and README"
```

## Configuration

The agent is configured via `agent.toml`. Key settings:

- **Quality threshold**: 0.85 (agent loops until score meets this)
- **Max iterations**: 15
- **Tools**: bash, files, git, code
- **Memory**: persistent (retains knowledge across runs)
- **Healing**: enabled with 7 strategies

## Example Tasks

```bash
# Build a simple API
argo run --config agent.toml "Create a Python Flask API with /users endpoint"

# Write a utility library
argo run --config agent.toml "Write a JavaScript utility library with string manipulation functions"

# Fix a bug
argo run --config agent.toml "Find and fix the bug in src/main.py that causes TypeError on empty input"
```

## Inspecting Results

After a run, inspect the agent's trace:

```bash
# List recent runs
argo memory list coding-agent

# Inspect a specific run
argo inspect <run-id>

# View healing steps
argo inspect <run-id> --heal
```
