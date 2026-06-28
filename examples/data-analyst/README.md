# Data Analyst Agent Example

A data analysis agent for processing, analyzing, and reporting on datasets.

## Features

- Processes CSV, JSON, and other data formats
- Generates statistical summaries
- Creates visualizations (charts, graphs)
- Identifies trends and anomalies
- Produces formatted reports

## Quick Start

```bash
# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Run the agent
argo run --config agent.toml "Analyze the sales data in data/sales.csv and create a summary report with charts"
```

## Configuration

Key settings in `agent.toml`:

- **Quality threshold**: 0.80
- **Max iterations**: 12
- **Tools**: bash, python, files, code
- **Memory**: persistent (retains analysis patterns)

## Example Tasks

```bash
# CSV analysis
argo run --config agent.toml "Load data/customers.csv, analyze demographics, and create a pie chart of age groups"

# Trend analysis
argo run --config agent.toml "Analyze monthly revenue data for 2024, identify trends and forecast Q1 2025"

# Data cleaning
argo run --config agent.toml "Clean the messy dataset in data/raw.csv: handle missing values, fix formats, remove duplicates"
```

## Self-Healing

The agent automatically handles common data issues:

- Missing values → fills with appropriate defaults
- Type mismatches → attempts conversion
- File format issues → tries alternative parsers
- Large datasets → processes in chunks

```bash
# See what the agent healed
argo inspect <run-id> --heal
```
