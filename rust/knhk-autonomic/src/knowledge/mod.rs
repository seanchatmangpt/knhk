//! # Knowledge Component - Persistent Learning
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Knowledge component stores and maintains learned patterns, success memories,
//! and predictive models. It enables the system to improve decision-making over time.
//!
//! ## Responsibilities
//!
//! - Store learned patterns (what problems occur)
//! - Track success memories (what actions work when)
//! - Maintain feedback cycle history
//! - Calculate pattern reliability and action success rates
//! - Persist knowledge across system restarts
//! - Provide fast lookup for planning (≤8 ticks)
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::knowledge::KnowledgeBase;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let kb = KnowledgeBase::new("./knowledge.db").await?;
//!
//! // Record successful action
//! kb.record_success("High error rate", action_id, true).await?;
//!
//! // Get success rate
//! let rate = kb.get_success_rate(&action_id).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{AutonomicError, Result};
use crate::types::{ActionExecution, FeedbackCycle, LearnedPattern, SuccessMemory};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Knowledge base for persistent learning
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    /// Storage path
    path: String,

    /// Learned patterns
    patterns: Arc<RwLock<HashMap<String, LearnedPattern>>>,

    /// Success memories
    memories: Arc<RwLock<HashMap<String, SuccessMemory>>>,

    /// Feedback cycle history
    cycles: Arc<RwLock<Vec<FeedbackCycle>>>,

    /// Action success rates cache
    success_rates: Arc<RwLock<HashMap<Uuid, f64>>>,

    /// Persistent storage (sled database)
    db: Arc<RwLock<Option<sled::Db>>>,
}

impl KnowledgeBase {
    /// Create a new knowledge base
    #[instrument]
    pub async fn new(path: impl Into<String>) -> Result<Self> {
        let path = path.into();

        // Open persistent database
        let db = sled::open(&path)
            .map_err(|e| AutonomicError::Storage(format!("Failed to open database: {}", e)))?;

        let mut kb = Self {
            path: path.clone(),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            memories: Arc::new(RwLock::new(HashMap::new())),
            cycles: Arc::new(RwLock::new(Vec::new())),
            success_rates: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(RwLock::new(Some(db))),
        };

        // Load existing knowledge
        kb.load().await?;

        debug!("Knowledge base initialized at {}", path);
        Ok(kb)
    }

    /// Record a learned pattern
    #[instrument(skip(self))]
    pub async fn record_pattern(
        &mut self,
        description: impl Into<String>,
        associated_actions: Vec<Uuid>,
    ) -> Result<Uuid> {
        let description = description.into();

        let mut patterns = self.patterns.write().await;

        // Update existing pattern or create new
        if let Some(pattern) = patterns.get_mut(&description) {
            pattern.frequency += 1;
            pattern.last_seen = Utc::now();
            Ok(pattern.id)
        } else {
            let pattern = LearnedPattern {
                id: Uuid::new_v4(),
                description: description.clone(),
                frequency: 1,
                reliability: 0.5,
                associated_actions,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
            };

            let id = pattern.id;
            patterns.insert(description, pattern);
            self.persist().await?;
            Ok(id)
        }
    }

    /// Record success or failure of an action
    #[instrument(skip(self))]
    pub async fn record_success(
        &mut self,
        situation: impl Into<String>,
        action_id: Uuid,
        success: bool,
    ) -> Result<()> {
        let situation = situation.into();

        let mut memories = self.memories.write().await;

        // Update existing memory or create new
        if let Some(memory) = memories.get_mut(&situation) {
            memory.attempts += 1;
            if success {
                memory.successes += 1;
            }
            memory.success_rate = memory.successes as f64 / memory.attempts as f64;

            // Update action list if not already present
            if !memory.successful_actions.contains(&action_id) && success {
                memory.successful_actions.push(action_id);
            }
        } else {
            let memory = SuccessMemory {
                id: Uuid::new_v4(),
                situation: situation.clone(),
                successful_actions: if success { vec![action_id] } else { vec![] },
                success_rate: if success { 1.0 } else { 0.0 },
                attempts: 1,
                successes: if success { 1 } else { 0 },
            };

            memories.insert(situation, memory);
        }

        // Update success rate cache
        self.update_success_rate_cache(action_id).await?;

        self.persist().await?;

        Ok(())
    }

    /// Record a feedback cycle
    #[instrument(skip(self, cycle))]
    pub async fn record_cycle(&mut self, cycle: FeedbackCycle) -> Result<()> {
        let mut cycles = self.cycles.write().await;
        cycles.push(cycle);

        // Keep last 1000 cycles
        if cycles.len() > 1000 {
            cycles.remove(0);
        }

        self.persist().await?;

        Ok(())
    }

    /// Get success rate for an action
    #[instrument(skip(self))]
    pub async fn get_success_rate(&self, action_id: &Uuid) -> Result<f64> {
        let rates = self.success_rates.read().await;
        Ok(rates.get(action_id).copied().unwrap_or(0.5))
    }

    /// Get all success rates
    pub async fn get_all_success_rates(&self) -> Result<HashMap<Uuid, f64>> {
        let rates = self.success_rates.read().await;
        Ok(rates.clone())
    }

    /// Update success rate cache for an action
    async fn update_success_rate_cache(&self, action_id: Uuid) -> Result<()> {
        let memories = self.memories.read().await;

        let mut total_attempts = 0u64;
        let mut total_successes = 0u64;

        for memory in memories.values() {
            if memory.successful_actions.contains(&action_id) {
                total_attempts += memory.attempts;
                total_successes += memory.successes;
            }
        }

        let rate = if total_attempts > 0 {
            total_successes as f64 / total_attempts as f64
        } else {
            0.5
        };

        let mut rates = self.success_rates.write().await;
        rates.insert(action_id, rate);

        Ok(())
    }

