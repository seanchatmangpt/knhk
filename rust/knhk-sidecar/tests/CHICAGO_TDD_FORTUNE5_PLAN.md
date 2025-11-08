# Chicago TDD Validation Plan for Fortune 5 Features

**Date**: 2025-01-XX  
**Status**: Test Plan Complete - Ready for Implementation

---

## Summary

Comprehensive Chicago TDD test plan for validating all Fortune 5 features. Tests follow Chicago TDD principles:
- **State-based verification** (not interaction-based)
- **Real collaborators** (minimal mocks)
- **Verify outputs and invariants** (not implementation details)
- **AAA pattern** (Arrange, Act, Assert)

---

## Test Coverage Plan

### 1. SPIFFE/SPIRE Tests (5 tests)

**Test: test_spiffe_config_validation**
- **Arrange**: Create SPIFFE config with nonexistent socket
- **Act**: Call `validate()`
- **Assert**: Should return error for missing socket

**Test: test_spiffe_certificate_loading**
- **Arrange**: Create temp directory with SPIRE cert files
- **Act**: Call `load_certificate()`
- **Assert**: Certificates should be loaded and available

**Test: test_spiffe_id_validation**
- **Arrange**: Various SPIFFE ID formats
- **Act**: Call `validate_spiffe_id()`
- **Assert**: Valid IDs pass, invalid fail

**Test: test_extract_trust_domain**
- **Arrange**: SPIFFE IDs with different formats
- **Act**: Call `extract_trust_domain()`
- **Assert**: Trust domain extracted correctly

**Test: test_spiffe_certificate_refresh**
- **Arrange**: Load certificate, wait for refresh interval
- **Act**: Check `needs_refresh()`
- **Assert**: Should need refresh after interval

### 2. KMS Tests (5 tests)

**Test: test_kms_config_validation_aws**
- **Arrange**: Create AWS KMS config
- **Act**: Call `validate()`
- **Assert**: Should pass validation, auto-rotation enabled

**Test: test_kms_config_validation_azure**
- **Arrange**: Create Azure Key Vault config
- **Act**: Call `validate()`
- **Assert**: Should pass validation

**Test: test_kms_config_validation_vault**
- **Arrange**: Create HashiCorp Vault config
- **Act**: Call `validate()`
- **Assert**: Should pass validation

**Test: test_kms_config_validation_empty_region**
- **Arrange**: Create AWS config with empty region
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_kms_config_rotation_interval_validation**
- **Arrange**: Create config with rotation interval > 24h
- **Act**: Call `validate()`
- **Assert**: Should return error (Fortune 5 requirement)

### 3. Key Rotation Tests (3 tests)

**Test: test_key_rotation_interval_validation**
- **Arrange**: Create rotation manager with various intervals
- **Act**: Call `new()`
- **Assert**: ≤24h passes, >24h fails

**Test: test_key_rotation_needs_rotation**
- **Arrange**: Create rotation manager
- **Act**: Check `needs_rotation()`
- **Assert**: Should need rotation initially

**Test: test_key_rotation_enable_disable**
- **Arrange**: Create rotation manager
- **Act**: Enable/disable rotation
- **Assert**: State changes correctly

### 4. Multi-Region Tests (6 tests)

**Test: test_region_config_validation**
- **Arrange**: Create valid region config
- **Act**: Call `validate()`
- **Assert**: Should pass validation

**Test: test_region_config_validation_empty_region**
- **Arrange**: Create config with empty region
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_region_config_validation_quorum_threshold**
- **Arrange**: Create config with quorum > total regions
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_receipt_sync_manager_creation**
- **Arrange**: Create valid region config
- **Act**: Create `ReceiptSyncManager`
- **Assert**: Should succeed

**Test: test_receipt_sync_disabled**
- **Arrange**: Create manager with sync disabled
- **Act**: Call `sync_receipt()`
- **Assert**: Should return success with 0 synced regions

**Test: test_legal_hold_manager**
- **Arrange**: Create legal hold manager with policy
- **Act**: Check `should_hold()` and `apply_hold()`
- **Assert**: Should hold matching receipts

### 5. SLO Admission Tests (7 tests)

**Test: test_slo_config_validation**
- **Arrange**: Create default SLO config
- **Act**: Call `validate()`
- **Assert**: Should pass, values match Fortune 5 requirements

**Test: test_slo_config_validation_r1_exceeds**
- **Arrange**: Create config with R1 > 2ns
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_slo_admission_strict_reject**
- **Arrange**: Create strict admission controller
- **Act**: Check admission with latency exceeding SLO
- **Assert**: Should reject request

**Test: test_slo_admission_strict_accept**
- **Arrange**: Create strict admission controller
- **Act**: Check admission with latency within SLO
- **Assert**: Should admit request

**Test: test_slo_admission_degrade**
- **Arrange**: Create degrade admission controller
- **Act**: Check admission with latency exceeding R1 but within W1
- **Assert**: Should degrade to W1

**Test: test_slo_admission_latency_tracking**
- **Arrange**: Create admission controller
- **Act**: Record latencies, estimate latency
- **Assert**: Should calculate p99 from history

