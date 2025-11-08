# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the KNHK project. ADRs document **significant architectural decisions**, their **context**, **alternatives considered**, and **consequences**.

---

## What is an ADR?

An **Architecture Decision Record** is a document that captures an important architectural decision along with its context and consequences. ADRs help:

- **Document reasoning**: Why this decision was made
- **Preserve knowledge**: Decisions survive team changes
- **Guide future work**: Understand constraints and trade-offs
- **Avoid revisiting**: Documented alternatives prevent rehashing

**Format**: Each ADR follows a standard structure:
1. **Context**: Problem statement and requirements
2. **Decision**: Chosen approach and rationale
3. **Consequences**: Positive, negative, and neutral impacts
4. **Alternatives Considered**: Other options and why they were rejected
5. **Implementation Details**: Technical specifics
6. **References**: Related decisions and external resources

---

## ADR Index

### Core Performance (v1.0.0)

| ADR | Title | Status | Date | Category |
|-----|-------|--------|------|----------|
| [ADR-001](ADR-001-buffer-pooling-strategy.md) | Buffer Pooling Strategy for Zero-Allocation Hot Path | Accepted | 2025-11-08 | Performance / Architecture |
| [ADR-002](ADR-002-simd-implementation-approach.md) | SIMD Implementation Approach for Cross-Platform Acceleration | Accepted | 2025-11-08 | Performance / Architecture |

### Quality Assurance (v1.0.0)

| ADR | Title | Status | Date | Category |
|-----|-------|--------|------|----------|
| [ADR-003](ADR-003-weaver-validation-source-of-truth.md) | Weaver OTEL Validation as Source of Truth | Accepted | 2025-11-08 | Quality Assurance / Architecture |
| [ADR-004](ADR-004-chicago-tdd-methodology.md) | Chicago TDD Methodology for Behavior-Focused Testing | Accepted | 2025-11-08 | Testing / Quality Assurance |

---

## Decision Status

**Accepted**: Decision is approved and implemented
**Proposed**: Decision is under review
**Rejected**: Decision was considered but not adopted
**Deprecated**: Decision was once valid but is now superseded
**Superseded**: Decision is replaced by a newer ADR

---

## Quick Reference

### By Category

**Performance**:
- ADR-001: Buffer Pooling Strategy
- ADR-002: SIMD Implementation Approach

**Quality Assurance**:
- ADR-003: Weaver OTEL Validation
- ADR-004: Chicago TDD Methodology

### By Impact

**Critical (Affects Core Architecture)**:
- ADR-001: Buffer Pooling (zero-allocation hot path)
- ADR-003: Weaver Validation (eliminates false positives)

**Important (Affects Development Workflow)**:
- ADR-002: SIMD Implementation (2-4x performance improvement)
- ADR-004: Chicago TDD (behavior-focused testing)

### By Dependencies

**ADR-001** (Buffer Pooling):
- Used by: ADR-002 (SIMD operations use pooled buffers)
- Related to: ADR-003 (pool metrics validated by Weaver)

**ADR-002** (SIMD Implementation):
- Depends on: ADR-001 (uses buffer pools for SIMD padding)
- Related to: ADR-003 (SIMD latency validated by Weaver)

**ADR-003** (Weaver Validation):
- Validates: ADR-001 (buffer pool telemetry), ADR-002 (SIMD latency)
- Complements: ADR-004 (Chicago TDD tests behavior, Weaver validates runtime)

**ADR-004** (Chicago TDD):
- Complements: ADR-003 (TDD validates logic, Weaver validates behavior)
- Used by: All testing (ADR-001, ADR-002 tested via Chicago TDD)

---

## Decision Relationships

```
                    ┌───────────────────────────┐
                    │     ADR-003: Weaver       │
                    │  (Source of Truth)        │
                    │  Validates ALL runtime    │
                    │  behavior                 │
                    └─────────┬─────────────────┘
                              │
                    Validates ↓ Telemetry
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ↓                     ↓                     ↓
┌───────────────┐   ┌─────────────────┐   ┌──────────────────┐
│  ADR-001:     │   │  ADR-002:       │   │  ADR-004:        │
│  Buffer       │──→│  SIMD           │   │  Chicago TDD     │
│  Pooling      │   │  Implementation │   │  (Test Logic)    │
└───────────────┘   └─────────────────┘   └──────────────────┘
      │                     ↑                       │
      │                     │                       │
      └─ Provides memory ───┘                       │
                                                    │
                                   Tests all features
```

