# KNHK Sidecar Fortune 5 Readiness Plan

**Version**: 1.0.0  
**Status**: Implementation Plan  
**Target**: Fortune 5 Enterprise Deployment

---

## Executive Summary

This document outlines the implementation plan to make the KNHK sidecar Fortune 5 ready, meeting all requirements from the Reflex Enterprise Blueprint including SPIFFE/SPIRE integration, HSM/KMS support, multi-region capabilities, and enterprise-grade security.

---

## Fortune 5 Requirements Gap Analysis

### Current State ✅

- ✅ Basic mTLS support (manual certificate configuration)
- ✅ TLS certificate loading and validation
- ✅ Health checks and metrics
- ✅ Circuit breaker and retry logic
- ✅ Weaver live-check integration
- ✅ Beat-driven admission control
- ✅ Request batching

### Missing for Fortune 5 ❌

1. **SPIFFE/SPIRE Integration** (P0 - CRITICAL)
   - No SPIFFE ID support
   - No SPIRE certificate management
   - Manual certificate configuration required

2. **HSM/KMS Integration** (P0 - CRITICAL)
   - No hardware security module support
   - No key management system integration
   - Keys stored in files (not secure for Fortune 5)

3. **Key Rotation** (P0 - CRITICAL)
   - No automatic key rotation
   - Manual rotation required
   - No ≤24h rotation enforcement

4. **Multi-Region Support** (P0 - CRITICAL)
   - No region configuration
   - No cross-region receipt sync
   - No quorum consensus

5. **Legal Hold** (P1 - HIGH)
   - No legal hold functionality
   - No compliance retention policies

6. **Capacity Planning** (P1 - HIGH)
   - No SLO-based admission control
   - No capacity planning models
   - No cache heat tracking

7. **Formal Promotion Gates** (P1 - HIGH)
   - No canary deployment support
   - No staging/production promotion
   - No feature flag integration

---

## Implementation Plan

### Phase 1: Security Foundation (Week 1-2)

#### 1.1 SPIFFE/SPIRE Integration

**Dependencies**: `spiffe` crate or `spire-rs` integration

**Implementation**:
- Add SPIFFE ID extraction from certificates
- Integrate with SPIRE workload API for automatic certificate management
- Replace manual certificate loading with SPIRE-based certificate rotation
- Add SPIFFE ID validation in mTLS handshake

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/spiffe.rs` - SPIFFE integration module
- `rust/knhk-sidecar/src/config.rs` - Add SPIFFE configuration
- `rust/knhk-sidecar/src/tls.rs` - Integrate SPIFFE certificate loading

**Configuration**:
```rust
pub struct SpiffeConfig {
    pub spiffe_socket_path: String, // Default: /tmp/spire-agent/public/api.sock
    pub spiffe_id: Option<String>,  // Optional: explicit SPIFFE ID
    pub trust_domain: String,       // Required: trust domain
}
```

#### 1.2 HSM/KMS Integration

**Dependencies**: AWS KMS SDK, Azure Key Vault SDK, or generic HSM interface

**Implementation**:
- Add HSM/KMS client abstraction
- Support AWS KMS, Azure Key Vault, HashiCorp Vault
- Use HSM/KMS for key storage and signing operations
- Implement key rotation via KMS APIs

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/kms.rs` - KMS abstraction layer
- `rust/knhk-sidecar/src/kms/aws.rs` - AWS KMS implementation
- `rust/knhk-sidecar/src/kms/azure.rs` - Azure Key Vault implementation
- `rust/knhk-sidecar/src/kms/vault.rs` - HashiCorp Vault implementation

**Configuration**:
```rust
pub enum KmsProvider {
    Aws { region: String, key_id: String },
    Azure { vault_url: String, key_name: String },
    Vault { addr: String, mount_path: String },
    None, // Fallback to file-based (not recommended for Fortune 5)
}
```

#### 1.3 Automatic Key Rotation

**Implementation**:
- Implement key rotation scheduler (≤24h)
- Monitor certificate expiration
- Automatic certificate refresh via SPIRE
- KMS key rotation support

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/key_rotation.rs` - Key rotation manager
- `rust/knhk-sidecar/src/tls.rs` - Add certificate expiration checking

---

### Phase 2: Multi-Region Support (Week 2-3)

#### 2.1 Region Configuration

**Implementation**:
- Add region identifier to sidecar config
- Region-aware service discovery
- Cross-region receipt synchronization

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/config.rs` - Add region configuration
- `rust/knhk-sidecar/src/multi_region.rs` - Multi-region coordination

**Configuration**:
```rust
pub struct MultiRegionConfig {
    pub region: String,                    // e.g., "us-east-1", "eu-west-1"
    pub primary_region: Option<String>,    // For quorum consensus
    pub cross_region_sync_enabled: bool,
    pub receipt_sync_endpoints: Vec<String>,
}
```

#### 2.2 Cross-Region Receipt Sync

**Implementation**:
- Receipt synchronization service
- Quorum consensus for cross-region receipts
- Legal hold support for compliance

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/receipt_sync.rs` - Receipt synchronization
- `rust/knhk-sidecar/src/quorum.rs` - Quorum consensus logic

---

### Phase 3: Capacity Planning & SLOs (Week 3-4)

#### 3.1 SLO-Based Admission Control

**Implementation**:
- R1/W1/C1 runtime class enforcement
- SLO-based admission decisions
- Capacity planning models

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/slo_admission.rs` - SLO-based admission
- `rust/knhk-sidecar/src/capacity.rs` - Capacity planning

