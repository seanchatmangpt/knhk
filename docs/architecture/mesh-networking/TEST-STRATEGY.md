# Mesh Networking Test Strategy

**Version**: 1.0.0 | **Date**: 2025-11-18

---

## Overview

This document defines the comprehensive testing strategy for KNHK distributed mesh networking, ensuring all components meet DOCTRINE requirements and scale from 10 to 1,000,000 agents.

## Test Pyramid

```
         ┌─────────────────────┐
         │  Chaos Engineering  │  (1% - 10 tests)
         │  - Partition tests  │
         │  - Byzantine tests  │
         └─────────────────────┘
              ┌──────────────────────────┐
              │  Integration Tests       │  (9% - 90 tests)
              │  - Multi-node gossip     │
              │  - Convergence testing   │
              └──────────────────────────┘
                   ┌─────────────────────────────┐
                   │  Chicago TDD Tests          │  (30% - 300 tests)
                   │  - Latency ≤8 ticks         │
                   │  - Performance validation   │
                   └─────────────────────────────┘
                        ┌──────────────────────────────────┐
                        │  Unit Tests                      │  (60% - 600 tests)
                        │  - Component isolation           │
                        │  - Edge cases                    │
                        └──────────────────────────────────┘
```

## Test Categories

### 1. Unit Tests (60% - ~600 tests)

**Coverage Target**: 95% line coverage

**Key Areas**:
- Peer registry operations
- Gossip message validation
- Topology manager logic
- Partition detector logic
- CRDT merge operations

**Example: Peer Registry Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[tokio::test]
    async fn test_register_peer() {
        // Arrange
        let registry = PeerRegistry::new(
            "node1".into(),
            5,  // quorum
            10, // fanout
            vec![],
        );

        let peer = PeerInfo {
            agent_id: "node2".into(),
            addr: "127.0.0.1:9000".parse().unwrap(),
            public_key: PublicKey::default(),
            last_seen: Instant::now(),
            reputation: 1.0,
            region: Region::UsEast1,
            latency_ms: 5.0,
        };

        // Act
        let result = registry.register_peer(peer.clone()).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(registry.peer_count(), 1);
    }

    #[tokio::test]
    async fn test_cannot_register_self() {
        // Arrange
        let registry = PeerRegistry::new(
            "node1".into(),
            5, 10, vec![],
        );

        let peer = PeerInfo {
            agent_id: "node1".into(), // Same as self
            ..Default::default()
        };

        // Act
        let result = registry.register_peer(peer).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_random_peers() {
        // Arrange
        let registry = setup_registry_with_10_peers().await;

        // Act
        let peers = registry.get_random_peers(5).await;

        // Assert
        assert_eq!(peers.len(), 5);
        // Verify randomness (run multiple times, should get different results)
    }

    #[tokio::test]
    async fn test_get_nearest_peers() {
        // Arrange
        let registry = setup_registry_with_latencies().await;

        // Act
        let peers = registry.get_nearest_peers(3).await;

        // Assert
        assert_eq!(peers.len(), 3);
        // Verify sorted by latency
        assert!(peers[0].latency_ms < peers[1].latency_ms);
        assert!(peers[1].latency_ms < peers[2].latency_ms);
    }

    #[test_case(0.5, 0.1, 0.6)]
    #[test_case(1.0, -0.2, 0.8)]
    #[test_case(0.3, -0.5, 0.0)] // Floor at 0.0
    #[tokio::test]
    async fn test_reputation_update(
        initial: f32,
        delta: f32,
        expected: f32,
    ) {
        let registry = setup_registry_with_peer().await;

        registry.update_reputation(&"peer1".into(), delta).await.unwrap();

        let peer = registry.get_peer(&"peer1".into()).unwrap();
        assert!((peer.reputation - expected).abs() < 0.01);
    }
}
```

### 2. Chicago TDD Tests (30% - ~300 tests)

**Purpose**: Validate performance constraints (Covenant 5: Chatman constant)

**Latency Requirements**:
- Gossip message processing: ≤8 ticks
- Signature verification: ≤2 ticks
- Peer selection: ≤1 tick
- CRDT merge: ≤3 ticks

**Example: Latency Tests**

```rust
#[cfg(test)]
mod chicago_tdd {
    use super::*;

