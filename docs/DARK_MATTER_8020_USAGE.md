# Dark Matter 80/20 Coverage Tracker - Usage Guide

## Overview

The Dark Matter 80/20 Coverage Tracker implements real-time analysis of predicate/hook access patterns to identify the critical 20% of operations that handle 80% of queries. This enables focused optimization of the hot path.

## Architecture

**Key Principles**:
- **Lock-free atomic counters** for zero-contention hot path tracking
- **Efficient hash-based indexing** for O(1) predicate recording
- **Pareto distribution analysis** to identify hot core (S ⊂ O)
- **Zero-allocation hot path** for ≤8 tick performance

## Usage

### Basic Usage

```rust
use knhk_connectors::{DarkMatterTracker, hash_predicate_iri};

// Create tracker (typically global singleton)
let mut tracker = DarkMatterTracker::new();

// Register predicates for debugging (optional)
let name_predicate = hash_predicate_iri("http://schema.org/name");
tracker.register_predicate(name_predicate, "http://schema.org/name".to_string());

// Record accesses (hot path - inline, lock-free)
for query in queries {
    let predicate_hash = hash_predicate_iri(&query.predicate);
    tracker.record(predicate_hash);
}

// Get coverage metrics
let metrics = tracker.metrics();

println!("Total predicates tracked: {}", metrics.total_predicates);
println!("Hot core size: {}", metrics.hot_core_size);
println!("Hot percentage: {:.1}%", metrics.hot_percentage);
println!("Coverage: {:.1}%", metrics.coverage_percentage);
println!("Total accesses: {}", metrics.total_accesses);

// Check if distribution meets 80/20 threshold
if metrics.meets_pareto_threshold() {
    println!("✅ Meets 80/20 threshold - excellent optimization!");
} else {
    println!("⚠️ Distribution needs optimization");
}

// Show top predicates
for (idx, count, pct) in &metrics.top_predicates {
    if let Some(iri) = tracker.get_predicate_iri(*idx as u64) {
        println!("  {}: {} accesses ({:.1}%)", iri, count, pct);
    }
}
```

### Enterprise Integration Example

```rust
use knhk_connectors::{DarkMatterTracker, hash_predicate_iri, CoverageMetrics};

struct QueryEngine {
    tracker: DarkMatterTracker,
}

impl QueryEngine {
    pub fn new() -> Self {
        let mut tracker = DarkMatterTracker::new();

        // Register common predicates
        for iri in &[
            "http://schema.org/name",
            "http://schema.org/email",
            "http://schema.org/address",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
        ] {
            let hash = hash_predicate_iri(iri);
            tracker.register_predicate(hash, iri.to_string());
        }

        Self { tracker }
    }

    pub fn execute_query(&self, predicate_iri: &str) -> Result<(), String> {
        // Record access (hot path - ≤8 ticks)
        let hash = hash_predicate_iri(predicate_iri);
        self.tracker.record(hash);

        // Execute query...
        Ok(())
    }

    pub fn analyze_coverage(&self) -> CoverageMetrics {
        self.tracker.metrics()
    }
}

// Usage
let engine = QueryEngine::new();

// Simulate query workload
for _ in 0..1000 {
    engine.execute_query("http://schema.org/name")?;  // Hot predicate
}
for _ in 0..100 {
    engine.execute_query("http://schema.org/email")?;  // Warm predicate
}
for _ in 0..10 {
    engine.execute_query("http://schema.org/address")?;  // Cold predicate
}

// Analyze coverage
let metrics = engine.analyze_coverage();

println!("🔥 Hot Core Analysis:");
println!("   Hot predicates: {}/{}", metrics.hot_core_size, metrics.total_predicates);
println!("   Coverage: {:.1}%", metrics.coverage_percentage);
println!("   Sparsity ratio (μ → S): {:.3}", metrics.sparsity_ratio());
```

## Metrics Explained

### `CoverageMetrics`

| Field | Description | Ideal Value |
|-------|-------------|-------------|
| `total_predicates` | Unique predicates tracked | N/A |
| `hot_core_size` | Predicates handling 80% of traffic | ≤20% of total |
| `hot_percentage` | Percentage of predicates in hot core | ≤20% |
| `coverage_percentage` | Traffic handled by hot core | ≥80% |
| `total_accesses` | Total predicate accesses recorded | N/A |
| `top_predicates` | Top 10 predicates by frequency | N/A |

### Interpreting Results

**Perfect 80/20 Distribution**:
```
Total predicates: 100
Hot core size: 20
Hot percentage: 20.0%
Coverage: 80.0%
```
✅ **Optimal** - Focus optimization on 20 predicates to cover 80% of queries

