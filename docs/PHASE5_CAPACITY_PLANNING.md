# Phase 5: Capacity Planning with SLO Models for KNHK Fortune 500

## Overview

Phase 5 implements comprehensive capacity planning and SLO-based admission control for Fortune 500 systems. The implementation uses OpenTelemetry-based metrics collection combined with predictive models to ensure mission-critical performance requirements.

## SLO Classes

Three SLO classes define performance requirements for different workload types:

### R1 (Red Line) - Hot Path
- **Target Latency**: ≤8 ticks (≤2 nanoseconds)
- **Required Hit Rate**: 99%+
- **Cache Strategy**: L1 pinning, HSM/ultra-fast storage
- **Use Case**: Microsecond-level operations (CPU instructions, L1 cache fetches)
- **Cost Tier**: Premium ($3/GB/day for L1)

### W1 (Warm Line) - Standard Operations
- **Target Latency**: ≤500 milliseconds
- **Required Hit Rate**: 95%+
- **Cache Strategy**: In-memory cache, fast SSD
- **Use Case**: Normal web services, API responses, standard queries
- **Cost Tier**: Standard ($0.10/GB/day for L2)

### C1 (Cold Line) - Background Operations
- **Target Latency**: ≤24 hours
- **Required Hit Rate**: 80%+
- **Cache Strategy**: Persistent storage, disk acceptable
- **Use Case**: Batch processing, analytics, archival
- **Cost Tier**: Economy ($0.02/GB/day for L3)

## Core Components

### 1. SloClass Enum

Defines SLO requirements with associated methods:

```rust
pub enum SloClass {
    R1,  // Hot path: ≤8 ticks (≤2ns)
    W1,  // Warm path: ≤500ms
    C1,  // Cold path: ≤24h
}

impl SloClass {
    pub fn target_latency(&self) -> Duration
    pub fn required_hit_rate(&self) -> f64  // 0.99, 0.95, 0.80
    pub fn name(&self) -> &'static str
}
```

### 2. SloPrediction Struct

Predicts resource requirements for achieving SLO targets:

```rust
pub struct SloPrediction {
    pub slo_class: SloClass,
    pub required_l1_size: usize,        // L1 cache bytes
    pub required_l2_size: usize,        // L2 cache bytes
    pub expected_hit_rate: f64,         // 0.0-1.0
    pub cost_estimate_daily: f64,       // $$$ per day
    pub confidence: f64,                // 0.0-1.0
    pub working_set_items: usize,
    pub l1_items: usize,
    pub l2_items: usize,
}

impl SloPrediction {
    pub fn meets_slo(&self) -> bool
    pub fn storage_tier(&self) -> &'static str
}
```

### 3. HeatItem Struct

Represents data item access patterns for Pareto analysis:

```rust
pub struct HeatItem {
    pub key: String,
    pub access_count: u64,
    pub last_access: SystemTime,
    pub size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
}

impl HeatItem {
    pub fn hit_rate(&self) -> f64
}
```

### 4. GrowthProjection Struct

Projects future capacity needs:

```rust
pub struct GrowthProjection {
    pub current_size: usize,
    pub projected_size_30d: usize,      // 30-day forecast
    pub projected_size_90d: usize,      // 90-day forecast
    pub growth_rate: f64,               // Items per day
    pub urgency: CapacityUrgency,
}

pub enum CapacityUrgency {
    Ok,         // Plenty of capacity
    Warning,    // Plan expansion soon
    Critical,   // Immediate expansion needed
}
```

### 5. CostModel Struct

Estimates operating costs for different cache tiers:

```rust
pub struct CostModel {
    pub l1_cost_per_gb_daily: f64,   // Premium: $3.0/GB/day
    pub l2_cost_per_gb_daily: f64,   // Standard: $0.10/GB/day
    pub l3_cost_per_gb_daily: f64,   // Economy: $0.02/GB/day
}

impl CostModel {
    pub fn estimate_daily_cost(&self, l1_gb: f64, l2_gb: f64) -> f64
}
```