    /// Get learned patterns
    pub async fn get_patterns(&self) -> Result<Vec<LearnedPattern>> {
        let patterns = self.patterns.read().await;
        Ok(patterns.values().cloned().collect())
    }

    /// Get success memories
    pub async fn get_memories(&self) -> Result<Vec<SuccessMemory>> {
        let memories = self.memories.read().await;
        Ok(memories.values().cloned().collect())
    }

    /// Get feedback cycle history
    pub async fn get_cycles(&self) -> Result<Vec<FeedbackCycle>> {
        let cycles = self.cycles.read().await;
        Ok(cycles.clone())
    }

    /// Persist knowledge to disk
    #[instrument(skip(self))]
    async fn persist(&self) -> Result<()> {
        let db_lock = self.db.read().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| AutonomicError::Storage("Database not initialized".to_string()))?;

        // Persist patterns
        let patterns = self.patterns.read().await;
        let patterns_json =
            serde_json::to_vec(&*patterns).map_err(|e| AutonomicError::Serialization(e))?;
        db.insert("patterns", patterns_json)
            .map_err(|e| AutonomicError::Storage(format!("Failed to persist patterns: {}", e)))?;

        // Persist memories
        let memories = self.memories.read().await;
        let memories_json =
            serde_json::to_vec(&*memories).map_err(|e| AutonomicError::Serialization(e))?;
        db.insert("memories", memories_json)
            .map_err(|e| AutonomicError::Storage(format!("Failed to persist memories: {}", e)))?;

        // Persist cycles (last 100)
        let cycles = self.cycles.read().await;
        let recent_cycles: Vec<_> = cycles.iter().rev().take(100).cloned().collect();
        let cycles_json =
            serde_json::to_vec(&recent_cycles).map_err(|e| AutonomicError::Serialization(e))?;
        db.insert("cycles", cycles_json)
            .map_err(|e| AutonomicError::Storage(format!("Failed to persist cycles: {}", e)))?;

        db.flush()
            .map_err(|e| AutonomicError::Storage(format!("Failed to flush database: {}", e)))?;

        debug!("Knowledge persisted to disk");
        Ok(())
    }

    /// Load knowledge from disk
    #[instrument(skip(self))]
    async fn load(&mut self) -> Result<()> {
        let db_lock = self.db.read().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| AutonomicError::Storage("Database not initialized".to_string()))?;

        // Load patterns
        if let Some(data) = db
            .get("patterns")
            .map_err(|e| AutonomicError::Storage(format!("Failed to load patterns: {}", e)))?
        {
            let patterns: HashMap<String, LearnedPattern> =
                serde_json::from_slice(&data).map_err(|e| AutonomicError::Serialization(e))?;
            let mut p = self.patterns.write().await;
            *p = patterns;
            debug!("Loaded {} patterns", p.len());
        }

        // Load memories
        if let Some(data) = db
            .get("memories")
            .map_err(|e| AutonomicError::Storage(format!("Failed to load memories: {}", e)))?
        {
            let memories: HashMap<String, SuccessMemory> =
                serde_json::from_slice(&data).map_err(|e| AutonomicError::Serialization(e))?;
            let mut m = self.memories.write().await;
            *m = memories;
            debug!("Loaded {} memories", m.len());
        }

        // Load cycles
        if let Some(data) = db
            .get("cycles")
            .map_err(|e| AutonomicError::Storage(format!("Failed to load cycles: {}", e)))?
        {
            let cycles: Vec<FeedbackCycle> =
                serde_json::from_slice(&data).map_err(|e| AutonomicError::Serialization(e))?;
            let mut c = self.cycles.write().await;
            *c = cycles;
            debug!("Loaded {} cycles", c.len());
        }

        // Rebuild success rate cache
        self.rebuild_success_rate_cache().await?;

        Ok(())
    }

    /// Rebuild success rate cache from memories
    async fn rebuild_success_rate_cache(&self) -> Result<()> {
        let memories = self.memories.read().await;
        let mut rates = self.success_rates.write().await;

        rates.clear();

        // Aggregate success rates by action
        let mut action_stats: HashMap<Uuid, (u64, u64)> = HashMap::new();

        for memory in memories.values() {
            for action_id in &memory.successful_actions {
                let stats = action_stats.entry(*action_id).or_insert((0, 0));
                stats.0 += memory.successes;
                stats.1 += memory.attempts;
            }
        }

        for (action_id, (successes, attempts)) in action_stats {
            let rate = if attempts > 0 {
                successes as f64 / attempts as f64
            } else {
                0.5
            };
            rates.insert(action_id, rate);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_record_and_retrieve_pattern() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut kb = KnowledgeBase::new(db_path.to_str().unwrap()).await.unwrap();

        let action_id = Uuid::new_v4();
        let pattern_id = kb
            .record_pattern("Test pattern", vec![action_id])
            .await
            .unwrap();

        assert!(pattern_id.as_u128() > 0);

        let patterns = kb.get_patterns().await.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].description, "Test pattern");
    }

    #[tokio::test]
    async fn test_record_success() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut kb = KnowledgeBase::new(db_path.to_str().unwrap()).await.unwrap();

        let action_id = Uuid::new_v4();

        // Record success
        kb.record_success("Test situation", action_id, true)
            .await
            .unwrap();

        // Check success rate
        let rate = kb.get_success_rate(&action_id).await.unwrap();
        assert_eq!(rate, 1.0);

        // Record failure
        kb.record_success("Test situation", action_id, false)
            .await
            .unwrap();

        // Check updated rate
        let rate = kb.get_success_rate(&action_id).await.unwrap();
        assert_eq!(rate, 0.5);
    }
}
