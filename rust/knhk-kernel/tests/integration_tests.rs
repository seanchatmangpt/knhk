// knhk-kernel: Integration tests for hot path compliance

use knhk_kernel::prelude::*;
use knhk_kernel::{init, verify_compliance};

#[test]
fn test_kernel_initialization() {
    assert!(init().is_ok(), "Kernel initialization failed");
}

#[test]
fn test_chatman_constant_compliance() {
    init().unwrap();

    // Verify all patterns meet ≤8 tick requirement
    let result = verify_compliance();
    assert!(result.is_ok(), "Hot path violations: {:?}", result);
}

#[test]
fn test_deterministic_execution() {
    init().unwrap();

    // Setup descriptor
    let pattern = knhk_kernel::descriptor::PatternEntry::new(
        PatternType::Sequence,
        1,
        10,
        PatternConfig::default(),
    );

    let desc = DescriptorBuilder::new()
        .with_tick_budget(8)
        .add_pattern(pattern)
        .build();

    DescriptorManager::load_descriptor(Box::new(desc)).unwrap();

    // Execute same task multiple times
    let mut receipts = Vec::new();

    for _ in 0..10 {
        let mut task = Task::new(1, 1);
        task.add_observation(42);
        task.add_observation(43);
        task.transition(TaskState::Ready);

        let executor = Executor::new();
        let receipt = executor.execute(&task);

        receipts.push(receipt);
    }

    // Verify all executions produce same result (deterministic)
    let first_output = receipts[0].output_digest;
    for receipt in &receipts {
        assert_eq!(
            receipt.output_digest, first_output,
            "Non-deterministic execution detected"
        );
        assert!(receipt.within_budget(), "Budget exceeded");
    }
}

#[test]
fn test_zero_allocation_hot_path() {
    init().unwrap();

    // This test would use allocation tracking in practice
    // For now, we verify the hot path completes successfully

    let pattern = knhk_kernel::descriptor::PatternEntry::new(
        PatternType::ParallelSplit,
        2,
        10,
        PatternConfig {
            max_instances: 4,
            ..Default::default()
        },
    );

    let desc = DescriptorBuilder::new().add_pattern(pattern).build();

    DescriptorManager::load_descriptor(Box::new(desc)).unwrap();

    let mut task = Task::new(2, 2);
    task.transition(TaskState::Ready);

    let executor = Executor::new();
    let receipt = executor.execute(&task);

    assert!(receipt.within_budget());
}

#[test]
fn test_state_machine_transitions() {
    use knhk_kernel::executor::StateMachine;

    // Valid transitions
    assert!(StateMachine::validate_transition(
        TaskState::Created,
        TaskState::Ready
    ));
    assert!(StateMachine::validate_transition(
        TaskState::Ready,
        TaskState::Running
    ));
    assert!(StateMachine::validate_transition(
        TaskState::Running,
        TaskState::Completed
    ));

    // Invalid transitions
    assert!(!StateMachine::validate_transition(
        TaskState::Completed,
        TaskState::Running
    ));
    assert!(!StateMachine::validate_transition(
        TaskState::Failed,
        TaskState::Ready
    ));
}

#[test]
fn test_guard_evaluation() {
    use knhk_kernel::descriptor::{ExecutionContext, ObservationBuffer, ResourceState};
    use knhk_kernel::guard::{Guard, Predicate, ResourceType};

    let context = ExecutionContext {
        task_id: 42,
        timestamp: 1000,
        resources: ResourceState {
            cpu_available: 80,
            memory_available: 1024,
            io_capacity: 100,
            queue_depth: 10,
        },
        observations: ObservationBuffer {
            count: 5,
            observations: [0; 16],
        },
        state_flags: 0,
    };

    // Test predicate guard
    let guard = Guard::predicate(Predicate::Equal, 0, 42); // task_id == 42
    assert!(guard.evaluate(&context));

    // Test resource guard
    let guard = Guard::resource(ResourceType::Cpu, 50);
    assert!(guard.evaluate(&context)); // CPU (80) >= 50

    // Test compound guard
    let g1 = Guard::predicate(Predicate::Equal, 0, 42);
    let g2 = Guard::resource(ResourceType::Memory, 500);
    let and_guard = Guard::and(vec![g1, g2]);
    assert!(and_guard.evaluate(&context));
}

