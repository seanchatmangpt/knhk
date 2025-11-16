# Phase 5: Quick Reference Guide

## Core Structures Location Map

### SLO Classes
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`

```
Lines 9-65:    SloClass enum definition
  - R1:        Hot path (≤2ns, 99% hit rate)
  - W1:        Warm path (≤500ms, 95% hit rate)
  - C1:        Cold path (≤24h, 80% hit rate)
```

### SLO Prediction
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`

```
Lines 67-112:  SloPrediction struct
  - required_l1_size:      L1 cache bytes needed
  - required_l2_size:      L2 cache bytes needed
  - expected_hit_rate:     Cache hit rate 0.0-1.0
  - cost_estimate_daily:   $ per day
  - confidence:            Prediction confidence 0.0-1.0
```

### Heat Analysis
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`

```
Lines 114-137: HeatItem struct for Pareto analysis
  - key:           Item identifier
  - access_count:  Total accesses
  - hit_count:     Number of hits
  - miss_count:    Number of misses
  - hit_rate():    Calculate hit rate
```

### Cost Model
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`

```
Lines 157-180: CostModel struct
  - l1_cost_per_gb_daily:  $3.0/GB/day (Premium HSM)
  - l2_cost_per_gb_daily:  $0.10/GB/day (Standard SSD)
  - l3_cost_per_gb_daily:  $0.02/GB/day (Economy Disk)
  - estimate_daily_cost(): Calculate total cost
```

### Growth Projection
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`

```
Lines 139-147: GrowthProjection struct
  - current_size:        Current working set size
  - projected_size_30d:  30-day forecast
  - projected_size_90d:  90-day forecast
  - growth_rate:         Items per day
  - urgency:             Ok/Warning/Critical
```

## Core Methods Location Map

### Heat Map Analysis (Pareto)
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs` **Lines 674-702**

```rust
pub fn analyze_heat_map(&mut self) -> Vec<HeatItem>
```

- Sorts by access frequency (Pareto principle)
- Returns top items generating 80% of traffic
- Time complexity: O(n log n)

### Capacity Prediction
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs` **Lines 704-811**

```rust
pub fn predict_capacity_needed(&self, slo_class: SloClass) -> SloPrediction
```

- Analyzes heat map for target SLO class
- Projects L1/L2 cache requirements
- Estimates daily cost
- Returns confidence level
- Time complexity: O(n)

### Admission Control
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs` **Lines 813-845**

```rust
pub fn should_admit_request(&mut self, slo_class: SloClass) -> bool
```

- Decision: Current hit rate >= required rate
- R1: >= 99%, W1: >= 95%, C1: >= 80%
- Tracks admission by SLO class
- Time complexity: O(1)

### Scale Recommendation
**File**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs` **Lines 847-923**

```rust
pub fn scale_recommendation(&self) -> ScaleRecommendation
pub fn record_growth_point(&mut self)
```

- 30-day and 90-day projections
- Growth rate calculation
- Urgency assessment (Ok/Warning/Critical)
- Capacity multiplier recommendations
- Time complexity: O(n) with bounded history

## Test Suite Location Map

**File**: `/home/user/knhk/rust/knhk-sidecar/tests/capacity_phase5_tests.rs`

### SLO Class Tests (5 tests)
- `test_slo_class_definitions` - Verify class properties
- `test_slo_class_default` - Default to W1
- `test_slo_prediction_creation` - Create predictions
- `test_slo_prediction_validation` - Validate SLO compliance
- `test_slo_prediction_storage_tier` - Storage tier selection

### Heat Map Tests (3 tests)
- `test_heat_map_analysis_empty` - Handle empty data
- `test_heat_map_analysis_with_data` - Sort by frequency
- `test_pareto_principle_working_set` - Verify 80/20 distribution

### Capacity Prediction Tests (3 tests)
- `test_slo_capacity_prediction_r1` - R1 prediction
- `test_slo_capacity_prediction_w1` - W1 prediction
- `test_slo_capacity_prediction_c1` - C1 prediction

### Admission Control Tests (3 tests)
- `test_should_admit_request_high_hit_rate` - 99%+ rate
- `test_should_admit_request_medium_hit_rate` - 92% rate
- `test_should_admit_request_low_hit_rate` - 50% rate

### Growth Projection Tests (4 tests)
- `test_scale_recommendation_no_growth` - Zero growth
- `test_scale_recommendation_ok` - Normal growth
- `test_record_growth_point` - Record snapshots
- `test_capacity_urgency_critical` - Critical scaling

### Cost Model Tests (3 tests)
- `test_cost_model_default` - Default pricing
- `test_cost_model_estimation` - Calculate costs
- `test_cost_estimation_scenarios` - Multiple scenarios

### Edge Case Tests (6+ tests)
- `test_heat_item_zero_hits` - Empty heat items
- `test_empty_capacity_manager_edge_cases` - No data
- `test_slo_prediction_meets_slo_boundary` - Boundary conditions
- Plus: Confidence, distribution, cloning, etc.

## Usage Examples

### Example 1: Basic Admission Control

```rust
let mut capacity = CapacityManager::new(0.95);

