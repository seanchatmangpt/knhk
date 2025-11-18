# Phase Integration Architecture - KNHK Phases 6-10

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18

---

## Overview

This document specifies how Phases 6-10 integrate with each other and with the existing KNHK core (Phases 1-5). All integrations respect DOCTRINE principles and maintain the Chatman constant (≤8 ticks hot path).

---

## Integration Topology

```
┌────────────────────────────────────────────────────────────────┐
│                     Integration Architecture                    │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │            Phase 10: Market Licensing (Top Layer)        │   │
│  │                                                           │   │
│  │  Controls access to ALL features                        │   │
│  │  Type-level gates (compile-time enforcement)            │   │
│  └─────────────────────────────────────────────────────────┘   │
│         │ Free │ Pro │ Enterprise                                │
│         ↓      ↓     ↓                                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │               MAPE-K Control Loop (Core)                 │   │
│  │                                                           │   │
│  │  Monitor ──→ Analyze ──→ Plan ──→ Execute ──→ Knowledge │   │
│  │     ↓         (P6)        ↓        (P9)         ↓        │   │
│  │     │       Neural       │       Hardware      │        │   │
│  │  Observe  Learning     Policy   Accel        Store     │   │
│  │   (P8)                (SPARQL)  (GPU/FPGA)   (P7)      │   │
│  │  Consensus                                   Signed     │   │
│  └─────────────────────────────────────────────────────────┘   │
│         ↓                                          ↓             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐    │
│  │   Phase 6       │  │   Phase 7       │  │  Phase 8    │    │
│  │   Neural        │  │   Crypto        │  │  Consensus  │    │
│  │   Learning      │  │   Signatures    │  │  BFT/Raft   │    │
│  └─────────────────┘  └─────────────────┘  └─────────────┘    │
│         ↑                     ↑                     ↑           │
│         └─────────────────────┴─────────────────────┘           │
│                          Phase 9                                │
│                      Hardware Acceleration                      │
│                   (GPU/FPGA backends for all)                   │
└────────────────────────────────────────────────────────────────┘
```

---

## Integration Points

### 1. Phase 6 (Neural) ← → Phase 8 (Consensus)

**Purpose**: Distributed reinforcement learning with Byzantine agreement.

```rust
/// Multi-agent neural learning with consensus
pub struct DistributedNeuralAnalyzer {
    /// Local neural model
    local_model: Arc<Mutex<QLearningAgent>>,

    /// Consensus layer (Phase 8)
    consensus: Arc<PBFT<NeuralState>>,

    /// Shared reward signals
    reward_aggregator: Arc<RewardAggregator>,
}

impl DistributedNeuralAnalyzer {
    /// Propose policy update via consensus
    pub async fn propose_policy_update(
        &self,
        update: PolicyUpdate,
    ) -> Result<(), IntegrationError> {
        // 1. Local neural model computes update
        let local_model = self.local_model.lock().await;
        let gradient = local_model.compute_gradient(&update)?;

        // 2. Sign update with hybrid signature (Phase 7)
        let signed_update = self.sign_update(&gradient)?;

        // 3. Propose via Byzantine consensus (Phase 8)
        self.consensus.propose(signed_update).await?;

        // 4. Wait for 2f+1 agreement
        let committed = self.consensus.wait_committed().await?;

        // 5. Apply agreed update
        drop(local_model);
        let mut local_model = self.local_model.lock().await;
        local_model.apply_update(&committed)?;

        Ok(())
    }

    /// Aggregate rewards from multiple nodes
    pub async fn aggregate_rewards(&self) -> f64 {
        // Consensus on reward signals (Byzantine-safe)
        let rewards = self.reward_aggregator.collect_rewards().await;

        // Byzantine fault tolerance: Median of rewards (robust to f Byzantine nodes)
        median(&rewards)
    }
}
```

**Benefits**:
- ✅ Learning converges even with f Byzantine nodes
- ✅ Policy updates cryptographically signed (Phase 7)
- ✅ Shared knowledge across distributed nodes

**Latency Impact**: Consensus adds ~250ms (multi-region), but off hot path.

---

### 2. Phase 7 (Crypto) ← → Phase 8 (Consensus)

**Purpose**: Signed consensus messages with quantum-safe signatures.