    #[test]
    fn test_gossip_message_processing_latency() {
        // CRITICAL: This is a Covenant 5 test
        // Gossip message processing MUST be ≤8 ticks

        let coordinator = setup_gossip_coordinator();
        let message = create_test_gossip_message();

        // Measure in CPU ticks (not time)
        let start_tick = rdtsc();

        coordinator.process_message(message).unwrap();

        let end_tick = rdtsc();
        let elapsed_ticks = end_tick - start_tick;

        // ASSERT: Covenant 5
        assert!(
            elapsed_ticks <= 8,
            "Gossip processing took {} ticks (max 8)",
            elapsed_ticks
        );
    }

    #[test]
    fn test_signature_verification_latency() {
        // CRITICAL: Signature verification MUST be ≤2 ticks

        let validator = ByzantineValidator::new(peer_registry);
        let message = create_signed_message();

        let start_tick = rdtsc();

        validator.verify_signature(&message).unwrap();

        let end_tick = rdtsc();
        let elapsed_ticks = end_tick - start_tick;

        // ASSERT: ≤2 ticks
        assert!(
            elapsed_ticks <= 2,
            "Signature verification took {} ticks (max 2)",
            elapsed_ticks
        );
    }

    #[test]
    fn test_peer_selection_latency() {
        // Peer selection (DashMap lookup) MUST be ≤1 tick

        let registry = setup_large_registry(1000); // 1000 peers

        let start_tick = rdtsc();

        let peers = registry.get_random_peers(10);

        let end_tick = rdtsc();
        let elapsed_ticks = end_tick - start_tick;

        assert!(
            elapsed_ticks <= 1,
            "Peer selection took {} ticks (max 1)",
            elapsed_ticks
        );
    }

    #[test]
    fn test_crdt_merge_latency() {
        // CRDT merge MUST be ≤3 ticks

        let state1 = create_versioned_state();
        let delta = create_state_delta();

        let start_tick = rdtsc();

        state1.merge(delta).unwrap();

        let end_tick = rdtsc();
        let elapsed_ticks = end_tick - start_tick;

        assert!(
            elapsed_ticks <= 3,
            "CRDT merge took {} ticks (max 3)",
            elapsed_ticks
        );
    }

    // Helper: Read CPU Time Stamp Counter
    #[cfg(target_arch = "x86_64")]
    fn rdtsc() -> u64 {
        unsafe {
            core::arch::x86_64::_rdtsc()
        }
    }
}
```

### 3. Integration Tests (9% - ~90 tests)

**Purpose**: Validate multi-node behavior and convergence

**Key Scenarios**:
- Gossip convergence (10-1000 nodes)
- Multi-region communication
- Network partition and recovery
- Byzantine node detection

**Example: Convergence Tests**

```rust
#[tokio::test]
async fn test_gossip_convergence_10_nodes() {
    // Setup: 10 nodes, k=5 fanout
    let nodes = setup_mesh_network(10, 5).await;

    // Inject initial state into node[0]
    let initial_state = create_test_state();
    nodes[0].update_state(initial_state.clone()).await;

    // Run gossip for sufficient rounds
    let max_rounds = 10;
    run_gossip_rounds(&nodes, max_rounds).await;

    // Assert: All nodes converged to same state
    for node in &nodes {
        let state = node.get_state().await;
        assert_eq!(
            state.merkle_root,
            initial_state.merkle_root,
            "Node {} did not converge",
            node.id
        );
    }

    // Measure convergence time
    let convergence_rounds = measure_convergence_rounds(&nodes).await;
    assert!(
        convergence_rounds <= 7,
        "Convergence took {} rounds (expected ≤7 for 10 nodes)",
        convergence_rounds
    );
}

