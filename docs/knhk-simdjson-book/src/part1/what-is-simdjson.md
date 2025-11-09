# What is simdjson?

simdjson is a high-performance JSON parser that achieves **gigabytes-per-second parsing speeds** through SIMD instructions, microparallel algorithms, and careful engineering.

## Core Philosophy

simdjson demonstrates that careful engineering, measurement-driven optimization, and pragmatic design choices can achieve **order-of-magnitude performance improvements** while maintaining code quality and usability.

## Key Performance Characteristics

- **Speed**: Parses JSON at gigabytes per second
- **Efficiency**: Uses SIMD instructions for parallel processing
- **Reliability**: Comprehensive testing and validation
- **Usability**: Simple API despite complex internals

## Architecture Principles

### Two-Stage Parsing

simdjson separates fast structural identification from slower semantic parsing:

1. **Stage 1 (Find marks)**: Fast SIMD-based identification of structural elements, strings, and UTF-8 validation
2. **Stage 2 (Structure building)**: Slower semantic parsing (numbers, atoms, tree construction)

**Benefit**: Optimize the critical path (structural validation) separately from less critical operations (semantic parsing).

### Pseudo-Structural Characters

simdjson redefines what constitutes "structural" to reduce branch misprediction:

A character is pseudo-structural if:
1. Not enclosed in quotes, AND
2. Is a non-whitespace character, AND
3. Its preceding character is either:
   - a structural character, OR
   - whitespace, OR
   - the final quote in a string

**Benefit**: Fewer branches = better branch prediction = faster execution.

### On-Demand Parsing

simdjson parses only what you use, when you use it:

- **On-Demand API**: Parse values only when accessed
- **Type-Specific Parsing**: Different parsers for `double`, `uint64_t`, `int64_t`
- **Forward-Only Iteration**: Single index maintained, no backtracking

**Benefit**: Avoid parsing unnecessary data, improving performance for partial access patterns.

### Runtime CPU Dispatching

simdjson compiles multiple optimized kernels and selects the best at runtime:

- **Multiple Implementations**: icelake, haswell, westmere, arm64, fallback
- **Runtime Detection**: CPU detection selects best implementation
- **Zero Overhead**: Dispatch table lookup, no branches

**Benefit**: Optimal performance on each CPU architecture without manual selection.

### Memory Reuse

simdjson reuses buffers to keep memory hot in cache:

- **Reusable Parsers**: Parser instances can be reused
- **Buffer Reuse**: Internal buffers reused across operations
- **Cache-Friendly**: Keeps memory hot in L1 cache

**Benefit**: Reduces allocation overhead and improves cache locality.

## Key Lessons for KNHK

### 1. Measure What Matters

simdjson uses comprehensive benchmarking to validate performance:

- **Reproducible Benchmarks**: Consistent measurement methodology
- **Performance Regression Testing**: Detect regressions early
- **Real-World Workloads**: Benchmark with actual use cases

**Application to KNHK**: Benchmark hot path operations to validate ≤8 ticks constraint.

### 2. Optimize the Critical Path

simdjson focuses optimization effort on the critical path:

- **80/20 Principle**: Optimize the 20% that provides 80% of value
- **Structural Validation**: Fast path for common operations
- **Semantic Parsing**: Slower path only when needed

**Application to KNHK**: Optimize hot path (≤8 ticks) separately from warm/cold paths.

### 3. Eliminate Branches

simdjson uses branchless operations for better performance:

- **Arithmetic Comparisons**: Use arithmetic instead of branches
- **Conditional Moves**: Compiler generates conditional moves, not branches
- **Constant-Time Operations**: Predictable performance characteristics

**Application to KNHK**: Branchless guard validation and predicate matching.

### 4. Cache-Friendly Design

simdjson optimizes for cache locality:

- **64-Byte Alignment**: Align data structures to cache lines
- **Structure-of-Arrays**: SoA layout for SIMD operations
- **Memory Reuse**: Reuse buffers to keep memory hot

**Application to KNHK**: SoA layout with 64-byte alignment for hot path operations.

### 5. Production-Ready Code

simdjson maintains code quality despite performance focus:

- **Comprehensive Testing**: Unit tests, integration tests, fuzzing
- **Error Handling**: Proper error handling throughout
- **Documentation**: Clear documentation and examples

**Application to KNHK**: Production-ready code with proper error handling and testing.

## Performance Metrics

simdjson achieves impressive performance:

- **Parsing Speed**: Gigabytes per second
- **Latency**: Sub-microsecond parsing for small documents
- **Memory Efficiency**: Minimal memory overhead
- **Scalability**: Performance scales with CPU capabilities

## Relationship to KNHK

KNHK applies simdjson's lessons to knowledge graph operations:

1. **Two-Stage Processing**: Fast structural validation + slower semantic parsing
2. **Memory Reuse**: Reuse SoAArrays buffers across operations
3. **Branchless Operations**: Eliminate branches in guard validation
4. **Cache Alignment**: 64-byte alignment for SIMD operations
5. **Measurement-Driven**: Benchmarking to validate performance

## Next Steps

- Learn [why we combine KNHK and simdjson](why-combine.md)
- Explore [performance philosophy](performance-philosophy.md)
- See [how simdjson lessons are applied](part4/80-20-implementation.md)