**Sub-optimal Distribution**:
```
Total predicates: 100
Hot core size: 50
Hot percentage: 50.0%
Coverage: 75.0%
```
⚠️ **Needs Work** - Distribution is too flat, hard to optimize

## Performance Characteristics

**Hot Path** (`record()` method):
- **Latency**: ≤8 ticks (sub-nanosecond)
- **Lock-free**: Single atomic increment
- **Zero-allocation**: No heap allocations
- **Branchless**: Optimized for CPU prediction

**Analysis** (`metrics()` method):
- **Latency**: ~100μs for 256 predicates
- **Allocation**: O(n) for sorting
- **Thread-safe**: Read-only atomic loads

## Integration with KNHK Architecture

The Dark Matter tracker integrates with KNHK's three-tier architecture:

### R1 (Hot Path)
```rust
// Record access in hot path (≤8 ticks)
#[inline(always)]
pub fn execute_hot_path_query(tracker: &PredicateCounter, predicate_hash: u64) {
    tracker.record(predicate_hash);
    // ... hot path execution
}
```

### W1 (Warm Path)
```rust
// Periodic analysis in warm path
pub fn periodic_analysis(tracker: &DarkMatterTracker) {
    let metrics = tracker.metrics();

    if !metrics.meets_pareto_threshold() {
        // Trigger optimization: move hot predicates to R1
        optimize_hot_predicates(&metrics.top_predicates);
    }
}
```

### C1 (Cold Path)
```rust
// Long-term trend analysis in cold path
pub async fn trend_analysis(tracker: &DarkMatterTracker) {
    let metrics = tracker.metrics();

    // Store metrics for historical analysis
    store_metrics_to_database(&metrics).await;

    // Generate optimization recommendations
    generate_optimization_plan(&metrics).await;
}
```

## Best Practices

1. **Use a Global Singleton**: Create one `DarkMatterTracker` per process for efficiency
2. **Record in Hot Path**: Call `record()` inline with actual queries for accurate tracking
3. **Analyze Periodically**: Check metrics every 10-60 seconds, not per-query
4. **Focus Optimization**: Use hot core to prioritize SIMD/cache optimizations
5. **Monitor Trends**: Track sparsity ratio over time to detect workload shifts

## Troubleshooting

### High Hot Percentage (>30%)

**Problem**: Too many predicates in hot core, distribution too flat.

**Solutions**:
- Analyze query patterns - may need better caching
- Check if workload is truly uniform (rare in practice)
- Consider increasing dataset size (larger S reveals better 80/20)

### Low Coverage (<70%)

**Problem**: Hot core doesn't cover enough traffic.

**Solutions**:
- Increase hot core threshold (e.g., 85% or 90%)
- Analyze long-tail predicates for optimization opportunities
- Check for workload changes or query pattern shifts

### Zero Predicates Tracked

**Problem**: No calls to `record()` or predicate hashing issues.

**Solutions**:
- Verify `record()` is called in query execution path
- Check `hash_predicate_iri()` produces non-zero hashes
- Enable debug logging to trace predicate registration

## References

- **YAWL.txt**: Dark-Matter 80/20 architecture (line 576)
- **KNHK Repository**: Three-tier performance architecture
- **Pareto Principle**: https://en.wikipedia.org/wiki/Pareto_principle
- **Lock-free Programming**: https://preshing.com/20120612/an-introduction-to-lock-free-programming/

## Example Output

```
🔥 Dark Matter 80/20 Coverage Analysis
=====================================
Total predicates tracked: 43
Hot core size: 8 predicates
Hot percentage: 18.6%
Coverage: 82.3%
Total accesses: 10,543
Sparsity ratio (μ → S): 0.186

✅ MEETS 80/20 THRESHOLD

Top Predicates by Frequency:
  1. http://schema.org/name: 4,231 (40.1%)
  2. http://schema.org/email: 2,105 (20.0%)
  3. http://www.w3.org/1999/02/22-rdf-syntax-ns#type: 1,052 (10.0%)
  4. http://schema.org/address: 527 (5.0%)
  5. http://schema.org/telephone: 316 (3.0%)
  6. http://schema.org/birthDate: 210 (2.0%)
  7. http://schema.org/jobTitle: 158 (1.5%)
  8. http://schema.org/organization: 105 (1.0%)
  ...

📊 Optimization Recommendations:
  ✅ Focus SIMD optimization on top 8 predicates (82.3% coverage)
  ✅ Move these predicates to R1 hot path
  ✅ Excellent 80/20 distribution - well-optimized workload
```
