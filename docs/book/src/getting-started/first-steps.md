# First Steps

Quick start guide for KNHK.

## Overview

Get started with KNHK in 5 minutes:
1. Install dependencies
2. Build the system
3. Run tests
4. Execute first query

## Quick Start

### 1. Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install C compiler
# macOS: Xcode Command Line Tools
# Linux: gcc
```

### 2. Build System

```bash
# Build C layer
cd c && make && cd ..

# Build Rust workspace
cd rust && cargo build --workspace && cd ..
```

### 3. Run Tests

```bash
cd rust && cargo test --workspace && cd ..
```

### 4. Execute First Query

```bash
cd rust && cargo run -p knhk-cli -- query "ASK { ?s ?p ?o }"
```

## Next Steps

- [Quick Start](quick-start.md) - Detailed quick start
- [Architecture](../../architecture/system-overview.md) - System design
- [API Reference](../../api/overview.md) - API documentation
