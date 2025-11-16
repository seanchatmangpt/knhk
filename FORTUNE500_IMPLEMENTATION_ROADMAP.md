# KNHK Fortune 500 Implementation Roadmap

**Status**: Gap Analysis Complete - Implementation in Progress
**Severity**: Critical - Blocks Fortune 500 certification
**Owner**: KNHK Team
**Target**: Full Fortune 500 end-to-end validation

---

## Executive Summary

KNHK has skeleton implementations of Fortune 500 features that return errors instead of working. This document provides a step-by-step roadmap to implement and validate each feature.

**Key Point**: Following CLAUDE.md's definition of done, ONLY **Weaver validation** proves features work. Tests and help text are not sufficient.

---

## Gap Status by Component

### 🔴 P0 - CRITICAL (Blocks Fortune 500)

| Component | Status | Gap | Fix Strategy |
|-----------|--------|-----|------|
| **KMS Integration** | ❌ Returns errors | AWS/Azure/Vault clients stub | Implement with AWS SDK, Azure REST, Vault HTTP |
| **SPIFFE/SPIRE** | ⚠️ File-based fallback | No SPIRE workload API connection | Add Unix socket + X.509-SVID + refresh loop |
| **Key Rotation** | ❌ Blocked by KMS | Depends on broken KMS | Implement after KMS |
| **Multi-Region** | ⚠️ Skeleton | No actual HTTP sync | Implement quorum + cross-region HTTP |

### 🟡 P1 - HIGH (Required for Enterprise)

| Component | Status | Gap | Fix Strategy |
|-----------|--------|-----|------|
| **Promotion Gates** | ⚠️ Skeleton | No feature flag logic | Implement canary + staging + prod gates |
| **Capacity Planning** | ⚠️ Skeleton | Metrics only, no models | Implement SLO-based scaling predictions |
| **Legal Hold** | ❌ Missing | Not implemented | Add compliance retention policies |

---

## Implementation Sequence

### Phase 1: KMS Integration (CRITICAL)

**Files to Update**:
- `rust/knhk-sidecar/src/kms.rs` → Already created: `kms_impl.rs`
- `rust/knhk-sidecar/src/lib.rs` → Add module export

**Implementation**:
```rust
// Status: ✅ Implementation template created in kms_impl.rs

// AWS KMS:
// - Uses aws-sdk-kms v1.0 (already in Cargo.toml)
// - Implements: sign(), rotate_key(), get_public_key(), get_key_metadata()
// - Error handling: Returns SidecarResult with proper error context

// Azure Key Vault:
// - Uses reqwest HTTP client (already in Cargo.toml)
// - Implements REST API calls to Azure Key Vault
// - Requires authentication context (will be set from env)

// HashiCorp Vault:
// - Uses reqwest for Transit API calls
// - Implements: POST /v1/{mount}/sign/{key_name}
// - Requires VAULT_ADDR and VAULT_TOKEN from environment
```

**Testing**:
```bash
# Compile with AWS SDK
cargo build --features fortune5 -p knhk-sidecar

# Run tests (will require actual AWS credentials)
cargo test --features fortune5 -p knhk-sidecar -- test_kms
```

**Weaver Validation**:
```bash
# Will validate telemetry against knhk-sidecar.yaml schema
weaver registry live-check --registry registry/
```

---

### Phase 2: SPIFFE/SPIRE Integration

**Files to Update**:
- `rust/knhk-sidecar/src/spiffe.rs` → Add workload API

**Current State** (file-based fallback):
```rust
// Current: Reads certificates from filesystem
let cert_path = socket_dir.join("svid.pem");
let key_path = socket_dir.join("key.pem");
self.current_cert = Some(std::fs::read(&cert_path)?);
```

