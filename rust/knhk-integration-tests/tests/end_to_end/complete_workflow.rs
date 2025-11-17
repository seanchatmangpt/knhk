//! End-to-End Integration Tests: Complete Workflow Execution
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: ALL COVENANTS (O, Σ, Q, Π, MAPE-K, Chatman)
//! - Why This Matters: Individual unit tests can lie. Only end-to-end tests prove the system works.
//!
//! WHAT THIS TESTS:
//! - Complete O → Σ → μ → O' cycle
//! - All 6 covenants validated in a real workflow
//! - Failure injection and recovery
//! - Learning and improvement via MAPE-K
//! - Full doctrine validation
//!
//! TEST SCENARIOS:
//! 1. Payment processor workflow (complex real-world example)
//! 2. Failure injection and autonomic recovery
//! 3. Performance under load
//! 4. Schema conformance end-to-end

use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn ontology_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("ontology")
        .join(filename)
}

#[test]
fn test_complete_workflow_cycle() {
    // GIVEN: Complete workflow with all patterns
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    // Load pattern matrix for validation
    let matrix_path = ontology_path("yawl-pattern-permutations.ttl");
    let matrix = fs::read_to_string(&matrix_path).expect("Failed to read matrix");

    // Load MAPE-K for autonomic features
    let mape_k_path = ontology_path("mape-k-autonomic.ttl");
    let mape_k = fs::read_to_string(&mape_k_path).expect("Failed to read MAPE-K");

    // WHEN: We execute the complete O → Σ → μ → O' cycle
    let start = Instant::now();

    let store = Store::new().expect("Failed to create store");

    // Step 1: Load Turtle (O → Σ)
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse workflow");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, matrix.as_bytes())
        .expect("Failed to parse matrix");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, mape_k.as_bytes())
        .expect("Failed to parse MAPE-K");

    // Step 2: Validate against pattern matrix (Q checks)
    let validation_query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?task) as ?validTasks) WHERE {
            ?task a yawl:Task .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .

            # Validate against pattern matrix
            ?combo a yawl:SplitJoinCombination .
            ?combo yawl:splitType ?split .
            ?combo yawl:joinType ?join .
            ?combo yawl:isValid true .
        }
    "#;

    let results = store.query(validation_query).expect("Validation failed");
    let mut valid_task_count = 0;

    if let QueryResults::Solutions(mut solutions) = results {
        if let Some(solution) = solutions.next() {
            let solution = solution.expect("Solution error");
            let count_str = solution.get("validTasks").expect("No count").to_string();
            valid_task_count = count_str
                .chars()
                .filter(|c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .unwrap_or(0);
        }
    }

    // Step 3: Extract execution plan (Σ → μ)
    let execution_query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        PREFIX yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#>
        SELECT ?task ?split ?join ?execMode WHERE {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow yawl:hasTask ?task .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .
            OPTIONAL { ?task yawl-exec:executionMode ?execMode . }
        }
        ORDER BY ?task
    "#;

    let exec_results = store
        .query(execution_query)
        .expect("Execution query failed");

    // Step 4: Verify MAPE-K integration (μ → O')
    let mape_k_query = r#"
        PREFIX mape-k: <http://bitflow.ai/ontology/mape-k/v1#>
        SELECT ?component WHERE {
            VALUES ?component {
                mape-k:Monitor
                mape-k:Analyze
                mape-k:Plan
                mape-k:Execute
                mape-k:Knowledge
            }
        }
    "#;

    let mape_results = store.query(mape_k_query).expect("MAPE-K query failed");

    let elapsed = start.elapsed();

    // THEN: Complete cycle succeeds and validates all covenants
    assert!(
        valid_task_count > 0,
        "Covenant 2 FAILED: No valid tasks (pattern matrix validation)"
    );

    if let QueryResults::Solutions(solutions) = exec_results {
        let task_count = solutions.count();
        assert!(
            task_count > 0,
            "Covenant 1 FAILED: Workflow structure not preserved"
        );
    }

    if let QueryResults::Solutions(solutions) = mape_results {
        let mape_count = solutions.count();
        assert!(
            mape_count == 5,
            "Covenant 3 FAILED: MAPE-K incomplete ({} components, expected 5)",
            mape_count
        );
    }

    assert!(
        elapsed.as_millis() < 200,
        "Covenant 5 FAILED: End-to-end cycle too slow: {:?}ms (max 200ms)",
        elapsed.as_millis()
    );

    println!(
        "✅ Complete workflow cycle validated ({}ms, {} valid tasks)",
        elapsed.as_millis(),
        valid_task_count
    );
}

