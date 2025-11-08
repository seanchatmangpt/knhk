# Building

## Build System Overview

KNHK uses a hybrid build system:
- **C Layer**: Make-based build for hot path components
- **Rust Layer**: Cargo workspace for all Rust crates

## C Layer Build

### Prerequisites

- GCC or Clang with C11 support
- Make

### Build Commands

```bash
# Build C hot path layer
cd c && make

# Build and run tests
cd c && make test

# Clean build artifacts
cd c && make clean
```

### Build Output

- `libknhk.a` - Static library
- Object files in `src/`
- Test binaries in `tests/`

## Rust Workspace Build

### Standard Build

```bash
# Build all crates in release mode
cd rust && cargo build --workspace --release

# Build all crates in debug mode
cd rust && cargo build --workspace
```

### Feature Flags

```bash
# Build with native features
cd rust && cargo build --workspace --features native

# Build specific crate with features
cd rust && cargo build -p knhk-unrdf --features native
```

### Build Targets

```bash
# Build only library crates
cd rust && cargo build --workspace --lib

# Build only binaries
cd rust && cargo build --workspace --bins

# Build specific crate
cd rust && cargo build -p knhk-etl
```

## Build Performance

### SLOs (Service Level Objectives)

- **First build**: ≤ 15s (Rust workspace)
- **Incremental builds**: ≤ 2s
- **Test execution**: ≤ 30s for full suite

### Optimization Tips

1. Use `--release` for production builds
2. Use `cargo build --workspace` for parallel builds
3. Use `cargo build -p <crate>` for single crate builds
4. Use `cargo build --lib` to skip binary builds

## Troubleshooting

### Common Issues

1. **C layer not found**: Build C layer first (`cd c && make`)
2. **FFI linking errors**: Ensure C layer is built
3. **Feature not found**: Check `Cargo.toml` for available features
4. **Out of memory**: Use `cargo build -p <crate>` instead of workspace

## Next Steps

- [Running Tests](testing.md) - Test execution guide
- [Architecture](../architecture/system-overview.md) - System design

