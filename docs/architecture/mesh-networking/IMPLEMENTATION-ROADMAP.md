# Mesh Networking Implementation Roadmap

**Version**: 1.0.0 | **Date**: 2025-11-18 | **Timeline**: 5 weeks

---

## Overview

This roadmap outlines the phased implementation of KNHK distributed mesh networking, from basic P2P communication to hierarchical 1M-agent scale.

## Implementation Phases

```
Week 1: Core P2P Layer
Week 2: Gossip Dissemination
Week 3: Topology Management
Week 4: Multi-Region Support
Week 5: Hierarchical Scaling + Testing
```

---

## Phase 1: Core P2P Layer (Week 1)

**Goal**: Establish basic peer-to-peer communication infrastructure

### Deliverables

1. **Peer Registry** (`mesh/peer_registry.rs`)
   - Peer storage (DashMap)
   - Registration/deregistration
   - Random peer selection
   - Bootstrap discovery

2. **QUIC Transport** (`mesh/transport/quic.rs`)
   - QUIC endpoint setup
   - TLS configuration (mTLS)
   - Send/receive message functions
   - Connection pooling

3. **Message Types** (`mesh/messages.rs`)
   - Gossip message structs
   - Serialization (bincode)
   - Message signing (ed25519)
   - Message validation

4. **Basic Tests**
   - Unit tests for peer registry
   - QUIC transport integration tests
   - Message serialization tests

### Files to Create

```
rust/knhk-consensus/src/mesh/
├── mod.rs
├── peer_registry.rs
├── messages.rs
├── transport/
│   ├── mod.rs
│   └── quic.rs
└── error.rs
```

### Week 1 Tasks

**Day 1-2: Peer Registry**
```rust
// Create peer_registry.rs
pub struct PeerRegistry {
    peers: Arc<DashMap<AgentId, PeerInfo>>,
    quorum_size: usize,
    peer_sample_size: usize,
    bootstrap_seeds: Vec<SocketAddr>,
    self_id: AgentId,
}

impl PeerRegistry {
    pub fn new(...) -> Self { ... }
    pub async fn register_peer(&self, peer: PeerInfo) -> Result<()> { ... }
    pub async fn get_random_peers(&self, k: usize) -> Vec<PeerInfo> { ... }
    pub fn peer_count(&self) -> usize { ... }
}

#[cfg(test)]
mod tests {
    // 20+ unit tests
}
```

**Day 3-4: QUIC Transport**
```rust
// Create transport/quic.rs
pub struct QuicTransport {
    endpoint: quinn::Endpoint,
    tls_config: Arc<rustls::ServerConfig>,
}

impl QuicTransport {
    pub async fn new(
        bind_addr: SocketAddr,
        tls_cert: Vec<u8>,
        tls_key: Vec<u8>,
    ) -> Result<Self> { ... }

    pub async fn send_message(
        &self,
        peer_addr: SocketAddr,
        message: &[u8],
    ) -> Result<()> { ... }

    pub async fn receive_messages(&self) -> impl Stream<Item = Result<(SocketAddr, Vec<u8>)>> { ... }
}
```

**Day 5: Message Types**
```rust
// Create messages.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    Push { from: AgentId, version: VersionVector, delta: StateDelta },
    Pull { from: AgentId, version: VersionVector },
    Delta { from: AgentId, delta: StateDelta },
}

impl GossipMessage {
    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<()> { ... }
    pub fn verify(&self, public_key: &PublicKey) -> Result<()> { ... }
}
```

### Week 1 Exit Criteria

- [ ] Peer registry stores and retrieves peers
- [ ] QUIC transport sends/receives messages
- [ ] Messages are signed and verified
- [ ] 50+ unit tests passing
- [ ] Weaver schema drafted

---

## Phase 2: Gossip Dissemination (Week 2)

**Goal**: Implement epidemic gossip protocol with convergence detection

### Deliverables

1. **Gossip Coordinator** (`mesh/gossip.rs`)
   - Push-pull gossip rounds
   - Version vector tracking
   - Convergence detection
   - Byzantine message validation

