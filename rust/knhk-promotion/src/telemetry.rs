//! Telemetry and observability for promotion pipeline

use knhk_ontology::SigmaSnapshotId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

/// Promotion phase identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromotionPhase {
    /// Validation phase
    Validation,
    /// Compilation phase
    Compilation,
    /// Guard creation phase
    GuardCreation,
    /// Ready transition phase
    Ready,
    /// Atomic promotion phase
    AtomicPromotion,
}

impl PromotionPhase {
    /// Get phase name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            PromotionPhase::Validation => "validation",
            PromotionPhase::Compilation => "compilation",
            PromotionPhase::GuardCreation => "guard_creation",
            PromotionPhase::Ready => "ready",
            PromotionPhase::AtomicPromotion => "atomic_promotion",
        }
    }
}

impl From<&str> for PromotionPhase {
    fn from(s: &str) -> Self {
        match s {
            "validation" => PromotionPhase::Validation,
            "compilation" => PromotionPhase::Compilation,
            "guard_creation" => PromotionPhase::GuardCreation,
            "ready" => PromotionPhase::Ready,
            "atomic_promotion" => PromotionPhase::AtomicPromotion,
            _ => PromotionPhase::Validation, // Default
        }
    }
}

/// Telemetry data for a single promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionTelemetryData {
    /// Snapshot that was promoted
    pub snapshot_id: SigmaSnapshotId,

    /// When promotion started
    pub started_at: SystemTime,

    /// When promotion completed
    pub completed_at: Option<SystemTime>,

    /// Total duration
    pub duration: Option<Duration>,

    /// Phase durations
    pub phase_durations: HashMap<String, Duration>,

    /// Whether promotion succeeded
    pub succeeded: bool,

    /// Error message if failed
    pub error: Option<String>,
}

/// Promotion telemetry tracker
pub struct PromotionTelemetry {
    /// Current promotion being tracked
    current: RwLock<Option<PromotionTelemetryData>>,

    /// Phase start times
    phase_starts: RwLock<HashMap<String, SystemTime>>,

    /// Historical promotions
    history: RwLock<Vec<PromotionTelemetryData>>,
}

impl PromotionTelemetry {
    /// Create new telemetry tracker
    pub fn new() -> Self {
        Self {
            current: RwLock::new(None),
            phase_starts: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
        }
    }

    /// Start tracking a new promotion
    pub fn start_promotion(&self, snapshot_id: SigmaSnapshotId) {
        let data = PromotionTelemetryData {
            snapshot_id,
            started_at: SystemTime::now(),
            completed_at: None,
            duration: None,
            phase_durations: HashMap::new(),
            succeeded: false,
            error: None,
        };

        *self.current.write() = Some(data);
        debug!("Started tracking promotion for {:?}", snapshot_id);
    }

    /// Record the start of a phase
    pub fn record_phase_start(&self, phase: &str) {
        self.phase_starts.write().insert(
            phase.to_string(),
            SystemTime::now(),
        );
        debug!("Phase started: {}", phase);
    }

    /// Record the end of a phase
    pub fn record_phase_end(&self, phase: &str) {
        if let Some(start) = self.phase_starts.read().get(phase) {
            if let Ok(duration) = SystemTime::now().duration_since(*start) {
                if let Some(current) = self.current.write().as_mut() {
                    current.phase_durations.insert(phase.to_string(), duration);
                    info!(
                        phase = phase,
                        duration_ms = duration.as_millis(),
                        "Phase completed"
                    );
                }
            }
        }
    }

    /// Record successful promotion
    pub fn record_success(&self, snapshot_id: SigmaSnapshotId, duration: Duration) {
        if let Some(current) = self.current.write().as_mut() {
            current.completed_at = Some(SystemTime::now());
            current.duration = Some(duration);
            current.succeeded = true;

            info!(
                snapshot_id = ?snapshot_id,
                duration_ms = duration.as_millis(),
                "Promotion succeeded"
            );

            // Archive to history
            let data = current.clone();
            self.history.write().push(data);
        }
    }

    /// Record failed promotion
    pub fn record_failure(&self, snapshot_id: SigmaSnapshotId, error: String) {
        if let Some(current) = self.current.write().as_mut() {
            current.completed_at = Some(SystemTime::now());
            current.succeeded = false;
            current.error = Some(error.clone());

            if let Ok(duration) = SystemTime::now().duration_since(current.started_at) {
                current.duration = Some(duration);
            }

            info!(
                snapshot_id = ?snapshot_id,
                error = error,
                "Promotion failed"
            );

            // Archive to history
            let data = current.clone();
            self.history.write().push(data);
        }
    }

    /// Get current promotion data
    pub fn current(&self) -> Option<PromotionTelemetryData> {
        self.current.read().clone()
    }

    /// Get promotion history
    pub fn history(&self) -> Vec<PromotionTelemetryData> {
        self.history.read().clone()
    }

    /// Get statistics
    pub fn stats(&self) -> PromotionStats {
        let history = self.history.read();

        let total = history.len();
        let succeeded = history.iter().filter(|d| d.succeeded).count();
        let failed = total - succeeded;

        let avg_duration = if !history.is_empty() {
            let sum: Duration = history
                .iter()
                .filter_map(|d| d.duration)
                .sum();
            Some(sum / total as u32)
        } else {
            None
        };

        PromotionStats {
            total_promotions: total,
            successful_promotions: succeeded,
            failed_promotions: failed,
            average_duration: avg_duration,
        }
    }
}

impl Default for PromotionTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

/// Promotion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionStats {
    /// Total number of promotions
    pub total_promotions: usize,

    /// Number of successful promotions
    pub successful_promotions: usize,

    /// Number of failed promotions
    pub failed_promotions: usize,

    /// Average promotion duration
    pub average_duration: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_as_str() {
        assert_eq!(PromotionPhase::Validation.as_str(), "validation");
        assert_eq!(PromotionPhase::Compilation.as_str(), "compilation");
        assert_eq!(PromotionPhase::AtomicPromotion.as_str(), "atomic_promotion");
    }

    #[test]
    fn test_phase_from_str() {
        assert_eq!(PromotionPhase::from("validation"), PromotionPhase::Validation);
        assert_eq!(PromotionPhase::from("compilation"), PromotionPhase::Compilation);
    }

    #[test]
    fn test_telemetry_lifecycle() {
        let telemetry = PromotionTelemetry::new();
        let snapshot_id = [1u8; 32];

        // Start promotion
        telemetry.start_promotion(snapshot_id);
        assert!(telemetry.current().is_some());

        // Record phases
        telemetry.record_phase_start("validation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        telemetry.record_phase_end("validation");

        telemetry.record_phase_start("compilation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        telemetry.record_phase_end("compilation");

        // Record success
        telemetry.record_success(snapshot_id, Duration::from_millis(20));

        // Check stats
        let stats = telemetry.stats();
        assert_eq!(stats.total_promotions, 1);
        assert_eq!(stats.successful_promotions, 1);
        assert_eq!(stats.failed_promotions, 0);
    }

    #[test]
    fn test_telemetry_failure() {
        let telemetry = PromotionTelemetry::new();
        let snapshot_id = [2u8; 32];

        telemetry.start_promotion(snapshot_id);
        telemetry.record_failure(snapshot_id, "Test error".to_string());

        let stats = telemetry.stats();
        assert_eq!(stats.total_promotions, 1);
        assert_eq!(stats.successful_promotions, 0);
        assert_eq!(stats.failed_promotions, 1);
    }
}
