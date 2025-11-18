# Phase 7: Quantum-Safe Cryptography - Detailed Specification

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18
**Phase Duration**: 6 weeks | **LOC Estimate**: ~8,000 lines

---

## DOCTRINE Alignment

**Principle**: Q (Hard Invariants) - "Total Quality Leadership" as executable law
**Covenant**: Covenant 2 (Invariants Are Law)
**Why This Matters**: Cryptographic guarantees are the ultimate Q invariants - mathematically provable security properties that cannot be bypassed.

**What This Means**:
Phase 7 implements post-quantum cryptography (PQC) to future-proof KNHK against quantum computing threats. Uses NIST-standardized algorithms with hybrid mode (classical + quantum-safe) for gradual migration.

**Anti-Patterns to Avoid**:
- ❌ Non-constant-time implementations (timing attacks)
- ❌ Quantum-only signatures without classical fallback (breaks existing systems)
- ❌ Mutable key material (must be sealed/zeroized)
- ❌ Signatures without verification (false positive validation)
- ❌ Blocking hot path with signature generation (must be async)

---

## Architecture Overview

### Hybrid Signature System

```
┌────────────────────────────────────────────────────────────────┐
│              Phase 7: Quantum-Safe Cryptography                 │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │             Hybrid Signature Trait                       │   │
│  │                                                           │   │
│  │  trait Signature<K: KeyCategory> {                      │   │
│  │    fn sign(&self, msg: &[u8]) -> Result<Sig, Error>;   │   │
│  │    fn verify(key: &K, msg: &[u8], sig: &Sig) -> bool;  │   │
│  │  }                                                       │   │
│  │                                                           │   │
│  │  Phantom type K enforces:                               │   │
│  │  - Classical | Hybrid | Quantum (compile-time)          │   │
│  │  - Sealed trait (no external impls)                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Kyber KEM  │  │  Dilithium   │  │    Falcon    │         │
│  │              │  │  Signature   │  │  Signature   │         │
│  │ • Key encap  │  │              │  │              │         │
│  │ • 512/768/   │  │ • Level 2/3/5│  │ • 512/1024   │         │
│  │   1024       │  │ • Lattice    │  │ • NTRU       │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────────────────────────────┐   │
│  │  SLH-DSA     │  │      Hybrid Mode (Default)           │   │
│  │  (SPHINCS+)  │  │                                       │   │
│  │              │  │  Sign:   Ed25519 || Dilithium3        │   │
│  │ • Stateless  │  │  Verify: Both must pass              │   │
│  │ • Hash-based │  │  Size:   64 bytes + 2420 bytes       │   │
│  │ • Conservative│ │  Latency: <1ms                       │   │
│  └──────────────┘  └──────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           Migration Path (Type-Level)                    │   │
│  │                                                           │   │
│  │  1. Classical only: Ed25519                             │   │
│  │  2. Hybrid: Ed25519 + Dilithium3                        │   │
│  │  3. Quantum-only: Dilithium3                            │   │
│  │                                                           │   │
│  │  Type system prevents:                                  │   │
│  │  - Quantum-only signing classical-only messages         │   │
│  │  - Classical-only verifying quantum-signed messages     │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Core Type Definitions

### 1. Key Category System (Phantom Types)

```rust
/// Key category marker (phantom types)
///
/// These are zero-sized types used to enforce signature
/// compatibility at compile time.
pub mod key_category {
    use core::marker::PhantomData;

    /// Classical cryptography (Ed25519, ECDSA)
    #[derive(Debug)]
    pub struct Classical;

    /// Hybrid mode (Classical + Quantum)
    #[derive(Debug)]
    pub struct Hybrid;

    /// Quantum-safe only (Dilithium, Falcon, SLH-DSA)
    #[derive(Debug)]
    pub struct Quantum;

    /// Sealed trait - prevents external implementations
    mod sealed {
        pub trait Sealed {}
        impl Sealed for super::Classical {}
        impl Sealed for super::Hybrid {}
        impl Sealed for super::Quantum {}
    }

    /// Key category trait
    ///
    /// Sealed to prevent external key types.
    pub trait KeyCategory: sealed::Sealed + 'static {
        /// Algorithm identifier
        const ALGORITHM: &'static str;

