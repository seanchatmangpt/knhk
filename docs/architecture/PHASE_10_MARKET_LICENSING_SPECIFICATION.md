# Phase 10: Market Licensing - Detailed Specification

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18
**Phase Duration**: 4 weeks | **LOC Estimate**: ~5,000 lines

---

## DOCTRINE Alignment

**Principle**: Π (Projection Layer) - "Projections derive from Σ definition"
**Covenant**: Covenant 1 (Turtle Is Definition and Cause)
**Why This Matters**: License tiers are projections from the ontology - features are declared in RDF, licenses control access at compile-time.

**What This Means**:
Phase 10 implements type-level license enforcement where feature gates are checked at compile-time, not runtime. All licensing rules derive from Turtle ontology definitions.

**Anti-Patterns to Avoid**:
- ❌ Runtime license checks on hot path (must be compile-time)
- ❌ Mutable license state (must be immutable)
- ❌ License violations that compile (type system must prevent)
- ❌ License terms not in ontology (must be in Turtle)
- ❌ Audit logs that can be tampered with (cryptographic receipts)

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│              Phase 10: Market Licensing System                  │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           License Trait (Type-Level)                     │   │
│  │                                                           │   │
│  │  trait License {                                        │   │
│  │    const MAX_WORKFLOWS: u32;                            │   │
│  │    const MAX_CONCURRENT: u32;                           │   │
│  │    const HARDWARE: HardwareAccess;                      │   │
│  │    const SUPPORT_SLA: Duration;                         │   │
│  │  }                                                       │   │
│  │                                                           │   │
│  │  Type-level feature gates prevent:                     │   │
│  │  - Compiling Enterprise code with Free license         │   │
│  │  - Accessing GPU/FPGA without Enterprise tier          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Free Tier   │  │   Pro Tier   │  │ Enterprise   │         │
│  │              │  │              │  │    Tier      │         │
│  │ • 10 wflows  │  │ • 100 wflows │  │ • Unlimited  │         │
│  │ • 1 conc     │  │ • 10 conc    │  │ • Unlimited  │         │
│  │ • CPU only   │  │ • CPU+SIMD   │  │ • All HW     │         │
│  │ • 24h support│  │ • 4h support │  │ • 1h support │         │
│  │ • No BFT     │  │ • No BFT     │  │ • Full BFT   │         │
│  │ • No ML      │  │ • Basic ML   │  │ • Full ML    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │            Cost Accounting (Type-Level)                  │   │
│  │                                                           │   │
│  │  struct Cost<L: License> {                              │   │
│  │    workflows: Counter<L::MAX_WORKFLOWS>,                │   │
│  │    concurrent: Semaphore<L::MAX_CONCURRENT>,            │   │
│  │  }                                                       │   │
│  │                                                           │   │
│  │  Type system prevents exceeding limits at compile-time │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Audit Trail (Cryptographic Receipts)             │   │
│  │                                                           │   │
│  │  Every feature access → Signed receipt                  │   │
│  │  Immutable append-only log                              │   │
│  │  Hybrid signatures (Phase 7)                            │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Core Type Definitions

### 1. License Trait (Associated Constants)

```rust
/// License tier trait (marker types)
///
/// Uses associated constants for compile-time enforcement.
pub trait License: 'static + Send + Sync {
    /// Maximum number of workflows
    const MAX_WORKFLOWS: u32;

    /// Maximum concurrent executions
    const MAX_CONCURRENT: u32;

    /// Hardware access level
    const HARDWARE: HardwareAccess;

    /// Support SLA (response time)
    const SUPPORT_SLA: Duration;

    /// Byzantine consensus enabled
    const BFT_ENABLED: bool;

    /// Neural learning enabled
    const ML_ENABLED: bool;

    /// Quantum-safe crypto enabled
    const QUANTUM_CRYPTO: bool;

    /// License tier name
    const TIER_NAME: &'static str;

    /// License expiration (Unix timestamp)
    const EXPIRATION: Option<u64>;
}

/// Hardware access levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareAccess {
    /// CPU only
    CPUOnly,
    /// CPU + SIMD
    CPUSIMD,
    /// CPU + SIMD + GPU
    AllGPU,
    /// All hardware (CPU/SIMD/GPU/FPGA)
    AllHardware,
}
```

