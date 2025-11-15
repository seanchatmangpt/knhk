# Tutorial: Getting Started with KNHK

**Level**: Beginner
**Time**: 15-20 minutes
**Learning Objectives**: Understand KNHK basics and run your first workflow

## What You'll Learn

By the end of this tutorial, you'll be able to:
- Understand what KNHK does and its core concepts
- Set up KNHK in your development environment
- Run your first test to verify the setup
- Understand how KNHK validates telemetry

## Prerequisites

- Git installed
- Rust toolchain (1.70+)
- Basic command-line familiarity
- 2GB free disk space

## Step 1: Clone the Repository

Start by getting the KNHK source code:

```bash
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk
```

Verify the directory structure:
```bash
ls -la
```

You should see:
- `src/` - Rust source code
- `tests/` - Test files
- `docs/` - Documentation (including this tutorial!)
- `Cargo.toml` - Rust project manifest
- `Makefile` - Build targets

## Step 2: Review the Chatman Equation Concept

Before diving into code, read the conceptual overview:

1. Open `docs/papers/explanation/the_chatman_equation_fortune5.md`
2. Read the first section to understand the core equation
3. Take note of:
   - What the Chatman Constant (≤8 ticks) represents
   - How Knowledge Hooks work
   - Why telemetry validation matters

**Key Concept**: KNHK uses OpenTelemetry schemas to validate that code behaves as documented. This eliminates false positives in testing.

## Step 3: Build the Project

Build the Rust components:

```bash
cargo build --release
```

This may take 2-3 minutes on first run.

**What's happening**:
- Rust compiler checks all code for errors
- Dependencies are downloaded and compiled
- Binary is optimized for performance

Verify it succeeded:
```bash
cargo --version
rustc --version
```

## Step 4: Run Your First Test

Let's run the Chicago TDD test suite:

```bash
make test-chicago-v04
```

**What You Should See**:
- Test output showing test names and results
- Final line showing passed/failed counts
- Green checkmarks (✓) for passed tests

**Understanding the Output**:
- Each test validates a specific behavior
- Tests follow the Arrange-Act-Assert pattern
- Green = behavior matches specification

## Step 5: Understand Test Results

After tests pass, let's interpret what happened:

1. **Tests Passed**: Code behavior matches expectations
2. **Chatman Constant**: Performance was within ≤8 tick limit
3. **Telemetry Validation**: OpenTelemetry output conforms to schema

This is the key innovation of KNHK: **tests validate actual runtime behavior, not just test assertions**.

## Step 6: Explore the Documentation Structure

Now understand how KNHK documentation is organized:

Navigate to `docs/papers/`:

```bash
cd docs/papers
```

You'll find four sections organized by the Diátaxis framework:

1. **Explanation/** - Read to understand concepts
   - Chatman Equation conceptual overview
   - Knowledge graph architecture
   - Mathematical foundations

2. **Reference/** - Use to look up exact specifications
   - Formal papers (PDFs)
   - LaTeX source code
   - Technical diagrams

3. **How-to Guides/** - Use to solve specific problems
   - Step-by-step solution guides
   - Common troubleshooting
   - Best practices

4. **Tutorials/** - Use to learn through practice
   - Step-by-step learning journeys
   - This tutorial!
   - Beginner-friendly examples

## Step 7: Run the Performance Test

Let's verify performance meets the Chatman Constant:

```bash
make test-performance-v04
```

**What This Validates**:
- Critical path operations complete in ≤8 ticks
- Performance is within specification
- Code meets Fortune 500 optimization requirements

**Expected Result**: All performance tests should pass.

## Step 8: Understand Weaver Validation

KNHK uses OpenTelemetry Weaver for schema validation:

```bash
# Check if weaver is installed
which weaver
```

If not installed:
```bash
cargo install opentelemetry-weaver
```

Validate the registry:
```bash
cd /home/user/knhk
weaver registry check -r registry/
```

**What This Does**:
- Verifies the telemetry schema is valid
- Ensures all spans/metrics/logs are documented
- Prevents undocumented telemetry

## Step 9: Review Example Code

Let's look at a simple Rust function:

1. Navigate to `src/` directory
2. Open any `.rs` file (start with a small one)
3. Look for functions using `tracing` macros:
   - `#[tracing::instrument]` - Traces function execution
   - `error!()`, `info!()`, `warn!()` - Log telemetry
4. These telemetry calls are validated by Weaver

## Step 10: Verify Your Setup

You've successfully completed the setup when:

✅ Project builds without errors
```bash
cargo build --release
```

✅ Tests pass completely
```bash
make test-chicago-v04
```

✅ Performance tests pass
```bash
make test-performance-v04
```

✅ Weaver validation passes
```bash
weaver registry check -r registry/
```

## What You've Learned

Congratulations! You now understand:

1. **KNHK's Core Purpose**: Eliminate false positives in testing through schema validation
2. **The Chatman Constant**: Performance constraint of ≤8 ticks
3. **Telemetry-First Design**: All behavior is documented and validated
4. **Documentation Structure**: How to navigate and use different doc categories

## Next Steps

### Want to Learn More?

1. **Understand the Theory**
   - Read: `docs/papers/explanation/formal-foundations.md`
   - Explore: `docs/papers/explanation/kgs_whitepaper_v2_0_sean_chatman.md`

2. **Solve a Problem**
   - Jump to: `docs/papers/how-to-guides/`
   - Choose a guide matching your need

3. **Deep Dive Implementation**
   - Read: `docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf`
   - Explore: LaTeX sources in `docs/papers/reference/chatman-equation/`

### Ready to Build?

See the [How-to Guides](../how-to-guides/) for:
- How to Set Up Your Development Environment
- How to Run Tests Efficiently
- How to Add New Features
- How to Validate Production Readiness

## Troubleshooting

**Compilation fails**: Check Rust is up to date
```bash
rustc --version
rustup update
```

**Tests fail**: Ensure you're on main branch
```bash
git status
git checkout main
```

**Performance tests fail**: System may be under load
```bash
# Close other applications and retry
make test-performance-v04
```

## Resources

- **KNHK Repository**: https://github.com/seanchatmangpt/knhk
- **Diátaxis Framework**: https://diataxis.fr/
- **OpenTelemetry**: https://opentelemetry.io/
- **Rust Book**: https://doc.rust-lang.org/book/

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~20 minutes
**Difficulty**: Beginner
