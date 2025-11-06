# Weaver Learnings Implementation Plan

## Overview

Implementing key learnings from Weaver codebase to improve KNHK validation, error handling, and architecture.

## Phase 1: Policy Engine Integration (P0) - IN PROGRESS

### Goals
- Integrate Rego-based policy engine into `knhk-validation`
- Create policies for guard constraints (max_run_len ≤ 8)
- Create policies for performance validation (8-tick budget)
- Create policies for receipt validation

### Implementation Steps
1. Add `regorus` dependency to `knhk-validation`
2. Create policy engine wrapper similar to Weaver's `Engine`
3. Create default policies for guard constraints
4. Create default policies for performance validation
5. Create default policies for receipt validation
6. Integrate with existing validation framework

## Phase 2: Error Diagnostics (P1) - PENDING

### Goals
- Adopt structured diagnostics with context
- JSON output for CI/CD
- Better error messages with OTEL integration

## Phase 3: Schema Resolution (P1) - PENDING

### Goals
- Implement resolved schema pattern for RDF schemas
- Version management and dependencies
- Schema catalog

## Phase 4: Streaming Processing (P2) - PENDING

### Goals
- Streaming ingesters for RDF
- Real-time pipeline execution
- Streaming validation

