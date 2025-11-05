# KNHK Documentation Index

**Last Updated**: November 2024

This index provides a comprehensive guide to all current KNHK documentation.

## Essential Documentation (Start Here)

### Getting Started
1. **[README.md](README.md)** - Documentation overview and structure
2. **[QUICK_START.md](QUICK_START.md)** - 5-minute setup guide
3. **[Architecture](architecture.md)** - System architecture overview
   - Three-tier architecture (Hot/Warm/Cold paths)
   - Component descriptions
   - Performance characteristics
   - Hot path routing (≤2ns execution)

### Core Reference
4. **[API Reference](api.md)** - Complete API documentation
   - C API (hot path)
   - Rust API (warm path)
   - Erlang API (cold path)
5. **[CLI Guide](cli.md)** - Command-line interface reference
6. **[Testing](testing.md)** - Testing documentation
   - 83 Chicago TDD tests
   - Coverage by module (~80%+ overall)
   - Test categories and running instructions

## Integration & Deployment

### Integration Guides
7. **[Integration Guide](integration.md)** - General integration examples
8. **[ggen Integration Guide](ggen-integration-guide.md)** - oxigraph integration
   - Warm path with oxigraph
   - Path selection logic
   - Performance tuning
   - Migration guide
9. **[unrdf Integration Status](unrdf-integration-status.md)** - unrdf integration status
10. **[unrdf Integration DoD](unrdf-integration-dod.md)** - unrdf DoD validation
11. **[Weaver Integration](weaver-integration.md)** - Weaver.ai integration

### Deployment & Operations
12. **[Deployment Guide](deployment.md)** - Deployment instructions
13. **[Configuration](configuration.md)** - Configuration reference
14. **[Performance](performance.md)** - Performance guide
    - Hot path: ≤2ns (≤8 ticks)
    - Warm path: ≤500ms
    - Optimization strategies

## Advanced Topics

### Architecture & Design
15. **[Code Organization](code-organization.md)** - Code structure and organization
16. **[Data Flow](data-flow.md)** - Data flow diagrams and execution flow
17. **[Autonomous Epistemology](autonomous-epistemology.md)** - Autonomous system design

### Planning & Requirements
18. **[v1.0 Requirements](v1-requirements.md)** - Forward-looking v1.0 requirements
19. **[v1.0 unrdf Integration Plan](v1.0-unrdf-integration-plan.md)** - unrdf integration requirements
20. **[v1.0 unrdf Gap Analysis](v1.0-unrdf-gap-analysis.md)** - Comprehensive gap analysis

### Research & Evaluation
21. **[ggen RDF Research](ggen-rdf-research.md)** - Research on ggen RDF handling
22. **[ggen Integration Evaluation](ggen-integration-evaluation.md)** - Integration evaluation
23. **[ggen Integration Validation](ggen-integration-validation.md)** - Integration validation

### Specialized Topics
24. **[Definition of Done](DEFINITION_OF_DONE.md)** - DoD criteria
25. **[Documentation Gaps](DOCUMENTATION_GAPS.md)** - Undocumented components
26. **[Documentation Organization](DOCUMENTATION_ORGANIZATION.md)** - Documentation structure guide
27. **[Unrdf Chicago TDD Validation](unrdf-chicago-tdd-validation.md)** - TDD validation results

## Archived Documentation

Historical and version-specific documentation is archived in `archived/`:

### Version-Specific
- `archived/v0.4.0/` - Version 0.4.0 documentation
- `archived/versions/` - Historical version docs (v0.2.0, v0.3.0, etc.)

### Status Reports
- `archived/status/` - Implementation status reports
  - Implementation Complete
  - Conceptual Tests Implemented
  - Consolidation Complete
  - Final Status
  - Code Quality Audit
  - Chicago TDD Validation

### Planning Documents
- `archived/planning/` - Project planning documents
  - Archive Revision Plan
  - Archive Summary
  - JIRA Tickets
  - Lean Six Sigma Project Charter

### Analysis & Implementation Details
- `archived/analysis/` - Analysis documents
- `archived/implementation-details/` - Detailed implementation docs

## Book Documentation

Structured book-formatted documentation is available in `book/src/`:
- Architecture chapters
- API references
- Integration guides
- CLI documentation

## Subproject Documentation

Each subproject has its own `docs/` directory with project-specific documentation:

### Rust Crates
- **[knhk-warm](../rust/knhk-warm/docs/README.md)** - Warm path operations (oxigraph)
- **[knhk-hot](../rust/knhk-hot/docs/README.md)** - Hot path operations (≤2ns)
- **[knhk-etl](../rust/knhk-etl/docs/README.md)** - ETL pipeline
- **[knhk-cli](../rust/knhk-cli/docs/README.md)** - CLI interface
- **[knhk-unrdf](../rust/knhk-unrdf/docs/README.md)** - unrdf integration
- **[knhk-lockchain](../rust/knhk-lockchain/docs/README.md)** - Lockchain
- **[knhk-connectors](../rust/knhk-connectors/docs/README.md)** - Data connectors
- **[knhk-otel](../rust/knhk-otel/docs/README.md)** - OpenTelemetry
- **[knhk-aot](../rust/knhk-aot/docs/README.md)** - AOT compilation
- **[knhk-config](../rust/knhk-config/docs/README.md)** - Configuration
- **[knhk-integration-tests](../rust/knhk-integration-tests/docs/README.md)** - Integration tests
- **[knhk-validation](../rust/knhk-validation/docs/README.md)** - Validation utilities

### Language-Specific
- **[C Hot Path](../c/docs/README.md)** - C hot path implementation
- **[Erlang Cold Path](../erlang/docs/README.md)** - Erlang cold path

### Playground Projects
- **[DoD Validator](../playground/dod-validator/docs/README.md)** - Definition of Done validator

## Documentation Standards

All documentation follows KNHK standards:
- **Production-ready**: Real implementations, no placeholders
- **Chicago TDD**: Test-driven development
- **80/20 Focus**: Critical path emphasis
- **Performance**: Hot path ≤2ns, warm path ≤500ms

## Quick Links

- **Getting Started**: [QUICK_START.md](QUICK_START.md)
- **Architecture**: [architecture.md](architecture.md)
- **API**: [api.md](api.md)
- **Testing**: [testing.md](testing.md)
- **Performance**: [performance.md](performance.md)
- **Integration**: [ggen-integration-guide.md](ggen-integration-guide.md)

## Contributing

When updating documentation:
1. Update this index if adding new docs
2. Follow existing documentation structure
3. Archive outdated versions appropriately
4. Maintain cross-references
