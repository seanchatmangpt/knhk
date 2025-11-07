// Hook Registry Demo - Validates implementation works correctly

use knhk_etl::hook_registry::{HookRegistry, guards};
use knhk_etl::ingest::RawTriple;
use knhk_hot::KernelType;

fn main() {
    println!("=== Hook Registry Demo ===\n");

    // Create registry
    let mut registry = HookRegistry::new();
    println!("✓ Created hook registry");

    // Register hooks for different predicates
    let hook1 = registry.register_hook(
        100, // predicate ID
        KernelType::AskSp,
        guards::always_valid,
        vec!["cardinality >= 1".to_string()],
    ).unwrap();
    println!("✓ Registered hook {} for predicate 100 (AskSp)", hook1);

    let hook2 = registry.register_hook(
        200,
        KernelType::ValidateSp,
        guards::check_subject_nonempty,
        vec!["subject must be non-empty".to_string()],
    ).unwrap();
    println!("✓ Registered hook {} for predicate 200 (ValidateSp)", hook2);

    let hook3 = registry.register_hook(
        300,
        KernelType::CountSpGe,
        guards::check_object_integer,
        vec!["object must be integer".to_string()],
    ).unwrap();
    println!("✓ Registered hook {} for predicate 300 (CountSpGe)", hook3);

    // Test kernel lookup
    println!("\n=== Kernel Lookup ===");
    assert_eq!(registry.get_kernel(100), KernelType::AskSp);
    println!("✓ Predicate 100 → AskSp");

    assert_eq!(registry.get_kernel(200), KernelType::ValidateSp);
    println!("✓ Predicate 200 → ValidateSp");

    assert_eq!(registry.get_kernel(300), KernelType::CountSpGe);
    println!("✓ Predicate 300 → CountSpGe");

    // Unregistered predicate uses default
    assert_eq!(registry.get_kernel(999), KernelType::AskSp);
    println!("✓ Predicate 999 (unregistered) → AskSp (default)");

    // Test guard execution
    println!("\n=== Guard Execution ===");

    let valid_triple = RawTriple {
        subject: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object: "http://example.org/object".to_string(),
        graph: None,
    };

    let empty_subject_triple = RawTriple {
        subject: "".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object: "http://example.org/object".to_string(),
        graph: None,
    };

    let integer_object_triple = RawTriple {
        subject: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object: "\"42\"".to_string(),
        graph: None,
    };

    // Predicate 100: always_valid guard
    assert!(registry.check_guard(100, &valid_triple));
    assert!(registry.check_guard(100, &empty_subject_triple));
    println!("✓ Predicate 100 guard (always_valid): passes all");

    // Predicate 200: check_subject_nonempty guard
    assert!(registry.check_guard(200, &valid_triple));
    assert!(!registry.check_guard(200, &empty_subject_triple));
    println!("✓ Predicate 200 guard (subject_nonempty): rejects empty subject");

    // Predicate 300: check_object_integer guard
    assert!(registry.check_guard(300, &integer_object_triple));
    assert!(!registry.check_guard(300, &valid_triple)); // URI not integer
    println!("✓ Predicate 300 guard (object_integer): validates integer objects");

    // Test hook metadata
    println!("\n=== Hook Metadata ===");
    let metadata = registry.get_hook_by_predicate(100).unwrap();
    println!("Hook for predicate 100:");
    println!("  - ID: {}", metadata.id);
    println!("  - Kernel: {:?}", metadata.kernel_type);
    println!("  - Invariants: {:?}", metadata.invariants);
    println!("  - Compiled at: {}", metadata.compiled_at);

    // List all hooks
    println!("\n=== All Registered Hooks ===");
    for hook in registry.list_hooks() {
        println!("Hook {}: predicate {} → {:?} ({} invariants)",
            hook.id, hook.predicate, hook.kernel_type, hook.invariants.len());
    }

    // Test duplicate registration
    println!("\n=== Error Handling ===");
    match registry.register_hook(100, KernelType::AskSp, guards::always_valid, vec![]) {
        Ok(_) => println!("✗ Should have failed on duplicate predicate"),
        Err(e) => println!("✓ Correctly rejected duplicate: {}", e),
    }

    println!("\n=== Demo Complete ===");
    println!("Hook registry working correctly!");
}
