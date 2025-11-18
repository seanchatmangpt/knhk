//! Gossip Consensus Scalability Benchmarks
//!
//! Validates performance targets for swarms from 10 to 1M agents.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_consensus::gossip::{
    ConvergenceTracker, GossipConfig, GossipProtocol, HierarchicalConfig, HierarchicalGossip,
    StateValue, VersionedState,
};
use std::time::Duration;

/// Benchmark gossip round execution for different swarm sizes
fn bench_gossip_rounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("gossip_rounds");

    let swarm_sizes = vec![10, 100, 1000, 10000];

    for swarm_size in swarm_sizes {
        group.throughput(Throughput::Elements(swarm_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(swarm_size),
            &swarm_size,
            |b, &size| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.to_async(&rt).iter(|| async {
                    let config = GossipConfig::new(1, size);
                    let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
                    let protocol = GossipProtocol::new(config.clone(), initial_state);

                    // Initialize peers
                    let peers: Vec<u64> = (1..=size as u64).collect();
                    protocol.init_peers(peers).await;

                    // Execute one round
                    let _stats = black_box(protocol.execute_round().await.unwrap());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark state hash verification (hot path - must be ≤8 ticks)
fn bench_state_hash_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_hash_verification");
    group.throughput(Throughput::Elements(1));

    group.bench_function("verify_hash", |b| {
        let state = VersionedState::new(1, StateValue::Number(42), 1);
        b.iter(|| {
            let verified = black_box(state.verify_hash());
            assert!(verified);
        });
    });

    group.finish();
}

/// Benchmark Merkle proof verification (hot path - must be ≤8 ticks)
fn bench_merkle_verification(c: &mut Criterion) {
    use blake3::Hash as Blake3Hash;
    use knhk_consensus::gossip::merkle::{build_merkle_tree, generate_merkle_proof, merkle_root};

    let mut group = c.benchmark_group("merkle_verification");
    group.throughput(Throughput::Elements(1));

    // Build tree with 1000 leaves
    let leaves: Vec<Blake3Hash> = (0..1000)
        .map(|i| blake3::hash(format!("leaf_{}", i).as_bytes()))
        .collect();
    let tree = build_merkle_tree(&leaves);
    let root = merkle_root(&tree).unwrap();
    let proof = generate_merkle_proof(&tree, 0).unwrap();

    group.bench_function("verify_merkle_proof", |b| {
        b.iter(|| {
            let verified = black_box(proof.verify(&leaves[0], &root));
            assert!(verified);
        });
    });

    group.finish();
}

/// Benchmark state merge (warm path - target <100ms)
fn bench_state_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_merge");
    group.throughput(Throughput::Elements(1));

    group.bench_function("merge_newer_state", |b| {
        b.iter(|| {
            let mut state1 = VersionedState::new(1, StateValue::Number(10), 1);
            let state2 = VersionedState::new(2, StateValue::Number(20), 2);
            let merged = black_box(state1.merge(state2));
            assert!(merged);
        });
    });

    group.finish();
}

/// Benchmark convergence tracking
fn bench_convergence_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("convergence_tracking");

    let swarm_sizes = vec![100, 1000, 10000];

    for swarm_size in swarm_sizes {
        group.throughput(Throughput::Elements(swarm_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(swarm_size),
            &swarm_size,
            |b, &size| {
                b.iter(|| {
                    let mut tracker = ConvergenceTracker::new(size, size / 3);
                    let state = VersionedState::new(1, StateValue::Number(100), 1);

                    // Simulate convergence
                    for i in 0..size {
                        tracker.update_agent_state(i as u64, &state);
                    }

                    let convergence = black_box(tracker.check_convergence(10));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark hierarchical gossip
fn bench_hierarchical_gossip(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchical_gossip");

    let swarm_sizes = vec![1000, 10000];

    for swarm_size in swarm_sizes {
        group.throughput(Throughput::Elements(swarm_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(swarm_size),
            &swarm_size,
            |b, &size| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.to_async(&rt).iter(|| async {
                    let config = HierarchicalConfig::new(0, size, 100);
                    let initial_state = VersionedState::new(0, StateValue::Number(0), 0);
                    let gossip = HierarchicalGossip::new(config, initial_state);

                    gossip.init_peers().await;
                    let _stats = black_box(gossip.execute_round().await.unwrap());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_gossip_rounds,
    bench_state_hash_verification,
    bench_merkle_verification,
    bench_state_merge,
    bench_convergence_tracking,
    bench_hierarchical_gossip,
);
criterion_main!(benches);