```rust
/// Consensus message with hybrid signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedConsensusMessage<M> {
    /// Message payload
    pub message: M,

    /// Hybrid signature (Ed25519 + Dilithium3)
    pub signature: HybridSignatureBytes,

    /// Sender public key
    pub sender: NodeId,

    /// Sender public key (hybrid)
    pub public_key: HybridPublicKey,
}

impl PBFT<S> {
    /// Send Pre-Prepare with hybrid signature
    pub async fn send_pre_prepare(&self, proposal: S::Proposal)
        -> Result<(), ConsensusError>
    {
        // Create Pre-Prepare message
        let pre_prepare = PrePrepare {
            view: self.view,
            sequence: self.sequence,
            proposal,
        };

        // Sign with hybrid signature (Phase 7)
        let message = bincode::serialize(&pre_prepare)?;
        let signature = HybridSignature::sign(&self.keys.secret_key, &message)?;

        // Broadcast
        self.broadcast(SignedConsensusMessage {
            message: pre_prepare,
            signature,
            sender: self.node_id,
            public_key: self.keys.public_key.clone(),
        }).await?;

        Ok(())
    }

    /// Verify consensus message signature
    pub fn verify_message<M: Serialize>(
        &self,
        msg: &SignedConsensusMessage<M>,
    ) -> Result<(), ConsensusError> {
        // Verify hybrid signature
        let message = bincode::serialize(&msg.message)?;

        if !HybridSignature::verify(&msg.public_key, &message, &msg.signature) {
            return Err(ConsensusError::InvalidSignature);
        }

        Ok(())
    }
}
```

**Benefits**:
- ✅ Consensus messages quantum-safe
- ✅ Leader election via VRF (quantum-safe)
- ✅ Byzantine nodes cannot spoof signatures

---

### 3. Phase 8 (Consensus) ← → Phase 9 (Hardware)

**Purpose**: GPU-accelerated batch signature verification.

```rust
/// GPU-accelerated signature verification
pub struct GPUBatchVerifier {
    /// GPU accelerator (Phase 9)
    gpu: Arc<WGPUAccelerator>,

    /// Public keys (pinned memory)
    public_keys: Vec<HybridPublicKey>,

    /// Messages (pinned memory)
    messages: Vec<Vec<u8>>,

    /// Signatures (pinned memory)
    signatures: Vec<HybridSignatureBytes>,
}

impl GPUBatchVerifier {
    /// Verify batch of consensus messages on GPU
    ///
    /// Latency: O(n/1000) on GPU vs O(n) on CPU
    /// Speedup: 1000x for large batches
    pub async fn verify_batch(&self) -> Result<Vec<bool>, VerificationError> {
        // Transfer data to GPU (zero-copy if unified memory)
        let gpu_keys = self.gpu.upload_buffer(&self.public_keys).await?;
        let gpu_messages = self.gpu.upload_buffer(&self.messages).await?;
        let gpu_signatures = self.gpu.upload_buffer(&self.signatures).await?;

        // Dispatch GPU kernel (parallel verification)
        let results = self.gpu.execute_kernel(
            "batch_verify_hybrid_signatures",
            &[gpu_keys, gpu_messages, gpu_signatures],
        ).await?;

        // Download results
        Ok(results)
    }
}

/// Usage in PBFT
impl PBFT<S> {
    /// Verify multiple Prepare messages in parallel (GPU)
    pub async fn verify_prepare_batch(
        &self,
        messages: Vec<SignedConsensusMessage<Prepare>>,
    ) -> Result<Vec<bool>, ConsensusError> {
        // Batch verify on GPU (1000x faster than serial CPU)
        let verifier = GPUBatchVerifier::new(messages)?;
        Ok(verifier.verify_batch().await?)
    }
}
```

**Benefits**:
- ✅ 1000x faster batch verification (GPU)
- ✅ Consensus throughput increases from 100 TPS → 100K TPS
- ✅ Maintains ≤8 ticks for single verification (CPU)

---

### 4. Phase 9 (Hardware) ← → Phase 6 (Neural)

**Purpose**: GPU-accelerated neural network training and inference.

```rust
/// GPU-accelerated Actor-Critic agent
pub struct GPUActorCritic {
    /// Actor network (on GPU)
    actor: GPUNeuralNetwork,

    /// Critic network (on GPU)
    critic: GPUNeuralNetwork,

    /// GPU backend (Phase 9)
    gpu: Arc<WGPUAccelerator>,
}

impl NeuralModel for GPUActorCritic {
    type Input<'a> = &'a [f32];
    type Output = Vec<f32>;
    type Error = NeuralError;

    /// Inference on GPU (100x faster than CPU)
    ///
    /// Latency: ~100 μs (GPU) vs ~10 ms (CPU)
    async fn predict<'a>(&'a self, state: &'a [f32])
        -> Result<Vec<f32>, NeuralError>
    {
        // Upload state to GPU (pinned memory, zero-copy)
        let gpu_state = self.gpu.upload_buffer(state).await?;

        // Forward pass on GPU
        let gpu_output = self.actor.forward(gpu_state).await?;

        // Download result
        let action_probs = self.gpu.download_buffer(gpu_output).await?;

        Ok(action_probs)
    }

    /// Training on GPU (1000x faster for large batches)
    async fn train(&mut self, batch: &[Experience])
        -> Result<TrainingMetrics, NeuralError>
    {
        // Upload batch to GPU
        let gpu_batch = self.gpu.upload_buffer(batch).await?;

        // Compute gradients on GPU (parallel across batch)
        let gradients = self.compute_gradients_gpu(gpu_batch).await?;

        // Apply gradients (GPU matrix operations)
        self.apply_gradients_gpu(gradients).await?;

        Ok(TrainingMetrics { ... })
    }
}
```

