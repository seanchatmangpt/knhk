# Lean Six Sigma Design for KNHK Quality Improvement

**Audit Date:** 2025-11-17
**Methodology:** DMAIC + Poka-Yoke + FMEA

---

## Executive Summary

This document applies Lean Six Sigma methodologies to address the gaps identified in the KNHK Rust workflow system audit. The current state shows **significant quality defects** (workflow execution is stubbed), requiring structured improvement.

**Current Process Capability:**
- **Cpk < 0:** Process cannot meet specifications (workflows don't execute)
- **DPMO:** ~1,000,000 (every workflow fails to execute)
- **Sigma Level:** <1σ (far below industry standard of 3-4σ)

---

## 1. POKA-YOKE (Error-Proofing)

### Philosophy
"Make it impossible to do the wrong thing."

### Current Failure Modes

#### A. Silent Failure (No Workflow Execution)

**Current Design (WRONG):**
```rust
fn parse_descriptor(descriptor: &str) -> Result<Vec<WorkflowStep>, Box<dyn std::error::Error>> {
    Ok(vec![])  // ← SILENT FAILURE
}
```

**Poka-Yoke Design:**
```rust
fn parse_descriptor(descriptor: &str) -> Result<Vec<WorkflowStep>, WorkflowError> {
    // STAGE 1: Poka-Yoke - Impossible to return empty on valid input
    if descriptor.is_empty() {
        return Err(WorkflowError::EmptyDescriptor);
    }

    if descriptor.len() > MAX_DESCRIPTOR_SIZE {
        return Err(WorkflowError::DescriptorTooLarge(descriptor.len()));
    }

    // STAGE 2: Parse with validation
    let parsed: WorkflowSpec = serde_yaml::from_str(descriptor)
        .map_err(|e| WorkflowError::ParseError(e.to_string()))?;

    // STAGE 3: Poka-Yoke - Ensure at least one step
    if parsed.steps.is_empty() {
        return Err(WorkflowError::NoSteps);
    }

    // STAGE 4: Validate each step
    for (i, step) in parsed.steps.iter().enumerate() {
        if step.name.is_empty() {
            return Err(WorkflowError::InvalidStep {
                index: i,
                reason: "Step name cannot be empty".to_string(),
            });
        }
    }

    // STAGE 5: Convert to internal representation
    let steps = parsed.steps.into_iter()
        .map(|s| WorkflowStep::from_spec(s))
        .collect();

    // STAGE 6: Final sanity check (defensive programming)
    assert!(!steps.is_empty(), "Steps cannot be empty after validation");

    Ok(steps)
}
```

**Error-Proofing Mechanisms:**
1. ✅ Empty descriptor caught immediately
2. ✅ Size limit prevents DoS
3. ✅ Schema validation ensures structure
4. ✅ Step count validation prevents empty workflows
5. ✅ Per-step validation catches malformed data
6. ✅ Runtime assertion as final guard

---

#### B. Fake Success (Workflow Execution)

**Current Design (WRONG):**
```rust
async fn execute_step(&self, step: &WorkflowStep) -> Result<Receipt, Error> {
    Ok(Receipt::default())  // ← LIES ABOUT SUCCESS
}
```

**Poka-Yoke Design:**
```rust
async fn execute_step(&self, step: &WorkflowStep) -> Result<Receipt, ExecutionError> {
    // STAGE 1: Type-state pattern - Receipt can only be created on success
    let mut executor = StepExecutor::new(step);

    // STAGE 2: Execute with timeout
    let result = timeout(
        step.timeout.unwrap_or(DEFAULT_TIMEOUT),
        executor.run()
    ).await
    .map_err(|_| ExecutionError::Timeout)?
    .map_err(|e| ExecutionError::StepFailed(e))?;

    // STAGE 3: Validate result integrity
    if !result.is_valid() {
        return Err(ExecutionError::InvalidResult);
    }

    // STAGE 4: Create receipt (impossible without success)
    let receipt = Receipt::builder()
        .step_id(&step.id)
        .timestamp(SystemTime::now())
        .result(result)
        .checksum()  // ← Cryptographic proof
        .build();

    // STAGE 5: Verify receipt integrity
    assert!(receipt.verify(), "Receipt failed integrity check");

    Ok(receipt)
}

// Type-state pattern makes fake receipts impossible
impl Receipt {
    // Private constructor - can only be created through builder
    fn new() -> Self { unreachable!() }

    pub fn builder() -> ReceiptBuilder {
        ReceiptBuilder::default()
    }
}

impl ReceiptBuilder {
    pub fn build(self) -> Receipt {
        // Impossible to build without all required fields
        assert!(self.step_id.is_some(), "Receipt requires step_id");
        assert!(self.result.is_some(), "Receipt requires result");
        assert!(self.timestamp.is_some(), "Receipt requires timestamp");

        Receipt {
            // ... all fields validated
        }
    }
}
```

**Error-Proofing Mechanisms:**
1. ✅ Timeout prevents infinite execution
2. ✅ Type-state pattern - no fake receipts possible
3. ✅ Builder pattern enforces all required fields
4. ✅ Cryptographic checksum prevents tampering
5. ✅ Runtime assertions catch logic errors
6. ✅ Private constructor prevents bypass

---

#### C. Cost Estimation Errors

**Current Design (WRONG):**
```rust
cpu_core_seconds: seconds * 0.5, // ← GUESS
```

**Poka-Yoke Design:**
```rust
// DESIGN 1: Type-safe measurement vs. estimate
enum ResourceMeasurement {
    Measured {
        value: f64,
        measurement_time: SystemTime,
        source: MeasurementSource,
    },
    Estimated {
        value: f64,
        confidence: f64,  // 0.0 = no confidence, 1.0 = perfect
        basis: EstimationBasis,
    },
}

// DESIGN 2: Impossible to confuse measured with estimated
struct ResourceUsage {
    cpu: ResourceMeasurement,
    memory: ResourceMeasurement,
    // ...
}

// DESIGN 3: Cost calculation requires acknowledgment of estimation
fn calculate_cost(&self, usage: &ResourceUsage) -> Result<Cost, CostError> {
    let mut warnings = Vec::new();

    let cpu_cost = match &usage.cpu {
        ResourceMeasurement::Measured { value, .. } => {
            value * self.pricing.cpu_per_core_hour
        }
        ResourceMeasurement::Estimated { value, confidence, basis } => {
            if *confidence < 0.5 {
                warnings.push(CostWarning::LowConfidenceEstimate {
                    resource: "CPU",
                    confidence: *confidence,
                });
            }
            value * self.pricing.cpu_per_core_hour
        }
    };

    if !warnings.is_empty() {
        return Err(CostError::EstimatesUsed { warnings });
    }

    Ok(Cost { cpu: cpu_cost, /* ... */ })
}
```

**Error-Proofing Mechanisms:**
1. ✅ Type system distinguishes measured from estimated
2. ✅ Confidence levels make uncertainty explicit
3. ✅ Warnings prevent silent estimation
4. ✅ Audit trail shows data source
5. ✅ Cannot bill customers with estimates (compile error)

---

### Poka-Yoke Checklist for New Features

Before merging ANY new code:

- [ ] **Can this return success when it failed?** → Add type-state pattern
- [ ] **Can this accept invalid input?** → Add validation layer
- [ ] **Can this produce wrong results silently?** → Add integrity checks
- [ ] **Can estimates be confused with facts?** → Use type-safe enums
- [ ] **Can this leak resources?** → Use RAII guards
- [ ] **Can this deadlock?** → Use lock ordering or lock-free structures
- [ ] **Can this overflow?** → Use checked arithmetic
- [ ] **Can this panic in production?** → Replace panic!() with Result<>

---

## 2. FMEA (Failure Mode & Effects Analysis)

### Workflow Execution FMEA

| Failure Mode | Potential Cause | Effect | Severity | Occurrence | Detection | RPN | Actions |
|--------------|----------------|--------|----------|------------|-----------|-----|---------|
| Empty workflow executed | parse_descriptor() stub | Silent failure, fake receipts | 10 | 10 | 10 | 1000 | Implement parser, add validation |
| Step execution skipped | execute_step() stub | No work done, fake success | 10 | 10 | 10 | 1000 | Implement executor, type-state pattern |
| Resource metrics missing | get_cpu_usage() stub | Wrong scaling decisions | 7 | 10 | 9 | 630 | Integrate sysinfo crate |
| Cost estimates wrong | Hardcoded multipliers | Financial misreporting | 8 | 10 | 10 | 800 | Measure actual usage |
| Learning never improves | All stubs in learning.rs | No optimization over time | 6 | 10 | 8 | 480 | Implement pattern recognition |
| Alerts not delivered | Channel stubs | Incidents not escalated | 7 | 8 | 7 | 392 | Integrate HTTP/email clients |
| Cluster fails to scale | Load balancer stub | Single node overload | 6 | 7 | 8 | 336 | Implement actual load balancing |
| State recovery fails | reconstruct_from_persistence() stub | Data loss on corruption | 9 | 2 | 5 | 90 | Query RocksDB for rebuild |

**RPN Threshold for Production:** 100
**Items Above Threshold:** 7 out of 8

**Conclusion:** System is NOT ready for production until RPN < 100 for all failure modes.

---

### FMEA-Driven Development Process

**Before implementing ANY feature:**

1. **Identify Failure Modes:**
   - What can go wrong with this feature?
   - List all possible failure scenarios

2. **Rate Each Failure:**
   - **Severity (1-10):** How bad if it happens?
   - **Occurrence (1-10):** How likely to happen?
   - **Detection (1-10):** How hard to detect? (10 = impossible)

3. **Calculate RPN:**
   - RPN = Severity × Occurrence × Detection
   - If RPN > 100: **Redesign before coding**

4. **Design Controls:**
   - **Prevention:** Poka-yoke to prevent failure
   - **Detection:** Automated tests to catch failure
   - **Mitigation:** Graceful degradation if failure occurs

5. **Verify Controls:**
   - Write tests for failure modes
   - Ensure RPN < 100 after controls

**Example for New Feature: "Add Retry Logic"**

| Failure Mode | S | O | D | RPN | Control |
|--------------|---|---|---|-----|---------|
| Infinite retry loop | 8 | 6 | 7 | 336 | Max retries = 3, exponential backoff |
| Retrying non-idempotent operation | 10 | 5 | 8 | 400 | Idempotency token required |
| All retries fail silently | 9 | 3 | 9 | 243 | Emit metric + alert on exhaustion |

Actions:
- Redesign to enforce idempotency
- Add circuit breaker after 3 failures
- Alert on retry exhaustion

---

## 3. PROCESS CAPABILITY (Cpk)

### Current State Analysis

**Specification:**
- Workflows must execute actual steps
- Receipts must be cryptographically valid
- Cost tracking must use measured resources

**Current Performance:**
- 0% of workflows execute steps (parse returns empty)
- 100% of receipts are fake (execute returns default)
- 100% of costs are estimates (hardcoded multipliers)

**Process Capability:**
```
USL (Upper Spec Limit) = 100% valid workflows
LSL (Lower Spec Limit) = 90% valid workflows
Target = 100%

Current Mean (μ) = 0%
Current StdDev (σ) = 0 (deterministic failure)

Cpk = min(
    (USL - μ) / (3σ),
    (μ - LSL) / (3σ)
) = undefined (divide by zero)
```

**Interpretation:** Process is incapable. Not even at 1σ level.

---

### Target State (Post-Fix)

**After implementing:**
- Actual workflow execution
- Type-safe receipt generation
- Measured resource tracking

**Expected Performance:**
- 99.9% of workflows execute correctly (0.1% failure from transient errors)
- 100% of receipts are cryptographically valid (enforced by type system)
- 95% of costs are measured (5% use validated estimates with warnings)

**Process Capability (Target):**
```
USL = 100%
LSL = 99%
Target = 100%

μ = 99.9%
σ = 0.05%

Cpk = min(
    (100 - 99.9) / (3 * 0.05),
    (99.9 - 99) / (3 * 0.05)
) = min(0.67, 6.0) = 0.67
```

**Interpretation:** Cpk ≈ 0.67 is below minimum (1.33 for production). Need tighter control.

---

### Improvement Plan to Cpk ≥ 1.33

**Root Causes of Variation:**
1. Transient network errors → Retry with exponential backoff
2. Resource exhaustion → Circuit breaker + load shedding
3. Invalid input → Pre-validation + schema enforcement
4. Concurrent access → Lock-free data structures

**Actions:**
- Implement retry logic (reduces σ by 50%)
- Add input validation (reduces σ by 30%)
- Use circuit breakers (reduces σ by 20%)

**New Expected:**
```
μ = 99.95%
σ = 0.01%

Cpk = (99.95 - 99) / (3 * 0.01) = 31.67
```

**Result:** Cpk > 1.33 ✅ (actually exceptional at >30)

---

## 4. CONTROL CHARTS

### What to Monitor (Real-Time SPC)

#### Chart 1: Workflow Success Rate (p-chart)
```
UCL (Upper Control Limit) = 100%
Target = 99.9%
LCL (Lower Control Limit) = 99.5%

Sampling: Every 1000 workflows
Plot: Success rate %

Alert if:
- Any point below LCL → Investigate immediately
- 7 consecutive points below target → Process degrading
- 2 out of 3 points in Zone A → Unusual variation
```

#### Chart 2: Workflow Latency (X-bar & R chart)
```
Target = 100ms (P50)
UCL = 300ms
LCL = 50ms

Sampling: Every 100 workflows
Plot: Mean latency & range

Alert if:
- Any point above UCL → Performance issue
- Trend of 7 increasing → Process degrading
- Range increasing → Inconsistent performance
```

#### Chart 3: Cost Per Workflow (I-MR chart)
```
Target = $0.10
UCL = $0.15
LCL = $0.05

Sampling: Daily aggregate
Plot: Individual costs & moving range

Alert if:
- Any point above UCL → Cost spike investigation
- 8 consecutive points above center → Cost creep
```

#### Chart 4: Error Rate (c-chart)
```
Expected errors per 1000 workflows = 1
UCL = 5
LCL = 0

Sampling: Every 1000 workflows
Plot: Count of errors

Alert if:
- Any point above UCL → Error surge
- 15 consecutive points below 1 → Suspiciously low (check detection)
```

---

### Control Chart Implementation

```rust
struct ControlChart {
    chart_type: ChartType,
    ucl: f64,
    target: f64,
    lcl: f64,
    samples: VecDeque<Sample>,
    alerts: Vec<ControlAlert>,
}

enum ChartType {
    P,      // Proportion (success rate)
    XBar,   // Mean (latency)
    I,      // Individual (cost)
    C,      // Count (errors)
}

struct ControlAlert {
    timestamp: SystemTime,
    rule_violated: ControlRule,
    value: f64,
    severity: Severity,
}

enum ControlRule {
    PointBeyondLimits,
    SevenConsecutiveBelowTarget,
    TrendOfSeven,
    TwoOfThreeInZoneA,
    // ... Western Electric rules
}

impl ControlChart {
    fn add_sample(&mut self, value: f64) {
        self.samples.push_back(Sample { value, timestamp: SystemTime::now() });

        // Check Western Electric rules
        if value > self.ucl || value < self.lcl {
            self.alerts.push(ControlAlert {
                timestamp: SystemTime::now(),
                rule_violated: ControlRule::PointBeyondLimits,
                value,
                severity: Severity::Critical,
            });
        }

        // Check for trends
        if self.has_trend_of_seven() {
            self.alerts.push(/* ... */);
        }

        // ... other rules
    }
}
```

---

## 5. DESIGN OF EXPERIMENTS (DOE)

### Factors Affecting Performance

**Objective:** Determine optimal configuration for 99.9% success rate at <100ms latency

**Factors to Test:**
1. **Executor Pool Size** (5, 10, 20, 50, 100)
2. **Workflow Timeout** (30s, 60s, 120s, 300s)
3. **Retry Policy** (None, 3-tries, 5-tries, Exponential)
4. **Buffer Size** (1000, 5000, 10000, 50000)
5. **Compression** (None, LZ4, Zstd)

**Design:** 2^k Factorial Design (5 factors, 2 levels each)

### Example DOE Matrix

| Run | Pool | Timeout | Retry | Buffer | Compression | Success Rate | P50 Latency | Cost |
|-----|------|---------|-------|--------|-------------|--------------|-------------|------|
| 1 | 10 | 60s | 3 | 5000 | LZ4 | 99.2% | 95ms | $0.12 |
| 2 | 50 | 120s | 5 | 10000 | Zstd | 99.8% | 110ms | $0.15 |
| ... | | | | | | | | |

**Analysis:**
- Main effects: Which factors matter most?
- Interactions: Do factors affect each other?
- Optimization: Find Pareto optimal configuration

**Expected Results:**
- Pool size has HIGH impact on latency
- Retry policy has HIGH impact on success rate
- Compression has MEDIUM impact on cost
- Buffer size has LOW impact (diminishing returns)

**Recommended Config:**
```yaml
executor_pool_size: 50  # Sweet spot for latency
workflow_timeout: 120s  # 2 minutes (95th percentile)
retry_policy:
  max_attempts: 3
  backoff: exponential
buffer_size: 10000  # Balances memory vs. performance
compression: lz4  # Fast enough, good ratio
```

---

## 6. KAIZEN (Continuous Improvement)

### Improvement Kata

**Weekly Cycle:**

1. **Understand Direction** (Monday)
   - Review control charts
   - Identify deviations from target

2. **Grasp Current Condition** (Tuesday)
   - Measure actual performance
   - Compare to baseline

3. **Establish Target Condition** (Wednesday)
   - Set next improvement goal
   - Define success criteria

4. **PDCA Iterations** (Thu-Fri)
   - Plan: Design small experiment
   - Do: Implement change
   - Check: Measure results
   - Act: Standardize or revert

**Example Kata: Reduce P99 Latency**

| Week | Current P99 | Target | Experiment | Result |
|------|-------------|--------|------------|--------|
| 1 | 500ms | 400ms | Increase pool to 20 | 420ms ✅ |
| 2 | 420ms | 350ms | Add step-level timeout | 380ms ✅ |
| 3 | 380ms | 300ms | Optimize receipt generation | 310ms ✅ |
| 4 | 310ms | 250ms | Cache common patterns | 260ms ✅ |

---

## Summary: Lean Six Sigma Action Plan

### Phase 1: Define (Week 1-2)
- [x] Audit current system
- [x] Identify critical gaps
- [x] Establish baseline metrics (Cpk < 0)
- [ ] Define target state (Cpk ≥ 1.33)
- [ ] Get stakeholder buy-in

### Phase 2: Measure (Week 3-4)
- [ ] Implement control charts
- [ ] Collect baseline data (100 workflows)
- [ ] Calculate current DPMO (1M)
- [ ] Identify top 3 defects (parse, execute, cost)

### Phase 3: Analyze (Week 5-8)
- [ ] FMEA for critical failures
- [ ] Root cause analysis (5 Whys)
- [ ] Pareto analysis (80/20 rule)
- [ ] Identify quick wins

### Phase 4: Improve (Week 9-20)
- [ ] Implement poka-yoke for workflow execution
- [ ] Type-safe receipt generation
- [ ] Measured resource tracking
- [ ] DOE for optimal configuration
- [ ] Reduce RPN < 100 for all failure modes

### Phase 5: Control (Week 21-24)
- [ ] Deploy control charts to production
- [ ] Establish Standard Operating Procedures
- [ ] Train team on new processes
- [ ] Continuous monitoring with SPC
- [ ] Monthly Kaizen reviews

**Expected Outcome:**
- Cpk: <0 → 1.5+
- DPMO: 1,000,000 → <1,000 (3σ)
- Success Rate: 0% → 99.9%
- Production Ready: No → Yes

**Timeline:** 6 months to production-ready quality
