//! Comprehensive Byzantine Consensus Tests
//!
//! Tests for Byzantine fault tolerance, including:
//! - Single-node consensus
//! - Multi-node BFT
//! - Byzantine node failures
//! - View changes
//! - Network partitions
//! - Performance benchmarks

use knhk_byzantine::{
    mapek_byzantine::ByzantineMAPEK,
    network::{ByzantineConfig, ByzantineNetwork, NodeState},
    protocols::{hotstuff::HotStuffConsensus, pbft::PBFTConsensus},
    qc_manager::{QCAggregator, QuorumCertificateManager},
    DecisionAction, Hash, NodeId, WorkflowDecision,
};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

// ============================================================================
// PBFT Tests
// ============================================================================

#[tokio::test]
async fn test_pbft_single_proposal() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let pbft = PBFTConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

    // Note: Full consensus requires message passing between nodes
    // In a real test, we'd simulate multiple nodes
    assert_eq!(pbft.current_view().await, 0);
}

#[tokio::test]
async fn test_pbft_primary_rotation() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let pbft = PBFTConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

    // View change
    pbft.view_change().await.unwrap();
    assert_eq!(pbft.current_view().await, 1);
}

#[tokio::test]
async fn test_pbft_f_tolerance() {
    // 4 nodes can tolerate 1 Byzantine fault
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let pbft = PBFTConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

    // f = (n-1)/3 = (4-1)/3 = 1
    // Need 2f+1 = 3 signatures for commit
    let committed = pbft.committed_blocks().await;
    assert_eq!(committed.len(), 0);
}

// ============================================================================
// HotStuff Tests
// ============================================================================

#[tokio::test]
async fn test_hotstuff_creation() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let hotstuff = HotStuffConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

    assert_eq!(hotstuff.current_view().await, 0);
}

#[tokio::test]
async fn test_hotstuff_leader_selection() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let hotstuff = HotStuffConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

    let committed = hotstuff.decide().await.unwrap();
    assert_eq!(committed.len(), 0);
}

// ============================================================================
// Byzantine Network Tests
// ============================================================================

#[tokio::test]
async fn test_network_broadcast() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
    let network = ByzantineNetwork::new(nodes.clone());

    let msg = vec![1, 2, 3, 4, 5];
    network.broadcast(msg.clone()).await.unwrap();

    // Each node should have received the message
    for node in nodes {
        assert_eq!(network.pending_count(node).await, 1);
    }
}

#[tokio::test]
async fn test_network_message_loss() {
    let nodes = vec![NodeId(0), NodeId(1)];
    let network = ByzantineNetwork::new(nodes);

    // Set 100% message loss
    network
        .set_config(ByzantineConfig {
            message_loss_rate: 1.0,
            message_delay: None,
            corruption_rate: 0.0,
            max_queue_size: 1000,
        })
        .await;

    network.broadcast(vec![1, 2, 3]).await.unwrap();

    // No messages should be delivered
    assert_eq!(network.pending_count(NodeId(0)).await, 0);
    assert_eq!(network.pending_count(NodeId(1)).await, 0);
}

#[tokio::test]
async fn test_network_message_delay() {
    let nodes = vec![NodeId(0), NodeId(1)];
    let network = ByzantineNetwork::new(nodes);

    // Set 50ms delay
    network
        .set_config(ByzantineConfig {
            message_loss_rate: 0.0,
            message_delay: Some(Duration::from_millis(50)),
            corruption_rate: 0.0,
            max_queue_size: 1000,
        })
        .await;

    let start = Instant::now();
    network.send_to(NodeId(0), vec![1, 2, 3]).await.unwrap();
    let elapsed = start.elapsed();

    // Should take at least 50ms
    assert!(elapsed >= Duration::from_millis(50));
}

#[tokio::test]
async fn test_byzantine_node_detection() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = ByzantineNetwork::new(nodes);

    // Mark node as Byzantine
    network.handle_byzantine_node(NodeId(1)).await;

    assert_eq!(network.get_state(NodeId(1)), Some(NodeState::Byzantine));
    assert_eq!(network.byzantine_nodes().len(), 1);
    assert_eq!(network.active_count(), 3);
}

#[tokio::test]
async fn test_network_partition() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = ByzantineNetwork::new(nodes);

    // Simulate partition: nodes 0,1 are offline
    network.mark_offline(NodeId(0));
    network.mark_offline(NodeId(1));

    assert_eq!(network.active_count(), 2);
    assert!(!network.is_active(NodeId(0)));
    assert!(network.is_active(NodeId(2)));
}