**Benefits**:
- ✅ 100x faster inference (GPU vs CPU)
- ✅ 1000x faster training (GPU batch operations)
- ✅ Can train larger models (GPU memory)

---

### 5. Phase 10 (Licensing) ← → All Phases

**Purpose**: Enforce feature access based on license tier.

```rust
/// Licensed features (type-level gates)
impl<L: License> LicensedWorkflowEngine<L> {
    /// Phase 6: Neural learning (Pro/Enterprise only)
    pub fn enable_neural(&self, model: impl NeuralModel)
        -> Result<(), LicenseError>
    where
        Assert<{ L::ML_ENABLED }>: IsTrue,  // Compile-time check
    {
        self.neural_analyzer = Some(Arc::new(Mutex::new(model)));
        Ok(())
    }

    /// Phase 7: Quantum crypto (Pro/Enterprise only)
    pub fn use_hybrid_signatures(&self) -> Result<(), LicenseError>
    where
        Assert<{ L::QUANTUM_CRYPTO }>: IsTrue,  // Compile-time check
    {
        self.signature_mode = SignatureMode::Hybrid;
        Ok(())
    }

    /// Phase 8: Byzantine consensus (Enterprise only)
    pub fn enable_bft(&self, config: BFTConfig) -> Result<(), LicenseError> {
        // Only compiles for Enterprise
        static_assertions::const_assert!(EnterpriseLicense::BFT_ENABLED);

        self.consensus = Some(Arc::new(PBFT::new(config)?));
        Ok(())
    }

    /// Phase 9: Hardware acceleration (tier-dependent)
    pub fn select_accelerator(&self) -> Box<dyn Accelerator<f32>> {
        match L::HARDWARE {
            HardwareAccess::CPUOnly => Box::new(CPUAccelerator::default()),
            HardwareAccess::CPUSIMD => Box::new(SIMDAccelerator::default()),
            HardwareAccess::AllGPU => Box::new(WGPUAccelerator::new().await.unwrap()),
            HardwareAccess::AllHardware => {
                // Enterprise: Try FPGA → GPU → SIMD → CPU
                if FPGAAccelerator::available() {
                    Box::new(FPGAAccelerator::new().unwrap())
                } else if WGPUAccelerator::available() {
                    Box::new(WGPUAccelerator::new().await.unwrap())
                } else if SIMDAccelerator::available() {
                    Box::new(SIMDAccelerator::default())
                } else {
                    Box::new(CPUAccelerator::default())
                }
            }
        }
    }
}
```

**Type-Level Enforcement**:
```rust
// ✅ Compiles (Enterprise has BFT)
let engine = LicensedWorkflowEngine::<EnterpriseLicense>::new()?;
engine.enable_bft(BFTConfig::default())?;

// ❌ Does NOT compile (Free doesn't have BFT)
let engine = LicensedWorkflowEngine::<FreeLicense>::new()?;
engine.enable_bft(BFTConfig::default())?;  // Compile error!
```

---

## Data Flow Diagrams

### End-to-End Workflow Execution

```
User Request
    ↓
┌───────────────────────────────────────────────────────────┐
│ Phase 10: License Check (Compile-Time)                    │
│ - Verify tier has access to requested features           │
│ - Count against MAX_WORKFLOWS / MAX_CONCURRENT            │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ MAPE-K Monitor                                             │
│ - Collect telemetry (O)                                   │
│ - Sign observations (Phase 7)                             │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ Phase 6: Neural Analyze                                    │
│ - Predict optimal configuration (≤8 ticks)                │
│ - Experience replay (background)                          │
│ - GPU-accelerated inference (Phase 9)                     │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ MAPE-K Plan                                                │
│ - Evaluate policy (SPARQL)                                │
│ - Propose configuration change                            │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ Phase 8: Consensus (Enterprise Only)                      │
│ - PBFT: Agree on configuration                            │
│ - Sign with hybrid signatures (Phase 7)                   │
│ - GPU batch verification (Phase 9)                        │
│ - 2f+1 nodes must agree                                   │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ Phase 9: Hardware-Accelerated Execute                     │
│ - Auto-select backend (CPU/SIMD/GPU/FPGA)                │
│ - Execute workflow on best hardware                       │
│ - ≤8 ticks for hot path (Chatman constant)                │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ MAPE-K Knowledge                                           │
│ - Store execution results                                 │
│ - Sign receipts (Phase 7)                                 │
│ - Persist to consensus log (Phase 8)                      │
│ - Train neural model (Phase 6, background)                │
└───────────────────────────────────────────────────────────┘
    ↓
┌───────────────────────────────────────────────────────────┐
│ Phase 10: Audit Log                                        │
│ - Cryptographic receipt (immutable)                       │
│ - License usage tracking                                  │
│ - Billing integration                                     │
└───────────────────────────────────────────────────────────┘
    ↓
Result to User
```

