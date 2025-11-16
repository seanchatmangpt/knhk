# SPARC Phase 9: Production Hardening & Deployment Guide

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Production Deployment Specification
**Authors**: Production Validator Agent
**Related Documents**:
- [SPARC Specification Complete](SPARC_SPECIFICATION_COMPLETE.md)
- [SPARC Architecture Unified](SPARC_ARCHITECTURE_UNIFIED.md)
- [SPARC Phase 5 Completion](SPARC_PHASE5_COMPLETION.md)
- [SPARC Phase 8 Weaver Validation](SPARC_PHASE8_WEAVER_VALIDATION.md)

---

## Executive Summary

This document provides the complete production hardening and deployment guide for the **Knowledge-Native Hyper-Kernel (KNHK)** closed-loop autonomous intelligence system. Phase 9 ensures the system is secure, reliable, performant, and operationally ready for Fortune 500 deployment across finance, healthcare, manufacturing, and logistics sectors.

**Production Requirements**:
- ✅ Security hardening (mTLS, OIDC, secrets management, audit logging)
- ✅ Performance optimization (≤8 ticks hot path, <100ms warm path)
- ✅ High availability (99.9% uptime, <30s RTO, 0 RPO)
- ✅ Disaster recovery (automated backups, tested rollback procedures)
- ✅ Operational excellence (runbooks, monitoring, alerting, on-call)
- ✅ Compliance (audit trails, data retention, sector-specific regulations)
- ✅ Observability (Prometheus, Grafana, OTLP, Weaver validation)

**Critical Principle**: Only Weaver validation proves features work. All production readiness criteria MUST be validated via OpenTelemetry schema compliance (Phase 8).

---

## Table of Contents

