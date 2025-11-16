# Phase 4: Promotion Gates - Production-Ready Canary Deployment

This document describes the Promotion Gates system for KNHK Fortune 500, enabling safe, deterministic canary deployments with automatic rollback.

## Overview

Promotion Gates manages the formal promotion workflow: **Canary → Staging → Production** with:

- **Deterministic Canary Routing**: Same request IDs always route to the same version
- **Automatic Rollback**: SLO violations trigger immediate rollback to stable version
- **Multi-Environment Support**: Canary (%), Staging, and Production environments
- **Feature Flags**: Per-environment feature enablement
- **Health Monitoring**: Continuous canary health scoring

## Core Components

### 1. Deterministic Canary Routing

The routing system uses SHA256-based hashing for deterministic traffic splitting:

```rust
// Same request_id always routes to same version (deterministic)
let decision = manager.route_request("user-12345-session");

// Canary gets request_id hash % 100 < traffic_percent
// Example: traffic_percent=20%, request hash=42%
// → Routes to production (42 >= 20)
// Example: traffic_percent=20%, request hash=15%
// → Routes to canary (15 < 20)
```

**Key Properties:**
- Deterministic: Same request_id always produces same routing decision
- Stateless: No shared state required, scales horizontally
- Predictable: Enables A/B testing with consistent user experiences
- Safe: New version failures don't affect stable version users

### 2. Traffic Splitting Algorithm

```
Canary Traffic = floor(hash(request_id) % 100) < traffic_percent
```

For traffic_percent=25:
- Requests with hash % 100 in [0, 25) → Canary
- Requests with hash % 100 in [25, 100) → Production
- Results in ~25% to canary, ~75% to production

### 3. SLO-Based Automatic Rollback

Monitors two metrics during canary:

1. **Error Rate** - Percentage of failed requests
   - Threshold: configurable (default 5%)
   - Triggers: If canary error rate > threshold

2. **P99 Latency** - 99th percentile latency
   - Threshold: configurable per runtime class
   - Triggers: If canary p99 > production p99 * 1.5

When either condition fails:
1. Log rollback event with reason
2. Disable all feature flags
3. Reset to Production environment
4. Notify operations team

### 4. Health Scoring

Canary health is computed as:

```
health_score = 1.0
  - (canary_error_rate > prod_rate * 1.1) ? (delta) : 0  // Up to -0.5
  - (canary_p99 > prod_p99) ? (latency_ratio - 1.0) * 0.2 : 0  // Up to -0.3
```

Recommendations:
- Score ≥ 0.9: **HEALTHY** - Ready to promote
- Score 0.8-0.9: **MONITOR** - Watch before promotion
- Score < 0.8: **ROLLBACK** - Automatic rollback triggered

## Configuration

### PromotionConfig

```rust
pub struct PromotionConfig {
    /// Deployment environment
    pub environment: Environment,

    /// Feature flags for this deployment
    pub feature_flags: Vec<String>,

    /// Enable automatic rollback on SLO violations
    pub auto_rollback_enabled: bool,

    /// SLO compliance threshold (0.0-1.0, default 0.95)
    pub slo_threshold: f64,

    /// Rollback monitoring window in seconds (default 300)
    pub rollback_window_seconds: u64,
}
```

### Environment

```rust
pub enum Environment {
    /// Canary: traffic_percent % of requests to new version
    Canary { traffic_percent: f64 },

    /// Staging: all traffic to new features (internal testing)
    Staging,

    /// Production: all traffic to stable version
    Production,
}
```

## Usage Examples

### Example 1: Canary with 10% Traffic

```rust
use knhk_sidecar::promotion::{Environment, PromotionConfig, PromotionGateManager};

// Create config
let config = PromotionConfig {
    environment: Environment::Canary { traffic_percent: 10.0 },
    feature_flags: vec!["new_api".to_string()],
    auto_rollback_enabled: true,
    slo_threshold: 0.95,
    rollback_window_seconds: 300,
};

// Create manager
let manager = PromotionGateManager::new(config, slo_controller)?;

// Route request
let decision = manager.route_request("user-session-123");
if decision.is_canary {
    // Use new version with enabled features
    use_version(Version::New, &decision.enabled_features);
} else {
    // Use stable version
    use_version(Version::Stable, &vec![]);
}
```

### Example 2: Feature Flag Control

```rust
let mut manager = PromotionGateManager::new(config, slo_controller)?;

// Enable new feature
manager.enable_feature("experimental_cache".to_string());

// Check if feature is enabled
if manager.is_feature_enabled("experimental_cache") {
    use_experimental_cache();
}

// Disable if issues detected
manager.disable_feature("experimental_cache".to_string());
```

### Example 3: Promotion Workflow

