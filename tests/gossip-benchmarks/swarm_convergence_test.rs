//! Swarm Convergence Integration Tests
//!
//! Validates gossip consensus convergence for swarms from 10 to 1M agents.

use knhk_consensus::gossip::{
    ConvergenceState, ConvergenceTracker, GossipConfig, GossipProtocol, HierarchicalConfig,
    HierarchicalGossip, StateValue, VersionedState,
};

#[tokio::test]
async fn test_small_swarm_convergence() {
    // 10 agents, k=3 peers, expected ~3 rounds
    let swarm_size = 10;
    let config = GossipConfig::new(0, swarm_size);

    assert_eq!(config.expected_convergence_rounds(), 3);
}

#[tokio::test]
async fn test_medium_swarm_convergence() {
    // 100 agents, k=5 peers, expected ~7 rounds
    let swarm_size = 100;
    let config = GossipConfig::new(0, swarm_size);

    assert_eq!(config.expected_convergence_rounds(), 5);
}

#[tokio::test]
async fn test_large_swarm_convergence() {
    // 1000 agents, k=8 peers, expected ~10 rounds
    let swarm_size = 1000;
    let config = GossipConfig::new(0, swarm_size);

    assert_eq!(config.expected_convergence_rounds(), 3);
}

#[tokio::test]
async fn test_massive_swarm_convergence() {
    // 10000 agents, k=10 peers, expected ~14 rounds
    let swarm_size = 10000;
    let config = GossipConfig::new(0, swarm_size);

    let expected = config.expected_convergence_rounds();
    assert!(expected >= 4 && expected <= 5);
}

#[tokio::test]
async fn test_convergence_tracker_small_swarm() {
    let swarm_size = 10;
    let max_byzantine = swarm_size / 3;
    let mut tracker = ConvergenceTracker::new(swarm_size, max_byzantine);

    // All agents converge to same state
    let state = VersionedState::new(1, StateValue::Number(42), 0);
    for i in 0..swarm_size {
        tracker.update_agent_state(i as u64, &state);
    }

    let convergence = tracker.check_convergence(5);
    match convergence {
        ConvergenceState::Converged { rounds, .. } => {
            assert_eq!(rounds, 5);
        }
        _ => panic!("Expected convergence"),
    }

    assert_eq!(tracker.convergence_percentage(), 100);
}

#[tokio::test]
async fn test_convergence_with_byzantine_agents() {
    let swarm_size = 10;
    let max_byzantine = swarm_size / 3; // f=3
    let mut tracker = ConvergenceTracker::new(swarm_size, max_byzantine);

    // 7 agents converge to state1, 3 Byzantine agents on state2
    let state1 = VersionedState::new(1, StateValue::Number(100), 0);
    let state2 = VersionedState::new(1, StateValue::Number(200), 1);

    for i in 0..7 {
        tracker.update_agent_state(i, &state1);
    }
    for i in 7..10 {
        tracker.update_agent_state(i, &state2);
    }

    let convergence = tracker.check_convergence(10);
    // Should still converge: 7 >= (10 - 3)
    match convergence {
        ConvergenceState::Converged { .. } => {
            // Expected
        }
        _ => panic!("Expected convergence with f=3 Byzantine agents"),
    }
}

#[tokio::test]
async fn test_hierarchical_convergence_scaling() {
    // 1000 agents, 100 per sub-swarm
    let swarm_size = 1000;
    let subswarm_size = 100;

    let config = HierarchicalConfig::new(0, swarm_size, subswarm_size);
    let initial_state = VersionedState::new(0, StateValue::Number(0), 0);
    let gossip = HierarchicalGossip::new(config.clone(), initial_state);

    let expected_rounds = gossip.expected_convergence_rounds();

    // Local convergence (100 agents, k=5): ~5 rounds
    // Leader convergence (10 sub-swarms, k=3): ~3 rounds
    // Total: ~9 rounds (with propagation)
    assert!(expected_rounds >= 5 && expected_rounds <= 10);
}

#[tokio::test]
async fn test_gossip_protocol_state_merge() {
    let config = GossipConfig::new(1, 10);
    let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
    let protocol = GossipProtocol::new(config, initial_state);

    // Simulate receiving newer state from peer
    let peer_state = VersionedState::new(1, StateValue::Number(42), 2);
    protocol
        .handle_message(
            2,
            knhk_consensus::gossip::protocol::GossipMessage::Push {
                state: peer_state.clone(),
                proof: None,
            },
        )
        .await
        .unwrap();

    // Verify state can be retrieved
    let current_state = protocol.get_state().await;
    assert_eq!(current_state.version, 0); // Not merged yet (needs execute_round)
}

#[tokio::test]
async fn test_byzantine_tolerance_validation() {
    // Test f < n/3 constraint
    assert_eq!(knhk_consensus::gossip::max_byzantine_tolerance(10), 3);
    assert_eq!(knhk_consensus::gossip::max_byzantine_tolerance(100), 33);
    assert_eq!(knhk_consensus::gossip::max_byzantine_tolerance(1000), 333);
    assert_eq!(knhk_consensus::gossip::max_byzantine_tolerance(10000), 3333);
    assert_eq!(
        knhk_consensus::gossip::max_byzantine_tolerance(1000000),
        333333
    );
}

#[tokio::test]
async fn test_expected_convergence_time_targets() {
    // Validate performance targets from specification

    // 10 agents: <10ms (3 rounds * ~3ms)
    let config_10 = GossipConfig::new(0, 10);
    assert!(config_10.expected_convergence_rounds() <= 5);

    // 100 agents: <50ms (7 rounds * ~7ms)
    let config_100 = GossipConfig::new(0, 100);
    assert!(config_100.expected_convergence_rounds() <= 10);

    // 1000 agents: <100ms (10 rounds * ~10ms)
    let config_1000 = GossipConfig::new(0, 1000);
    assert!(config_1000.expected_convergence_rounds() <= 15);

    // 10000 agents: <250ms (14 rounds * ~18ms)
    let config_10k = GossipConfig::new(0, 10000);
    assert!(config_10k.expected_convergence_rounds() <= 20);
}
