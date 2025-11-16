# Phase 8 Architecture Reference

## System Design

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    KNHK Distributed System                      │
└─────────────────────────────────────────────────────────────────┘
                            │
                ┌───────────┴────────────┐
                │                        │
        ┌───────▼────────┐      ┌────────▼────────┐
        │  Application   │      │  Workflow Engine│
        │     Layer      │      │    (Phase 1-6)  │
        └────────┬────────┘      └────────┬────────┘
                 │                        │
        ┌────────▼──────────────────────────────┐
        │    State Machine Replication (SMR)    │
        │     Command Log + State Snapshots     │
        └────────┬───────────────────────────────┘
                 │
        ┌────────▼──────────────────────────────┐
        │        Consensus Protocol Layer       │
        │  ┌──────────────┐   ┌──────────────┐  │
        │  │    PBFT      │   │  HotStuff    │  │
        │  │ (3f+1 model) │   │(Leader-based)│  │
        │  └──────────────┘   └──────────────┘  │
        └────────┬───────────────────────────────┘
                 │
        ┌────────▼──────────────────────────────┐
        │     Network & Byzantine Detection      │
        │  Peer Discovery + Message Ordering    │
        └────────┬───────────────────────────────┘
                 │
        ┌────────▼──────────────────────────────┐
        │      Validator Set Management         │
        │  Reputation + Rotation + Activation   │
        └────────┬───────────────────────────────┘
                 │
        ┌────────▼──────────────────────────────┐
        │        Transport Layer (P2P)           │
        │  UDP/TCP with OpenTelemetry Tracing  │
        └───────────────────────────────────────┘
```

## PBFT vs HotStuff Comparison

| Aspect | PBFT | HotStuff |
|--------|------|----------|
| **Model** | State machine replication | Optimistic responsiveness |
| **Leader Selection** | View number rotation | Deterministic based on view |
| **Commit Rule** | 2f+1 commits in phase 3 | Three consecutive confirmed blocks |
| **Message Complexity** | O(n²) per view | O(n) per view |
| **Latency** | 3 network delays | 4 network delays (with pipelining) |
| **Throughput** | ~1000 ops/sec | ~2000 ops/sec (pipelined) |
| **Implementation** | Explicit phases | Generic commit rule |
| **Use Case** | Conservative, proven | Modern, efficient |

## Data Flow: PBFT Consensus

### Normal Operation (Happy Path)

```
Client Request
     │
     ├─────→ Leader (Primary) [P]
     │            │
     │      Assign Sequence #n
     │      Create Digest D = hash(request)
     │            │
     │            └─────→ [PrePrepare, n, D, req] ──┐
     │                                                │
     ├─────────────────────────────────┐         [F1, F2]
     │                    Replicas      │
     │              [F1: n, D Prepared] │
     │              [F2: n, D Prepared] │         Quorum = 2f+1
     │                    │◄────────────┴─────────────┘
     │            [Prepare, n, D] ←────────
     │                    │
     │    When 2f+1 Prepares received:
     │                    │
     │                [F1: n, D Committed]
     │                [F2: n, D Committed]
     │                    │
     │            [Commit, n, D] ←────────
     │                    │
     └─────→ Client   Reply with committed state
```

### View Change (Timeout)

```
Current View v:
    Primary [P] crashed/slow

    Timeout triggered at replicas
            │
    [ViewChange, v+1] ←─────┐
                    ┌─ [F1, F2]
                    │
    New Primary = (v+1) mod n
            │
            └─→ [NewView, v+1]
                    │
                    └─→ [PrePrepare for new requests]
```

## Data Flow: HotStuff Consensus

### Block Proposal & Voting

```
View v:
    Leader [L]
         │
    Propose Block B (height h, parent parent_qc)
         │
         ├─────→ [Propose, B] ────┐
         │                         │
         │    Replicas [F1, F2, F3]
         │         │
         │    Verify B, lock_qc ← parent_qc
         │         │
         │    [Vote, B] ←────┐
         │                   │
         └─────→ [Vote] ────→ Collect votes
                   │
            Quorum = 2f+1 votes
                   │
                [QC = 2f+1 votes for B]

View v+1:
    Next block B' (parent = B, parent_qc)
         │
    [Propose, B'] ───→ [Vote] ───→ [QC for B']

View v+2:
    Block B'' (parent = B', parent_qc for B')
         │
    [Propose, B''] ───→ [Vote] ───→ [QC for B'']
                                        │
                            ┌───────────┘
                            │
                    Three-chain rule fires:
                    Commit B, B', B''
                            │
                            └─→ Deterministic State
```

## State Machine Replication

### Command Execution Pipeline

```
Consensus
  │
  └─→ Committed Sequence
       │
       ├─ [seq=0, cmd="transfer 100"]
       ├─ [seq=1, cmd="balance query"]
       ├─ [seq=2, cmd="approve tx"]
       │
       Ordered Application
       │
       ├─→ Execute cmd[0] ──→ Transfer 100 ──→ State update
       │
       ├─→ Execute cmd[1] ──→ Query balance ──→ Result = 900
       │
       ├─→ Execute cmd[2] ──→ Approve tx ──→ State update
       │
       Snapshot every k commands
       │
       ├─ Snapshot[v=0]: state={...}, hash=h0
       │
       ├─ Snapshot[v=1]: state={...}, hash=h1
       │
       Recovery/Rollback
       │
       └─→ restore_snapshot(v=0) ──→ State deterministically recovered
