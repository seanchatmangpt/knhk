# PMU Benchmark Collection Procedure

**Evidence ID**: `ev:pmu_bench`
**Owner**: Platform Engineering
**Frequency**: On demand, pre-release
**PRD Section**: 5 (Non-Functional Requirements), 11 (Performance Engineering), 13 (Test Plan)
**DFLSS Section**: 12 (CTQ-1: Hot-path speed ≤2 ns/op)

---

## Objective

Collect performance measurement unit (PMU) cycle counts for all hot path operations to validate 8-tick budget compliance (≤2ns @ 4GHz).

---

## Required Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| `cycles` | CPU cycles per operation | ≤32 cycles @ 4GHz (8 ticks × 4 cycles/tick) |
| `latency_ns` | Latency in nanoseconds | ≤2 ns |
| `cache_misses` | L1 cache misses | 0 (hot path data in L1) |
| `branch_misses` | Branch mispredictions | 0 (branchless operations) |
| `ipc` | Instructions per cycle | ≥2.0 (SIMD efficiency) |

---

## Operations to Benchmark

1. **ASK(S,P)** - Subject-predicate existence check
2. **COUNT(S,P)** - Count triples matching S,P
3. **COMPARE(O)** - Object comparison (>, <, ==, >=, <=)
4. **VALIDATE** - Datatype/cardinality validation
5. **SELECT(S,P)** - Select triples by S,P
6. **CONSTRUCT8** - Fixed-template emit (≤8 triples) [Expected: W1 warm path]

---

## Tools Required

- `perf` (Linux Performance Counters) - PMU cycle counting
- `make test-performance-v04` - KNHK performance test suite
- `cargo test` - Rust integration performance tests
- `gcc` - C compiler for Chicago TDD performance tests

---

## Collection Steps

### Step 1: Prepare Environment

```bash
# Ensure performance governor (not powersave)
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable frequency scaling
sudo cpupower frequency-set --governor performance

# Disable hyper-threading (optional, for consistency)
echo 0 | sudo tee /sys/devices/system/cpu/cpu*/online

# Pin to specific CPU core
taskset -c 0 <command>
```

### Step 2: Run Performance Tests with PMU

```bash
cd /Users/sac/knhk

# Option 1: Run KNHK performance test suite
perf stat -e cycles,instructions,cache-misses,branch-misses,L1-dcache-load-misses \
  make test-performance-v04 2>&1 | tee evidence/pmu_bench/perf_output.txt

# Option 2: Run Rust integration performance tests
perf stat -e cycles,instructions,cache-misses,branch-misses \
  cargo test --package knhk-integration-tests --test performance_tests -- --nocapture \
  2>&1 | tee evidence/pmu_bench/rust_perf_output.txt

# Option 3: Run Chicago TDD performance suite
gcc tests/chicago_performance_v04.c -o chicago_perf -O2 -march=native
perf stat -e cycles,instructions,cache-misses,branch-misses \
  taskset -c 0 ./chicago_perf \
  2>&1 | tee evidence/pmu_bench/chicago_perf_output.txt

# Option 4: Run detailed perf record (for flamegraphs)
perf record -F 99 -a -g -- ./chicago_perf
perf script > evidence/pmu_bench/perf_script.txt
```

### Step 3: Extract Operation-Level Metrics

Create a parser script to extract per-operation metrics:

```python
# parse_perf_results.py
import re
import csv

def parse_perf_output(perf_file):
    """Parse perf output to extract per-operation metrics."""
    results = []

    # Example: Extract from test output format
    with open(perf_file, 'r') as f:
        for line in f:
            # Match: Operation: ASK_SP | Cycles: 3.2 | Latency: 0.8 ns
            match = re.search(r'Operation: (\w+) \| Cycles: ([\d.]+) \| Latency: ([\d.]+)', line)
            if match:
                op_name = match.group(1)
                cycles = float(match.group(2))
                latency_ns = float(match.group(3))

                results.append({
                    'operation': op_name,
                    'data_size': 8,  # Fixed for hot path
                    'cycles': cycles,
                    'latency_ns': latency_ns,
                    'cache_misses': 0,  # Extract from separate perf stat
                    'branch_misses': 0,
                    'ipc': 0.0
                })

    return results

def write_csv(results, output_file):
    """Write results to CSV."""
    with open(output_file, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=[
            'operation', 'data_size', 'cycles', 'latency_ns',
            'cache_misses', 'branch_misses', 'ipc'
        ])
        writer.writeheader()
        writer.writerows(results)

if __name__ == '__main__':
    results = parse_perf_output('evidence/pmu_bench/perf_output.txt')
    write_csv(results, 'evidence/pmu_bench/benchmark_results.csv')
    print(f"Extracted {len(results)} benchmark results")
```

Run parser:
```bash
python3 scripts/parse_perf_results.py
```

