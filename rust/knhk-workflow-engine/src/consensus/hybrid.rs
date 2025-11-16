//! Hybrid consensus protocol dispatcher
//!
//! Automatically selects between Raft and BFT based on threat model:
//! - **Raft**: Fast crash fault tolerance for trusted environments
//! - **BFT**: Byzantine fault tolerance for untrusted/multi-party environments
//!
//! # Threat Model Detection
//!
//! The hybrid protocol monitors for:
//! - Message tampering
//! - Signature verification failures
//! - Equivocation (sending conflicting messages)
//! - Timing attacks
//!
//! When Byzantine behavior is detected, it automatically falls back to BFT.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Threat level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// No threats detected - use Raft
    None,

    /// Low threat - monitor closely
    Low,

    /// Medium threat - consider BFT
    Medium,

    /// High threat - use BFT
    High,

    /// Critical threat - Byzantine behavior detected
    Critical,
}

impl ThreatLevel {
    /// Check if we should use BFT
    pub fn should_use_bft(&self) -> bool {
        matches!(self, ThreatLevel::High | ThreatLevel::Critical)
    }

    /// Check if we should fall back to BFT
    pub fn should_fallback_to_bft(&self) -> bool {
        matches!(self, ThreatLevel::Critical)
    }
}

/// Threat model for hybrid consensus
pub struct ThreatModel {
    /// Signature verification failures
    signature_failures: AtomicU64,

    /// Message tampering incidents
    tampering_incidents: AtomicU64,

    /// Equivocation detections
    equivocations: AtomicU64,

    /// Timing anomalies
    timing_anomalies: AtomicU64,

    /// Current threat level
    threat_level: Arc<RwLock<ThreatLevel>>,
}

impl ThreatModel {
    /// Create a new threat model
    pub fn new() -> Self {
        Self {
            signature_failures: AtomicU64::new(0),
            tampering_incidents: AtomicU64::new(0),
            equivocations: AtomicU64::new(0),
            timing_anomalies: AtomicU64::new(0),
            threat_level: Arc::new(RwLock::new(ThreatLevel::None)),
        }
    }

    /// Record a signature verification failure
    pub async fn record_signature_failure(&self) {
        let count = self.signature_failures.fetch_add(1, Ordering::SeqCst) + 1;

        warn!(count, "Signature verification failure detected");

        self.update_threat_level(count, 5).await;
    }

    /// Record message tampering
    pub async fn record_tampering(&self) {
        let count = self.tampering_incidents.fetch_add(1, Ordering::SeqCst) + 1;

        error!(count, "Message tampering detected");

        // Tampering is critical - immediately elevate to Critical
        *self.threat_level.write().await = ThreatLevel::Critical;
    }

    /// Record equivocation
    pub async fn record_equivocation(&self) {
        let count = self.equivocations.fetch_add(1, Ordering::SeqCst) + 1;

        error!(count, "Equivocation detected");

        // Equivocation is Byzantine behavior - immediately elevate to Critical
        *self.threat_level.write().await = ThreatLevel::Critical;
    }

    /// Record timing anomaly
    pub async fn record_timing_anomaly(&self) {
        let count = self.timing_anomalies.fetch_add(1, Ordering::SeqCst) + 1;

        debug!(count, "Timing anomaly detected");

        self.update_threat_level(count, 10).await;
    }

    /// Update threat level based on incident count
    async fn update_threat_level(&self, count: u64, threshold: u64) {
        let level = match count {
            0 => ThreatLevel::None,
            1..=2 => ThreatLevel::Low,
            3..=5 => ThreatLevel::Medium,
            6..=10 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        };

        *self.threat_level.write().await = level;
    }

    /// Get current threat level
    pub async fn threat_level(&self) -> ThreatLevel {
        *self.threat_level.read().await
    }

    /// Reset threat model
    pub async fn reset(&self) {
        self.signature_failures.store(0, Ordering::SeqCst);
        self.tampering_incidents.store(0, Ordering::SeqCst);
        self.equivocations.store(0, Ordering::SeqCst);
        self.timing_anomalies.store(0, Ordering::SeqCst);
        *self.threat_level.write().await = ThreatLevel::None;
    }
}

impl Default for ThreatModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Hybrid consensus combining Raft and BFT
pub struct HybridConsensus {
    /// Raft cluster
    raft: Arc<RaftCluster>,

    /// BFT cluster
    bft: Arc<BftCluster>,

    /// Threat model
    threat_model: Arc<ThreatModel>,

    /// Current protocol in use
    current_protocol: Arc<RwLock<ConsensusProtocol>>,

    /// Metrics
    metrics: Arc<RwLock<ConsensusMetrics>>,
}

