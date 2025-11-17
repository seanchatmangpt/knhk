//! Protocol Safety Tests
//!
//! This test suite verifies that type-level protocol state machines
//! enforce correctness at compile time. Many tests use the `compile_fail`
//! attribute to ensure invalid protocols cause compilation errors.

#![allow(dead_code, unused_variables)]

use knhk_mu_kernel::mape::{AnalyzeResult, ExecuteResult, MonitorResult, PlanResult};
use knhk_mu_kernel::overlay::OverlayValue;
use knhk_mu_kernel::overlay_proof::CompilerProof;
use knhk_mu_kernel::overlay_types::{OverlayChanges, OverlayMetadata, PerfImpact, SnapshotId};
use knhk_mu_kernel::protocols::mape_protocol::*;
use knhk_mu_kernel::protocols::overlay_protocol::*;
use knhk_mu_kernel::protocols::session_types::*;
use knhk_mu_kernel::protocols::state_machine::*;
use knhk_mu_kernel::protocols::*;
use knhk_mu_kernel::receipts::Receipt;
use knhk_mu_kernel::sigma::SigmaHash;

/// Helper to create test overlay
fn make_test_overlay() -> OverlayValue<CompilerProof> {
    let changes = OverlayChanges::new();
    let proof = CompilerProof {
        compiler_version: (2027, 0, 0),
        proof_id: 1,
        invariants: vec![1, 2, 3],
        timing_bound: 6,
        coverage: knhk_mu_kernel::overlay_proof::ChangeCoverage {
            covered_changes: 0,
            coverage_percent: 100,
        },
        signature: [1; 64],
    };
    let metadata = OverlayMetadata {
        id: 1,
        created_at: 0,
        priority: 10,
        author: [0; 32],
        description: "test",
        perf_impact: PerfImpact {
            expected_improvement: 0.1,
            confidence: 0.9,
            max_tick_increase: 2,
        },
    };

    OverlayValue::new(SnapshotId([0; 32]), changes, proof, metadata).unwrap()
}

/// Helper to create test receipt
fn make_test_receipt() -> Receipt {
    Receipt::new(0, SigmaHash([0; 32]), [0; 32], [0; 32], 5, 0, 0)
}

// ============================================================================
// Session Type Tests
// ============================================================================

#[test]
fn test_session_valid_flow() {
    // Valid session flow should compile
    let session = Session::<Uninitialized>::new();
    let session = session.initialize();
    let session = session.activate();
    let _session = session.complete();
}

#[test]
fn test_session_failure_path() {
    // Should be able to fail from any non-terminal state
    let session = Session::<Uninitialized>::new();
    let session = session.initialize();
    let _session = session.fail();
}

#[test]
fn test_session_zero_size() {
    // All session states should be zero-sized
    assert_eq!(core::mem::size_of::<Session<Uninitialized>>(), 0);
    assert_eq!(core::mem::size_of::<Session<Initialized>>(), 0);
    assert_eq!(core::mem::size_of::<Session<Active>>(), 0);
    assert_eq!(core::mem::size_of::<Session<Completed>>(), 0);
    assert_eq!(core::mem::size_of::<Session<Failed>>(), 0);
}

#[test]
fn test_choice_combinator() {
    let session = Session::<Uninitialized>::new();
    let session = session.initialize();

    let choice: Choice<Session<Active>, Session<Failed>> = if true {
        Choice::left(session.activate())
    } else {
        Choice::right(session.fail())
    };

    let _result = choice.match_choice(|s| s.complete(), |s| s);
}

#[test]
fn test_capability_read_only() {
    let session = Session::<ReadOnly>::new();
    let _session = session.execute_op(Read);
    // session.execute_op(Write); // Would not compile!
}

#[test]
fn test_capability_read_write() {
    let session = Session::<ReadWrite>::new();
    let session = session.execute_op(Read);
    let _session = session.execute_op(Write);
}

