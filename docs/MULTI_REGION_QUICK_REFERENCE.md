# Multi-Region Receipt Sync - Quick Reference

## Key Files
- **Implementation**: `/home/user/knhk/rust/knhk-sidecar/src/multi_region.rs` (800 lines)
- **Tests**: `/home/user/knhk/rust/knhk-sidecar/tests/multi_region_sync_tests.rs`
- **Full Docs**: `/home/user/knhk/docs/PHASE3_MULTI_REGION_IMPLEMENTATION.md`

---

## Creating a Manager

### Basic Setup
```rust
use knhk_sidecar::multi_region::*;

let config = MultiRegionConfig::new("us-east-1".to_string())
    .add_region(RemoteRegion::new(
        "us-west-1".to_string(),
        "http://replica-west.example.com".to_string(),
    ))
    .with_cross_region_sync(true)
    .with_quorum_threshold(2);

let mut manager = ReceiptSyncManager::new(config)?;
```

### From Legacy Config
```rust
let legacy = RegionConfig { /* ... */ };
let mut manager = ReceiptSyncManager::from_region_config(legacy)?;
```

---

## Syncing a Receipt

```rust
let receipt = Receipt {
    receipt_id: "rec-123".to_string(),
    transaction_id: "tx-456".to_string(),
    hash: vec![1, 2, 3, 4],
    ticks: 42,
    span_id: 789,
    committed: true,
};

let result = manager.sync_receipt(&receipt).await?;

assert!(result.quorum_achieved);
assert_eq!(result.synced_regions, 2);
assert!(result.errors.is_empty());
```

---

## Verifying Consensus

```rust
let quorum_met = manager.verify_quorum("rec-123").await?;

if quorum_met {
    println!("Receipt consensus reached across regions");
} else {
    println!("Quorum not met - handling degraded mode");
}
```

---

## Checking Status

### Get Per-Region Status
```rust
let status = manager.get_sync_status().await?;

for region in status {
    println!(
        "{}: available={}, failures={}, last_sync={:?}",
        region.region_id,
        region.is_available,
        region.failure_count,
        region.last_sync_timestamp
    );
}
```

### Get Health Overview
```rust
let (available, total, quorum_ok) = manager.get_health_status().await?;

println!("Regions: {}/{} available", available, total);
println!("Quorum: {}", if quorum_ok { "OK" } else { "FAILED" });
```

---

## Configuration Patterns

### Three-Region High Availability
```rust
MultiRegionConfig::new("us-east-1".to_string())
    .add_region(RemoteRegion::new("us-west-1".to_string(), "http://west.com".into()))
    .add_region(RemoteRegion::new("eu-west-1".to_string(), "http://eu.com".into()))
    .with_cross_region_sync(true)
    .with_quorum_threshold(2)   // 2 of 3
    .with_max_retries(3)
```

### Custom Timeouts
```rust
RemoteRegion::new("us-west-1".to_string(), "http://slow.com".into())
    .with_timeout(Duration::from_secs(30))  // Slower region
```

### Weighted Regions
```rust
RemoteRegion::new("primary-replica".to_string(), "http://primary.com".into())
    .with_weight(2)  // Double weight for weighted quorum
```

---

## Retry Behavior

### Exponential Backoff Sequence
```
Attempt 1: Immediate
Attempt 2: 1s wait
Attempt 3: 2s wait
Attempt 4: 4s wait
Attempt 5+: 4s wait (capped)
```

### Retryable Errors
- Network timeouts
- Connection refused/reset
- HTTP 502, 503, 504

### Non-Retryable Errors
- Authentication failures
- Invalid receipt data
- 4xx HTTP errors

---

## Error Handling

### Check Quorum Achievement
```rust
let result = manager.sync_receipt(&receipt).await?;

if result.quorum_achieved {
    // Safe to proceed
} else {
    // Log error details
    for (region, error) in &result.errors {
        eprintln!("Region {} failed: {}", region, error);
    }
}
```

### Handle Region Failures
```rust
let status = manager.get_sync_status().await?;

let unavailable: Vec<_> = status
    .iter()
    .filter(|s| !s.is_available)
    .collect();

if !unavailable.is_empty() {
    warn!("Unavailable regions: {:?}", unavailable);
    // Implement circuit breaker or alerting
}
```

---

## Logging Output

### Success Case
```
2025-11-16T12:00:00Z INFO: Starting receipt sync for rec-123 to 2 regions (quorum: 2)
2025-11-16T12:00:00Z INFO: Receipt rec-123 successfully synced to region: us-west-1
2025-11-16T12:00:00Z INFO: Receipt rec-123 successfully synced to region: eu-west-1
2025-11-16T12:00:00Z INFO: Receipt rec-123 sync result: 3/3 regions synced, quorum=true
```

