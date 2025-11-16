//! Comprehensive Raft consensus tests

use knhk_workflow_engine::consensus::*;
use std::time::Duration;

#[tokio::test]
async fn test_raft_cluster_initialization() {
    let cluster = RaftCluster::builder()
        .node_id(1)
        .peers(vec![])
        .heartbeat_interval(Duration::from_millis(50))
        .build()
        .unwrap();

    assert_eq!(cluster.role().await, RaftRole::Follower);
    assert_eq!(cluster.term().await.inner(), 0);
    assert_eq!(cluster.leader().await, None);
}

#[tokio::test]
async fn test_raft_start_stop() {
    let cluster = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    cluster.start().await.unwrap();
    assert_eq!(cluster.role().await, RaftRole::Follower);

    cluster.stop().await.unwrap();
}

#[tokio::test]
async fn test_raft_log_operations() {
    let cluster = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    // Start as follower - proposing should fail
    cluster.start().await.unwrap();

    let result = cluster.propose("test entry").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConsensusError::NotLeader { .. }));
}

#[tokio::test]
async fn test_raft_builder() {
    let peers = vec![
        "127.0.0.1:9001".parse().unwrap(),
        "127.0.0.1:9002".parse().unwrap(),
    ];

    let cluster = RaftCluster::builder()
        .node_id(1)
        .peers(peers)
        .heartbeat_interval(Duration::from_millis(50))
        .election_timeout(Duration::from_millis(150))
        .max_entries_per_rpc(100)
        .snapshot_threshold(10000)
        .build()
        .unwrap();

    assert_eq!(cluster.role().await, RaftRole::Follower);
}

#[tokio::test]
async fn test_raft_metrics() {
    let cluster = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    let metrics = cluster.metrics().await;
    assert_eq!(metrics.proposals_submitted, 0);
    assert_eq!(metrics.proposals_committed, 0);
    assert_eq!(metrics.leader_elections, 0);
}

#[tokio::test]
async fn test_raft_node_wrapper() {
    let cluster = RaftCluster::builder()
        .node_id(1)
        .build()
        .unwrap();

    let node = RaftNode::new(cluster);

    node.start().await.unwrap();
    assert_eq!(node.cluster().role().await, RaftRole::Follower);

    node.stop().await.unwrap();
}