### 2. License Tier Implementations

```rust
/// Free tier (limited features)
#[derive(Debug, Clone, Copy)]
pub struct FreeLicense;

impl License for FreeLicense {
    const MAX_WORKFLOWS: u32 = 10;
    const MAX_CONCURRENT: u32 = 1;
    const HARDWARE: HardwareAccess = HardwareAccess::CPUOnly;
    const SUPPORT_SLA: Duration = Duration::from_secs(24 * 3600); // 24 hours
    const BFT_ENABLED: bool = false;
    const ML_ENABLED: bool = false;
    const QUANTUM_CRYPTO: bool = false;
    const TIER_NAME: &'static str = "Free";
    const EXPIRATION: Option<u64> = None; // No expiration
}

/// Pro tier (professional features)
#[derive(Debug, Clone, Copy)]
pub struct ProLicense;

impl License for ProLicense {
    const MAX_WORKFLOWS: u32 = 100;
    const MAX_CONCURRENT: u32 = 10;
    const HARDWARE: HardwareAccess = HardwareAccess::CPUSIMD;
    const SUPPORT_SLA: Duration = Duration::from_secs(4 * 3600); // 4 hours
    const BFT_ENABLED: bool = false;
    const ML_ENABLED: bool = true; // Basic ML
    const QUANTUM_CRYPTO: bool = true; // Hybrid signatures
    const TIER_NAME: &'static str = "Pro";
    const EXPIRATION: Option<u64> = Some(1735689600); // 2025-01-01
}

/// Enterprise tier (unlimited features)
#[derive(Debug, Clone, Copy)]
pub struct EnterpriseLicense;

impl License for EnterpriseLicense {
    const MAX_WORKFLOWS: u32 = u32::MAX; // Unlimited
    const MAX_CONCURRENT: u32 = u32::MAX; // Unlimited
    const HARDWARE: HardwareAccess = HardwareAccess::AllHardware;
    const SUPPORT_SLA: Duration = Duration::from_secs(3600); // 1 hour
    const BFT_ENABLED: bool = true; // Full Byzantine consensus
    const ML_ENABLED: bool = true; // Full ML (all algorithms)
    const QUANTUM_CRYPTO: bool = true; // All PQC algorithms
    const TIER_NAME: &'static str = "Enterprise";
    const EXPIRATION: Option<u64> = None; // Contract-based
}
```

### 3. Type-Level Feature Gates