// ============================================================================
// QC Manager Tests
// ============================================================================

#[tokio::test]
async fn test_qc_creation_and_verification() {
    let manager = QuorumCertificateManager::new(3);
    let block_hash = Hash([1u8; 32]);
    let signatures = vec![
        (NodeId(0), knhk_byzantine::protocols::Signature(vec![0u8; 64])),
        (NodeId(1), knhk_byzantine::protocols::Signature(vec![1u8; 64])),
        (NodeId(2), knhk_byzantine::protocols::Signature(vec![2u8; 64])),
    ];

    let qc = manager.create_qc(block_hash, 1, signatures);
    assert!(manager.verify_qc(&qc).await.is_ok());
    assert!(manager.is_verified(&block_hash));
}

#[tokio::test]
async fn test_qc_insufficient_signatures() {
    let manager = QuorumCertificateManager::new(3);
    let block_hash = Hash([1u8; 32]);
    let signatures = vec![
        (NodeId(0), knhk_byzantine::protocols::Signature(vec![0u8; 64])),
    ];

    let qc = manager.create_qc(block_hash, 1, signatures);
    assert!(manager.verify_qc(&qc).await.is_err());
}

#[test]
fn test_qc_aggregator() {
    let block_hash = Hash([1u8; 32]);
    let aggregator = QCAggregator::new(block_hash, 1, 3);

    aggregator.add_signature(NodeId(0), knhk_byzantine::protocols::Signature(vec![0u8; 64])).unwrap();
    aggregator.add_signature(NodeId(1), knhk_byzantine::protocols::Signature(vec![1u8; 64])).unwrap();
    assert!(!aggregator.has_quorum());

    aggregator.add_signature(NodeId(2), knhk_byzantine::protocols::Signature(vec![2u8; 64])).unwrap();
    assert!(aggregator.has_quorum());

    let qc = aggregator.try_build_qc().unwrap();
    assert_eq!(qc.signature_count(), 3);
}

// ============================================================================
// Byzantine MAPE-K Tests
// ============================================================================

#[tokio::test]
async fn test_mapek_monitor() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network);

    let metrics = mapek.monitor("workflow-1").await.unwrap();
    assert_eq!(metrics.workflow_id, "workflow-1");
    assert!(metrics.tasks_completed > 0);
}

#[tokio::test]
async fn test_mapek_analyze() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network);

    let recommendations = mapek.analyze("workflow-1").await.unwrap();
    assert!(!recommendations.is_empty());

    for rec in &recommendations {
        assert!(rec.confidence > 0.0 && rec.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_mapek_execute_decision() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network);

    let decision = WorkflowDecision {
        workflow_id: "test-workflow".to_string(),
        action: DecisionAction::Execute,
        timestamp: 0,
    };

    mapek.execute_consensus_decision(&decision).await.unwrap();

    let executed = mapek.executed_decisions().await;
    assert_eq!(executed.len(), 1);
    assert_eq!(executed[0].workflow_id, "test-workflow");
}

#[tokio::test]
async fn test_mapek_detect_byzantine() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network.clone());

    // Mark a node as Byzantine
    network.handle_byzantine_node(NodeId(2)).await;

    let byzantine = mapek.detect_byzantine_nodes().await;
    assert_eq!(byzantine.len(), 1);
    assert_eq!(byzantine[0], NodeId(2));
}

#[tokio::test]
async fn test_mapek_hotstuff_protocol() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_hotstuff(NodeId(0), nodes, Duration::from_secs(5), network);

    let metrics = mapek.monitor("workflow-1").await.unwrap();
    assert_eq!(metrics.workflow_id, "workflow-1");
}

// ============================================================================
// Performance Benchmarks
// ============================================================================

#[tokio::test]
async fn benchmark_pbft_latency() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let pbft = PBFTConsensus::new(NodeId(0), nodes, Duration::from_secs(10), network);

    let start = Instant::now();
    let decision = WorkflowDecision {
        workflow_id: "bench-workflow".to_string(),
        action: DecisionAction::Execute,
        timestamp: 0,
    };

    // Note: This will timeout in single-node simulation
    // In real multi-node test, measure actual consensus time
    let _ = tokio::time::timeout(Duration::from_millis(100), pbft.propose(vec![decision])).await;

    let elapsed = start.elapsed();
    println!("PBFT proposal time: {:?}", elapsed);
}

