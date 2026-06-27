use crate::embedding::EmbeddingProvider;
use crate::error::MemoryError;
use crate::qdrant::QdrantMemory;
use crate::surreal::SurrealMemory;

pub struct ExperienceRetrieval {
    pub qdrant: QdrantMemory,
    pub surreal: SurrealMemory,
    pub embedding: Box<dyn EmbeddingProvider>,
}

impl ExperienceRetrieval {
    pub fn new(
        qdrant: QdrantMemory,
        surreal: SurrealMemory,
        embedding: Box<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            qdrant,
            surreal,
            embedding,
        }
    }

    pub async fn retrieve_context(
        &self,
        _agent_id: &str,
        task_description: &str,
        limit: usize,
    ) -> Result<String, MemoryError> {
        let embedding = self.embedding.embed(task_description).await?;
        let similar = self
            .qdrant
            .query_similar_experiences(&embedding, limit as u64)
            .await?;

        if similar.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("Past experience:\n");
        for item in &similar {
            context.push_str(&format!(
                "- [{}] {} (outcome: {}, tools: {:?})\n",
                item.payload.run_id,
                item.payload.task_summary,
                item.payload.outcome,
                item.payload.tools_used
            ));
        }
        Ok(context)
    }

    pub async fn retrieve_error_lessons(
        &self,
        error_description: &str,
        limit: usize,
    ) -> Result<String, MemoryError> {
        let embedding = self.embedding.embed(error_description).await?;
        let lessons = self.qdrant.query_lessons(&embedding, limit as u64).await?;

        if lessons.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("Lessons from past errors:\n");
        for item in &lessons {
            context.push_str(&format!(
                "- Error: {} | Root cause: {} | Prevention: {}\n",
                item.payload.error_type, item.payload.root_cause, item.payload.prevention
            ));
        }
        Ok(context)
    }
}