2. **CRDT State** (`mesh/crdt.rs`)
   - Versioned state structure
   - CRDT merge operations
   - Merkle tree for fast comparison
   - State delta compression

3. **Byzantine Validator** (`mesh/security.rs`)
   - Signature verification (≤2 ticks)
   - Timestamp validation
   - Sender verification
   - Reputation tracking

4. **Chicago TDD Tests**
   - Gossip latency ≤8 ticks
   - Signature verification ≤2 ticks
   - CRDT merge ≤3 ticks

### Week 2 Tasks

**Day 1-2: CRDT State**
```rust
// Create crdt.rs
#[derive(Debug, Clone)]
pub struct VersionedState {
    pub version: HashMap<AgentId, u64>,  // Version vector
    pub data: CrdtState,                  // CRDT payload
    pub merkle_root: Hash,                // For fast comparison
}

impl VersionedState {
    pub fn merge(&mut self, delta: StateDelta) -> Result<()> {
        // CRITICAL: Must be ≤3 ticks
        let start = Instant::now();

        // Merge logic here

        let elapsed_ticks = start.elapsed().as_nanos() / 250;
        assert!(elapsed_ticks <= 3);
        Ok(())
    }
}
```

**Day 3-4: Gossip Coordinator**
```rust
// Create gossip.rs
pub struct GossipCoordinator {
    peer_registry: Arc<PeerRegistry>,
    my_state: Arc<RwLock<VersionedState>>,
    message_tx: UnboundedSender<GossipMessage>,
    message_rx: UnboundedReceiver<GossipMessage>,
    convergence_detector: Arc<ConvergenceDetector>,
}

impl GossipCoordinator {
    pub async fn run_gossip_loop(self: Arc<Self>) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        let mut round = 0u64;

        loop {
            interval.tick().await;
            round += 1;

            // CRITICAL: Entire round must be ≤8 ticks per message
            self.gossip_round(round).await?;
        }
    }

    pub async fn gossip_round(&self, round: u64) -> Result<()> {
        // 1. SELECT PEERS
        let peers = self.peer_registry.get_random_peers(10).await;

        // 2. PUSH PHASE
        self.gossip_push(peers.clone()).await?;

        // 3. PULL PHASE
        self.gossip_pull(peers.clone()).await?;

        // 4. CHECK CONVERGENCE
        if self.check_convergence().await {
            tracing::info!("Gossip converged at round {}", round);
        }

        Ok(())
    }
}
```

**Day 5: Byzantine Validator**
```rust
// Create security.rs
pub struct ByzantineValidator {
    peer_registry: Arc<PeerRegistry>,
}

impl ByzantineValidator {
    pub async fn validate_message(&self, msg: &GossipMessage) -> Result<()> {
        // CRITICAL: Must be ≤2 ticks
        let start = rdtsc();

        // 1. Check sender is known
        // 2. Verify signature
        // 3. Check timestamp

        let end = rdtsc();
        assert!(end - start <= 2);

        Ok(())
    }
}
```

### Week 2 Exit Criteria

- [ ] Gossip protocol converges in O(log n) rounds
- [ ] Byzantine messages rejected
- [ ] Latency ≤8 ticks (Chicago TDD)
- [ ] 10-node integration test passes
- [ ] Weaver telemetry emitted

---

## Phase 3: Topology Management (Week 3)

**Goal**: Optimize peer connections for latency and resilience

### Deliverables

1. **Topology Manager** (`mesh/topology.rs`)
   - Latency measurement (ping-pong)
   - Latency-based clustering
   - Periodic rebalancing
   - Geographic grouping

2. **Partition Detector** (`mesh/partition.rs`)
   - Quorum-based detection
   - Health check loop
   - Partition handling (read-only mode)
   - Recovery coordination

3. **Integration Tests**
   - 100-node convergence
   - Partition detection <1s
   - Recovery <5s
   - Multi-region (3 regions)

### Week 3 Tasks

