use crate::error::MemoryError;
use crate::redis::StoredTurn;
use crate::surreal::SurrealMemory;

pub struct ContextOverflowHandler {
    pub surreal: SurrealMemory,
    pub context_limit: usize,
}

impl ContextOverflowHandler {
    pub fn new(surreal: SurrealMemory, context_limit: usize) -> Self {
        Self {
            surreal,
            context_limit,
        }
    }

    pub fn estimate_tokens(text: &str) -> usize {
        text.split_whitespace().count()
    }

    pub fn should_compress(&self, turns: &[StoredTurn]) -> bool {
        let total: usize = turns
            .iter()
            .map(|t| Self::estimate_tokens(&t.content))
            .sum();
        total > self.context_limit
    }

    pub fn select_turns_to_summarize(
        &self,
        turns: &[StoredTurn],
    ) -> (Vec<StoredTurn>, Vec<StoredTurn>) {
        let mut accumulated = 0;
        let cutoff = turns
            .iter()
            .position(|t| {
                accumulated += Self::estimate_tokens(&t.content);
                accumulated > self.context_limit / 2
            })
            .unwrap_or(turns.len() / 2);

        let to_summarize = turns[..=cutoff].to_vec();
        let remaining = turns[cutoff + 1..].to_vec();
        (to_summarize, remaining)
    }

    pub async fn archive_turns(
        &self,
        run_id: &str,
        turns: &[StoredTurn],
    ) -> Result<(), MemoryError> {
        let _ = run_id;
        let _ = turns;
        Ok(())
    }
}