**Test: test_slo_admission_metrics**
- **Arrange**: Create admission controller
- **Act**: Admit and reject requests
- **Assert**: Metrics should be updated correctly

### 6. Capacity Planning Tests (4 tests)

**Test: test_capacity_manager_record_access**
- **Arrange**: Create capacity manager
- **Act**: Record cache hits and misses
- **Assert**: Metrics should be tracked correctly

**Test: test_capacity_manager_hit_rate**
- **Arrange**: Create capacity manager, record 95% hits
- **Act**: Check `meets_capacity()`
- **Assert**: Should meet capacity threshold

**Test: test_capacity_manager_hottest_predicates**
- **Arrange**: Create manager, record accesses for multiple predicates
- **Act**: Get top-N hottest predicates
- **Assert**: Should be sorted by hit rate

**Test: test_capacity_manager_l1_locality_prediction**
- **Arrange**: Create manager, record L1 hits/misses
- **Act**: Predict L1 locality
- **Assert**: Should calculate L1 hit rate correctly

### 7. Promotion Gates Tests (7 tests)

**Test: test_promotion_config_validation**
- **Arrange**: Create default promotion config
- **Act**: Call `validate()`
- **Assert**: Should pass validation

**Test: test_promotion_config_validation_invalid_threshold**
- **Arrange**: Create config with SLO threshold > 1.0
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_promotion_config_validation_canary_traffic**
- **Arrange**: Create canary config with traffic > 100%
- **Act**: Call `validate()`
- **Assert**: Should return error

**Test: test_promotion_gate_manager_creation**
- **Arrange**: Create valid promotion and SLO configs
- **Act**: Create `PromotionGateManager`
- **Assert**: Should succeed

**Test: test_promotion_gate_feature_flags**
- **Arrange**: Create manager with feature flags
- **Act**: Enable/disable features
- **Assert**: Feature flags should update correctly

**Test: test_promotion_gate_slo_compliance_no_requests**
- **Arrange**: Create manager
- **Act**: Check compliance with no requests
- **Assert**: Should return true (no data to evaluate)

**Test: test_promotion_gate_slo_compliance_below_threshold**
- **Arrange**: Create manager, process requests with 50% admission
- **Act**: Check compliance
- **Assert**: Should return false, trigger rollback

**Test: test_promotion_gate_promote_canary_to_staging**
- **Arrange**: Create manager in canary environment
- **Act**: Promote to staging
- **Assert**: Should succeed

**Test: test_promotion_gate_promote_invalid**
- **Arrange**: Create manager in production
- **Act**: Try to promote to staging
- **Assert**: Should return error

### 8. Integration Tests (3 tests)

**Test: test_fortune5_integration_spiffe_kms**
- **Arrange**: Create SPIFFE and KMS managers
- **Act**: Validate both configurations
- **Assert**: Both should be valid

**Test: test_fortune5_integration_slo_capacity**
- **Arrange**: Create SLO and capacity managers
- **Act**: Record capacity and check admission
- **Assert**: Should work together

**Test: test_fortune5_integration_multi_region_legal_hold**
- **Arrange**: Create multi-region and legal hold managers
- **Act**: Apply legal hold to receipt
- **Assert**: Should hold matching receipts

---

## Test Implementation Notes

### Dependencies Required

```toml
[dev-dependencies]
tempfile = "3.8"  # For SPIFFE certificate file testing
```

### Test Structure

All tests follow AAA pattern:
```rust
#[test]
fn test_feature_name() {
    // Arrange
    let config = ...;
    
    // Act
    let result = ...;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}
```

### Helper Functions

- `create_temp_spiffe_dir()`: Creates temporary directory with SPIRE cert files
- Uses `tempfile` crate for temporary file system

---

## Test Execution

```bash
# Run all Fortune 5 tests
cargo test --test chicago_tdd_fortune5

# Run specific test
cargo test --test chicago_tdd_fortune5 test_spiffe_config_validation

# Run with output
cargo test --test chicago_tdd_fortune5 -- --nocapture
```

---

## Validation Checklist

- [x] Test plan created for all Fortune 5 features
- [x] Tests follow Chicago TDD principles
- [x] Tests use AAA pattern
- [x] Tests verify actual behavior, not implementation
- [x] Tests cover error paths
- [x] Tests cover edge cases
- [ ] Test file created (pending disk space)
- [ ] Tests compiled successfully
- [ ] All tests pass

---

## Next Steps

1. **Free Disk Space**: Clear disk space to allow test file creation
2. **Create Test File**: Write `tests/chicago_tdd_fortune5.rs` with all tests
3. **Run Tests**: Execute test suite
4. **Fix Failures**: Address any test failures
5. **Add Integration Tests**: Add tests with mock HTTP servers
6. **Add Performance Tests**: Add tests for hot path operations

---

## Status

**Test Plan**: ✅ **COMPLETE**

All test cases are planned and ready for implementation. The test plan covers:
- 40+ test cases
- 7 test categories
- 3 integration tests
- All Fortune 5 features

**Implementation**: ⏳ **PENDING** (disk space issue)