        /// Public key size in bytes
        const PUBLIC_KEY_SIZE: usize;

        /// Signature size in bytes
        const SIGNATURE_SIZE: usize;

        /// Security level (NIST)
        const SECURITY_LEVEL: u8;
    }

    impl KeyCategory for Classical {
        const ALGORITHM: &'static str = "Ed25519";
        const PUBLIC_KEY_SIZE: usize = 32;
        const SIGNATURE_SIZE: usize = 64;
        const SECURITY_LEVEL: u8 = 1; // ~128 bits classical
    }

    impl KeyCategory for Hybrid {
        const ALGORITHM: &'static str = "Ed25519+Dilithium3";
        const PUBLIC_KEY_SIZE: usize = 32 + 1952; // Ed25519 + Dilithium3
        const SIGNATURE_SIZE: usize = 64 + 3293; // Combined
        const SECURITY_LEVEL: u8 = 3; // ~192 bits classical + quantum
    }

    impl KeyCategory for Quantum {
        const ALGORITHM: &'static str = "Dilithium3";
        const PUBLIC_KEY_SIZE: usize = 1952;
        const SIGNATURE_SIZE: usize = 3293;
        const SECURITY_LEVEL: u8 = 3; // NIST Level 3
    }
}
```

### 2. Signature Trait

```rust
use key_category::KeyCategory;

/// Generic signature trait parameterized by key category
///
/// Type-level guarantees:
/// - Classical keys cannot verify Quantum signatures
/// - Hybrid mode requires both signatures to pass
/// - All operations are constant-time
pub trait Signature: Sized {
    /// Key category
    type Category: KeyCategory;

    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Public key type
    type PublicKey: Clone + Zeroize;

    /// Secret key type (automatically zeroized on drop)
    type SecretKey: Clone + Zeroize;

    /// Signature bytes
    type Bytes: AsRef<[u8]> + Clone;

    /// Generate new keypair
    ///
    /// Latency: <10 ms (off critical path)
    /// Telemetry: "crypto.keygen" span
    fn generate_keypair() -> Result<(Self::PublicKey, Self::SecretKey), Self::Error>;

    /// Sign message
    ///
    /// Latency: <1 ms (warm path)
    /// Telemetry: "crypto.sign" span
    /// CRITICAL: Must be constant-time to prevent timing attacks
    fn sign(secret_key: &Self::SecretKey, message: &[u8])
        -> Result<Self::Bytes, Self::Error>;

    /// Verify signature
    ///
    /// Latency: <1 ms (warm path)
    /// Telemetry: "crypto.verify" span
    /// CRITICAL: Must be constant-time
    fn verify(
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Bytes,
    ) -> bool;

    /// Export public key (DER format)
    fn export_public_key(key: &Self::PublicKey) -> Vec<u8>;

    /// Import public key (DER format)
    fn import_public_key(data: &[u8]) -> Result<Self::PublicKey, Self::Error>;
}
```

### 3. Hybrid Signature Implementation

```rust
use pqcrypto_dilithium::dilithium3;
use ed25519_dalek as ed25519;

/// Hybrid signature (Ed25519 + Dilithium3)
///
/// Both signatures must be valid for verification to pass.
/// Provides defense-in-depth:
/// - Ed25519: Fast, proven, classical security
/// - Dilithium3: Quantum-safe, NIST standardized
pub struct HybridSignature;

impl Signature for HybridSignature {
    type Category = key_category::Hybrid;
    type Error = CryptoError;
    type PublicKey = HybridPublicKey;
    type SecretKey = HybridSecretKey;
    type Bytes = HybridSignatureBytes;

    #[instrument(skip_all)]
    fn generate_keypair() -> Result<(Self::PublicKey, Self::SecretKey), Self::Error> {
        // Generate Ed25519 keypair
        let ed25519_secret = ed25519::SigningKey::generate(&mut OsRng);
        let ed25519_public = ed25519_secret.verifying_key();

        // Generate Dilithium3 keypair
        let (dilithium_public, dilithium_secret) = dilithium3::keypair();

        Ok((
            HybridPublicKey {
                ed25519: ed25519_public,
                dilithium: dilithium_public,
            },
            HybridSecretKey {
                ed25519: ed25519_secret,
                dilithium: dilithium_secret,
            },
        ))
    }

