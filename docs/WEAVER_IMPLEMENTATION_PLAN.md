# Weaver-Inspired Implementation Plan for KNHK

## Overview

This document outlines the implementation plan for adopting Weaver's architectural patterns and design principles in KNHK, prioritized by impact and effort.

## Phase 1: Policy Engine Integration (P0 - High Priority)

### Goal
Integrate Rego-based policy engine for custom validation rules, applying to guard constraints, performance budgets, and receipt validation.

### Tasks

#### 1.1: Add Rego Policy Engine Dependency
- [ ] Add `opaque-rego` or `regorus` crate to `knhk-validation/Cargo.toml`
- [ ] Create feature flag `rego-policies` for optional policy engine
- [ ] Document policy engine requirements

#### 1.2: Create Policy Advisor Trait
- [ ] Define `PolicyAdvisor` trait in `knhk-validation/src/policy.rs`
- [ ] Implement built-in advisors:
  - `GuardConstraintAdvisor` (max_run_len ≤ 8)
  - `PerformanceBudgetAdvisor` (8-tick budget)
  - `ReceiptValidationAdvisor` (hash verification)
- [ ] Support custom Rego policies via file/string

#### 1.3: Integrate with Existing Validation
- [ ] Update `knhk-validation` to use policy advisors
- [ ] Add policy evaluation to guard constraint checks
- [ ] Add policy evaluation to performance validation
- [ ] Add policy evaluation to receipt validation

#### 1.4: Default Policies
- [ ] Create `policies/` directory in `knhk-validation`
- [ ] Add default Rego policies:
  - `guard_constraints.rego`
  - `performance_budget.rego`
  - `receipt_validation.rego`
- [ ] Embed policies using `include_str!` macro

#### 1.5: Testing
- [ ] Unit tests for each advisor
- [ ] Integration tests with custom Rego policies
- [ ] Chicago TDD validation tests

**Estimated Effort:** 2-3 weeks
**Dependencies:** None
**Impact:** High - Enables flexible, policy-driven validation

---

## Phase 2: Error Diagnostics (P1 - High Priority)

### Goal
Adopt structured diagnostics with context, JSON output for CI/CD, and better error messages with OTEL integration.

### Tasks

#### 2.1: Add Diagnostic Infrastructure
- [ ] Add `miette` crate to `knhk-etl` and `knhk-validation`
- [ ] Create `knhk-diagnostics` crate (or add to `knhk-otel`)
- [ ] Define `DiagnosticMessage` struct with:
  - Error code
  - Message
  - Context (key-value pairs)
  - Source location
  - OTEL span ID

#### 2.2: Update Error Types
- [ ] Add `#[non_exhaustive]` to error enums for extensibility
- [ ] Implement `Diagnostic` trait for all error types
- [ ] Add context methods to error types
- [ ] Integrate with OTEL spans

#### 2.3: Diagnostic Output Formats
- [ ] Implement ANSI output (default, using miette)
- [ ] Implement JSON output for CI/CD
- [ ] Add `--format` CLI flag (ansi, json)
- [ ] Add `--no-color` flag for CI environments

#### 2.4: Error Context Collection
- [ ] Add context collection to pipeline stages
- [ ] Add context collection to validation
- [ ] Add context collection to connector errors
- [ ] Integrate with OTEL spans for error tracking

#### 2.5: Testing
- [ ] Unit tests for diagnostic formatting
- [ ] Integration tests for error context
- [ ] CI/CD tests with JSON output

**Estimated Effort:** 2-3 weeks
**Dependencies:** None
**Impact:** High - Better developer experience and CI/CD integration

---

## Phase 3: Schema Resolution (P1 - High Priority)

### Goal
Implement resolved schema pattern for RDF schemas with version management, dependency tracking, and schema catalog.

### Tasks

#### 3.1: Schema Resolution Infrastructure
- [ ] Create `knhk-schema-resolver` crate
- [ ] Define `ResolvedSchema` struct:
  - Catalog (all schemas)
  - Registry (versioned schemas)
  - Dependencies (dependency graph)
- [ ] Implement schema resolution algorithm

#### 3.2: Schema Versioning
- [ ] Add version tracking to schema definitions
- [ ] Implement version compatibility checking
- [ ] Add dependency resolution
- [ ] Support semantic versioning

#### 3.3: Schema Catalog
- [ ] Create schema catalog structure
- [ ] Implement catalog loading from files/registry
- [ ] Add catalog query API
- [ ] Support schema registration

#### 3.4: Hook Registry Integration
- [ ] Integrate schema resolution with hook registry
- [ ] Add version management to hooks
- [ ] Implement dependency resolution for hooks
- [ ] Add version compatibility checks

#### 3.5: Testing
- [ ] Unit tests for schema resolution
- [ ] Integration tests for version management
- [ ] Tests for dependency resolution

**Estimated Effort:** 3-4 weeks
**Dependencies:** None
**Impact:** High - Better schema management and versioning

---

## Phase 4: Streaming Processing (P2 - Medium Priority)

### Goal
Implement streaming ingesters for RDF parsing, real-time pipeline execution, and streaming validation.

### Tasks