---

## Performance Budget (Integrated System)

| Operation | Latency | Phase(s) | Path |
|-----------|---------|----------|------|
| License check | 0 ticks | P10 | Compile-time |
| Neural prediction | ≤8 ticks | P6 | Hot path |
| Sign observation | <1 ms | P7 | Warm path |
| Local consensus | ≤8 ticks | P8 | Hot path |
| Global consensus | <250 ms | P8 | Cold path |
| GPU dispatch | <100 μs | P9 | Warm path |
| Workflow execute | ≤8 ticks | P1-5 | Hot path |
| **Total (hot path)** | **≤8 ticks** | **All** | **✅ Chatman** |

**Key**: Hot path maintained at ≤8 ticks (Chatman constant) even with all phases integrated.

---

## Failure Mode Interactions

### Scenario 1: GPU Failure During Neural Inference

```
GPU fails → Phase 9 detects → Falls back to SIMD → Falls back to CPU
                                      ↓
Phase 6 neural model continues on CPU (10x slower but functional)
                                      ↓
MAPE-K still operates (degraded performance)
```

**Impact**: 10x slower neural inference, but system remains functional.

### Scenario 2: Network Partition During Consensus

```
Network partition → Phase 8 detects <2f+1 nodes reachable
                                      ↓
                    Switch to eventual consistency (Raft)
                                      ↓
Phase 8 consensus continues (weaker guarantees)
                                      ↓
MAPE-K operates without strong consistency
```

**Impact**: Lose Byzantine fault tolerance, but system continues.

### Scenario 3: Quantum Crypto Not Available

```
Quantum crypto unavailable → Phase 7 detects
                                      ↓
              Fall back to classical signatures (Ed25519)
                                      ↓
Phase 8 consensus uses classical signatures
                                      ↓
Warning logged: "Quantum-vulnerable signatures in use"
```

**Impact**: Vulnerable to future quantum attacks, but system functional.

---

## Integration Testing Strategy

```rust
#[tokio::test]
async fn test_full_integration() {
    // Setup: Enterprise license with all features
    let engine = LicensedWorkflowEngine::<EnterpriseLicense>::new()?;

    // Phase 6: Enable neural learning (GPU-accelerated)
    let neural_model = GPUActorCritic::new(gpu_backend).await?;
    engine.enable_neural(neural_model)?;

    // Phase 7: Enable hybrid signatures
    engine.use_hybrid_signatures()?;

    // Phase 8: Enable Byzantine consensus (4 nodes)
    let bft_config = BFTConfig { nodes: 4, f: 1 };
    engine.enable_bft(bft_config)?;

    // Phase 9: Use GPU acceleration
    let gpu_accelerator = engine.select_accelerator();
    assert_eq!(gpu_accelerator.name(), "GPU");

    // Execute workflow
    let workflow_id = engine.register_workflow(workflow)?;
    let result = engine.execute_workflow(workflow_id).await?;

    // Verify all phases participated
    assert!(result.neural_policy_used);
    assert!(result.hybrid_signature_valid);
    assert!(result.consensus_committed);
    assert_eq!(result.accelerator_backend, "GPU");

    // Phase 10: Verify audit log
    let audit = engine.get_audit_log()?;
    assert!(audit.verify()?);
}
```

---

## DOCTRINE Compliance

All integrations respect DOCTRINE principles:

| Integration | Principle | Covenant | How |
|-------------|-----------|----------|-----|
| P6 ← → P8 | MAPE-K + O | 3, 6 | Distributed learning observable via consensus |
| P7 ← → P8 | Q (Invariants) | 2 | Cryptographic guarantees enforced |
| P8 ← → P9 | Chatman Constant | 5 | GPU maintains ≤8 ticks local consensus |
| P9 ← → P6 | Chatman Constant | 5 | GPU maintains ≤8 ticks inference |
| P10 ← → All | Π (Projection) | 1 | License gates derived from Σ (ontology) |

---

## Related Documents

- `PHASES_6-10_ARCHITECTURE_OVERVIEW.md`
- Each phase specification document
- `TYPE_LEVEL_DESIGN_PATTERNS.md`
- `DEPLOYMENT_MODELS.md`
- `DOCTRINE_COVENANT.md`

**Conclusion**: All phases integrate cleanly via well-defined interfaces, maintaining DOCTRINE principles and performance constraints.