**Day 1-2: Topology Manager**
```rust
// Create topology.rs
pub struct TopologyManager {
    peer_registry: Arc<PeerRegistry>,
    latency_tracker: Arc<LatencyTracker>,
    rebalance_interval: Duration,
}

impl TopologyManager {
    pub async fn run_rebalance_loop(self: Arc<Self>) -> Result<()> {
        let mut interval = tokio::time::interval(self.rebalance_interval);

        loop {
            interval.tick().await;
            self.rebalance_topology().await?;
        }
    }

    pub async fn rebalance_topology(&self) -> Result<()> {
        // 1. Measure latencies
        // 2. Cluster by latency
        // 3. Update peer preferences
        Ok(())
    }
}
```

**Day 3-4: Partition Detector**
```rust
// Create partition.rs
pub struct PartitionDetector {
    peer_registry: Arc<PeerRegistry>,
    reachable_peers: Arc<AtomicUsize>,
    quorum_size: usize,
    health_check_interval: Duration,
}

impl PartitionDetector {
    pub async fn detect_partition(&self) -> PartitionStatus {
        let reachable = self.reachable_peers.load(Ordering::Relaxed);

        if reachable < self.quorum_size {
            PartitionStatus::Partitioned { reachable, required: self.quorum_size }
        } else {
            PartitionStatus::Healthy
        }
    }
}
```

**Day 5: Integration Tests**
```rust
// tests/mesh_integration.rs
#[tokio::test]
async fn test_100_node_convergence() {
    let nodes = setup_mesh_network(100, 10).await;
    // ... test convergence
}

#[tokio::test]
async fn test_partition_detection() {
    // ... test partition scenarios
}
```

### Week 3 Exit Criteria

- [ ] Topology rebalances every 100 rounds
- [ ] Partition detected <1s
- [ ] Recovery <5s
- [ ] 100-node test passes
- [ ] Latency clustering works

---

## Phase 4: Multi-Region Support (Week 4)

**Goal**: Enable cross-region gossip with regional leaders

### Deliverables

1. **Regional Coordinator** (`mesh/regional.rs`)
   - Regional sub-meshes
   - Leader election (Raft)
   - Inter-region gossip
   - State aggregation

2. **Geographic Routing** (`mesh/routing.rs`)
   - Region-aware peer selection
   - Cross-region latency tracking
   - Bandwidth optimization
   - Compression (zstd)

3. **Multi-Region Tests**
   - 3 regions × 100 nodes
   - Leader failover
   - Cross-region convergence
   - 1000-node test

### Week 4 Tasks

**Day 1-2: Regional Coordinator**
```rust
// Create regional.rs
pub struct RegionalCoordinator {
    region: Region,
    local_mesh: Arc<GossipCoordinator>,
    leader_state: Arc<RwLock<LeaderState>>,
    other_leaders: Vec<LeaderInfo>,
}

impl RegionalCoordinator {
    pub async fn run_inter_region_gossip(&self) -> Result<()> {
        // Gossip between regional leaders
        Ok(())
    }
}
```

**Day 3-4: Geographic Routing**
```rust
// Create routing.rs
pub struct GeographicRouter {
    peer_registry: Arc<PeerRegistry>,
    region_map: HashMap<Region, Vec<AgentId>>,
}

impl GeographicRouter {
    pub async fn route_by_region(&self, region: &Region, k: usize) -> Vec<PeerInfo> {
        // Return k peers from specified region
        vec![]
    }
}
```

**Day 5: Multi-Region Tests**
```rust
#[tokio::test]
async fn test_multi_region_convergence() {
    let us = setup_regional_mesh("us-east-1", 100).await;
    let eu = setup_regional_mesh("eu-west-1", 100).await;
    let ap = setup_regional_mesh("ap-southeast-1", 100).await;

    // Test convergence across regions
}
```

### Week 4 Exit Criteria

- [ ] 3 regions communicate
- [ ] Leader election works
- [ ] Cross-region convergence <5s
- [ ] 1000-node test passes
- [ ] Bandwidth optimized

---

## Phase 5: Hierarchical Scaling + Testing (Week 5)

**Goal**: Scale to 1M agents with hierarchical topology

### Deliverables