#### 4.1: Streaming Ingester Trait
- [ ] Create `Ingester` trait in `knhk-etl`
- [ ] Define streaming methods:
  - `ingest_stream()` for async streaming
  - `ingest_chunk()` for chunked processing
- [ ] Support multiple input types (file, stdin, network)

#### 4.2: RDF Streaming Parser
- [ ] Implement streaming RDF/Turtle parser
- [ ] Support incremental parsing
- [ ] Handle large files efficiently
- [ ] Add backpressure support

#### 4.3: Streaming Pipeline Execution
- [ ] Update pipeline to support streaming execution
- [ ] Add streaming validation
- [ ] Implement streaming receipt generation
- [ ] Add progress reporting

#### 4.4: Streaming Validation
- [ ] Add streaming support to validation
- [ ] Implement incremental validation
- [ ] Add streaming policy evaluation
- [ ] Support real-time validation

#### 4.5: Testing
- [ ] Unit tests for streaming ingesters
- [ ] Integration tests for streaming pipeline
- [ ] Performance tests for large files

**Estimated Effort:** 4-5 weeks
**Dependencies:** None
**Impact:** Medium - Better performance for large datasets

---

## Phase 5: CLI Improvements (P2 - Medium Priority)

### Goal
Better subcommand organization, structured diagnostic output, and format options.

### Tasks

#### 5.1: CLI Reorganization
- [ ] Reorganize CLI subcommands:
  - `knhk registry check` (schema validation)
  - `knhk registry live-check` (Weaver integration)
  - `knhk pipeline execute` (ETL execution)
- [ ] Use clap derive macros consistently
- [ ] Add flattened args where appropriate

#### 5.2: Diagnostic Output
- [ ] Integrate miette diagnostics
- [ ] Add `--format` flag (ansi, json)
- [ ] Add `--no-color` flag
- [ ] Improve error messages with context

#### 5.3: Structured Validation Reports
- [ ] Add validation report generation
- [ ] Support JSON output for reports
- [ ] Add summary statistics
- [ ] Integrate with CI/CD

#### 5.4: Testing
- [ ] Unit tests for CLI commands
- [ ] Integration tests for output formats
- [ ] CI/CD tests with JSON output

**Estimated Effort:** 1-2 weeks
**Dependencies:** Phase 2 (Error Diagnostics)
**Impact:** Medium - Better CLI experience

---

## Phase 6: Template Engine Enhancement (P3 - Low Priority)

### Goal
Improve AOT template engine with Jinja2-like features, embedded default templates, and better code generation.

### Tasks

#### 6.1: Template Engine Improvements
- [ ] Evaluate template engine options (Tera, Handlebars, etc.)
- [ ] Add template inheritance
- [ ] Add template includes
- [ ] Add template filters/functions

#### 6.2: Embedded Templates
- [ ] Use `include_dir!` macro for default templates
- [ ] Ship default templates with binary
- [ ] Support template override via CLI/config
- [ ] Document template system

#### 6.3: Code Generation
- [ ] Improve AOT code generation
- [ ] Add template-based documentation generation
- [ ] Support client SDK generation
- [ ] Add template validation

#### 6.4: Testing
- [ ] Unit tests for template engine
- [ ] Integration tests for code generation
- [ ] Tests for embedded templates

**Estimated Effort:** 2-3 weeks
**Dependencies:** None
**Impact:** Low - Nice to have, but not critical

---

## Implementation Timeline

### Q1 2024
- Phase 1: Policy Engine Integration (Weeks 1-3)
- Phase 2: Error Diagnostics (Weeks 4-6)

### Q2 2024
- Phase 3: Schema Resolution (Weeks 1-4)
- Phase 4: Streaming Processing (Weeks 5-9)

### Q3 2024
- Phase 5: CLI Improvements (Weeks 1-2)
- Phase 6: Template Engine Enhancement (Weeks 3-5)

## Success Metrics

### Phase 1: Policy Engine
- [ ] All guard constraints validated via policies
- [ ] Performance budgets enforced via policies
- [ ] Receipt validation via policies
- [ ] Custom Rego policies supported

### Phase 2: Error Diagnostics
- [ ] All errors have structured context
- [ ] JSON output works for CI/CD
- [ ] OTEL span integration for errors
- [ ] Developer satisfaction improved

### Phase 3: Schema Resolution
- [ ] Schema versioning implemented
- [ ] Dependency resolution working
- [ ] Schema catalog functional
- [ ] Hook registry integrated

### Phase 4: Streaming Processing
- [ ] Streaming RDF parser working
- [ ] Streaming pipeline execution
- [ ] Streaming validation functional
- [ ] Performance improved for large files

### Phase 5: CLI Improvements
- [ ] CLI reorganized
- [ ] Diagnostic output improved
- [ ] JSON output for CI/CD
- [ ] User satisfaction improved

### Phase 6: Template Engine
- [ ] Template engine enhanced
- [ ] Embedded templates working
- [ ] Code generation improved
- [ ] Documentation generation working

## Notes

- All phases should follow Chicago TDD methodology
- All code must be production-ready (no placeholders)
- All features must be feature-gated for `no_std` compatibility
- All changes must maintain backward compatibility where possible
- All implementations must include comprehensive tests