```

## Network Layer Design

### Peer Management

```
Node [N1]
  │
  ├─ Peer Registry
  │  ├─ [node2: ip, pk, metrics]
  │  ├─ [node3: ip, pk, metrics]
  │  └─ [node4: ip, pk, metrics]
  │
  ├─ Message Ordering
  │  └─ Track seq per peer ──→ Detect out-of-order
  │
  ├─ Byzantine Detection
  │  ├─ Invalid signatures ──→ mark Byzantine
  │  ├─ Out-of-order messages ──→ mark Byzantine
  │  └─ Timestamp violations ──→ mark Byzantine
  │
  └─ Broadcasting
     ├─ Consensus msgs ──→ all peers
     └─ Tracked per sequence number
```

## Validator Set Lifecycle

```
Phase 1: Initialization
    ├─ Create ValidatorSet(max=10, min=3)
    ├─ Add initial validators
    │  └─ All have uptime=100%, metrics=empty
    └─ All marked active

Phase 2: Operations
    ├─ Update metrics periodically
    │  ├─ valid_messages += 1
    │  ├─ invalid_signatures += errors
    │  └─ byzantine_behaviors += violations
    │
    ├─ Check health
    │  ├─ reputation_score = (valid% * 0.7) + (uptime * 0.3)
    │  ├─ If score < 0.7 ──→ deactivate
    │  └─ If byzantine > 0 ──→ remove
    │
    └─ Track inactivity
       └─ last_activity_ms > timeout ──→ candidate for removal

Phase 3: Rotation
    ├─ Remove inactive validators
    │  └─ (now - last_activity) > 5 minutes
    │
    ├─ Add new validators
    │  └─ Fill slots up to max
    │
    └─ Emit event: ValidatorSetChanged
       └─ Next view uses new set
```

## Error Handling & Recovery

### Consensus Errors

```
enum ConsensusError {
    QuorumNotReached(current, required),
    InvalidSignature,
    StateMismatch { expected, actual },
    ByzantineNodeDetected(node_id),
    ViewSyncTimeout,
    InvalidValidatorSet(reason),
}
```

### Recovery Strategies

| Error | Root Cause | Recovery |
|-------|-----------|----------|
| `QuorumNotReached` | Network partition | Trigger view change timeout |
| `InvalidSignature` | Message tampering | Mark peer Byzantine |
| `StateMismatch` | Replica divergence | Roll back to last good snapshot |
| `ByzantineNodeDetected` | Malicious node | Remove from validator set |
| `ViewSyncTimeout` | Slow/dead leader | Trigger view change (new view) |

## Performance Characteristics

### PBFT (7-node cluster)

```
Latency:
  ├─ PrePrepare broadcast: ~30ms
  ├─ Prepare phase: ~30ms
  ├─ Commit phase: ~30ms
  └─ Total: ~90ms (3 network delays)

Throughput:
  ├─ Sequential: ~1000 ops/sec
  └─ Batched: ~5000 ops/sec

Memory:
  ├─ Per node: ~2MB protocol state
  ├─ Message log: ~50MB (1000 messages)
  └─ Snapshots: ~100MB (10 snapshots)
```

### HotStuff (7-node cluster)

```
Latency:
  ├─ Propose: ~40ms
  ├─ Vote: ~40ms
  ├─ QC formation: ~40ms
  ├─ Generic commit: ~40ms
  └─ Total: ~160ms (4 network delays, but pipelined)

Throughput:
  ├─ Sequential: ~2000 ops/sec (pipelined)
  └─ Optimized: ~10000 ops/sec (optimistic)

Memory:
  ├─ Per node: ~1MB protocol state
  ├─ Block tree: ~20MB (100 blocks)
  └─ QCs: ~10MB cached
```

## Integration Points

### With Workflow Engine (Phases 1-6)

```
Workflow Execution
        │
        └─→ Command Generation
            │
            ├─ Each step ──→ Command
            ├─ Parallel steps ──→ Multiple commands
            └─ Dependencies ──→ Ordered commands
                    │
                    └─→ Submit to Consensus
                        │
                        ├─ Append to log
                        ├─ Execute on all replicas
                        └─ Deterministic results
                            │
                            └─→ Commit ──→ Durable state
                                    │
                                    └─→ Next workflow step
```

### With OTEL Observability

```
Every Consensus Operation:

  tracing::info!(
      node_id = %self.node_id,
      consensus_algo = "pbft" | "hotstuff",
      phase = "preprepare" | "prepare" | "commit",
      sequence = seq,
      view = view,
      latency_ms = elapsed,
      "consensus phase"
  );

Metrics exported:
  ├─ consensus_phases_total (counter by phase)
  ├─ consensus_latency_seconds (histogram by algorithm)
  ├─ validator_reputation (gauge)
  ├─ Byzantine_nodes (gauge)
  └─ state_commit_rate (counter)
```

## Configuration Best Practices

### For Enterprise Deployments (100+ nodes)

```rust
let pbft_config = PBFTConfig::new(99)?;  // 3f+1 = 99
// Tolerates f=32 Byzantine nodes
// Quorum = 67 votes needed

let hotstuff_config = HotStuffConfig::new(99)?;
// Same tolerance, but O(n) messages
```

### For Edge Deployments (4-7 nodes)

```rust
let config = PBFTConfig::new(7)?;  // 3f+1 = 7
// Tolerates f=2 Byzantine nodes
// Quorum = 5 votes needed
// Suitable for branch offices, edge datacenters
```

### For High-Throughput (Blockchain-style)

```rust
let config = HotStuffConfig::new(31)?;  // 3f+1 = 31
// Tolerates f=10 Byzantine nodes
// O(n) = 31 messages per view
// Pipelined: 10x throughput of PBFT
```

---

**Last Updated:** 2025-11-16
**Maintainer:** KNHK Team
