use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::MemoryError;

pub struct RedisMemory {
    client: redis::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTurn {
    pub role: String,
    pub content: String,
}

impl RedisMemory {
    pub async fn new(url: &str) -> Result<Self, MemoryError> {
        let client = redis::Client::open(url)
            .map_err(|e| MemoryError::ConnectionFailed(e.to_string()))?;
        Ok(Self { client })
    }

    fn context_key(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:context", agent_id, run_id)
    }

    fn turns_key(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:turns", agent_id, run_id)
    }

    fn scratch_key(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:scratch", agent_id, run_id)
    }

    fn plan_key(agent_id: &str, run_id: &str) -> String {
        format!("argo:agent:{}:run:{}:plan", agent_id, run_id)
    }

    pub async fn store_context(
        &self,
        agent_id: &str,
        run_id: &str,
        context: &str,
        ttl: Duration,
    ) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::context_key(agent_id, run_id);
        let _: () = conn
            .set_ex(&key, context, ttl.as_secs() as usize)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn get_context(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::context_key(agent_id, run_id);
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(result)
    }

    pub async fn store_turns(
        &self,
        agent_id: &str,
        run_id: &str,
        turns: &[StoredTurn],
        ttl: Duration,
    ) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::turns_key(agent_id, run_id);
        let json = serde_json::to_string(turns)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;
        let _: () = conn
            .set_ex(&key, json, ttl.as_secs() as usize)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn get_turns(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Vec<StoredTurn>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::turns_key(agent_id, run_id);
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        match result {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| MemoryError::Serialization(e.to_string())),
            None => Ok(Vec::new()),
        }
    }

    pub async fn store_scratch(
        &self,
        agent_id: &str,
        run_id: &str,
        data: &str,
        ttl: Duration,
    ) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::scratch_key(agent_id, run_id);
        let _: () = conn
            .set_ex(&key, data, ttl.as_secs() as usize)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn get_scratch(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::scratch_key(agent_id, run_id);
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(result)
    }

    pub async fn store_plan(
        &self,
        agent_id: &str,
        run_id: &str,
        plan: &str,
        ttl: Duration,
    ) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::plan_key(agent_id, run_id);
        let _: () = conn
            .set_ex(&key, plan, ttl.as_secs() as usize)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn get_plan(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<Option<String>, MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        let key = Self::plan_key(agent_id, run_id);
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        Ok(result)
    }

    pub async fn cleanup(
        &self,
        agent_id: &str,
        run_id: &str,
    ) -> Result<(), MemoryError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| MemoryError::Redis(e.to_string()))?;
        for key in [
            Self::context_key(agent_id, run_id),
            Self::turns_key(agent_id, run_id),
            Self::scratch_key(agent_id, run_id),
            Self::plan_key(agent_id, run_id),
        ] {
            let _: () = conn
                .del(&key)
                .await
                .map_err(|e| MemoryError::Redis(e.to_string()))?;
        }
        Ok(())
    }
}
