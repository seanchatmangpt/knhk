// rust/knhk-etl/tests/chicago_tdd_hook_registry.rs
// Chicago TDD tests for Hook Registry
// Focus: Behavior verification using AAA pattern (Arrange, Act, Assert)

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use knhk_etl::hook_registry::{guards, HookRegistry, HookRegistryError};
use knhk_hot::KernelType;

#[test]
fn test_hook_registry_creation() {
    // Arrange & Act: Create new hook registry
    let registry = HookRegistry::new();

    // Assert: Registry is empty
    assert_eq!(registry.list_hooks().len(), 0);
}

#[test]
fn test_hook_registry_register_hook() {
    // Arrange: Create registry
    let mut registry = HookRegistry::new();

    // Act: Register a hook
    let hook_id = registry
        .register_hook(
            100,
            KernelType::AskSp,
            guards::always_valid,
            vec!["cardinality >= 1".to_string()],
        )
        .expect("Should register hook");

    // Assert: Hook registered with ID 0, predicate mapped
    assert_eq!(hook_id, 0);
    assert_eq!(registry.get_kernel(100), KernelType::AskSp);
    assert!(registry.has_hook(100));
}

#[test]
fn test_hook_registry_duplicate_predicate() {
    // Arrange: Create registry and register hook
    let mut registry = HookRegistry::new();
    registry
        .register_hook(100, KernelType::AskSp, guards::always_valid, vec![])
        .expect("Should register first hook");

    // Act: Try to register duplicate predicate
    let result = registry.register_hook(100, KernelType::CountSpGe, guards::always_valid, vec![]);

    // Assert: Registration fails with duplicate error
    assert!(result.is_err());
    if let Err(HookRegistryError::DuplicatePredicate(pred)) = result {
        assert_eq!(pred, 100);
    } else {
        panic!("Expected DuplicatePredicate error");
    }
}

#[test]
fn test_hook_registry_get_hook_by_predicate() {
    // Arrange: Create registry and register hooks
    let mut registry = HookRegistry::new();
    registry
        .register_hook(100, KernelType::AskSp, guards::always_valid, vec![])
        .expect("Should register hook");
    registry
        .register_hook(200, KernelType::CountSpGe, guards::always_valid, vec![])
        .expect("Should register hook");

    // Act: Get hook by predicate
    let hook = registry
        .get_hook_by_predicate(100)
        .expect("Should find hook");

    // Assert: Correct hook returned
    assert_eq!(hook.predicate, 100);
    assert_eq!(hook.kernel_type, KernelType::AskSp);
}

#[test]
fn test_hook_registry_unregister_hook() {
    // Arrange: Create registry and register hook
    let mut registry = HookRegistry::new();
    registry
        .register_hook(100, KernelType::AskSp, guards::always_valid, vec![])
        .expect("Should register hook");

    // Act: Unregister hook
    registry
        .unregister_hook(100)
        .expect("Should unregister hook");

    // Assert: Hook no longer exists
    assert!(!registry.has_hook(100));
    assert!(registry.get_hook_by_predicate(100).is_none());
}
