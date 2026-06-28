# Research Agent Example

A web research agent with persistent memory and source tracking.

## Features

- Searches the web for information
- Browses and extracts content from pages
- Tracks sources and citations
- Synthesizes findings into structured reports
- Remembers past research for efficiency

## Quick Start

```bash
# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Run the agent
argo run --config agent.toml "Research the top 5 Rust web frameworks in 2025, compare their features, and write a summary report"
```

## Configuration

Key settings in `agent.toml`:

- **Quality threshold**: 0.80
- **Max iterations**: 10
- **Tools**: web_search, browser, files, http
- **Memory**: persistent (retains research across runs)
- **Network**: enabled for web access

## Example Tasks

```bash
# Technology comparison
argo run --config agent.toml "Compare React vs Vue vs Svelte for a new project, recommend based on requirements"

# Market research
argo run --config agent.toml "Research the current state of AI code assistants, list top 10 with features and pricing"

# Academic research
argo run --config agent.toml "Find recent papers on transformer efficiency improvements, summarize key findings"
```

## Memory Advantage

The research agent uses persistent memory, so:

1. First run about a topic builds a knowledge base
2. Subsequent runs retrieve and build on past research
3. Sources and findings are retained for future reference

```bash
# Check what the agent remembers
argo memory search research-agent "AI frameworks"

# View evolution over time
argo stats research-agent --range 30d
```