### Retry Case
```
2025-11-16T12:00:00Z WARN: Retrying receipt rec-123 sync to us-west-1 (attempt 1/3, backoff: 1000ms)
2025-11-16T12:00:01Z WARN: Retrying receipt rec-123 sync to us-west-1 (attempt 2/3, backoff: 2000ms)
2025-11-16T12:00:03Z INFO: Receipt rec-123 successfully synced to region: us-west-1 on attempt 3
```

### Failure Case
```
2025-11-16T12:00:00Z WARN: Region us-west-1 marked as unavailable (failure_count: 3)
2025-11-16T12:00:00Z ERROR: Quorum consensus failed for receipt rec-123: 2/3 regions confirmed
```

---

## Testing

### Run All Tests
```bash
cd /home/user/knhk/rust
cargo test --package knhk-sidecar --test multi_region_sync_tests
```

### Run Specific Test
```bash
cargo test --package knhk-sidecar --test multi_region_sync_tests -- test_sync_receipt_disabled
```

### Test Coverage
- Configuration validation (10 tests)
- Backward compatibility (2 tests)
- Manager operations (4 tests)
- Status tracking (3 tests)
- Builder pattern (2 tests)

---

## Common Patterns

### Initialize on Startup
```rust
let config = sidecar_config.to_multi_region_config();
let manager = ReceiptSyncManager::new(config)
    .map_err(|e| {
        error!("Failed to initialize multi-region: {}", e);
        e
    })?;

info!("Multi-region receipt sync ready");
```

### Health Check Endpoint
```rust
pub async fn health_check(manager: &ReceiptSyncManager) -> HealthStatus {
    match manager.get_health_status().await {
        Ok((available, total, quorum_ok)) => {
            HealthStatus {
                regions_available: available,
                regions_total: total,
                quorum_status: quorum_ok,
            }
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            HealthStatus::degraded()
        }
    }
}
```

### Graceful Degradation
```rust
let result = manager.sync_receipt(&receipt).await?;

if result.quorum_achieved {
    // Proceed normally
    Ok(())
} else if result.synced_regions > 1 {
    // Partial success - log warning but continue
    warn!("Partial quorum: {} regions", result.synced_regions);
    Ok(())
} else {
    // Complete failure
    error!("No regions synced successfully");
    Err(SidecarError::consensus_error("..."))
}
```

---

## Performance Tuning

### Reduce Latency
```rust
// Increase quorum threshold to allow faster returns
.with_quorum_threshold(1)  // Just local + 1 remote

// Reduce timeout for quick failures
RemoteRegion::new("fast-region".into(), url)
    .with_timeout(Duration::from_millis(500))
```

### Improve Reliability
```rust
// Increase max retries for unstable networks
.with_max_retries(5)

// Increase initial backoff to reduce thundering herd
.with_retry_backoff(Duration::from_secs(2))
```

### Monitor Performance
```rust
let start = Instant::now();
let result = manager.sync_receipt(&receipt).await?;
let elapsed = start.elapsed();

info!("Sync took {:?}, quorum={}", elapsed, result.quorum_achieved);
```

---

## Troubleshooting

### Issue: Always Returns Unavailable
**Solution**: Check that regions are reachable
```bash
curl http://replica-west.example.com/receipt-sync
```

### Issue: Quorum Never Met
**Solution**: Reduce quorum threshold
```rust
.with_quorum_threshold(1)  // Only need local + 1
```

### Issue: Timeouts
**Solution**: Increase per-region timeout
```rust
RemoteRegion::new("slow-region".into(), url)
    .with_timeout(Duration::from_secs(30))
```

### Issue: Too Much Retry Traffic
**Solution**: Reduce max retries or increase backoff
```rust
.with_max_retries(1)
.with_retry_backoff(Duration::from_secs(5))
```

---

## Integration Checklist

- [ ] Add multi-region config to SidecarConfig
- [ ] Initialize ReceiptSyncManager in lib.rs run()
- [ ] Call manager.sync_receipt() after receipt generation
- [ ] Check result.quorum_achieved before committing
- [ ] Add get_sync_status() to health check endpoint
- [ ] Monitor logs for region failures
- [ ] Set up alerts for quorum failures
- [ ] Test failover scenarios
- [ ] Document region endpoints in runbooks
- [ ] Set up cross-region monitoring

---

## Related Documentation

- **Full Implementation**: `PHASE3_MULTI_REGION_IMPLEMENTATION.md`
- **Fortune 500 Features**: See main project README
- **OpenTelemetry**: Weaver validation docs
- **KNHK Design**: Orthogonal features spec
