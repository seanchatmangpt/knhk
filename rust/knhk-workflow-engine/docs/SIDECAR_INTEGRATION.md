# Sidecar Integration Guide

## Overview

The KNHK Workflow Engine integrates with the KNHK sidecar to provide Fortune 5 enterprise features:
- **SPIFFE/SPIRE Authentication**: Service identity and mTLS
- **KMS Integration**: Secure secret management
- **Service Mesh**: Traffic management and observability
- **Health Checks**: Sidecar health monitoring

## Features

### SPIFFE/SPIRE Authentication

The sidecar provides SPIFFE-based service identity for secure service-to-service communication:

```rust
use knhk_workflow_engine::{WorkflowEngine, SidecarIntegration};

let engine = WorkflowEngine::new(state_store)?;
if let Some(sidecar) = engine.sidecar_integration() {
    let identity = sidecar.get_identity().await?;
    println!("Service identity: {}", identity.unwrap_or_default());
}
```

### KMS Integration

Retrieve secrets securely from KMS:

```rust
if let Some(sidecar) = engine.sidecar_integration() {
    let api_key = sidecar.get_secret("api-key").await?;
    // Use secret securely
}
```

### Peer Verification

Verify peer service identity for mTLS:

```rust
if let Some(sidecar) = engine.sidecar_integration() {
    let is_valid = sidecar.verify_peer("spiffe://trust-domain/service").await?;
    if is_valid {
        // Proceed with request
    }
}
```

## Configuration

### Enable Sidecar Integration

Add the `sidecar` feature to `Cargo.toml`:

```toml
[dependencies]
knhk-workflow-engine = { path = "../knhk-workflow-engine", features = ["sidecar"] }
```

### Environment Variables

```bash
# Enable sidecar
WORKFLOW_SIDECAR_ENABLED=true

# SPIFFE configuration
SPIFFE_SOCKET_PATH=/tmp/spire-agent.sock
SPIFFE_TRUST_DOMAIN=knhk.org

# KMS configuration
KMS_PROVIDER=aws  # aws, azure, gcp, vault
KMS_REGION=us-east-1
```

## Usage

### Initialize Sidecar

```rust
use knhk_workflow_engine::integration::SidecarIntegration;

let sidecar = SidecarIntegration::new(true);
sidecar.initialize().await?;
```

### With Custom Config

```rust
use knhk_sidecar::Config as SidecarConfig;
use knhk_workflow_engine::integration::SidecarIntegration;

let config = SidecarConfig::default()
    .with_trust_domain("knhk.org")
    .with_spiffe_socket("/tmp/spire-agent.sock");

let sidecar = SidecarIntegration::with_config(config);
sidecar.initialize().await?;
```

## Integration with Workflow Engine

The sidecar integration is automatically available when the `sidecar` feature is enabled:

```rust
use knhk_workflow_engine::{WorkflowEngine, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);

// Sidecar integration is available if enabled
if let Some(sidecar) = engine.sidecar_integration() {
    // Use sidecar features
    let identity = sidecar.get_identity().await?;
}
```

## Security

### Service Identity

All service-to-service communication uses SPIFFE identities:
- Each service has a unique SPIFFE ID
- mTLS certificates are automatically rotated
- Trust domain validation ensures secure communication

### Secret Management

Secrets are retrieved from KMS:
- Never stored in code or configuration
- Automatically rotated by KMS
- Access controlled via IAM/RBAC

## Troubleshooting

### Sidecar Not Initialized

Check if sidecar is enabled:
```rust
if let Some(sidecar) = engine.sidecar_integration() {
    if sidecar.is_enabled() {
        // Sidecar is available
    }
}
```

### SPIFFE Identity Not Available

Verify SPIFFE agent is running:
```bash
# Check SPIFFE agent socket
ls -la /tmp/spire-agent.sock

# Check SPIFFE agent logs
journalctl -u spire-agent
```

### KMS Access Denied

Verify IAM/RBAC permissions:
- Service account has KMS read permissions
- Trust domain matches configuration
- Region matches KMS region

## Future Enhancements

1. **Automatic Certificate Rotation**: Sidecar handles certificate rotation
2. **Service Mesh Integration**: Istio/Linkerd integration
3. **Advanced Observability**: Sidecar metrics and tracing
4. **Policy Enforcement**: Sidecar-based policy enforcement

