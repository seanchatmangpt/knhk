# KNHK Tools

Development and benchmarking tools for KNHK.

## Overview

This directory contains tools for development, testing, and benchmarking KNHK components.

## Tools

### knhk_bench

Performance benchmarking tool for hot path operations.

#### Purpose

`knhk_bench` measures execution time of hot path operations to verify they meet the ≤8 tick (≤2ns) constraint. The tool supports both synthetic data generation and loading from RDF files.

#### Usage

```bash
# Build benchmark tool
cd tools
gcc -o knhk_bench knhk_bench.c -lknhk -I../c/include -L../c

# Run with synthetic data (default)
./knhk_bench

# Run with RDF file
./knhk_bench ../test_rdf.ttl

# Run with specific operation
./knhk_bench --op ASK_SP test_rdf.ttl
```

#### Output

```
Using 1000 triples from RDF file
Operation: ASK_SP
Ticks: 5
Time: 1.25ns
Status: PASS (≤8 ticks)
```

#### Benchmark Methodology

1. **Setup**: Initialize context with SoA arrays (64-byte aligned)
2. **Load Data**: Load triples from RDF file or generate synthetic data
3. **Pin Run**: Pin predicate run (max_run_len ≤ 8)
4. **Execute**: Execute operation and measure ticks
5. **Verify**: Check if ticks ≤ 8 (Chatman Constant)

#### Performance Targets

- **Hot Path Operations**: ≤8 ticks (≤2ns at ~250 ps/tick)
- **ASK_SP**: ~1-2ns typical
- **COUNT_SP_GE**: ~1.5-2.5ns typical
- **CONSTRUCT8**: ~2-3ns typical

#### Output Interpretation

- **Ticks**: Number of CPU ticks (should be ≤8)
- **Time**: Actual execution time in nanoseconds
- **Status**: PASS if ticks ≤ 8, FAIL otherwise
- **Lanes**: Number of SIMD lanes used (typically 8)

#### Example Output

```
Benchmark Results:
  Operation: ASK_SP
  Ticks: 5
  Time: 1.25ns
  Lanes: 8
  Status: PASS

Benchmark Results:
  Operation: COUNT_SP_GE
  Ticks: 7
  Time: 1.75ns
  Lanes: 8
  Status: PASS

Benchmark Results:
  Operation: CONSTRUCT8
  Ticks: 8
  Time: 2.00ns
  Lanes: 8
  Status: PASS
```

#### Command-Line Options

```bash
# Show help
./knhk_bench --help

# Specify operation
./knhk_bench --op ASK_SP test_rdf.ttl

# Verbose output
./knhk_bench --verbose test_rdf.ttl

# Output to file
./knhk_bench test_rdf.ttl > benchmark_results.txt
```

#### Building

```bash
# From project root
cd tools
gcc -O3 -o knhk_bench knhk_bench.c \
    -lknhk \
    -I../c/include \
    -L../c \
    -lm

# Or use Makefile
make bench
```

#### Troubleshooting

**Build Errors**:
- Ensure `libknhk.a` is built: `cd c && make`
- Check include paths: `-I../c/include`
- Check library paths: `-L../c`

**Runtime Errors**:
- Verify RDF file exists and is valid
- Check that triples are loaded (triple_count > 0)
- Ensure predicate run length ≤ 8

**Performance Issues**:
- Use `-O3` optimization flag
- Ensure CPU frequency scaling is disabled
- Run on isolated CPU core for consistent results

## Related Tools

### calc_avg.sh

Script for calculating average benchmark results:

```bash
./calc_avg.sh benchmark_results.txt
```

### test_nrows.sh

Script for testing different row counts:

```bash
./test_nrows.sh
```

## Related Documentation

- [Performance Guide](../../docs/performance.md) - Performance characteristics
- [C Hot Path](../../c/docs/README.md) - C implementation details
- [Architecture](../../docs/architecture.md) - System architecture

