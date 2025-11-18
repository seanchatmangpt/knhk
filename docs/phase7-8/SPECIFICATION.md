# Phase 7+8: Quantum-Safe Cryptography & Byzantine Consensus Specification

**Version:** 1.0.0
**Status:** Design
**Authors:** KNHK Core Team
**Date:** 2025-11-18

---

## DOCTRINE ALIGNMENT

**Principles:**
- **Q (Hard Invariants):** Cryptographic proofs are non-negotiable invariants
- **O (Observation/Proof):** All consensus and signing events are observable via telemetry
- **Π (Projection):** Multi-region consensus projects global state across distributed nodes

**Covenants:**
- **Covenant 2 (Invariants Are Law):** Cryptographic verification failures are hard errors, never warnings
- **Covenant 5 (Chatman Constant):** All hot-path operations ≤8 ticks
- **Covenant 6 (Observations Drive Everything):** Every consensus round emits telemetry

**Why This Matters:**
KNHK must be provably secure against quantum computers and Byzantine attackers. Workflows orchestrated across multiple regions must reach consensus even when up to f < n/3 nodes are malicious or compromised.

---

## PHASE 7: QUANTUM-SAFE CRYPTOGRAPHY

### 1.1 NIST Post-Quantum Cryptography Algorithms

**Selected Algorithms:**

| Algorithm | Type | Use Case | Key Size | Signature Size | Performance |
|-----------|------|----------|----------|----------------|-------------|
| **Kyber** | KEM | Encryption/key exchange | 1568 bytes | N/A | ~100μs |
| **Dilithium** | Signature | Digital signatures | 2528 bytes | 3293 bytes | ~200μs |
| **Falcon** | Signature | Compact signatures | 1793 bytes | 809 bytes | ~150μs |
| **SLH-DSA** | Signature | Stateless hash-based | 64 bytes | 49856 bytes | ~10ms |

**Rationale:**
- **Dilithium:** NIST PQC winner, best balance of security/performance
- **Falcon:** Smallest signatures, best for bandwidth-constrained scenarios
- **SLH-DSA:** Quantum-proof forever (hash-based), best for archival/long-term
- **Kyber:** KEM for encrypted channels (future use)

**Hybrid Approach:**
Use BOTH classical (Ed25519) AND quantum-safe (Dilithium) signatures. If one is broken, the other remains secure.

### 1.2 Type-Level Cryptographic Security

**Phantom Types for Key Categories:**

```rust
// Phantom type markers
pub struct ClassicalKey;
pub struct QuantumSafeKey;
pub struct HybridKey;

// Generic signature scheme trait
pub trait SignatureScheme<K: KeyCategory>: Sized {
    type PublicKey: Clone + Serialize + DeserializeOwned;
    type SecretKey: Zeroize + Drop;
    type Signature: Clone + Serialize + DeserializeOwned;

    fn keygen() -> (Self::PublicKey, Self::SecretKey);
    fn sign(sk: &Self::SecretKey, msg: &[u8]) -> Self::Signature;
    fn verify(pk: &Self::PublicKey, msg: &[u8], sig: &Self::Signature) -> bool;
}
```

**Classical Implementation (Ed25519):**

```rust
pub struct Ed25519;

impl SignatureScheme<ClassicalKey> for Ed25519 {
    type PublicKey = ed25519_dalek::PublicKey;
    type SecretKey = ed25519_dalek::SecretKey;
    type Signature = ed25519_dalek::Signature;

    fn keygen() -> (Self::PublicKey, Self::SecretKey) {
        let sk = SecretKey::generate(&mut OsRng);
        let pk = PublicKey::from(&sk);
        (pk, sk)
    }

    fn sign(sk: &Self::SecretKey, msg: &[u8]) -> Self::Signature {
        sk.sign(msg)
    }

    fn verify(pk: &Self::PublicKey, msg: &[u8], sig: &Self::Signature) -> bool {
        pk.verify(msg, sig).is_ok()
    }
}
```

**Quantum-Safe Implementation (Dilithium):**

```rust
pub struct Dilithium;

impl SignatureScheme<QuantumSafeKey> for Dilithium {
    type PublicKey = pqcrypto_dilithium::PublicKey;
    type SecretKey = pqcrypto_dilithium::SecretKey;
    type Signature = pqcrypto_dilithium::Signature;

    fn keygen() -> (Self::PublicKey, Self::SecretKey) {
        pqcrypto_dilithium::keypair()
    }

    fn sign(sk: &Self::SecretKey, msg: &[u8]) -> Self::Signature {
        pqcrypto_dilithium::sign(msg, sk)
    }

    fn verify(pk: &Self::PublicKey, msg: &[u8], sig: &Self::Signature) -> bool {
        pqcrypto_dilithium::verify(msg, sig, pk).is_ok()
    }
}
```

