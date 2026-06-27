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