## CapacityManager Methods

### 1. analyze_heat_map()

Identifies hot data using Pareto principle (80/20).

**Algorithm**:
1. Collect all predicates with their access frequencies
2. Sort by access count (descending)
3. Return sorted list for working set identification

**Returns**: `Vec<HeatItem>` sorted by access frequency

**Use Case**: Identify which data should be pinned to L1/L2 caches

```rust
pub fn analyze_heat_map(&mut self) -> Vec<HeatItem>
```

### 2. predict_capacity_needed()

Projects cache sizes required for target SLO hit rate.

**Algorithm**:
1. Analyze heat map from Pareto analysis
2. Calculate cumulative access percentages
3. Allocate L1 cache to top 20% of items
4. Allocate L2 cache to next 20% of items
5. Estimate expected hit rate with allocation
6. Calculate cost based on cache sizes
7. Return confidence based on data quality

**Returns**: `SloPrediction` with:
- L1/L2 size requirements
- Expected hit rate
- Daily cost estimate
- Confidence level (0.5-0.95)

```rust
pub fn predict_capacity_needed(&self, slo_class: SloClass) -> SloPrediction
```

**Example**:
```
R1 (99% hit rate required):
  Working set: 1000 items
  L1 allocation: 200 items (top 20%), 64KB
  L2 allocation: 200 items (next 20%), 1MB
  Expected hit rate: 98.5%
  Daily cost: $3.06
  Confidence: 0.85
```

### 3. should_admit_request()

SLO-based admission control for new requests.

**Algorithm**:
1. Get current system hit rate
2. Check against required rate for SLO class
3. Admit if current >= required
4. Reject if current < required (backpressure)
5. Track admission distribution by SLO class

**Returns**: `bool` - true if request should be admitted

```rust
pub fn should_admit_request(&mut self, slo_class: SloClass) -> bool
```

**Decision Logic**:
- R1: Current hit rate >= 99%
- W1: Current hit rate >= 95%
- C1: Current hit rate >= 80%

### 4. scale_recommendation()

Projects future capacity and provides scaling guidance.

**Algorithm**:
1. Calculate growth rate from historical measurements
2. Project sizes for 30 and 90 day horizons
3. Determine urgency level:
   - Critical: Growth > 50% per month
   - Warning: Growth > 20% per month
   - Ok: Growth < 20% per month
4. Recommend capacity multipliers (1.1x to 1.5x projected)

**Returns**: `ScaleRecommendation` with:
- Current capacity
- Recommended capacity
- Urgency level
- Growth projection details

```rust
pub fn scale_recommendation(&self) -> ScaleRecommendation
```

### 5. record_growth_point()

Records current capacity for trend analysis.

**Usage**: Call daily to build growth history

```rust
pub fn record_growth_point(&mut self)
```

### 6. get_slo_distribution()

Returns admission statistics by SLO class.

```rust
pub fn get_slo_distribution(&self) -> HashMap<SloClass, u64>
```

## Pareto Principle Implementation

The heat map analysis implements the Pareto principle (80/20 rule):

**Key Insight**: Typically 20% of data items generate 80% of traffic

**Working Set Calculation**:
1. Sort all predicates by access frequency (descending)
2. Cumulatively sum access counts
3. Identify items representing target hit rate
4. Calculate L1 size from top 20% of items
5. Calculate L2 size from next 20% of items
6. Remaining items on disk (L3)

**Benefits**:
- Minimal L1 cache needed (64KB-256KB typically)
- Efficient L2 cache sizing (1MB-1GB typically)
- Cold data doesn't need expensive storage

## Capacity Planning Workflow

### Daily Operations

```rust
// 1. Record cache accesses
manager.record_access("predicate_name", hit, l1_hit);

// 2. Admission control decisions
let admit = manager.should_admit_request(SloClass::W1);
if !admit {
    return Err("Backpressure: capacity limit reached");
}
```

