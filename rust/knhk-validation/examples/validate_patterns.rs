// Example: Pattern Validation with Coverage Report
// Demonstrates how to use the Pattern Matrix Validator
// Shows permutation decomposition and coverage analysis

use knhk_validation::pattern::{
    CancellationType, IterationType, JoinType, PatternModifiers, PatternValidator, SplitType,
    TaskPattern, WorkflowDefinition,
};

fn main() {
    println!("=== Pattern Matrix Validator Example ===\n");

    // Create validator
    let validator = match PatternValidator::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to create validator: {}", e);
            return;
        }
    };

    // Example 1: Validate individual patterns
    println!("--- Example 1: Individual Pattern Validation ---\n");

    // Sequence pattern
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task1_sequence".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            PatternModifiers::default(),
        ),
    );

    // Parallel split with synchronization
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task2_parallel_sync".to_string(),
            SplitType::AND,
            JoinType::AND,
            PatternModifiers::default(),
        ),
    );

    // Exclusive choice
    let mut choice_mods = PatternModifiers::default();
    choice_mods.flow_predicate = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task3_exclusive_choice".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            choice_mods,
        ),
    );

    // Multi-choice
    let mut multi_mods = PatternModifiers::default();
    multi_mods.flow_predicate = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task4_multi_choice".to_string(),
            SplitType::OR,
            JoinType::XOR,
            multi_mods,
        ),
    );

    // Discriminator
    let mut disc_mods = PatternModifiers::default();
    disc_mods.quorum = Some(1);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task5_discriminator".to_string(),
            SplitType::AND,
            JoinType::Discriminator,
            disc_mods,
        ),
    );

    // Arbitrary cycles
    let mut cycle_mods = PatternModifiers::default();
    cycle_mods.backward_flow = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task6_cycles".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            cycle_mods,
        ),
    );

    // Deferred choice
    let mut deferred_mods = PatternModifiers::default();
    deferred_mods.deferred_choice = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task7_deferred".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            deferred_mods,
        ),
    );

    // Interleaved parallel
    let mut interleave_mods = PatternModifiers::default();
    interleave_mods.interleaving = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task8_interleaved".to_string(),
            SplitType::AND,
            JoinType::AND,
            interleave_mods,
        ),
    );

    // Critical section
    let mut critical_mods = PatternModifiers::default();
    critical_mods.critical_section = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task9_critical".to_string(),
            SplitType::AND,
            JoinType::AND,
            critical_mods,
        ),
    );

    // Milestone
    let mut milestone_mods = PatternModifiers::default();
    milestone_mods.milestone = true;
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task10_milestone".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            milestone_mods,
        ),
    );

    // Cancellation patterns
    let mut cancel_task_mods = PatternModifiers::default();
    cancel_task_mods.cancellation = Some(CancellationType::Task);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task11_cancel_task".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            cancel_task_mods,
        ),
    );

    let mut cancel_case_mods = PatternModifiers::default();
    cancel_case_mods.cancellation = Some(CancellationType::Case);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task12_cancel_case".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            cancel_case_mods,
        ),
    );

    let mut cancel_region_mods = PatternModifiers::default();
    cancel_region_mods.cancellation = Some(CancellationType::Region);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task13_cancel_region".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            cancel_region_mods,
        ),
    );

    // Iteration patterns
    let mut loop_mods = PatternModifiers::default();
    loop_mods.iteration = Some(IterationType::StructuredLoop);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task14_loop".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            loop_mods,
        ),
    );

    let mut recursion_mods = PatternModifiers::default();
    recursion_mods.iteration = Some(IterationType::Recursion);
    validate_and_print(
        &validator,
        TaskPattern::new(
            "task15_recursion".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            recursion_mods,
        ),
    );

    // Example 2: Validate a complete workflow
    println!("\n--- Example 2: Complete Workflow Validation ---\n");

    let mut workflow = WorkflowDefinition::new("autonomous_workflow".to_string());

    // Add all the tasks
    workflow.add_task(TaskPattern::new(
        "initiate".to_string(),
        SplitType::XOR,
        JoinType::XOR,
        PatternModifiers::default(),
    ));

    workflow.add_task(TaskPattern::new(
        "parallel_processing".to_string(),
        SplitType::AND,
        JoinType::AND,
        PatternModifiers::default(),
    ));

    let mut choice_mods2 = PatternModifiers::default();
    choice_mods2.flow_predicate = true;
    workflow.add_task(TaskPattern::new(
        "decision_routing".to_string(),
        SplitType::XOR,
        JoinType::XOR,
        choice_mods2,
    ));

    let result = validator.validate_workflow_complete(&workflow);
    println!("Workflow '{}' validation:", workflow.workflow_id);
    println!("  Valid: {}", result.is_valid);
    if !result.errors.is_empty() {
        println!("  Errors:");
        for error in &result.errors {
            println!("    - {}", error);
        }
    }
    if !result.warnings.is_empty() {
        println!("  Warnings:");
        for warning in &result.warnings {
            println!("    - {}", warning);
        }
    }
    println!();

    // Example 3: Pattern decomposition
    println!("\n--- Example 3: Pattern Decomposition ---\n");

    let patterns_to_decompose = vec![
        "Sequence",
        "ParallelSplit",
        "ExclusiveChoice",
        "Discriminator",
        "ArbitraryCycles",
    ];

    for pattern_name in patterns_to_decompose {
        match validator.decompose_pattern(pattern_name) {
            Ok(combinations) => {
                println!("Pattern '{}' decomposes into {} combination(s):", pattern_name, combinations.len());
                for (i, combo) in combinations.iter().enumerate() {
                    println!(
                        "  {}. {}-{} (generates: {:?})",
                        i + 1,
                        combo.split_type.as_str(),
                        combo.join_type.as_str(),
                        combo.generated_patterns
                    );
                    println!("     {}", combo.comment);
                }
            }
            Err(e) => {
                println!("Pattern '{}' not found: {}", pattern_name, e);
            }
        }
        println!();
    }

    // Example 4: Coverage Report
    println!("\n--- Example 4: Coverage Report ---\n");

    let report = validator.coverage_report();
    report.print_summary();

    // Example 5: Invalid pattern detection
    println!("\n--- Example 5: Invalid Pattern Detection ---\n");

    // Try an invalid combination (XOR-AND)
    let invalid_task = TaskPattern::new(
        "invalid_xor_and".to_string(),
        SplitType::XOR,
        JoinType::AND,
        PatternModifiers::default(),
    );
    let result = validator.validate_task(&invalid_task);
    println!("Invalid pattern (XOR-AND) validation:");
    println!("  Valid: {}", result.is_valid);
    if !result.errors.is_empty() {
        println!("  Errors:");
        for error in &result.errors {
            println!("    - {}", error);
        }
    }
    if !result.suggestions.is_empty() {
        println!("  Suggestions:");
        for suggestion in &result.suggestions {
            println!("    - {}", suggestion);
        }
    }
    println!();

    // Try interleaving with wrong split type
    let mut bad_interleave_mods = PatternModifiers::default();
    bad_interleave_mods.interleaving = true;
    let bad_interleave = TaskPattern::new(
        "bad_interleave".to_string(),
        SplitType::XOR, // Should be AND
        JoinType::XOR,
        bad_interleave_mods,
    );
    let result = validator.validate_task(&bad_interleave);
    println!("Invalid pattern (Interleaving with XOR) validation:");
    println!("  Valid: {}", result.is_valid);
    if !result.errors.is_empty() {
        println!("  Errors:");
        for error in &result.errors {
            println!("    - {}", error);
        }
    }
    println!();

    println!("=== Example Complete ===");
}

fn validate_and_print(validator: &PatternValidator, task: TaskPattern) {
    let result = validator.validate_task(&task);
    println!("Task '{}' ({}-{}):", task.task_id, task.split_type.as_str(), task.join_type.as_str());
    println!("  Valid: {}", result.is_valid);
    if let Some(pattern_name) = &result.pattern_name {
        println!("  Pattern: {}", pattern_name);
    }
    if !result.errors.is_empty() {
        println!("  Errors: {}", result.errors.join("; "));
    }
    if !result.warnings.is_empty() {
        println!("  Warnings: {}", result.warnings.join("; "));
    }
    if !result.suggestions.is_empty() {
        println!("  Suggestions: {}", result.suggestions.join("; "));
    }
    println!();
}