1. **Hierarchical Coordinator** (`mesh/hierarchical.rs`)
   - 3-level hierarchy (edge/aggregator/coordinator)
   - State aggregation
   - Hierarchical gossip
   - Global convergence

2. **Performance Benchmarks**
   - 10k, 100k, 1M node tests
   - Latency profiling
   - Bandwidth measurement
   - Convergence analysis

3. **Weaver Validation**
   - Complete schema
   - Live validation tests
   - Covenant verification
   - Telemetry coverage

4. **Documentation**
   - API documentation
   - Deployment guide
   - Runbook
   - Troubleshooting guide

### Week 5 Tasks

**Day 1-2: Hierarchical Coordinator**
```rust
// Create hierarchical.rs
pub struct HierarchicalCoordinator {
    hierarchy_level: HierarchyLevel,  // Edge/Aggregator/Coordinator
    parent: Option<Arc<HierarchicalCoordinator>>,
    children: Vec<Arc<HierarchicalCoordinator>>,
    aggregated_state: Arc<RwLock<AggregatedState>>,
}

impl HierarchicalCoordinator {
    pub async fn aggregate_from_children(&self) -> Result<AggregatedState> {
        // Aggregate states from all children
        Ok(AggregatedState::default())
    }

    pub async fn propagate_to_parent(&self, state: AggregatedState) -> Result<()> {
        // Send aggregated state to parent
        Ok(())
    }
}
```

**Day 3: Performance Benchmarks**
```rust
// benches/mesh_latency.rs
fn bench_10k_nodes(c: &mut Criterion) { ... }
fn bench_100k_nodes(c: &mut Criterion) { ... }
fn bench_1m_nodes(c: &mut Criterion) { ... }
```

**Day 4: Weaver Validation**
```bash
# Validate schema
weaver registry check -r registry/mesh-networking-schema.yaml

# Live validation
weaver registry live-check --registry registry/ --duration 60s
```

**Day 5: Documentation**
- API docs (`cargo doc`)
- Deployment guide
- Runbook
- Performance tuning guide

### Week 5 Exit Criteria

- [ ] 1M-node simulation passes
- [ ] Convergence <30s for 1M nodes
- [ ] Weaver validation passes
- [ ] All documentation complete
- [ ] Performance benchmarks green

---

## Success Metrics

### Performance Targets

| Metric | Target | Actual |
|--------|--------|--------|
| Gossip latency | ≤8 ticks | __ |
| Signature verify | ≤2 ticks | __ |
| Peer selection | ≤1 tick | __ |
| CRDT merge | ≤3 ticks | __ |
| Convergence (10 nodes) | ≤100ms | __ |
| Convergence (1000 nodes) | ≤1s | __ |
| Convergence (1M nodes) | ≤30s | __ |
| Partition detection | ≤1s | __ |
| Partition recovery | ≤5s | __ |

### Test Coverage

- Unit tests: 95%+ line coverage
- Integration tests: All scenarios passing
- Chicago TDD: All latency tests ≤8 ticks
- Chaos tests: All fault scenarios handled
- Weaver: 100% telemetry coverage

### Code Quality

- `cargo clippy` - Zero warnings
- `cargo fmt` - All files formatted
- `cargo doc` - All public APIs documented
- Zero `unwrap()` or `expect()` in production paths

---

## Risk Mitigation

| Risk | Mitigation | Owner |
|------|------------|-------|
| Gossip doesn't converge | Add convergence detection, fallback to tree | Week 2 |
| Latency exceeds 8 ticks | Profile, optimize, use SIMD | Week 2-3 |
| Byzantine flooding | Rate limiting, reputation system | Week 2 |
| Partition doesn't detect | Add multiple detection methods | Week 3 |
| 1M-node test fails | Implement hierarchical aggregation | Week 5 |

---

## Related Documents

- `ADR-001-MESH-NETWORK-ARCHITECTURE.md` - Architecture decisions
- `SYSTEM-ARCHITECTURE.md` - System design
- `COMPONENT-DESIGN.md` - Implementation details
- `TEST-STRATEGY.md` - Test approach
- `DEPLOYMENT-TOPOLOGIES.md` - Deployment models