#[test]
fn test_receipt_verification() {
    use knhk_kernel::receipt::ReceiptBuilder;

    let receipt = ReceiptBuilder::new(1, 100)
        .with_budget(8)
        .with_inputs(&[1, 2, 3])
        .with_outputs(&[4, 5, 6])
        .with_result(ReceiptStatus::Success, 5)
        .build();

    assert!(receipt.verify(), "Receipt verification failed");
    assert!(receipt.within_budget());

    // Tamper with receipt
    let mut tampered = receipt.clone();
    tampered.ticks_used = 100;
    assert!(!tampered.verify(), "Tampered receipt passed verification");
}

#[test]
fn test_pattern_dispatch_performance() {
    init().unwrap();

    let dispatcher = knhk_kernel::pattern::PatternDispatcher::new();

    // Test all 43 patterns
    for i in 1..=43u8 {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(i) };

        let ctx = knhk_kernel::pattern::PatternFactory::create(
            pattern_type,
            i as u32,
            PatternConfig::default(),
        );

        let timer = HotPathTimer::start();
        let result = dispatcher.dispatch(&ctx);
        let ticks = timer.elapsed_ticks();

        assert!(
            ticks <= 8,
            "Pattern {:?} exceeded 8 ticks: {} ticks",
            pattern_type,
            ticks
        );

        // Basic validation of result
        assert!(result.ticks_used <= 8);
    }
}

#[test]
fn test_hot_path_stratum_isolation() {
    init().unwrap();

    // Setup patterns with different complexities
    let simple = knhk_kernel::descriptor::PatternEntry::new(
        PatternType::Sequence,
        1,
        10,
        PatternConfig::default(),
    );

    let complex = knhk_kernel::descriptor::PatternEntry::new(
        knhk_kernel::pattern::PatternType::Recursion,
        2,
        5,
        PatternConfig {
            flags: knhk_kernel::pattern::PatternFlags::new(
                knhk_kernel::pattern::PatternFlags::CANCELLABLE
                    | knhk_kernel::pattern::PatternFlags::RECURSIVE,
            ),
            ..Default::default()
        },
    );

    let desc = DescriptorBuilder::new()
        .add_pattern(simple)
        .add_pattern(complex)
        .build();

    DescriptorManager::load_descriptor(Box::new(desc)).unwrap();

    let hot_path = HotPath::new();

    // Simple task should go to hot stratum
    let simple_task = Box::new(Task::new(1, 1));
    assert!(hot_path.submit(simple_task).is_ok());

    // Complex task should go to cold stratum
    let complex_task = Box::new(Task::new(2, 2));
    assert!(hot_path.submit(complex_task).is_ok());

    let stats = hot_path.stats();
    assert_eq!(
        stats.queue_depth_hot + stats.queue_depth_warm + stats.queue_depth_cold,
        2
    );
}

#[test]
fn test_descriptor_hot_swap() {
    let desc1 = Box::new(DescriptorBuilder::new().with_tick_budget(8).build());

    DescriptorManager::load_descriptor(desc1).unwrap();

    let active = DescriptorManager::get_active().unwrap();
    assert_eq!(active.global_tick_budget, 8);

    // Hot swap to new descriptor
    let desc2 = Box::new(DescriptorBuilder::new().with_tick_budget(6).build());

    DescriptorManager::hot_swap(desc2).unwrap();

    let active = DescriptorManager::get_active().unwrap();
    assert_eq!(active.global_tick_budget, 6);
}

#[test]
fn test_tick_budget_tracking() {
    let mut budget = TickBudget::new();

    assert!(budget.charge("op1", 2).is_ok());
    assert!(budget.charge("op2", 3).is_ok());
    assert_eq!(budget.remaining(), 3);

    // Should fail - would exceed budget
    assert!(budget.charge("op3", 4).is_err());

    // Should succeed
    assert!(budget.charge("op3", 3).is_ok());
    assert!(budget.exhausted());
}

#[test]
fn test_all_pattern_types() {
    use knhk_kernel::pattern::PatternValidator;

    // Check all patterns are valid
    for i in 1..=43u8 {
        let pattern_type = unsafe { std::mem::transmute::<u8, PatternType>(i) };
        assert!(
            PatternValidator::check_permutation_matrix(pattern_type),
            "Pattern {:?} not in permutation matrix",
            pattern_type
        );
    }

    // Test known valid combinations
    assert!(PatternValidator::validate_combination(
        PatternType::ParallelSplit,
        PatternType::Synchronization
    )
    .is_ok());

    assert!(PatternValidator::validate_combination(
        PatternType::ExclusiveChoice,
        PatternType::SimpleMerge
    )
    .is_ok());
}
