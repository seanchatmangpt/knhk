// knhk-unrdf: Stress tests for native hooks engine
// Tests reliability and performance under load

#[cfg(test)]
#[cfg(feature = "native")]
mod stress_tests {
    use crate::hooks_native::*;
    use crate::types::HookDefinition;
    use serde_json::Value as JsonValue;
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;

    /// Generate test Turtle data
    fn generate_test_data(count: usize) -> String {
        let mut turtle = String::from(
            "@prefix ex: <http://example.org/> .\n@prefix foaf: <http://xmlns.com/foaf/0.1/> .\n\n",
        );
        for i in 0..count {
            turtle.push_str(&format!("ex:person{} foaf:name \"Person {}\" .\n", i, i));
        }
        turtle
    }

    /// Generate a test hook
    fn generate_hook(id: usize, query: &str) -> HookDefinition {
        HookDefinition {
            id: format!("hook-{}", id),
            name: format!("Hook {}", id),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert(
                    "kind".to_string(),
                    JsonValue::String("sparql-ask".to_string()),
                );
                when.insert("query".to_string(), JsonValue::String(query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        }
    }

    #[test]
    fn test_concurrent_hook_execution() {
        // Stress test: Execute hooks concurrently from multiple threads
        let turtle_data = generate_test_data(100);
        let hook = generate_hook(1, "ASK { ?s ?p ?o }");

        let start = Instant::now();
        let num_threads = 10;
        let hooks_per_thread = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let hook_clone = hook.clone();
                let data_clone = turtle_data.clone();
                thread::spawn(move || {
                    let mut results = Vec::new();
                    for _ in 0..hooks_per_thread {
                        if let Ok(result) = evaluate_hook_native(&hook_clone, &data_clone) {
                            results.push(result.fired);
                        }
                    }
                    results
                })
            })
            .collect();

        let all_results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should join successfully"))
            .flatten()
            .collect();

        let duration = start.elapsed();

        assert_eq!(all_results.len(), num_threads * hooks_per_thread);
        assert!(all_results.iter().all(|&fired| fired == true)); // All hooks should fire

        println!(
            "Concurrent execution: {} hooks in {:?} ({:.2} hooks/sec)",
            all_results.len(),
            duration,
            all_results.len() as f64 / duration.as_secs_f64()
        );
    }

    #[test]
    fn test_large_batch_evaluation() {
        // Stress test: Evaluate large batch of hooks
        let turtle_data = generate_test_data(1000);

        let hooks: Vec<HookDefinition> = (0..1000)
            .map(|i| generate_hook(i, "ASK { ?s ?p ?o }"))
            .collect();

        let start = Instant::now();
        let results = evaluate_hooks_batch_native(&hooks, &turtle_data).expect("Failed to evaluate large batch of 1000 hooks");
        let duration = start.elapsed();

        assert_eq!(results.len(), 1000);
        assert!(results.iter().all(|r| r.fired == true));

        println!(
            "Large batch: {} hooks in {:?} ({:.2} hooks/sec)",
            results.len(),
            duration,
            results.len() as f64 / duration.as_secs_f64()
        );
    }

