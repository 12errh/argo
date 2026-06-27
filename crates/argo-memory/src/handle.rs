use std::time::Duration;

use crate::error::MemoryError;
use crate::redis::{RedisMemory, StoredTurn};
use crate::surreal::{Entity, SurrealMemory, TaskRecord};

pub struct MemoryHandle {
    pub redis: RedisMemory,
    pub surreal: SurrealMemory,
}

impl MemoryHandle {
    pub fn new(redis: RedisMemory, surreal: SurrealMemory) -> Self {
        Self { redis, surreal }
    }

    pub async fn store_context(
        &self,
        agent_id: &str,
        run_id: &str,
        context: &str,
    ) -> Result<(), MemoryError> {
        self.redis
            .store_context(agent_id, run_id, context, Duration::from_secs(3600))
            .await
    }

    pub async fn get_context(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        self.redis.get_context(agent_id, run_id).await
    }

    pub async fn store_turns(
        &self,
        agent_id: &str,
        run_id: &str,
        turns: &[StoredTurn],
    ) -> Result<(), MemoryError> {
        self.redis
            .store_turns(agent_id, run_id, turns, Duration::from_secs(3600))
            .await
    }

    pub async fn get_turns(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Vec<StoredTurn>, MemoryError> {
        self.redis.get_turns(agent_id, run_id).await
    }

    pub async fn store_scratch(
        &self,
        agent_id: &str,
        run_id: &str,
        data: &str,
    ) -> Result<(), MemoryError> {
        self.redis
            .store_scratch(agent_id, run_id, data, Duration::from_secs(3600))
            .await
    }

    pub async fn get_scratch(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        self.redis.get_scratch(agent_id, run_id).await
    }

    pub async fn store_plan(
        &self,
        agent_id: &str,
        run_id: &str,
        plan: &str,
    ) -> Result<(), MemoryError> {
        self.redis
            .store_plan(agent_id, run_id, plan, Duration::from_secs(3600))
            .await
    }

    pub async fn get_plan(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        self.redis.get_plan(agent_id, run_id).await
    }

    pub async fn store_task_record(&self, record: &TaskRecord) -> Result<(), MemoryError> {
        self.surreal.store_task_record(record).await
    }

    pub async fn get_task_record(&self, run_id: &str) -> Result<Option<TaskRecord>, MemoryError> {
        self.surreal.get_task_record(run_id).await
    }

    pub async fn store_entity(&self, entity: &Entity) -> Result<(), MemoryError> {
        self.surreal.store_entity(entity).await
    }

    pub async fn get_entity(
        &self,
        entity_type: &str,
        identifier: &str,
    ) -> Result<Option<Entity>, MemoryError> {
        self.surreal.get_entity(entity_type, identifier).await
    }

    pub async fn create_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
    ) -> Result<(), MemoryError> {
        self.surreal.create_relationship(from_id, to_id, rel_type).await
    }

    pub async fn cleanup(&self, agent_id: &str, run_id: &str) -> Result<(), MemoryError> {
        self.redis.cleanup(agent_id, run_id).await
    }
}
