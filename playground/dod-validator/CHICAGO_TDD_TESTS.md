# Chicago TDD Tests for Autonomous DoD Validator

**Version**: v2.0  
**Methodology**: Chicago TDD (Classicist Approach)  
**Status**: Production Ready

## Overview

Comprehensive Chicago TDD test suite for the Autonomous DoD Validator, validating autonomics principles: **A = μ(O)**, **μ∘μ = μ**, **preserve(Q)**.

## Chicago TDD Principles Applied

### ✅ State-Based Tests, Not Interaction-Based

Tests verify **outputs and invariants**, not implementation details.

### ✅ Real Collaborators, No Mocks

Tests use **real KNHK components** - no mocks or stubs.

### ✅ Verify Outputs and Invariants

Tests verify autonomics loop, idempotence, invariant preservation, and receipt generation.

### ✅ OTEL Validation

Test results are truth source - receipts contain real span IDs.

See [tests/chicago_autonomous_dod_validator.c](tests/chicago_autonomous_dod_validator.c) and [rust/dod-validator-autonomous/src/chicago_tdd_tests.rs](rust/dod-validator-autonomous/src/chicago_tdd_tests.rs) for full test implementations.

## Running Tests

```bash
# C tests
cd playground/dod-validator
make -f Makefile.chicago test-chicago-autonomous-c

# Rust tests
cd playground/dod-validator/rust/dod-validator-autonomous
cargo test

# All tests
make -f Makefile.chicago test-chicago-autonomous
```