```rust
let mut manager = PromotionGateManager::new(
    PromotionConfig {
        environment: Environment::Canary { traffic_percent: 5.0 },
        ..
    },
    slo_controller,
)?;

// Monitor canary health
let health = manager.monitor_canary_health();
println!("Canary error rate: {}", health.canary_error_rate);
println!("Health score: {}", health.health_score);
println!("Recommendation: {}", health.recommendation);

// When ready, promote to staging
if health.health_score >= 0.9 {
    manager.promote(Environment::Staging)?;
}

// Later, promote from staging to production
if let Ok(staging_health) = manager.check_slo_compliance() {
    if staging_health {
        manager.promote(Environment::Production)?;
    }
}
```

## Routing Decision Details

### RoutingDecision

```rust
pub struct RoutingDecision {
    /// Target environment (Canary, Staging, or Production)
    pub target_environment: Environment,

    /// Whether this request is canary traffic
    pub is_canary: bool,

    /// Feature flags enabled for this request
    pub enabled_features: Vec<String>,

    /// Reason for routing decision (with hash and threshold info)
    pub reason: String,
}
```

### Feature Flag Behavior

| Environment | Feature Flags | User Impact |
|-------------|---------------|-------------|
| Canary (✓ to new) | All enabled | Gets new features |
| Canary (✗ to stable) | Empty | Uses stable features |
| Staging | All enabled | For internal testing |
| Production | Stable only | No experimental features |

## Automatic Rollback Conditions

Rollback is triggered when:

1. **SLO Violation**: Compliance rate < threshold
2. **Canary Error Rate**: Exceeds production error rate significantly
3. **Latency Degradation**: P99 latency exceeds acceptable range
4. **Feature Disabled**: Manual disable triggers rollback recording
5. **Health Score Low**: Score < 0.8 when auto_rollback_enabled

### Rollback Actions

When rollback is triggered:

```
1. Log rollback event with reason + timestamp
2. Disable all feature flags
3. Reset environment to Production
4. Reset canary metrics
5. Record in rollback history
6. Alert operations (via logging/tracing)
```

### Rollback History

```rust
pub struct RollbackEvent {
    pub feature: String,          // "Canary{50.0}" or feature name
    pub reason: String,            // Why rollback occurred
    pub timestamp: SystemTime,     // When it happened
    pub environment: Environment,  // Previous environment
}

let history = manager.get_rollback_history();
for event in history {
    println!("Rollback: {} - {} at {:?}",
             event.feature, event.reason, event.timestamp);
}
```

## Monitoring and Observability

### Canary Health Metrics

```rust
pub struct CanaryHealth {
    pub traffic_percent: f64,
    pub canary_requests: u64,
    pub production_requests: u64,
    pub canary_error_rate: f64,
    pub production_error_rate: f64,
    pub canary_p99_latency: Duration,
    pub production_p99_latency: Duration,
    pub health_score: f64,
    pub recommendation: String,
}
```

### Logging

Promotion Gates emits structured logs:

```rust
// Debug: routing decision (per-request, disabled in production)
debug!("Request {} routed to canary (hash {}%)", request_id, percent);

// Info: feature flag changes
info!("Feature flag enabled: {}", feature_name);

// Warn: approaching rollback window
warn!("Canary health score {} below threshold", health_score);

// Error: rollback triggered
error!("Triggering automatic rollback: {}", reason);

// Error: SLO violations
error!("Canary SLO compliance {} below threshold {}", compliance, threshold);
```

### Telemetry Integration

Promotion Gates integrates with OpenTelemetry:

```rust
// Spans for routing decisions
tracer.start_span("promotion.route_request")
    .with_attribute("promotion.request_id", request_id)
    .with_attribute("promotion.is_canary", is_canary)
    .with_attribute("promotion.environment", environment.name());

// Spans for rollback events
tracer.start_span("promotion.rollback")
    .with_attribute("promotion.reason", rollback_reason)
    .with_attribute("promotion.previous_env", env_name);

// Metrics for SLO compliance
tracer.record_gauge("promotion.canary_error_rate", error_rate);
tracer.record_gauge("promotion.health_score", health_score);
```

## Error Handling

### Validation Errors

Configuration validation catches:

- Invalid traffic_percent (must be 0-100)
- Invalid slo_threshold (must be 0-1)
- Invalid promotion paths
- Missing required fields

Example:
```rust
match PromotionGateManager::new(config, slo_controller) {
    Ok(manager) => { /* Valid configuration */ }
    Err(e) => {
        eprintln!("Invalid promotion config: {}", e);
        // Return error to caller, don't unwrap
    }
}
```

### Runtime Errors

Production code uses Result types, never panics:

```rust
// ✓ Correct: Handle errors explicitly
match manager.check_slo_compliance() {
    Ok(compliant) => {
        if !compliant {
            warn!("SLO violations detected");
        }
    }
    Err(e) => {
        error!("SLO check failed: {}", e);
        // Safe fallback: stay in current environment
    }
}

// ✗ Wrong: Never use unwrap/expect
// let result = manager.check_slo_compliance().unwrap(); // PANIC!
```

## Testing

### Unit Tests

Comprehensive test coverage includes:

