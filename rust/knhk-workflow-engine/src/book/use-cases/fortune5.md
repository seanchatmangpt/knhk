# Fortune 5 Use Cases

Enterprise deployment use cases for Fortune 5 environments.

## Use Cases

### SLO Compliance

Track and enforce Service Level Objectives:

```rust
let compliant = engine.check_slo_compliance().await?;
```

### Promotion Gates

Control deployment promotion:

```rust
let allowed = engine.fortune5_integration()
    .unwrap()
    .check_promotion_gate()
    .await?;
```

### Multi-Region Deployment

Cross-region replication and failover:

```rust
let config = Fortune5Config {
    multi_region: Some(MultiRegionConfig {
        current_region: "us-east-1".to_string(),
        replication_regions: vec!["us-west-2".to_string()],
        replication_strategy: ReplicationStrategy::Sync,
    }),
    // ...
};
```

## Next Steps

- [Full Use Cases](../docs/FORTUNE5_USE_CASES.md) - Complete documentation
- [Fortune 5 Integration](../advanced/fortune5.md) - Integration guide