**Hybrid Implementation:**

```rust
pub struct HybridSignature {
    pub ed25519: ed25519_dalek::Signature,
    pub dilithium: pqcrypto_dilithium::Signature,
}

pub struct HybridPublicKey {
    pub ed25519: ed25519_dalek::PublicKey,
    pub dilithium: pqcrypto_dilithium::PublicKey,
}

pub struct HybridSecretKey {
    ed25519: ed25519_dalek::SecretKey,
    dilithium: pqcrypto_dilithium::SecretKey,
}

impl Drop for HybridSecretKey {
    fn drop(&mut self) {
        // Zeroize secret keys on drop
    }
}

pub struct Hybrid;

impl SignatureScheme<HybridKey> for Hybrid {
    type PublicKey = HybridPublicKey;
    type SecretKey = HybridSecretKey;
    type Signature = HybridSignature;

    fn keygen() -> (Self::PublicKey, Self::SecretKey) {
        let (ed_pk, ed_sk) = Ed25519::keygen();
        let (dil_pk, dil_sk) = Dilithium::keygen();

        let pk = HybridPublicKey {
            ed25519: ed_pk,
            dilithium: dil_pk,
        };

        let sk = HybridSecretKey {
            ed25519: ed_sk,
            dilithium: dil_sk,
        };

        (pk, sk)
    }

    fn sign(sk: &Self::SecretKey, msg: &[u8]) -> Self::Signature {
        HybridSignature {
            ed25519: Ed25519::sign(&sk.ed25519, msg),
            dilithium: Dilithium::sign(&sk.dilithium, msg),
        }
    }

    fn verify(pk: &Self::PublicKey, msg: &[u8], sig: &Self::Signature) -> bool {
        // BOTH signatures must verify
        Ed25519::verify(&pk.ed25519, msg, &sig.ed25519)
            && Dilithium::verify(&pk.dilithium, msg, &sig.dilithium)
    }
}
```

### 1.3 Migration Strategy

**Timeline:**

| Phase | Period | Signature Type | Notes |
|-------|--------|---------------|-------|
| **Phase 1: Hybrid** | 2028-2029 | Ed25519 + Dilithium | Both signatures required |
| **Phase 2: Deprecation** | 2029-2030 | Hybrid with warnings | Classical-only descriptors deprecated |
| **Phase 3: Quantum-Only** | 2030+ | Dilithium only | Ed25519 support removed |

**Implementation:**

```rust
pub enum SignaturePolicy {
    ClassicalOnly,    // Pre-2028 (legacy)
    Hybrid,           // 2028-2030 (transition)
    QuantumOnly,      // 2030+ (future)
}

pub fn current_policy() -> SignaturePolicy {
    let now = SystemTime::now();
    if now < PHASE1_START {
        SignaturePolicy::ClassicalOnly
    } else if now < PHASE3_START {
        SignaturePolicy::Hybrid
    } else {
        SignaturePolicy::QuantumOnly
    }
}

pub fn verify_signature(
    descriptor: &WorkflowDescriptor,
    policy: SignaturePolicy,
) -> Result<(), CryptoError> {
    match (policy, &descriptor.signature) {
        (SignaturePolicy::ClassicalOnly, Signature::Classical(_)) => Ok(()),
        (SignaturePolicy::Hybrid, Signature::Hybrid(_)) => Ok(()),
        (SignaturePolicy::QuantumOnly, Signature::Quantum(_)) => Ok(()),
        _ => Err(CryptoError::PolicyViolation),
    }
}
```

### 1.4 Key Management

**Key Generation:**
- CSPRNG: Use OS entropy via `OsRng` (platform-specific: `/dev/urandom`, `BCryptGenRandom`, `getrandom()`)
- Key derivation: PBKDF2-HMAC-SHA256 (100,000 rounds) for password-based keys
- Key stretching: Argon2id for user passwords

**Key Storage:**
- At-rest encryption: AES-256-GCM with PBKDF2-derived key
- Key wrapping: Encrypt secret keys with master key
- Hardware security: TPM/Secure Enclave support (future)

**Key Rotation:**
- Automatic rotation: Every 90 days
- Versioned keys: `key_id = blake3(public_key || timestamp)`
- Graceful rollover: Old keys valid for 30 days after rotation

**Key Compromise:**
- Immediate revocation: Descriptor invalidation
- Revocation list: Distributed via consensus
- Re-signing: All affected descriptors re-signed with new keys

