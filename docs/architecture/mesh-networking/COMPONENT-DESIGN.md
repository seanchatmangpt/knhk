# Mesh Networking - Component Design & Implementation

**Version**: 1.0.0 | **Date**: 2025-11-18

---

## Overview

This document provides detailed component design and implementation specifications for the KNHK distributed mesh networking system.

## Component Catalog

| Component | Module | Responsibility | Dependencies |
|-----------|--------|----------------|--------------|
| **Peer Registry** | `mesh::peer_registry` | Peer discovery, management | DashMap, tokio |
| **Gossip Coordinator** | `mesh::gossip` | Epidemic broadcast | CRDT, async |
| **Topology Manager** | `mesh::topology` | Latency-aware optimization | Statistics |
| **Partition Detector** | `mesh::partition` | Split-brain detection | Health checks |
| **QUIC Transport** | `mesh::transport::quic` | P2P communication | quinn, rustls |
| **Byzantine Validator** | `mesh::security` | Message validation | ed25519-dalek |
| **Telemetry Collector** | `mesh::telemetry` | OpenTelemetry export | otel-sdk |

---

## 1. Peer Registry

**Module**: `rust/knhk-consensus/src/mesh/peer_registry.rs`

### Interface

```rust
/// Peer registry for mesh networking
pub struct PeerRegistry {
    /// Peers indexed by agent_id
    peers: Arc<DashMap<AgentId, PeerInfo>>,
    /// Quorum size (n - f where f < n/3)
    quorum_size: usize,
    /// Gossip fanout (k peers per round)
    peer_sample_size: usize,
    /// Bootstrap seed nodes
    bootstrap_seeds: Vec<SocketAddr>,
    /// Self node ID
    self_id: AgentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub agent_id: AgentId,
    pub addr: SocketAddr,
    pub public_key: PublicKey,
    pub last_seen: Instant,
    pub reputation: f32,  // 0.0-1.0
    pub region: Region,
    pub latency_ms: f32,
}

impl PeerRegistry {
    /// Create new peer registry
    pub fn new(
        self_id: AgentId,
        quorum_size: usize,
        peer_sample_size: usize,
        bootstrap_seeds: Vec<SocketAddr>,
    ) -> Self;

    /// Register a new peer
    pub async fn register_peer(&self, peer: PeerInfo) -> Result<()>;

    /// Get random peers for gossip (Byzantine-robust)
    pub async fn get_random_peers(&self, k: usize) -> Vec<PeerInfo>;

    /// Get nearest peers (latency-aware)
    pub async fn get_nearest_peers(&self, k: usize) -> Vec<PeerInfo>;

    /// Get peers by region
    pub async fn get_peers_by_region(&self, region: &Region) -> Vec<PeerInfo>;

    /// Update peer reputation
    pub async fn update_reputation(&self, agent_id: &AgentId, delta: f32) -> Result<()>;

    /// Update peer last_seen timestamp
    pub async fn touch_peer(&self, agent_id: &AgentId) -> Result<()>;

    /// Prune stale peers
    pub async fn prune_stale_peers(&self, timeout: Duration) -> usize;

    /// Get quorum size
    pub fn quorum_size(&self) -> usize;

    /// Get total peer count
    pub fn peer_count(&self) -> usize;

    /// Get active peer count (seen in last 30s)
    pub fn active_peer_count(&self) -> usize;
}
```

### Implementation Details

**Peer Selection Strategies**:

```rust
impl PeerRegistry {
    /// Random selection (Byzantine-robust)
    pub async fn get_random_peers(&self, k: usize) -> Vec<PeerInfo> {
        use rand::seq::IteratorRandom;

        let mut rng = rand::thread_rng();
        self.peers
            .iter()
            .filter(|entry| {
                // Filter out self and low-reputation peers
                entry.key() != &self.self_id && entry.value().reputation >= 0.5
            })
            .choose_multiple(&mut rng, k)
            .into_iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Latency-aware selection (prefer nearby)
    pub async fn get_nearest_peers(&self, k: usize) -> Vec<PeerInfo> {
        let mut peers: Vec<_> = self.peers
            .iter()
            .filter(|entry| entry.key() != &self.self_id)
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by latency (ascending)
        peers.sort_by(|a, b| {
            a.latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        peers.truncate(k);
        peers
    }

    /// Hybrid selection (80% latency-aware, 20% random)
    pub async fn get_hybrid_peers(&self, k: usize) -> Vec<PeerInfo> {
        let k_nearby = (k as f32 * 0.8) as usize;
        let k_random = k - k_nearby;

        let mut peers = self.get_nearest_peers(k_nearby).await;
        peers.extend(self.get_random_peers(k_random).await);

        peers
    }
}
```