**Key Insight**: ADR-003 (Weaver) is the **validation hub**. All other decisions are validated by their runtime telemetry.

---

## Creating a New ADR

### When to Create an ADR

Create an ADR when:
- Making a **significant architectural decision** (affects multiple packages)
- Choosing between **multiple viable approaches** (trade-offs involved)
- Decision has **long-term consequences** (hard to reverse)
- Decision **constrains future work** (establishes patterns)

**Examples**:
- ✅ Choosing buffer pooling over arena allocation (ADR-001)
- ✅ Adopting Weaver validation over custom tooling (ADR-003)
- ❌ Changing function name (not architectural)
- ❌ Fixing bug (not a decision)

### ADR Template

```markdown
# ADR-NNN: Title (Brief Description)

**Status**: Proposed | Accepted | Rejected | Deprecated | Superseded
**Date**: YYYY-MM-DD
**Decision Makers**: [Team or individuals]
**Category**: [Performance | Architecture | Testing | etc.]

---

## Context

[Problem statement, requirements, constraints]

## Decision

[Chosen approach and rationale]

## Consequences

### Positive
[Benefits of this decision]

### Negative
[Drawbacks and risks]

### Neutral
[Trade-offs without clear good/bad]

## Alternatives Considered

### Alternative 1: [Name]
[Approach, pros, cons, decision]

## Implementation Details

[Technical specifics, code examples]

## References

[Related ADRs, external resources]

## Review & Approval

**Proposed**: [Date]
**Reviewed**: [Date]
**Approved**: [Date]

**Validation**:
[How was this decision validated?]

**Next Review**: [When to revisit]

---

**Document Version**: 1.0
**Last Updated**: YYYY-MM-DD
```

### ADR Numbering

- **ADR-001 to ADR-099**: Core architecture (v1.x)
- **ADR-100 to ADR-199**: Performance optimizations
- **ADR-200 to ADR-299**: Testing & validation
- **ADR-300 to ADR-399**: Infrastructure & tooling
- **ADR-400+**: Future major decisions

---

## Maintenance

### Updating an ADR

**When to Update**:
- Clarification needed (context unclear)
- Implementation details changed (consequences same)
- References outdated (link rot)

**How to Update**:
1. Increment document version (e.g., 1.0 → 1.1)
2. Update "Last Updated" date
3. Add note at top: "Updated: [what changed]"
4. Do NOT change decision or status (create new ADR if decision changes)

### Superseding an ADR

**When to Supersede**:
- New decision replaces old decision
- Better approach found
- Requirements changed significantly

**How to Supersede**:
1. Create new ADR (ADR-NNN-new-approach.md)
2. Reference old ADR in "Context" section
3. Update old ADR:
   - Status: Superseded
   - Add note: "Superseded by ADR-NNN on YYYY-MM-DD"
   - Do NOT delete old ADR (preserve history)

---

## Reading Guide

### For New Team Members

**Start here** (understand core decisions):
1. ADR-003: Weaver Validation (why we validate this way)
2. ADR-001: Buffer Pooling (why hot path is fast)
3. ADR-004: Chicago TDD (how we write tests)
4. ADR-002: SIMD Implementation (why we use SIMD)

### For Contributors

**Read before**:
- **Adding features**: ADR-003 (Weaver validation required)
- **Optimizing performance**: ADR-001, ADR-002
- **Writing tests**: ADR-004 (Chicago TDD)

### For Architects

**Review all ADRs** to understand:
- Architectural constraints (what's hard to change)
- Design patterns (buffer pooling, SIMD abstraction)
- Validation approach (Weaver as source of truth)
- Testing philosophy (Chicago TDD)

---

## External References

### ADR Best Practices

- **Michael Nygard's ADR Template**: http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions
- **ADR GitHub Organization**: https://adr.github.io/
- **"Documenting Architecture Decisions"** by Michael Nygard

### KNHK-Specific Context

- **simdjson Lessons**: https://github.com/simdjson/simdjson/blob/master/doc/basics.md
- **OpenTelemetry Weaver**: https://github.com/open-telemetry/weaver
- **Chicago TDD (GOOS)**: https://www.growing-object-oriented-software.com/

---

## Questions?

**For ADR process questions**:
- Review this README
- Check existing ADRs for examples
- Ask in #architecture channel (if applicable)

**For technical questions**:
- See "References" section in each ADR
- Review implementation details
- Check related ADRs

---

**ADR Index Version**: 1.0
**Last Updated**: 2025-11-08
**Total ADRs**: 4 (all Accepted)