### 1.5 Performance Budget

**Signing Latency:**

| Operation | Ed25519 | Dilithium | Hybrid | Budget |
|-----------|---------|-----------|--------|--------|
| Sign | ~50μs | ~200μs | ~250μs | <1ms |
| Verify | ~100μs | ~300μs | ~400μs | <1ms |
| Keygen | ~50μs | ~150μs | ~200μs | <1ms |

**Chatman Constant Compliance:**
- 1 tick = ~125μs (8 MHz cycle time)
- Hybrid signing: ~250μs = **2 ticks** ✅
- Hot path budget: 8 ticks = 1ms ✅

---

## PHASE 8: BYZANTINE CONSENSUS

### 2.1 Consensus Algorithms

**Algorithm Comparison:**

| Algorithm | Byzantine Tolerance | Finality | Latency (Single Region) | Latency (Multi-Region) | Node Scalability |
|-----------|---------------------|----------|------------------------|------------------------|------------------|
| **PBFT** | f < n/3 | After prepare phase | 10-50ms | 300ms | <20 |
| **HotStuff** | f < n/3 | After 3 consecutive blocks | 50-100ms | 300ms | 20-100 |
| **Raft** | Crash-only | After log replication | 1-5ms | 50ms | <10 |

**Consensus Selection Matrix:**

| Use Case | Algorithm | Rationale |
|----------|-----------|-----------|
| Financial transactions | PBFT | Strong Byzantine tolerance, fast finality |
| High-throughput workflows | HotStuff | Pipelined consensus, better scalability |
| Internal infrastructure | Raft | Simpler, faster, sufficient for non-Byzantine |

### 2.2 PBFT (Practical Byzantine Fault Tolerance)

**Protocol:**

1. **Pre-prepare:** Leader broadcasts `<PRE-PREPARE, v, n, d>` where:
   - `v` = view number
   - `n` = sequence number
   - `d` = digest of request

2. **Prepare:** Replica `i` broadcasts `<PREPARE, v, n, d, i>` if:
   - Signature is valid
   - `v` is current view
   - `n` is in sequence window

3. **Commit:** Replica `i` broadcasts `<COMMIT, v, n, d, i>` after receiving `2f` matching prepares

4. **Execute:** Apply command after receiving `2f+1` matching commits

**Safety Proof:**
- Quorum intersection: Any two quorums of size `2f+1` intersect in at least `f+1` nodes
- At most `f` are Byzantine → At least 1 honest node in intersection
- Honest nodes never sign conflicting prepares → Safety

**Implementation:**

```rust
pub struct PBFTNode<S: ConsensusState> {
    id: usize,
    state: S,
    view: u64,
    seq_num: u64,
    log: Vec<LogEntry<S::Command>>,

    // Quorum tracking
    n: usize,              // Total nodes
    f: usize,              // Max Byzantine (f = (n-1)/3)
    quorum_size: usize,    // 2f + 1

    // Phase tracking
    pending: HashMap<u64, PBFTPhase>,
}

pub enum PBFTPhase {
    PrePrepare { cmd: Command, leader: usize },
    Prepare { prepares: Vec<PrepareMsg> },
    Commit { commits: Vec<CommitMsg> },
    Executed,
}

impl<S: ConsensusState> PBFTNode<S> {
    pub async fn propose(&mut self, cmd: S::Command) -> Result<S::Response, ConsensusError> {
        let seq = self.seq_num;
        self.seq_num += 1;

        // Phase 1: Pre-prepare (leader only)
        if self.is_leader() {
            let msg = Message::PrePrepare {
                view: self.view,
                seq,
                digest: blake3::hash(&bincode::serialize(&cmd)?),
            };
            self.broadcast(&msg).await?;
        }

        // Phase 2: Prepare (all replicas)
        let prepare_msg = Message::Prepare {
            view: self.view,
            seq,
            id: self.id,
        };
        self.broadcast(&prepare_msg).await?;

        // Wait for 2f prepares
        let prepares = self.wait_prepares(seq, 2 * self.f).await?;

        // Phase 3: Commit
        let commit_msg = Message::Commit {
            view: self.view,
            seq,
            id: self.id,
        };
        self.broadcast(&commit_msg).await?;

        // Wait for 2f+1 commits (including self)
        let commits = self.wait_commits(seq, self.quorum_size).await?;

        // Phase 4: Execute
        let resp = self.state.apply(cmd);
        self.log.push(LogEntry {
            seq,
            command: cmd,
            state_hash: self.state.hash(),
        });

        Ok(resp)
    }

    async fn wait_prepares(&mut self, seq: u64, count: usize) -> Result<Vec<PrepareMsg>, ConsensusError> {
        // Wait for at least `count` prepare messages with matching seq/view
        // Timeout after 1 second
        tokio::time::timeout(Duration::from_secs(1), async {
            // Implementation: listen on message channel
        }).await?
    }
}
```

