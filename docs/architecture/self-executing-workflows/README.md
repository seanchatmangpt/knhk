# Self-Executing Workflows Architecture

**Status**: Design Specification
**Version**: 1.0
**Last Updated**: 2025-11-16

---

## Overview

This directory contains the complete architectural specification for KNHK's self-executing workflows system - a 5-layer architecture that enables workflows to autonomously adapt and evolve based on runtime observations while preserving formal mathematical guarantees.

## Documents

### Core Architecture

| Document | Purpose | Status |
|----------|---------|--------|
| [architecture.md](./architecture.md) | Complete system architecture specification | ✅ Complete |
| [data-flow.puml](./data-flow.puml) | Data flow diagram (PlantUML) | ✅ Complete |
| [c4-layer-integration.puml](./c4-layer-integration.puml) | C4 architecture diagrams | ✅ Complete |
| [sequence-mape-k.puml](./sequence-mape-k.puml) | MAPE-K sequence diagrams | ✅ Complete |

### Quick Links

- **Primary Document**: Start with [architecture.md](./architecture.md)
- **Diagrams**: Generate diagrams from `.puml` files using PlantUML
- **Implementation**: See roadmap in [architecture.md](./architecture.md#implementation-roadmap)

## Architecture Layers

### Layer 1: Ontology (Σ)
- **Purpose**: Source of truth for patterns, policies, and invariants
- **Technologies**: RDF/Turtle, YAWL, MAPE-K
- **Key Files**: `ontology/*.ttl`

### Layer 2: Projection (Π via ggen)
- **Purpose**: Transform ontology into executable code
- **Technologies**: SPARQL, Tera templates, Code generation
- **Key Files**: `rust/knhk-workflow-engine/src/ggen/`

### Layer 3: Execution (μ via KNHK)
- **Purpose**: Execute workflows deterministically
- **Technologies**: Rust, 43 Van der Aalst patterns, Hooks
- **Key Files**: `rust/knhk-workflow-engine/src/engine.rs`

### Layer 4: Observation (O)
- **Purpose**: Collect telemetry and generate receipts
- **Technologies**: OTEL, Weaver, Lockchain
- **Key Files**: `rust/knhk-workflow-engine/src/integration/`

### Layer 5: MAPE-K Feedback
- **Purpose**: Autonomic control loop
- **Technologies**: Monitor, Analyze, Plan, Execute, Knowledge
- **Key Files**: `rust/knhk-workflow-engine/src/mape/`

## Mathematical Foundation

```
Properties:
1. A = μ(O)                    (Deterministic execution)
2. μ ∘ μ = μ                   (Idempotence)
3. O ⊨ Σ                       (Observations respect ontology)
4. Σ ⊨ Q                       (Ontology respects invariants)
5. latency(μ) ≤ 8 ticks        (Chatman constant for hot path)
6. Σ_t → Σ_{t+1}              (Ontology evolution via MAPE-K)
```

## Viewing Diagrams

### Using PlantUML

```bash
# Install PlantUML
brew install plantuml  # macOS
apt install plantuml   # Ubuntu

# Generate PNG diagrams
plantuml data-flow.puml
plantuml c4-layer-integration.puml
plantuml sequence-mape-k.puml

# Generate SVG diagrams
plantuml -tsvg *.puml
```

### Using VS Code

Install the [PlantUML extension](https://marketplace.visualstudio.com/items?itemName=jebbs.plantuml) and preview `.puml` files directly.

### Online

Paste diagram contents into [PlantUML Online Editor](http://www.plantuml.com/plantuml/uml/)

## Implementation Status

| Phase | Duration | Status | Deliverables |
|-------|----------|--------|--------------|
| Phase 1: Foundation | Weeks 1-2 | 🔴 Not Started | Ontology snapshots, SPARQL projection |
| Phase 2: Execution Integration | Weeks 3-4 | 🔴 Not Started | Hooks, receipts, OTEL |
| Phase 3: MAPE-K Loop | Weeks 5-6 | 🔴 Not Started | Monitor, Analyze, Plan, Execute |
| Phase 4: Validation | Weeks 7-8 | 🔴 Not Started | Weaver, tests, benchmarks |

## Key Design Decisions

### ADR-001: Weaver as Source of Truth

**Decision**: Use OTel Weaver schema validation as the single source of truth for feature validation.

**Rationale**: Traditional tests can produce false positives. Weaver validates actual runtime telemetry against declared schemas, eliminating the circular dependency where tests validate test code rather than production behavior.

**Consequences**:
- All features MUST emit telemetry conforming to Weaver schemas
- Schema validation failures mean the feature does NOT work
- Traditional tests provide supporting evidence only

### ADR-002: Atomic Ontology Updates via Snapshots

**Decision**: Use versioned snapshots with atomic pointer swaps for ontology updates.

**Rationale**: Enables linearizable consistency, rollback capability, and provable history.

**Consequences**:
- All ontology reads are consistent (no torn reads)
- Updates are atomic (all or nothing)
- History is auditable (Merkle-linked snapshots)

### ADR-003: MAPE-K for Autonomic Control

**Decision**: Implement full MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge) autonomic loop.

**Rationale**: Self-executing workflows require closed-loop feedback from observations back to ontology updates.

**Consequences**:
- System can adapt to runtime performance changes
- Ontology evolves based on actual execution patterns
- Learning accumulates in knowledge base

## Validation Strategy

### Hierarchy of Trust

1. **Weaver Schema Validation** (MANDATORY - Source of Truth)
   ```bash
   weaver registry check -r registry/
   weaver registry live-check --registry registry/
   ```

2. **Compilation & Code Quality** (Baseline)
   ```bash
   cargo build --release
   cargo clippy --workspace -- -D warnings
   ```

3. **Traditional Tests** (Supporting Evidence - Can Have False Positives)
   ```bash
   cargo test --workspace
   cargo test --test self_executing_integration
   ```

## Performance Guarantees

```
Hot Path:  ≤8 ticks    (P99)  - Pattern execution
Warm Path: ≤500ms      (P99)  - Batch operations
Cold Path: ≤5s         (P95)  - Complex SPARQL queries

MAPE-K Loop:
  Monitor:  ≤100ms
  Analyze:  ≤500ms
  Plan:     ≤1s
  Execute:  ≤2s
  Total:    ≤4s latency
```

## Directory Structure

```
docs/architecture/self-executing-workflows/
├── README.md                      # This file
├── architecture.md                # Complete specification
├── data-flow.puml                 # Data flow diagram
├── c4-layer-integration.puml      # C4 diagrams (Context, Container, Component)
└── sequence-mape-k.puml           # Sequence diagrams (MAPE-K, Receipt, Snapshot)
```

## Related Documentation

- [YAWL Integration](../../YAWL_INTEGRATION.md) - 82% YAWL parity
- [Workflow Engine](../../WORKFLOW_ENGINE.md) - 43 Van der Aalst patterns
- [Weaver Validation](../../WEAVER.md) - Schema validation
- [Console Commands](../../console-commands.md) - Interactive workflow management

## Contributing

### Updating Architecture

1. Modify `architecture.md` with proposed changes
2. Update corresponding diagrams if structure changes
3. Ensure all mathematical properties still hold
4. Update implementation roadmap
5. Submit for architecture review

### Adding Diagrams

1. Create `.puml` file in this directory
2. Follow PlantUML C4 model conventions
3. Reference from `architecture.md`
4. Generate PNG/SVG previews

## Support

For architecture questions, contact:
- Architecture Team: See `architecture.md` document header
- GitHub Issues: Tag with `architecture` label

---

**Last Updated**: 2025-11-16
**Status**: Design Specification
**Review Status**: Pending Architecture Review
