# KNHK Documentation Index

**Last Updated:** 2025-01-XX  
**Status:** Consolidated (80/20) - 61 files → 45 active docs (26% reduction)

---

## Quick Start

- **[README](../README.md)** - Project overview and quick start
- **[QUICK_START.md](QUICK_START.md)** - Getting started guide
- **[v1.0 Definition of Done](v1.0-definition-of-done.md)** - Complete acceptance criteria

---

## Core Documentation (80/20 Guides)

**📖 Consolidated guides covering 80% of use cases:**

- **[WORKFLOW_ENGINE.md](WORKFLOW_ENGINE.md)** - Workflow engine guide (Quick Start, Core API, Critical Patterns)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture guide (Hot/Warm/Cold paths, Core Components)
- **[YAWL_INTEGRATION.md](YAWL_INTEGRATION.md)** - YAWL integration guide (Status, Critical Gaps, Quick Reference)
- **[ONTOLOGY.md](ONTOLOGY.md)** - Ontology integration guide (Integration Patterns, Common Operations)
- **[PERFORMANCE.md](PERFORMANCE.md)** - Performance guide (Hot Path ≤8 ticks, Benchmarks, Optimization)
- **[PRODUCTION.md](PRODUCTION.md)** - Production readiness guide (Status, Deployment, Troubleshooting)
- **[TESTING.md](TESTING.md)** - Testing guide (Chicago TDD, Validation, Test Coverage)
- **[API.md](API.md)** - 🆕 API guide (C, Rust, Erlang APIs - 80% use cases)
- **[CLI.md](CLI.md)** - 🆕 CLI guide (Command reference - 80% use cases)
- **[INTEGRATION.md](INTEGRATION.md)** - 🆕 Integration guide (Integration patterns - 80% use cases)

---

## Core Documentation (20% - 80% Value)

### Requirements & Definition of Done
- **[v1.0 Requirements](v1-requirements.md)** - Complete v1.0 requirements specification
- **[v1.0 Definition of Done](v1.0-definition-of-done.md)** - Acceptance criteria and validation process
- **[Validation Script](../scripts/validate-v1-dod.sh)** - Automated DoD validation

### Architecture
- **[Architecture Guide](ARCHITECTURE.md)** - 🆕 Consolidated 80/20 guide (Hot/Warm/Cold paths, Core Components)
- **[Architecture Overview](archived/reference-docs/architecture.md) (archived - consolidated)** - Detailed architecture reference
- **[8-Beat PRD](8BEAT-PRD.txt)** - 8-beat epoch system requirements
- **[8-Beat Integration Complete](8BEAT-INTEGRATION-COMPLETE.md)** - C/Rust integration status
- **[Branchless C Engine](BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)** - Hot path implementation

### Integration Guides
- **[Integration Guide](INTEGRATION.md)** - 🆕 Consolidated 80/20 guide (Integration patterns - 80% use cases)
- **[Integration Guide](archived/reference-docs/integration-guide.md) (archived - consolidated)** - Detailed integration reference
- **[Weaver Integration](WEAVER.md)** - OpenTelemetry Weaver integration
- **[unrdf Integration](v1.0-unrdf-integration-plan.md)** - Cold path integration

### Implementation
- **[Testing Guide](TESTING.md)** - 🆕 Consolidated 80/20 guide (Chicago TDD, Validation, Test Coverage)
- **[Chicago TDD](archived/consolidation/CHICAGO_TDD.md)** - Detailed testing methodology (archived - consolidated)
- **[Lockchain Implementation](LOCKCHAIN_INTEGRATION_COMPLETE.md)** - Provenance system
- **[OTEL Instrumentation](OTEL_INSTRUMENTATION_SUMMARY.md)** - Observability
- **[PMU Implementation](PMU_IMPLEMENTATION_SUMMARY.md)** - Performance measurement
- **[Performance Guide](PERFORMANCE.md)** - 🆕 Consolidated 80/20 guide (Hot Path ≤8 ticks, Benchmarks)
- **[YAWL Integration Guide](YAWL_INTEGRATION.md)** - 🆕 Consolidated 80/20 guide (Status, Critical Gaps)
- **[Ontology Guide](ONTOLOGY.md)** - 🆕 Consolidated 80/20 guide (Integration Patterns, Common Operations)

### Product
- **[Reflex Enterprise Blueprint](REFLEX_ENTERPRISE_BLUEPRINT.md)** - Product architecture (canonical)
- **[Reflex Enterprise Press Release](REFLEX_ENTERPRISE_PRESS_RELEASE.md)** - Product announcement

---

## Reference Documentation

