# Tutorial 1: Your First KNHK Workflow

## Learning Objectives

By the end of this tutorial, you will:
- Understand the KNHK project structure and core concepts
- Set up your development environment
- Run your first telemetry-validated workflow
- Understand how Weaver validation works
- Know the difference between tests and schema validation

**Estimated Time**: 20-30 minutes
**Prerequisites**: Basic familiarity with command line, Rust, and Docker (optional)
**Difficulty**: Beginner

---

## Part 1: Understanding KNHK Fundamentals

### What is KNHK?

KNHK (Knowledge Hooks) is a framework that eliminates false positives in testing through **schema-first validation**. Instead of relying on traditional tests (which can pass when features don't actually work), KNHK validates that your code's telemetry matches its declared behavior.

### The Core Problem KNHK Solves

```
Traditional Testing Problem:
  Test passes ✅ → Code "works"? → MAYBE (false positive)
  └─ Tests validate test logic, not production behavior

KNHK Solution:
  Schema declares behavior → Weaver validates runtime telemetry ✅
  └─ Schema validation proves actual runtime behavior
```

### Key Concepts

**1. The Chatman Equation**
- Mathematical model for Fortune 500 optimization
- Foundation for KNHK's approach to performance
- ≤8 ticks per hot path (the "Chatman Constant")

**2. Knowledge Hooks (K-Hooks)**
- Points where your code emits telemetry
- Connect behavior to OpenTelemetry schemas
- Enable Weaver validation

**3. Weaver Validation**
- Validates that runtime telemetry matches schema
- Schema-first development approach
- The source of truth for feature validation

**4. The 43 Workflow Patterns**
- Proven patterns for production-ready code
- Reduce cognitive load through structure
- Enable systematic development

---

## Part 2: Environment Setup

### Prerequisites Installation

```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Cargo tools
cargo install cargo-edit
cargo install cargo-nextest  # Faster testing

# Clone the repository
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk

# Verify installation
cargo --version    # Should be 1.70+
rustc --version    # Should be 1.70+
make --version     # Should be 4.0+
```

### Verify Your Setup

```bash
# Run the setup verification
cargo build --workspace

# Check key tools
which cargo
which rustc
which make
```

✅ **Success**: All commands execute without errors

---

## Part 3: Understanding the Project Structure

### Key Directories

```
knhk/
├── src/                    # Source code (Rust implementation)
│   ├── lib.rs             # Main library
│   ├── bin/               # Binary implementations
│   └── ...
├── tests/                  # Test files
├── Cargo.toml             # Rust workspace configuration
├── Makefile               # Build automation
├── docs/
│   ├── papers/            # Research and documentation (Diátaxis)
│   └── ...
├── registry/              # OpenTelemetry schemas
└── README.md              # Project overview
```

### Understanding Configuration Files

**Cargo.toml**
- Defines Rust workspace and dependencies
- Specifies version information
- Contains build profiles

**Makefile**
- Automates build and test workflows
- Provides test commands (Chicago TDD)
- Performance testing commands

**CLAUDE.md**
- Development guidelines and standards
- SPARC methodology documentation
- Agent coordination patterns

---

## Part 4: Running Your First Workflow

### Step 1: Build the Project

```bash
# Navigate to project root
cd knhk

# Build everything
cargo build --workspace

# Expected output:
# Compiling knhk v0.1.0 (...)
# Finished dev [unoptimized + debuginfo] target(s) in Xs
```

✅ **Checkpoint**: Build completes with no errors

### Step 2: Run Unit Tests

```bash
# Run Rust tests
cargo test --workspace

# Expected: All tests pass
# Example output:
# test result: ok. XX passed; 0 failed; 0 ignored; 0 measured
```

✅ **Checkpoint**: Tests pass successfully

### Step 3: Run Chicago TDD Tests

```bash
# Run Chicago Test-Driven Development suite
make test-chicago-v04

# This tests:
# - Core functionality
# - Integration points
# - Chicago-style assertions
```

✅ **Checkpoint**: Chicago tests pass

### Step 4: Validate Performance

```bash
# Run performance tests
make test-performance-v04

# Verify ≤8 ticks constraint (Chatman Constant)
# Output shows:
# - Operation timing
# - Tick count
# - Performance results
```

✅ **Checkpoint**: Performance meets Chatman Constant (≤8 ticks)

---

## Part 5: Understanding Weaver Validation

### What is Weaver Validation?

Weaver is OpenTelemetry's schema validator. It ensures your code's telemetry matches its declared schema.

```bash
# Install Weaver (if needed)
# Weaver is typically installed separately
weaver --version
```

### Running Schema Validation

```bash
# Check schema validity
weaver registry check -r registry/

# Expected: Schema validation passes
```

### Live Validation (The Source of Truth)

```bash
# Validate runtime telemetry against schema
weaver registry live-check --registry registry/

# This proves:
# ✅ Code emits telemetry as declared
# ✅ Telemetry matches schema
# ✅ Feature actually works
```

**Important**: This validation is MORE important than traditional tests because it validates actual runtime behavior, not test code.

---

## Part 6: Your First Feature Development

### Create a Simple Feature

```bash
# Create a new test module
mkdir -p tests/tutorials

# Create tutorial test
cat > tests/tutorials/hello_knhk.rs << 'EOF'
#[test]
fn hello_knhk_works() {
    // Your test implementation
    assert!(true);
}
EOF
```

### Run Your Test

```bash
cargo test --test hello_knhk
```

### Add Telemetry

Your feature should emit OpenTelemetry signals:

```rust
use tracing::{info, instrument};

#[instrument]
fn my_feature() {
    info!("Feature executed");
    // Implementation
}
```

### Validate with Weaver

```bash
# Schema validation ensures telemetry matches declaration
weaver registry live-check --registry registry/
```

---

## Part 7: Understanding the Validation Hierarchy

KNHK uses a **three-tier validation hierarchy**:

### Tier 1: Weaver Schema Validation ⭐ (Source of Truth)
```bash
weaver registry check -r registry/              # Schema validity
weaver registry live-check --registry registry/ # Runtime telemetry
```
**What it validates**: Actual runtime behavior matches declared schema
**Why it's the source of truth**: Validates real telemetry, not test code

### Tier 2: Compilation & Code Quality (Baseline)
```bash
cargo build --release                           # Compilation
cargo clippy --workspace -- -D warnings         # Zero warnings
```
**What it validates**: Code is valid and follows best practices
**Why it matters**: Necessary but not sufficient

### Tier 3: Traditional Tests (Supporting Evidence)
```bash
cargo test --workspace                          # Unit tests
make test-chicago-v04                          # Integration tests
make test-performance-v04                       # Performance tests
```
**What it validates**: Test code behavior
**Why it's supporting evidence**: Tests can pass while features are broken

**⚠️ CRITICAL**: If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.

---

## Part 8: Troubleshooting

### Issue: Tests Pass But Weaver Fails

**Cause**: Tests don't validate actual telemetry
**Solution**:
- Review your telemetry emissions
- Update schema to match actual behavior
- Add proper instrumentation

### Issue: Build Errors

**Cause**: Missing dependencies or version conflicts
**Solution**:
```bash
cargo update
cargo clean
cargo build --workspace
```

### Issue: Performance Tests Fail

**Cause**: Operations exceed 8 ticks (Chatman Constant)
**Solution**:
- Profile the code with `perf` or `flamegraph`
- Optimize hot paths
- Review algorithm complexity

---

## Part 9: What You've Accomplished

### You Have:
- ✅ Set up KNHK development environment
- ✅ Built the project successfully
- ✅ Ran tests and validated performance
- ✅ Understood schema-first validation
- ✅ Learned the three-tier validation hierarchy
- ✅ Created your first test

### Next Steps

1. **Read Explanatory Docs**
   - [Chatman Equation explained](../explanation/the_chatman_equation_fortune5.md)
   - [Formal foundations](../explanation/formal-foundations.md)

2. **Try Tutorials**
   - Tutorial 2: Understanding Telemetry (coming soon)
   - Tutorial 3: Building Production Features (coming soon)

3. **Use How-to Guides**
   - [How to Debug Failing Tests](../how-to-guides/03-debug-failing-tests.md) (coming soon)
   - [How to Optimize Performance](../how-to-guides/11-optimize-performance.md) (coming soon)

4. **Reference Materials**
   - [Technical Papers](../reference/)
   - [Architecture Documentation](../reference/the_chatman_equation_fortune5_v1.2.0.pdf)

---

## Part 10: Common Questions

**Q: What's the difference between tests and Weaver validation?**
A: Tests validate test code. Weaver validates actual runtime telemetry against schema. Weaver is the source of truth.

**Q: What's the Chatman Constant?**
A: The ≤8 ticks performance constraint for hot path operations. This comes from the Chatman Equation's optimization model.

**Q: Do I need to understand all the math?**
A: No. Start with conceptual understanding (Explanation docs), then dive into formulas as needed (Reference docs).

**Q: What are Knowledge Hooks?**
A: Points where your code emits telemetry to enable validation. See the formal papers for detailed specification.

**Q: How do I add new features?**
A: Follow the pattern: Create test → Implement feature → Add telemetry → Validate with Weaver. See How-to Guides for details.

---

## Summary

You've completed your first KNHK workflow:

1. Set up development environment
2. Built project successfully
3. Ran tests and performance validation
4. Understood schema-first validation approach
5. Learned validation hierarchy

**Key Insight**: KNHK's strength is eliminating false positives through schema validation, not just traditional testing.

---

**Created**: 2025-11-15
**Level**: Beginner
**Status**: Complete
**Next**: [Tutorial 2: Understanding Telemetry](02-understanding-telemetry.md) (coming soon)