### 2.3 HotStuff (Pipelined BFT)

**Protocol:**

1. **Prepare Phase:** Leader proposes block `B`
2. **Pre-commit Phase:** Replicas vote for `B` if parent is committed
3. **Commit Phase:** Replicas vote for `B` if 2f+1 pre-commits
4. **Decide Phase:** Replicas finalize `B` if 2f+1 commits

**Pipelining:**
- 3 consecutive blocks in flight simultaneously
- Each phase for block `n` overlaps with next phase for block `n+1`
- Throughput: 3x PBFT

**Rotating Leader:**
- Leader rotates every block
- Prevents single point of failure
- View change on timeout

**Implementation:**

```rust
pub struct HotStuffNode<S: ConsensusState> {
    id: usize,

    // Three-chain rule
    leaf: Block<S>,        // Highest block
    locked: Block<S>,      // Highest pre-committed block
    committed: Block<S>,   // Highest committed block

    // Consensus state
    view: u64,
    phase: Phase,
}

pub struct Block<S: ConsensusState> {
    height: u64,
    parent_hash: [u8; 32],
    command: S::Command,
    justify: QuorumCert,   // 2f+1 signatures
}

pub struct QuorumCert {
    block_hash: [u8; 32],
    view: u64,
    signatures: Vec<Signature>,
}

impl<S: ConsensusState> HotStuffNode<S> {
    pub async fn propose(&mut self, cmd: S::Command) -> Result<(), ConsensusError> {
        // Create new block extending leaf
        let block = Block {
            height: self.leaf.height + 1,
            parent_hash: blake3::hash(&bincode::serialize(&self.leaf)?),
            command: cmd,
            justify: self.leaf.justify.clone(),
        };

        // Broadcast to replicas
        self.broadcast(&Message::Propose { block }).await?;

        Ok(())
    }

    pub async fn on_receive_propose(&mut self, block: Block<S>) -> Result<(), ConsensusError> {
        // Verify block extends locked block
        if block.height > self.locked.height {
            // Vote for block
            let vote = Vote {
                block_hash: blake3::hash(&bincode::serialize(&block)?),
                view: self.view,
                voter: self.id,
            };

            self.send_to_leader(&Message::Vote { vote }).await?;

            // Update leaf
            self.leaf = block;
        }

        Ok(())
    }

    pub async fn on_receive_votes(&mut self, votes: Vec<Vote>) -> Result<(), ConsensusError> {
        // Aggregate 2f+1 votes into QuorumCert
        if votes.len() >= self.quorum_size() {
            let qc = QuorumCert {
                block_hash: votes[0].block_hash,
                view: self.view,
                signatures: votes.iter().map(|v| v.signature.clone()).collect(),
            };

            // Advance commit chain
            self.update_commit_chain(&qc)?;
        }

        Ok(())
    }

    fn update_commit_chain(&mut self, qc: &QuorumCert) -> Result<(), ConsensusError> {
        // Three-chain rule:
        // If block B has 3 consecutive QCs, commit B's grandparent

        let b3 = self.get_block(&qc.block_hash)?;
        let b2 = self.get_block(&b3.parent_hash)?;
        let b1 = self.get_block(&b2.parent_hash)?;

        if b3.height == b2.height + 1 && b2.height == b1.height + 1 {
            // Commit b1
            self.committed = b1;
        }

        Ok(())
    }
}
```

### 2.4 Raft (Crash Fault Tolerance)

**Protocol:**

1. **Leader Election:** Followers timeout → become candidates → request votes
2. **Log Replication:** Leader appends to local log → replicates to followers
3. **Commitment:** Leader commits after majority replication

**State Machine:**

