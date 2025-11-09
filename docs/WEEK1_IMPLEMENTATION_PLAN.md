# One Week Implementation Plan - Critical Gaps

**Date**: 2025-11-08  
**Status**: Ready for Implementation  
**Estimated Effort**: 40-60 hours (1 week)

## Overview

This plan addresses critical production blockers and incomplete features identified in `docs/implementation-gaps.md`. The plan is organized by priority, with critical items first.

## Day 1-2: Critical Production Blockers (16-20 hours)

### 1. Case History Storage Implementation

**Priority**: 🔴 CRITICAL  
**Files**: 
- `rust/knhk-workflow-engine/src/api/rest/handlers.rs`
- `rust/knhk-workflow-engine/src/executor/engine.rs`
- `rust/knhk-workflow-engine/src/state/manager.rs`

**Current State**:
- `StateManager` exists with `get_case_history()` method
- `StateEvent` enum tracks case state transitions
- REST API handler returns empty history

**Implementation Steps**:
1. Add `StateManager` field to `WorkflowEngine` struct in `executor/engine.rs`
2. Initialize `StateManager` in `WorkflowEngine::new()` in `executor/construction.rs`
3. Emit `StateEvent::CaseStateChanged` events when case state changes in `executor/case.rs`
4. Update `get_case_history()` handler in `api/rest/handlers.rs` to:
   - Get `StateManager` from `WorkflowEngine`
   - Call `state_manager.get_case_history(case_id).await`
   - Transform `StateEvent` enum variants to JSON response format
5. Add tests for case history retrieval

**Expected Outcome**: Full audit trail for case state transitions via REST API

---

### 2. REST API Re-enablement

**Priority**: 🔴 CRITICAL  
**Files**:
- `rust/knhk-workflow-engine/src/api/rest/server.rs`
- `rust/knhk-workflow-engine/src/integration/lockchain.rs`

**Current State**:
- Router returns empty `Router::new()` due to LockchainStorage Sync issue
- LockchainIntegration contains `git2::Repository` which is not `Sync`

**Implementation Steps**:
1. Option A (Recommended): Wrap LockchainIntegration in `Mutex` or `RwLock` to make it `Sync`
   - Modify `LockchainIntegration` to use `Arc<RwLock<git2::Repository>>` internally
   - Update all methods to acquire lock before accessing repository
2. Option B: Use `Rc` instead of `Arc` for LockchainIntegration (single-threaded only)
3. Re-enable router routes in `api/rest/server.rs`:
   - `/health` - health check endpoint
   - `/workflows` - register workflow
   - `/cases` - create case
   - `/cases/:id/execute` - execute case
   - `/cases/:id` - get case
   - `/cases/:id/history` - get case history
4. Add integration tests for REST API endpoints

**Expected Outcome**: Fully functional REST API with all endpoints working

---

### 3. Connector Integration for Automated Tasks

**Priority**: 🔴 CRITICAL  
**Files**:
- `rust/knhk-workflow-engine/src/executor/task.rs`
- `rust/knhk-workflow-engine/src/executor/engine.rs`
- `rust/knhk-workflow-engine/src/integration/connectors.rs`

**Current State**:
- `ConnectorIntegration` exists with `execute_task()` method
- Task executor returns error for automated tasks
- WorkflowEngine doesn't have ConnectorIntegration field

**Implementation Steps**:
1. Add `connector_integration: Option<Arc<ConnectorIntegration>>` field to `WorkflowEngine` in `executor/engine.rs`
2. Initialize connector integration in `WorkflowEngine::new()` in `executor/construction.rs`
3. Update `execute_task_with_allocation()` in `executor/task.rs`:
   - Check if `engine.connector_integration` is available
   - For automated tasks (no `required_roles`), call connector integration
   - Map connector name from task metadata or use default connector
   - Execute task via `connector_integration.execute_task(connector_name, case.data).await`
   - Update case with connector result
4. Add connector name to task metadata or use configuration
5. Add error handling for connector failures
6. Add tests for automated task execution

**Expected Outcome**: Automated tasks execute via connector integration instead of returning error

---

## Day 3-4: API and Integration Completion (16-20 hours)

### 4. gRPC Service Implementation

**Priority**: 🟡 MEDIUM (High Value)  
**Files**:
- `rust/knhk-workflow-engine/src/api/grpc.rs`
- `rust/knhk-workflow-engine/proto/workflow_engine.proto` (new file)
- `rust/knhk-workflow-engine/build.rs` (update)