**Telemetry**:

```rust
impl PeerRegistry {
    async fn emit_telemetry(&self) {
        let total = self.peer_count();
        let active = self.active_peer_count();
        let byzantine = self.peers.iter()
            .filter(|e| e.value().reputation < 0.5)
            .count();

        tracing::info!(
            mesh.node_id = %self.self_id,
            mesh.peer.count = total,
            mesh.peer.active = active,
            mesh.peer.byzantine = byzantine,
            "Peer registry telemetry"
        );
    }
}
```

---

## 2. Gossip Coordinator

**Module**: `rust/knhk-consensus/src/mesh/gossip.rs`

### Interface

```rust
/// Gossip coordinator for epidemic broadcast
pub struct GossipCoordinator {
    peer_registry: Arc<PeerRegistry>,
    my_state: Arc<RwLock<VersionedState>>,
    message_tx: UnboundedSender<GossipMessage>,
    message_rx: UnboundedReceiver<GossipMessage>,
    convergence_detector: Arc<ConvergenceDetector>,
    validator: Arc<ByzantineValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedState {
    /// Version vector (Lamport clock per peer)
    pub version: HashMap<AgentId, u64>,
    /// State payload (CRDT)
    pub data: CrdtState,
    /// Merkle root for fast comparison
    pub merkle_root: Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    Push {
        from: AgentId,
        version: HashMap<AgentId, u64>,
        delta: StateDelta,
        merkle_root: Hash,
    },
    Pull {
        from: AgentId,
        version: HashMap<AgentId, u64>,
    },
    Delta {
        from: AgentId,
        delta: StateDelta,
    },
}

impl GossipCoordinator {
    /// Create new gossip coordinator
    pub fn new(
        peer_registry: Arc<PeerRegistry>,
        initial_state: CrdtState,
    ) -> Self;

    /// Start gossip rounds
    pub async fn run_gossip_loop(self: Arc<Self>) -> Result<()>;

    /// Single gossip round (push-pull-merge)
    pub async fn gossip_round(&self, round: u64) -> Result<()>;

    /// Push phase: send state to k peers
    async fn gossip_push(&self, peers: Vec<PeerInfo>) -> Result<()>;

    /// Pull phase: request missing state
    async fn gossip_pull(&self, peers: Vec<PeerInfo>) -> Result<()>;

    /// Merge phase: integrate received deltas
    async fn merge_state(&self, delta: StateDelta) -> Result<()>;

    /// Check convergence
    async fn check_convergence(&self) -> bool;
}
```

### Gossip Round Implementation

```rust
impl GossipCoordinator {
    pub async fn run_gossip_loop(self: Arc<Self>) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        let mut round = 0u64;

        loop {
            interval.tick().await;
            round += 1;

            // Execute gossip round
            let span = tracing::span!(
                tracing::Level::INFO,
                "mesh.gossip.round",
                mesh.node_id = %self.peer_registry.self_id,
                gossip.round = round,
            );

            let _enter = span.enter();

            // CRITICAL: Measure latency (must be ≤8 ticks per message)
            let start = Instant::now();

            self.gossip_round(round).await?;

            let elapsed_ticks = start.elapsed().as_nanos() / 250; // 1 tick = 250ns

            tracing::info!(
                gossip.latency_ticks = elapsed_ticks,
                "Gossip round complete"
            );

            // Covenant 5 validation
            if elapsed_ticks > 8 {
                tracing::error!(
                    gossip.latency_ticks = elapsed_ticks,
                    "COVENANT VIOLATION: Gossip round exceeded 8 ticks"
                );
            }
        }
    }

    pub async fn gossip_round(&self, round: u64) -> Result<()> {
        // 1. SELECT PEERS (1 tick)
        let k = self.peer_registry.peer_sample_size;
        let peers = self.peer_registry.get_random_peers(k).await;

        tracing::event!(name: "gossip.push.start");

        // 2. PUSH PHASE (2 ticks)
        self.gossip_push(peers.clone()).await?;

        tracing::event!(name: "gossip.push.complete");
        tracing::event!(name: "gossip.pull.start");

        // 3. PULL PHASE (2 ticks)
        self.gossip_pull(peers.clone()).await?;

        tracing::event!(name: "gossip.pull.complete");
        tracing::event!(name: "gossip.merge.start");

        // 4. MERGE PHASE (3 ticks)
        // Merge happens async via message handler

        tracing::event!(name: "gossip.merge.complete");

        // 5. CHECK CONVERGENCE
        if self.check_convergence().await {
            tracing::event!(name: "gossip.converged");
            tracing::info!(
                gossip.round = round,
                "Gossip converged"
            );
        }

        Ok(())
    }
}
```