    #[test]
    fn test_registry_concurrent_access() {
        // Stress test: Concurrent registry access
        let registry = Arc::new(NativeHookRegistry::new());

        let num_threads = 20;
        let hooks_per_thread = 50;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let reg = Arc::clone(&registry);
                thread::spawn(move || {
                    for i in 0..hooks_per_thread {
                        let hook_id = format!("hook-{}-{}", thread_id, i);
                        let hook = HookDefinition {
                            id: hook_id.clone(),
                            name: format!("Hook {}", hook_id),
                            hook_type: "sparql-ask".to_string(),
                            definition: {
                                let mut def = serde_json::Map::new();
                                let mut when = serde_json::Map::new();
                                when.insert(
                                    "kind".to_string(),
                                    JsonValue::String("sparql-ask".to_string()),
                                );
                                when.insert(
                                    "query".to_string(),
                                    JsonValue::String("ASK { ?s ?p ?o }".to_string()),
                                );
                                def.insert("when".to_string(), JsonValue::Object(when));
                                JsonValue::Object(def)
                            },
                        };
                        reg.register(hook).expect("Failed to register hook in concurrent test");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should join successfully");
        }

        let hooks = registry.list().expect("Failed to list hooks after concurrent registration");
        assert_eq!(hooks.len(), num_threads * hooks_per_thread);
    }

    #[test]
    fn test_memory_pressure() {
        // Stress test: Execute hooks with large data sets
        let large_data = generate_test_data(10000);
        let hook = generate_hook(1, "ASK { ?s ?p ?o }");

        let start = Instant::now();
        for _ in 0..100 {
            let result = evaluate_hook_native(&hook, &large_data).expect("Failed to evaluate hook under memory pressure");
            assert!(result.fired);
        }
        let duration = start.elapsed();

        println!(
            "Memory pressure: 100 hooks on 10k triples in {:?} ({:.2} hooks/sec)",
            duration,
            100.0 / duration.as_secs_f64()
        );
    }

    #[test]
    fn test_hook_receipt_uniqueness() {
        // Stress test: Verify receipt uniqueness across multiple executions
        let turtle_data = generate_test_data(100);
        let hook = generate_hook(1, "ASK { ?s ?p ?o }");

        let mut receipts = std::collections::HashSet::new();
        for _ in 0..1000 {
            let result = evaluate_hook_native(&hook, &turtle_data).expect("Failed to evaluate hook for receipt uniqueness test");
            if let Some(receipt) = result.receipt {
                // Receipts should be unique (includes timestamp)
                assert!(
                    receipts.insert(receipt.clone()),
                    "Duplicate receipt found: {}",
                    receipt
                );
            }
        }

        assert_eq!(receipts.len(), 1000);
    }

    #[test]
    fn test_batch_with_varying_complexity() {
        // Stress test: Batch with hooks of varying query complexity
        let turtle_data = generate_test_data(500);

        let hooks = vec![
            generate_hook(1, "ASK { ?s ?p ?o }"),              // Simple
            generate_hook(2, "ASK { ?s ?p ?o . ?s ?p2 ?o2 }"), // More complex
            generate_hook(3, "ASK { ?s ?p ?o . FILTER(?o > \"100\") }"), // With filter
            generate_hook(4, "ASK { ?s ?p ?o . ?s ?p2 ?o2 . FILTER(?o != ?o2) }"), // Complex filter
        ];

        let start = Instant::now();
        let results = evaluate_hooks_batch_native(&hooks, &turtle_data).expect("Failed to evaluate batch with varying complexity");
        let duration = start.elapsed();

        assert_eq!(results.len(), 4);
        println!(
            "Varying complexity batch: {} hooks in {:?}",
            results.len(),
            duration
        );
    }

    #[test]
    fn test_error_handling_under_load() {
        // Stress test: Error handling with invalid hooks
        let turtle_data = generate_test_data(100);

        // Valid hook
        let valid_hook = generate_hook(1, "ASK { ?s ?p ?o }");

        // Invalid hook (not ASK query)
        let invalid_hook = HookDefinition {
            id: "invalid".to_string(),
            name: "Invalid".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert(
                    "kind".to_string(),
                    JsonValue::String("sparql-ask".to_string()),
                );
                when.insert(
                    "query".to_string(),
                    JsonValue::String("SELECT * WHERE { ?s ?p ?o }".to_string()),
                );
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };

        // Valid hook should succeed
        let valid_result = evaluate_hook_native(&valid_hook, &turtle_data).expect("Valid hook should succeed");
        assert!(valid_result.fired);

        // Invalid hook should fail gracefully
        let invalid_result = evaluate_hook_native(&invalid_hook, &turtle_data);
        assert!(invalid_result.is_err());
    }
}
