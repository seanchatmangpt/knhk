# Introduction

Welcome to the KNHK (Knowledge Graph Hot Path Engine) documentation.

KNHK is a high-performance knowledge graph engine optimized for hot path operations (≤2ns latency), implementing the Dark Matter 80/20 architecture with Rust-native RDF capabilities and knowledge hook automation.

## What is KNHK?

KNHK is a production-ready knowledge graph engine designed for real-time graph operations with strict performance constraints. The system implements guard functions, invariant preservation, and cryptographic provenance through a hooks-based architecture.

**Built for Reflex Enterprise™**: KNHK powers Reflex Enterprise™, a 2-ns, law-driven compute fabric that replaces procedural software.

## Key Features

- **8-Beat Epoch System**: Fixed-cadence reconciliation with branchless cycle/tick/pulse generation (τ=8)
- **Hot Path**: ≤2ns latency (8 ticks) for critical operations
- **Fiber Execution**: Per-shard execution units with tick-based rotation and park/escalate
- **Ring Buffers**: SoA-optimized Δ-ring (input) and A-ring (output) with per-tick isolation
- **Rust-Native RDF**: Pure Rust SPARQL execution via oxigraph
- **Knowledge Hooks**: Policy-driven automation triggers
- **Cold Path Integration**: unrdf JavaScript integration for complex queries
- **Weaver Integration**: OpenTelemetry live-check validation for telemetry
- **Policy Engine**: Rego-based policy validation for guard constraints and performance budgets

## Formal Foundation

KNHK's behavior is defined through 17 foundational laws (the Constitution) that give rise to emergent properties enabling safe parallelism, cryptographic verification, and deterministic execution.

**Key Insight**: At the end of each cycle: **A = μ(O)** - The enterprise's current state of action (A) is a verified, deterministic projection of its knowledge (O), within 2ns per rule check.

## Quick Start

If you're new to KNHK, start with:

1. [Quick Start](getting-started/quick-start.md) - Get up and running in 5 minutes
2. [Architecture Overview](architecture/system-overview.md) - Understand the system design
3. [8-Beat System](architecture/8beat-system.md) - Learn about the epoch system

## Documentation Structure

This book is organized into the following sections:

- **Getting Started**: Installation, building, and first steps
- **Architecture**: System design and key components
- **API Reference**: Complete API documentation for C, Rust, and Erlang
- **Integration**: How to integrate KNHK into your system
- **Development**: Development guides and best practices
- **Reference**: Detailed reference documentation

## Status

✅ **v1.0 DFLSS Implementation**: 12-agent ultrathink hive mind swarm deployed

See [V1 Status](../V1-STATUS.md) for detailed status and metrics.

## Related Documentation

- [Repository Overview](../../REPOSITORY_OVERVIEW.md) - Complete system overview
- [Documentation Policy](../DOCUMENTATION_POLICY.md) - LEAN pull-based documentation policy
- [Evidence Index](../EVIDENCE_INDEX.md) - Validation evidence catalog