### Byzantine Message Validation

```rust
pub struct ByzantineValidator {
    peer_registry: Arc<PeerRegistry>,
}

impl ByzantineValidator {
    /// Validate gossip message (≤2 ticks)
    pub async fn validate_message(&self, msg: &GossipMessage) -> Result<()> {
        let start = Instant::now();

        // 1. Check sender is known peer
        let sender = msg.sender();
        let peer = self.peer_registry.get_peer(&sender)
            .ok_or_else(|| ConsensusError::ByzantineNodeDetected(
                format!("Unknown sender: {}", sender)
            ))?;

        // 2. Verify ed25519 signature
        let signature = msg.signature();
        let payload = msg.payload_bytes();

        peer.public_key.verify(payload, signature)
            .map_err(|_| ConsensusError::ByzantineNodeDetected(
                format!("Invalid signature from {}", sender)
            ))?;

        // 3. Check timestamp is reasonable (within 5 minutes)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;

        let timestamp = msg.timestamp_ms();
        if timestamp > now + 300_000 || timestamp + 300_000 < now {
            return Err(ConsensusError::ByzantineNodeDetected(
                format!("Timestamp out of bounds: {}", sender)
            ));
        }

        let elapsed_ticks = start.elapsed().as_nanos() / 250;

        tracing::info!(
            validation.latency_ticks = elapsed_ticks,
            "Message validated"
        );

        // Covenant 5: Signature verification must be ≤2 ticks
        if elapsed_ticks > 2 {
            tracing::warn!(
                validation.latency_ticks = elapsed_ticks,
                "Signature verification slow (should be ≤2 ticks)"
            );
        }

        Ok(())
    }
}
```

---

## 3. Topology Manager

**Module**: `rust/knhk-consensus/src/mesh/topology.rs`

### Interface

```rust
/// Topology manager for latency-aware peer selection
pub struct TopologyManager {
    peer_registry: Arc<PeerRegistry>,
    latency_tracker: Arc<LatencyTracker>,
    rebalance_interval: Duration,
}

pub struct LatencyTracker {
    /// RTT measurements per peer
    latencies: Arc<DashMap<AgentId, LatencyStats>>,
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub peer_id: AgentId,
    pub current_ms: f32,
    pub min_ms: f32,
    pub max_ms: f32,
    pub avg_ms: f32,
    pub p99_ms: f32,
    pub sample_count: usize,
}

impl TopologyManager {
    /// Create topology manager
    pub fn new(
        peer_registry: Arc<PeerRegistry>,
        rebalance_interval: Duration,
    ) -> Self;

    /// Start topology rebalancing loop
    pub async fn run_rebalance_loop(self: Arc<Self>) -> Result<()>;

    /// Rebalance topology based on latency
    pub async fn rebalance_topology(&self) -> Result<()>;

    /// Cluster peers by latency
    async fn cluster_by_latency(&self, peers: Vec<PeerInfo>) -> Vec<Vec<PeerInfo>>;

    /// Measure peer latency (ping-pong)
    pub async fn measure_latency(&self, peer_id: &AgentId) -> Result<f32>;
}
```

### Implementation

```rust
impl TopologyManager {
    pub async fn run_rebalance_loop(self: Arc<Self>) -> Result<()> {
        let mut interval = tokio::time::interval(self.rebalance_interval);

        loop {
            interval.tick().await;

            let span = tracing::span!(
                tracing::Level::INFO,
                "mesh.topology.rebalance",
            );
            let _enter = span.enter();

            self.rebalance_topology().await?;
        }
    }

    pub async fn rebalance_topology(&self) -> Result<()> {
        // 1. Measure all peer latencies
        let peers = self.peer_registry.get_all_peers().await;

        let mut latencies = Vec::new();
        for peer in &peers {
            if let Ok(rtt) = self.measure_latency(&peer.agent_id).await {
                self.latency_tracker.record_latency(&peer.agent_id, rtt).await;
                latencies.push((peer.clone(), rtt));
            }
        }

        // 2. Cluster by latency (k-means)
        let clusters = self.cluster_by_latency(
            latencies.into_iter().map(|(p, _)| p).collect()
        ).await;

        tracing::info!(
            mesh.topology.clusters = clusters.len(),
            "Topology rebalanced"
        );

        Ok(())
    }

    async fn cluster_by_latency(&self, peers: Vec<PeerInfo>) -> Vec<Vec<PeerInfo>> {
        // Simple latency-based clustering
        let mut clusters: Vec<Vec<PeerInfo>> = Vec::new();

        // Sort by latency
        let mut sorted = peers.clone();
        sorted.sort_by(|a, b| {
            a.latency_ms.partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Group into clusters of ~100 peers each
        for chunk in sorted.chunks(100) {
            clusters.push(chunk.to_vec());
        }

        clusters
    }
}
```

