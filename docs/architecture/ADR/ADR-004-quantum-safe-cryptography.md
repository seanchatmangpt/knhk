# ADR-004: Quantum-Safe Cryptography (Phase 7)

**Status**: Proposed
**Date**: 2025-11-18
**Deciders**: System Architecture Team, Security Team, DOCTRINE Compliance Board
**Technical Story**: Future-proof KNHK against quantum computing threats

---

## Context and Problem Statement

Current cryptographic primitives (Ed25519, ECDSA, RSA) will be vulnerable to quantum computers (Shor's algorithm). NIST has standardized post-quantum cryptography (PQC) algorithms. How do we migrate to quantum-safe signatures while maintaining:
- Backward compatibility (existing systems use classical crypto)
- Latency bounds (<1ms for signing/verification)
- Security guarantees (constant-time, no side channels)
- Gradual migration (no big-bang deployment)

---

## Decision Drivers

- **DOCTRINE Alignment**: Q (Invariants) - Cryptographic guarantees are Q constraints
- **Performance**: Signing/verification <1ms (warm path, not hot path)
- **Security**: NIST PQC certified, constant-time implementations
- **Migration**: Gradual rollout (Classical → Hybrid → Quantum)
- **Compliance**: FIPS 140-3, FedRAMP, NIST SP 800-208

---

## Considered Options

### Option 1: Quantum-Only (Dilithium3)
**Description**: Immediately switch to Dilithium3 signatures.

**Pros**:
- ✅ Fully quantum-safe (NIST Level 3)
- ✅ Standardized (NIST FIPS 204)
- ✅ Fast signing (~500 μs)

**Cons**:
- ❌ Large signatures (3293 bytes vs 64 bytes Ed25519)
- ❌ Breaks compatibility with existing systems
- ❌ No classical security guarantees
- ❌ Rollback impossible if quantum not viable

**Verdict**: REJECTED (too risky for immediate deployment)

### Option 2: Classical-Only (Ed25519)
**Description**: Continue using Ed25519, wait for quantum threat.

**Pros**:
- ✅ Proven secure (classical)
- ✅ Small signatures (64 bytes)
- ✅ Fast (< 100 μs)
- ✅ No migration needed

**Cons**:
- ❌ Vulnerable to quantum computers (Shor's algorithm)
- ❌ No future-proofing
- ❌ Late migration will be costly

**Verdict**: REJECTED (ignores known future threat)

### Option 3: Hybrid (Ed25519 + Dilithium3)
**Description**: Sign with both algorithms; verify both signatures.

**Pros**:
- ✅ Secure against classical AND quantum attacks
- ✅ Gradual migration path (add quantum, then drop classical)
- ✅ Defense-in-depth (if one algorithm breaks, other protects)
- ✅ Rollback possible (can drop quantum if issues)

**Cons**:
- ❌ Larger signatures (64 + 3293 = 3357 bytes)
- ❌ 2x signing/verification time (~600 μs total)
- ❌ More complex implementation

**Verdict**: ACCEPTED (best balance of security and migration)

---

## Decision Outcome

**Chosen option**: **Hybrid Signatures (Ed25519 + Dilithium3)**

### Three-Phase Migration

#### Phase 1: Classical Only (Current)
- All signatures use Ed25519
- No changes to existing systems
- Duration: Until quantum computers become threat (5-10 years?)

#### Phase 2: Hybrid (Default for New Workflows)
- New workflows signed with Ed25519 + Dilithium3
- Old workflows can still use Ed25519 (backward compatible)
- Both signatures must verify for hybrid mode
- Duration: 1-2 years (gradual rollout)

#### Phase 3: Quantum-Only (Future)
- New workflows signed with Dilithium3 only
- Ed25519 deprecated (optional verification for legacy)
- Duration: After quantum computers are viable threat

### Type-Level Enforcement

```rust
/// Key categories (phantom types)
pub struct Classical;   // Ed25519
pub struct Hybrid;      // Ed25519 + Dilithium3
pub struct Quantum;     // Dilithium3 only

/// Signature trait parameterized by category
trait Signature<K: KeyCategory> {
    fn sign(key: &SecretKey<K>, msg: &[u8]) -> Sig<K>;
    fn verify(key: &PublicKey<K>, msg: &[u8], sig: &Sig<K>) -> bool;
}

// Type system prevents:
// - Verifying Classical signature with Quantum key
// - Signing Hybrid message with Classical-only key
```

---

## NIST PQC Algorithm Selection

| Algorithm | Type | Security Level | Key Size | Sig Size | Speed | Selected |
|-----------|------|----------------|----------|----------|-------|----------|
| Dilithium2 | Signature | Level 2 (~128 bits) | 1312 B | 2420 B | ~300 μs | ❌ |
| **Dilithium3** | Signature | Level 3 (~192 bits) | 1952 B | 3293 B | ~500 μs | ✅ |
| Dilithium5 | Signature | Level 5 (~256 bits) | 2592 B | 4595 B | ~700 μs | ❌ |
| Falcon-512 | Signature | Level 1 | 897 B | ~666 B | ~100 μs | ⚠️ |
| Falcon-1024 | Signature | Level 5 | 1793 B | ~1330 B | ~150 μs | ⚠️ |
| SLH-DSA | Signature | Configurable | 32-64 B | 8-49 KB | 10-200 ms | ❌ |

**Rationale**:
- **Dilithium3**: Best balance (Level 3 security, reasonable size, fast)
- **Falcon**: Optional for bandwidth-constrained scenarios (smaller sigs)
- **SLH-DSA**: Too slow for our <1ms requirement

---

## Performance Impact

| Operation | Classical | Hybrid | Quantum | Limit |
|-----------|-----------|--------|---------|-------|
| Keygen | ~50 μs | ~10 ms | ~10 ms | Cold path |
| Sign | ~50 μs | ~550 μs | ~500 μs | <1 ms ✅ |
| Verify | ~100 μs | ~600 μs | ~500 μs | <1 ms ✅ |
| Signature size | 64 B | 3357 B | 3293 B | Network |
| Public key size | 32 B | 1984 B | 1952 B | Storage |

**Impact**: Acceptable (warm path, not hot path)

---

## Security Guarantees

### Constant-Time Implementation

All signature operations must be constant-time to prevent timing attacks:

```rust
fn constant_time_verify(a: bool, b: bool) -> bool {
    let a_byte = a as u8;
    let b_byte = b as u8;
    (a_byte & b_byte) == 1  // Constant-time AND
}

// Hybrid verification (both must pass)
let valid = constant_time_verify(
    verify_ed25519(key, msg, sig),
    verify_dilithium3(key, msg, sig),
);
```

### Key Zeroization

Secret keys automatically zeroized on drop:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct HybridSecretKey {
    ed25519: Ed25519SecretKey,
    dilithium: Dilithium3SecretKey,
}
// Keys zeroized when dropped (prevents memory dumps)
```

---

## Compliance

- **NIST FIPS 204**: Dilithium3 standardized
- **FIPS 140-3**: Constant-time implementations required
- **FedRAMP**: Quantum-safe crypto recommended
- **DOCTRINE Covenant 2**: Cryptographic invariants are Q constraints

---

## Validation

### Security Audit Checklist
- [ ] All crypto operations constant-time
- [ ] Secret keys zeroized on drop
- [ ] No key material in logs/telemetry
- [ ] Hybrid mode requires BOTH signatures valid
- [ ] RNG uses OS entropy (OsRng)
- [ ] DER encoding/decoding validated

### Weaver Schema
```yaml
spans:
  - span_name: crypto.sign
    attributes:
      - name: algorithm
        type: string
        values: [ed25519, hybrid, dilithium3]
      - name: latency_us
        type: int
        brief: "Must be <1000 μs"
```

---

## Risks and Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Dilithium broken | High | Low | Hybrid mode (Ed25519 still protects) |
| Signature size overhead | Medium | High | Optional Falcon for bandwidth-constrained |
| Performance regression | Medium | Low | Async signing (off hot path) |
| Migration complexity | Medium | Medium | Type-level enforcement (compile-time safety) |

---

## Positive Consequences

- ✅ Future-proof against quantum computers
- ✅ Gradual migration (no big-bang)
- ✅ Defense-in-depth (dual algorithms)
- ✅ Type-safe migration (invalid states unrepresentable)
- ✅ Rollback possible (can drop quantum if needed)

---

## Negative Consequences

- ⚠️ 50x larger signatures (3357 B vs 64 B)
- ⚠️ 5-6x slower signing/verification
- ⚠️ More complex key management
- ⚠️ Network bandwidth increase

---

## References

- [NIST PQC Standardization](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [FIPS 204: Dilithium](https://csrc.nist.gov/pubs/fips/204/final)
- [pqcrypto Rust crate](https://github.com/rustpq/pqcrypto)
- `PHASE_7_QUANTUM_CRYPTO_SPECIFICATION.md`
- `DOCTRINE_COVENANT.md` - Covenant 2

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 2025-11-18 | Initial ADR | System Architecture Team |