```rust
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftNode<S: ConsensusState> {
    id: usize,
    state: RaftState,

    // Persistent state
    current_term: u64,
    voted_for: Option<usize>,
    log: Vec<LogEntry<S::Command>>,

    // Volatile state
    commit_index: u64,
    last_applied: u64,

    // Leader state
    next_index: HashMap<usize, u64>,
    match_index: HashMap<usize, u64>,
}

impl<S: ConsensusState> RaftNode<S> {
    pub async fn propose(&mut self, cmd: S::Command) -> Result<S::Response, ConsensusError> {
        // Only leader can propose
        if !matches!(self.state, RaftState::Leader) {
            return Err(ConsensusError::NotLeader);
        }

        // Append to local log
        let entry = LogEntry {
            term: self.current_term,
            index: self.log.len() as u64,
            command: cmd,
        };
        self.log.push(entry.clone());

        // Replicate to followers
        self.replicate_log().await?;

        // Wait for majority
        self.wait_for_commit(entry.index).await?;

        // Apply to state machine
        let resp = self.apply(entry.command)?;

        Ok(resp)
    }

    async fn replicate_log(&mut self) -> Result<(), ConsensusError> {
        // Send AppendEntries to all followers
        for follower in self.followers() {
            let next = self.next_index[&follower];
            let entries = self.log[next as usize..].to_vec();

            let msg = Message::AppendEntries {
                term: self.current_term,
                leader_id: self.id,
                prev_log_index: next - 1,
                prev_log_term: self.log[(next - 1) as usize].term,
                entries,
                leader_commit: self.commit_index,
            };

            self.send_to(follower, &msg).await?;
        }

        Ok(())
    }
}
```

### 2.5 Multi-Region Deployment

**Topology:**

```
Region 1 (US-East)          Region 2 (EU)              Region 3 (APAC)
┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐
│  Node 1         │◄───────┤  Node 4         │◄───────┤  Node 7         │
│  Node 2         │        │  Node 5         │        │  Node 8         │
│  Node 3         │        │  Node 6         │        │  Node 9         │
└─────────────────┘        └─────────────────┘        └─────────────────┘
        │                          │                          │
        └──────────────────────────┴──────────────────────────┘
                    Cross-region consensus
```

**Consensus Quorum:**
- Total nodes: 9 (3 per region)
- Byzantine tolerance: f = 2 (can tolerate 2 Byzantine nodes)
- Quorum size: 2f + 1 = 5

**Network Latency:**
- Intra-region: ~1-5ms
- Inter-region (US↔EU): ~80-100ms
- Inter-region (US↔APAC): ~150-200ms
- Inter-region (EU↔APAC): ~200-250ms

**Global Consensus Latency:**
- PBFT: ~300ms (3 round-trips across regions)
- HotStuff: ~400ms (pipelined, but higher latency per round)
- Raft: Not Byzantine-tolerant (unsuitable for multi-region)

**Partition Tolerance:**
- If 1 region partitions: Remaining 2 regions (6 nodes) can reach quorum (5)
- If 2 regions partition: System halts (cannot reach quorum)
- Recovery: Automatic when partition heals

### 2.6 Cryptographic Receipts

**Receipt Format:**

```json
{
  "workflow_id": "uuid-v7",
  "execution_id": "uuid-v7",
  "timestamp": "2028-01-15T10:30:00.123456789Z",

  "state": {
    "hash": "blake3:0x1234...",
    "transitions": [
      {
        "task": "Task1",
        "result": "completed",
        "duration_ns": 123456,
        "state_before": "blake3:0xabcd...",
        "state_after": "blake3:0xef01..."
      }
    ]
  },

  "signatures": {
    "hybrid": {
      "ed25519": "base64-encoded-signature",
      "dilithium": "base64-encoded-signature"
    },
    "public_keys": {
      "ed25519": "base64-encoded-pubkey",
      "dilithium": "base64-encoded-pubkey"
    }
  },

  "consensus": {
    "algorithm": "HotStuff",
    "block_height": 12345,
    "block_hash": "blake3:0x5678...",
    "view": 42,
    "finalized": true,

    "quorum": {
      "size": 5,
      "required": 5,
      "signatures": [
        {
          "node_id": 1,
          "signature": "base64-encoded-hybrid-signature",
          "timestamp": "2028-01-15T10:30:00.100Z"
        },
        // ... 4 more signatures
      ]
    }
  },

  "telemetry": {
    "trace_id": "otel-trace-id",
    "span_id": "otel-span-id",
    "metrics": {
      "consensus_latency_ms": 287,
      "signature_latency_us": 245,
      "total_latency_ms": 312
    }
  }
}
```

**Verification:**