**Current State**:
- `GrpcService` struct exists but no tonic service trait implementation
- No proto definitions
- No tonic-build integration

**Implementation Steps**:
1. Create proto definitions in `proto/workflow_engine.proto`:
   - `RegisterWorkflowRequest/Response`
   - `CreateCaseRequest/Response`
   - `ExecuteCaseRequest/Response`
   - `GetCaseRequest/Response`
   - `GetCaseHistoryRequest/Response`
2. Add `tonic-build` dependency to `Cargo.toml`
3. Update `build.rs` to compile proto files:
   ```rust
   tonic_build::compile_protos("proto/workflow_engine.proto")?;
   ```
4. Implement `#[tonic::async_trait]` trait for `GrpcService`:
   - `register_workflow()` - delegate to `engine.register_workflow()`
   - `create_case()` - delegate to `engine.create_case()`
   - `execute_case()` - delegate to `engine.execute_case()`
   - `get_case()` - delegate to `engine.get_case()`
   - `get_case_history()` - delegate to `engine.get_case_history()`
5. Add gRPC server setup in `api/grpc.rs`:
   - Create tonic server with `GrpcService`
   - Add server start/stop methods
6. Add integration tests for gRPC endpoints

**Expected Outcome**: Fully functional gRPC API matching REST API functionality

---

### 5. Integration Health Checks Implementation

**Priority**: 🟡 MEDIUM  
**Files**:
- `rust/knhk-workflow-engine/src/integration/check.rs`
- `rust/knhk-workflow-engine/src/integration/fortune5/integration.rs`
- `rust/knhk-workflow-engine/src/integration/lockchain.rs`
- `rust/knhk-workflow-engine/src/integration/connectors.rs`
- `rust/knhk-workflow-engine/src/integration/otel.rs`

**Current State**:
- `perform_health_check()` has `unimplemented!()` for all integrations
- Individual check methods exist but are stubs

**Implementation Steps**:
1. Implement `check_fortune5_health()`:
   - Check if Fortune5Integration is available
   - Verify SLO endpoint is reachable (if configured)
   - Check promotion gate status
   - Return `Ok(())` if healthy, `Err` if unhealthy
2. Implement `check_lockchain_health()`:
   - Check if LockchainIntegration is available
   - Verify receipt storage is accessible
   - Test append operation (if possible)
   - Return health status
3. Implement `check_connectors_health()`:
   - Check if ConnectorIntegration is available
   - Verify registered connectors are accessible
   - Test connector connectivity (Kafka, Salesforce, etc.)
   - Return health status
4. Implement `check_otel_health()`:
   - Check if OtelIntegration is available
   - Verify OTLP exporter is reachable
   - Test span export (if possible)
   - Return health status
5. Implement `check_sidecar_health()`:
   - Check if SidecarIntegration is available
   - Verify gRPC endpoint is reachable
   - Test sidecar process status
   - Return health status
6. Implement `check_etl_health()`:
   - Check if ETL pipeline is available
   - Verify pipeline stages are operational
   - Test reflex bridge connectivity
   - Return health status
7. Add tests for each health check

**Expected Outcome**: All integrations have working health checks

---

## Day 5: Security and Performance (8-12 hours)

### 6. SPIFFE Authentication Implementation

**Priority**: 🟡 MEDIUM  
**Files**:
- `rust/knhk-workflow-engine/src/security/auth.rs`
- `rust/knhk-sidecar/src/spiffe.rs` (reference)

**Current State**:
- `authenticate()` method has `unimplemented!()` for SPIFFE/SPIRE integration
- `Principal` struct exists with SPIFFE ID support
- Sidecar has SPIFFE certificate management

**Implementation Steps**:
1. Review `knhk-sidecar/src/spiffe.rs` for SPIFFE certificate handling
2. Implement SPIFFE ID validation:
   - Parse SPIFFE ID format: `spiffe://trust-domain/path`
   - Validate trust domain matches configured trust domain
   - Extract path component as principal ID
3. Implement mTLS certificate validation:
   - Extract SPIFFE ID from certificate SAN extension
   - Validate certificate chain against trust bundle
   - Extract principal attributes from certificate
4. Update `authenticate()` method:
   - Extract SPIFFE ID from request headers or certificate
   - Validate SPIFFE ID format
   - Create `Principal` with SPIFFE ID and attributes
   - Return `Ok(Principal)` or `Err(AuthError)`