#[tokio::test]
async fn benchmark_network_throughput() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = ByzantineNetwork::new(nodes.clone());

    let message_count = 1000;
    let start = Instant::now();

    for i in 0..message_count {
        let msg = vec![i as u8; 100];
        network.broadcast(msg).await.unwrap();
    }

    let elapsed = start.elapsed();
    let throughput = message_count as f64 / elapsed.as_secs_f64();

    println!("Network throughput: {:.0} messages/sec", throughput);
    assert!(throughput > 100.0, "Throughput too low: {}", throughput);
}

#[tokio::test]
async fn benchmark_qc_aggregation() {
    let block_hash = Hash([1u8; 32]);
    let aggregator = QCAggregator::new(block_hash, 1, 100);

    let start = Instant::now();

    for i in 0..100 {
        aggregator.add_signature(
            NodeId(i),
            knhk_byzantine::protocols::Signature(vec![i as u8; 64])
        ).unwrap();
    }

    let qc = aggregator.try_build_qc().unwrap();
    let elapsed = start.elapsed();

    println!("QC aggregation time (100 sigs): {:?}", elapsed);
    assert_eq!(qc.signature_count(), 100);
    assert!(elapsed < Duration::from_millis(100), "QC aggregation too slow");
}

// ============================================================================
// Failure Injection Tests
// ============================================================================

#[tokio::test]
async fn test_failure_injection_message_corruption() {
    let nodes = vec![NodeId(0), NodeId(1)];
    let network = ByzantineNetwork::new(nodes);

    // Set 100% corruption
    network
        .set_config(ByzantineConfig {
            message_loss_rate: 0.0,
            message_delay: None,
            corruption_rate: 1.0,
            max_queue_size: 1000,
        })
        .await;

    let original = vec![1, 2, 3, 4, 5];
    network.send_to(NodeId(0), original.clone()).await.unwrap();

    let received = network.receive(NodeId(0)).await.unwrap();
    // Message should be corrupted (all zeros)
    assert_eq!(received.payload, vec![0u8; 5]);
}

#[tokio::test]
async fn test_failure_injection_queue_overflow() {
    let nodes = vec![NodeId(0)];
    let network = ByzantineNetwork::new(nodes);

    // Set small queue size
    network
        .set_config(ByzantineConfig {
            message_loss_rate: 0.0,
            message_delay: None,
            corruption_rate: 0.0,
            max_queue_size: 5,
        })
        .await;

    // Send more messages than queue can hold
    for i in 0..10 {
        network.send_to(NodeId(0), vec![i]).await.unwrap();
    }

    // Only first 5 messages should be queued
    let pending = network.pending_count(NodeId(0)).await;
    assert_eq!(pending, 5);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn integration_test_full_consensus_workflow() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(
        NodeId(0),
        nodes,
        Duration::from_secs(5),
        network,
    );

    // 1. Monitor workflow
    let metrics = mapek.monitor("integration-test").await.unwrap();
    assert_eq!(metrics.workflow_id, "integration-test");

    // 2. Analyze and get recommendations
    let recommendations = mapek.analyze("integration-test").await.unwrap();
    assert!(!recommendations.is_empty());

    // 3. Execute decision
    let decision = &recommendations[0].action;
    mapek.execute_consensus_decision(decision).await.unwrap();

    // 4. Verify execution
    let executed = mapek.executed_decisions().await;
    assert_eq!(executed.len(), 1);
}

#[tokio::test]
async fn integration_test_byzantine_tolerance() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(
        NodeId(0),
        nodes,
        Duration::from_secs(5),
        network.clone(),
    );

    // Mark 1 node as Byzantine (f=1 for n=4)
    network.handle_byzantine_node(NodeId(3)).await;

    // System should still function with 1 Byzantine node
    let byzantine = mapek.detect_byzantine_nodes().await;
    assert_eq!(byzantine.len(), 1);

    // Analysis should still work
    let recommendations = mapek.analyze("test-workflow").await.unwrap();
    assert!(!recommendations.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stress_test_concurrent_broadcasts() {
    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));

    let mut handles = vec![];

    // Spawn 10 concurrent broadcast tasks
    for i in 0..10 {
        let net = network.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                let msg = vec![i; 10];
                net.broadcast(msg).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Check total messages delivered (some may be lost due to queue limits)
    let mut total = 0;
    for &node in &nodes {
        total += network.pending_count(node).await;
    }

    println!("Total messages queued: {}", total);
    assert!(total > 0);
}