```rust
pub fn verify_receipt(receipt: &Receipt) -> Result<(), VerificationError> {
    // 1. Verify hybrid signatures on state hash
    let state_hash = blake3::hash(&bincode::serialize(&receipt.state)?);

    if !Hybrid::verify(
        &receipt.signatures.public_keys,
        &state_hash,
        &receipt.signatures.hybrid,
    ) {
        return Err(VerificationError::SignatureInvalid);
    }

    // 2. Verify consensus quorum
    if receipt.consensus.quorum.signatures.len() < receipt.consensus.quorum.required {
        return Err(VerificationError::QuorumNotReached);
    }

    // 3. Verify each quorum signature
    let consensus_msg = bincode::serialize(&receipt.consensus)?;

    for sig in &receipt.consensus.quorum.signatures {
        if !Hybrid::verify(&sig.public_key, &consensus_msg, &sig.signature) {
            return Err(VerificationError::QuorumSignatureInvalid(sig.node_id));
        }
    }

    // 4. Verify state transitions
    let mut current_hash = receipt.state.transitions[0].state_before;

    for transition in &receipt.state.transitions {
        if transition.state_before != current_hash {
            return Err(VerificationError::StateTransitionInvalid);
        }

        // Verify hash of (state_before || task || result) == state_after
        let computed = blake3::hash(&bincode::serialize(&(
            transition.state_before,
            &transition.task,
            &transition.result,
        ))?);

        if computed != transition.state_after {
            return Err(VerificationError::StateHashMismatch);
        }

        current_hash = transition.state_after;
    }

    Ok(())
}
```

---

## OBSERVABILITY & TELEMETRY

### 3.1 OpenTelemetry Schema

**Consensus Spans:**

```yaml
# registry/consensus/consensus.yaml
groups:
  - id: consensus.pbft
    type: span
    brief: PBFT consensus round
    attributes:
      - id: consensus.algorithm
        type: string
        brief: Consensus algorithm
        examples: ["PBFT", "HotStuff", "Raft"]
        requirement_level: required

      - id: consensus.view
        type: int
        brief: Current view number
        requirement_level: required

      - id: consensus.sequence
        type: int
        brief: Sequence number
        requirement_level: required

      - id: consensus.phase
        type: string
        brief: Current phase
        examples: ["pre-prepare", "prepare", "commit", "execute"]
        requirement_level: required

      - id: consensus.quorum_size
        type: int
        brief: Required quorum size (2f+1)
        requirement_level: required

      - id: consensus.received_votes
        type: int
        brief: Number of votes received
        requirement_level: required

  - id: crypto.sign
    type: span
    brief: Cryptographic signing operation
    attributes:
      - id: crypto.algorithm
        type: string
        brief: Signature algorithm
        examples: ["Ed25519", "Dilithium", "Hybrid"]
        requirement_level: required

      - id: crypto.key_id
        type: string
        brief: Key identifier
        requirement_level: required

      - id: crypto.message_size
        type: int
        brief: Message size in bytes
        requirement_level: required

      - id: crypto.signature_size
        type: int
        brief: Signature size in bytes
        requirement_level: required

  - id: consensus.finality
    type: event
    brief: Consensus finality achieved
    attributes:
      - id: consensus.block_height
        type: int
        brief: Finalized block height
        requirement_level: required

      - id: consensus.block_hash
        type: string
        brief: Finalized block hash
        requirement_level: required

      - id: consensus.quorum_signatures
        type: int
        brief: Number of quorum signatures
        requirement_level: required
```

**Metrics:**

```yaml
# registry/consensus/metrics.yaml
groups:
  - id: metric.consensus
    type: metric
    metric_name: consensus.latency
    brief: Consensus round latency
    instrument: histogram
    unit: ms
    attributes:
      - ref: consensus.algorithm
      - ref: consensus.phase

  - id: metric.crypto
    type: metric
    metric_name: crypto.sign.duration
    brief: Signature generation duration
    instrument: histogram
    unit: us
    attributes:
      - ref: crypto.algorithm
```

### 3.2 Weaver Validation

**Schema Check:**

```bash
weaver registry check -r /home/user/knhk/registry/
```

**Live Validation:**

```bash
# Start consensus node with OTLP export
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  cargo run --bin consensus-node

# Validate live telemetry
weaver registry live-check --registry /home/user/knhk/registry/
```

---

## TESTING STRATEGY

### 4.1 Chicago TDD Tests

**Test Categories:**

1. **Cryptographic Correctness:**
   - Sign/verify round-trip
   - Hybrid signature validation
   - Key generation randomness
   - Signature non-malleability

2. **Consensus Safety:**
   - No double-commit (different values at same height)
   - Quorum intersection
   - Byzantine node detection
   - View change correctness

3. **Consensus Liveness:**
   - Progress under f Byzantine nodes
   - Recovery from partition
   - Leader election termination

4. **Performance:**
   - Signing latency ≤250μs (2 ticks)
   - Consensus latency ≤50ms (single region)
   - Multi-region latency ≤300ms