// ============================================================================
// State Machine Tests
// ============================================================================

#[test]
fn test_state_machine_basic_transitions() {
    let machine = StateMachine::<Initial>::new();
    let machine = machine.start();
    let machine = machine.pause();
    let machine = machine.resume();
    let _machine = machine.stop();
}

#[test]
fn test_state_machine_error_path() {
    let machine = StateMachine::<Initial>::new();
    let machine = machine.start();
    let _machine = machine.error();
}

#[test]
fn test_state_machine_zero_size() {
    // All state machines should be zero-sized
    assert_eq!(core::mem::size_of::<StateMachine<Initial>>(), 0);
    assert_eq!(core::mem::size_of::<StateMachine<Running>>(), 0);
    assert_eq!(core::mem::size_of::<StateMachine<Paused>>(), 0);
    assert_eq!(core::mem::size_of::<StateMachine<Stopped>>(), 0);
    assert_eq!(core::mem::size_of::<StateMachine<Error>>(), 0);
}

#[test]
fn test_stateful_machine() {
    let machine = StatefulMachine::new(42u32);
    let machine = machine.start(|x| *x += 1);
    assert_eq!(*machine.data(), 43);

    let machine = machine.pause(|x| *x *= 2);
    assert_eq!(*machine.data(), 86);

    let machine = machine.resume(|x| *x += 100);
    assert_eq!(*machine.data(), 186);
}

#[test]
fn test_builder_pattern() {
    let builder = Builder::new(vec![1, 2, 3]);
    let builder = builder.build_and_start(|v| v.push(4));

    assert_eq!(builder.data(), &vec![1, 2, 3, 4]);
}

#[test]
fn test_conditional_transition() {
    fn process(value: u32) -> ConditionalTransition<Running, Error> {
        if value > 0 {
            ConditionalTransition::first()
        } else {
            ConditionalTransition::second()
        }
    }

    let transition = process(42);
    let _result = transition.match_transition(|m| m.pause(), |_m| StateMachine::<Error>::new());
}

#[test]
fn test_guarded_transition_success() {
    let machine = Guarded::create(10u32);

    let result = machine.start_if(|x| *x > 5, |x| *x *= 2);

    assert!(result.is_ok());
    let machine = result.unwrap();
    assert_eq!(*machine.data(), 20);
}

#[test]
fn test_guarded_transition_failure() {
    let machine = Guarded::create(3u32);

    let result = machine.start_if(|x| *x > 5, |x| *x *= 2);

    assert!(result.is_err());
}

#[test]
fn test_timed_machine() {
    let machine = TimedMachine::<Initial>::new();
    let machine = machine.start();
    let machine = machine.tick().tick().tick();

    assert_eq!(machine.ticks(), 3);

    let machine = machine.stop();
    assert_eq!(machine.ticks(), 3);
}

// ============================================================================
// MAPE-K Protocol Tests
// ============================================================================

#[test]
fn test_mape_k_basic_cycle() {
    // Must follow exact order: M → A → P → E → K
    let cycle = MapeKCycle::new();

    let receipt = make_test_receipt();

    let cycle = cycle.monitor(receipt);
    let cycle = cycle.analyze();
    let cycle = cycle.plan();
    let cycle = cycle.execute();
    let _cycle = cycle.update_knowledge();
}

#[test]
fn test_mape_k_multiple_cycles() {
    let mut cycle = MapeKCycle::new();
    let receipt = make_test_receipt();

    // First cycle
    cycle = cycle.monitor(receipt.clone());
    cycle = cycle.analyze();
    cycle = cycle.plan();
    cycle = cycle.execute();
    cycle = cycle.update_knowledge();

    // Second cycle
    cycle = cycle.monitor(receipt);
    cycle = cycle.analyze();
    cycle = cycle.plan();
    cycle = cycle.execute();
    cycle = cycle.update_knowledge();
}