**Required Implementation** (SPIRE workload API):
```rust
// TODO: Replace filesystem reads with SPIRE API
// 1. Connect to Unix socket at config.socket_path
// 2. Send X-SPIRE-WorkloadAPI request
// 3. Receive X.509-SVID bundle
// 4. Parse certificate chain and private key
// 5. Implement refresh on timer (config.refresh_interval)

// Pseudocode:
async fn load_certificate_from_spire(&mut self) -> SidecarResult<()> {
    let socket = UnixStream::connect(&self.config.socket_path).await?;
    let svid_bundle = request_x509_svid_bundle(socket).await?;
    self.current_cert = Some(svid_bundle.cert);
    self.current_key = Some(svid_bundle.key);
    Ok(())
}
```

**Dependencies to Add**:
```toml
tokio = { version = "1", features = ["net"] }  # Already present
prost = "0.12"  # For SPIRE API protobuf
```

---

### Phase 3: Multi-Region Support

**Files to Update**:
- `rust/knhk-sidecar/src/multi_region.rs` → Add actual sync

**Current State** (skeleton):
```rust
pub struct ReceiptSyncManager {
    config: RegionConfig,
    sync_clients: HashMap<String, ReceiptSyncClient>,  // Never used!
}
```

**Required Implementation**:
```rust
pub async fn sync_receipt(&mut self, receipt: &Receipt) -> SidecarResult<()> {
    // 1. For each configured region:
    //    - Make HTTP POST to receipt_sync_endpoint
    //    - Include receipt data
    //    - Collect responses

    // 2. Implement quorum consensus:
    //    - Count successful syncs
    //    - Verify >= quorum_threshold regions acknowledged
    //    - Return error if quorum not reached

    // 3. Track cross-region state:
    //    - Update last_sync_timestamp
    //    - Log sync events
    //    - Report failures to metrics
}
```

---

### Phase 4: Promotion Gates

**Files to Update**:
- `rust/knhk-sidecar/src/promotion.rs` → Add routing logic

**Current State** (skeleton):
```rust
pub struct PromotionGateManager {
    // No actual routing implementation!
}
```

**Required Implementation**:
```rust
pub fn should_route_to_new_version(&self, request_id: &str) -> bool {
    match self.config.environment {
        Environment::Canary { traffic_percent } => {
            // Hash request_id deterministically
            // Route to new version if hash % 100 < traffic_percent
            self.route_canary(request_id, traffic_percent)
        }
        Environment::Staging => {
            // Route to staging infrastructure
            true
        }
        Environment::Production => {
            // Monitor SLOs
            // Auto-rollback if violations detected
            self.check_slo_and_route(request_id)
        }
    }
}

pub async fn auto_rollback_if_needed(&mut self) -> SidecarResult<()> {
    if !self.config.auto_rollback_enabled {
        return Ok(());
    }

    let slo_met = self.check_slo_metrics().await?;
    if !slo_met && std::time::Instant::now()
        .duration_since(self.promotion_started)
        > Duration::from_secs(self.config.rollback_window_seconds)
    {
        // Trigger rollback
        self.initiate_rollback().await?;
    }
    Ok(())
}
```

---

### Phase 5: Capacity Planning

**Files to Update**:
- `rust/knhk-sidecar/src/capacity.rs` → Add prediction models

**Current State** (metrics only):
```rust
pub struct CapacityManager {
    heat_metrics: HashMap<String, CacheHeatMetrics>,
    // No prediction models!
}
```

**Required Implementation**:
```rust
pub fn predict_capacity_needed(&self, slo_threshold: f64) -> SidecarResult<CapacityPrediction> {
    // 1. Analyze heat metrics
    //    - Identify hot data (high access frequency)
    //    - Calculate working set size
    //    - Project future growth

    // 2. Model different SLO targets
    //    - R1 (≤8 ticks): Requires hot data in L1
    //    - W1 (≤500ms): Requires warm cache hits
    //    - C1 (≤24h): Allows cold data from persistent

    // 3. Return prediction:
    //    - Required L1 size (bytes)
    //    - Expected hit rate at that size
    //    - Cost estimate
}

pub fn should_admit_request(&self, slo_class: SloClass) -> bool {
    // Check if admitting this request keeps cache hit rate above threshold
    let current_hit_rate = self.calculate_hit_rate();
    match slo_class {
        SloClass::R1 => current_hit_rate > 0.99,  // 99% hit rate for hot path
        SloClass::W1 => current_hit_rate > 0.95,  // 95% for warm path
        SloClass::C1 => true,  // All requests admitted
    }
}
```