**Test Harness:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_hybrid_signature_correctness() {
        // AAA: Arrange, Act, Assert

        // Arrange: Generate keypair
        let (pk, sk) = Hybrid::keygen();
        let msg = b"Hello, quantum-safe world!";

        // Act: Sign and verify
        let sig = Hybrid::sign(&sk, msg);
        let valid = Hybrid::verify(&pk, msg, &sig);

        // Assert: Verification succeeds
        assert!(valid, "Hybrid signature verification failed");
    }

    #[test]
    fn test_hybrid_signature_latency() {
        // Chatman constant: ≤8 ticks (1ms)
        let (pk, sk) = Hybrid::keygen();
        let msg = b"Performance test message";

        let start = Instant::now();
        let sig = Hybrid::sign(&sk, msg);
        let sign_duration = start.elapsed();

        assert!(
            sign_duration.as_micros() <= 1000,
            "Hybrid signing took {:?}μs (>1ms budget)",
            sign_duration.as_micros()
        );

        let start = Instant::now();
        let _ = Hybrid::verify(&pk, msg, &sig);
        let verify_duration = start.elapsed();

        assert!(
            verify_duration.as_micros() <= 1000,
            "Hybrid verification took {:?}μs (>1ms budget)",
            verify_duration.as_micros()
        );
    }

    #[test]
    fn test_pbft_safety_no_double_commit() {
        // Safety: Two different values cannot be committed at same sequence

        let mut nodes = create_pbft_cluster(4); // n=4, f=1

        // Byzantine node tries to propose two different values
        let cmd1 = Command::new("value1");
        let cmd2 = Command::new("value2");

        // Honest nodes reject double-proposal
        let result1 = nodes[0].propose(cmd1).await;
        let result2 = nodes[0].propose(cmd2).await;

        // Only one can succeed
        assert!(
            result1.is_ok() ^ result2.is_ok(),
            "Both commands committed at same sequence (safety violation)"
        );
    }

    #[test]
    fn test_pbft_liveness_under_byzantine() {
        // Liveness: Progress with f Byzantine nodes

        let mut nodes = create_pbft_cluster(7); // n=7, f=2

        // Make 2 nodes Byzantine (unresponsive)
        nodes[5].disconnect();
        nodes[6].disconnect();

        // Remaining 5 nodes (>2f+1=5) can still reach consensus
        let cmd = Command::new("test");
        let result = nodes[0].propose(cmd).await;

        assert!(result.is_ok(), "Consensus failed with f Byzantine nodes");
    }
}
```

### 4.2 Property-Based Testing

**QuickCheck Properties:**

```rust
use quickcheck::{Arbitrary, Gen, QuickCheck};

#[quickcheck]
fn prop_signature_verify_after_sign(msg: Vec<u8>) -> bool {
    let (pk, sk) = Hybrid::keygen();
    let sig = Hybrid::sign(&sk, &msg);
    Hybrid::verify(&pk, &msg, &sig)
}

#[quickcheck]
fn prop_signature_fails_wrong_message(msg: Vec<u8>, wrong_msg: Vec<u8>) -> bool {
    if msg == wrong_msg {
        return true; // Skip
    }

    let (pk, sk) = Hybrid::keygen();
    let sig = Hybrid::sign(&sk, &msg);

    // Signature on msg should NOT verify for wrong_msg
    !Hybrid::verify(&pk, &wrong_msg, &sig)
}

#[quickcheck]
fn prop_consensus_commits_same_value(cmds: Vec<Command>) -> bool {
    // All honest nodes commit the same sequence of commands

    let mut nodes = create_pbft_cluster(4);

    for cmd in cmds {
        for node in &mut nodes {
            node.propose(cmd.clone()).await.unwrap();
        }
    }

    // Check all nodes have identical logs
    let log0 = nodes[0].get_log();
    nodes.iter().all(|n| n.get_log() == log0)
}
```

### 4.3 Chaos Engineering

**Fault Injection:**

```rust
#[test]
fn test_network_partition_recovery() {
    let mut nodes = create_pbft_cluster(7);

    // Partition: Nodes 0-3 vs 4-6
    partition_network(&mut nodes, vec![0, 1, 2, 3], vec![4, 5, 6]);

    // Neither partition can reach quorum (need 5, have 4 and 3)
    let cmd = Command::new("test");
    assert!(nodes[0].propose(cmd.clone()).await.is_err());
    assert!(nodes[4].propose(cmd.clone()).await.is_err());

    // Heal partition
    heal_network(&mut nodes);

    // Consensus should recover
    assert!(nodes[0].propose(cmd).await.is_ok());
}