#[tokio::test]
async fn test_gossip_convergence_1000_nodes() {
    // Setup: 1000 nodes, k=10 fanout
    let nodes = setup_mesh_network(1000, 10).await;

    let initial_state = create_test_state();
    nodes[0].update_state(initial_state.clone()).await;

    let max_rounds = 20;
    run_gossip_rounds(&nodes, max_rounds).await;

    // Assert: All nodes converged
    for node in &nodes {
        let state = node.get_state().await;
        assert_eq!(state.merkle_root, initial_state.merkle_root);
    }

    // Measure convergence (should be O(log n))
    let convergence_rounds = measure_convergence_rounds(&nodes).await;
    let expected_rounds = (1000f64.log2() + 3.0) as usize; // log2(1000) + 3
    assert!(
        convergence_rounds <= expected_rounds,
        "Convergence took {} rounds (expected ≤{})",
        convergence_rounds,
        expected_rounds
    );
}

#[tokio::test]
async fn test_multi_region_gossip() {
    // Setup: 3 regions, 100 nodes each
    let us_east = setup_regional_mesh("us-east-1", 100, 10).await;
    let eu_west = setup_regional_mesh("eu-west-1", 100, 10).await;
    let ap_se = setup_regional_mesh("ap-southeast-1", 100, 10).await;

    // Setup leaders
    let us_leader = elect_leader(&us_east).await;
    let eu_leader = elect_leader(&eu_west).await;
    let ap_leader = elect_leader(&ap_se).await;

    let leaders = vec![us_leader, eu_leader, ap_leader];

    // Inject state into US region
    let initial_state = create_test_state();
    us_east[0].update_state(initial_state.clone()).await;

    // Run intra-region gossip
    run_regional_gossip(&us_east, 10).await;
    run_regional_gossip(&eu_west, 10).await;
    run_regional_gossip(&ap_se, 10).await;

    // Run inter-region leader gossip
    run_leader_gossip(&leaders, 5).await;

    // Assert: All regions converged
    for region in [&us_east, &eu_west, &ap_se] {
        for node in region {
            let state = node.get_state().await;
            assert_eq!(state.merkle_root, initial_state.merkle_root);
        }
    }
}
```

### 4. Chaos Engineering Tests (1% - ~10 tests)

**Purpose**: Validate fault tolerance under adversarial conditions

**Scenarios**:
- Network partition
- Byzantine nodes
- Message loss
- High latency
- Node failures

**Example: Partition Tests**

```rust
#[tokio::test]
async fn test_network_partition_detection() {
    // Setup: 10 nodes with quorum=7
    let nodes = setup_mesh_network(10, 5).await;
    let partition_detector = PartitionDetector::new(
        nodes[0].peer_registry.clone(),
        7, // quorum
        Duration::from_secs(1),
    );

    // Simulate partition: isolate 3 nodes
    partition_network(&nodes, vec![0, 1, 2], vec![3, 4, 5, 6, 7, 8, 9]).await;

    // Wait for detection
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Assert: Partition detected on minority side
    let status = partition_detector.detect_partition().await;
    match status {
        PartitionStatus::Partitioned { reachable, required } => {
            assert_eq!(reachable, 2); // Only sees 2 other nodes
            assert_eq!(required, 7);  // Needs 7 for quorum
        }
        _ => panic!("Partition not detected"),
    }

    // Heal partition
    heal_network(&nodes).await;

    // Wait for recovery
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Assert: Partition healed
    let status = partition_detector.detect_partition().await;
    assert!(matches!(status, PartitionStatus::Healthy));
}

#[tokio::test]
async fn test_byzantine_node_detection() {
    // Setup: 10 nodes
    let nodes = setup_mesh_network(10, 5).await;

    // Inject Byzantine node (sends invalid signatures)
    let byzantine_node = nodes[5].clone();
    byzantine_node.enable_byzantine_mode().await;

    // Run gossip
    run_gossip_rounds(&nodes, 10).await;

    // Assert: Byzantine node detected
    for (i, node) in nodes.iter().enumerate() {
        if i != 5 {
            let byzantine_list = node.get_byzantine_nodes().await;
            assert!(
                byzantine_list.contains(&byzantine_node.id),
                "Node {} did not detect Byzantine node",
                i
            );
        }
    }

    // Assert: Byzantine node reputation dropped
    let peer_info = nodes[0].peer_registry
        .get_peer(&byzantine_node.id)
        .unwrap();
    assert!(
        peer_info.reputation < 0.5,
        "Byzantine node reputation should be <0.5"
    );
}