---

## Implementation Checklist

### KMS Integration
- [ ] Move `kms_impl.rs` content into `kms.rs`
- [ ] Update `lib.rs` to export KMS modules
- [ ] Add tests for AWS KMS signing (requires AWS credentials)
- [ ] Add tests for Azure Key Vault integration
- [ ] Add tests for HashiCorp Vault integration
- [ ] Verify compilation with `--features fortune5`
- [ ] Test actual KMS operations (not just structure)
- [ ] Update Weaver schema to match telemetry

### SPIFFE/SPIRE
- [ ] Implement Unix socket connection to SPIRE agent
- [ ] Implement X.509-SVID bundle parsing
- [ ] Implement certificate refresh timer
- [ ] Add tests with SPIRE agent running
- [ ] Validate SPIFFE ID extraction
- [ ] Test certificate rotation

### Multi-Region
- [ ] Implement HTTP client for cross-region sync
- [ ] Implement quorum consensus logic
- [ ] Add failure handling and retry logic
- [ ] Test with multiple regions (can use localhost:port)
- [ ] Implement receipt ACK tracking

### Promotion Gates
- [ ] Implement canary routing logic
- [ ] Implement SLO monitoring and auto-rollback
- [ ] Add feature flag evaluation
- [ ] Test with various traffic percentages
- [ ] Implement rollback coordination

### Capacity Planning
- [ ] Implement capacity prediction models
- [ ] Add SLO-based admission control
- [ ] Implement heat map analysis
- [ ] Test with various cache sizes
- [ ] Add metrics reporting

---

## Weaver Validation Requirements

### Current Schema Status
```
/home/user/knhk/registry/
├── knhk-sidecar.yaml          ← Defines expected telemetry
├── knhk-operation.yaml        ← R1 requirements (≤8 ticks)
├── knhk-warm.yaml            ← W1 requirements (≤500ms)
└── ...
```

### What Must Match
```
Schema declares: "kms.sign() emits span with attributes X, Y, Z"
Runtime telemetry must: Actually emit those spans with exact attributes
Validation: weaver registry live-check --registry registry/
```

### Example: KMS Signing Telemetry

**Schema (knhk-sidecar.yaml)**:
```yaml
- name: kms.sign
  description: "KMS signing operation"
  span_kind: INTERNAL
  attributes:
    - name: kms.provider
      description: "AWS|Azure|Vault"
    - name: kms.key_id
      description: "Key identifier"
    - name: kms.region
      description: "Region for AWS KMS"
```

**Code Implementation**:
```rust
async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
    let span = tracer.start("kms.sign");
    span.set_attribute("kms.provider", "aws");
    span.set_attribute("kms.key_id", &self.key_id);
    span.set_attribute("kms.region", &self.region);

    // Actual signing...
    self.client.sign(...).await
}
```

---

## Testing Strategy

### Unit Tests (Compilable)
```bash
cargo test --features fortune5 -p knhk-sidecar
```

### Integration Tests (Requires Infrastructure)
```bash
# Start AWS KMS (local or real)
# Start SPIRE agent
# Start Vault
# Start second region (localhost:8081)

# Then run end-to-end tests
cargo test --features fortune5 -p knhk-sidecar -- --ignored
```

### Weaver Validation (Schema Compliance)
```bash
# Validate schema definition
weaver registry check -r registry/

# Validate runtime telemetry matches schema
weaver registry live-check --registry registry/
```

