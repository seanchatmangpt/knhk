# KNHK Documentation Index

**Last Updated:** 2025-01-XX  
**Status:** Consolidated (80/20)

---

## Quick Start

- **[README](../README.md)** - Project overview and quick start
- **[QUICK_START.md](QUICK_START.md)** - Getting started guide
- **[v1.0 Definition of Done](v1.0-definition-of-done.md)** - Complete acceptance criteria

---

## Core Documentation (20% - 80% Value)

### Requirements & Definition of Done
- **[v1.0 Requirements](v1-requirements.md)** - Complete v1.0 requirements specification
- **[v1.0 Definition of Done](v1.0-definition-of-done.md)** - Acceptance criteria and validation process
- **[Validation Script](../scripts/validate-v1-dod.sh)** - Automated DoD validation

### Architecture
- **[Architecture Overview](architecture.md)** - System architecture
- **[8-Beat PRD](8BEAT-PRD.txt)** - 8-beat epoch system requirements
- **[8-Beat Integration Complete](8BEAT-INTEGRATION-COMPLETE.md)** - C/Rust integration status
- **[Branchless C Engine](BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)** - Hot path implementation

### Integration Guides
- **[Integration Guide](integration-guide.md)** - How to integrate with KNHK
- **[Weaver Integration](WEAVER.md)** - OpenTelemetry Weaver integration
- **[unrdf Integration](v1.0-unrdf-integration-plan.md)** - Cold path integration

### Implementation
- **[Chicago TDD](CHICAGO_TDD.md)** - Testing methodology
- **[Lockchain Implementation](LOCKCHAIN_INTEGRATION_COMPLETE.md)** - Provenance system
- **[OTEL Instrumentation](OTEL_INSTRUMENTATION_SUMMARY.md)** - Observability
- **[PMU Implementation](PMU_IMPLEMENTATION_SUMMARY.md)** - Performance measurement

### Product
- **[Reflex Enterprise Blueprint](REFLEX_ENTERPRISE_BLUEPRINT.md)** - Product architecture (canonical)
- **[Reflex Enterprise Press Release](REFLEX_ENTERPRISE_PRESS_RELEASE.md)** - Product announcement

---

## Reference Documentation

### API Reference
- **[API Documentation](api.md)** - Public API reference
- **[CLI Documentation](cli.md)** - Command-line interface
- **[Configuration](configuration.md)** - Configuration options

### Development
- **[Code Organization](code-organization.md)** - Codebase structure
- **[Testing Guide](testing.md)** - Testing practices
- **[Performance](performance.md)** - Performance considerations
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

- **[V1 Status](V1-STATUS.md)** - Current v1.0 status (canonical status document)
- **[DFLSS Definition of Done](DFLSS_DEFINITION_OF_DONE.spr.md)** - DFLSS DoD specification

## Archived Documentation

Historical and duplicate documentation has been archived:
- **[docs/archived/status-reports/](archived/status-reports/)** - Historical status and completion reports
- **[docs/archived/planning/](archived/planning/)** - Planning documents and charters
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
- **Architecture**: See [architecture.md](architecture.md) and [8BEAT-PRD.txt](8BEAT-PRD.txt)
- **Integration**: See [integration-guide.md](integration-guide.md)
- **Validation**: Run `./scripts/validate-v1-dod.sh`

---

**Note**: This index follows the 80/20 principle - focusing on the 20% of documentation that provides 80% of the value. Detailed reports and evidence are in the `evidence/` directory.
