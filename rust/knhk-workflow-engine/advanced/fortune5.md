# Fortune 5 Integration

Enterprise-grade features for Fortune 5 deployments.

## Features

- **SLO Tracking**: Runtime class-based SLO monitoring
- **Promotion Gates**: Safe deployment with canary/staging/production
- **Multi-Region Support**: Cross-region replication
- **SPIFFE/SPIRE**: Service identity and authentication
- **KMS Integration**: Key management for secrets

## SLO Tracking

Track Service Level Objectives:

```rust
use knhk_workflow_engine::integration::fortune5::{Fortune5Config, SloConfig};

let fortune5_config = Fortune5Config {
    slo: Some(SloConfig {
        r1_p99_max_ns: 2,
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        window_size_seconds: 60,
    }),
    // ...
};

let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)?;
```

## Promotion Gates

Control deployment promotion:

```rust
let allowed = engine.fortune5_integration()
    .unwrap()
    .check_promotion_gate()
    .await?;
```

## Runtime Classes

- **R1**: Hot path (≤2ns P99)
- **W1**: Warm path (≤1ms P99)
- **C1**: Cold path (≤500ms P99)

## Next Steps

- [Fortune 5 Use Cases](use-cases/fortune5.md) - Real-world examples
- [Configuration](reference/configuration.md) - Configuration details