    #[instrument(skip_all, fields(msg_len = message.len()))]
    fn sign(secret_key: &Self::SecretKey, message: &[u8])
        -> Result<Self::Bytes, Self::Error>
    {
        // Sign with Ed25519 (constant-time)
        let ed25519_sig = secret_key.ed25519.sign(message);

        // Sign with Dilithium3 (constant-time)
        let dilithium_sig = dilithium3::sign(message, &secret_key.dilithium);

        Ok(HybridSignatureBytes {
            ed25519: ed25519_sig,
            dilithium: dilithium_sig,
        })
    }

    #[instrument(skip_all, fields(msg_len = message.len()))]
    fn verify(
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Bytes,
    ) -> bool {
        // Verify Ed25519 signature (constant-time)
        let ed25519_valid = public_key.ed25519
            .verify_strict(message, &signature.ed25519)
            .is_ok();

        // Verify Dilithium3 signature (constant-time)
        let dilithium_valid = dilithium3::verify(
            &signature.dilithium,
            message,
            &public_key.dilithium,
        ).is_ok();

        // Both must pass (constant-time AND)
        // CRITICAL: Use constant-time boolean AND to prevent timing attacks
        constant_time_and(ed25519_valid, dilithium_valid)
    }
}

/// Constant-time boolean AND
///
/// Prevents timing attacks by ensuring execution time
/// doesn't depend on input values.
#[inline(always)]
fn constant_time_and(a: bool, b: bool) -> bool {
    // Convert to u8, AND bitwise, convert back
    // Compiler cannot optimize this to short-circuit
    let a_byte = a as u8;
    let b_byte = b as u8;
    (a_byte & b_byte) == 1
}
```

### 4. Dilithium-Only Implementation

```rust
/// Dilithium3 signature (quantum-safe only)
///
/// NIST Level 3 security (~192 bits classical, quantum-safe)
/// Larger signatures but future-proof.
pub struct DilithiumSignature;

impl Signature for DilithiumSignature {
    type Category = key_category::Quantum;
    type Error = CryptoError;
    type PublicKey = dilithium3::PublicKey;
    type SecretKey = dilithium3::SecretKey;
    type Bytes = dilithium3::SignedMessage;

    #[instrument(skip_all)]
    fn generate_keypair() -> Result<(Self::PublicKey, Self::SecretKey), Self::Error> {
        Ok(dilithium3::keypair())
    }

    #[instrument(skip_all)]
    fn sign(secret_key: &Self::SecretKey, message: &[u8])
        -> Result<Self::Bytes, Self::Error>
    {
        Ok(dilithium3::sign(message, secret_key))
    }

    #[instrument(skip_all)]
    fn verify(
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &Self::Bytes,
    ) -> bool {
        dilithium3::verify(signature, message, public_key).is_ok()
    }
}
```

---

## NIST PQC Algorithm Selection

| Algorithm | Type | Use Case | Key Size | Sig Size | Latency | Security |
|-----------|------|----------|----------|----------|---------|----------|
| **Kyber-1024** | KEM | Key exchange | 1568 B | N/A | ~200 μs | Level 5 |
| **Dilithium3** | Signature | Workflows | 1952 B | 3293 B | ~500 μs | Level 3 |
| **Falcon-1024** | Signature | Compact | 1793 B | ~1330 B | ~100 μs | Level 5 |
| **SLH-DSA** | Signature | Conservative | 64 B | ~8KB | ~50 ms | Hash-based |

**Choice for KNHK**: **Dilithium3** (default) + **Falcon-1024** (optional compact mode)

**Rationale**:
- Dilithium3: Best balance of security, performance, NIST standardization
- Falcon-1024: Smaller signatures for bandwidth-constrained deployments
- SLH-DSA: Ultra-conservative backup (hash-based, slower)
- Kyber-1024: For TLS/network encryption (future Phase 11)

---

## Integration with Workflow Descriptors

### Signed Workflow Descriptors

```rust
/// Workflow descriptor with hybrid signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedWorkflowDescriptor<K: KeyCategory> {
    /// Workflow definition (RDF Turtle)
    pub workflow: String,

    /// Signature over workflow
    pub signature: SignatureBytes<K>,

    /// Public key (for verification)
    pub public_key: PublicKeyBytes<K>,

    /// Signature timestamp (Unix epoch)
    pub timestamp: u64,

    /// Phantom data for key category
    _phantom: PhantomData<K>,
}

