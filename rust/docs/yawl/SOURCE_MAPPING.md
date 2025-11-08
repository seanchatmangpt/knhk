# YAWL Artifacts → Source Code Mapping

**Last Updated**: January 2025  
**Purpose**: Map extracted YAWL artifacts to existing source code and WIP items

## Overview

This document maps extracted YAWL artifacts (9 C4 diagrams, 407 code files) to:
- Existing source code implementations
- Work-in-progress (WIP) items marked with `unimplemented!()` or `FUTURE` comments
- Architecture documentation
- Related planning documents

## Architecture Diagrams → Source Code

### C1 Context Diagram

**Extracted**: `diagrams/C1_Context.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/api/rest.rs` - REST API endpoints
- `rust/knhk-workflow-engine/src/api/grpc.rs` - gRPC API endpoints
- `rust/knhk-workflow-engine/src/state/store.rs` - State Store (Sled)
- `rust/knhk-workflow-engine/src/integration/sidecar.rs` - External event sources
- `rust/knhk-workflow-engine/src/services/timer.rs` - Timer service

**Documentation**:
- `rust/knhk-workflow-engine/reference/architecture.md`
- `rust/knhk-workflow-engine/core/yawl.md`

### C2 Container Diagram

**Extracted**: `diagrams/C2_Container.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/api/` - API Gateway (REST/gRPC)
- `rust/knhk-workflow-engine/src/execution/engine.rs` - Execution Engine
- `rust/knhk-workflow-engine/src/execution/pipeline.rs` - Execution Pipeline
- `rust/knhk-workflow-engine/src/execution/queue.rs` - Work Queue
- `rust/knhk-workflow-engine/src/state/manager.rs` - State Manager
- `rust/knhk-workflow-engine/src/state/store.rs` - State Store (Sled)
- `rust/knhk-workflow-engine/src/patterns/mod.rs` - Pattern Registry
- `rust/knhk-workflow-engine/src/parser/mod.rs` - Workflow Parser
- `rust/knhk-workflow-engine/src/resource/allocation.rs` - Resource Allocator
- `rust/knhk-workflow-engine/src/resource/pool.rs` - Resource Pool Manager
- `rust/knhk-workflow-engine/src/resilience/circuit_breaker.rs` - Circuit Breaker
- `rust/knhk-workflow-engine/src/resilience/retry.rs` - Retry Policy
- `rust/knhk-workflow-engine/src/resilience/rate_limit.rs` - Rate Limiter
- `rust/knhk-workflow-engine/src/worklets/mod.rs` - Worklet Repository
- `rust/knhk-workflow-engine/src/observability/` - OTEL Observability
- `rust/knhk-workflow-engine/src/integration/lockchain.rs` - Provenance Tracker

**Documentation**:
- `rust/knhk-workflow-engine/FORTUNE5_IMPLEMENTATION_PLAN.md`
- `rust/knhk-workflow-engine/reference/architecture.md`

### C3 Component Diagrams

#### C3: Basic and Advanced Control Flow

**Extracted**: `diagrams/C3_Components_Basic_Advanced.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/basic.rs` - Patterns 1-5
- `rust/knhk-workflow-engine/src/patterns/advanced.rs` - Patterns 6-11
- `rust/knhk-patterns/src/patterns.rs` - Pattern implementations

**Pattern Files**:
- Pattern 1 (Sequence): `rust/knhk-workflow-engine/src/patterns/basic.rs:13`
- Pattern 2 (Parallel Split): `rust/knhk-workflow-engine/src/patterns/basic.rs:34`
- Pattern 3 (Synchronization): `rust/knhk-workflow-engine/src/patterns/basic.rs:55`
- Pattern 4 (Exclusive Choice): `rust/knhk-workflow-engine/src/patterns/basic.rs:76`
- Pattern 5 (Simple Merge): `rust/knhk-workflow-engine/src/patterns/basic.rs:97`
- Patterns 6-11: `rust/knhk-workflow-engine/src/patterns/advanced.rs`

#### C3: Multiple Instance and Cancellation

**Extracted**: `diagrams/C3_Components_MI_Cancellation.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/mi.rs` - Multiple Instance patterns
- `rust/knhk-workflow-engine/src/patterns/multiple_instance.rs` - MI implementations
- `rust/knhk-workflow-engine/src/patterns/cancellation.rs` - Cancellation patterns
- `rust/knhk-workflow-engine/src/patterns/joins.rs` - Join patterns (33-37)
- `rust/knhk-workflow-engine/src/resource/allocation.rs` - Resource Allocator