- **Deterministic Routing**: Same request_id always produces same decision
- **Traffic Splitting**: Correct percentage split across large sample
- **Edge Cases**: 0%, 100%, boundary conditions
- **Promotion Logic**: Valid/invalid transitions
- **Rollback Triggering**: SLO violation detection
- **Feature Flags**: Enable/disable behavior
- **Configuration Validation**: Invalid inputs rejected
- **Error Handling**: No panics, proper error returns

Run tests:
```bash
cargo test --package knhk-sidecar --test promotion_gates_test
```

### Integration Tests

Test with real SLO admission controller:

```rust
#[test]
fn test_canary_with_real_slo_controller() {
    let slo_config = SloConfig::default();
    let slo_controller = SloAdmissionController::new(slo_config)?;
    let config = create_canary_config(20.0);
    let manager = PromotionGateManager::new(config, slo_controller)?;

    // Record request outcomes
    manager.record_request_outcome("req-1", true, Duration::from_millis(5));

    // Check compliance
    let compliant = manager.check_slo_compliance()?;
    assert!(compliant);
}
```

## Best Practices

### 1. Start Small with Canary

```
Day 1: Canary 1%  (0.01-0.1% of users)
Day 2: Canary 5%  (0.05-0.5% of users)
Day 3: Canary 10% (0.1-1% of users)
Day 4: Staging 100% (internal validation)
Day 5: Production (full rollout)
```

### 2. Monitor Continuously

Always run:
- `monitor_canary_health()` every 5 minutes
- `check_slo_compliance()` before each promotion
- Track `get_rollback_history()` for patterns

### 3. Gradual Traffic Increase

Don't jump from 10% → 100%. Use steps:
- 10% → 25% → 50% → 75% → 100%

### 4. Feature Flag Gates

Use feature flags for:
- A/B testing variants
- Gradual rollout of features
- Quick disable if issues found
- Per-environment feature control

### 5. Rollback Planning

Have automated rollback ready:
- One-click rollback dashboard
- Alert on SLO violations
- Auto-disable problematic features
- Communicate rollback to teams

## Performance Characteristics

### Routing Decision (O(1))

```
Request → Hash (SHA256) → Modulo 100 → Compare to traffic_percent → Route
Time: ~1-2 microseconds per request
```

### SLO Compliance Check

```
Check metrics → Calculate rates → Compare thresholds → Record decision
Time: ~100-500 microseconds
Memory: ~1KB for metrics
```

### Health Monitoring

```
Calculate: error_rates, latency_percentiles, health_score, recommendation
Time: ~5-10 milliseconds (with large history)
Memory: ~100KB for 1000 request history
```

### Rollback Execution

```
Disable flags → Reset env → Clear metrics → Log event
Time: ~1 millisecond
Memory: 0 (reference counted)
```

## Migration Guide

### From Manual Promotion

**Before:**
```bash
# Manual promotion with risk of human error
# 1. Deploy new version
# 2. Manually monitor metrics
# 3. Manually decide when to promote
# 4. Manually decide when to rollback
```

**After:**
```rust
// Automated, deterministic promotion
manager.route_request(request_id); // Deterministic routing
manager.monitor_canary_health();   // Automatic monitoring
manager.promote(target_env);       // Gated promotion
// Automatic rollback on SLO violation
```

### From Percentage-Based Routing

**Before:**
```rust
// Non-deterministic: same user gets different versions
if random() < 0.1 {
    use_new_version();
} else {
    use_stable_version();
}
```

**After:**
```rust
// Deterministic: same user always gets same version
let decision = manager.route_request(user_id);
if decision.is_canary {
    use_new_version();
}
```

## Troubleshooting

### Rollback Keeps Triggering

**Symptom:** Canary keeps rolling back on high error rate

**Causes:**
1. New version has actual bugs
2. SLO threshold too strict
3. Canary traffic hitting different code paths

**Solutions:**
1. Fix bugs in new version
2. Adjust `slo_threshold` (e.g., 0.90 instead of 0.95)
3. Increase canary percentage to hit same paths as production
4. Review traffic distribution algorithm

### Canary Gets No Traffic

**Symptom:** All requests route to production even with high traffic_percent

**Causes:**
1. Configuration not updated
2. Manager using wrong config
3. Hash distribution issue

**Solutions:**
1. Verify `config.environment = Canary { traffic_percent: X }`
2. Check manager was created with correct config
3. Log hash distribution and verify it's uniform

### Promotion Blocked

**Symptom:** `promote()` returns error "SLO compliance below threshold"

**Causes:**
1. Insufficient traffic to establish metrics
2. Canary actually has higher error rate
3. Latency worse than production

**Solutions:**
1. Wait for more requests (need 100+ for stable metrics)
2. Fix actual performance issues in canary
3. Gradually increase canary traffic instead of promoting immediately

## See Also

- [SLO Admission Control](../slo_admission.rs) - Detailed SLO implementation
- [Error Handling](../error.rs) - Error types and context
- [Health Monitoring](../health.rs) - System health tracking
- [Metrics Collection](../metrics.rs) - Metrics implementation