impl<K: KeyCategory> SignedWorkflowDescriptor<K> {
    /// Create signed descriptor
    #[instrument(skip(secret_key))]
    pub fn sign<S: Signature<Category = K>>(
        workflow: String,
        secret_key: &S::SecretKey,
    ) -> Result<Self, CryptoError> {
        // Create canonical message (workflow || timestamp)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let message = format!("{}{}", workflow, timestamp);

        // Sign message
        let signature = S::sign(secret_key, message.as_bytes())?;

        Ok(Self {
            workflow,
            signature: signature.as_ref().to_vec().into(),
            public_key: vec![].into(), // TODO: Extract from secret_key
            timestamp,
            _phantom: PhantomData,
        })
    }

    /// Verify signature
    #[instrument(skip(public_key))]
    pub fn verify<S: Signature<Category = K>>(
        &self,
        public_key: &S::PublicKey,
    ) -> bool {
        // Reconstruct message
        let message = format!("{}{}", self.workflow, self.timestamp);

        // Verify signature
        S::verify(public_key, message.as_bytes(), &self.signature.0)
    }
}
```

### Integration with MAPE-K Knowledge Store

```rust
/// Cryptographic receipt for MAPE-K decisions
///
/// Every autonomic decision is signed and stored immutably.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDecisionReceipt {
    /// Decision data
    pub decision: Decision,

    /// Hybrid signature (Ed25519 + Dilithium3)
    pub signature: HybridSignatureBytes,

    /// Signer public key
    pub signer: HybridPublicKey,

    /// Timestamp (Unix epoch)
    pub timestamp: u64,
}

impl SignedDecisionReceipt {
    /// Create signed receipt
    pub fn sign(decision: Decision, secret_key: &HybridSecretKey)
        -> Result<Self, CryptoError>
    {
        let timestamp = current_timestamp();

        // Canonical encoding (deterministic)
        let message = bincode::serialize(&(&decision, timestamp))?;

        // Sign
        let signature = HybridSignature::sign(secret_key, &message)?;

        Ok(Self {
            decision,
            signature,
            signer: extract_public_key(secret_key),
            timestamp,
        })
    }

    /// Verify receipt
    pub fn verify(&self) -> bool {
        let message = bincode::serialize(&(&self.decision, self.timestamp))
            .expect("Serialization cannot fail");

        HybridSignature::verify(&self.signer, &message, &self.signature)
    }
}
```

---

## Migration Strategy

### Three-Phase Migration

```rust
/// Migration phase (type-level state machine)
pub mod migration {
    /// Phase 1: Classical only
    pub type Phase1 = key_category::Classical;

    /// Phase 2: Hybrid (Classical + Quantum)
    pub type Phase2 = key_category::Hybrid;

    /// Phase 3: Quantum only
    pub type Phase3 = key_category::Quantum;

    /// Migration controller
    pub struct MigrationController<Phase: KeyCategory> {
        current_phase: PhantomData<Phase>,
    }

    impl MigrationController<Phase1> {
        /// Upgrade to hybrid mode
        pub fn upgrade_to_hybrid(self) -> MigrationController<Phase2> {
            MigrationController {
                current_phase: PhantomData,
            }
        }
    }

    impl MigrationController<Phase2> {
        /// Upgrade to quantum-only
        pub fn upgrade_to_quantum(self) -> MigrationController<Phase3> {
            MigrationController {
                current_phase: PhantomData,
            }
        }