**Pattern Files**:
- Patterns 12-15: `rust/knhk-workflow-engine/src/patterns/multiple_instance.rs`
- Patterns 19-25: `rust/knhk-workflow-engine/src/patterns/cancellation.rs`
- Patterns 33-37: `rust/knhk-workflow-engine/src/patterns/joins.rs`

#### C3: State-Based, Triggers, and Human Interaction

**Extracted**: `diagrams/C3_Components_State_Triggers_Humans.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/state_based.rs` - Patterns 16-18
- `rust/knhk-workflow-engine/src/patterns/trigger.rs` - Patterns 30-31, 40-43
- `rust/knhk-workflow-engine/src/patterns/advanced_control.rs` - Patterns 26-29
- `rust/knhk-workflow-engine/src/services/work_items.rs` - Work Item Service
- `rust/knhk-workflow-engine/src/services/timer.rs` - Timer Service
- `rust/knhk-workflow-engine/src/integration/sidecar.rs` - Event Sidecar

**Pattern Files**:
- Patterns 16-18: `rust/knhk-workflow-engine/src/patterns/state_based.rs`
- Patterns 26-29: `rust/knhk-workflow-engine/src/patterns/advanced_control.rs`
- Patterns 30-31, 40-43: `rust/knhk-workflow-engine/src/patterns/trigger.rs`

#### C3: Legacy Mode Interface

**Extracted**: `diagrams/C3_Legacy_Mode_Interface.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/api/rest.rs` - REST/gRPC API
- `rust/knhk-workflow-engine/src/parser/mod.rs` - Workflow Parser
- `rust/knhk-workflow-engine/src/parser/extractor.rs` - YAWL/Turtle parsing
- `rust/knhk-workflow-engine/src/execution/engine.rs` - Workflow Engine
- `rust/knhk-workflow-engine/src/patterns/mod.rs` - Pattern Registry
- `rust/knhk-workflow-engine/src/validation/mod.rs` - Workflow Validator
- `rust/knhk-workflow-engine/src/legacy.rs` - Legacy compatibility

**Documentation**:
- `rust/knhk-workflow-engine/core/yawl.md`

### C4 Code Level Diagram

