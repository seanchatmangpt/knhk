# Phase 8: Byzantine Consensus - Detailed Specification

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18
**Phase Duration**: 10 weeks | **LOC Estimate**: ~15,000 lines

---

## DOCTRINE Alignment

**Principle**: O (Observation Plane) - "Model reality carefully"
**Covenant**: Covenant 6 (Observations Drive Everything)
**Why This Matters**: Byzantine consensus creates observable, cryptographically-provable agreement on workflow state across untrusted nodes.

**What This Means**:
Distributed KNHK nodes may be malicious (Byzantine faults) or simply unreachable (crash faults). Phase 8 ensures that all honest nodes agree on workflow state despite f Byzantine nodes, where f < n/3.

**Anti-Patterns to Avoid**:
- ❌ Consensus blocking hot path (must be async)
- ❌ Unbounded consensus rounds (must timeout)
- ❌ Accepting f ≥ n/3 Byzantine nodes (violates PBFT bound)
- ❌ Unsigned consensus messages (enables spoofing)
- ❌ Consensus without finality detection (cannot prove irreversibility)

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│             Phase 8: Byzantine Consensus System                 │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │          Consensus State Machine Trait                   │   │
│  │                                                           │   │
│  │  trait ConsensusState {                                 │   │
│  │    type Proposal;                                       │   │
│  │    fn apply(&mut self, proposal: Proposal) -> Result;  │   │
│  │    fn snapshot(&self) -> StateSnapshot;                │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │     PBFT     │  │   HotStuff   │  │     Raft     │         │
│  │              │  │              │  │              │         │
│  │ • 3-phase   │  │ • Pipelined  │  │ • Leader     │         │
│  │ • f < n/3   │  │ • f < n/3    │  │ • Crash-fault│         │
│  │ • View change│ │ • Rotating   │  │ • Simpler    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Leader Election (VRF-based)                      │   │
│  │                                                           │   │
│  │  • Verifiable Random Function (quantum-safe)            │   │
│  │  • Cryptographic sortition (unpredictable)              │   │
│  │  • Fair rotation (prevents centralization)              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │          Three-Phase Protocol (PBFT)                     │   │
│  │                                                           │   │
│  │  Pre-Prepare → Prepare → Commit → Execute               │   │
│  │      ↓           ↓         ↓                             │   │
│  │    Leader    Quorum-1  Quorum-2  (2f+1 nodes)           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │            Finality Detection                             │   │
│  │                                                           │   │
│  │  • Irreversible commitment detection                    │   │
│  │  • Checkpoint blocks (every 100 blocks)                 │   │
│  │  • BFT finality gadget                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Network Partition Tolerance                      │   │
│  │                                                           │   │
│  │  Strong mode: Requires 2f+1 nodes (0% partition tol)   │   │
│  │  Weak mode:   Eventual consistency (partition tol)     │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Core Trait Definitions

### 1. Consensus State Machine

```rust
/// State machine replicated via Byzantine consensus
///
/// Generic over proposal type (workflow transitions, config changes, etc.)
pub trait ConsensusState: Clone + Serialize + DeserializeOwned {
    /// Proposal type (what is being agreed upon)
    type Proposal: Clone + Serialize + DeserializeOwned;

    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Apply proposal to state (deterministic)
    ///
    /// CRITICAL: Must be deterministic (same proposal → same result)
    /// Latency: ≤8 ticks (hot path)
    /// Telemetry: "consensus.apply" span
    fn apply(&mut self, proposal: Self::Proposal) -> Result<(), Self::Error>;

    /// Create state snapshot (for checkpointing)
    ///
    /// Latency: <10 ms (warm path)
    fn snapshot(&self) -> StateSnapshot;

    /// Restore from snapshot
    ///
    /// Latency: <100 ms (cold path)
    fn restore(snapshot: StateSnapshot) -> Result<Self, Self::Error>;

    /// Validate proposal (before consensus)
    ///
    /// Latency: ≤8 ticks (hot path)
    fn validate(&self, proposal: &Self::Proposal) -> bool;
}
```

### 2. PBFT Implementation

```rust
use phase7::HybridSignature;

/// Practical Byzantine Fault Tolerance (PBFT) consensus
///
/// Tolerates f Byzantine nodes where n = 3f + 1
/// Three-phase protocol: Pre-Prepare → Prepare → Commit
///
/// Reference: Castro & Liskov (1999)
pub struct PBFT<S: ConsensusState> {
    /// Node ID
    node_id: NodeId,

    /// Total nodes (n = 3f + 1)
    total_nodes: usize,

    /// Max Byzantine nodes (f = (n-1)/3)
    max_byzantine: usize,

    /// Current view number
    view: u64,

    /// Sequence number
    sequence: u64,

    /// State machine
    state: S,

    /// Message log
    log: MessageLog<S::Proposal>,

    /// Prepared certificates
    prepared: HashMap<u64, PreparedCert<S::Proposal>>,

    /// Committed certificates
    committed: HashMap<u64, CommittedCert<S::Proposal>>,

    /// Cryptographic keys (hybrid signatures)
    keys: Arc<ConsensusKeys>,
}

impl<S: ConsensusState> PBFT<S> {
    /// Create new PBFT instance
    pub fn new(
        node_id: NodeId,
        total_nodes: usize,
        state: S,
        keys: Arc<ConsensusKeys>,
    ) -> Result<Self, ConsensusError> {
        // Verify n = 3f + 1
        if (total_nodes - 1) % 3 != 0 {
            return Err(ConsensusError::InvalidNodeCount(total_nodes));
        }

        let max_byzantine = (total_nodes - 1) / 3;

        Ok(Self {
            node_id,
            total_nodes,
            max_byzantine,
            view: 0,
            sequence: 0,
            state,
            log: MessageLog::new(),
            prepared: HashMap::new(),
            committed: HashMap::new(),
            keys,
        })
    }

    /// Leader for current view
    ///
    /// Leader = view mod n (deterministic rotation)
    fn current_leader(&self) -> NodeId {
        NodeId((self.view % self.total_nodes as u64) as u32)
    }

    /// Propose new value (leader only)
    ///
    /// Latency: <1 ms (sign proposal)
    /// Telemetry: "pbft.propose" span
    #[instrument(skip(self, proposal))]
    pub async fn propose(&mut self, proposal: S::Proposal)
        -> Result<(), ConsensusError>
    {
        // Verify we are the leader
        if self.current_leader() != self.node_id {
            return Err(ConsensusError::NotLeader);
        }

        // Validate proposal
        if !self.state.validate(&proposal) {
            return Err(ConsensusError::InvalidProposal);
        }

        // Increment sequence number
        self.sequence += 1;

        // Create Pre-Prepare message
        let pre_prepare = PrePrepare {
            view: self.view,
            sequence: self.sequence,
            proposal: proposal.clone(),
        };

        // Sign message
        let message = bincode::serialize(&pre_prepare)?;
        let signature = HybridSignature::sign(&self.keys.secret_key, &message)?;

        // Broadcast to replicas
        self.broadcast(ConsensusMessage::PrePrepare {
            message: pre_prepare,
            signature,
        }).await?;

        // Log our own message
        self.log.insert_pre_prepare(self.sequence, pre_prepare);

        Ok(())
    }

    /// Handle Pre-Prepare message (replica)
    ///
    /// Latency: ≤8 ticks (verify signature)
    /// Telemetry: "pbft.pre_prepare" span
    #[instrument(skip(self, msg, sig))]
    pub async fn handle_pre_prepare(
        &mut self,
        msg: PrePrepare<S::Proposal>,
        sig: HybridSignatureBytes,
        sender: NodeId,
    ) -> Result<(), ConsensusError> {
        // Verify sender is leader
        if self.current_leader() != sender {
            return Err(ConsensusError::NotLeader);
        }

        // Verify signature
        let message = bincode::serialize(&msg)?;
        if !HybridSignature::verify(&self.keys.leader_public_key, &message, &sig) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Verify view and sequence
        if msg.view != self.view || msg.sequence != self.sequence + 1 {
            return Err(ConsensusError::InvalidSequence);
        }

        // Validate proposal
        if !self.state.validate(&msg.proposal) {
            return Err(ConsensusError::InvalidProposal);
        }

        // Log Pre-Prepare
        self.log.insert_pre_prepare(msg.sequence, msg.clone());

        // Send Prepare message
        let prepare = Prepare {
            view: msg.view,
            sequence: msg.sequence,
            digest: hash_proposal(&msg.proposal),
        };

        let prepare_msg = bincode::serialize(&prepare)?;
        let prepare_sig = HybridSignature::sign(&self.keys.secret_key, &prepare_msg)?;

        self.broadcast(ConsensusMessage::Prepare {
            message: prepare,
            signature: prepare_sig,
        }).await?;

        Ok(())
    }

    /// Handle Prepare message
    ///
    /// Latency: ≤8 ticks
    /// Telemetry: "pbft.prepare" span
    #[instrument(skip(self, msg, sig))]
    pub async fn handle_prepare(
        &mut self,
        msg: Prepare,
        sig: HybridSignatureBytes,
        sender: NodeId,
    ) -> Result<(), ConsensusError> {
        // Verify signature
        let message = bincode::serialize(&msg)?;
        let sender_key = self.get_node_public_key(sender)?;
        if !HybridSignature::verify(&sender_key, &message, &sig) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Log Prepare
        self.log.insert_prepare(msg.sequence, sender, msg.clone());

        // Check if we have 2f Prepare messages (prepared certificate)
        let prepare_count = self.log.count_prepares(msg.sequence, &msg.digest);

        if prepare_count >= 2 * self.max_byzantine {
            // We are PREPARED
            let prepared_cert = PreparedCert {
                view: msg.view,
                sequence: msg.sequence,
                digest: msg.digest.clone(),
            };

            self.prepared.insert(msg.sequence, prepared_cert);

            // Send Commit message
            let commit = Commit {
                view: msg.view,
                sequence: msg.sequence,
                digest: msg.digest,
            };

            let commit_msg = bincode::serialize(&commit)?;
            let commit_sig = HybridSignature::sign(&self.keys.secret_key, &commit_msg)?;

            self.broadcast(ConsensusMessage::Commit {
                message: commit,
                signature: commit_sig,
            }).await?;
        }

        Ok(())
    }

    /// Handle Commit message
    ///
    /// Latency: ≤8 ticks (verify) + O(1) (apply)
    /// Telemetry: "pbft.commit" span
    #[instrument(skip(self, msg, sig))]
    pub async fn handle_commit(
        &mut self,
        msg: Commit,
        sig: HybridSignatureBytes,
        sender: NodeId,
    ) -> Result<(), ConsensusError> {
        // Verify signature
        let message = bincode::serialize(&msg)?;
        let sender_key = self.get_node_public_key(sender)?;
        if !HybridSignature::verify(&sender_key, &message, &sig) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Log Commit
        self.log.insert_commit(msg.sequence, sender, msg.clone());

        // Check if we have 2f+1 Commit messages (committed certificate)
        let commit_count = self.log.count_commits(msg.sequence, &msg.digest);

        if commit_count >= 2 * self.max_byzantine + 1 {
            // We are COMMITTED (irreversible!)
            let committed_cert = CommittedCert {
                view: msg.view,
                sequence: msg.sequence,
                digest: msg.digest.clone(),
            };

            self.committed.insert(msg.sequence, committed_cert);

            // Apply proposal to state
            let proposal = self.log.get_proposal(msg.sequence)?;
            self.state.apply(proposal)?;

            // Emit finality event
            tracing::info!(
                sequence = msg.sequence,
                digest = ?msg.digest,
                "Consensus reached (COMMITTED)"
            );

            // Update sequence number
            self.sequence = msg.sequence;
        }

        Ok(())
    }
}
```

### 3. HotStuff (Pipelined BFT)

```rust
/// HotStuff: Pipelined Byzantine Fault Tolerance
///
/// Improvements over PBFT:
/// - Linear communication complexity (O(n) vs O(n²))
/// - Pipelined consensus (multiple proposals in flight)
/// - Rotating leader (every view)
/// - Responsive (2Δ latency where Δ = network delay)
///
/// Reference: Yin et al. (2019)
pub struct HotStuff<S: ConsensusState> {
    /// Node ID
    node_id: NodeId,

    /// Total nodes
    total_nodes: usize,

    /// Max Byzantine (f < n/3)
    max_byzantine: usize,

    /// Current view
    view: u64,

    /// State machine
    state: S,

    /// Block tree
    blocks: BlockTree<S::Proposal>,

    /// QC (Quorum Certificate) for highest block
    highest_qc: Option<QuorumCert>,

    /// VRF keys for leader election
    vrf_keys: Arc<VRFKeys>,

    /// Signature keys
    sig_keys: Arc<ConsensusKeys>,
}

impl<S: ConsensusState> HotStuff<S> {
    /// Propose new block (leader only)
    ///
    /// Latency: <1 ms (sign block)
    #[instrument(skip(self, proposal))]
    pub async fn propose(&mut self, proposal: S::Proposal)
        -> Result<(), ConsensusError>
    {
        // Verify leadership via VRF
        if !self.am_i_leader()? {
            return Err(ConsensusError::NotLeader);
        }

        // Create new block
        let block = Block {
            view: self.view,
            parent: self.highest_qc.as_ref().map(|qc| qc.block_hash),
            proposal,
            justify: self.highest_qc.clone(),
        };

        // Sign block
        let block_hash = hash_block(&block);
        let signature = HybridSignature::sign(
            &self.sig_keys.secret_key,
            block_hash.as_bytes(),
        )?;

        // Broadcast
        self.broadcast(HotStuffMessage::Propose {
            block,
            signature,
        }).await?;

        Ok(())
    }

    /// Leader election via VRF
    ///
    /// Latency: <100 μs (VRF evaluation)
    fn am_i_leader(&self) -> Result<bool, ConsensusError> {
        // VRF: hash(view || vrf_secret_key) → random value
        let vrf_input = self.view.to_le_bytes();
        let (vrf_output, vrf_proof) = self.vrf_keys.evaluate(&vrf_input)?;

        // Leader is node with smallest VRF output
        let my_value = u64::from_le_bytes(vrf_output[..8].try_into().unwrap());

        // TODO: Query all nodes for their VRF outputs
        // For now, assume deterministic rotation
        Ok(self.view % self.total_nodes as u64 == self.node_id.0 as u64)
    }
}
```

### 4. Raft (Crash-Fault Consensus)

```rust
/// Raft consensus (crash-fault tolerance only)
///
/// Simpler than PBFT/HotStuff but assumes non-Byzantine faults.
/// Use for trusted environments or as fallback.
///
/// Reference: Ongaro & Ousterhout (2014)
pub struct Raft<S: ConsensusState> {
    /// Node ID
    node_id: NodeId,

    /// Node role
    role: RaftRole,

    /// Current term
    term: u64,

    /// Voted for (in current term)
    voted_for: Option<NodeId>,

    /// Log entries
    log: Vec<LogEntry<S::Proposal>>,

    /// Commit index
    commit_index: u64,

    /// State machine
    state: S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}
```

---

## Leader Election (VRF-Based)

```rust
use vrf::{VRF, VRFProof};

/// Verifiable Random Function keys
///
/// VRF provides unpredictable but verifiable randomness
/// for fair leader election.
pub struct VRFKeys {
    /// VRF secret key
    secret_key: vrf::SecretKey,

    /// VRF public key
    public_key: vrf::PublicKey,
}

impl VRFKeys {
    /// Generate new VRF keypair
    pub fn generate() -> Self {
        let (public_key, secret_key) = vrf::keypair();
        Self { secret_key, public_key }
    }

    /// Evaluate VRF
    ///
    /// Output: Deterministic random value
    /// Proof: Can be verified by anyone with public key
    pub fn evaluate(&self, input: &[u8])
        -> Result<([u8; 32], VRFProof), VRFError>
    {
        let output = vrf::prove(&self.secret_key, input)?;
        let proof = vrf::proof(&self.secret_key, input)?;
        Ok((output, proof))
    }

    /// Verify VRF proof
    pub fn verify(
        public_key: &vrf::PublicKey,
        input: &[u8],
        output: &[u8; 32],
        proof: &VRFProof,
    ) -> bool {
        vrf::verify(public_key, input, output, proof).is_ok()
    }
}
```

---

## Finality Detection

```rust
/// Finality gadget for BFT consensus
///
/// Detects when a block is irreversibly committed
/// (cannot be reverted even if f Byzantine nodes collude)
pub struct FinalityGadget {
    /// Checkpoint interval (blocks)
    checkpoint_interval: u64,

    /// Finalized checkpoints
    finalized: BTreeMap<u64, CheckpointCert>,
}

impl FinalityGadget {
    /// Check if block is finalized
    ///
    /// A block is finalized if:
    /// 1. It has 2f+1 commit votes
    /// 2. A checkpoint including it has been signed by 2f+1 nodes
    pub fn is_finalized(&self, block_num: u64) -> bool {
        // Find latest checkpoint <= block_num
        let checkpoint_num = (block_num / self.checkpoint_interval)
            * self.checkpoint_interval;

        self.finalized.contains_key(&checkpoint_num)
    }

    /// Finalize checkpoint
    ///
    /// Requires 2f+1 signatures on checkpoint
    pub fn finalize_checkpoint(
        &mut self,
        checkpoint_num: u64,
        cert: CheckpointCert,
    ) -> Result<(), FinalityError> {
        // Verify certificate has 2f+1 signatures
        if cert.signatures.len() < 2 * self.max_byzantine() + 1 {
            return Err(FinalityError::InsufficientSignatures);
        }

        // Verify all signatures
        for (node_id, signature) in &cert.signatures {
            let public_key = self.get_node_public_key(*node_id)?;
            let message = bincode::serialize(&checkpoint_num)?;

            if !HybridSignature::verify(&public_key, &message, signature) {
                return Err(FinalityError::InvalidSignature);
            }
        }

        // Mark as finalized
        self.finalized.insert(checkpoint_num, cert);

        Ok(())
    }
}
```

---

## Performance Constraints

### Latency Budgets

| Operation | Latency | Validation |
|-----------|---------|------------|
| Local apply | ≤8 ticks | chicago-tdd |
| Sign message | <1 ms | crypto-bench |
| Verify signature | <1 ms | crypto-bench |
| Broadcast | <10 ms | Same datacenter |
| Global consensus | <250 ms | Multi-region |

### Throughput Targets

- **PBFT**: ~1,000 TPS (single datacenter), ~100 TPS (multi-region)
- **HotStuff**: ~5,000 TPS (pipelined), ~500 TPS (multi-region)
- **Raft**: ~10,000 TPS (crash-fault only)

---

## OpenTelemetry Schema

```yaml
# registry/phases_6_10/consensus.yaml
spans:
  - span_name: consensus.propose
    attributes:
      - name: algorithm
        type: string
        values: [pbft, hotstuff, raft]
      - name: view
        type: int
      - name: sequence
        type: int

  - span_name: consensus.commit
    attributes:
      - name: sequence
        type: int
      - name: quorum_size
        type: int
      - name: latency_ms
        type: int

metrics:
  - metric_name: consensus.latency
    instrument: histogram
    unit: ms

  - metric_name: consensus.throughput
    instrument: counter
    unit: proposals
```

---

## Testing Strategy

### Byzantine Fault Injection

```rust
#[tokio::test]
async fn test_byzantine_tolerance() {
    // Setup 4 nodes (f=1, n=4)
    let mut nodes = create_pbft_cluster(4).await;

    // Make node 0 Byzantine (send conflicting proposals)
    nodes[0].set_byzantine(ByzantineBehavior::DoublePropose);

    // Honest nodes should still reach consensus
    let proposal = WorkflowProposal::new(...);
    nodes[1].propose(proposal.clone()).await.unwrap();

    // Wait for consensus
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify all honest nodes agree
    assert_eq!(nodes[1].get_state(), nodes[2].get_state());
    assert_eq!(nodes[2].get_state(), nodes[3].get_state());
}
```

---

## Related Documents

- `PHASE_7_QUANTUM_CRYPTO_SPECIFICATION.md` (signatures)
- `PHASE_6_NEURAL_SPECIFICATION.md` (multi-agent learning)
- `ADR/ADR-005-byzantine-consensus-selection.md`

**Next**: See `PHASE_9_HARDWARE_ACCELERATION_SPECIFICATION.md`