### Weekly Analysis

```rust
// Analyze current heat map
let heat_map = manager.analyze_heat_map();
for item in heat_map.iter().take(10) {
    println!("Hot: {} ({}% hit rate)", item.key, item.hit_rate() * 100.0);
}

// Predict capacity for next quarter
let r1_pred = manager.predict_capacity_needed(SloClass::R1);
let w1_pred = manager.predict_capacity_needed(SloClass::W1);

if !r1_pred.meets_slo() {
    warn!("R1 SLO not achievable - need {} bytes L1", r1_pred.required_l1_size);
}
```

### Monthly Planning

```rust
// Record growth point (typically daily)
manager.record_growth_point();

// Generate scaling recommendation
let recommendation = manager.scale_recommendation();

match recommendation.urgency {
    CapacityUrgency::Critical => {
        // Purchase capacity: recommendation.recommended_capacity
        println!("URGENT: Scale to {} items", recommendation.recommended_capacity);
    }
    CapacityUrgency::Warning => {
        // Plan scaling in next quarter
        println!("Plan scaling to {} items", recommendation.recommended_capacity);
    }
    CapacityUrgency::Ok => {
        // Continue monitoring
    }
}
```

## Cost Model

Default costs (Fortune 500 pricing, 2024):

```
L1 (HSM/Ultra-fast NVMe):     $3.00/GB/day   = $36-$45/GB/year
L2 (In-memory/SSD):            $0.10/GB/day   = $1.20-$3.00/GB/year
L3 (Persistent storage/Disk):  $0.02/GB/day   = $0.24-$0.60/GB/year
```

**Example Cost Calculation**:
- 100MB L1 + 10GB L2 = (0.1 * 3.0) + (10 * 0.10) = $1.30/day = $474/year

## Confidence Levels

Prediction confidence based on data quality:

```
< 10 predicates:      0.50 (Low - insufficient data)
10-100 predicates:    0.70 (Medium - some patterns visible)
100-1000 predicates:  0.85 (High - good statistical sample)
> 1000 predicates:    0.95 (Very High - excellent coverage)
```

## Error Handling

All operations use proper Rust error handling:

- No `unwrap()` or `expect()` in production code paths
- Safe division by zero checks
- Graceful handling of empty data sets
- Comprehensive logging with structured tracing

## Integration with KNHK

The capacity manager integrates with:

1. **Metrics Collection**: Reads cache hit/miss from MetricsCollector
2. **Beat Admission**: Feeds SLO requirements to BeatAdmission
3. **SLO Controller**: Provides capacity predictions for SloAdmissionController
4. **Promotion Gates**: Informs auto-rollback decisions based on capacity

## Testing

Comprehensive test suite in `tests/capacity_phase5_tests.rs`:

- SLO class definitions
- SloPrediction validation
- Heat map analysis (Pareto principle)
- Capacity predictions for R1/W1/C1
- Admission control decisions
- Growth projections
- Cost model calculations
- Confidence level validation
- Edge cases and boundary conditions

## Performance Characteristics

- Heat map analysis: O(n log n) where n = number of predicates
- Capacity prediction: O(n) single pass through heat items
- Admission control: O(1) constant time
- Scale recommendation: O(n) with small history buffer

## Fortune 500 Requirements Met

✓ SLO-based admission control (R1/W1/C1)
✓ Pareto principle working set identification
✓ Cost modeling and daily cost estimation
✓ Growth projection (30/90 day forecasts)
✓ Capacity urgency assessment
✓ Production-grade error handling
✓ Comprehensive logging
✓ No unsafe constructs
✓ Comprehensive test coverage

## Related Files

- **Implementation**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`
- **Tests**: `/home/user/knhk/rust/knhk-sidecar/tests/capacity_phase5_tests.rs`
- **Integration**: `/home/user/knhk/rust/knhk-sidecar/src/lib.rs` (capacity manager initialization)