#[test]
fn test_byzantine_leader() {
    let mut nodes = create_pbft_cluster(7);

    // Make leader Byzantine (sends conflicting pre-prepares)
    make_byzantine(&mut nodes[0], ByzantineBehavior::Equivocate);

    // View change should occur
    tokio::time::sleep(Duration::from_secs(2)).await;

    // New leader elected
    assert_ne!(nodes[1].get_view(), 0);

    // Consensus continues
    let cmd = Command::new("test");
    assert!(nodes[1].propose(cmd).await.is_ok());
}
```

---

## SUCCESS CRITERIA

### Phase 7: Quantum-Safe Cryptography

✅ **Compliance:**
- [ ] 100% NIST PQC compliance (Dilithium for signatures)
- [ ] Hybrid signatures functional (Ed25519 + Dilithium)
- [ ] Migration path tested (classical → hybrid → quantum)
- [ ] Key rotation automated (90-day cycle)

✅ **Performance:**
- [ ] Hybrid signing ≤250μs (2 ticks)
- [ ] Hybrid verification ≤400μs (3 ticks)
- [ ] Keygen ≤200μs (2 ticks)
- [ ] All operations within Chatman constant (≤8 ticks)

✅ **Security:**
- [ ] Zero secret key leaks (Zeroize on drop)
- [ ] CSPRNG for key generation (OsRng)
- [ ] Secure key storage (AES-256-GCM)
- [ ] Revocation mechanism functional

### Phase 8: Byzantine Consensus

✅ **Safety:**
- [ ] No double-commit (proven by tests)
- [ ] Quorum intersection (2f+1)
- [ ] Byzantine node tolerance (f < n/3)
- [ ] View change correctness

✅ **Liveness:**
- [ ] Progress with f Byzantine nodes
- [ ] Recovery from network partition
- [ ] Leader election terminates

✅ **Performance:**
- [ ] PBFT single-region ≤50ms
- [ ] HotStuff single-region ≤100ms
- [ ] Multi-region ≤300ms (3 regions)
- [ ] Throughput: >1000 consensus/sec (single region)

### Combined: Cryptographic Receipts

✅ **Immutability:**
- [ ] Receipt hash includes all transitions
- [ ] Hybrid signatures on state hash
- [ ] Consensus quorum signatures (2f+1)
- [ ] Tamper-evident (any change invalidates signatures)

✅ **Observability:**
- [ ] 100% telemetry coverage (all consensus rounds emit spans)
- [ ] Weaver schema validation passes
- [ ] Live telemetry validation passes
- [ ] Metrics dashboards functional

✅ **Compliance:**
- [ ] Covenant 2: All crypto failures are hard errors
- [ ] Covenant 5: All hot-path operations ≤8 ticks
- [ ] Covenant 6: All consensus events observable

---

## IMPLEMENTATION ROADMAP

**Week 1: Cryptographic Foundations**
- [ ] Implement `SignatureScheme` trait
- [ ] Ed25519 implementation
- [ ] Dilithium integration (pqcrypto crate)
- [ ] Hybrid signature scheme
- [ ] Key management infrastructure

**Week 2: Consensus Algorithms**
- [ ] PBFT implementation
- [ ] HotStuff implementation
- [ ] Raft implementation (baseline)
- [ ] Network layer (P2P messaging)
- [ ] State machine replication

**Week 3: Multi-Region Deployment**
- [ ] Cross-region networking
- [ ] Partition detection
- [ ] Recovery mechanisms
- [ ] Load balancing

**Week 4: Integration & Testing**
- [ ] Cryptographic receipt generation
- [ ] End-to-end workflow tests
- [ ] Chaos engineering tests
- [ ] Performance benchmarks
- [ ] Weaver validation

**Week 5: Observability**
- [ ] OTel schema definitions
- [ ] Span instrumentation
- [ ] Metrics collection
- [ ] Live validation tests

**Week 6: Documentation & Release**
- [ ] API documentation
- [ ] Deployment guides
- [ ] Migration playbooks
- [ ] Production readiness review

---

## REFERENCES

**NIST PQC:**
- [NIST PQC Standardization](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [Dilithium Specification](https://pq-crystals.org/dilithium/)
- [Falcon Specification](https://falcon-sign.info/)

**Consensus:**
- [PBFT Paper](http://pmg.csail.mit.edu/papers/osdi99.pdf)
- [HotStuff Paper](https://arxiv.org/abs/1803.05069)
- [Raft Paper](https://raft.github.io/raft.pdf)

**KNHK Doctrine:**
- `/home/user/knhk/DOCTRINE_2027.md`
- `/home/user/knhk/DOCTRINE_COVENANT.md`

---

**Document Status:** DRAFT v1.0.0
**Next Review:** 2025-11-25
**Authors:** KNHK Core Team