// Record access patterns
capacity.record_access("query_1", true, true);  // L1 hit
capacity.record_access("query_2", true, false); // L2 hit

// Decide on admission
match capacity.should_admit_request(SloClass::W1) {
    true => {
        // Process request
    }
    false => {
        // Apply backpressure
    }
}
```

### Example 2: Weekly Capacity Analysis

```rust
let mut capacity = CapacityManager::new(0.95);

// ... record accesses during week ...

// Analyze heat map
let hot_items = capacity.analyze_heat_map();
println!("Top 10 hot items:");
for item in hot_items.iter().take(10) {
    println!("  {} - {:.1}% hit rate", item.key, item.hit_rate() * 100.0);
}

// Predict capacity needs
let r1_need = capacity.predict_capacity_needed(SloClass::R1);
let w1_need = capacity.predict_capacity_needed(SloClass::W1);

println!("R1 needs: L1={}KB, L2={}MB, cost=${}/day",
         r1_need.required_l1_size / 1024,
         r1_need.required_l2_size / (1024 * 1024),
         r1_need.cost_estimate_daily);
```

### Example 3: Monthly Scaling Decision

```rust
let mut capacity = CapacityManager::new(0.95);

// ... record accesses during month, daily snapshots ...

// Get scaling recommendation
let rec = capacity.scale_recommendation();

match rec.urgency {
    CapacityUrgency::Critical => {
        println!("URGENT: Scale to {} items", rec.recommended_capacity);
        // Purchase capacity immediately
    }
    CapacityUrgency::Warning => {
        println!("Plan scaling to {} items", rec.recommended_capacity);
        // Schedule expansion within month
    }
    CapacityUrgency::Ok => {
        println!("Growth: {:.1} items/day", rec.growth_projection.growth_rate);
        // Continue monitoring
    }
}
```

### Example 4: Custom Cost Model

```rust
let mut capacity = CapacityManager::new(0.95);

// Set custom pricing for your infrastructure
let custom_cost = CostModel {
    l1_cost_per_gb_daily: 2.0,    // Custom L1 cost
    l2_cost_per_gb_daily: 0.08,   // Custom L2 cost
    l3_cost_per_gb_daily: 0.01,   // Custom L3 cost
};

capacity.set_cost_model(custom_cost);

// Now predictions use your costs
let pred = capacity.predict_capacity_needed(SloClass::W1);
println!("Daily cost: ${:.2}", pred.cost_estimate_daily);
```

## Decision Tree

### When to Use Each SLO Class

```
User Request
  |
  +-- Microsecond latency required? --> YES --> R1 (≤2ns)
  |       (99% hit rate, HSM)
  |
  +-- NO, millisecond latency OK? --> YES --> W1 (≤500ms)
  |       (95% hit rate, SSD)
  |
  +-- NO, hours/days OK? --> YES --> C1 (≤24h)
         (80% hit rate, Disk)
```

### When to Admit/Reject Request

```
Request arrives (SLO class specified)
  |
  +-- Calculate current hit rate from metrics
  |
  +-- Compare: current >= required?
  |       |
  |       +-- YES --> ADMIT (process request)
  |       |
  |       +-- NO --> REJECT (apply backpressure)
  |
  +-- Update SLO distribution statistics
```

### When to Scale Capacity

```
Daily/Weekly: Record growth point with record_growth_point()
Monthly:      Call scale_recommendation()
  |
  +-- Check urgency level
  |       |
  |       +-- CRITICAL (>50% growth/month)
  |       |       --> Immediate purchase
  |       |
  |       +-- WARNING (20-50% growth/month)
  |       |       --> Plan expansion within 30 days
  |       |
  |       +-- OK (<20% growth/month)
  |               --> Continue monitoring
```

## Configuration

### CapacityManager Creation
```rust
// Standard setup with 95% cache hit threshold
let manager = CapacityManager::new(0.95);
```

### Default Cost Model
```
L1 (Premium):    $3.00/GB/day  = $36.00-$45.00/GB/year
L2 (Standard):   $0.10/GB/day  = $1.20-$3.00/GB/year
L3 (Economy):    $0.02/GB/day  = $0.24-$0.60/GB/year
```

## Performance Characteristics

| Operation | Time | Space |
|-----------|------|-------|
| record_access() | O(1) | O(1) |
| analyze_heat_map() | O(n log n) | O(n) |
| predict_capacity_needed() | O(n) | O(1) |
| should_admit_request() | O(1) | O(1) |
| scale_recommendation() | O(log h) | O(1) |
| record_growth_point() | O(1) | O(h) |

Where: n = number of predicates, h = growth history size (max 100)

## See Also

- **Full Documentation**: `/home/user/knhk/docs/PHASE5_CAPACITY_PLANNING.md`
- **Implementation Summary**: `/home/user/knhk/docs/PHASE5_IMPLEMENTATION_SUMMARY.md`
- **Source Code**: `/home/user/knhk/rust/knhk-sidecar/src/capacity.rs`
- **Test Suite**: `/home/user/knhk/rust/knhk-sidecar/tests/capacity_phase5_tests.rs`