/// Consensus protocol selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConsensusProtocol {
    /// Using Raft
    Raft,

    /// Using BFT
    Bft,
}

impl HybridConsensus {
    /// Create a new hybrid consensus
    pub fn new(raft: RaftCluster, bft: BftCluster) -> Self {
        Self {
            raft: Arc::new(raft),
            bft: Arc::new(bft),
            threat_model: Arc::new(ThreatModel::new()),
            current_protocol: Arc::new(RwLock::new(ConsensusProtocol::Raft)),
            metrics: Arc::new(RwLock::new(ConsensusMetrics::new())),
        }
    }

    /// Propose a value with automatic threat detection
    pub async fn propose_with_threat_detection<T: Serialize>(
        &self,
        value: T,
    ) -> ConsensusResult<LogIndex> {
        let start = Instant::now();

        // Check threat level
        let threat_level = self.threat_model.threat_level().await;

        // Select protocol based on threat level
        let protocol = if threat_level.should_use_bft() {
            ConsensusProtocol::Bft
        } else {
            ConsensusProtocol::Raft
        };

        // Switch protocol if needed
        self.switch_protocol(protocol).await;

        // Execute consensus
        let result = match protocol {
            ConsensusProtocol::Raft => {
                info!("Using Raft consensus (threat level: {:?})", threat_level);
                self.raft.propose(value).await
            }
            ConsensusProtocol::Bft => {
                info!("Using BFT consensus (threat level: {:?})", threat_level);
                // BFT returns Decision, convert to LogIndex
                let decision = self.bft.propose(value).await?;
                // Map sequence number to log index
                Ok(LogIndex::new(decision.sequence.inner()))
            }
        };

        // Record metrics
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut metrics = self.metrics.write().await;
        metrics.record_proposal(latency_ms, result.is_ok());

        result
    }

    /// Switch to a different consensus protocol
    async fn switch_protocol(&self, new_protocol: ConsensusProtocol) {
        let mut current = self.current_protocol.write().await;

        if *current != new_protocol {
            match new_protocol {
                ConsensusProtocol::Raft => {
                    info!("Switching to Raft consensus");
                }
                ConsensusProtocol::Bft => {
                    warn!("Switching to BFT consensus due to threat detection");
                    let mut metrics = self.metrics.write().await;
                    metrics.record_byzantine_failure();
                }
            }

            *current = new_protocol;
        }
    }

    /// Get current protocol
    pub async fn current_protocol(&self) -> String {
        match *self.current_protocol.read().await {
            ConsensusProtocol::Raft => "Raft".to_string(),
            ConsensusProtocol::Bft => "BFT".to_string(),
        }
    }

    /// Get threat model
    pub fn threat_model(&self) -> Arc<ThreatModel> {
        Arc::clone(&self.threat_model)
    }

    /// Get metrics
    pub async fn metrics(&self) -> ConsensusMetrics {
        self.metrics.read().await.clone()
    }

    /// Force BFT mode
    pub async fn force_bft_mode(&self) {
        self.switch_protocol(ConsensusProtocol::Bft).await;
    }

    /// Force Raft mode (only if threat level is low)
    pub async fn force_raft_mode(&self) -> ConsensusResult<()> {
        let threat_level = self.threat_model.threat_level().await;

        if threat_level.should_use_bft() {
            return Err(ConsensusError::Internal(
                "Cannot use Raft mode with high threat level".to_string(),
            ));
        }

        self.switch_protocol(ConsensusProtocol::Raft).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_threat_model_escalation() {
        let model = ThreatModel::new();

        assert_eq!(model.threat_level().await, ThreatLevel::None);

        // Record signature failures
        model.record_signature_failure().await;
        assert_eq!(model.threat_level().await, ThreatLevel::Low);

        model.record_signature_failure().await;
        model.record_signature_failure().await;
        assert_eq!(model.threat_level().await, ThreatLevel::Medium);

        // Tampering immediately elevates to Critical
        model.record_tampering().await;
        assert_eq!(model.threat_level().await, ThreatLevel::Critical);
        assert!(model.threat_level().await.should_use_bft());
    }

    #[tokio::test]
    async fn test_threat_model_equivocation() {
        let model = ThreatModel::new();

        model.record_equivocation().await;
        assert_eq!(model.threat_level().await, ThreatLevel::Critical);
        assert!(model.threat_level().await.should_fallback_to_bft());
    }

    #[tokio::test]
    async fn test_threat_model_reset() {
        let model = ThreatModel::new();

        model.record_signature_failure().await;
        model.record_signature_failure().await;
        assert_eq!(model.threat_level().await, ThreatLevel::Low);

        model.reset().await;
        assert_eq!(model.threat_level().await, ThreatLevel::None);
    }
}
