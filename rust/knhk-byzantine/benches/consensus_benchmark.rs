//! Performance benchmarks for Byzantine consensus
//!
//! Measures throughput and latency for PBFT and HotStuff consensus protocols.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_byzantine::{
    mapek_byzantine::ByzantineMAPEK,
    network::ByzantineNetwork,
    protocols::{hotstuff::HotStuffConsensus, pbft::PBFTConsensus},
    DecisionAction, NodeId, WorkflowDecision,
};
use std::{sync::Arc, time::Duration};

fn bench_network_broadcast(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_broadcast");

    for node_count in [4, 7, 10, 13] {
        let nodes: Vec<NodeId> = (0..node_count).map(NodeId).collect();
        let network = Arc::new(ByzantineNetwork::new(nodes));

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            &node_count,
            |b, _| {
                let net = network.clone();
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let msg = black_box(vec![1u8; 100]);
                        net.broadcast(msg).await.unwrap();
                    });
            },
        );
    }

    group.finish();
}

fn bench_qc_aggregation(c: &mut Criterion) {
    use knhk_byzantine::{
        protocols::Signature,
        qc_manager::QCAggregator,
        Hash,
    };

    let mut group = c.benchmark_group("qc_aggregation");

    for sig_count in [7, 13, 25, 50] {
        group.throughput(Throughput::Elements(sig_count));
        group.bench_with_input(
            BenchmarkId::from_parameter(sig_count),
            &sig_count,
            |b, &count| {
                b.iter(|| {
                    let block_hash = Hash([1u8; 32]);
                    let threshold = (count * 2) / 3 + 1;
                    let aggregator = QCAggregator::new(block_hash, 1, threshold as usize);

                    for i in 0..count {
                        aggregator
                            .add_signature(NodeId(i), Signature(vec![i as u8; 64]))
                            .unwrap();
                    }

                    black_box(aggregator.try_build_qc().unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_pbft_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("pbft_initialization");

    for node_count in [4, 7, 10, 13] {
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            &node_count,
            |b, &count| {
                b.iter(|| {
                    let nodes: Vec<NodeId> = (0..count).map(NodeId).collect();
                    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
                    black_box(PBFTConsensus::new(
                        NodeId(0),
                        nodes,
                        Duration::from_secs(5),
                        network,
                    ));
                });
            },
        );
    }

    group.finish();
}

fn bench_hotstuff_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("hotstuff_initialization");

    for node_count in [4, 7, 10, 13] {
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            &node_count,
            |b, &count| {
                b.iter(|| {
                    let nodes: Vec<NodeId> = (0..count).map(NodeId).collect();
                    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
                    black_box(HotStuffConsensus::new(
                        NodeId(0),
                        nodes,
                        Duration::from_secs(5),
                        network,
                    ));
                });
            },
        );
    }

    group.finish();
}

fn bench_mapek_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("mapek_analysis");

    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network);

    group.bench_function("analyze_workflow", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let _ = black_box(mapek.analyze("benchmark-workflow").await.unwrap());
            });
    });

    group.finish();
}

fn bench_decision_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_execution");

    let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
    let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
    let mapek = ByzantineMAPEK::new_pbft(NodeId(0), nodes, Duration::from_secs(5), network);

    let decision = WorkflowDecision {
        workflow_id: "benchmark-workflow".to_string(),
        action: DecisionAction::Execute,
        timestamp: 0,
    };

    group.bench_function("execute_decision", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                mapek.execute_consensus_decision(&decision).await.unwrap();
            });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_network_broadcast,
    bench_qc_aggregation,
    bench_pbft_initialization,
    bench_hotstuff_initialization,
    bench_mapek_analysis,
    bench_decision_execution,
);

criterion_main!(benches);