### End-to-End Testing
```bash
# Execute actual commands (not just --help)
knhk fortune5 test \
  --spiffe-domain example.com \
  --kms-provider aws \
  --aws-region us-east-1 \
  --aws-key-id arn:aws:kms:...

# Must not return errors, must emit proper telemetry
```

---

## Success Criteria

**KMS Integration** ✅ Complete when:
- [ ] AWS SDK calls succeed (sign, rotate, get_public_key)
- [ ] Azure Key Vault REST calls succeed (when authenticated)
- [ ] HashiCorp Vault Transit API calls succeed
- [ ] Weaver validates telemetry schema
- [ ] Tests pass with real credentials
- [ ] Error cases handled properly

**SPIFFE/SPIRE** ✅ Complete when:
- [ ] Connects to SPIRE agent via Unix socket
- [ ] Parses X.509-SVID bundle correctly
- [ ] Extracts certificate and key successfully
- [ ] Refresh timer works (can be tested with mock timer)
- [ ] Weaver validates telemetry

**Multi-Region** ✅ Complete when:
- [ ] Cross-region HTTP sync works
- [ ] Quorum consensus implemented
- [ ] Handles region failures gracefully
- [ ] Weaver validates telemetry

**Promotion Gates** ✅ Complete when:
- [ ] Canary routing works (deterministic, per-request)
- [ ] Auto-rollback triggers on SLO violations
- [ ] Feature flags evaluated correctly
- [ ] Weaver validates telemetry

**Capacity Planning** ✅ Complete when:
- [ ] Prediction models implemented
- [ ] SLO-based admission works
- [ ] Heat map analysis accurate
- [ ] Weaver validates telemetry

---

## Critical Notes

### No False Positives Allowed
Per CLAUDE.md's definition of done:

```
❌ WRONG - These don't prove Fortune 500 readiness:
- knhk fortune5 test --help      (help text exists)
- cargo test --workspace          (tests pass)
- cargo build                      (compiles)

✅ CORRECT - These prove it works:
- knhk fortune5 test <with real args>  (actual execution succeeds)
- weaver registry live-check           (telemetry matches schema)
- End-to-end integration test          (whole workflow succeeds)
```

### Async/Await Design
All KMS operations are async due to AWS SDK requirements. Update call sites:

```rust
// Old (sync):
let signature = manager.sign(data)?;

// New (async):
let signature = manager.sign(data).await?;
```

### Testing Without Cloud Resources
Use local mocks for testing:
- AWS KMS: Can use LocalStack (Docker)
- Azure Key Vault: Can mock REST endpoints
- HashiCorp Vault: Can run local dev server
- SPIRE: Can run SPIRE agent locally
- Multi-region: Can use localhost:8081, 8082, 8083

---

## Next Steps

1. **Review this roadmap** - Ensure alignment with requirements
2. **Implement Phase 1 (KMS)** - Foundation for other features
3. **Add tests** - Each phase must have working tests
4. **Run Weaver validation** - Prove telemetry matches schema
5. **Document results** - Update status as features complete
6. **Commit to branch** - `claude/verify-knhk-fortune500-*`

---

## References

- [Root Cause Analysis](./FORTUNE500_CONFIDENCE_GAP_ROOT_CAUSE.md) - Why confidence was low
- [CLAUDE.md](./CLAUDE.md) - Definition of done, Weaver validation
- [Weaver Docs](https://opentelemetry.io/docs/instrumentation/) - Schema validation
- [AWS KMS SDK](https://github.com/awslabs/aws-sdk-rust) - AWS documentation
- [Azure Key Vault REST](https://learn.microsoft.com/en-us/rest/api/keyvault/) - Azure docs
- [HashiCorp Vault](https://www.vaultproject.io/docs/secrets/transit) - Vault Transit API

---

**Owner**: KNHK Team
**Status**: Gap Analysis Complete - Awaiting Implementation
**Last Updated**: 2025-11-16
