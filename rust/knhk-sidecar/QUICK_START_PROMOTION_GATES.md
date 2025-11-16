# Promotion Gates - Quick Start Guide

## 5-Minute Overview

Promotion Gates enables safe canary deployment with:
- **Deterministic routing**: Same user always gets same version
- **Automatic rollback**: Rolls back if SLOs violated
- **Multi-environment**: Canary → Staging → Production

## Installation

The promotion gates are already integrated in `knhk-sidecar`. Just import and use:

```rust
use knhk_sidecar::promotion::{Environment, PromotionConfig, PromotionGateManager};
```

## Basic Usage (30 seconds)

```rust
// 1. Create config for 10% canary
let config = PromotionConfig {
    environment: Environment::Canary { traffic_percent: 10.0 },
    feature_flags: vec!["new_api".to_string()],
    auto_rollback_enabled: true,
    slo_threshold: 0.95,
    rollback_window_seconds: 300,
};

// 2. Create SLO controller
let slo_controller = SloAdmissionController::new(SloConfig::default())?;

// 3. Create manager
let mut manager = PromotionGateManager::new(config, slo_controller)?;

// 4. Route requests
let decision = manager.route_request("user-123-session");
if decision.is_canary {
    use_new_version(&decision.enabled_features);
} else {
    use_stable_version();
}

// 5. Record outcomes
manager.record_request_outcome("user-123-session", success, duration);

// 6. Monitor health
let health = manager.monitor_canary_health();
if health.health_score >= 0.9 {
    manager.promote(Environment::Staging)?;
}
```

## Routing Algorithm (1 minute)

```
hash = SHA256(request_id)
percent = hash % 100

if percent < traffic_percent:
    route to CANARY
else:
    route to PRODUCTION
```

**Key**: Same `request_id` always produces same routing (deterministic)

## Configuration Examples

### 5% Canary (Very Conservative)
```rust
PromotionConfig {
    environment: Environment::Canary { traffic_percent: 5.0 },
    auto_rollback_enabled: true,
    slo_threshold: 0.95,
    rollback_window_seconds: 300,
    ..
}
```

### Staging (Full Testing)
```rust
PromotionConfig {
    environment: Environment::Staging,
    auto_rollback_enabled: true,
    slo_threshold: 0.95,
    ..
}
```

### Production (Stable)
```rust
PromotionConfig {
    environment: Environment::Production,
    auto_rollback_enabled: true,
    slo_threshold: 0.98,  // Stricter in production
    ..
}
```

## Promotion Workflow

```
DAY 1:  Deploy new version
        Set canary 5%
        Monitor: error_rate, latency
                ↓
DAY 2:  Increase to 10%
        Monitor for 12 hours
                ↓
DAY 3:  Increase to 25%
        Monitor for 12 hours
                ↓
DAY 4:  Promote to Staging
        Full feature testing
                ↓
DAY 5:  Promote to Production
        100% rollout
```

## Key Methods

### Route Request
```rust
let decision = manager.route_request("request-id");
// Returns: RoutingDecision {
//   target_environment: Environment,
//   is_canary: bool,
//   enabled_features: Vec<String>,
//   reason: String,
// }
```

### Record Outcome
```rust
manager.record_request_outcome("request-id", success, duration);
// Tracks metrics for SLO compliance and health monitoring
```

### Check Health
```rust
let health = manager.monitor_canary_health();
// Returns: CanaryHealth {
//   health_score: f64,          // 0.0-1.0
//   canary_error_rate: f64,
//   canary_p99_latency: Duration,
//   recommendation: String,     // HEALTHY / MONITOR / ROLLBACK
// }
```

### Check SLO Compliance
```rust
let compliant = manager.check_slo_compliance()?;
// Returns: true if SLOs are met, false if violated
```

### Promote to Next Environment
```rust
manager.promote(Environment::Staging)?;
// Valid paths:
// - Canary → Staging
// - Staging → Production
// - Canary → Production (direct, not recommended)
```

### Feature Flags
```rust
manager.enable_feature("new_feature".to_string());
manager.disable_feature("new_feature".to_string());
if manager.is_feature_enabled("new_feature") {
    // Use new feature
}
```

### Rollback History
```rust
let history = manager.get_rollback_history();
for event in history {
    println!("{}: {}", event.reason, event.timestamp);
}
```

## Automatic Rollback Triggers

Rollback happens automatically when:

1. **SLO Violation**
   - Canary compliance < threshold
   - Overall compliance < threshold

2. **Health Degradation**
   - Health score < 0.8

3. **Feature Disabled**
   - Manual disable triggers rollback event

When rollback happens:
- All features disabled
- Environment reset to Production
- Metrics cleared
- Event logged in history

## Monitoring Metrics

### Error Rate
```rust
health.canary_error_rate        // Canary error rate
health.production_error_rate    // Production error rate
// Rollback if: canary > production * 1.5
```