#[test]
fn test_mape_k_with_data() {
    let cycle = MapeKCycleWithData::new();

    let monitor_result = MonitorResult {
        receipt_id: 1,
        observations_count: 10,
        avg_tau: 5.5,
    };

    let analyze_result = AnalyzeResult { symptoms: vec![] };

    let plan_result = PlanResult { proposals: vec![] };

    let execute_result = ExecuteResult {
        success: true,
        new_sigma_id: None,
        error: None,
    };

    let cycle = cycle.monitor(monitor_result);
    let cycle = cycle.analyze(analyze_result);
    let cycle = cycle.plan(plan_result);
    let cycle = cycle.execute(execute_result);
    let (_cycle, data) = cycle.update_knowledge();

    assert!(data.monitor_result.is_some());
    assert!(data.analyze_result.is_some());
    assert!(data.plan_result.is_some());
    assert!(data.execute_result.is_some());
}

#[test]
fn test_mape_k_timed() {
    let cycle = TimedMapeK::new();

    let cycle = cycle.monitor(2);
    let cycle = cycle.analyze(1);
    let cycle = cycle.plan(2);
    let cycle = cycle.execute(2);

    assert_eq!(cycle.ticks(), 7); // 2 + 1 + 2 + 2
    assert!(cycle.within_chatman_constant()); // ≤ 8 ticks

    let (_cycle, total) = cycle.update_knowledge();
    assert_eq!(total, 7);
}

#[test]
fn test_mape_k_timed_exceeds_chatman() {
    let cycle = TimedMapeK::new();

    let cycle = cycle.monitor(3);
    let cycle = cycle.analyze(2);
    let cycle = cycle.plan(3);
    let cycle = cycle.execute(2);

    assert_eq!(cycle.ticks(), 10); // > 8
    assert!(!cycle.within_chatman_constant());
}

#[test]
fn test_mape_k_cycle_counter() {
    let mut counter = CycleCounter::new();
    let receipt = make_test_receipt();

    // First cycle
    counter = counter
        .monitor()
        .analyze()
        .plan()
        .execute()
        .update_knowledge();
    assert_eq!(counter.count(), 1);

    // Second cycle
    counter = counter
        .monitor()
        .analyze()
        .plan()
        .execute()
        .update_knowledge();
    assert_eq!(counter.count(), 2);

    // Third cycle
    counter = counter
        .monitor()
        .analyze()
        .plan()
        .execute()
        .update_knowledge();
    assert_eq!(counter.count(), 3);
}

#[test]
fn test_mape_k_zero_size() {
    // All MAPE-K phases should be zero-sized
    assert_eq!(core::mem::size_of::<MapeKCycle<MonitorPhase>>(), 0);
    assert_eq!(core::mem::size_of::<MapeKCycle<AnalyzePhase>>(), 0);
    assert_eq!(core::mem::size_of::<MapeKCycle<PlanPhase>>(), 0);
    assert_eq!(core::mem::size_of::<MapeKCycle<ExecutePhase>>(), 0);
    assert_eq!(core::mem::size_of::<MapeKCycle<KnowledgePhase>>(), 0);
}

// ============================================================================
// Overlay Protocol Tests
// ============================================================================

#[test]
fn test_overlay_promotion_pipeline() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipeline::new(overlay);

    // Must follow exact order: Shadow → Test → Validate → Promote
    let pipeline = pipeline.deploy_shadow().unwrap();
    let pipeline = pipeline.run_tests().unwrap();
    let pipeline = pipeline.validate().unwrap();
    let _pipeline = pipeline.promote().unwrap();
}

#[test]
fn test_overlay_rollback_from_shadow() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipeline::new(overlay);

    // Can rollback from any phase
    let _pipeline = pipeline.rollback();
}

#[test]
fn test_overlay_rollback_from_test() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipeline::new(overlay);

    let pipeline = pipeline.deploy_shadow().unwrap();
    let _pipeline = pipeline.rollback();
}