```rust
/// Licensed workflow engine
///
/// Generic over license type - features gated at compile-time.
pub struct LicensedWorkflowEngine<L: License> {
    /// License marker (zero-sized)
    _license: PhantomData<L>,

    /// Workflow counter (enforces MAX_WORKFLOWS)
    workflow_count: AtomicU32,

    /// Concurrency semaphore (enforces MAX_CONCURRENT)
    concurrency: Arc<Semaphore>,

    /// Core engine
    engine: Arc<WorkflowEngine>,

    /// Audit log (immutable)
    audit_log: Arc<AuditLog<L>>,
}

impl<L: License> LicensedWorkflowEngine<L> {
    /// Create new licensed engine
    pub fn new() -> Result<Self, LicenseError> {
        // Check expiration
        if let Some(expiration) = L::EXPIRATION {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now > expiration {
                return Err(LicenseError::Expired);
            }
        }

        Ok(Self {
            _license: PhantomData,
            workflow_count: AtomicU32::new(0),
            concurrency: Arc::new(Semaphore::new(L::MAX_CONCURRENT as usize)),
            engine: Arc::new(WorkflowEngine::new()),
            audit_log: Arc::new(AuditLog::new()),
        })
    }

    /// Register workflow (enforces MAX_WORKFLOWS)
    pub fn register_workflow(&self, workflow: Workflow)
        -> Result<WorkflowId, LicenseError>
    {
        // Atomic increment and check
        let count = self.workflow_count.fetch_add(1, Ordering::SeqCst);

        if count >= L::MAX_WORKFLOWS {
            self.workflow_count.fetch_sub(1, Ordering::SeqCst);
            return Err(LicenseError::MaxWorkflowsExceeded);
        }

        // Audit log
        self.audit_log.log_event(AuditEvent::WorkflowRegistered {
            workflow_id: workflow.id,
            timestamp: current_timestamp(),
        })?;

        // Delegate to core engine
        Ok(self.engine.register_workflow(workflow)?)
    }

    /// Execute workflow (enforces MAX_CONCURRENT)
    pub async fn execute_workflow(&self, workflow_id: WorkflowId)
        -> Result<ExecutionResult, LicenseError>
    {
        // Acquire concurrency permit
        let _permit = self.concurrency.acquire().await
            .map_err(|_| LicenseError::MaxConcurrencyExceeded)?;

        // Audit log
        self.audit_log.log_event(AuditEvent::WorkflowExecuted {
            workflow_id,
            timestamp: current_timestamp(),
        })?;

        // Delegate to core engine
        Ok(self.engine.execute_workflow(workflow_id).await?)
    }
}

/// Enterprise-only: Enable Byzantine consensus
impl LicensedWorkflowEngine<EnterpriseLicense> {
    /// Enable BFT consensus (Enterprise only)
    ///
    /// This method only exists for EnterpriseLicense.
    /// Attempting to call it with FreeLicense or ProLicense
    /// will fail at compile-time.
    pub fn enable_bft(&self, config: BFTConfig) -> Result<(), LicenseError> {
        // Only compiles for Enterprise tier
        static_assertions::const_assert!(EnterpriseLicense::BFT_ENABLED);

        self.engine.enable_bft(config)?;
        Ok(())
    }

    /// Use FPGA acceleration (Enterprise only)
    pub fn use_fpga(&self) -> Result<FPGAAccelerator, LicenseError> {
        // Only compiles for Enterprise tier
        static_assertions::const_assert_eq!(
            EnterpriseLicense::HARDWARE,
            HardwareAccess::AllHardware
        );

        Ok(FPGAAccelerator::new()?)
    }
}

/// Pro/Enterprise: Enable neural learning
impl<L: License> LicensedWorkflowEngine<L>
where
    Assert<{ L::ML_ENABLED }>: IsTrue,
{
    /// Enable neural learning (Pro/Enterprise only)
    ///
    /// Only compiles if L::ML_ENABLED == true.
    pub fn enable_neural(&self, model: NeuralModel) -> Result<(), LicenseError> {
        self.engine.enable_neural(model)?;
        Ok(())
    }
}

/// Type-level assertion helper
pub struct Assert<const CHECK: bool>;
pub trait IsTrue {}
impl IsTrue for Assert<true> {}
```

### 4. Audit Log (Cryptographic Receipts)

```rust
use phase7::HybridSignature;

/// Immutable audit log with cryptographic receipts
pub struct AuditLog<L: License> {
    /// Append-only event log
    events: Arc<RwLock<Vec<SignedAuditEvent>>>,

    /// License tier
    _license: PhantomData<L>,

    /// Signing keys
    keys: Arc<ConsensusKeys>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Workflow registered
    WorkflowRegistered {
        workflow_id: WorkflowId,
        timestamp: u64,
    },

    /// Workflow executed
    WorkflowExecuted {
        workflow_id: WorkflowId,
        timestamp: u64,
    },

    /// Feature accessed
    FeatureAccessed {
        feature: Feature,
        timestamp: u64,
    },

    /// License violation attempted
    LicenseViolation {
        violation: LicenseError,
        timestamp: u64,
    },
}

/// Feature types (for audit logging)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Feature {
    ByzantineConsensus,
    NeuralLearning,
    GPUAcceleration,
    FPGAAcceleration,
    QuantumCrypto,
}

/// Signed audit event (tamper-proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    /// Event data
    pub event: AuditEvent,

    /// Hybrid signature
    pub signature: HybridSignatureBytes,

    /// Timestamp
    pub timestamp: u64,

    /// Event index (sequential)
    pub index: u64,

    /// Previous event hash (blockchain-style)
    pub prev_hash: [u8; 32],
}

impl<L: License> AuditLog<L> {
    /// Log audit event (append-only)
    pub fn log_event(&self, event: AuditEvent) -> Result<(), LicenseError> {
        let mut events = self.events.write();

        // Get previous hash
        let prev_hash = events.last()
            .map(|e| hash_event(e))
            .unwrap_or([0u8; 32]);

        // Create signed event
        let timestamp = current_timestamp();
        let index = events.len() as u64;

        let message = bincode::serialize(&(&event, timestamp, index, prev_hash))?;
        let signature = HybridSignature::sign(&self.keys.secret_key, &message)?;

        let signed_event = SignedAuditEvent {
            event,
            signature,
            timestamp,
            index,
            prev_hash,
        };

        // Append to log (immutable)
        events.push(signed_event);

        Ok(())
    }

    /// Verify audit log integrity
    pub fn verify(&self) -> Result<bool, LicenseError> {
        let events = self.events.read();

        for (i, event) in events.iter().enumerate() {
            // Verify signature
            let message = bincode::serialize(&(
                &event.event,
                event.timestamp,
                event.index,
                event.prev_hash,
            ))?;

            if !HybridSignature::verify(&self.keys.public_key, &message, &event.signature) {
                return Ok(false);
            }

            // Verify chain
            if i > 0 {
                let prev_hash = hash_event(&events[i - 1]);
                if event.prev_hash != prev_hash {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}
```