#[tokio::test]
async fn test_message_loss_resilience() {
    // Setup: 10 nodes with 30% message loss
    let nodes = setup_mesh_network(10, 5).await;
    enable_message_loss(&nodes, 0.3).await; // 30% loss

    let initial_state = create_test_state();
    nodes[0].update_state(initial_state.clone()).await;

    // Run gossip (should still converge, just slower)
    let max_rounds = 20;
    run_gossip_rounds(&nodes, max_rounds).await;

    // Assert: Still converges despite message loss
    for node in &nodes {
        let state = node.get_state().await;
        assert_eq!(state.merkle_root, initial_state.merkle_root);
    }
}
```

## Scale Testing Matrix

| Test | Nodes | Fanout (k) | Expected Rounds | Max Time | Pass Criteria |
|------|-------|------------|----------------|----------|---------------|
| Small mesh | 10 | 5 | ≤7 | 100ms | Converges |
| Medium mesh | 100 | 10 | ≤10 | 500ms | Converges |
| Large mesh | 1,000 | 10 | ≤13 | 1s | Converges |
| Extra large | 10,000 | 20 | ≤17 | 5s | Converges |
| Massive | 100,000 | 50 | ≤20 | 10s | Converges |
| Ultimate | 1,000,000 | 100 | ≤23 | 30s | Converges |

## Performance Benchmarks

**Location**: `rust/knhk-consensus/benches/mesh_latency.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_gossip_message_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("gossip_processing");

    for size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                let coordinator = setup_coordinator_with_peers(size);
                let message = create_test_message();

                b.iter(|| {
                    coordinator.process_message(black_box(&message))
                });
            },
        );
    }

    group.finish();
}

fn bench_peer_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_selection");

    for size in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                let registry = setup_registry(size);

                b.iter(|| {
                    registry.get_random_peers(black_box(10))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_gossip_message_processing,
    bench_peer_selection,
);
criterion_main!(benches);
```

## Weaver Validation Tests

**Purpose**: Verify runtime telemetry conforms to schema

```bash
# Test 1: Schema validation
weaver registry check -r /home/user/knhk/registry/mesh-networking-schema.yaml

# Test 2: Live validation (requires running system)
weaver registry live-check \
  --registry /home/user/knhk/registry/ \
  --endpoint http://localhost:4317 \
  --duration 60s

# Test 3: Covenant validation
weaver registry validate-covenants \
  --registry /home/user/knhk/registry/ \
  --covenant 5  # Chatman constant
```

## Test Execution

### Local Development

```bash
# Unit tests
cargo test --package knhk-consensus --lib mesh

# Chicago TDD tests
make test-chicago-v04

# Integration tests
cargo test --package knhk-consensus --test mesh_integration

# All tests
cargo test --workspace
```

### CI/CD Pipeline

```yaml
name: Mesh Networking Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --lib mesh

  chicago-tdd:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: make test-chicago-v04

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --test mesh_integration

  weaver-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: open-telemetry/weaver-action@v1
        with:
          command: registry check -r registry/

  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo bench --bench mesh_latency
```

## Success Criteria

### Definition of Done

All tests MUST pass before merging:

- [ ] Unit tests: 95%+ coverage
- [ ] Chicago TDD: All latency tests ≤8 ticks
- [ ] Integration: Convergence in O(log n) rounds
- [ ] Chaos: Partition detection <1s, recovery <5s
- [ ] Weaver: Schema validation passes
- [ ] Benchmarks: No performance regressions

### Covenant Validation

- **Covenant 5**: All hot path operations ≤8 ticks ✅
- **Covenant 6**: 100% telemetry coverage ✅

## Related Documents

- `ADR-001-MESH-NETWORK-ARCHITECTURE.md` - Architecture decisions
- `COMPONENT-DESIGN.md` - Implementation details
- `registry/mesh-networking-schema.yaml` - Weaver schema
- `DOCTRINE_COVENANT.md` - Covenant 5, Covenant 6