### API Reference
- **[API Guide](API.md)** - 🆕 Consolidated 80/20 guide (C, Rust, Erlang APIs - 80% use cases)
- **[API Documentation](archived/reference-docs/api.md) (archived - consolidated)** - Complete API reference with all functions
- **[CLI Guide](CLI.md)** - 🆕 Consolidated 80/20 guide (Command reference - 80% use cases)
- **[CLI Documentation](archived/reference-docs/cli.md) (archived - consolidated)** - Complete CLI command reference
- **[Configuration](configuration.md)** - Configuration options

### Development
- **[Code Organization](code-organization.md)** - Codebase structure
- **[Testing Guide](archived/reference-docs/testing.md) (archived - consolidated)** - Testing practices
- **[Performance](archived/reference-docs/performance.md) (archived - consolidated)** - Performance considerations
- **[Deployment](deployment.md)** - Deployment guide

### Formal Foundations
- **[Formal Foundations](formal-foundations.md)** - Mathematical foundations
- **[Autonomous Epistemology](autonomous-epistemology.md)** - Knowledge representation

---

## Evidence & Reports

All validation reports, test results, and evidence are in:
- **[docs/evidence/](evidence/)** - Test results, validation reports, benchmarks

Key evidence files:
- `evidence/8BEAT_INTEGRATION_SYNTHESIS.md` - Integration analysis
- `evidence/architect_8beat_gaps.md` - Gap analysis
- `evidence/V1_*` - Various v1.0 validation reports

---

## Status & Current State

- **[Production Guide](PRODUCTION.md)** - 🆕 Consolidated 80/20 guide (Status, Deployment, Troubleshooting)
- **[V1 Release Status](V1-RELEASE-STATUS.md)** - Current v1.0 release status and validation progress
- **[V1 Status](V1-STATUS.md)** - Historical v1.0 development status
- **[DFLSS Definition of Done](DFLSS_DEFINITION_OF_DONE.spr.md)** - DFLSS DoD specification

## Release Documentation

- **[Release Notes v1.0.0](RELEASE_NOTES_v1.0.0.md)** - Comprehensive v1.0.0 release notes
- **[Release Checklist](V1-RELEASE-CHECKLIST.md)** - Pre-release validation checklist
- **[Release Status](V1-RELEASE-STATUS.md)** - Current release readiness status
- **[Changelog](CHANGELOG.md)** - Complete version history

## Archived Documentation

Historical and duplicate documentation has been archived following 80/20 consolidation:
- **[docs/archived/status-reports/](archived/status-reports/)** - Historical status and completion reports
- **[docs/archived/implementation-summaries/](archived/implementation-summaries/)** - Completed implementation summaries
- **[docs/archived/planning/](archived/planning/)** - Planning documents and charters
  - `archived/planning/internal/` - Internal planning and analysis docs
- **[docs/archived/research/](archived/research/)** - Research and evaluation documents
- **[docs/archived/weaver-docs/](archived/weaver-docs/)** - Consolidated Weaver documentation
- **[docs/archived/product/](archived/product/)** - Product documentation variants
- **[docs/archived/v1-dod/](archived/v1-dod/)** - Old DoD documents
- **[docs/archived/v1-reports/](archived/v1-reports/)** - Historical status reports
- **[docs/archived/integration/](archived/integration/)** - Superseded integration docs

See [ARCHIVE_INDEX.md](ARCHIVE_INDEX.md) for complete archive catalog.

---

## Documentation Structure

```
docs/
├── INDEX.md (this file)
├── v1.0-definition-of-done.md (canonical DoD)
├── v1-requirements.md (canonical requirements)
├── 8BEAT-INTEGRATION-COMPLETE.md (integration status)
├── architecture.md (system architecture)
├── integration-guide.md (integration guide)
├── evidence/ (validation reports, test results)
└── archived/ (historical/duplicate docs)
    ├── v1-dod/
    ├── v1-reports/
    └── integration/
```

---

## Finding Documentation

- **Getting Started**: See [QUICK_START.md](QUICK_START.md)
- **v1.0 Release**: See [v1.0-definition-of-done.md](v1.0-definition-of-done.md)
- **Architecture**: See [architecture.md](archived/reference-docs/architecture.md) (archived - consolidated) and [8BEAT-PRD.txt](8BEAT-PRD.txt)
- **Integration**: See [integration-guide.md](archived/reference-docs/integration-guide.md) (archived - consolidated)
- **Validation**: Run `./scripts/validate-v1-dod.sh`

---

**Note**: This index follows the 80/20 principle - focusing on the 20% of documentation that provides 80% of the value. Detailed reports and evidence are in the `evidence/` directory.
