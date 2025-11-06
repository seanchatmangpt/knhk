# KNHK Documentation Index

**Last Updated**: January 2025 (80/20 Consolidation - Status docs merged into core documents)

This index provides a comprehensive guide to all current KNHK documentation.

## Essential Documentation (Start Here)

### Getting Started
1. **[README.md](README.md)** - Documentation overview and structure
2. **[QUICK_START.md](QUICK_START.md)** - 5-minute setup guide
3. **[Current Status](STATUS.md)** - Implementation status and known limitations
4. **[Architecture](architecture.md)** - System architecture overview
   - Three-tier architecture (Hot/Warm/Cold paths)
   - Component descriptions
   - Performance characteristics
   - Hot path routing (≤2ns execution)

### Core Reference
5. **[API Reference](api.md)** - Complete API documentation
   - C API (hot path)
   - Rust API (warm path)
   - Erlang API (cold path)
6. **[CLI Guide](cli.md)** - Command-line interface reference
7. **[Chicago TDD](CHICAGO_TDD.md)** - Chicago TDD methodology and test coverage
   - Test principles and patterns
   - Coverage summary
   - Test execution status
8. **[Testing](testing.md)** - Testing documentation
   - 83 Chicago TDD tests
   - Coverage by module (~80%+ overall)
   - Test categories and running instructions
9. **[Formal Mathematical Foundations](formal-foundations.md)** - Formal laws and emergent properties
   - 17 foundational laws (Constitution)
   - 10 emergent computational properties
   - Mathematical rigor and verification
   - Connection to implementation

### Hooks Engine Documentation
7. **[Hooks Engine: 2ns Use Cases](hooks-engine-2ns-use-cases.md)** - Complete hooks engine architecture and laws
   - Guard function: μ ⊣ H (partial)
   - Use Case 1: Single hook execution (2ns target)
   - Use Case 2: Batch hook evaluation (cold path)
   - Mathematical foundations and laws
8. **[Hooks Engine: Chicago TDD Coverage](hooks-engine-chicago-tdd-coverage.md)** - Test coverage by law and use case
   - 14 Chicago TDD tests covering all laws
   - Coverage mapping by law
   - Use case validation
9. **[Hooks Engine: Error Validation Tests](hooks-engine-error-validation-tests.md)** - What works and what doesn't
   - 17 error validation tests
   - Query type validation
   - Hook definition validation
   - Success/failure boundaries
10. **[Hooks Engine: Stress Tests & Benchmarks](hooks-engine-stress-tests.md)** - Performance validation
    - 7 stress tests
    - Performance benchmarks
    - Concurrent execution tests
    - Memory pressure tests

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
11. **[Weaver Integration](WEAVER_INTEGRATION.md)** - Complete Weaver integration guide
    - Architecture patterns learned (Ingester, Advisor, Diagnostic)
    - Implementation status and usage examples
    - Test coverage (44 Chicago TDD tests)
    - Next steps and roadmap
12. **[Weaver Analysis and Learnings](WEAVER_ANALYSIS_AND_LEARNINGS.md)** - Architectural patterns learned from Weaver

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
17. **[Formal Mathematical Foundations](formal-foundations.md)** - Deep formal insights and emergent properties
18. **[Autonomous Epistemology](autonomous-epistemology.md)** - Autonomous system design

### Planning & Requirements
19. **[v1.0 Requirements](v1-requirements.md)** - Forward-looking v1.0 requirements
20. **[v1.0 unrdf Integration Plan](v1.0-unrdf-integration-plan.md)** - unrdf integration requirements
21. **[v1.0 unrdf Gap Analysis](v1.0-unrdf-gap-analysis.md)** - Comprehensive gap analysis

### Product Information
22. **[Reflex Enterprise Press Release](REFLEX_ENTERPRISE_PRESS_RELEASE.md)** - Product launch announcement for Reflex Enterprise™
   - Built on KNHK and unrdf
   - ≤2ns hot path guards
   - Law-driven compute fabric
   - Customer outcomes and metrics
23. **[DFLSS Project Charter](DFLSS_PROJECT_CHARTER.md)** - DMADV project charter for Reflex Enterprise rollout
   - CTQs (Critical-to-Quality metrics)
   - DMADV tollgates (Define, Measure, Analyze, Design, Verify)
   - Risk mitigation strategies
   - Acceptance criteria and governance
24. **[DFLSS Project Charter DoD](DFLSS_PROJECT_CHARTER_DOD.md)** - Definition of Done for project charter
   - 12-section acceptance checklist
   - Evidence requirements for each item
   - KGC invariants verification
   - Sign-off criteria for Measure phase entry
25. **[Reflex Enterprise Blueprint](REFLEX_ENTERPRISE_BLUEPRINT.md)** - Fortune-5 enterprise-grade specification
   - Runtime classes and SLOs (R1/W1/C1)
   - Multi-region, zero-trust topology
   - Performance engineering (AOT, MPHF, preloading)
   - Security, reliability, governance
   - ERP/CRM/ATS replacement path
   - Acceptance criteria and rollout plan

