# Cursor Configuration Integration Summary

This document summarizes the integration of cursor rules and commands from `ggen` and `clnrm` projects into KNHK.

## Integrated Best Practices

### From `clnrm/.cursorrules`:

1. **Async/Sync Patterns**
   - Never make trait methods async (breaks dyn compatibility)
   - Use async for I/O and long-running operations
   - Use sync for pure computation and simple operations
   - Proper use of `block_in_place` for async operations in sync trait methods

2. **Anti-False-Positive Rules**
   - Never fake implementation with `Ok(())` stubs
   - Incomplete implementations must call `unimplemented!()` with clear messages
   - No partial implementations that lie about success

3. **Testing Philosophy**
   - Test behaviors, not implementation details
   - Use AAA pattern (Arrange, Act, Assert)
   - Use descriptive test names
   - Prefer real collaborators over mocks

4. **Definition of Done Checklist**
   - Comprehensive checklist for production readiness
   - Validation criteria for all code changes

### From `ggen/.cursorrules`:

1. **Build System Practices**
   - Emphasis on consistent build commands
   - Determinism requirements
   - SLOs for build and runtime performance

2. **Determinism**
   - Same inputs → identical outputs
   - Fixed seeds for tests
   - Deterministic RNG usage

## Updated Files

### Rules Files:

1. **`.cursor/rules/80-20-best-practices.mdc`**
   - Added Async/Sync Best Practices section
   - Added Anti-False-Positive Rules section
   - Enhanced Testing Requirements with behavior-focused testing
   - Added comprehensive Definition of Done checklist
   - Updated Code Review Checklist with new items

2. **`.cursor/rules/rust-standards.mdc`**
   - Added Async/Sync Patterns section
   - Added Trait Design section (dyn compatibility)
   - Enhanced Code Organization with module structure guidelines
   - Enhanced Testing section with behavior-focused testing

3. **`.cursor/rules/build-system-practices.mdc`** (NEW)
   - Build command guidelines
   - Determinism requirements
   - SLOs for build and runtime performance
   - CI/CD practices
   - Development workflow guidelines

### Command Files:

1. **`.cursor/commands/code-review-checklist.md`**
   - Added checks for fake implementations
   - Added trait compatibility checks
   - Added behavior-focused testing checks
   - Added async/sync pattern checks
   - Added backward compatibility checks

2. **`.cursor/commands/check-fake-implementations.md`** (NEW)
   - Command to identify fake implementations
   - Patterns to search for
   - Guidelines for fixing fake implementations

3. **`.cursor/commands/check-trait-compatibility.md`** (NEW)
   - Command to verify dyn compatibility
   - Common issues and fixes
   - Pattern examples

4. **`.cursor/commands/validate-definition-of-done.md`** (NEW)
   - Command to validate Definition of Done criteria
   - Step-by-step validation process
   - Checklist for production readiness

5. **`.cursor/commands/aa-dflss-align.md`** (UPDATED)
   - Refactored to exploration-first approach
   - Requires reading 10+ files from `docs/v1/dflss/` before work
   - Added Mathematical Foundations section with vocabulary, laws, and calculus
   - Removed prescriptive implementation steps
   - Emphasizes context-driven discovery

6. **`.cursor/commands/aa-implement-spec.md`** (UPDATED)
   - Refactored to exploration-first approach
   - Requires reading 10+ files from `docs/v1/dflss/` before work
   - Added Mathematical Foundations section with vocabulary, laws, and calculus
   - Removed prescriptive gap identification steps
   - Emphasizes discovery through exploration

7. **`.cursor/commands/aa-validate-implementation.md`** (UPDATED)
   - Refactored to exploration-first approach
   - Requires reading 10+ files from `docs/v1/dflss/` before work
   - Added Mathematical Foundations section with vocabulary, laws, and calculus
   - Removed prescriptive validation code writing steps
   - Emphasizes discovery through exploration

## Key Principles Integrated

1. **Never Trust the Text, Only Trust Test Results**
   - OTEL validation is ultimate truth source
   - Test results > code comments > agent claims

2. **No Placeholders, Real Implementations**
   - All code must be production-ready
   - Incomplete features must call `unimplemented!()`
   - No fake `Ok(())` returns

3. **Trait Design for Compatibility**
   - Never use async trait methods
   - Keep traits `dyn` compatible
   - Use sync methods in traits, async in implementations

4. **Behavior-Focused Testing**
   - Test what code does, not how it does it
   - Use AAA pattern
   - Prefer real collaborators over mocks

5. **Determinism**
   - Same inputs → identical outputs
   - Fixed seeds for tests
   - Reproducible builds and tests

6. **Exploration-First DFLSS Commands**
   - Agents must read 10+ files from `docs/v1/dflss/` before starting work
   - No prescriptive file lists - agents choose files for genetic variety
   - Context-driven discovery rather than rigid workflows
   - Mathematical foundations guide understanding

7. **Mathematical Foundations**
   - Vocabulary: {O,A,μ,Σ,Λ,Π,τ,Q,Δ,Γ,⊕,⊔,≺,≤,=,⊨}
   - Core Law: A = μ(O) - Actions are measurements of Observations
   - Idempotence: μ∘μ = μ
   - Typing: O ⊨ Σ
   - ACHI Identity Principles: Source O only, Determinism, No partials, Closed world
   - Language constraints: Turtle + KGC only, avoid "semantic" and "self-" terminology
   - Systems controlled by KGC and LLMs, not humans

## Usage

### Review Code:
```bash
# Use cursor command: code-review-checklist
```

### Check for Fake Implementations:
```bash
# Use cursor command: check-fake-implementations
```

### Verify Trait Compatibility:
```bash
# Use cursor command: check-trait-compatibility
```

### Validate Definition of Done:
```bash
# Use cursor command: validate-definition-of-done
```

### DFLSS Documentation Alignment:
```bash
# Use cursor command: aa-dflss-align
# Explores dflss documentation, understands mathematical foundations, aligns code and docs
```

### Implement DFLSS Requirements:
```bash
# Use cursor command: aa-implement-spec
# Explores dflss documentation, understands mathematical foundations, implements requirements
```

### Validate Implementation Against DFLSS:
```bash
# Use cursor command: aa-validate-implementation
# Explores dflss documentation, understands mathematical foundations, validates implementation
```

## Integration Date

Integrated: 2025-01-27

Updated: 2025-01-27 (DFLSS command refactoring and mathematical foundations)

## Notes

- All rules maintain consistency with KNHK's existing 80/20 production-ready code standards
- Performance requirements (≤8 ticks) remain unchanged
- OTEL validation remains the ultimate truth source
- All patterns align with FAANG-level code quality standards
- DFLSS commands emphasize exploration and mathematical foundations over prescriptive steps
- Agents have genetic variety in file selection for exploration (no prescribed file lists)
- Mathematical vocabulary and laws guide understanding: A = μ(O), idempotence, typing, order, merge, sheaf, Van Kampen, shard, provenance, guard, epoch, sparsity, minimality, invariant
- Language constraints: Turtle + KGC only, avoid "semantic" and "self-" terminology
- Systems are controlled by KGC and LLMs, not humans

