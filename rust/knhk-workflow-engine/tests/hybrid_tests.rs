//! Hybrid consensus protocol tests

use knhk_workflow_engine::consensus::*;
use knhk_workflow_engine::consensus::bft::*;

#[tokio::test]
async fn test_threat_model() {
    let model = ThreatModel::new();

    assert_eq!(model.threat_level().await, ThreatLevel::None);

    // Record signature failures
    model.record_signature_failure().await;
    assert_eq!(model.threat_level().await, ThreatLevel::Low);

    model.record_signature_failure().await;
    model.record_signature_failure().await;
    assert_eq!(model.threat_level().await, ThreatLevel::Medium);

    // Tampering immediately escalates to Critical
    model.record_tampering().await;
    assert_eq!(model.threat_level().await, ThreatLevel::Critical);
}

#[tokio::test]
async fn test_threat_model_equivocation() {
    let model = ThreatModel::new();

    // Equivocation is Byzantine behavior
    model.record_equivocation().await;
    assert_eq!(model.threat_level().await, ThreatLevel::Critical);
    assert!(model.threat_level().await.should_use_bft());
}

#[tokio::test]
async fn test_threat_model_timing_anomalies() {
    let model = ThreatModel::new();

    // Multiple timing anomalies
    for _ in 0..3 {
        model.record_timing_anomaly().await;
    }

    assert_eq!(model.threat_level().await, ThreatLevel::Medium);
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

#[tokio::test]
async fn test_threat_level_bft_decision() {
    assert!(!ThreatLevel::None.should_use_bft());
    assert!(!ThreatLevel::Low.should_use_bft());
    assert!(!ThreatLevel::Medium.should_use_bft());
    assert!(ThreatLevel::High.should_use_bft());
    assert!(ThreatLevel::Critical.should_use_bft());
}

#[tokio::test]
async fn test_threat_level_fallback() {
    assert!(!ThreatLevel::None.should_fallback_to_bft());
    assert!(!ThreatLevel::Low.should_fallback_to_bft());
    assert!(!ThreatLevel::Medium.should_fallback_to_bft());
    assert!(!ThreatLevel::High.should_fallback_to_bft());
    assert!(ThreatLevel::Critical.should_fallback_to_bft());
}

#[tokio::test]
async fn test_hybrid_consensus_creation() {
    let raft = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    let crypto = CryptoProvider::new();
    let bft = BftCluster::new_hotstuff(NodeId::new(1), vec![], crypto).unwrap();

    let hybrid = HybridConsensus::new(raft, bft);

    assert_eq!(hybrid.current_protocol().await, "Raft");
}

#[tokio::test]
async fn test_hybrid_consensus_protocol_switch() {
    let raft = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    let crypto = CryptoProvider::new();
    let bft = BftCluster::new_hotstuff(NodeId::new(1), vec![], crypto).unwrap();

    let hybrid = HybridConsensus::new(raft, bft);

    // Initially using Raft
    assert_eq!(hybrid.current_protocol().await, "Raft");

    // Force BFT mode
    hybrid.force_bft_mode().await;
    assert_eq!(hybrid.current_protocol().await, "BFT");

    // Reset threat model and force Raft mode
    hybrid.threat_model().reset().await;
    hybrid.force_raft_mode().await.unwrap();
    assert_eq!(hybrid.current_protocol().await, "Raft");
}

#[tokio::test]
async fn test_hybrid_consensus_threat_detection() {
    let raft = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    let crypto = CryptoProvider::new();
    let bft = BftCluster::new_hotstuff(NodeId::new(1), vec![], crypto).unwrap();

    let hybrid = HybridConsensus::new(raft, bft);

    // Detect Byzantine behavior
    hybrid.threat_model().record_equivocation().await;

    // Should not be able to force Raft mode with high threat
    let result = hybrid.force_raft_mode().await;
    assert!(result.is_err());
}
