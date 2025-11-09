# Documentation Organization

**Last Updated**: January 2025  
**Status**: Updated for 80/20 Consolidation

## Overview

This document describes the organization of KNHK documentation files following the 80/20 principle. Documentation has been consolidated into 80/20 guides covering 80% of use cases, with detailed reference docs available for edge cases.

## Active Documentation

### Consolidated Guides (80/20) - Primary Documentation
**These guides cover 80% of use cases and are the primary entry points:**

- **[WORKFLOW_ENGINE.md](WORKFLOW_ENGINE.md)** - Workflow engine guide
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture guide
- **[YAWL_INTEGRATION.md](YAWL_INTEGRATION.md)** - YAWL integration guide
- **[ONTOLOGY.md](ONTOLOGY.md)** - Ontology integration guide
- **[PERFORMANCE.md](PERFORMANCE.md)** - Performance guide
- **[PRODUCTION.md](PRODUCTION.md)** - Production readiness guide
- **[TESTING.md](TESTING.md)** - Testing guide
- **[API.md](API.md)** - API guide
- **[CLI.md](CLI.md)** - CLI guide
- **[INTEGRATION.md](INTEGRATION.md)** - Integration guide

### Essential (Indexed in INDEX.md)
- Core documentation for users and developers
- Getting started guides
- API references
- Architecture documentation (detailed reference)
- Integration guides

### Reference (Indexed in INDEX.md)
- Code organization
- Data flow diagrams
- Documentation gaps
- v1.0 planning documents
- Detailed implementation guides (for edge cases)

### Archived (Not in INDEX.md)
- Historical version docs
- Analysis documents
- Implementation details
- Status reports
- Project planning documents (LSS, Jira, DoD, Autonomic Implementation)
- Detailed YAWL docs (archived to `docs/archived/yawl/`)
- Redundant ontology docs (archived to `docs/archived/ontology-integration/`)

## File Organization

### Active Documentation Files
- `INDEX.md` - Main documentation index
- `QUICK_START.md` - Quick start guide
- `cli.md` - CLI documentation
- `architecture.md` - Architecture overview
- `api.md` - API reference
- `integration.md` - Integration guide
- `deployment.md` - Deployment guide
- `performance.md` - Performance docs
- `code-organization.md` - Code structure
- `data-flow.md` - Data flow diagrams
- `WEAVER.md` - Weaver integration (canonical)
- `DOCUMENTATION_GAPS.md` - Undocumented components
- `v0.4.0-status.md` - v0.4.0 status
- `v1-requirements.md` - v1.0 requirements
- `v1.0-unrdf-integration-plan.md` - unrdf integration plan
- `v1.0-unrdf-gap-analysis.md` - unrdf gap analysis

### Archived Documentation Files
Located in `docs/archived/`:
- **versions/** - Version-specific documentation
- **analysis/** - Analysis documents
- **status/** - Status reports
- **implementation-details/** - Implementation details

### Project Planning Documents (Not Indexed)
These documents exist but are not included in the main index as they are internal planning documents:
- `AUTONOMIC_IMPLEMENTATION.md` - Autonomic DoD implementation
- `CHICAGO_TDD_VALIDATION_AUTONOMIC.md` - Chicago TDD validation
- `CONCEPTUAL_TESTS_IMPLEMENTED.md` - Conceptual tests implementation
- `CONCEPTUAL_TESTS_IMPLEMENTATION_COMPLETE.md` - Implementation completion
- `DEFINITION_OF_DONE.md` - DoD criteria
- `FALSE_POSITIVES_FIXED.md` - False positives fixes
- `IMPLEMENTATION_COMPLETE.md` - Implementation completion summary
- `JIRA_TICKETS.md` - Jira ticket breakdown
- `LEAN_SIX_SIGMA_PROJECT_CHARTER.md` - LSS project charter
- `CONSOLIDATION_COMPLETE.md` - Consolidation summary
- `FINAL_STATUS.md` - Final status report
- `ARCHIVE_REVISION_PLAN.md` - Archive revision plan
- `ARCHIVE_SUMMARY.md` - Archive summary

### Other Documentation Files
- `unrdf-integration-dod.md` - unrdf integration DoD
- `unrdf-chicago-tdd-validation.md` - unrdf Chicago TDD validation
- `unrdf-integration-status.md` - unrdf integration status
- `cursor-rules-commands.md` - Cursor IDE rules
- `epistemology-generation.md` - Epistemology generation
- `autonomous-epistemology.md` - Autonomous epistemology
- `ggen-integration-evaluation.md` - GGen integration evaluation
- `ggen-rdf-research.md` - GGen RDF research

## Guidelines

### When to Index
- Essential user/developer documentation
- API references
- Architecture documentation
- Integration guides
- Getting started guides

### When NOT to Index
- Internal planning documents
- Historical documents
- Status reports
- Analysis documents
- Implementation details (unless essential)

### When to Archive
- Version-specific documentation (after version is released)
- Historical analysis documents
- Completed implementation details
- Status reports from past releases

---

**Maintained By**: Core Team  
**Purpose**: Maintain documentation organization consistency