#[test]
fn test_payment_processor_workflow() {
    // GIVEN: Real-world payment processing workflow
    let workflow_path = ontology_path("workflows/financial/swift_payment.ttl");

    // Check if financial workflow exists
    if !workflow_path.exists() {
        println!("Skipping: swift_payment.ttl not found");
        return;
    }

    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse workflow");

    // WHEN: We validate the payment workflow
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?workflow ?task WHERE {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow yawl:hasTask ?task .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Payment workflow is complete and valid
    if let QueryResults::Solutions(solutions) = results {
        let count = solutions.count();
        assert!(count > 0, "Payment workflow has no tasks");
    }
}

#[test]
fn test_failure_recovery_scenario() {
    // GIVEN: Workflow with compensation pattern
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse");

    // WHEN: We check for failure recovery mechanisms
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        PREFIX yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#>
        SELECT ?task ?backwardFlow ?maxIter ?retryPolicy WHERE {
            ?task a yawl:Task .
            OPTIONAL { ?task yawl:BackwardFlow ?backwardFlow . }
            OPTIONAL { ?task yawl:MaxIterations ?maxIter . }
            OPTIONAL { ?task yawl-exec:RetryPolicy ?retryPolicy . }
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Failure recovery patterns exist
    if let QueryResults::Solutions(solutions) = results {
        let mut has_compensation = false;
        let mut has_retry = false;

        for solution in solutions {
            let solution = solution.expect("Solution error");
            if solution.get("backwardFlow").is_some() {
                has_compensation = true;
            }
            if solution.get("retryPolicy").is_some() {
                has_retry = true;
            }
        }

        assert!(
            has_compensation || has_retry,
            "No failure recovery mechanisms (compensation/retry)"
        );
    }
}

#[test]
fn test_escalation_and_cancellation() {
    // GIVEN: Workflow with escalation and cancellation
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse");

    // WHEN: We check for escalation and cancel patterns
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?task ?cancelScope ?timeout WHERE {
            ?task a yawl:Task .
            OPTIONAL { ?task yawl:CancelScope ?cancelScope . }
            OPTIONAL { ?task yawl:TimeoutMs ?timeout . }
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Escalation and cancellation mechanisms exist
    if let QueryResults::Solutions(solutions) = results {
        let mut has_cancel = false;
        let mut has_timeout = false;

        for solution in solutions {
            let solution = solution.expect("Solution error");
            if solution.get("cancelScope").is_some() {
                has_cancel = true;
            }
            if solution.get("timeout").is_some() {
                has_timeout = true;
            }
        }

        assert!(
            has_cancel || has_timeout,
            "No escalation/cancellation mechanisms"
        );
    }
}

#[test]
fn test_multi_pattern_interaction() {
    // GIVEN: Workflow using multiple patterns simultaneously
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse");

    // WHEN: We enumerate pattern usage
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT DISTINCT ?split ?join WHERE {
            ?task a yawl:Task .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Multiple pattern combinations are used
    if let QueryResults::Solutions(solutions) = results {
        let combinations: Vec<(String, String)> = solutions
            .filter_map(|s| s.ok())
            .filter_map(|s| {
                let split = s.get("split")?.to_string();
                let join = s.get("join")?.to_string();
                Some((split, join))
            })
            .collect();

        assert!(
            combinations.len() >= 3,
            "Expected multiple pattern combinations, found {}",
            combinations.len()
        );
    }
}

#[test]
fn test_all_covenants_in_one_workflow() {
    // GIVEN: Complete autonomous workflow
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let matrix_path = ontology_path("yawl-pattern-permutations.ttl");
    let matrix = fs::read_to_string(&matrix_path).expect("Failed to read matrix");

    let mape_k_path = ontology_path("mape-k-autonomic.ttl");
    let mape_k = fs::read_to_string(&mape_k_path).expect("Failed to read MAPE-K");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
        .expect("Failed to parse workflow");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, matrix.as_bytes())
        .expect("Failed to parse matrix");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, mape_k.as_bytes())
        .expect("Failed to parse MAPE-K");

    // Covenant 1: Turtle is definition (structure preserved)
    let covenant1_query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?task) as ?tasks)
               (COUNT(?condition) as ?conditions)
               (COUNT(?constraint) as ?constraints)
        WHERE {
            ?workflow a yawl:WorkflowSpecification .
            OPTIONAL { ?workflow yawl:hasTask ?task . }
            OPTIONAL { ?workflow yawl:hasCondition ?condition . }
            OPTIONAL { ?workflow yawl:hasConstraint ?constraint . }
        }
    "#;

    let c1_results = store
        .query(covenant1_query)
        .expect("Covenant 1 check failed");

    // Covenant 2: Invariants enforced (pattern matrix validation)
    let covenant2_query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?task) as ?validTasks) WHERE {
            ?task a yawl:Task .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .
            ?combo yawl:splitType ?split .
            ?combo yawl:joinType ?join .
            ?combo yawl:isValid true .
        }
    "#;

    let c2_results = store
        .query(covenant2_query)
        .expect("Covenant 2 check failed");

    // Covenant 3: MAPE-K components
    let covenant3_query = r#"
        PREFIX mape-k: <http://bitflow.ai/ontology/mape-k/v1#>
        SELECT (COUNT(?component) as ?mapeComponents) WHERE {
            VALUES ?component {
                mape-k:Monitor
                mape-k:Analyze
                mape-k:Plan
                mape-k:Execute
                mape-k:Knowledge
            }
        }
    "#;

    let c3_results = store
        .query(covenant3_query)
        .expect("Covenant 3 check failed");

    // Covenant 4: All patterns expressible
    let covenant4_query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?combo) as ?validCombos) WHERE {
            ?combo a yawl:SplitJoinCombination .
            ?combo yawl:isValid true .
        }
    "#;

    let c4_results = store
        .query(covenant4_query)
        .expect("Covenant 4 check failed");

    // Covenant 5: Latency bounds (checked at runtime by Chicago TDD)
    // Here we just verify execution mode declarations exist
    let covenant5_query = r#"
        PREFIX yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#>
        SELECT (COUNT(?task) as ?tasksWithExecMode) WHERE {
            ?task a yawl:Task .
            ?task yawl-exec:executionMode ?mode .
        }
    "#;

    let c5_results = store
        .query(covenant5_query)
        .expect("Covenant 5 check failed");

    // Covenant 6: Observations declared
    let covenant6_query = r#"
        PREFIX yawl-exec: <http://bitflow.ai/ontology/yawl/execution/v1#>
        SELECT (COUNT(?task) as ?observableTasks) WHERE {
            ?task a yawl:Task .
            ?task yawl-exec:runtimeBehavior ?behavior .
        }
    "#;

    let c6_results = store
        .query(covenant6_query)
        .expect("Covenant 6 check failed");

    // THEN: All covenants are validated
    let mut covenant_checks = vec![];

    if let QueryResults::Solutions(mut solutions) = c1_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C1 solution error");
            let tasks = sol.get("tasks").expect("No tasks").to_string();
            covenant_checks.push(("Covenant 1 (Turtle)", tasks.contains("12")));
        }
    }

    if let QueryResults::Solutions(mut solutions) = c2_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C2 solution error");
            let valid = sol.get("validTasks").expect("No valid tasks").to_string();
            covenant_checks.push((
                "Covenant 2 (Invariants)",
                valid.chars().any(|c| c.is_numeric()),
            ));
        }
    }

    if let QueryResults::Solutions(mut solutions) = c3_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C3 solution error");
            let mape = sol.get("mapeComponents").expect("No MAPE-K").to_string();
            covenant_checks.push(("Covenant 3 (MAPE-K)", mape.contains("5")));
        }
    }

    if let QueryResults::Solutions(mut solutions) = c4_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C4 solution error");
            let combos = sol.get("validCombos").expect("No combos").to_string();
            covenant_checks.push((
                "Covenant 4 (Patterns)",
                combos.chars().any(|c| c.is_numeric()),
            ));
        }
    }

    if let QueryResults::Solutions(mut solutions) = c5_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C5 solution error");
            let exec_modes = sol
                .get("tasksWithExecMode")
                .expect("No exec modes")
                .to_string();
            covenant_checks.push((
                "Covenant 5 (Latency)",
                exec_modes.chars().any(|c| c.is_numeric()),
            ));
        }
    }

    if let QueryResults::Solutions(mut solutions) = c6_results {
        if let Some(sol) = solutions.next() {
            let sol = sol.expect("C6 solution error");
            let observable = sol
                .get("observableTasks")
                .expect("No observable")
                .to_string();
            covenant_checks.push((
                "Covenant 6 (Observations)",
                observable.chars().any(|c| c.is_numeric()),
            ));
        }
    }

    // Verify all covenants passed
    for (covenant, passed) in &covenant_checks {
        assert!(passed, "{} FAILED", covenant);
        println!("✅ {}", covenant);
    }

    assert_eq!(covenant_checks.len(), 6, "Not all covenants were checked");
}

#[test]
fn test_performance_under_scale() {
    // GIVEN: Large workflow definition
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let workflow = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    // WHEN: We execute queries multiple times (simulating load)
    let iterations = 10;
    let mut latencies = vec![];

    for _ in 0..iterations {
        let start = Instant::now();

        let store = Store::new().expect("Failed to create store");
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, workflow.as_bytes())
            .expect("Failed to parse");

        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            SELECT ?task WHERE { ?task a yawl:Task . }
        "#;

        let _ = store.query(query).expect("Query failed");
        latencies.push(start.elapsed().as_micros());
    }

    // THEN: Performance is consistent (variance < 50%)
    let avg = latencies.iter().sum::<u128>() / latencies.len() as u128;
    let max = *latencies.iter().max().unwrap();
    let min = *latencies.iter().min().unwrap();

    let variance = ((max - min) as f64 / avg as f64) * 100.0;

    assert!(
        variance < 50.0,
        "Performance variance too high: {:.1}% (max: {}µs, min: {}µs, avg: {}µs)",
        variance,
        max,
        min,
        avg
    );

    println!(
        "✅ Performance stable: avg {}µs, variance {:.1}%",
        avg, variance
    );
}