**Extracted**: `diagrams/C4_Code_Level.puml`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/mod.rs` - PatternRegistry class
- `rust/knhk-workflow-engine/src/patterns/adapter.rs` - PatternAdapter
- `rust/knhk-workflow-engine/src/execution/engine.rs` - ExecutionEngine
- `rust/knhk-workflow-engine/src/state/manager.rs` - StateManager
- `rust/knhk-workflow-engine/src/execution/pipeline.rs` - ExecutionPipeline
- `rust/knhk-workflow-engine/src/execution/queue.rs` - WorkQueue
- `rust/knhk-workflow-engine/src/resource/pool.rs` - ResourcePoolManager
- `rust/knhk-workflow-engine/src/resilience/circuit_breaker.rs` - CircuitBreaker

**All 43 Pattern Executors**:
- Patterns 1-5: `rust/knhk-workflow-engine/src/patterns/basic.rs`
- Patterns 6-11: `rust/knhk-workflow-engine/src/patterns/advanced.rs`
- Patterns 12-15: `rust/knhk-workflow-engine/src/patterns/multiple_instance.rs`
- Patterns 16-18: `rust/knhk-workflow-engine/src/patterns/state_based.rs`
- Patterns 19-25: `rust/knhk-workflow-engine/src/patterns/cancellation.rs`
- Patterns 26-29: `rust/knhk-workflow-engine/src/patterns/advanced_control.rs`
- Patterns 30-31: `rust/knhk-workflow-engine/src/patterns/trigger.rs`
- Patterns 33-37: `rust/knhk-workflow-engine/src/patterns/joins.rs`
- Patterns 38-39: `rust/knhk-workflow-engine/src/patterns/advanced_control.rs`
- Patterns 40-43: `rust/knhk-workflow-engine/src/patterns/trigger.rs`

### Reflex Enterprise Container View

**Extracted**: `diagrams/Reflex_Enterprise_Container_View.puml`

**Maps To**:
- `rust/knhk-hot/` - R1 Hot Path
- `rust/knhk-warm/` - W1 Warm Path
- `rust/knhk-etl/` - C1 Cold Path (ETL)
- `rust/knhk-workflow-engine/src/reflex.rs` - Reflex bridge
- `rust/knhk-workflow-engine/src/integration/fortune5/` - Control Plane

**Documentation**:
- `rust/knhk-workflow-engine/FORTUNE5_IMPLEMENTATION_PLAN.md`
- `rust/knhk-workflow-engine/docs/FORTUNE5_PERFORMANCE_ENGINEERING.md`

## Extracted Code → Source Code Mapping

### Pattern Registry

**Extracted**: `code/src_registry.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/mod.rs` - PatternRegistry implementation
- `rust/knhk-patterns/src/patterns.rs` - Pattern trait and implementations

**Comparison**:
- Extracted uses `HashMap<u8, Arc<dyn Pattern>>`
- Current uses `PatternRegistry` with similar structure
- Both register all 43 patterns

### Pattern Types

**Extracted**: `code/src_types.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/mod.rs:40-84` - PatternExecutionContext
- `rust/knhk-workflow-engine/src/patterns/mod.rs:67-120` - PatternExecutionResult

**Comparison**:
- Extracted: `PatternExecutionContext` with `case_id`, `workflow_id`, `variables`
- Current: Similar structure with additional fields (`arrived_from`, `scope_id`)
- Extracted: `PatternExecutionResult` with `success`, `next_activities`, `updates`
- Current: Enhanced with `next_state`, `cancel_activities`, `terminates`

### Pattern Executors

**Extracted**: `code/src_exec/mod.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/patterns/basic.rs` - Patterns 1-5
- `rust/knhk-workflow-engine/src/patterns/advanced.rs` - Patterns 6-11
- `rust/knhk-workflow-engine/src/patterns/multiple_instance.rs` - Patterns 12-15
- `rust/knhk-workflow-engine/src/patterns/state_based.rs` - Patterns 16-18
- `rust/knhk-workflow-engine/src/patterns/cancellation.rs` - Patterns 19-25
- `rust/knhk-workflow-engine/src/patterns/advanced_control.rs` - Patterns 26-29
- `rust/knhk-workflow-engine/src/patterns/trigger.rs` - Patterns 30-31, 40-43
- `rust/knhk-workflow-engine/src/patterns/joins.rs` - Patterns 33-37

**Note**: Extracted code shows scaffold structure; current implementation is more complete.

### State Store

**Extracted**: `code/src_store.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/state/store.rs` - StateStore implementation

**Comparison**:
- Extracted: Basic Sled operations
- Current: Enhanced with `ReflexCache` hot cache layer
- Both use Sled for persistence
- Current has additional methods for case listing and state management

### State Types

**Extracted**: `code/src_types.rs` (Case, WorkflowSpec)

**Maps To**:
- `rust/knhk-workflow-engine/src/case.rs` - Case implementation
- `rust/knhk-workflow-engine/src/parser/types.rs` - WorkflowSpec

**Comparison**:
- Extracted: Basic Case and WorkflowSpec structures
- Current: Enhanced with more fields and methods
- Both use serde for serialization

### Admission Gate

**Extracted**: `code/src_lib.rs` (AdmissionGate)

**Maps To**:
- `rust/knhk-workflow-engine/src/services/admission.rs` - Admission service
- `rust/knhk-admission/src/lib.rs` - Admission gate crate

**Comparison**:
- Extracted: Basic admission gate stub
- Current: Full implementation with SHACL validation

### Timer Service

**Extracted**: `code/src_lib.rs` (TimerService)

**Maps To**:
- `rust/knhk-workflow-engine/src/services/timer.rs` - Timer service

**Comparison**:
- Extracted: Basic timer with tokio
- Current: Enhanced timer service with timing wheels

### Work Item Service

**Extracted**: `code/src_lib.rs` (WorkItemService)

**Maps To**:
- `rust/knhk-workflow-engine/src/services/work_items.rs` - Work item service

**Comparison**:
- Extracted: Basic work item lifecycle
- Current: Full implementation with state management

### RDF Integration

**Extracted**: `code/rdf_src_lib.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/compiler/rdf.rs` - RDF compiler
- `rust/knhk-workflow-engine/src/parser/extractor.rs` - RDF extraction
- `rust/knhk-unrdf/` - unrdf integration

**WIP**: `rust/knhk-workflow-engine/src/compiler/mod.rs:104` - `unimplemented!("compile_rdf_to_ir")`

### Timebase Integration

**Extracted**: `code/crates_timebase_src_lib.rs`

**Maps To**:
- `rust/knhk-workflow-engine/src/timebase.rs` - Timebase integration

## WIP Items → Extracted Artifacts

### 1. RDF Compiler

**WIP Location**: `rust/knhk-workflow-engine/src/compiler/mod.rs:104`

**Status**: `unimplemented!("compile_rdf_to_ir: needs RDF parsing, SHACL validation, SPARQL extraction, and IR generation")`

**Extracted Artifacts That Can Help**:
- `code/rdf_src_lib.rs` - RDF integration reference
- `code/workflow.ttl` - Workflow specification example
- `code/sequence.ttl` - Sequence pattern example
- `code/xml_*.xml` - YAWL XML format examples

**Related Documentation**:
- `rust/knhk-workflow-engine/FORTUNE5_IMPLEMENTATION_PLAN.md` - RDF/Turtle support plan

### 2. Connection Pooling

**WIP Location**: `rust/knhk-workflow-engine/src/performance/pooling.rs:53`

**Status**: `unimplemented!("get_connection: needs real connection pooling implementation")`

**Extracted Artifacts That Can Help**:
- `code/src_store.rs` - State store patterns
- `code/src_types.rs` - Resource management patterns

### 3. Connector Task Execution

**WIP Location**: `rust/knhk-workflow-engine/src/integration/connectors.rs:37`

**Status**: `unimplemented!("execute_task: needs connector-specific task execution implementation")`

**Extracted Artifacts That Can Help**:
- `code/src_lib.rs` - Integration patterns
- `code/crates_timebase_src_lib.rs` - Integration examples

### 4. SPARQL Filter

**WIP Location**: `rust/knhk-workflow-engine/src/ggen/mod.rs:219`

**Status**: `unimplemented!("sparql_filter: needs SPARQL query execution implementation")`

**Extracted Artifacts That Can Help**:
- `code/rdf_src_lib.rs` - RDF/SPARQL integration
- `code/workflow.ttl` - SPARQL query examples

### 5. SPIFFE Authentication

**WIP Location**: `rust/knhk-workflow-engine/src/security/auth.rs:87`

**Status**: `unimplemented!("authenticate: needs SPIFFE/SPIRE integration")`

**Extracted Artifacts That Can Help**:
- `code/src_lib.rs` - Security patterns
- `code/weaver.yaml` - Security configuration examples

### 6. Best Practices Integration

**WIP Location**: `rust/knhk-workflow-engine/src/integration/best_practices.rs`

**Status**: Multiple `FUTURE` comments for unified integration

**Extracted Artifacts That Can Help**:
- `code/src_lib.rs` - Integration patterns
- `code/weaver.yaml` - Weaver integration config
- `code/crates_timebase_src_lib.rs` - Timebase integration
- `code/rdf_src_lib.rs` - RDF integration

**Related Documentation**:
- `rust/knhk-workflow-engine/FORTUNE5_INTEGRATION.md`
- `rust/knhk-workflow-engine/docs/FORTUNE5_USE_CASES.md`

## Configuration Files → Project Setup

### Cargo.toml

**Extracted**: `code/Cargo.toml`

**Maps To**:
- `rust/Cargo.toml` - Workspace Cargo.toml
- `rust/knhk-workflow-engine/Cargo.toml` - Workflow engine Cargo.toml

**Comparison**: Extracted shows workspace structure; current has more dependencies.

### Toolchain

**Extracted**: `code/toolchain.toml`

**Maps To**:
- `rust/rust-toolchain.toml` - Rust toolchain configuration

**Comparison**: Both specify Rust version and components.

### Weaver Configuration

**Extracted**: `code/weaver.yaml`, `code/weaver_checks.yaml`, `code/weaver_registry.yaml`

**Maps To**:
- `rust/knhk-workflow-engine/src/integration/otel.rs` - OTEL integration
- `rust/knhk-workflow-engine/src/integration/fortune5/` - Fortune 5 integration

**Related Documentation**:
- `rust/knhk-workflow-engine/docs/SIDECAR_INTEGRATION.md`
- `rust/knhk-workflow-engine/FORTUNE5_INTEGRATION.md`

### CI/CD Configuration

**Extracted**: `code/ci.yml`

**Maps To**:
- `.github/workflows/` - GitHub Actions workflows
- `rust/knhk-workflow-engine/docs/` - CI/CD documentation

## Hot Path Code → C Implementations

### Beat Scheduler

**Extracted**: `code/beat.c`

**Maps To**:
- `rust/knhk-hot/src/beat.c` - Beat scheduler C implementation
- `rust/knhk-etl/src/beat_scheduler.rs` - Beat scheduler Rust wrapper

### Ring Buffer

**Extracted**: `code/ring_buffer.c`, `code/ring.c`

**Maps To**:
- `rust/knhk-hot/src/ring*.c` - Ring buffer implementations

### Kernels

**Extracted**: `code/kernels.c`

**Maps To**:
- `rust/knhk-hot/src/kernels.c` - Hot path kernels

### Warm Path

**Extracted**: `code/warm_path.c`

**Maps To**:
- `rust/knhk-warm/src/` - Warm path implementation

### AOT Guard

**Extracted**: `code/aot_guard.h`

**Maps To**:
- `rust/knhk-hot/src/*.h` - Hot path headers
- `rust/knhk-workflow-engine/src/performance/aot.rs` - AOT implementation

## Workflow Definitions → Parser Tests

### Turtle Workflows

**Extracted**: `code/workflow.ttl`, `code/sequence.ttl`, `code/turtle_*.ttl`

**Maps To**:
- `rust/knhk-workflow-engine/examples/simple-sequence.ttl` - Example workflow
- `rust/knhk-workflow-engine/tests/` - Parser tests

**Usage**: Can be added to test suite for parser validation.

### XML Workflows

**Extracted**: `code/xml_001.xml` through `code/xml_008.xml`

**Maps To**:
- `rust/knhk-workflow-engine/src/parser/extractor.rs` - YAWL parser
- `rust/knhk-workflow-engine/tests/` - Parser tests

**Usage**: Reference for YAWL XML format support.

## API Examples → Documentation

### JSON Examples

**Extracted**: `code/json_001.json` through `code/json_021.json`

**Maps To**:
- `rust/knhk-workflow-engine/src/api/rest.rs` - REST API
- `rust/knhk-workflow-engine/src/api/models.rs` - API models
- `rust/knhk-workflow-engine/api/rest.md` - API documentation

**Usage**: Can be used for API documentation examples and integration tests.

### Usage Examples

**Extracted**: `code/basic_usage.rs`

**Maps To**:
- `rust/knhk-workflow-engine/examples/` - Example code
- `rust/knhk-workflow-engine/getting-started/quick-start.md` - Quick start guide

**Usage**: Reference for usage examples and documentation.

## Planning Documents → Implementation Status

### Fortune 5 Implementation Plan

**Extracted**: References in `code/src_lib.rs` and scaffold structure

**Maps To**:
- `rust/knhk-workflow-engine/FORTUNE5_IMPLEMENTATION_PLAN.md` - Implementation plan
- `rust/knhk-workflow-engine/FORTUNE5_READINESS.md` - Readiness status
- `rust/knhk-workflow-engine/FORTUNE5_INTEGRATION.md` - Integration guide

**Status**: Implementation in progress; extracted artifacts provide reference structure.

### Architecture Improvements

**Extracted**: C4 diagrams showing architecture

**Maps To**:
- `rust/docs/yawl/ARCHITECTURE_IMPROVEMENTS.md` - Architecture improvements
- `rust/knhk-workflow-engine/reference/architecture.md` - Architecture reference

**Status**: Architecture matches diagrams; diagrams provide validation reference.

## Summary

### Fully Mapped (Implementation Complete)

- Pattern Registry: ✅ Implemented
- Pattern Executors (1-43): ✅ Implemented
- State Store: ✅ Implemented (enhanced with cache)
- Execution Engine: ✅ Implemented
- API (REST/gRPC): ✅ Implemented
- Timer Service: ✅ Implemented
- Work Item Service: ✅ Implemented
- Basic Patterns (1-5): ✅ Implemented
- Advanced Patterns (6-11): ✅ Implemented

### Partially Mapped (WIP)

- RDF Compiler: ⏳ WIP - `unimplemented!()` in `compiler/mod.rs:104`
- Connection Pooling: ⏳ WIP - `unimplemented!()` in `performance/pooling.rs:53`
- Connector Execution: ⏳ WIP - `unimplemented!()` in `integration/connectors.rs:37`
- SPARQL Filter: ⏳ WIP - `unimplemented!()` in `ggen/mod.rs:219`
- SPIFFE Auth: ⏳ WIP - `unimplemented!()` in `security/auth.rs:87`
- Best Practices Integration: ⏳ WIP - Multiple `FUTURE` comments

### Reference Only (Documentation/Examples)

- Configuration files: Reference for project setup
- Workflow definitions: Can be added to test suite
- API examples: Can be used for documentation
- Hot path C code: Reference for C implementations

## Next Steps

1. **Use extracted RDF code** to implement `compile_rdf_to_ir`
2. **Add workflow examples** to test suite (`code/*.ttl`, `code/*.xml`)
3. **Use API examples** for documentation (`code/json_*.json`)
4. **Reference hot path code** for C implementation validation
5. **Use configs** as reference for project setup

## Related Documentation

- [APPLICATION_ANALYSIS.md](APPLICATION_ANALYSIS.md) - Application analysis
- [README.md](README.md) - YAWL artifacts overview
- [CODE_INDEX.md](CODE_INDEX.md) - Code file index
- [DIAGRAMS_README.md](DIAGRAMS_README.md) - Diagram documentation