### Latency (P99)
```rust
health.canary_p99_latency       // Canary p99
health.production_p99_latency   // Production p99
// Rollback if: canary > production * 1.5
```

### Health Score
```rust
health.health_score              // 0.0 - 1.0
health.recommendation            // String with guidance
// Scores:
// 0.9-1.0: HEALTHY (OK to increase traffic)
// 0.8-0.9: MONITOR (wait before increasing)
// <0.8:    ROLLBACK (auto-triggered)
```

## Logging Output

```rust
// Feature flag changes
info!("Feature flag enabled: new_api")

// Per-request routing (debug level only)
debug!("Request user-123 routed to canary (hash 42%)")

// Health alerts
warn!("Canary health score 0.75 below threshold")

// Rollback events
error!("Triggering automatic rollback: Canary SLO violation: 0.88 < 0.95")
```

## Common Scenarios

### Scenario: Slow Canary Promotion
```rust
// Start: 5% traffic
let decision = manager.route_request("user-123");
// → Detects issues, but limited impact

// After 24 hours: increase to 10%
// After 48 hours: increase to 25%
// After 72 hours: promote to staging
// After 96 hours: promote to production
```

### Scenario: Quick Canary Rollback
```rust
let health = manager.monitor_canary_health();
if health.canary_error_rate > health.production_error_rate * 2.0 {
    println!("Recommendation: {}", health.recommendation);
    // → "ROLLBACK: Canary error rate significantly higher"
}
// Automatic rollback triggered if auto_rollback_enabled
```

### Scenario: Feature A/B Testing
```rust
// Feature flag enables A/B test
manager.enable_feature("new_checkout_flow".to_string());

// Canary users get new flow
let decision = manager.route_request(user_id);
if decision.is_canary && decision.enabled_features.contains(&"new_checkout_flow") {
    use_new_checkout_flow();
} else {
    use_old_checkout_flow();
}

// If new flow has issues:
manager.disable_feature("new_checkout_flow".to_string());
// → All users immediately revert to old flow
```

## Testing Your Integration

```bash
# Run all promotion gates tests
cargo test --package knhk-sidecar --test promotion_gates_test

# Run example
cargo run --example promotion_gates_example

# Expected output shows:
# - Deterministic routing verified
# - Traffic distribution correct
# - Health monitoring working
# - Promotion workflow functional
```

## Troubleshooting

### "Promotion failed: SLO compliance below threshold"
- **Cause**: New version has actual issues or SLO too strict
- **Solution**: Fix bugs, wait for metrics, or lower threshold temporarily

### "Canary keeps rolling back"
- **Cause**: New version has real bugs
- **Solution**: Fix the bugs, don't just adjust thresholds

### "All requests go to production even with high traffic_percent"
- **Cause**: Config not updated or manager created with wrong config
- **Solution**: Verify config.environment is Canary with correct traffic_percent

### "Health score is 0.0"
- **Cause**: Canary has much higher error rate than production
- **Solution**: Investigate canary performance, may need to rollback

## Best Practices

1. **Start Small**
   - Begin with 1-5% canary traffic
   - Give each tier 24 hours to stabilize

2. **Monitor Continuously**
   - Watch canary error rates every 5 minutes
   - Check health score before each promotion
   - Review rollback history regularly

3. **Gradual Ramps**
   - 1% → 5% → 10% → 25% → 50% → 100%
   - Don't jump by more than 2-5x traffic

4. **Feature Flags**
   - Use for quick disable if issues found
   - Combine with percentage-based canary
   - Test different features independently

5. **SLO Settings**
   - Start conservative (0.95 = 95% must succeed)
   - Canary usually more lenient than production
   - Increase strictness as confidence grows

## Quick Reference Card

```
ROUTING:      hash(request_id) % 100 < traffic_percent
DETERMINISTIC: Same request_id always routes same version
AUTO-ROLLBACK: On SLO violation or health < 0.8
ENVIRONMENTS: Canary (%) → Staging (100%) → Production
HEALTH:       0.9-1.0 green, 0.8-0.9 yellow, <0.8 red
METRICS:      error_rate, p99_latency, health_score
TIMELINE:     ~5 days from canary to production
PROGRESSION:  5% → 10% → 25% → Staging → Production
```

## Related Documentation

- Full Docs: `docs/PROMOTION_GATES.md`
- Configuration: `examples/promotion_config.toml`
- Example Code: `examples/promotion_gates_example.rs`
- Implementation: `src/promotion.rs`
- Tests: `tests/promotion_gates_test.rs`

## Support

For detailed information, see:
- **Architecture**: `docs/PROMOTION_GATES.md`
- **Configuration**: `examples/promotion_config.toml`
- **Troubleshooting**: `docs/PROMOTION_GATES.md#troubleshooting`
- **Examples**: `examples/promotion_gates_example.rs`
