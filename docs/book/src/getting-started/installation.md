# Installation

## Prerequisites

- Rust 1.70+ (2021 edition)
- C compiler (GCC/Clang) with C11 support (for hot path C layer)
- Node.js 18+ (for unrdf integration)
- Cargo with `native` feature enabled
- Make (for C build system)

## Building

### Build C Hot Path Layer First

The C hot path layer is required by `knhk-hot`:

```bash
cd c && make && cd ..
```

### Build Rust Workspace

Build all Rust crates using workspace (recommended):

```bash
cd rust && cargo build --workspace --release
```

### Build Individual Crates

Build specific crates from workspace root:

```bash
cd rust && cargo build -p knhk-etl --release
cd rust && cargo build -p knhk-unrdf --release --features native
cd rust && cargo build -p knhk-sidecar --release
```

### Alternative: Build from Crate Directories

```bash
cd rust/knhk-etl && cargo build --release
```

## Verification

Verify installation:

```bash
# Check C layer
cd c && make test

# Check Rust workspace
cd rust && cargo test --workspace
```

## Next Steps

- [Building](building.md) - Detailed build instructions
- [Running Tests](testing.md) - Test execution guide
- [Quick Start](../getting-started/quick-start.md) - Get started quickly