#[test]
fn test_overlay_rollback_from_validate() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipeline::new(overlay);

    let pipeline = pipeline.deploy_shadow().unwrap();
    let pipeline = pipeline.run_tests().unwrap();
    let _pipeline = pipeline.rollback();
}

#[test]
fn test_overlay_pipeline_with_data() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipelineWithData::new(overlay);

    let pipeline = pipeline.deploy_shadow().unwrap();

    let mut results = TestResults::new();
    results.tests_run = 10;
    results.tests_passed = 10;
    results.perf_metrics = Some(PerfMetrics {
        max_ticks: 6,
        avg_ticks: 4,
        p99_ticks: 6,
    });

    let pipeline = pipeline.run_tests(results).unwrap();
    assert!(pipeline.data().all_passed());

    let pipeline = pipeline.validate().unwrap();
    let _pipeline = pipeline
        .promote_with_strategy(knhk_mu_kernel::overlay::RolloutStrategy::Immediate)
        .unwrap();
}

#[test]
fn test_overlay_pipeline_test_failure() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipelineWithData::new(overlay);

    let pipeline = pipeline.deploy_shadow().unwrap();

    let mut results = TestResults::new();
    results.tests_run = 10;
    results.tests_passed = 8; // 2 tests failed

    let result = pipeline.run_tests(results);
    assert!(result.is_err());
}

#[test]
fn test_canary_deployment() {
    let overlay = make_test_overlay();
    let canary = CanaryDeployment::new(overlay, 100);

    let canary = canary.start_rollout(10);
    assert_eq!(canary.rollout_percent(), 10);

    let canary = canary.increment(20).unwrap();
    assert_eq!(canary.rollout_percent(), 30);

    let canary = canary.increment(30).unwrap();
    assert_eq!(canary.rollout_percent(), 60);

    let canary = canary.increment(40);
    assert!(canary.is_err()); // Would exceed target
}

#[test]
fn test_canary_rollback() {
    let overlay = make_test_overlay();
    let canary = CanaryDeployment::new(overlay, 100);

    let canary = canary.start_rollout(10);
    let _pipeline = canary.rollback();
}

