//! Timer service for workflow engine
//!
//! Provides timer management for transient triggers (P30) and persistent triggers (P31).

use crate::patterns::PatternId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

/// Timer fired event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerFired {
    /// Pattern ID (P30 or P31)
    pub pattern_id: u32,
    /// Case ID
    pub case_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Timer key/identifier
    pub key: String,
    /// Fired timestamp
    pub fired_at: u64,
}

/// Timer service for managing workflow timers
pub struct TimerService {
    /// Channel for timer fired events
    tx: mpsc::Sender<TimerFired>,
}

impl TimerService {
    /// Create a new timer service
    pub fn new(tx: mpsc::Sender<TimerFired>) -> Self {
        Self { tx }
    }

    /// Schedule a transient trigger (P30)
    pub async fn schedule_transient(
        &self,
        case_id: String,
        workflow_id: String,
        key: String,
        due_at: Instant,
    ) -> Result<(), TimerError> {
        let pattern_id = PatternId::new(30).unwrap().0;
        self.schedule(pattern_id, case_id, workflow_id, key, due_at)
            .await
    }

    /// Schedule a persistent trigger (P31)
    pub async fn schedule_persistent(
        &self,
        case_id: String,
        workflow_id: String,
        key: String,
        due_at: Instant,
    ) -> Result<(), TimerError> {
        let pattern_id = PatternId::new(31).unwrap().0;
        self.schedule(pattern_id, case_id, workflow_id, key, due_at)
            .await
    }

    /// Schedule a timer
    async fn schedule(
        &self,
        pattern_id: u32,
        case_id: String,
        workflow_id: String,
        key: String,
        due_at: Instant,
    ) -> Result<(), TimerError> {
        // Spawn background task to wait until due_at
        let tx = self.tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep_until(due_at).await;
            let fired_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let event = TimerFired {
                pattern_id,
                case_id,
                workflow_id,
                key,
                fired_at,
            };

            let _ = tx.send(event).await;
        });

        Ok(())
    }
}

/// Timer error
#[derive(Debug, thiserror::Error)]
pub enum TimerError {
    #[error("Timer channel closed")]
    ChannelClosed,
    #[error("Invalid pattern ID")]
    InvalidPatternId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timer_service() {
        let (tx, mut rx) = mpsc::channel(10);
        let service = TimerService::new(tx);

        let due_at = Instant::now() + Duration::from_millis(10);
        service
            .schedule_transient(
                "case1".to_string(),
                "workflow1".to_string(),
                "key1".to_string(),
                due_at,
            )
            .await
            .unwrap();

        let fired = rx.recv().await.unwrap();
        assert_eq!(fired.pattern_id, 30);
        assert_eq!(fired.case_id, "case1");
        assert_eq!(fired.key, "key1");
    }
}