### Step 4: Validate Against Tick Budget

```python
# validate_tick_budget.py
import csv

TICK_BUDGET = 8  # ticks
CYCLES_PER_TICK = 4  # @ 4GHz
CYCLE_BUDGET = TICK_BUDGET * CYCLES_PER_TICK  # 32 cycles

def validate_budget(csv_file):
    """Validate operations against 8-tick budget."""
    violations = []
    compliant = []

    with open(csv_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            op = row['operation']
            cycles = float(row['cycles'])
            latency_ns = float(row['latency_ns'])

            if cycles > CYCLE_BUDGET:
                violations.append({
                    'operation': op,
                    'cycles': cycles,
                    'budget': CYCLE_BUDGET,
                    'exceeds_by': cycles - CYCLE_BUDGET
                })
            else:
                compliant.append({
                    'operation': op,
                    'cycles': cycles,
                    'latency_ns': latency_ns
                })

    return compliant, violations

if __name__ == '__main__':
    compliant, violations = validate_budget('evidence/pmu_bench/benchmark_results.csv')

    print(f"✅ Compliant operations: {len(compliant)}")
    for op in compliant:
        print(f"  {op['operation']}: {op['cycles']} cycles ({op['latency_ns']} ns)")

    if violations:
        print(f"\n⚠️  Budget violations: {len(violations)}")
        for op in violations:
            print(f"  {op['operation']}: {op['cycles']} cycles (exceeds by {op['exceeds_by']} cycles)")
    else:
        print("\n✅ All operations within 8-tick budget")
```

Run validation:
```bash
python3 scripts/validate_tick_budget.py
```

---

## Expected Output Format

**CSV Structure** (`benchmark_results.csv`):

```csv
operation,data_size,cycles,latency_ns,cache_misses,branch_misses,ipc
ASK_SP,8,3.2,0.8,0,0,2.5
COUNT_SP,8,4.1,1.0,0,0,2.4
COMPARE_O,8,3.6,0.9,0,0,2.8
VALIDATE,8,6.0,1.5,0,0,2.3
SELECT_SP,8,5.6,1.4,1,0,2.2
CONSTRUCT8,8,164,41,12,0,2.1
```

**Notes**:
- `cycles` should be ≤32 for hot path (≤8 ticks @ 4GHz)
- `latency_ns` should be ≤2 ns for hot path
- `cache_misses` should be 0 for hot path (L1 cache-resident)
- `branch_misses` should be 0 (branchless operations)
- `ipc` should be ≥2.0 (SIMD efficiency)
- CONSTRUCT8 expected to exceed budget (routed to W1 warm path)

---

## Validation Criteria

**Pass Criteria**:
- All R1 hot path operations ≤8 ticks (≤32 cycles @ 4GHz)
- All R1 hot path operations ≤2 ns latency
- Zero L1 cache misses for hot path operations
- Zero branch mispredictions for hot path operations

**Known Exceptions**:
- CONSTRUCT8: 41-83 ticks (exceeds budget, routed to W1 warm path)

---

## Troubleshooting

### Issue: Inconsistent Cycle Counts

**Symptom**: Cycle counts vary significantly between runs

**Causes**:
- Frequency scaling enabled
- CPU governor set to "powersave"
- Turbo boost enabled
- Background processes interfering

**Solutions**:
```bash
# Disable frequency scaling
sudo cpupower frequency-set --governor performance

# Disable turbo boost
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Pin to CPU core
taskset -c 0 <command>

# Disable ASLR (for consistency)
echo 0 | sudo tee /proc/sys/kernel/randomize_va_space
```

### Issue: High Cache Misses

**Symptom**: L1 cache misses > 0 for hot path operations

**Causes**:
- Data not L1-resident
- SoA arrays not 64-byte aligned
- Working set exceeds L1 cache size

**Solutions**:
- Validate SoA array alignment (`assert(addr % 64 == 0)`)
- Reduce working set size (run_len ≤ 8)
- Pre-warm cache before benchmarking

### Issue: Branch Mispredictions

**Symptom**: Branch misses > 0

**Causes**:
- Conditional branches in hot path
- Not using branchless operations (masks, cmov, blend)

**Solutions**:
- Review hot path code for branches
- Replace `if` with mask operations
- Use SIMD blend/select instead of conditionals

---

## References

- [8-Beat PRD Section 5](../../docs/8BEAT-PRD.txt) - Non-functional requirements
- [8-Beat PRD Section 11](../../docs/8BEAT-PRD.txt) - Performance engineering
- [Performance Compliance Report](../../docs/performance-compliance-report.md) - Current analysis
- [Chicago TDD Performance Tests](../../tests/chicago_performance_v04.c) - Test suite

---

**Collection Status**: ⚠️ Pending
**Next Action**: Run performance tests and extract PMU data
**Owner**: Platform Engineering
**Target Date**: Week 1