        /// Rollback to classical (emergency only)
        pub fn rollback_to_classical(self) -> MigrationController<Phase1> {
            MigrationController {
                current_phase: PhantomData,
            }
        }
    }
}
```

### Gradual Rollout Timeline

| Week | Phase | Action | Compatibility |
|------|-------|--------|---------------|
| 1 | Classical | No change | 100% existing systems |
| 2-3 | Hybrid (opt-in) | New workflows use hybrid | Backwards compatible |
| 4-8 | Hybrid (default) | All new workflows hybrid | Can verify classical |
| 9-12 | Quantum (opt-in) | Some workflows quantum-only | Cannot verify classical |
| 13+ | Quantum (default) | New workflows quantum-only | Classical deprecated |

---

## Performance Constraints

### Latency Budgets

| Operation | Latency | Path | Validation |
|-----------|---------|------|------------|
| Keygen | <10 ms | Cold | keygen-bench |
| Sign (Classical) | <100 μs | Warm | sign-bench |
| Sign (Hybrid) | <1 ms | Warm | sign-bench |
| Sign (Quantum) | <500 μs | Warm | sign-bench |
| Verify (Hybrid) | <1 ms | Warm | verify-bench |

### Memory Safety

All secret keys automatically zeroized on drop:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HybridSecretKey {
    ed25519: ed25519::SigningKey,
    dilithium: dilithium3::SecretKey,
}
// Automatically zeroized when dropped (prevents memory dumps)
```

---

## OpenTelemetry Schema

```yaml
# registry/phases_6_10/crypto.yaml
spans:
  - span_name: crypto.sign
    attributes:
      - name: algorithm
        type: string
        requirement_level: required
        brief: "Signature algorithm (Ed25519, Hybrid, Dilithium3)"
      - name: message.size
        type: int
        requirement_level: required
      - name: signature.size
        type: int
        requirement_level: required
      - name: latency_us
        type: int
        requirement_level: required

  - span_name: crypto.verify
    attributes:
      - name: algorithm
        type: string
        requirement_level: required
      - name: result
        type: boolean
        requirement_level: required
        brief: "Verification result (true = valid)"

metrics:
  - metric_name: crypto.sign.latency
    instrument: histogram
    unit: us
    description: "Signature generation latency"

  - metric_name: crypto.verify.latency
    instrument: histogram
    unit: us
    description: "Signature verification latency"
```

---

## Testing Strategy

### Constant-Time Validation

```rust
#[test]
fn test_constant_time_verification() {
    use criterion::black_box;

    let (public_key, secret_key) = HybridSignature::generate_keypair().unwrap();

    let valid_msg = b"valid message";
    let invalid_msg = b"invalid message";

    let signature = HybridSignature::sign(&secret_key, valid_msg).unwrap();

    // Time valid verification
    let start_valid = Instant::now();
    let result_valid = black_box(
        HybridSignature::verify(&public_key, valid_msg, &signature)
    );
    let time_valid = start_valid.elapsed();

    // Time invalid verification
    let start_invalid = Instant::now();
    let result_invalid = black_box(
        HybridSignature::verify(&public_key, invalid_msg, &signature)
    );
    let time_invalid = start_invalid.elapsed();

    // Verify results
    assert!(result_valid);
    assert!(!result_invalid);

    // Constant-time check: times should be within 5% of each other
    let ratio = time_valid.as_nanos() as f64 / time_invalid.as_nanos() as f64;
    assert!(
        0.95 <= ratio && ratio <= 1.05,
        "Timing attack detected: ratio = {}",
        ratio
    );
}
```

---

## Security Audit Checklist

- [ ] All signature operations constant-time
- [ ] Secret keys zeroized on drop
- [ ] No key material in logs/telemetry
- [ ] Hybrid mode requires both signatures to pass
- [ ] DER encoding/decoding validated
- [ ] RNG uses OS entropy (OsRng)
- [ ] No panics in crypto code
- [ ] Weaver validation for all crypto operations

---

## Related Documents

- `PHASES_6-10_ARCHITECTURE_OVERVIEW.md`
- `PHASE_8_BYZANTINE_CONSENSUS_SPECIFICATION.md` (uses hybrid signatures)
- `ADR/ADR-004-quantum-safe-cryptography.md`
- `DOCTRINE_COVENANT.md` - Covenant 2 (Invariants Are Law)

**Next**: See `PHASE_8_BYZANTINE_CONSENSUS_SPECIFICATION.md`