**Configuration**:
```rust
pub struct SloConfig {
    pub r1_p99_max_ns: u64,      // 2ns for R1
    pub w1_p99_max_ms: u64,      // 1ms for W1
    pub c1_p99_max_ms: u64,      // 500ms for C1
    pub admission_strategy: AdmissionStrategy,
}

pub enum AdmissionStrategy {
    Strict,    // Reject if SLO cannot be met
    Degrade,   // Park to lower tier if SLO cannot be met
}
```

#### 3.2 Cache Heat Tracking

**Implementation**:
- Track cache hit/miss rates
- Predictive preloading hints
- MPHF cache monitoring

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/cache_heat.rs` - Cache heat tracking

---

### Phase 4: Promotion Gates (Week 4)

#### 4.1 Canary Deployment Support

**Implementation**:
- Feature flag integration
- Canary/staging/production promotion gates
- Automatic rollback on SLO violations

**Files to Create/Modify**:
- `rust/knhk-sidecar/src/promotion.rs` - Promotion gate logic
- `rust/knhk-sidecar/src/feature_flags.rs` - Feature flag integration

**Configuration**:
```rust
pub struct PromotionConfig {
    pub environment: Environment,  // Canary, Staging, Production
    pub feature_flags: Vec<String>,
    pub auto_rollback_enabled: bool,
    pub slo_threshold: f64,       // Rollback if SLO drops below threshold
}

pub enum Environment {
    Canary { traffic_percent: f64 },
    Staging,
    Production,
}
```

---

## Kubernetes Deployment Updates

### Updated DaemonSet Manifest

**File**: `k8s/daemonset-sidecar-fortune5.yaml`

**Key Changes**:
- SPIFFE/SPIRE volume mounts
- KMS credentials (via secrets)
- Region configuration
- Multi-region endpoints
- SLO configuration

---

## Configuration Examples

### Fortune 5 Configuration

```toml
# Sidecar Fortune 5 Configuration
[sidecar]
listen_address = "0.0.0.0:50051"

# SPIFFE/SPIRE Configuration
[sidecar.spiffe]
enabled = true
spiffe_socket_path = "/tmp/spire-agent/public/api.sock"
trust_domain = "fortune5.example.com"
spiffe_id = "spiffe://fortune5.example.com/sidecar/prod"

# KMS Configuration
[sidecar.kms]
provider = "aws"
region = "us-east-1"
key_id = "arn:aws:kms:us-east-1:123456789012:key/abc123"
rotation_interval_hours = 24

# Multi-Region Configuration
[sidecar.multi_region]
region = "us-east-1"
primary_region = "us-east-1"
cross_region_sync_enabled = true
receipt_sync_endpoints = [
    "https://sidecar.us-west-2.fortune5.example.com:50051",
    "https://sidecar.eu-west-1.fortune5.example.com:50051",
]

# SLO Configuration
[sidecar.slo]
r1_p99_max_ns = 2
w1_p99_max_ms = 1
c1_p99_max_ms = 500
admission_strategy = "strict"

# Promotion Gates
[sidecar.promotion]
environment = "production"
auto_rollback_enabled = true
slo_threshold = 0.95
```

---

## Testing Requirements

### Security Testing
- [ ] SPIFFE ID validation tests
- [ ] SPIRE certificate rotation tests
- [ ] KMS key rotation tests
- [ ] mTLS handshake tests

### Multi-Region Testing
- [ ] Cross-region receipt sync tests
- [ ] Quorum consensus tests
- [ ] Region failover tests

### SLO Testing
- [ ] R1/W1/C1 admission tests
- [ ] Capacity planning tests
- [ ] Cache heat tracking tests

### Promotion Testing
- [ ] Canary deployment tests
- [ ] Automatic rollback tests
- [ ] Feature flag tests

---

## Migration Path

### From Current to Fortune 5

1. **Phase 1**: Deploy SPIFFE/SPIRE (non-breaking)
   - Sidecar continues to work with file-based certs
   - Gradually migrate to SPIRE

2. **Phase 2**: Enable KMS (non-breaking)
   - Fallback to file-based keys if KMS unavailable
   - Gradually migrate keys to KMS

3. **Phase 3**: Enable Multi-Region (requires coordination)
   - Deploy sidecars in multiple regions
   - Enable cross-region sync

4. **Phase 4**: Enable SLO Admission (requires monitoring)
   - Monitor SLO compliance
   - Enable strict admission gradually

---

## Success Criteria

### Fortune 5 Readiness Checklist

- [ ] SPIFFE/SPIRE integrated and tested
- [ ] HSM/KMS integrated and tested
- [ ] Key rotation ≤24h implemented
- [ ] Multi-region deployment tested
- [ ] Cross-region receipt sync working
- [ ] SLO-based admission control implemented
- [ ] Capacity planning models validated
- [ ] Canary deployment tested
- [ ] Kubernetes manifests updated
- [ ] Documentation complete

---

## Dependencies

### New Crate Dependencies

```toml
# SPIFFE/SPIRE (if available)
# spiffe = "0.1"  # Or spire-rs integration

# AWS KMS
aws-sdk-kms = "1.0"

# Azure Key Vault
azure_identity = "0.20"
azure_keyvault = "0.20"

# HashiCorp Vault
vaultrs = "0.5"
```

---

## Timeline

- **Week 1-2**: Security Foundation (SPIFFE/SPIRE, KMS, Key Rotation)
- **Week 2-3**: Multi-Region Support
- **Week 3-4**: Capacity Planning & SLOs
- **Week 4**: Promotion Gates
- **Week 5**: Testing & Documentation

**Total**: 5 weeks to Fortune 5 readiness

---

## Related Documentation

- [Reflex Enterprise Blueprint](docs/REFLEX_ENTERPRISE_BLUEPRINT.md)
- [Sidecar README](rust/knhk-sidecar/README.md)
- [Kubernetes Deployment Guide](k8s/README.md)

---

**Status**: Plan Complete - Ready for Implementation

