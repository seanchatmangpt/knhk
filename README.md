# KNHK: Knowledge Graph Hot Path Engine

**High-performance knowledge graph engine optimized for hot path operations (≤2ns latency)**

[![Production Ready](https://img.shields.io/badge/status-production--ready-green)](docs/PRODUCTION.md)
[![Version](https://img.shields.io/badge/version-1.0.0-blue)](docs/RELEASE_NOTES_v1.0.0.md)

**Built for Reflex Enterprise™**: KNHK powers Reflex Enterprise™, a 2-ns, law-driven compute fabric. See [Reflex Enterprise Press Release](docs/REFLEX_ENTERPRISE_PRESS_RELEASE.md) for product details.

---

## What is KNHK?

KNHK is a production-ready knowledge graph engine designed for real-time graph operations with strict performance constraints. The system implements guard functions, invariant preservation, and cryptographic provenance through a hooks-based architecture.

**Key Value Proposition**: At the end of each cycle: **A = μ(O)** - The enterprise's current state of action (A) is a verified, deterministic projection of its knowledge (O), within 2ns per rule check.

**Target Audience**: 
- Developers building high-performance knowledge graph applications
- Enterprises requiring deterministic, verifiable graph operations
- Systems needing sub-nanosecond query performance

See [Repository Overview](REPOSITORY_OVERVIEW.md) for complete system overview.

---

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- C compiler (GCC/Clang) with C11 support
- Make (for C build system)

### 5-Minute Setup

```bash
# 1. Build C hot path layer
cd c && make && cd ..

# 2. Build Rust workspace
cd rust && cargo build --workspace --release

# 3. Initialize system
cd rust/knhk-cli && cargo run -- boot init schema.ttl invariants.sparql

# 4. Run pipeline
cargo run -- pipeline run --connectors kafka-prod
```

**Basic Usage**:
```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
engine.register_workflow(spec).await?;
```

See [Quick Start Guide](docs/QUICK_START.md) for detailed setup instructions.

---

## Core Features

- **Hot Path Engine** (C) - ≤8 tick query execution (ASK, COUNT, COMPARE, VALIDATE)
- **Warm Path Engine** (Rust) - ≤500ms emit operations (CONSTRUCT8)
- **8-Beat Epoch System** - Fixed-cadence reconciliation with branchless operations
- **Workflow Engine** - YAWL-compatible workflow execution (42/43 patterns)
- **OTEL Observability** - Full OpenTelemetry integration with Weaver validation
- **Lockchain Provenance** - Cryptographic audit trails for all operations
- **Chicago TDD** - Comprehensive test coverage with real collaborators

See [Architecture Guide](docs/ARCHITECTURE.md) for complete system architecture.

---

## Documentation

### Consolidated Guides (80/20)

**📖 Essential guides covering 80% of use cases:**

- **[Architecture Guide](docs/ARCHITECTURE.md)** - System architecture (Hot/Warm/Cold paths, Core Components)
- **[Workflow Engine Guide](docs/WORKFLOW_ENGINE.md)** - Workflow engine (Quick Start, Core API, Critical Patterns)
- **[Performance Guide](docs/PERFORMANCE.md)** - Performance optimization (Hot Path ≤8 ticks, Benchmarks)
- **[Testing Guide](docs/TESTING.md)** - Chicago TDD methodology and validation
- **[Production Guide](docs/PRODUCTION.md)** - Production readiness and deployment
- **[YAWL Integration Guide](docs/YAWL_INTEGRATION.md)** - YAWL compatibility and status
- **[Ontology Guide](docs/ONTOLOGY.md)** - Ontology integration patterns
- **[API Guide](docs/API.md)** - 🆕 C, Rust, Erlang APIs (80% use cases)
- **[CLI Guide](docs/CLI.md)** - 🆕 Command-line interface (80% use cases)
- **[Integration Guide](docs/INTEGRATION.md)** - 🆕 Integration patterns (80% use cases)

### By User Type

**New Users**:
- [Quick Start Guide](docs/QUICK_START.md) - 5-minute setup
- [Architecture Guide](docs/ARCHITECTURE.md) - System overview
- [API Guide](docs/API.md) - 🆕 C, Rust, Erlang APIs (80% use cases)
- [CLI Guide](docs/CLI.md) - 🆕 Command-line interface (80% use cases)

**For workflow engine users**:
- [Workflow Engine Guide](docs/WORKFLOW_ENGINE.md) - Complete workflow engine guide
- [YAWL Integration Guide](docs/YAWL_INTEGRATION.md) - YAWL compatibility and status

**Developers**:
- [Testing Guide](docs/TESTING.md) - Chicago TDD methodology
- [Performance Guide](docs/PERFORMANCE.md) - Hot path optimization
- [Production Guide](docs/PRODUCTION.md) - Deployment and troubleshooting
- [Integration Guide](docs/INTEGRATION.md) - 🆕 Integration patterns (80% use cases)

**Complete Documentation**: See [Documentation Index](docs/INDEX.md) for all available guides.

---

## Getting Started

### Build

```bash
# Build C hot path layer
cd c && make && cd ..

# Build Rust workspace
cd rust && cargo build --workspace --release
```

### Run Tests

```bash
# Run all tests
cd rust && cargo test --workspace

# Run C tests
make test-chicago-v04
```

### Run Examples

```bash
# See examples directory
cd examples && ls
```

See [Getting Started Guide](docs/QUICK_START.md) for detailed instructions.

---

## Performance Highlights

**Hot Path Performance**:
- ≤8 ticks for all hot path operations (Chatman Constant)
- ≤2ns per operation (ASK, COUNT, COMPARE, VALIDATE)
- Zero branch mispredicts (branchless C engine)
- 10,000-100,000x faster than traditional SPARQL engines

**Key Metrics**:
- ASK operations: ~1.0-1.1 ns ✅
- COUNT operations: ~1.0-1.1 ns ✅
- COMPARE operations: ~0.9 ns ✅
- VALIDATE operations: ~1.5 ns ✅

**18/19 enterprise use cases qualify for hot path!**

See [Performance Guide](docs/PERFORMANCE.md) for complete benchmarks and optimization strategies.

---

## Contributing

KNHK follows core team best practices:

- **80/20 Principle** - Focus on critical 20% features providing 80% value
- **Chicago TDD** - State-based tests with real collaborators (no mocks)
- **Production-Ready** - No placeholders, real implementations only
- **Performance Constraints** - Hot path ≤8 ticks (Chatman Constant)

**Development Standards**:
- All functions use `Result<T, E>` for error handling
- No `unwrap()` or `expect()` in production code paths
- Comprehensive test coverage with Weaver validation
- Zero linting warnings required

See [Development Guides](docs/TESTING.md) for complete development standards.

---

## License

MIT License

## Related Projects

- **[unrdf](https://github.com/seanchatmangpt/unrdf)** - JavaScript knowledge graph engine
- **[oxigraph](https://github.com/oxigraph/oxigraph)** - Rust SPARQL engine

## Status

✅ **v1.0 Production-Ready** - See [Production Guide](docs/PRODUCTION.md) for deployment status

**Current Release**: v1.0.0 - See [Release Notes](docs/RELEASE_NOTES_v1.0.0.md) for details

---

**Never use**: "semantic", "self-" prefixes  
**Always use**: Measurable terms (ontology, schema, invariants, guards)