### Research & Evaluation
26. **[ggen RDF Research](ggen-rdf-research.md)** - Research on ggen RDF handling
27. **[ggen Integration Evaluation](ggen-integration-evaluation.md)** - Integration evaluation
28. **[ggen Integration Validation](ggen-integration-validation.md)** - Integration validation

### Specialized Topics
29. **[Definition of Done](DEFINITION_OF_DONE.md)** - DoD criteria
30. **[Documentation Gaps](DOCUMENTATION_GAPS.md)** - Documentation status and gaps (all READMEs complete, enhancements applied)
31. **[Documentation Organization](DOCUMENTATION_ORGANIZATION.md)** - Documentation structure guide
32. **[Unrdf Chicago TDD Validation](unrdf-chicago-tdd-validation.md)** - TDD validation results
33. **[Chicago TDD Verification Report](chicago-tdd-verification-report.md)** - Chicago TDD verification status
   - Runtime classes and SLOs implementation
   - Test coverage summary
   - Production-ready status
34. **[Validation Reports Index](VALIDATION_INDEX.md)** - Quick reference to active validation reports
   - Latest comprehensive reports
   - Specialized validation reports
   - Running validations guide
35. **[Performance Compliance Report](performance-compliance-report.md)** - 8-tick performance compliance verification
   - Tick budget enforcement (≤8 ticks)
   - SLO monitoring (p99 ≤2ns)
   - OTEL metrics integration
   - Performance compliance status
   - Failure actions on budget exceeded
35. **[False Positives Resolved](FALSE_POSITIVES_RESOLVED.md)** - Current state of false positives and unfinished work

## Archived Documentation

Historical and version-specific documentation is archived in `archived/`:

### Version-Specific
- `archived/v0.4.0/` - Version 0.4.0 documentation
- `archived/versions/` - Historical version docs (v0.2.0, v0.3.0, etc.)

### Status Reports
- `archived/status/` - Implementation status reports
  - Chicago TDD completion reports
  - False positives summaries
  - Capability validation reports
  - Orchestration final reports
  - Performance validation reports
  - Historical status documents

### Weaver Documentation
- `archived/weaver/` - Weaver learnings implementation summaries
  - Phase completion reports
  - Implementation summaries
  - Chicago TDD validation reports
  - Historical status documents

### Planning Documents
- `archived/planning/` - Project planning documents
  - Archive Revision Plan
  - Archive Summary
  - JIRA Tickets
  - Lean Six Sigma Project Charter

### Analysis & Implementation Details
- `archived/analysis/` - Analysis documents
- `archived/implementation-details/` - Detailed implementation docs
- `archived/implementation/` - Integration analysis and deliverables summaries

## Book Documentation

Structured book-formatted documentation is available in `book/src/`:
- Architecture chapters
- API references
- Integration guides
- CLI documentation

## Subproject Documentation

Each subproject has its own `docs/` directory with project-specific documentation:

### Rust Crates
Each crate has a README.md in its root directory with quick start and overview:
- **[knhk-warm](../rust/knhk-warm/docs/README.md)** - Warm path operations (oxigraph)
- **[knhk-hot](../rust/knhk-hot/README.md)** - Hot path operations (≤2ns) | [Technical Docs](../rust/knhk-hot/docs/README.md)
- **[knhk-etl](../rust/knhk-etl/README.md)** - ETL pipeline with Ingester pattern | [Technical Docs](../rust/knhk-etl/docs/README.md)
- **[knhk-cli](../rust/knhk-cli/docs/README.md)** - CLI interface
- **[knhk-unrdf](../rust/knhk-unrdf/docs/README.md)** - unrdf integration
- **[knhk-lockchain](../rust/knhk-lockchain/README.md)** - Lockchain | [Technical Docs](../rust/knhk-lockchain/docs/README.md)
- **[knhk-connectors](../rust/knhk-connectors/README.md)** - Data connectors | [Technical Docs](../rust/knhk-connectors/docs/README.md)
- **[knhk-otel](../rust/knhk-otel/README.md)** - OpenTelemetry | [Technical Docs](../rust/knhk-otel/docs/README.md)
- **[knhk-aot](../rust/knhk-aot/README.md)** - AOT compilation | [Technical Docs](../rust/knhk-aot/docs/README.md)
- **[knhk-config](../rust/knhk-config/docs/README.md)** - Configuration
- **[knhk-integration-tests](../rust/knhk-integration-tests/docs/README.md)** - Integration tests
- **[knhk-validation](../rust/knhk-validation/README.md)** - Validation with Advisor & Diagnostic patterns | [Technical Docs](../rust/knhk-validation/docs/README.md)

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
