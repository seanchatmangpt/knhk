# Summary

[Introduction](intro.md)

# Part I: Foundations

- [What is KNHK?](part1/what-is-knhk.md)
- [What is simdjson?](part1/what-is-simdjson.md)
- [Why Combine Them?](part1/why-combine.md)
- [Performance Philosophy](part1/performance-philosophy.md)
  - [80/20 Principle](part1/performance-philosophy/80-20.md)
  - [Chatman Constant](part1/performance-philosophy/chatman-constant.md)
  - [Measurement-Driven Optimization](part1/performance-philosophy/measurement-driven.md)

# Part II: simdjson Lessons

- [Performance Optimization Techniques](part2/optimization-techniques.md)
  - [Two-Stage Parsing](part2/optimization-techniques/two-stage-parsing.md)
  - [Pseudo-Structural Characters](part2/optimization-techniques/pseudo-structural.md)
  - [On-Demand Parsing](part2/optimization-techniques/on-demand.md)
  - [Runtime CPU Dispatching](part2/optimization-techniques/runtime-dispatch.md)
  - [Memory Reuse](part2/optimization-techniques/memory-reuse.md)
  - [Free Padding Optimization](part2/optimization-techniques/free-padding.md)
- [Architecture Patterns](part2/architecture-patterns.md)
  - [Generic Code + Specialization](part2/architecture-patterns/generic-specialization.md)
  - [Single-Header Distribution](part2/architecture-patterns/single-header.md)
  - [Developer vs Consumer Mode](part2/architecture-patterns/dev-consumer-mode.md)
- [Testing and Validation](part2/testing-validation.md)
  - [Fuzzing Strategy](part2/testing-validation/fuzzing.md)
  - [Differential Testing](part2/testing-validation/differential.md)
  - [Performance Regression Testing](part2/testing-validation/regression.md)
- [API Design Principles](part2/api-design.md)
  - [On-Demand API](part2/api-design/on-demand-api.md)
  - [Type-Specific Parsers](part2/api-design/type-specific.md)
  - [Forward-Only Iteration](part2/api-design/forward-only.md)

# Part III: KNHK Architecture

- [Three-Tier Architecture](part3/three-tier.md)
  - [Hot Path (≤8 ticks)](part3/three-tier/hot-path.md)
  - [Warm Path (≤500ms)](part3/three-tier/warm-path.md)
  - [Cold Path (unbounded)](part3/three-tier/cold-path.md)
- [Hot Path Implementation](part3/hot-path-implementation.md)
  - [SoA Layout](part3/hot-path-implementation/soa-layout.md)
  - [SIMD Operations](part3/hot-path-implementation/simd.md)
  - [Branchless Operations](part3/hot-path-implementation/branchless.md)
  - [Cache Optimization](part3/hot-path-implementation/cache.md)
- [ETL Pipeline](part3/etl-pipeline.md)
  - [Ingest Stage](part3/etl-pipeline/ingest.md)
  - [Transform Stage](part3/etl-pipeline/transform.md)
  - [Load Stage](part3/etl-pipeline/load.md)
  - [Reflex Stage](part3/etl-pipeline/reflex.md)
  - [Emit Stage](part3/etl-pipeline/emit.md)
- [Guard Constraints](part3/guard-constraints.md)
  - [max_run_len ≤ 8](part3/guard-constraints/max-run-len.md)
  - [Tick Budget](part3/guard-constraints/tick-budget.md)
  - [Capacity Limits](part3/guard-constraints/capacity.md)

# Part IV: Applied Optimizations

- [80/20 Implementation](part4/80-20-implementation.md)
  - [Memory Reuse Engine](part4/80-20-implementation/memory-reuse.md)
  - [Branchless Guard Validation](part4/80-20-implementation/branchless-guards.md)
  - [Zero-Copy Triple Views](part4/80-20-implementation/zero-copy-views.md)
  - [Performance Benchmarking](part4/80-20-implementation/benchmarking.md)
- [Performance Patterns](part4/performance-patterns.md)
  - [Buffer Reuse](part4/performance-patterns/buffer-reuse.md)
  - [Cache Alignment](part4/performance-patterns/cache-alignment.md)
  - [Branch Elimination](part4/performance-patterns/branch-elimination.md)
  - [SIMD Vectorization](part4/performance-patterns/simd.md)
- [Measurement and Validation](part4/measurement.md)
  - [Benchmarking Infrastructure](part4/measurement/benchmarking.md)
  - [Performance Metrics](part4/measurement/metrics.md)
  - [OTEL Integration](part4/measurement/otel.md)
  - [Tick Budget Validation](part4/measurement/tick-budget.md)

# Part V: Implementation Guide

- [Getting Started](part5/getting-started.md)
  - [Prerequisites](part5/getting-started/prerequisites.md)
  - [Building KNHK](part5/getting-started/building.md)
  - [Running Benchmarks](part5/getting-started/benchmarks.md)
- [Using Optimizations](part5/using-optimizations.md)
  - [Hot Path Engine](part5/using-optimizations/hot-path-engine.md)
  - [Guard Validation](part5/using-optimizations/guard-validation.md)
  - [Triple Views](part5/using-optimizations/triple-views.md)
- [Best Practices](part5/best-practices.md)
  - [Performance Guidelines](part5/best-practices/performance.md)
  - [Memory Management](part5/best-practices/memory.md)
  - [Error Handling](part5/best-practices/error-handling.md)
  - [Testing](part5/best-practices/testing.md)
- [Troubleshooting](part5/troubleshooting.md)
  - [Performance Issues](part5/troubleshooting/performance.md)
  - [Memory Issues](part5/troubleshooting/memory.md)
  - [Cache Issues](part5/troubleshooting/cache.md)

# Part VI: Case Studies

- [Case Study: Memory Reuse](part6/case-study-memory-reuse.md)
- [Case Study: Branchless Guards](part6/case-study-branchless.md)
- [Case Study: Zero-Copy Views](part6/case-study-zero-copy.md)
- [Case Study: Benchmarking](part6/case-study-benchmarking.md)
- [Performance Comparison](part6/performance-comparison.md)
  - [Before Optimizations](part6/performance-comparison/before.md)
  - [After Optimizations](part6/performance-comparison/after.md)
  - [Lessons Learned](part6/performance-comparison/lessons.md)

# Appendices

- [simdjson API Reference](appendices/simdjson-api.md)
- [KNHK API Reference](appendices/knhk-api.md)
- [Performance Metrics](appendices/performance-metrics.md)
- [Glossary](appendices/glossary.md)
- [Bibliography](appendices/bibliography.md)