5. Add configuration for trust domain and trust bundle path
6. Add tests for SPIFFE authentication

**Expected Outcome**: SPIFFE/SPIRE authentication working for mTLS requests

---

### 7. Connection Pooling Completion

**Priority**: 🟡 MEDIUM  
**Files**:
- `rust/knhk-workflow-engine/src/performance/pooling.rs`

**Current State**:
- `ConnectionPool` struct exists with basic structure
- `get_connection()` method has implementation but may need completion
- Connection lifecycle management exists

**Implementation Steps**:
1. Review current `get_connection()` implementation
2. Verify connection validation logic:
   - Test connection before returning from pool
   - Remove invalid connections from pool
   - Create new connections if pool is empty
3. Verify connection cleanup:
   - Remove expired idle connections
   - Close connections on pool drop
   - Handle connection errors gracefully
4. Add connection metrics:
   - Track active connections
   - Track pool size
   - Track connection wait time
5. Add configuration for:
   - Max pool size
   - Connection timeout
   - Idle connection timeout
6. Add tests for connection pooling:
   - Test connection reuse
   - Test connection creation
   - Test connection cleanup
   - Test pool exhaustion

**Expected Outcome**: Fully functional connection pooling with proper lifecycle management

---

## Day 6-7: Integration and Testing (8-12 hours)

### 8. Best Practices Integration Implementation

**Priority**: 🟡 MEDIUM  
**Files**:
- `rust/knhk-workflow-engine/src/integration/best_practices.rs`

**Current State**:
- `BestPracticesIntegration` struct exists
- `create_unified_integration()`, `execute_with_best_features()`, `health_check_all()` have `unimplemented!()`

**Implementation Steps**:
1. Implement `create_unified_integration()`:
   - Initialize all integrations (Fortune5, Lockchain, OTEL, Connectors)
   - Create `IntegrationHealthChecker` with all integrations
   - Return `BestPracticesIntegration` instance
2. Implement `execute_with_best_features()`:
   - Execute workflow with Fortune5 SLO tracking
   - Record provenance via Lockchain
   - Emit OTEL spans for observability
   - Use connectors for external tasks
   - Return unified result
3. Implement `health_check_all()`:
   - Call health check for each integration
   - Aggregate health status
   - Return overall health status with per-integration details
4. Add example usage in documentation
5. Add tests for unified integration

**Expected Outcome**: Unified integration combining all best features

---

### 9. Testing and Validation

**Priority**: 🔴 CRITICAL  
**Files**: All modified files

**Implementation Steps**:
1. Add unit tests for each implemented feature
2. Add integration tests for API endpoints (REST and gRPC)
3. Add integration tests for connector execution
4. Add integration tests for case history
5. Add integration tests for health checks
6. Run full test suite and fix any failures
7. Run clippy and fix warnings
8. Run cargo fmt to ensure code formatting
9. Verify all `unimplemented!()` calls are removed (except documented ones)
10. Verify all `FUTURE:` comments are addressed or documented

**Expected Outcome**: All tests passing, code ready for production

---

## Summary

### Critical Items (Must Complete)
1. ✅ Case History Storage
2. ✅ REST API Re-enablement
3. ✅ Connector Integration
4. ✅ Testing and Validation

### High Value Items (Should Complete)
5. ✅ gRPC Service
6. ✅ Integration Health Checks

### Nice to Have (If Time Permits)
7. SPIFFE Authentication
8. Connection Pooling Completion
9. Best Practices Integration

### Estimated Timeline
- **Day 1-2**: Critical production blockers (16-20 hours)
- **Day 3-4**: API and integration completion (16-20 hours)
- **Day 5**: Security and performance (8-12 hours)
- **Day 6-7**: Integration and testing (8-12 hours)

**Total**: 48-64 hours (1 week with buffer)

---

## Success Criteria

1. ✅ All critical production blockers resolved
2. ✅ REST API fully functional
3. ✅ Case history available via REST API
4. ✅ Automated tasks execute via connectors
5. ✅ gRPC service implemented (if time permits)
6. ✅ All integration health checks working
7. ✅ All tests passing
8. ✅ Zero `unimplemented!()` calls in production code (except documented)
9. ✅ Code follows production best practices (no unwrap/expect in production paths)

---

## Notes

- All implementations must follow production best practices (no unwrap/expect in production code)
- All error handling must use `Result<T, E>` types
- All async operations must be properly awaited
- All resources must be properly cleaned up
- All code must be tested and validated

