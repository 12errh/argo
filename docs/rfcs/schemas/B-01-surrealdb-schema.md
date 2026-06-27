# B-01: SurrealDB Schema

**Status:** Proposed
**Author:** Argo Core Team
**Created:** 2026-06-27

---

## Overview

Complete SurrealDB SCHEMAFULL definitions for all Argo tables.

## Schema

```sql
-- Use memory database
USE NS argo DB memory;

-- Task record: tracks completed agent tasks
DEFINE TABLE task SCHEMAFULL;
DEFINE FIELD agent_id    ON task TYPE string;
DEFINE FIELD goal        ON task TYPE string;
DEFINE FIELD outcome     ON task TYPE string ASSERT $value INSIDE ['success', 'partial', 'failed'];
DEFINE FIELD summary     ON task TYPE string;
DEFINE FIELD tools_used  ON task TYPE array;
DEFINE FIELD duration_ms ON task TYPE int;
DEFINE FIELD started_at  ON task TYPE datetime;
DEFINE FIELD ended_at    ON task TYPE datetime;
DEFINE FIELD run_id      ON task TYPE string;
DEFINE FIELD metadata    ON task TYPE option<object>;
DEFINE INDEX task_agent_idx ON task FIELDS agent_id;
DEFINE INDEX task_run_idx ON task FIELDS run_id;
DEFINE INDEX task_started_idx ON task FIELDS started_at;

-- Entity record: anything the agent worked with
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD type        ON entity TYPE string ASSERT $value INSIDE ['file', 'api', 'repo', 'person', 'project', 'tool', 'other'];
DEFINE FIELD identifier  ON entity TYPE string;
DEFINE FIELD metadata    ON entity TYPE option<object>;
DEFINE FIELD created_at  ON entity TYPE datetime;
DEFINE FIELD updated_at  ON entity TYPE datetime;
DEFINE INDEX entity_type_idx ON entity FIELDS type;
DEFINE INDEX entity_identifier_idx ON entity FIELDS identifier;

-- Graph relationships
DEFINE TABLE interacted_with SCHEMAFULL;
DEFINE FIELD from_task    ON interacted_with TYPE string;
DEFINE FIELD to_entity    ON interacted_with TYPE string;
DEFINE FIELD relation     ON interacted_with TYPE string;
DEFINE FIELD metadata     ON interacted_with TYPE option<object>;
DEFINE FIELD created_at   ON interacted_with TYPE datetime;

DEFINE TABLE depends_on SCHEMAFULL;
DEFINE FIELD from_entity  ON depends_on TYPE string;
DEFINE FIELD to_entity    ON depends_on TYPE string;
DEFINE FIELD relation     ON depends_on TYPE string;
DEFINE FIELD metadata     ON depends_on TYPE option<object>;

-- Error record: tracks errors encountered during execution
DEFINE TABLE error_record SCHEMAFULL;
DEFINE FIELD task_id      ON error_record TYPE string;
DEFINE FIELD run_id       ON error_record TYPE string;
DEFINE FIELD error_type   ON error_record TYPE string;
DEFINE FIELD message      ON error_record TYPE string;
DEFINE FIELD context      ON error_record TYPE option<object>;
DEFINE FIELD resolution   ON error_record TYPE option<string>;
DEFINE FIELD strategy     ON error_record TYPE option<string>;
DEFINE FIELD occurred_at  ON error_record TYPE datetime;
DEFINE FIELD resolved_at  ON error_record TYPE option<datetime>;
DEFINE INDEX error_task_idx ON error_record FIELDS task_id;
DEFINE INDEX error_type_idx ON error_record FIELDS error_type;

-- Lesson record: structured knowledge from post-mortems
DEFINE TABLE lesson SCHEMAFULL;
DEFINE FIELD error_type      ON lesson TYPE string;
DEFINE FIELD context_summary ON lesson TYPE string;
DEFINE FIELD root_cause      ON lesson TYPE string;
DEFINE FIELD resolution      ON lesson TYPE string;
DEFINE FIELD prevention      ON lesson TYPE string;
DEFINE FIELD confidence      ON lesson TYPE float ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD source_error    ON lesson TYPE string;
DEFINE FIELD created_at      ON lesson TYPE datetime;
DEFINE INDEX lesson_error_idx ON lesson FIELDS error_type;

-- Agent record: agent configuration and metadata
DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD name        ON agent TYPE string;
DEFINE FIELD model       ON agent TYPE string;
DEFINE FIELD config      ON agent TYPE object;
DEFINE FIELD created_at  ON agent TYPE datetime;
DEFINE FIELD updated_at  ON agent TYPE datetime;
DEFINE INDEX agent_name_idx ON agent FIELDS name;

-- Growth report: records from daily growth cycles
DEFINE TABLE growth_report SCHEMAFULL;
DEFINE FIELD agent_id          ON growth_report TYPE string;
DEFINE FIELD timestamp         ON growth_report TYPE datetime;
DEFINE FIELD errors_analyzed   ON growth_report TYPE int;
DEFINE FIELD patterns_detected ON growth_report TYPE int;
DEFINE FIELD proposals_generated ON growth_report TYPE int;
DEFINE FIELD proposals_applied ON growth_report TYPE int;
DEFINE FIELD patterns          ON growth_report TYPE array;
DEFINE FIELD proposals         ON growth_report TYPE array;
DEFINE INDEX growth_agent_idx ON growth_report FIELDS agent_id;

-- Archived turns: for context window overflow handling
DEFINE TABLE archived_turns SCHEMAFULL;
DEFINE FIELD run_id      ON archived_turns TYPE string;
DEFINE FIELD agent_id    ON archived_turns TYPE string;
DEFINE FIELD turns       ON archived_turns TYPE array;
DEFINE FIELD archived_at ON archived_turns TYPE datetime;
DEFINE INDEX archived_run_idx ON archived_turns FIELDS run_id;
```
