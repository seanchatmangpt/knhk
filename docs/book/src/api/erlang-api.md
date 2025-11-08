# Erlang API

The Erlang API provides cold path integration for KNHK.

## Overview

The Erlang layer provides:
- Cold path operations
- Schema registry
- Invariant registry
- Hook management

## Core Modules

### knhk_rc

Main application module:
- Application supervision
- Module coordination
- System initialization

### knhk_sigma

Schema registry:
- Schema management
- Schema validation
- Schema queries

### knhk_q

Invariant registry:
- Invariant management
- Invariant validation
- Invariant queries

### knhk_hooks

Hook management:
- Hook registration
- Hook execution
- Hook scheduling

## Related Documentation

- [C API](c-api.md) - C hot path implementation
- [Rust API](rust-api.md) - Rust FFI bindings
- [Integration](../integration/overview.md) - Integration guide