---

## License Ontology (RDF Turtle)

```turtle
# ontology/licenses.ttl
@prefix knhk: <https://knhk.io/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

knhk:FreeLicense a knhk:LicenseTier ;
    knhk:maxWorkflows 10 ;
    knhk:maxConcurrent 1 ;
    knhk:hardwareAccess knhk:CPUOnly ;
    knhk:supportSLA "PT24H"^^xsd:duration ;
    knhk:bftEnabled false ;
    knhk:mlEnabled false ;
    knhk:quantumCrypto false ;
    knhk:tierName "Free" .

knhk:ProLicense a knhk:LicenseTier ;
    knhk:maxWorkflows 100 ;
    knhk:maxConcurrent 10 ;
    knhk:hardwareAccess knhk:CPUSIMD ;
    knhk:supportSLA "PT4H"^^xsd:duration ;
    knhk:bftEnabled false ;
    knhk:mlEnabled true ;
    knhk:quantumCrypto true ;
    knhk:tierName "Pro" ;
    knhk:expirationDate "2025-01-01"^^xsd:date .

knhk:EnterpriseLicense a knhk:LicenseTier ;
    knhk:maxWorkflows -1 ;  # Unlimited
    knhk:maxConcurrent -1 ;  # Unlimited
    knhk:hardwareAccess knhk:AllHardware ;
    knhk:supportSLA "PT1H"^^xsd:duration ;
    knhk:bftEnabled true ;
    knhk:mlEnabled true ;
    knhk:quantumCrypto true ;
    knhk:tierName "Enterprise" .
```

**Code Generation**: License Rust code is generated from this Turtle definition.

---

## Cost Accounting

```rust
/// Type-level cost accounting
///
/// Prevents exceeding license limits at compile-time.
pub struct Cost<L: License> {
    /// Workflow count (bounded by L::MAX_WORKFLOWS)
    workflows: BoundedCounter<{ L::MAX_WORKFLOWS }>,

    /// Concurrency semaphore (bounded by L::MAX_CONCURRENT)
    concurrent: BoundedSemaphore<{ L::MAX_CONCURRENT }>,
}

/// Bounded counter (const generic)
pub struct BoundedCounter<const MAX: u32> {
    count: AtomicU32,
}

impl<const MAX: u32> BoundedCounter<MAX> {
    /// Increment (fails if would exceed MAX)
    pub fn increment(&self) -> Result<(), LicenseError> {
        let count = self.count.fetch_add(1, Ordering::SeqCst);

        if count >= MAX {
            self.count.fetch_sub(1, Ordering::SeqCst);
            Err(LicenseError::LimitExceeded)
        } else {
            Ok(())
        }
    }

    /// Decrement
    pub fn decrement(&self) {
        self.count.fetch_sub(1, Ordering::SeqCst);
    }
}
```

---

## OpenTelemetry Schema

