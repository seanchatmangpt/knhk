# Dependency Configuration Guide

## Overview

`knhk-etl` uses optional dependencies that are enabled via feature flags. This document explains how to configure and use these dependencies.

## Feature Flags

### `std` Feature

The `std` feature enables standard library support and all optional dependencies:

```toml
[features]
std = [
  "reqwest",
  "serde_json",
  "knhk-hot",
  "knhk-otel",
  "knhk-otel/std",
  "knhk-lockchain",
  "knhk-lockchain/std",
]
```

When enabled, this feature:
- Enables standard library (`std`) instead of `no_std`
- Enables all optional dependencies with their `std` features
- Provides full functionality including HTTP clients, JSON serialization, and observability

### Building with Features

```bash
# Build with std features
cargo build --features std

# Build with specific features
cargo build --features std,kafka

# Build without std (no_std mode)
cargo build --no-default-features
```

## Optional Dependencies

### knhk-hot

**Purpose**: Hot path operations (C FFI)

**Build Requirements**:
1. Build the C library: `cd c && make lib`
2. Ensure `libknhk.a` exists in `c/` directory
3. The build script (`build.rs`) will automatically link against it

**Configuration**: The `build.rs` file sets the library search path to `../../c` to locate `libknhk.a`.

**Usage**: Automatically enabled when `std` feature is enabled.

### knhk-lockchain

**Purpose**: Merkle-linked receipt storage

**Feature Gate**: `#[cfg(feature = "knhk-lockchain")]`

**Usage**: Used in `emit.rs` for writing receipts to the lockchain. Only available when the `knhk-lockchain` feature is enabled (via `std` feature).

### knhk-otel

**Purpose**: OpenTelemetry observability integration

**Feature Gate**: `#[cfg(feature = "knhk-otel")]`

**Usage**: Used in `reflex.rs` and `integration.rs` for generating span IDs and recording metrics. Only available when the `knhk-otel` feature is enabled (via `std` feature).

## no_std Configuration

The crate supports both `no_std` and `std` modes:

- **no_std mode** (default): Uses `alloc` crate, no standard library
- **std mode**: Full standard library support, all optional dependencies available

The configuration is handled via:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;
```

## Troubleshooting

### Optional Dependencies Not Linking

If optional dependencies aren't being linked even when features are enabled:

1. **Clean and rebuild**:
   ```bash
   cargo clean
   cargo build --features std
   ```

2. **Verify feature is enabled**:
   ```bash
   cargo check --features std --verbose 2>&1 | grep "feature="
   ```

3. **Check Cargo.toml**: Ensure optional dependencies are listed in the `std` feature:
   ```toml
   std = [
     "knhk-lockchain",
     "knhk-lockchain/std",
     "knhk-otel",
     "knhk-otel/std",
   ]
   ```

### C Library Not Found (knhk-hot)

If you see errors about `libknhk.a` not found:

1. **Build the C library**:
   ```bash
   cd c
   make lib
   ```

2. **Verify library exists**:
   ```bash
   ls -la c/libknhk.a
   ```

3. **Check build.rs**: The build script should set the search path to `../../c`

### Feature Gate Errors

If you see "unresolved import" errors for optional dependencies:

1. **Ensure feature is enabled**: Use `--features std` when building
2. **Check feature gates**: Code using optional dependencies must be behind `#[cfg(feature = "...")]` gates
3. **Verify Cargo.toml**: Optional dependencies must be listed in the feature that enables them

## Best Practices

1. **Always use feature gates** when using optional dependencies:
   ```rust
   #[cfg(feature = "knhk-lockchain")]
   {
       use knhk_lockchain::Lockchain;
       // ... code using Lockchain
   }
   ```

2. **Provide fallbacks** for no_std mode:
   ```rust
   #[cfg(feature = "knhk-otel")]
   {
       use knhk_otel::generate_span_id;
       generate_span_id()
   }
   #[cfg(not(feature = "knhk-otel"))]
   {
       // Fallback implementation
   }
   ```

3. **Document feature requirements** in function documentation:
   ```rust
   /// Write receipt to lockchain
   /// 
   /// Requires: `knhk-lockchain` feature
   #[cfg(feature = "knhk-lockchain")]
   fn write_receipt_to_lockchain(...) { ... }
   ```

