use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant};
use serde::{Deserialize, Serialize};

use crate::error::MemoryError;

pub const VECTOR_DIMENSION: usize = 1536;

pub mod collections {
    pub const EXPERIENCES: &str = "argo_experiences";
    pub const ERRORS: &str = "argo_errors";
    pub const LESSONS: &str = "argo_lessons";
    pub const TOOL_PATTERNS: &str = "argo_tool_patterns";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceRecord {
    pub task_summary: String,
    pub outcome: String,
    pub tools_used: Vec<String>,
    pub duration_ms: i64,
    pub agent_id: String,
    pub run_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResolution {
    pub error_type: String,
    pub context_summary: String,
    pub resolution: String,
    pub strategy: String,
    pub confidence: f32,
    pub agent_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonRecord {
    pub error_type: String,
    pub root_cause: String,
    pub prevention: String,
    pub confidence: f32,
    pub agent_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPattern {
    pub tool_name: String,
    pub task_type: String,
    pub success_rate: f32,
    pub avg_duration_ms: i64,
    pub agent_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredResult<T> {
    pub payload: T,
    pub score: f32,
}

pub struct QdrantMemory {
    client: Qdrant,
}

impl QdrantMemory {
    pub async fn new(url: &str) -> Result<Self, MemoryError> {
        let client = Qdrant::from_url(url)
            .build()
            .map_err(|e| MemoryError::Qdrant(e.to_string()))?;
        Ok(Self { client })
    }

    pub async fn ensure_collections(&self) -> Result<(), MemoryError> {
        let collections = [
            collections::EXPERIENCES,
            collections::ERRORS,
            collections::LESSONS,
            collections::TOOL_PATTERNS,
        ];

        for name in &collections {
            let req = CreateCollectionBuilder::new(*name).vectors_config(VectorParamsBuilder::new(
                VECTOR_DIMENSION as u64,
                Distance::Cosine,
            ));
            let _ = self.client.create_collection(req).await;
        }
        Ok(())
    }

    fn next_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    async fn upsert_record<T: Serialize>(
        &self,
        collection: &str,
        embedding: &[f32],
        record: &T,
    ) -> Result<(), MemoryError> {
        let json_val =
            serde_json::to_value(record).map_err(|e| MemoryError::Serialization(e.to_string()))?;
        let payload =
            Payload::try_from(json_val).map_err(|e| MemoryError::Serialization(e.to_string()))?;
        let point = PointStruct::new(Self::next_id(), embedding.to_vec(), payload);
        let req = UpsertPointsBuilder::new(collection, vec![point]);
        self.client
            .upsert_points(req)
            .await
            .map_err(|e| MemoryError::Qdrant(e.to_string()))?;
        Ok(())
    }

    async fn search<T: for<'de> Deserialize<'de>>(
        &self,
        collection: &str,
        query_embedding: &[f32],
        limit: u64,
    ) -> Result<Vec<ScoredResult<T>>, MemoryError> {
        let req = SearchPointsBuilder::new(collection, query_embedding.to_vec(), limit);
        let response = self
            .client
            .search_points(req)
            .await
            .map_err(|e| MemoryError::Qdrant(e.to_string()))?;

        let mut scored = Vec::new();
        for point in response.result {
            let payload: Payload = point.payload.into();
            let record: T = payload
                .deserialize()
                .map_err(|e| MemoryError::Serialization(e.to_string()))?;
            scored.push(ScoredResult {
                payload: record,
                score: point.score,
            });
        }
        Ok(scored)
    }

    pub async fn store_experience(
        &self,
        embedding: &[f32],
        record: &ExperienceRecord,
    ) -> Result<(), MemoryError> {
        self.upsert_record(collections::EXPERIENCES, embedding, record)
            .await
    }

    pub async fn query_similar_experiences(
        &self,
        query_embedding: &[f32],
        limit: u64,
    ) -> Result<Vec<ScoredResult<ExperienceRecord>>, MemoryError> {
        self.search(collections::EXPERIENCES, query_embedding, limit)
            .await
    }

    pub async fn store_error_resolution(
        &self,
        embedding: &[f32],
        record: &ErrorResolution,
    ) -> Result<(), MemoryError> {
        self.upsert_record(collections::ERRORS, embedding, record)
            .await
    }

    pub async fn query_similar_errors(
        &self,
        query_embedding: &[f32],
        limit: u64,
    ) -> Result<Vec<ScoredResult<ErrorResolution>>, MemoryError> {
        self.search(collections::ERRORS, query_embedding, limit)
            .await
    }

    pub async fn store_lesson(
        &self,
        embedding: &[f32],
        record: &LessonRecord,
    ) -> Result<(), MemoryError> {
        self.upsert_record(collections::LESSONS, embedding, record)
            .await
    }

    pub async fn query_lessons(
        &self,
        query_embedding: &[f32],
        limit: u64,
    ) -> Result<Vec<ScoredResult<LessonRecord>>, MemoryError> {
        self.search(collections::LESSONS, query_embedding, limit)
            .await
    }

    pub async fn store_tool_pattern(
        &self,
        embedding: &[f32],
        record: &ToolPattern,
    ) -> Result<(), MemoryError> {
        self.upsert_record(collections::TOOL_PATTERNS, embedding, record)
            .await
    }

    pub async fn query_tool_patterns(
        &self,
        query_embedding: &[f32],
        limit: u64,
    ) -> Result<Vec<ScoredResult<ToolPattern>>, MemoryError> {
        self.search(collections::TOOL_PATTERNS, query_embedding, limit)
            .await
    }
}
