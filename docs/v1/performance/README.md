# v1.0 Performance

**Status**: Performance Baselines and Benchmarks  
**Last Updated**: 2025-11-09

---

## Overview

This directory contains performance baselines, benchmarks, and PMU (Performance Monitoring Unit) analysis for KNHK v1.0.

---

## Documents

- **[Performance Baseline](./baseline.md)** - Performance baseline metrics and targets
- **[PMU Benchmark Report](./pmu-benchmark.md)** - PMU benchmark analysis and results

---

## Performance Requirements

- Hot path operations: ≤8 ticks (Chatman Constant: 2ns = 8 ticks)
- Zero-copy when possible (references over clones)
- Branchless operations for hot path (constant-time execution)
- SIMD-aware (64-byte alignment for SoA arrays)

---

## Related Documentation

- [Definition of Done](../definition-of-done/)
- [Validation Results](../validation/)
- [Status Reports](../status/)