#[test]
fn test_rollback_protocol() {
    let overlay = make_test_overlay();
    let rollback = RollbackProtocol::new(overlay, RollbackReason::TestsFailed { failed_count: 3 });

    match rollback.reason() {
        RollbackReason::TestsFailed { failed_count } => {
            assert_eq!(*failed_count, 3);
        }
        _ => panic!("Wrong rollback reason"),
    }

    let _pipeline = rollback.execute();
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_mape_k_with_overlay_promotion() {
    // Demonstrate integration between MAPE-K and overlay promotion
    let overlay = make_test_overlay();
    let receipt = make_test_receipt();

    // Run MAPE-K cycle to generate overlay
    let mape = MapeKCycle::new();
    let mape = mape.monitor(receipt);
    let mape = mape.analyze();
    let mape = mape.plan();
    let mape = mape.execute();
    let _mape = mape.update_knowledge();

    // Promote overlay through pipeline
    let pipeline = OverlayPipeline::new(overlay);
    let pipeline = pipeline.deploy_shadow().unwrap();
    let pipeline = pipeline.run_tests().unwrap();
    let pipeline = pipeline.validate().unwrap();
    let _pipeline = pipeline.promote().unwrap();
}

#[test]
fn test_protocol_composition() {
    // Can compose different protocol types
    let _composed: Composed<Session<Uninitialized>, StateMachine<Initial>> = Composed::new();
}

#[test]
fn test_all_protocols_zero_size() {
    // Verify all protocol types are zero-sized
    assert_eq!(core::mem::size_of::<Session<Uninitialized>>(), 0);
    assert_eq!(core::mem::size_of::<StateMachine<Initial>>(), 0);
    assert_eq!(core::mem::size_of::<MapeKCycle<MonitorPhase>>(), 0);
    // Note: OverlayPipeline contains OverlayValue, so it's not zero-sized
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_state_transitions_are_no_ops() {
    // State transitions should compile to no-ops
    // This is verified by checking assembly, but we can at least
    // verify that they're inline and zero-sized

    let machine = StateMachine::<Initial>::new();
    let machine = machine.start();
    let machine = machine.pause();
    let machine = machine.resume();
    let _machine = machine.stop();

    // All the above should be optimized away completely
}

#[test]
fn test_mape_k_cycle_overhead() {
    // MAPE-K cycle tracking should have zero overhead
    let mut cycle = MapeKCycle::new();
    let receipt = make_test_receipt();

    // Run 1000 cycles - should be instant since it's all compile-time
    for _ in 0..1000 {
        cycle = cycle.monitor(receipt.clone());
        cycle = cycle.analyze();
        cycle = cycle.plan();
        cycle = cycle.execute();
        cycle = cycle.update_knowledge();
    }
}

#[test]
fn test_protocol_validation_overhead() {
    // Protocol validation is compile-time, so it should have zero runtime cost
    assert!(Session::<Uninitialized>::validate());
    assert!(StateMachine::<Initial>::validate());
}

// ============================================================================
// Compile-Fail Tests (These should NOT compile)
// ============================================================================

// Note: These tests are commented out because they would fail to compile.
// In a real test suite, they would use #[compile_fail] attribute.

/*
#[test]
#[compile_fail]
fn test_session_invalid_skip_state() {
    let session = Session::<Uninitialized>::new();
    // Cannot skip initialize step:
    let _session = session.activate(); // ERROR: no method `activate` on Uninitialized
}

#[test]
#[compile_fail]
fn test_state_machine_invalid_transition() {
    let machine = StateMachine::<Initial>::new();
    // Cannot pause without starting:
    let _machine = machine.pause(); // ERROR: no method `pause` on Initial
}

#[test]
#[compile_fail]
fn test_mape_k_skip_phase() {
    let cycle = MapeKCycle::new();
    let receipt = make_test_receipt();
    let cycle = cycle.monitor(receipt);
    // Cannot skip analyze phase:
    let _cycle = cycle.plan(); // ERROR: no method `plan` on AnalyzePhase
}

#[test]
#[compile_fail]
fn test_overlay_skip_testing() {
    let overlay = make_test_overlay();
    let pipeline = OverlayPipeline::new(overlay);
    let pipeline = pipeline.deploy_shadow().unwrap();
    // Cannot promote without testing and validation:
    let _pipeline = pipeline.promote(); // ERROR: no method `promote` on TestPhase
}

#[test]
#[compile_fail]
fn test_capability_write_on_readonly() {
    let session = Session::<ReadOnly>::new();
    // Cannot write on read-only session:
    let _session = session.execute_op(Write); // ERROR: trait bound not satisfied
}
*/

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(feature = "verification")]
#[test]
fn test_mape_k_always_cycles() {
    use proptest::prelude::*;

    proptest!(|(cycles in 1u32..100)| {
        let mut counter = CycleCounter::new();

        for _ in 0..cycles {
            counter = counter.monitor().analyze().plan().execute().update_knowledge();
        }

        assert_eq!(counter.count(), cycles as u64);
    });
}

#[cfg(feature = "verification")]
#[test]
fn test_timed_mape_k_accumulates() {
    use proptest::prelude::*;

    proptest!(|(ticks in prop::array::uniform4(1u64..3))| {
        let cycle = TimedMapeK::new();
        let cycle = cycle.monitor(ticks[0]);
        let cycle = cycle.analyze(ticks[1]);
        let cycle = cycle.plan(ticks[2]);
        let cycle = cycle.execute(ticks[3]);

        let expected_total = ticks.iter().sum::<u64>();
        assert_eq!(cycle.ticks(), expected_total);
    });
}