---

## 4. Partition Detector

**Module**: `rust/knhk-consensus/src/mesh/partition.rs`

### Interface

```rust
/// Network partition detector
pub struct PartitionDetector {
    peer_registry: Arc<PeerRegistry>,
    reachable_peers: Arc<AtomicUsize>,
    quorum_size: usize,
    health_check_interval: Duration,
}

#[derive(Debug, Clone)]
pub enum PartitionStatus {
    Healthy,
    Partitioned {
        reachable: usize,
        required: usize,
    },
    Recovering,
}

impl PartitionDetector {
    /// Create partition detector
    pub fn new(
        peer_registry: Arc<PeerRegistry>,
        quorum_size: usize,
        health_check_interval: Duration,
    ) -> Self;

    /// Run health check loop
    pub async fn run_health_check_loop(self: Arc<Self>) -> Result<()>;

    /// Detect partition
    pub async fn detect_partition(&self) -> PartitionStatus;

    /// Handle partition
    pub async fn handle_partition(&self) -> Result<()>;

    /// Handle partition recovery
    pub async fn handle_recovery(&self) -> Result<()>;
}
```

### Implementation

```rust
impl PartitionDetector {
    pub async fn run_health_check_loop(self: Arc<Self>) -> Result<()> {
        let mut interval = tokio::time::interval(self.health_check_interval);

        loop {
            interval.tick().await;

            // Count reachable peers
            let active = self.peer_registry.active_peer_count();
            self.reachable_peers.store(active, Ordering::Relaxed);

            // Check partition status
            let status = self.detect_partition().await;

            match status {
                PartitionStatus::Healthy => {
                    // No action needed
                }
                PartitionStatus::Partitioned { reachable, required } => {
                    tracing::warn!(
                        mesh.partition.reachable_peers = reachable,
                        mesh.partition.quorum_size = required,
                        partition.status = "partitioned",
                        "Network partition detected"
                    );

                    self.handle_partition().await?;
                }
                PartitionStatus::Recovering => {
                    tracing::info!(
                        partition.status = "recovering",
                        "Partition healed - recovering"
                    );

                    self.handle_recovery().await?;
                }
            }
        }
    }

    pub async fn detect_partition(&self) -> PartitionStatus {
        let reachable = self.reachable_peers.load(Ordering::Relaxed);

        if reachable < self.quorum_size {
            PartitionStatus::Partitioned {
                reachable,
                required: self.quorum_size,
            }
        } else {
            PartitionStatus::Healthy
        }
    }

    pub async fn handle_partition(&self) -> Result<()> {
        // Enter read-only mode
        tracing::warn!("Entering read-only mode due to partition");

        // Emit telemetry
        tracing::info!(
            mesh.partition.detected = 1,
            "Partition detected - read-only mode"
        );

        Ok(())
    }

    pub async fn handle_recovery(&self) -> Result<()> {
        let start = Instant::now();

        // Re-sync state via gossip
        tracing::info!("Re-syncing state after partition recovery");

        // CRDT merge happens automatically via gossip

        let recovery_time_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            mesh.partition.recovery.time = recovery_time_ms,
            "Partition recovery complete"
        );

        Ok(())
    }
}
```

---

## 5. QUIC Transport

**Module**: `rust/knhk-consensus/src/mesh/transport/quic.rs`

### Dependencies

```toml
[dependencies]
quinn = "0.10"
rustls = "0.21"
```

### Interface

```rust
/// QUIC transport for P2P communication
pub struct QuicTransport {
    endpoint: quinn::Endpoint,
    tls_config: Arc<rustls::ServerConfig>,
}

impl QuicTransport {
    /// Create QUIC transport with TLS
    pub async fn new(
        bind_addr: SocketAddr,
        tls_cert: Vec<u8>,
        tls_key: Vec<u8>,
    ) -> Result<Self>;

    /// Send message to peer
    pub async fn send_message(
        &self,
        peer_addr: SocketAddr,
        message: &[u8],
    ) -> Result<()>;

    /// Receive messages
    pub async fn receive_messages(
        &self,
    ) -> impl Stream<Item = Result<(SocketAddr, Vec<u8>)>>;
}
```

---

## Related Documents

- `ADR-001-MESH-NETWORK-ARCHITECTURE.md` - Architecture decisions
- `SYSTEM-ARCHITECTURE.md` - High-level design
- `DEPLOYMENT-TOPOLOGIES.md` - Deployment models
- `registry/mesh-networking-schema.yaml` - Weaver schema
- `DOCTRINE_COVENANT.md` - Covenant 5 (Chatman), Covenant 6 (Observability)