```yaml
# registry/phases_6_10/licensing.yaml
spans:
  - span_name: license.check
    attributes:
      - name: tier
        type: string
        values: [free, pro, enterprise]
      - name: feature
        type: string
      - name: allowed
        type: boolean

  - span_name: license.violation
    attributes:
      - name: tier
        type: string
      - name: limit_type
        type: string
        values: [workflows, concurrency, hardware, feature]

metrics:
  - metric_name: license.usage.workflows
    instrument: gauge
    unit: workflows
    description: "Current number of registered workflows"

  - metric_name: license.usage.concurrent
    instrument: gauge
    unit: executions
    description: "Current concurrent executions"
```

---

## Testing Strategy

```rust
#[test]
fn test_free_license_limits() {
    let engine = LicensedWorkflowEngine::<FreeLicense>::new().unwrap();

    // Can register up to 10 workflows
    for i in 0..10 {
        let workflow = Workflow::new(format!("workflow_{}", i));
        engine.register_workflow(workflow).unwrap();
    }

    // 11th workflow should fail
    let workflow = Workflow::new("workflow_11");
    assert!(matches!(
        engine.register_workflow(workflow),
        Err(LicenseError::MaxWorkflowsExceeded)
    ));
}

#[test]
fn test_enterprise_bft_access() {
    let engine = LicensedWorkflowEngine::<EnterpriseLicense>::new().unwrap();

    // This compiles (Enterprise has BFT)
    engine.enable_bft(BFTConfig::default()).unwrap();
}

#[test]
#[should_not_compile]
fn test_free_bft_access() {
    let engine = LicensedWorkflowEngine::<FreeLicense>::new().unwrap();

    // This does NOT compile (Free doesn't have BFT)
    engine.enable_bft(BFTConfig::default()).unwrap();
}
```

---

## Migration from Free to Pro

```rust
/// Upgrade license tier
pub fn upgrade_license<OldL: License, NewL: License>(
    old_engine: LicensedWorkflowEngine<OldL>,
) -> Result<LicensedWorkflowEngine<NewL>, LicenseError>
where
    Assert<{ NewL::MAX_WORKFLOWS >= OldL::MAX_WORKFLOWS }>: IsTrue,
    Assert<{ NewL::MAX_CONCURRENT >= OldL::MAX_CONCURRENT }>: IsTrue,
{
    // Type system ensures new license is strictly more permissive
    LicensedWorkflowEngine::<NewL>::new()
}

// Example usage:
let free_engine = LicensedWorkflowEngine::<FreeLicense>::new().unwrap();
let pro_engine = upgrade_license::<FreeLicense, ProLicense>(free_engine).unwrap();

// Cannot downgrade (compile error):
// let free_engine = upgrade_license::<ProLicense, FreeLicense>(pro_engine).unwrap();
```

---

## Audit Trail Example

```
Audit Log for Enterprise License (License ID: ent-2025-001)
─────────────────────────────────────────────────────────

[0] 2025-11-18 10:00:00 UTC
    Event: WorkflowRegistered { workflow_id: "wf-001", timestamp: 1731924000 }
    Signature: 0x4a3f... (Ed25519+Dilithium3)
    Prev Hash: 0x0000...

[1] 2025-11-18 10:05:00 UTC
    Event: FeatureAccessed { feature: ByzantineConsensus, timestamp: 1731924300 }
    Signature: 0x7b2e... (Ed25519+Dilithium3)
    Prev Hash: 0x4a3f...

[2] 2025-11-18 10:10:00 UTC
    Event: WorkflowExecuted { workflow_id: "wf-001", timestamp: 1731924600 }
    Signature: 0x9c1d... (Ed25519+Dilithium3)
    Prev Hash: 0x7b2e...

Audit Log Verification: ✅ VALID (all signatures verified, chain intact)
```

---

## Related Documents

- `PHASES_6-10_ARCHITECTURE_OVERVIEW.md`
- `ADR/ADR-007-licensing-model.md`
- `DOCTRINE_COVENANT.md` - Covenant 1 (Turtle Is Definition)
- `ontology/licenses.ttl` - License tier definitions

---

**Architecture Design Complete**: All 5 phases (6-10) specified in detail.
**Next**: Review ADRs, type-level patterns, and integration architecture.