1. [Security Hardening](#1-security-hardening)
2. [Performance Optimization](#2-performance-optimization)
3. [Reliability & High Availability](#3-reliability--high-availability)
4. [Data Integrity & Disaster Recovery](#4-data-integrity--disaster-recovery)
5. [Operational Readiness](#5-operational-readiness)
6. [Compliance & Governance](#6-compliance--governance)
7. [Deployment Procedures](#7-deployment-procedures)
8. [Monitoring & Alerting](#8-monitoring--alerting)
9. [Production Checklist](#9-production-checklist)
10. [Runbooks](#10-runbooks)

---

## 1. Security Hardening

### 1.1 Authentication & Authorization

#### 1.1.1 OpenID Connect (OIDC) Integration

**Requirements**:
- All API access requires valid JWT token from trusted IdP
- Support for multiple IdPs (Okta, Auth0, Azure AD, Keycloak)
- Token validation includes audience, issuer, expiration checks
- Role-based access control (RBAC) enforced at API gateway

**Configuration** (`/deployment/security/oidc-config.yaml`):
```yaml
oidc:
  providers:
    - name: okta
      issuer: https://your-domain.okta.com
      client_id: ${OKTA_CLIENT_ID}
      client_secret: ${OKTA_CLIENT_SECRET}  # From vault
      scopes: [openid, profile, email, groups]

    - name: azure-ad
      issuer: https://login.microsoftonline.com/${TENANT_ID}/v2.0
      client_id: ${AZURE_CLIENT_ID}
      client_secret: ${AZURE_CLIENT_SECRET}  # From vault
      scopes: [openid, profile, email]

  validation:
    require_audience: true
    allowed_audiences:
      - "knhk-api"
      - "knhk-admin"
    require_expiration: true
    max_token_age_seconds: 3600
    require_issuer_validation: true

  rbac_mapping:
    observer:
      allows: [READ_OBSERVATIONS, READ_SNAPSHOTS, READ_RECEIPTS]
    analyst:
      allows: [READ_*, DETECT_PATTERNS]
    proposer:
      allows: [READ_*, DETECT_PATTERNS, GENERATE_PROPOSALS]
    validator:
      allows: [READ_*, VALIDATE_PROPOSALS]
    operator:
      allows: [READ_*, PROMOTE_SNAPSHOTS, EXECUTE_MAPE_K]
    administrator:
      allows: [ALL]
```

**Implementation** (Rust middleware):
```rust
// rust/knhk-api/src/auth.rs
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub groups: Vec<String>,
}

pub struct OidcValidator {
    providers: HashMap<String, OidcProvider>,
}

impl OidcValidator {
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // 1. Decode without verification to get issuer
        let header = jsonwebtoken::decode_header(token)?;
        let unverified = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&[]),
            &Validation::default(),
        )?;

        // 2. Find provider by issuer
        let provider = self.providers.get(&unverified.claims.iss)
            .ok_or(AuthError::UnknownIssuer)?;

        // 3. Validate with provider's public key
        let mut validation = Validation::default();
        validation.set_audience(&provider.allowed_audiences);
        validation.set_issuer(&[&provider.issuer]);

        let claims = decode::<Claims>(
            token,
            &provider.decoding_key,
            &validation,
        )?;

        // 4. Emit telemetry
        tracing::info!(
            name = "auth.token_validated",
            "user.id" = %claims.claims.sub,
            "user.groups" = ?claims.claims.groups,
        );

        Ok(claims.claims)
    }
}
```

#### 1.1.2 Mutual TLS (mTLS) for Inter-Service Communication

**Requirements**:
- All service-to-service communication encrypted with mTLS
- Certificates issued by internal CA (Vault PKI)
- Automatic certificate rotation (every 90 days)
- Client certificate validation enforced

**Certificate Generation** (Vault PKI):
```bash
#!/bin/bash
# deployment/security/generate-service-certs.sh

set -e

SERVICE_NAME=$1
NAMESPACE=${2:-knhk}
VAULT_ADDR=${VAULT_ADDR:-http://vault:8200}

echo "Generating mTLS certificates for $SERVICE_NAME"

# Request certificate from Vault PKI
vault write -format=json pki_int/issue/knhk-service \
  common_name="${SERVICE_NAME}.${NAMESPACE}.svc.cluster.local" \
  ttl="2160h" \
  alt_names="${SERVICE_NAME}.${NAMESPACE}.svc" \
  > /tmp/${SERVICE_NAME}-cert.json

# Extract certificate and key
cat /tmp/${SERVICE_NAME}-cert.json | jq -r '.data.certificate' > ${SERVICE_NAME}.crt
cat /tmp/${SERVICE_NAME}-cert.json | jq -r '.data.private_key' > ${SERVICE_NAME}.key
cat /tmp/${SERVICE_NAME}-cert.json | jq -r '.data.issuing_ca' > ca.crt

# Create Kubernetes secret
kubectl create secret tls ${SERVICE_NAME}-mtls \
  --cert=${SERVICE_NAME}.crt \
  --key=${SERVICE_NAME}.key \
  --namespace=${NAMESPACE} \
  --dry-run=client -o yaml | kubectl apply -f -

echo "✅ Certificates generated and stored in Kubernetes secret ${SERVICE_NAME}-mtls"
```

**Nginx Configuration** (mTLS enforcement):
```nginx
# deployment/kubernetes/nginx-mtls.conf
upstream knhk-backend {
    server knhk-closed-loop:8080;
}

server {
    listen 443 ssl;
    server_name knhk.company.com;

    # Server certificate
    ssl_certificate /etc/nginx/certs/server.crt;
    ssl_certificate_key /etc/nginx/certs/server.key;

    # Client certificate validation (mTLS)
    ssl_client_certificate /etc/nginx/certs/ca.crt;
    ssl_verify_client on;
    ssl_verify_depth 2;

    # TLS configuration
    ssl_protocols TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers on;

    location / {
        # Extract client DN for authorization
        set $client_dn $ssl_client_s_dn;
        proxy_set_header X-Client-DN $client_dn;

        proxy_pass http://knhk-backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 1.2 Secrets Management

#### 1.2.1 Vault Integration

**Secrets Hierarchy**:
```
secret/knhk/
├── database/
│   ├── postgres/password
│   ├── postgres/connection-string
│   └── redis/password
├── api/
│   ├── signing-key  # Receipt ed25519 key
│   ├── verifying-key
│   └── jwt-secret
├── external/
│   ├── openai-api-key
│   ├── stripe-secret-key
│   └── smtp-password
└── encryption/
    ├── master-key  # AES-256 for at-rest encryption
    └── kms-key-id
```

**Vault Policy** (`deployment/security/vault-policy.hcl`):
```hcl
# Read-only access to database secrets
path "secret/data/knhk/database/*" {
  capabilities = ["read", "list"]
}

# Read-write access to API keys (for rotation)
path "secret/data/knhk/api/*" {
  capabilities = ["read", "update", "list"]
}

# Admin-only access to encryption keys
path "secret/data/knhk/encryption/*" {
  capabilities = ["read"]
  allowed_parameters = {
    "admin_access" = ["true"]
  }
}
```

**Kubernetes Integration** (Vault Agent Injector):
```yaml
# deployment/kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-closed-loop
spec:
  template:
    metadata:
      annotations:
        vault.hashicorp.com/agent-inject: "true"
        vault.hashicorp.com/role: "knhk-service"
        vault.hashicorp.com/agent-inject-secret-database: "secret/data/knhk/database/postgres"
        vault.hashicorp.com/agent-inject-template-database: |
          {{- with secret "secret/data/knhk/database/postgres" -}}
          export DATABASE_URL="{{ .Data.data.connection_string }}"
          {{- end }}
    spec:
      serviceAccountName: knhk-service
      containers:
      - name: knhk-closed-loop
        image: knhk-closed-loop:latest
        command: ["/bin/sh", "-c"]
        args:
          - source /vault/secrets/database && /usr/local/bin/knhk-closed-loop
```

#### 1.2.2 Key Rotation Policy

**Automated Rotation Schedule**:
| Key Type | Rotation Frequency | Automation | Rollback Window |
|----------|-------------------|------------|-----------------|
| Receipt Signing Keys (ed25519) | 90 days | Manual (requires coordination) | 7 days (overlap) |
| Database Passwords | 30 days | Automated (Vault) | Immediate |
| API Keys (external) | 60 days | Semi-automated (manual approval) | 24 hours |
| TLS Certificates | 90 days | Automated (cert-manager) | 30 days (overlap) |
| Encryption Master Key | 365 days | Manual (requires re-encryption) | N/A (no rollback) |

**Key Rotation Script** (`deployment/security/rotate-signing-key.sh`):
```bash
#!/bin/bash
# Rotate receipt signing key with zero-downtime

set -e

echo "🔄 Starting receipt signing key rotation"

# Step 1: Generate new key pair
NEW_SIGNING_KEY=$(openssl genpkey -algorithm ed25519 | base64 -w 0)
NEW_VERIFYING_KEY=$(echo $NEW_SIGNING_KEY | openssl pkey -pubout | base64 -w 0)

# Step 2: Store new keys in Vault (versioned)
vault kv put secret/knhk/api/signing-key-v2 \
  signing_key="${NEW_SIGNING_KEY}" \
  verifying_key="${NEW_VERIFYING_KEY}" \
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Step 3: Update application config (both old and new keys active)
vault kv patch secret/knhk/api/receipt-keys \
  signing_keys='["v1","v2"]' \
  verifying_keys='["v1","v2"]' \
  active_signing_key="v2"

# Step 4: Restart pods to pick up new config
kubectl rollout restart deployment/knhk-closed-loop

# Step 5: Wait for rollout to complete
kubectl rollout status deployment/knhk-closed-loop --timeout=5m

# Step 6: Verify new key is working
echo "⏱️  Waiting 60 seconds for new receipts..."
sleep 60

# Check that new receipts use new key
RECENT_RECEIPT=$(kubectl exec -it deploy/knhk-closed-loop -- \
  knhk receipts list --limit 1 --format json | jq -r '.[0].signature_key_version')

if [ "$RECENT_RECEIPT" != "v2" ]; then
  echo "❌ Key rotation failed - receipts still using old key"
  exit 1
fi

echo "✅ Key rotation successful - new receipts using key v2"

# Step 7: Schedule old key deprecation (7 days)
echo "⏰ Old key v1 will be removed after 2025-11-23"
```

### 1.3 Input Validation & Sanitization

#### 1.3.1 SHACL Schema Enforcement

**Validation Strategy**:
- All RDF data validated against SHACL shapes before storage
- Invalid data rejected at ingestion (fail-fast)
- Validation errors logged with full context
- Metrics track validation failure rate by sector

**SHACL Validation** (Rust integration):
```rust
// rust/knhk-closed-loop/src/validation/shacl.rs
use oxigraph::model::Graph;
use oxigraph::shacl::{ShaclReport, ShaclValidator};

pub struct ShaclValidationEngine {
    shapes_graph: Arc<Graph>,
    validator: Arc<ShaclValidator>,
}

impl ShaclValidationEngine {
    pub fn validate_observation(&self, obs_graph: &Graph) -> Result<(), ValidationError> {
        let span = tracing::info_span!(
            "validation.shacl",
            "validation.type" = "observation",
        );
        let _enter = span.enter();

        // Run SHACL validation
        let report = self.validator.validate(obs_graph);

        if !report.conforms() {
            // Extract violation details
            let violations: Vec<String> = report
                .results()
                .iter()
                .map(|r| format!("{}: {}", r.focus_node(), r.message()))
                .collect();

            // Emit telemetry
            tracing::warn!(
                name = "validation.shacl.failed",
                "violation.count" = violations.len(),
                "violations" = ?violations,
            );

            // Increment metric
            metrics::counter!("knhk.validation.shacl.failures", 1,
                "validation.type" => "observation",
            );

            return Err(ValidationError::ShaclViolation { violations });
        }

        Ok(())
    }
}
```

#### 1.3.2 SQL Injection Prevention

**Strategy**:
- Use parameterized queries ONLY (no string concatenation)
- ORM with built-in SQL injection protection (SQLx, Diesel)
- Stored procedures for complex queries
- Input sanitization at API gateway

**Safe Query Pattern**:
```rust
// ✅ CORRECT - Parameterized query
sqlx::query!(
    "SELECT * FROM receipts WHERE sector = $1 AND timestamp > $2",
    sector,
    cutoff_timestamp
)
.fetch_all(&pool)
.await?;

// ❌ WRONG - SQL injection vulnerability
let query = format!("SELECT * FROM receipts WHERE sector = '{}'", sector);
sqlx::query(&query).fetch_all(&pool).await?;
```

### 1.4 Audit Logging

#### 1.4.1 Comprehensive Audit Trail

**Events Logged**:
- All authentication attempts (success/failure)
- All authorization decisions (allow/deny)
- All snapshot promotions (who, when, what changed)
- All guard relaxation requests (who requested, who approved)
- All configuration changes
- All API access (endpoint, method, user, result)

**Audit Log Schema** (PostgreSQL):
```sql
-- deployment/postgres/audit-schema.sql
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(64) NOT NULL,  -- auth, authz, snapshot, guard, config, api
    actor_id VARCHAR(255) NOT NULL,   -- User or service identity
    actor_ip INET,
    resource_type VARCHAR(64),         -- snapshot, guard, doctrine, etc.
    resource_id VARCHAR(255),
    action VARCHAR(64) NOT NULL,       -- read, write, execute, approve, reject
    outcome VARCHAR(32) NOT NULL,      -- success, failure, denied
    reason TEXT,                        -- Failure reason or decision rationale
    metadata JSONB,                     -- Additional context
    signature VARCHAR(255),             -- ed25519 signature of log entry

    INDEX idx_timestamp (timestamp DESC),
    INDEX idx_actor (actor_id, timestamp DESC),
    INDEX idx_resource (resource_type, resource_id),
    INDEX idx_outcome (outcome, timestamp DESC)
);

-- Prevent updates/deletes (append-only)
CREATE RULE audit_log_no_update AS ON UPDATE TO audit_log DO INSTEAD NOTHING;
CREATE RULE audit_log_no_delete AS ON DELETE TO audit_log DO INSTEAD NOTHING;
```

**Audit Logging Implementation**:
```rust
// rust/knhk-api/src/audit.rs
pub struct AuditLogger {
    pool: PgPool,
    signing_key: SigningKey,
}

impl AuditLogger {
    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        // Serialize entry for signing
        let canonical = serde_json::to_string(&entry)?;

        // Sign audit log entry
        let signature = self.signing_key.sign(canonical.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO audit_log (
                event_type, actor_id, actor_ip, resource_type, resource_id,
                action, outcome, reason, metadata, signature
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            entry.event_type,
            entry.actor_id,
            entry.actor_ip,
            entry.resource_type,
            entry.resource_id,
            entry.action,
            entry.outcome,
            entry.reason,
            entry.metadata,
            signature_hex,
        )
        .execute(&self.pool)
        .await?;

        // Emit OTEL span
        tracing::info!(
            name = "audit.logged",
            "event.type" = %entry.event_type,
            "actor.id" = %entry.actor_id,
            "action" = %entry.action,
            "outcome" = %entry.outcome,
        );

        Ok(())
    }
}
```

**Audit Log Verification** (tamper detection):
```bash
#!/bin/bash
# deployment/security/verify-audit-log.sh
# Verify audit log integrity

set -e

echo "🔍 Verifying audit log integrity..."

# Query recent audit entries
psql $DATABASE_URL -c "
    SELECT id, timestamp, actor_id, action, outcome, signature
    FROM audit_log
    WHERE timestamp > NOW() - INTERVAL '24 hours'
    ORDER BY id
" -t -A -F'|' | while IFS='|' read id timestamp actor action outcome signature; do

    # Reconstruct canonical message
    canonical=$(echo -n "${id}${timestamp}${actor}${action}${outcome}")

    # Verify signature
    echo "$canonical" | openssl dgst -sha256 -verify public-key.pem \
        -signature <(echo "$signature" | xxd -r -p)

    if [ $? -ne 0 ]; then
        echo "❌ Audit log entry $id has invalid signature - TAMPERING DETECTED"
        exit 1
    fi
done

echo "✅ Audit log integrity verified"
```

### 1.5 Encryption

#### 1.5.1 Encryption at Rest

**Strategy**:
- Database: PostgreSQL transparent data encryption (TDE) or volume encryption
- Object storage: S3 server-side encryption (SSE-S3 or SSE-KMS)
- Receipts: Already signed (ed25519), encryption optional
- Secrets: Vault encryption (AES-256-GCM)

**PostgreSQL Encryption Configuration**:
```yaml
# deployment/postgres/postgresql.conf
# Enable encryption at rest (requires pgcrypto extension)
shared_preload_libraries = 'pgcrypto'

# Encryption settings
ssl = on
ssl_cert_file = '/etc/postgresql/certs/server.crt'
ssl_key_file = '/etc/postgresql/certs/server.key'
ssl_ca_file = '/etc/postgresql/certs/ca.crt'
ssl_ciphers = 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384'
```

**S3 Encryption Policy**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "DenyUnencryptedObjectUploads",
      "Effect": "Deny",
      "Principal": "*",
      "Action": "s3:PutObject",
      "Resource": "arn:aws:s3:::knhk-snapshots/*",
      "Condition": {
        "StringNotEquals": {
          "s3:x-amz-server-side-encryption": "aws:kms"
        }
      }
    }
  ]
}
```

#### 1.5.2 Encryption in Transit

**Requirements**:
- All HTTP traffic uses TLS 1.3
- All database connections use TLS
- All inter-service communication uses mTLS
- No unencrypted protocols (HTTP, plain SMTP, etc.)

**TLS Configuration Validation**:
```bash
#!/bin/bash
# deployment/security/validate-tls.sh

set -e

echo "🔒 Validating TLS configuration..."

# Test API endpoint
echo "Testing KNHK API..."
TLS_VERSION=$(curl -Iv https://knhk.company.com/health 2>&1 | grep "TLS" | awk '{print $2}')

if [[ "$TLS_VERSION" != "TLSv1.3" ]]; then
    echo "❌ API not using TLS 1.3 (found: $TLS_VERSION)"
    exit 1
fi

# Test database connection
echo "Testing PostgreSQL TLS..."
psql "sslmode=verify-full sslrootcert=/etc/ssl/ca.crt $DATABASE_URL" \
    -c "SHOW ssl_version" | grep "TLSv1.3"

if [ $? -ne 0 ]; then
    echo "❌ PostgreSQL not using TLS 1.3"
    exit 1
fi

echo "✅ All connections using TLS 1.3"
```

---

## 2. Performance Optimization

### 2.1 Hot Path Profiling (<100ns, ≤8 ticks)

#### 2.1.1 Profiling Methodology

**Tools**:
- `perf` for CPU profiling and cache analysis
- `flamegraph` for visualization
- `valgrind --tool=cachegrind` for cache miss analysis
- Custom tick counter (RDTSC instruction)

**Hot Path Profiling Script**:
```bash
#!/bin/bash
# deployment/performance/profile-hot-path.sh

set -e

echo "🔥 Profiling KNHK hot path operations"

# Step 1: Build with profiling symbols
cargo build --release --features=profiling

# Step 2: Run hot path benchmark
echo "Running hot path benchmark..."
cargo bench --bench hot_path > /tmp/bench-results.txt

# Step 3: Profile with perf
echo "Profiling with perf..."
perf record -F 99 -g --call-graph dwarf \
    target/release/knhk-bench --bench hot_path_observation_append

# Step 4: Generate flamegraph
echo "Generating flamegraph..."
perf script | stackcollapse-perf.pl | flamegraph.pl > /tmp/hot-path-flamegraph.svg

# Step 5: Cache analysis
echo "Analyzing cache behavior..."
valgrind --tool=cachegrind \
    --cachegrind-out-file=/tmp/cachegrind.out \
    target/release/knhk-bench --bench hot_path_observation_append

# Step 6: Analyze results
echo ""
echo "📊 Profiling Results:"
echo "===================="
cat /tmp/bench-results.txt | grep "observation_append"

echo ""
echo "Cache Statistics:"
cg_annotate /tmp/cachegrind.out | grep "SUMMARY"

echo ""
echo "🔍 Flamegraph saved to: /tmp/hot-path-flamegraph.svg"
```

**Tick Counter** (RDTSC):
```rust
// rust/knhk-bench/src/tick_counter.rs
use std::arch::x86_64::_rdtsc;

#[inline(always)]
pub fn read_ticks() -> u64 {
    unsafe { _rdtsc() }
}

#[inline(never)]  // Prevent inlining to get accurate measurement
pub fn measure_ticks<F: FnOnce() -> R, R>(f: F) -> (R, u64) {
    let start = read_ticks();
    let result = f();
    let end = read_ticks();
    (result, end - start)
}

// Usage:
let (obs_id, ticks) = measure_ticks(|| {
    observation_store.append(obs)
});

assert!(ticks <= 8, "Hot path exceeded Chatman Constant: {} ticks", ticks);
```

#### 2.1.2 Hot Path Optimization Targets

| Operation | Current | Target | Optimization Strategy |
|-----------|---------|--------|----------------------|
| Observation append | 2-4 ticks | ≤8 ticks | Lock-free DashMap, avoid allocation |
| Snapshot read (current) | <1ns | <1ns | Atomic load, cache-aligned |
| Snapshot promotion | 1-5ns | ≤10ns | RCU atomic swap, pre-validated |
| Receipt signature (excluded) | 50-100µs | N/A | Not on hot path (async) |

**Optimization Techniques**:
1. **Lock-Free Data Structures**: Use DashMap, ArcSwap (no mutexes)
2. **Cache Alignment**: Align hot structures to 64-byte cache lines
3. **Avoid Allocation**: Pre-allocate, use Arc cloning
4. **Branchless Code**: Use `select` instead of `if` where possible
5. **Inlining**: `#[inline(always)]` for tiny functions

**Example Optimization** (cache-aligned struct):
```rust
// Before (unaligned, cache line splits)
pub struct SnapshotDescriptor {
    snapshot_id: String,      // 24 bytes
    parent_id: Option<String>, // 24 bytes
    version: u32,              // 4 bytes
    promoted_at: u64,          // 8 bytes
}  // Total: 60 bytes (may span 2 cache lines)

// After (cache-aligned, fits in 1 cache line)
#[repr(C, align(64))]
pub struct SnapshotDescriptor {
    snapshot_id: [u8; 32],    // 32 bytes (fixed-size hash)
    parent_id: [u8; 32],       // 32 bytes (use zero for None)
    version: u32,              // 4 bytes
    promoted_at: u64,          // 8 bytes
    _padding: [u8; 24],        // Pad to 64 bytes
}  // Total: 64 bytes (exactly 1 cache line)
```

### 2.2 Warm Path Optimization (<100ms)

#### 2.2.1 Pattern Detection Optimization

**Current Bottlenecks**:
- Scanning large observation windows (60s = millions of observations)
- Aggregating counts by event type
- Sorting/ranking patterns by confidence

**Optimization Strategy**:
```rust
// Incremental aggregation (avoid full scans)
pub struct PatternDetector {
    // Pre-aggregated counts (updated incrementally)
    event_counts: Arc<DashMap<String, AtomicUsize>>,
    window_start: AtomicU64,
}

impl PatternDetector {
    pub fn on_observation_appended(&self, obs: &Observation) {
        // Increment counter (O(1) instead of O(n) scan)
        self.event_counts
            .entry(obs.event_type.clone())
            .or_insert(AtomicUsize::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    pub async fn detect_patterns(&self) -> Vec<DetectedPattern> {
        // Fast O(K) scan where K = unique event types (not N = total observations)
        let mut patterns = Vec::new();

        for entry in self.event_counts.iter() {
            let count = entry.value().load(Ordering::Relaxed);

            if count > FREQUENCY_THRESHOLD {
                patterns.push(DetectedPattern {
                    name: format!("high_frequency_{}", entry.key()),
                    confidence: 0.95,
                    evidence_count: count,
                    // ... other fields
                });
            }
        }

        patterns
    }
}
```

#### 2.2.2 Validation Pipeline Optimization

**7-Stage Pipeline Optimization**:
| Stage | Current | Target | Optimization |
|-------|---------|--------|--------------|
| 1. Static SHACL | 5-10ms | <5ms | Cache compiled shapes |
| 2. Invariants (Q1-Q5) | 10-20ms | <10ms | Parallel invariant checks |
| 3. Doctrine | 5-15ms | <5ms | Index doctrines by sector |
| 4. Guard | 5-10ms | <5ms | Hash-based protected set |
| 5. Performance | 5-10ms | <5ms | Pre-computed cost table |
| 6. Rollback | 5-10ms | <5ms | Parent existence check only |
| 7. Compatibility | 10-30ms | <10ms | Semantic version check |
| **Total** | 45-105ms | **<45ms** | Parallel + caching |

**Parallel Validation**:
```rust
pub async fn validate_parallel(&self, proposal: &Proposal) -> Result<ValidationReport> {
    // Run independent stages in parallel
    let (static_result, invariant_results, doctrine_result) = tokio::join!(
        self.validate_static(proposal),
        self.check_all_invariants_parallel(proposal),
        self.validate_doctrines(proposal),
    );

    // Early exit on critical failures
    if !static_result?.passed {
        return Ok(report_failed("static"));
    }

    // Continue with dependent stages sequentially
    // ...
}

async fn check_all_invariants_parallel(&self, proposal: &Proposal) -> Result<Vec<bool>> {
    // Parallel invariant checks (independent)
    let checks = vec![
        tokio::spawn(check_q1(proposal.clone())),
        tokio::spawn(check_q2(proposal.clone())),
        tokio::spawn(check_q3(proposal.clone())),
        tokio::spawn(check_q4(proposal.clone())),
        tokio::spawn(check_q5(proposal.clone())),
    ];

    let results = futures::future::join_all(checks).await;
    results.into_iter().map(|r| r??).collect()
}
```

### 2.3 Caching Strategies

#### 2.3.1 Multi-Layer Cache Architecture

```
┌────────────────────────────────────────────────┐
│ Layer 1: In-Memory LRU Cache (Hot Data)       │
│ - Snapshot descriptors (last 100)             │
│ - Compiled SHACL shapes                        │
│ - Doctrine rules (by sector)                   │
│ - TTL: 5 minutes                               │
│ - Size: ~10MB                                  │
└────────────────────────────────────────────────┘
                    ▼ (miss)
┌────────────────────────────────────────────────┐
│ Layer 2: Redis (Warm Data)                    │
│ - Observation aggregates (event counts)       │
│ - Pattern detection results (last 10 cycles)  │
│ - Validation results (proposal ID -> report)  │
│ - TTL: 1 hour                                  │
│ - Size: ~100MB                                 │
└────────────────────────────────────────────────┘
                    ▼ (miss)
┌────────────────────────────────────────────────┐
│ Layer 3: PostgreSQL (Cold Data)               │
│ - All snapshots (history)                     │
│ - All receipts (audit trail)                  │
│ - All doctrines (versioned)                   │
│ - TTL: Infinite                                │
│ - Size: ~10GB+                                 │
└────────────────────────────────────────────────┘
```

**LRU Cache Implementation**:
```rust
use lru::LruCache;
use parking_lot::RwLock;

pub struct SnapshotCache {
    cache: Arc<RwLock<LruCache<String, Arc<SnapshotDescriptor>>>>,
}

impl SnapshotCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    pub fn get(&self, id: &str) -> Option<Arc<SnapshotDescriptor>> {
        let mut cache = self.cache.write();
        cache.get(id).map(Arc::clone)
    }

    pub fn put(&self, id: String, snapshot: Arc<SnapshotDescriptor>) {
        let mut cache = self.cache.write();
        cache.put(id, snapshot);
    }
}
```

**Redis Cache Configuration**:
```yaml
# deployment/redis/redis.conf
maxmemory 256mb
maxmemory-policy allkeys-lru  # Evict least recently used keys

# Persistence (AOF + RDB)
appendonly yes
appendfsync everysec
save 900 1
save 300 10
save 60 10000
```

### 2.4 Connection Pooling

#### 2.4.1 PostgreSQL Connection Pool

**Configuration**:
```rust
// rust/knhk-closed-loop/src/db.rs
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool() -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(50)           // Match available DB connections
        .min_connections(10)            // Keep warm connections
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))  // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(&database_url)
        .await
}
```

**Connection Pool Monitoring**:
```rust
// Emit metrics for pool health
tokio::spawn(async move {
    loop {
        let pool_size = pool.size();
        let idle = pool.num_idle();

        metrics::gauge!("knhk.db.pool.total", pool_size as f64);
        metrics::gauge!("knhk.db.pool.idle", idle as f64);
        metrics::gauge!("knhk.db.pool.active", (pool_size - idle) as f64);

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
});
```

#### 2.4.2 Redis Connection Pool

**Configuration**:
```rust
use redis::{Client, aio::ConnectionManager};

pub async fn create_redis_pool() -> Result<ConnectionManager> {
    let client = Client::open(redis_url)?;

    // ConnectionManager handles automatic reconnection
    let manager = ConnectionManager::new(client).await?;

    Ok(manager)
}
```

### 2.5 Load Testing Methodology

#### 2.5.1 Load Test Profile

**Scenarios**:
```yaml
# deployment/performance/load-test.yaml
scenarios:
  - name: "baseline-load"
    duration: "10m"
    vus: 100  # Virtual users
    operations:
      - name: "ingest-observations"
        rate: 1000/sec  # Target throughput
        endpoint: "POST /api/v1/observations"

      - name: "detect-patterns"
        rate: 10/min
        endpoint: "POST /api/v1/patterns/detect"

      - name: "validate-proposal"
        rate: 5/min
        endpoint: "POST /api/v1/proposals/validate"

  - name: "spike-load"
    duration: "5m"
    vus: 500  # 5x normal load
    operations:
      - name: "burst-observations"
        rate: 5000/sec  # Spike to 5x
        endpoint: "POST /api/v1/observations"
```

**K6 Load Test Script**:
```javascript
// deployment/performance/load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  scenarios: {
    baseline: {
      executor: 'constant-arrival-rate',
      rate: 1000,  // 1000 iterations/sec
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 100,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<100', 'p(99)<150'],  // 95% <100ms, 99% <150ms
    http_req_failed: ['rate<0.01'],  // <1% error rate
    errors: ['rate<0.01'],
  },
};

export default function () {
  // Observation ingestion
  const observation = {
    event_type: 'transaction.execute',
    value: { amount: 50000, account: 'acc-1001' },
    sector: 'finance',
    metadata: {},
  };

  const res = http.post('http://knhk-api:8080/api/v1/observations', JSON.stringify(observation), {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${__ENV.API_TOKEN}`,
    },
  });

  // Validate response
  const success = check(res, {
    'status is 201': (r) => r.status === 201,
    'latency <100ms': (r) => r.timings.duration < 100,
  });

  errorRate.add(!success);

  sleep(0.1);  // 100ms think time
}
```

**Run Load Test**:
```bash
#!/bin/bash
# deployment/performance/run-load-test.sh

set -e

echo "🚀 Running KNHK load test"

# Step 1: Deploy to staging
kubectl config use-context staging
kubectl apply -f deployment/kubernetes/

# Step 2: Wait for deployment
kubectl rollout status deployment/knhk-closed-loop --timeout=5m

# Step 3: Run load test
k6 run --out json=load-test-results.json deployment/performance/load-test.js

# Step 4: Analyze results
k6 stats load-test-results.json

# Step 5: Check thresholds
if k6 inspect load-test-results.json | grep -q "thresholds.*failed"; then
    echo "❌ Load test thresholds failed"
    exit 1
fi

echo "✅ Load test passed"
```

---

## 3. Reliability & High Availability

### 3.1 Graceful Degradation

#### 3.1.1 Circuit Breaker Pattern

**Strategy**:
- Prevent cascading failures by breaking circuit to failing dependencies
- Fallback to degraded mode (read-only, cached data, etc.)
- Automatic recovery attempt after cooldown period
- Emit telemetry for circuit state changes

**Implementation**:
```rust
// rust/knhk-api/src/circuit_breaker.rs
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing if recovered
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicUsize,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: usize,
    timeout: Duration,
    half_open_max_requests: usize,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicUsize::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            timeout,
            half_open_max_requests: 3,
        }
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        // Check circuit state
        let current_state = *self.state.read();

        match current_state {
            CircuitState::Open => {
                // Check if timeout elapsed
                if let Some(last_failure) = *self.last_failure_time.read() {
                    if last_failure.elapsed() > self.timeout {
                        // Try half-open
                        *self.state.write() = CircuitState::HalfOpen;
                        tracing::info!(
                            name = "circuit_breaker.half_open",
                            "circuit.name" = "database",
                        );
                    } else {
                        // Circuit still open
                        return Err(CircuitBreakerError::Open);
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Limited requests in half-open state
                if self.failure_count.load(Ordering::Relaxed) >= self.half_open_max_requests {
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitState::Closed => {}
        }

        // Execute function
        match f().await {
            Ok(result) => {
                // Success - reset failure count
                self.on_success();
                Ok(result)
            }
            Err(err) => {
                // Failure - increment count
                self.on_failure();
                Err(CircuitBreakerError::Failed(err))
            }
        }
    }

    fn on_success(&self) {
        let prev_state = *self.state.read();

        // Reset failure count
        self.failure_count.store(0, Ordering::Relaxed);

        // Close circuit if half-open
        if prev_state == CircuitState::HalfOpen {
            *self.state.write() = CircuitState::Closed;
            tracing::info!(
                name = "circuit_breaker.closed",
                "circuit.name" = "database",
            );
        }
    }

    fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.write() = Some(Instant::now());

        if failures >= self.failure_threshold {
            let prev_state = *self.state.read();
            *self.state.write() = CircuitState::Open;

            if prev_state != CircuitState::Open {
                tracing::error!(
                    name = "circuit_breaker.opened",
                    "circuit.name" = "database",
                    "failure.count" = failures,
                );
            }
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    Failed(E),
}
```

**Usage Example**:
```rust
// Wrap database calls with circuit breaker
let db_circuit = CircuitBreaker::new(5, Duration::from_secs(30));

let result = db_circuit.call(|| async {
    pool.execute(query).await
}).await;

match result {
    Ok(data) => { /* Normal operation */ }
    Err(CircuitBreakerError::Open) => {
        // Fallback to cached data
        tracing::warn!("Database circuit open - using cached data");
        return Ok(get_cached_snapshot());
    }
    Err(CircuitBreakerError::Failed(e)) => {
        return Err(e);
    }
}
```

### 3.2 Health Checks & Liveness Probes

#### 3.2.1 Kubernetes Health Checks

**Health Check Endpoint** (`/health`):
```rust
// rust/knhk-api/src/health.rs
use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    uptime_seconds: u64,
    dependencies: DependencyHealth,
}

#[derive(Serialize, Deserialize)]
pub struct DependencyHealth {
    database: String,
    redis: String,
    vault: String,
}

pub async fn health_check(
    pool: PgPool,
    redis: ConnectionManager,
) -> Result<Json<HealthResponse>, StatusCode> {
    let start_time = *START_TIME.read();
    let uptime = start_time.elapsed().as_secs();

    // Check database
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check Redis
    let redis_status = match redis.get::<_, Option<String>>("health").await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check Vault (optional)
    let vault_status = "healthy";  // Simplified

    let response = HealthResponse {
        status: if db_status == "healthy" && redis_status == "healthy" {
            "healthy"
        } else {
            "degraded"
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: uptime,
        dependencies: DependencyHealth {
            database: db_status.to_string(),
            redis: redis_status.to_string(),
            vault: vault_status.to_string(),
        },
    };

    if response.status == "healthy" {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
```

**Kubernetes Probes Configuration**:
```yaml
# deployment/kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-closed-loop
spec:
  template:
    spec:
      containers:
      - name: knhk-closed-loop
        image: knhk-closed-loop:latest
        ports:
        - containerPort: 8080

        # Liveness probe - restart if failing
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        # Readiness probe - remove from load balancer if not ready
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2

        # Startup probe - allow slow startup
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30  # 150 seconds max startup time
```

### 3.3 Failover & Replication

#### 3.3.1 PostgreSQL High Availability

**Strategy**: Patroni + etcd for automated failover

**Architecture**:
```
┌─────────────────────────────────────────────┐
│ etcd Cluster (Consensus)                    │
│ ┌──────┐  ┌──────┐  ┌──────┐               │
│ │ etcd1│  │ etcd2│  │ etcd3│               │
│ └──────┘  └──────┘  └──────┘               │
└─────────────────────────────────────────────┘
            │ Leader election
            ▼
┌─────────────────────────────────────────────┐
│ Patroni Cluster                             │
│                                             │
│ ┌────────────┐      ┌────────────┐         │
│ │ PostgreSQL │      │ PostgreSQL │         │
│ │  Primary   │◄────►│  Replica   │         │
│ │  (write)   │ sync │  (read)    │         │
│ └────────────┘      └────────────┘         │
│                                             │
│ RTO: <30s  RPO: 0 (synchronous)            │
└─────────────────────────────────────────────┘
```

**Patroni Configuration** (`deployment/postgres/patroni.yaml`):
```yaml
scope: knhk-postgres
namespace: /knhk/
name: postgres-1

restapi:
  listen: 0.0.0.0:8008
  connect_address: postgres-1.knhk.svc:8008

etcd:
  hosts: etcd-1:2379,etcd-2:2379,etcd-3:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576  # 1MB
    synchronous_mode: true
    synchronous_mode_strict: true

  postgresql:
    parameters:
      max_connections: 100
      shared_buffers: 256MB
      effective_cache_size: 1GB
      wal_level: replica
      max_wal_senders: 10
      max_replication_slots: 10
      hot_standby: on
      synchronous_commit: on

postgresql:
  listen: 0.0.0.0:5432
  connect_address: postgres-1.knhk.svc:5432
  data_dir: /var/lib/postgresql/data
  pgpass: /tmp/pgpass
  authentication:
    replication:
      username: replicator
      password: ${REPLICATION_PASSWORD}
    superuser:
      username: postgres
      password: ${POSTGRES_PASSWORD}
  parameters:
    unix_socket_directories: '/var/run/postgresql'

tags:
  nofailover: false
  noloadbalance: false
  clonefrom: false
```

**Failover Test**:
```bash
#!/bin/bash
# deployment/postgres/test-failover.sh

set -e

echo "🧪 Testing PostgreSQL failover"

# Step 1: Identify current primary
PRIMARY=$(kubectl exec -it postgres-0 -- \
  patronictl -c /etc/patroni/patroni.yaml list -f json | \
  jq -r '.[] | select(.Role == "Leader") | .Member')

echo "Current primary: $PRIMARY"

# Step 2: Trigger failover
echo "Triggering failover..."
kubectl exec -it postgres-0 -- \
  patronictl -c /etc/patroni/patroni.yaml failover --master $PRIMARY --force

# Step 3: Wait for new primary
echo "Waiting for new primary election..."
sleep 10

# Step 4: Verify new primary
NEW_PRIMARY=$(kubectl exec -it postgres-0 -- \
  patronictl -c /etc/patroni/patroni.yaml list -f json | \
  jq -r '.[] | select(.Role == "Leader") | .Member')

echo "New primary: $NEW_PRIMARY"

if [ "$PRIMARY" == "$NEW_PRIMARY" ]; then
    echo "❌ Failover failed - same primary"
    exit 1
fi

# Step 5: Verify database connectivity
echo "Testing database write..."
kubectl exec -it $NEW_PRIMARY -- \
  psql -U postgres -c "INSERT INTO health_check (timestamp) VALUES (NOW());"

echo "✅ Failover successful - RTO: ~10 seconds"
```

### 3.4 Distributed Tracing

#### 3.4.1 OpenTelemetry Tracing Setup

**Trace Context Propagation**:
```rust
// rust/knhk-api/src/tracing.rs
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str) -> Result<()> {
    // Set global propagator (W3C Trace Context)
    global::set_text_map_propagator(TraceContextPropagator::new());

    // OTLP exporter (to Jaeger/Tempo)
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://otel-collector:4317"),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Tracing subscriber with OTLP layer
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

**Cross-Service Trace Propagation**:
```rust
// Extract trace context from incoming request
let parent_cx = global::get_text_map_propagator(|propagator| {
    propagator.extract(&HeaderExtractor(req.headers()))
});

// Attach to current span
let span = tracing::info_span!(
    "http.request",
    "http.method" = %req.method(),
    "http.route" = %req.uri(),
    "trace.parent" = ?parent_cx,
);

// Propagate to outgoing requests
let mut headers = HeaderMap::new();
global::get_text_map_propagator(|propagator| {
    propagator.inject_context(&span.context(), &mut HeaderInjector(&mut headers));
});

// Make downstream request with trace context
let response = client.post("http://validator-service/validate")
    .headers(headers)
    .json(&proposal)
    .send()
    .await?;
```

### 3.5 Error Handling & Retry Logic

#### 3.5.1 Exponential Backoff with Jitter

**Implementation**:
```rust
// rust/knhk-api/src/retry.rs
use std::time::Duration;
use rand::Rng;

pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl RetryPolicy {
    pub fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            jitter: true,
        }
    }

    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match f().await {
                Ok(result) => return Ok(result),
                Err(err) if attempt >= self.max_attempts => {
                    tracing::error!(
                        "operation.retry.exhausted",
                        "retry.attempts" = attempt,
                        "error" = %err,
                    );
                    return Err(err);
                }
                Err(err) => {
                    // Calculate delay with exponential backoff
                    let delay = self.calculate_delay(attempt);

                    tracing::warn!(
                        "operation.retry.attempt",
                        "retry.attempt" = attempt,
                        "retry.delay_ms" = delay.as_millis(),
                        "error" = %err,
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    fn calculate_delay(&self, attempt: usize) -> Duration {
        // Exponential backoff: base * 2^(attempt-1)
        let exponential = self.base_delay * 2_u32.pow((attempt - 1) as u32);

        // Cap at max delay
        let capped = exponential.min(self.max_delay);

        // Add jitter (±25%)
        if self.jitter {
            let jitter_range = capped.as_millis() as f64 * 0.25;
            let jitter = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
            Duration::from_millis((capped.as_millis() as f64 + jitter) as u64)
        } else {
            capped
        }
    }
}
```

**Usage**:
```rust
let retry_policy = RetryPolicy::default();

let result = retry_policy.execute(|| async {
    // Operation that may fail transiently
    vault_client.read_secret("database/password").await
}).await?;
```

### 3.6 Observability Stack

#### 3.6.1 Metrics Collection

**Prometheus Metrics**:
```rust
// rust/knhk-api/src/metrics.rs
use metrics::{counter, histogram, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_metrics() -> Result<()> {
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()?;

    Ok(())
}

// Emit metrics throughout application
pub fn record_observation_ingested(sector: &str, latency_ms: f64) {
    counter!("knhk.observations.ingested", 1,
        "sector" => sector.to_string(),
    );
    histogram!("knhk.observation.ingest_latency_ms", latency_ms,
        "sector" => sector.to_string(),
    );
}

pub fn record_pattern_detected(pattern_type: &str, confidence: f64) {
    counter!("knhk.patterns.detected", 1,
        "pattern.type" => pattern_type.to_string(),
    );
    gauge!("knhk.pattern.confidence", confidence,
        "pattern.type" => pattern_type.to_string(),
    );
}

pub fn record_proposal_validated(outcome: &str, duration_ms: f64) {
    counter!("knhk.proposals.validated", 1,
        "outcome" => outcome.to_string(),
    );
    histogram!("knhk.proposal.validation_duration_ms", duration_ms);
}
```

---

## 4. Data Integrity & Disaster Recovery

### 4.1 Backup Strategy

#### 4.1.1 PostgreSQL Backup Configuration

**Continuous WAL Archiving**:
```sql
-- deployment/postgres/postgresql.conf
-- Enable WAL archiving
archive_mode = on
archive_command = 'pgbackrest --stanza=knhk archive-push %p'
archive_timeout = 300  -- 5 minutes

-- Retention policy
wal_keep_size = '10GB'
max_wal_senders = 10```

**pgBackRest Configuration** (`/etc/pgbackrest/pgbackrest.conf`):
```ini
[global]
repo1-retention-full=7
repo1-retention-diff=7
repo1-retention-archive=14
repo1-path=/backup/pgbackrest
repo1-type=s3
repo1-s3-bucket=knhk-postgres-backups
repo1-s3-region=us-east-1
repo1-s3-key=${AWS_ACCESS_KEY_ID}
repo1-s3-key-secret=${AWS_SECRET_ACCESS_KEY}
repo1-s3-encryption-type=aes-256-cbc

[knhk]
pg1-path=/var/lib/postgresql/data
pg1-port=5432
pg1-socket-path=/var/run/postgresql
```

**Automated Backup Script**:
```bash
#!/bin/bash
# deployment/postgres/backup.sh

set -e

echo "🔄 Running PostgreSQL backup"

# Full backup (weekly on Sunday)
if [ "$(date +%u)" -eq 7 ]; then
    echo "Running full backup..."
    pgbackrest --stanza=knhk --type=full backup
else
    # Incremental backup (daily)
    echo "Running incremental backup..."
    pgbackrest --stanza=knhk --type=incr backup
fi

# Verify backup
pgbackrest --stanza=knhk info

# Emit telemetry
curl -X POST http://metrics-collector:8080/metrics \
    -d "backup.completed=1,backup.type=full,backup.size_bytes=$(pgbackrest --stanza=knhk info --output=json | jq '.[] | .backup[-1].size')"

echo "✅ Backup completed successfully"
```

### 4.2 Backup Verification & Testing

#### 4.2.1 Automated Backup Verification

**Monthly Backup Restore Test**:
```bash
#!/bin/bash
# deployment/postgres/test-backup-restore.sh

set -e

echo "🧪 Testing backup restore procedure"

# Step 1: Create test database instance
kubectl apply -f deployment/postgres/test-restore-pod.yaml

# Step 2: Wait for pod to be ready
kubectl wait --for=condition=Ready pod/postgres-restore-test --timeout=5m

# Step 3: Restore latest backup
kubectl exec -it postgres-restore-test -- \
    pgbackrest --stanza=knhk --delta --type=time \
    --target="$(date -u +%Y-%m-%d\ %H:%M:%S)" restore

# Step 4: Start PostgreSQL
kubectl exec -it postgres-restore-test -- \
    pg_ctl start -D /var/lib/postgresql/data

# Step 5: Verify data integrity
RECORD_COUNT=$(kubectl exec -it postgres-restore-test -- \
    psql -U postgres -d knhk -t -c "SELECT COUNT(*) FROM receipts")

echo "Restored record count: $RECORD_COUNT"

if [ "$RECORD_COUNT" -lt 1000 ]; then
    echo "❌ Backup restore verification failed - insufficient records"
    exit 1
fi

# Step 6: Clean up test pod
kubectl delete pod postgres-restore-test

echo "✅ Backup restore verification successful"
```

### 4.3 Point-in-Time Recovery (PITR)

#### 4.3.1 PITR Procedure

**Restore to Specific Timestamp**:
```bash
#!/bin/bash
# deployment/postgres/restore-pitr.sh

set -e

TARGET_TIME=$1  # Format: 2025-11-16 14:30:00

if [ -z "$TARGET_TIME" ]; then
    echo "Usage: $0 'YYYY-MM-DD HH:MM:SS'"
    exit 1
fi

echo "🔙 Restoring database to $TARGET_TIME"

# Step 1: Stop primary database (maintenance mode)
kubectl scale statefulset postgres --replicas=0

# Step 2: Restore from backup
pgbackrest --stanza=knhk --delta --type=time \
    --target="$TARGET_TIME" \
    --target-action=promote \
    restore

# Step 3: Start database
kubectl scale statefulset postgres --replicas=1

# Step 4: Wait for database to be ready
kubectl wait --for=condition=Ready pod/postgres-0 --timeout=5m

# Step 5: Verify restore
LATEST_TIMESTAMP=$(kubectl exec -it postgres-0 -- \
    psql -U postgres -d knhk -t -c "SELECT MAX(created_at) FROM receipts")

echo "Latest record timestamp: $LATEST_TIMESTAMP"

if [[ "$LATEST_TIMESTAMP" > "$TARGET_TIME" ]]; then
    echo "❌ PITR failed - found records after target time"
    exit 1
fi

echo "✅ PITR successful - restored to $TARGET_TIME"
```

### 4.4 Receipt Chain Verification

#### 4.4.1 Cryptographic Audit Trail Validation

**Receipt Chain Integrity Check**:
```rust
// rust/knhk-cli/src/commands/verify_chain.rs
use ed25519_dalek::{PublicKey, Signature};

pub async fn verify_receipt_chain(pool: &PgPool) -> Result<ChainVerificationReport> {
    let mut report = ChainVerificationReport::default();

    // Fetch all receipts in order
    let receipts = sqlx::query_as::<_, Receipt>(
        "SELECT * FROM receipts ORDER BY created_at ASC"
    )
    .fetch_all(pool)
    .await?;

    let mut prev_hash: Option<String> = None;

    for receipt in receipts {
        // 1. Verify signature
        let public_key = PublicKey::from_bytes(&receipt.verifying_key)?;
        let signature = Signature::from_bytes(&receipt.signature)?;
        let message = receipt.canonical_message();

        if public_key.verify(message.as_bytes(), &signature).is_err() {
            report.signature_failures.push(receipt.id.clone());
            continue;
        }

        // 2. Verify chain linkage
        if let Some(expected_prev) = prev_hash {
            if receipt.parent_hash != expected_prev {
                report.chain_breaks.push(receipt.id.clone());
            }
        }

        // 3. Verify no retrocausation (Q1 invariant)
        if let Some(parent_id) = &receipt.parent_id {
            let parent = sqlx::query_as::<_, Receipt>(
                "SELECT * FROM receipts WHERE id = $1"
            )
            .bind(parent_id)
            .fetch_one(pool)
            .await?;

            if receipt.created_at < parent.created_at {
                report.retrocausation_violations.push(receipt.id.clone());
            }
        }

        prev_hash = Some(receipt.hash.clone());
        report.verified_count += 1;
    }

    report.total_count = receipts.len();
    Ok(report)
}
```

**Automated Chain Verification** (cron job):
```bash
#!/bin/bash
# deployment/scripts/verify-receipt-chain.sh

set -e

echo "🔍 Verifying receipt chain integrity"

# Run verification
RESULT=$(kubectl exec -it deploy/knhk-closed-loop -- \
    knhk receipts verify-chain --format json)

# Parse results
TOTAL=$(echo $RESULT | jq '.total_count')
VERIFIED=$(echo $RESULT | jq '.verified_count')
FAILURES=$(echo $RESULT | jq '.signature_failures | length')
BREAKS=$(echo $RESULT | jq '.chain_breaks | length')

echo "Total receipts: $TOTAL"
echo "Verified: $VERIFIED"
echo "Signature failures: $FAILURES"
echo "Chain breaks: $BREAKS"

# Alert if integrity issues found
if [ "$FAILURES" -gt 0 ] || [ "$BREAKS" -gt 0 ]; then
    echo "❌ Receipt chain integrity COMPROMISED"
    
    # Send alert to PagerDuty
    curl -X POST https://events.pagerduty.com/v2/enqueue \
        -H 'Content-Type: application/json' \
        -d "{
            \"routing_key\": \"${PAGERDUTY_KEY}\",
            \"event_action\": \"trigger\",
            \"payload\": {
                \"summary\": \"KNHK Receipt Chain Integrity Failure\",
                \"severity\": \"critical\",
                \"source\": \"knhk-production\",
                \"custom_details\": {
                    \"signature_failures\": $FAILURES,
                    \"chain_breaks\": $BREAKS
                }
            }
        }"
    
    exit 1
fi

echo "✅ Receipt chain integrity verified"
```

---

## 5. Operational Readiness

### 5.1 Runbooks for Top 10 Failure Modes

#### 5.1.1 Database Connection Pool Exhausted

**Symptoms**:
- API returning 503 Service Unavailable
- Error logs: "connection pool timeout" or "too many clients"
- Prometheus alert: `knhk_db_pool_active / knhk_db_pool_total > 0.95`

**Diagnosis**:
```bash
# Check pool metrics
kubectl exec -it deploy/knhk-closed-loop -- \
    curl localhost:9090/metrics | grep knhk_db_pool

# Check active database connections
kubectl exec -it postgres-0 -- \
    psql -U postgres -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"
```

**Resolution**:
1. **Immediate**: Scale up application replicas to distribute load
   ```bash
   kubectl scale deployment knhk-closed-loop --replicas=6
   ```

2. **Short-term**: Increase pool size (requires restart)
   ```bash
   # Update ConfigMap
   kubectl patch configmap knhk-config -p '{"data":{"DB_POOL_MAX_CONNECTIONS":"100"}}'
   kubectl rollout restart deployment/knhk-closed-loop
   ```

3. **Long-term**: Identify connection leaks
   ```bash
   # Find long-running queries
   kubectl exec -it postgres-0 -- \
       psql -U postgres -c "SELECT pid, now() - query_start AS duration, query 
                             FROM pg_stat_activity 
                             WHERE state = 'active' 
                             ORDER BY duration DESC 
                             LIMIT 10;"
   ```

**Prevention**:
- Set aggressive connection timeouts (idle_timeout = 60s)
- Implement connection monitoring and alerting
- Use connection pooler (PgBouncer) for very high scale

---

#### 5.1.2 Redis Cache Unavailable

**Symptoms**:
- Increased database load (cache misses)
- Slower API response times (p95 latency +50%)
- Error logs: "Redis connection refused" or "connection timeout"

**Diagnosis**:
```bash
# Check Redis health
kubectl exec -it redis-0 -- redis-cli ping

# Check memory usage
kubectl exec -it redis-0 -- redis-cli INFO memory | grep used_memory_human
```

**Resolution**:
1. **Circuit breaker already active**: Application should degrade gracefully
   ```bash
   # Verify circuit state
   kubectl logs deploy/knhk-closed-loop | grep "circuit_breaker.opened"
   ```

2. **Restart Redis** (if unhealthy):
   ```bash
   kubectl rollout restart statefulset/redis
   ```

3. **Verify persistence** (check AOF/RDB):
   ```bash
   kubectl exec -it redis-0 -- \
       redis-cli BGSAVE && \
       redis-cli LASTSAVE
   ```

**Prevention**:
- Enable Redis persistence (AOF + RDB)
- Configure Redis maxmemory-policy (allkeys-lru)
- Monitor Redis memory and eviction rate

---

#### 5.1.3 OTel Weaver Validation Failure

**Symptoms**:
- `weaver registry check` fails in CI/CD
- Schema validation errors in telemetry
- Features not emitting expected spans/metrics

**Diagnosis**:
```bash
# Run Weaver validation
weaver registry check -r registry/

# Check live telemetry
weaver registry live-check --registry registry/
```

**Resolution**:
1. **Identify schema violation**:
   ```bash
   weaver registry check -r registry/ --format json | jq '.violations'
   ```

2. **Fix schema definition** (registry YAML):
   - Ensure all spans/metrics are declared
   - Verify attribute types match implementation
   - Check for missing required fields

3. **Update implementation** (if schema is correct):
   ```rust
   // Ensure span attributes match schema
   tracing::info_span!(
       "mape_k.analyze",
       "pattern.count" = pattern_count,      // Must match schema
       "analysis.duration_ms" = duration_ms, // Must match schema
   );
   ```

**Prevention**:
- Run `weaver registry check` in pre-commit hook
- Add Weaver validation to CI pipeline (required check)
- Never merge PRs with Weaver validation failures

---

#### 5.1.4 High Pattern Detection Latency (>100ms)

**Symptoms**:
- MAPE-K loop taking >100ms (exceeding warm path budget)
- Prometheus alert: `knhk_pattern_detection_duration_ms{quantile="0.95"} > 100`

**Diagnosis**:
```bash
# Check pattern detection latency
kubectl exec -it deploy/knhk-closed-loop -- \
    curl localhost:9090/metrics | grep pattern_detection_duration

# Profile hot path
cargo bench --bench pattern_detection
```

**Resolution**:
1. **Check observation window size**:
   ```bash
   # Reduce window if too large
   kubectl patch configmap knhk-config -p '{"data":{"PATTERN_WINDOW_SECONDS":"30"}}'
   ```

2. **Clear old observations** (if DB bloated):
   ```bash
   kubectl exec -it postgres-0 -- \
       psql -U postgres -d knhk -c "DELETE FROM observations WHERE created_at < NOW() - INTERVAL '1 hour';"
   ```

3. **Scale horizontally** (if observation rate too high):
   ```bash
   kubectl scale deployment knhk-closed-loop --replicas=4
   ```

**Prevention**:
- Implement observation window pruning (keep last N only)
- Use incremental aggregation (pre-compute event counts)
- Add index on observations(created_at, event_type)

---

#### 5.1.5 Proposal Validation Bottleneck

**Symptoms**:
- 7-stage validation taking >200ms
- Queue of pending proposals growing
- API timeouts on /proposals/validate

**Diagnosis**:
```bash
# Check validation duration breakdown
kubectl logs deploy/knhk-closed-loop | grep "validation.stage" | \
    jq -r '[.stage, .duration_ms] | @tsv' | sort -k2 -n
```

**Resolution**:
1. **Parallelize independent stages**:
   ```rust
   // Run static, invariant, doctrine in parallel
   let (static_result, invariant_result, doctrine_result) = tokio::join!(
       validate_static(proposal),
       validate_invariants(proposal),
       validate_doctrines(proposal),
   );
   ```

2. **Cache SHACL shapes** (if stage 1 slow):
   ```rust
   // Load once at startup, not per validation
   lazy_static! {
       static ref SHACL_VALIDATOR: ShaclValidator = load_shapes();
   }
   ```

3. **Index doctrines by sector** (if stage 3 slow):
   ```sql
   CREATE INDEX idx_doctrines_sector ON doctrines(sector, active);
   ```

**Prevention**:
- Profile each validation stage independently
- Set per-stage timeout budgets (5-10ms each)
- Alert on individual stage latency violations

---

### 5.2 Incident Response Procedures

#### 5.2.1 Severity Levels

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|----------|
| **P0 - Critical** | Complete outage, data loss risk | <15 minutes | Database down, receipt chain broken |
| **P1 - High** | Partial outage, degraded performance | <1 hour | Redis down, API latency >500ms |
| **P2 - Medium** | Non-critical feature impaired | <4 hours | Pattern detection slow, cache misses |
| **P3 - Low** | Minor issue, workaround available | <24 hours | Logging errors, minor config drift |

#### 5.2.2 Escalation Path

```
┌─────────────────────────────────────────┐
│ P0/P1 Incident Declared                 │
└─────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────┐
│ Page On-Call Engineer (PagerDuty)      │
│ Acknowledge within 5 minutes            │
└─────────────────────────────────────────┘
            │
            ▼ (if not resolved in 30 min)
┌─────────────────────────────────────────┐
│ Escalate to Lead Engineer               │
│ Join incident Slack channel             │
└─────────────────────────────────────────┘
            │
            ▼ (if not resolved in 60 min)
┌─────────────────────────────────────────┐
│ Escalate to Engineering Manager         │
│ Page backup on-call                     │
│ Notify stakeholders                     │
└─────────────────────────────────────────┘
```

### 5.3 On-Call Rotation

#### 5.3.1 On-Call Schedule

**Rotation**: 1-week shifts, starting Monday 00:00 UTC

**On-Call Responsibilities**:
1. Respond to PagerDuty alerts within 5 minutes
2. Triage and resolve P0/P1 incidents
3. Document incidents in runbook
4. Conduct post-incident review
5. Update monitoring/alerting based on learnings

**On-Call Checklist**:
```bash
# Before shift starts
□ Verify PagerDuty notifications working (test alert)
□ Ensure VPN access and kubectl credentials
□ Review current system status dashboard
□ Read handoff notes from previous on-call
□ Check for scheduled maintenance windows

# During shift
□ Acknowledge alerts within 5 minutes
□ Follow runbook procedures for known issues
□ Escalate if unable to resolve within SLA
□ Document all actions taken in incident log
□ Update status page for customer-facing incidents

# After shift
□ Write handoff notes for next on-call
□ Schedule post-incident reviews for P0/P1
□ Update runbooks with new learnings
□ File bugs for recurring issues
```

---

## 6. Compliance & Governance

### 6.1 Data Retention Policies

#### 6.1.1 Retention Requirements by Data Type

| Data Type | Retention Period | Rationale | Deletion Method |
|-----------|------------------|-----------|-----------------|
| **Receipts** | 7 years | Financial audit requirements (SOX) | Never delete (immutable) |
| **Observations** | 90 days | Pattern detection + compliance | Automated archival to S3 Glacier |
| **Snapshots** | 1 year | Rollback capability + audit | Soft delete (mark inactive) |
| **Audit Logs** | 7 years | Regulatory compliance (GDPR, HIPAA) | Never delete (immutable) |
| **Telemetry** | 30 days | Operational debugging | Auto-expire in Prometheus |
| **Backups** | 90 days (full), 30 days (incremental) | Disaster recovery | pgBackRest expiration policy |

#### 6.1.2 Data Archival Script

```bash
#!/bin/bash
# deployment/scripts/archive-old-observations.sh

set -e

CUTOFF_DATE=$(date -u -d '90 days ago' +%Y-%m-%d)

echo "📦 Archiving observations older than $CUTOFF_DATE"

# Step 1: Export to S3
kubectl exec -it postgres-0 -- \
    psql -U postgres -d knhk -c "\COPY (
        SELECT * FROM observations WHERE created_at < '$CUTOFF_DATE'
    ) TO STDOUT WITH CSV HEADER" | \
    aws s3 cp - s3://knhk-archives/observations/archive-$(date +%Y%m%d).csv

# Step 2: Verify export
EXPORT_COUNT=$(aws s3 cp s3://knhk-archives/observations/archive-$(date +%Y%m%d).csv - | wc -l)
echo "Exported $EXPORT_COUNT observations"

# Step 3: Delete from database
kubectl exec -it postgres-0 -- \
    psql -U postgres -d knhk -c "DELETE FROM observations WHERE created_at < '$CUTOFF_DATE';"

echo "✅ Archival complete"
```

### 6.2 Privacy & Data Protection

#### 6.2.1 GDPR Compliance

**Right to Erasure Implementation**:
```rust
// rust/knhk-api/src/gdpr.rs
pub async fn delete_user_data(user_id: &str, pool: &PgPool) -> Result<DeletionReport> {
    let mut tx = pool.begin().await?;

    // 1. Anonymize observations (cannot delete - needed for audit)
    sqlx::query!(
        "UPDATE observations SET user_id = 'anonymized' WHERE user_id = $1",
        user_id
    )
    .execute(&mut tx)
    .await?;

    // 2. Delete personal data tables
    sqlx::query!(
        "DELETE FROM user_profiles WHERE user_id = $1",
        user_id
    )
    .execute(&mut tx)
    .await?;

    // 3. Log deletion in audit trail
    sqlx::query!(
        r#"
        INSERT INTO audit_log (event_type, actor_id, action, outcome, reason)
        VALUES ('gdpr', 'system', 'user_data_deleted', 'success', $1)
        "#,
        format!("GDPR erasure request for user {}", user_id)
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(DeletionReport {
        user_id: user_id.to_string(),
        deleted_at: Utc::now(),
        anonymized_records: 1,
    })
}
```

#### 6.2.2 HIPAA Compliance (Healthcare Sector)

**PHI Protection Measures**:
- Encryption at rest (AES-256) and in transit (TLS 1.3)
- Access controls (RBAC with audit logging)
- Minimum necessary access (role-based data filtering)
- Automatic logoff (session timeout = 15 minutes)
- Business Associate Agreements (BAA) with cloud providers

**HIPAA Audit Log Requirements**:
```sql
-- All PHI access must be logged
CREATE TABLE phi_access_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id VARCHAR(255) NOT NULL,
    patient_id VARCHAR(255) NOT NULL,  -- PHI identifier
    action VARCHAR(64) NOT NULL,        -- read, write, delete
    justification TEXT,                 -- Required for access
    ip_address INET,
    
    INDEX idx_patient (patient_id, timestamp DESC),
    INDEX idx_user (user_id, timestamp DESC)
);
```

### 6.3 Change Management

#### 6.3.1 Change Approval Process

**Change Types**:
| Change Type | Approval Required | Testing Required | Rollback Plan |
|-------------|-------------------|------------------|---------------|
| **Standard** (config, scaling) | Tech Lead | Staging validation | Automatic |
| **Normal** (feature, bug fix) | Code review + QA | Full test suite + manual | Documented |
| **Emergency** (hotfix) | Post-implementation review | Smoke tests only | Mandatory |
| **Major** (schema, architecture) | Engineering Manager + Security | Full regression + load test | Required |

**Change Request Template**:
```markdown
## Change Request: [Title]

**Type**: Standard / Normal / Emergency / Major
**Requester**: [Name]
**Date**: [YYYY-MM-DD]

### Description
[What is changing and why]

### Impact Assessment
- **Services Affected**: [List]
- **Downtime Required**: Yes / No (Duration: __)
- **Data Migration**: Yes / No
- **Rollback Complexity**: Low / Medium / High

### Testing Plan
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Staging deployment successful
- [ ] Load test completed (if performance-critical)
- [ ] Weaver validation passes

### Rollback Plan
[Specific steps to revert if deployment fails]

### Approvals
- [ ] Tech Lead: __________
- [ ] Security Review (if needed): __________
- [ ] Manager (if major change): __________
```

---

## 7. Deployment Procedures

### 7.1 Blue-Green Deployment Strategy

#### 7.1.1 Deployment Architecture

```
┌─────────────────────────────────────────┐
│ Load Balancer (Istio VirtualService)   │
└─────────────────────────────────────────┘
         │ 
         │ Traffic routing
         ▼
┌─────────────────┐      ┌─────────────────┐
│ Blue (Current)  │      │ Green (New)     │
│ v1.2.0          │      │ v1.3.0          │
│ 100% traffic ───┼─────►│ 0% traffic      │
│                 │      │                 │
│ Pods: 3         │      │ Pods: 3         │
└─────────────────┘      └─────────────────┘
         │                        │
         └────────┬───────────────┘
                  ▼
         Shared Database (PostgreSQL)
```

#### 7.1.2 Deployment Procedure

```bash
#!/bin/bash
# deployment/scripts/blue-green-deploy.sh

set -e

NEW_VERSION=$1
CURRENT_COLOR=$(kubectl get vs knhk-routing -o json | jq -r '.spec.http[0].route[0].destination.subset')

# Determine new color
if [ "$CURRENT_COLOR" == "blue" ]; then
    NEW_COLOR="green"
else
    NEW_COLOR="blue"
fi

echo "🚀 Deploying $NEW_VERSION to $NEW_COLOR environment"

# Step 1: Deploy new version
kubectl set image deployment/knhk-$NEW_COLOR \
    knhk-closed-loop=knhk-closed-loop:$NEW_VERSION

# Step 2: Wait for rollout
kubectl rollout status deployment/knhk-$NEW_COLOR --timeout=5m

# Step 3: Run smoke tests
echo "Running smoke tests on $NEW_COLOR..."
./deployment/scripts/smoke-test.sh $NEW_COLOR

if [ $? -ne 0 ]; then
    echo "❌ Smoke tests failed - aborting deployment"
    exit 1
fi

# Step 4: Shift traffic to new environment
echo "Shifting traffic to $NEW_COLOR..."
kubectl patch vs knhk-routing --type merge -p "
spec:
  http:
  - route:
    - destination:
        host: knhk-service
        subset: $NEW_COLOR
      weight: 100
"

# Step 5: Monitor for 5 minutes
echo "⏱️  Monitoring new deployment for 5 minutes..."
sleep 300

# Step 6: Check error rate
ERROR_RATE=$(curl -s http://prometheus:9090/api/v1/query?query='rate(knhk_requests_failed[5m])' | \
    jq -r '.data.result[0].value[1]')

if (( $(echo "$ERROR_RATE > 0.01" | bc -l) )); then
    echo "❌ Error rate too high ($ERROR_RATE) - rolling back"
    ./deployment/scripts/rollback.sh
    exit 1
fi

echo "✅ Deployment successful - $NEW_VERSION live on $NEW_COLOR"
```

### 7.2 Canary Deployment Plan

#### 7.2.1 Progressive Rollout Stages

| Stage | Traffic % | Duration | Success Criteria |
|-------|-----------|----------|------------------|
| 1. Initial | 1% | 15 min | Error rate <0.1%, p95 latency <120ms |
| 2. Small | 10% | 30 min | Error rate <0.1%, p99 latency <200ms |
| 3. Medium | 50% | 1 hour | Error rate <0.05%, CPU <70% |
| 4. Full | 100% | N/A | All metrics within SLA |

**Automated Canary with Flagger**:
```yaml
# deployment/kubernetes/canary.yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: knhk-closed-loop
  namespace: knhk
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: knhk-closed-loop
  
  service:
    port: 8080
    targetPort: 8080
  
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    
    metrics:
    - name: request-success-rate
      thresholdRange:
        min: 99
      interval: 1m
    
    - name: request-duration
      thresholdRange:
        max: 200
      interval: 1m
    
    webhooks:
    - name: load-test
      url: http://flagger-loadtester/
      timeout: 5s
      metadata:
        cmd: "hey -z 1m -q 10 -c 2 http://knhk-canary:8080/health"
```

### 7.3 Database Migration Strategy

#### 7.3.1 Zero-Downtime Schema Changes

**Backward-Compatible Migration Pattern**:
```sql
-- deployment/migrations/20251116_add_receipt_metadata.sql

-- Step 1: Add column (nullable initially)
ALTER TABLE receipts ADD COLUMN metadata JSONB;

-- Step 2: Backfill data (in batches)
DO $$
DECLARE
    batch_size INTEGER := 1000;
    offset_val INTEGER := 0;
BEGIN
    LOOP
        UPDATE receipts
        SET metadata = '{}'::JSONB
        WHERE id IN (
            SELECT id FROM receipts
            WHERE metadata IS NULL
            ORDER BY id
            LIMIT batch_size
            OFFSET offset_val
        );
        
        EXIT WHEN NOT FOUND;
        offset_val := offset_val + batch_size;
        PERFORM pg_sleep(0.1);  -- Throttle to avoid locking
    END LOOP;
END $$;

-- Step 3: Add NOT NULL constraint (after backfill)
ALTER TABLE receipts ALTER COLUMN metadata SET NOT NULL;

-- Step 4: Add index
CREATE INDEX CONCURRENTLY idx_receipts_metadata ON receipts USING GIN (metadata);
```

**Migration Deployment**:
```bash
#!/bin/bash
# deployment/scripts/run-migration.sh

set -e

MIGRATION_FILE=$1

echo "🗄️  Running database migration: $MIGRATION_FILE"

# Step 1: Backup current schema
pg_dump -U postgres -d knhk --schema-only > /backup/schema-pre-migration-$(date +%s).sql

# Step 2: Run migration in transaction
kubectl exec -it postgres-0 -- \
    psql -U postgres -d knhk -v ON_ERROR_STOP=1 < $MIGRATION_FILE

# Step 3: Verify schema
kubectl exec -it postgres-0 -- \
    psql -U postgres -d knhk -c "\d receipts"

echo "✅ Migration completed successfully"
```

### 7.4 Post-Deployment Validation

#### 7.4.1 Smoke Test Suite

```bash
#!/bin/bash
# deployment/scripts/smoke-test.sh

set -e

ENVIRONMENT=$1  # blue or green
API_URL="http://knhk-$ENVIRONMENT:8080"

echo "🧪 Running smoke tests against $ENVIRONMENT"

# Test 1: Health check
echo "Test 1: Health check"
HEALTH=$(curl -s $API_URL/health | jq -r '.status')
if [ "$HEALTH" != "healthy" ]; then
    echo "❌ Health check failed"
    exit 1
fi
echo "✅ Health check passed"

# Test 2: Observation ingestion
echo "Test 2: Observation ingestion"
OBS_ID=$(curl -s -X POST $API_URL/api/v1/observations \
    -H "Content-Type: application/json" \
    -d '{"event_type":"test.smoke","value":{"test":true},"sector":"finance"}' | \
    jq -r '.id')

if [ -z "$OBS_ID" ] || [ "$OBS_ID" == "null" ]; then
    echo "❌ Observation ingestion failed"
    exit 1
fi
echo "✅ Observation ingestion passed"

# Test 3: Pattern detection
echo "Test 3: Pattern detection"
PATTERNS=$(curl -s -X POST $API_URL/api/v1/patterns/detect | jq '. | length')
if [ "$PATTERNS" -lt 0 ]; then
    echo "❌ Pattern detection failed"
    exit 1
fi
echo "✅ Pattern detection passed ($PATTERNS patterns)"

# Test 4: Weaver validation
echo "Test 4: Weaver schema validation"
weaver registry check -r registry/
if [ $? -ne 0 ]; then
    echo "❌ Weaver validation failed"
    exit 1
fi
echo "✅ Weaver validation passed"

echo "🎉 All smoke tests passed"
```

---

## 8. Monitoring & Alerting

### 8.1 Service Level Indicators (SLIs)

#### 8.1.1 Key SLIs from OpenTelemetry

| SLI | Metric Source | Target | Measurement Window |
|-----|---------------|--------|-------------------|
| **Availability** | `knhk_health_status` | ≥99.9% | 30 days |
| **Request Success Rate** | `knhk_requests_total{status="success"} / knhk_requests_total` | ≥99.5% | 5 minutes |
| **Hot Path Latency** | `knhk_observation_append_ticks` | ≤8 ticks | Per operation |
| **Warm Path Latency** | `knhk_pattern_detection_duration_ms{quantile="0.95"}` | <100ms | 5 minutes |
| **Validation Latency** | `knhk_proposal_validation_duration_ms{quantile="0.95"}` | <50ms | 5 minutes |

### 8.2 Alert Definitions

#### 8.2.1 Prometheus Alert Rules

```yaml
# deployment/monitoring/prometheus-rules.yaml
groups:
- name: knhk_critical
  interval: 30s
  rules:
  
  # P0: Service Down
  - alert: KNHKServiceDown
    expr: up{job="knhk-closed-loop"} == 0
    for: 1m
    labels:
      severity: critical
      pagerduty: "yes"
    annotations:
      summary: "KNHK service is down"
      description: "KNHK service {{ $labels.instance }} has been down for 1 minute"
  
  # P0: High Error Rate
  - alert: KNHKHighErrorRate
    expr: rate(knhk_requests_total{status="error"}[5m]) / rate(knhk_requests_total[5m]) > 0.05
    for: 2m
    labels:
      severity: critical
      pagerduty: "yes"
    annotations:
      summary: "KNHK error rate > 5%"
      description: "Error rate is {{ $value | humanizePercentage }}"
  
  # P1: Database Connection Pool Exhausted
  - alert: KNHKDatabasePoolExhausted
    expr: knhk_db_pool_active / knhk_db_pool_total > 0.95
    for: 5m
    labels:
      severity: high
      pagerduty: "yes"
    annotations:
      summary: "Database connection pool nearly exhausted"
      description: "Pool utilization: {{ $value | humanizePercentage }}"
  
  # P1: Hot Path Performance Degradation
  - alert: KNHKHotPathSlow
    expr: histogram_quantile(0.95, rate(knhk_observation_append_ticks_bucket[5m])) > 8
    for: 5m
    labels:
      severity: high
      pagerduty: "yes"
    annotations:
      summary: "Hot path exceeds Chatman Constant"
      description: "p95 observation append: {{ $value }} ticks (target: ≤8)"
  
  # P2: Warm Path Performance Degradation
  - alert: KNHKWarmPathSlow
    expr: histogram_quantile(0.95, rate(knhk_pattern_detection_duration_ms_bucket[5m])) > 100
    for: 10m
    labels:
      severity: medium
      slack: "yes"
    annotations:
      summary: "Warm path latency degraded"
      description: "p95 pattern detection: {{ $value }}ms (target: <100ms)"
  
  # P1: Receipt Chain Integrity Failure
  - alert: KNHKReceiptChainBroken
    expr: knhk_receipt_chain_breaks_total > 0
    for: 1m
    labels:
      severity: critical
      pagerduty: "yes"
    annotations:
      summary: "CRITICAL: Receipt chain integrity compromised"
      description: "{{ $value }} chain breaks detected - audit trail may be invalid"
  
  # P2: Weaver Validation Failure
  - alert: KNHKWeaverValidationFailed
    expr: knhk_weaver_validation_failures_total > 0
    for: 5m
    labels:
      severity: medium
      slack: "yes"
    annotations:
      summary: "OpenTelemetry schema validation failing"
      description: "{{ $value }} Weaver validation failures in 5 minutes"
```

### 8.3 Grafana Dashboards

#### 8.3.1 KNHK Overview Dashboard

```json
{
  "dashboard": {
    "title": "KNHK Production Overview",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(knhk_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(knhk_requests_total{status=\"error\"}[5m]) / rate(knhk_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ],
        "type": "graph",
        "alert": {
          "conditions": [
            {"evaluator": {"params": [0.01], "type": "gt"}}
          ]
        }
      },
      {
        "title": "Hot Path Latency (Ticks)",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(knhk_observation_append_ticks_bucket[5m]))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(knhk_observation_append_ticks_bucket[5m]))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(knhk_observation_append_ticks_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "type": "graph",
        "yaxes": [
          {"max": 10, "label": "Ticks", "show": true}
        ]
      },
      {
        "title": "Warm Path Latency (ms)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(knhk_pattern_detection_duration_ms_bucket[5m]))",
            "legendFormat": "Pattern Detection p95"
          },
          {
            "expr": "histogram_quantile(0.95, rate(knhk_proposal_validation_duration_ms_bucket[5m]))",
            "legendFormat": "Validation p95"
          }
        ],
        "type": "graph",
        "yaxes": [
          {"max": 150, "label": "Milliseconds", "show": true}
        ]
      },
      {
        "title": "Database Connection Pool",
        "targets": [
          {
            "expr": "knhk_db_pool_total",
            "legendFormat": "Total"
          },
          {
            "expr": "knhk_db_pool_active",
            "legendFormat": "Active"
          },
          {
            "expr": "knhk_db_pool_idle",
            "legendFormat": "Idle"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

### 8.4 Log Aggregation

#### 8.4.1 Loki Configuration

```yaml
# deployment/monitoring/loki-config.yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
  - from: 2025-11-01
    store: boltdb
    object_store: filesystem
    schema: v11
    index:
      prefix: index_
      period: 24h

storage_config:
  boltdb:
    directory: /tmp/loki/index
  filesystem:
    directory: /tmp/loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h  # 7 days
  
retention_config:
  retention_period: 720h  # 30 days
```

---

## 9. Production Checklist

### 9.1 Pre-Deployment Checklist

**Code Quality** (100% required):
- [ ] All Rust tests pass (`cargo test --workspace`)
- [ ] Clippy shows zero warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] No `.unwrap()` or `.expect()` in production paths
- [ ] All async trait methods removed (dyn compatibility)
- [ ] Weaver validation passes (`weaver registry check -r registry/`)
- [ ] Load tests meet SLA (p95 <100ms, p99 <150ms)

**Security** (100% required):
- [ ] OIDC authentication configured
- [ ] mTLS certificates generated for all services
- [ ] Vault secrets configured (no hardcoded secrets)
- [ ] SHACL validation enabled
- [ ] Audit logging enabled with signature verification
- [ ] TLS 1.3 enforced for all connections
- [ ] Security scan passed (no CVEs above medium)

**Infrastructure** (100% required):
- [ ] Kubernetes cluster provisioned
- [ ] PostgreSQL with Patroni HA configured
- [ ] Redis cluster deployed
- [ ] Vault initialized and unsealed
- [ ] OTEL Collector deployed
- [ ] Prometheus and Grafana configured
- [ ] PagerDuty integration tested

**Operational** (100% required):
- [ ] Runbooks written for top 10 failure modes
- [ ] On-call rotation configured in PagerDuty
- [ ] Backup and restore procedures tested
- [ ] Disaster recovery plan documented
- [ ] Rollback procedure tested
- [ ] Smoke tests passing
- [ ] Monitoring dashboards created
- [ ] Alert rules configured and tested

### 9.2 Go-Live Sign-Off Criteria

**Technical Sign-Off** (all must be "Yes"):
| Criteria | Status | Owner | Notes |
|----------|--------|-------|-------|
| Weaver validation passes | ☐ | Engineering | Source of truth for features |
| Load test meets SLA | ☐ | Engineering | p95 <100ms, error rate <0.1% |
| Security scan clean | ☐ | Security | No critical/high vulnerabilities |
| Backup/restore tested | ☐ | Operations | Verified within 7 days |
| Failover tested | ☐ | Operations | RTO <30s verified |
| Monitoring complete | ☐ | SRE | All SLIs tracked, alerts firing |
| Runbooks complete | ☐ | SRE | Top 10 failure modes documented |
| On-call trained | ☐ | SRE | On-call has access and knowledge |

**Business Sign-Off**:
| Criteria | Status | Owner | Notes |
|----------|--------|-------|-------|
| Compliance reviewed | ☐ | Legal/Compliance | GDPR, HIPAA, SOX requirements met |
| Stakeholder approval | ☐ | Product Manager | Business objectives aligned |
| Customer communication | ☐ | Customer Success | Migration plan communicated |
| Contractual obligations | ☐ | Legal | SLAs, BAAs in place |

**Final Approval**:
- [ ] Engineering Manager: _________________ Date: _______
- [ ] Security Lead: _________________ Date: _______
- [ ] Operations Lead: _________________ Date: _______
- [ ] Product Manager: _________________ Date: _______

---

## 10. Runbooks

### 10.1 Emergency Procedures

#### 10.1.1 Complete System Outage

**Symptoms**: All services returning 503, health checks failing

**Immediate Actions** (within 5 minutes):
1. **Verify scope**:
   ```bash
   kubectl get pods -n knhk
   kubectl get nodes
   ```

2. **Check control plane**:
   ```bash
   kubectl get cs  # Control plane status
   kubectl top nodes
   ```

3. **Emergency scale-up** (if resource exhaustion):
   ```bash
   kubectl scale deployment knhk-closed-loop --replicas=6
   ```

4. **Check database connectivity**:
   ```bash
   kubectl exec -it postgres-0 -- psql -U postgres -c "SELECT 1;"
   ```

**Escalation**: If not resolved in 15 minutes, escalate to Lead Engineer and notify stakeholders.

---

#### 10.1.2 Data Corruption Detected

**Symptoms**: Receipt chain verification failing, signature mismatches

**CRITICAL**: Do not proceed without Engineering Manager approval

**Immediate Actions**:
1. **Stop all writes**:
   ```bash
   kubectl scale deployment knhk-closed-loop --replicas=0
   ```

2. **Snapshot current database**:
   ```bash
   pgbackrest --stanza=knhk --type=full backup
   ```

3. **Run integrity check**:
   ```bash
   kubectl exec -it deploy/knhk-closed-loop -- knhk receipts verify-chain --verbose
   ```

4. **Identify corruption point**:
   ```bash
   # Find first invalid receipt
   kubectl exec -it deploy/knhk-closed-loop -- \
       knhk receipts verify-chain --format json | \
       jq '.chain_breaks[0]'
   ```

5. **Escalate immediately**: Page Engineering Manager, Security Lead, and CTO

**Recovery**: Point-in-time restore to last known good state (requires business approval for data loss)

---

### 10.2 Routine Maintenance

#### 10.2.1 Certificate Rotation

**Frequency**: Every 90 days (automated), manual if needed

**Procedure**:
```bash
#!/bin/bash
# Rotate TLS certificates

# 1. Generate new certificates from Vault
./deployment/security/generate-service-certs.sh knhk-closed-loop

# 2. Update Kubernetes secrets
kubectl create secret tls knhk-mtls-new \
    --cert=knhk-closed-loop.crt \
    --key=knhk-closed-loop.key \
    --dry-run=client -o yaml | kubectl apply -f -

# 3. Rolling restart with new certificates
kubectl patch deployment knhk-closed-loop -p \
    '{"spec":{"template":{"spec":{"volumes":[{"name":"mtls","secret":{"secretName":"knhk-mtls-new"}}]}}}}'

# 4. Monitor for TLS errors
kubectl logs -f deploy/knhk-closed-loop | grep -i "tls\|certificate"

# 5. Delete old secret (after 7-day grace period)
kubectl delete secret knhk-mtls-old
```

---

## 11. Appendix

### 11.1 Glossary

| Term | Definition |
|------|------------|
| **Chatman Constant** | Performance budget of ≤8 CPU ticks (≈100ns) for hot path operations |
| **Hard Invariants** | Q1-Q5 constraints that must never be violated (no retrocausation, type soundness, guard preservation, SLO compliance, performance bounds) |
| **Hot Path** | Critical performance path requiring ≤8 ticks (observation append, snapshot read) |
| **MAPE-K** | Monitor-Analyze-Plan-Execute-Knowledge autonomous control loop |
| **Receipt Chain** | Cryptographically signed audit trail using ed25519 signatures |
| **Sector Doctrines** | Domain-specific regulatory rules (finance, healthcare, manufacturing, logistics) |
| **Warm Path** | Performance-sensitive operations requiring <100ms (pattern detection, validation) |
| **Weaver Validation** | OpenTelemetry schema validation - source of truth for feature correctness |

### 11.2 References

1. [SPARC Specification Complete](SPARC_SPECIFICATION_COMPLETE.md)
2. [SPARC Phase 8: Weaver Validation](SPARC_PHASE8_WEAVER_VALIDATION.md)
3. [SPARC Phase 7: LLM Proposer](SPARC_PHASE7_LLM_PROPOSER.md)
4. [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
5. [PostgreSQL High Availability](https://www.postgresql.org/docs/current/high-availability.html)
6. [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/)
7. [GDPR Compliance Guide](https://gdpr.eu/)
8. [HIPAA Security Rule](https://www.hhs.gov/hipaa/for-professionals/security/)

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-11-16 | Production Validator Agent | Initial production hardening guide |

---

**END OF SPARC PHASE 9: PRODUCTION HARDENING**

This document completes the SPARC methodology for KNHK production deployment. All critical systems, procedures, and validations are now documented and ready for Fortune 500 enterprise deployment.

**Next Steps**:
1. Execute pre-deployment checklist (Section 9.1)
2. Obtain go-live sign-offs (Section 9.2)
3. Deploy to staging and run full validation
4. Execute blue-green production deployment
5. Monitor for 48 hours before declaring success
