//! Comprehensive Tests for Const-Eval Timing Analysis
//!
//! This test suite validates compile-time timing analysis and guarantees.

use knhk_mu_kernel::patterns::PatternId;
use knhk_mu_kernel::timing_const::budgets::*;
use knhk_mu_kernel::timing_const::proofs::*;
use knhk_mu_kernel::timing_const::wcet::*;
use knhk_mu_kernel::timing_const::*;
use knhk_mu_kernel::CHATMAN_CONSTANT;

/// Test const tick cost trait implementations
#[test]
fn test_const_tick_cost_implementations() {
    // Verify pattern costs
    assert_eq!(SequencePattern::TICK_COST, 1);
    assert_eq!(ParallelSplitPattern::TICK_COST, 2);
    assert_eq!(SynchronizationPattern::TICK_COST, 3);
    assert_eq!(ExclusiveChoicePattern::TICK_COST, 1);
    assert_eq!(MultiChoicePattern::TICK_COST, 3);

    // Verify μ-op costs
    assert_eq!(GuardEvalOp::TICK_COST, 1);
    assert_eq!(ReceiptWriteOp::TICK_COST, 1);
    assert_eq!(SigmaLoadOp::TICK_COST, 1);
    assert_eq!(PatternDispatchOp::TICK_COST, 1);

    // Verify hot path eligibility
    assert!(SequencePattern::IS_HOT_PATH);
    assert!(ParallelSplitPattern::IS_HOT_PATH);
    assert!(SynchronizationPattern::IS_HOT_PATH);

    // Verify names
    assert_eq!(SequencePattern::NAME, "Sequence");
    assert_eq!(GuardEvalOp::NAME, "GuardEval");
}

/// Test compile-time tick cost calculations
#[test]
fn test_total_tick_cost_calculation() {
    const COSTS1: [u64; 5] = [1, 2, 1, 3, 1];
    const TOTAL1: u64 = total_tick_cost(COSTS1);
    assert_eq!(TOTAL1, 8);

    const COSTS2: [u64; 3] = [2, 3, 2];
    const TOTAL2: u64 = total_tick_cost(COSTS2);
    assert_eq!(TOTAL2, 7);

    const EMPTY: [u64; 0] = [];
    const ZERO: u64 = total_tick_cost(EMPTY);
    assert_eq!(ZERO, 0);
}

/// Test Chatman Constant compliance checking
#[test]
fn test_within_chatman_constant() {
    const VALID1: bool = within_chatman([1, 2, 3, 2]);
    assert!(VALID1);

    const VALID2: bool = within_chatman([4, 4]);
    assert!(VALID2);

    const EDGE_CASE: bool = within_chatman([8]);
    assert!(EDGE_CASE);

    const INVALID: bool = within_chatman([5, 5]);
    assert!(!INVALID);

    const INVALID2: bool = within_chatman([9]);
    assert!(!INVALID2);
}

/// Test task WCET computation
#[test]
fn test_compute_task_wcet() {
    // Sequence pattern (cost 1) + 2 guards
    // load(1) + dispatch(1) + guards(2) + pattern(1) + receipt(1) = 6
    const WCET1: u64 = compute_task_wcet(1, 2);
    assert_eq!(WCET1, 6);

    // Parallel split (cost 2) + 3 guards
    // load(1) + dispatch(1) + guards(3) + pattern(2) + receipt(1) = 8
    const WCET2: u64 = compute_task_wcet(2, 3);
    assert_eq!(WCET2, 8);

    // Synchronization (cost 3) + 4 guards
    // load(1) + dispatch(1) + guards(4) + pattern(3) + receipt(1) = 10
    const WCET3: u64 = compute_task_wcet(3, 4);
    assert_eq!(WCET3, 10); // Exceeds Chatman, but function doesn't enforce

    // Maximum hot-path eligible: pattern(3) + 3 guards
    // load(1) + dispatch(1) + guards(3) + pattern(3) + receipt(1) = 9
    // This would exceed Chatman
}

/// Test cost composition
#[test]
fn test_cost_composition() {
    const COMPOSED1: u64 = compose_costs(3, 4);
    assert_eq!(COMPOSED1, 7);

    const COMPOSED2: u64 = compose_costs(1, 1);
    assert_eq!(COMPOSED2, 2);

    const MAX1: u64 = max_cost(5, 3);
    assert_eq!(MAX1, 5);

    const MIN1: u64 = min_cost(5, 3);
    assert_eq!(MIN1, 3);
}

/// Test parallel vs sequential costs
#[test]
fn test_parallel_vs_sequential_costs() {
    const OPS: [u64; 4] = [2, 3, 1, 2];

    const SEQUENTIAL: u64 = sequential_cost(OPS);
    assert_eq!(SEQUENTIAL, 8);

    const PARALLEL: u64 = parallel_cost(OPS);
    assert_eq!(PARALLEL, 3); // Maximum branch

    // Parallel should always be ≤ sequential
    assert!(PARALLEL <= SEQUENTIAL);
}

/// Test WCET analyzer for patterns
#[test]
fn test_wcet_analyzer_patterns() {
    const SEQ_WCET: WcetResult = WcetAnalyzer::analyze_pattern(PatternId::Sequence);
    assert_eq!(SEQ_WCET.worst_case_ticks, 1);
    assert_eq!(SEQ_WCET.best_case_ticks, 1);
    assert!(SEQ_WCET.is_hot_path);

    const PARA_WCET: WcetResult = WcetAnalyzer::analyze_pattern(PatternId::ParallelSplit);
    assert_eq!(PARA_WCET.worst_case_ticks, 2);
    assert!(PARA_WCET.is_hot_path);

    const SYNC_WCET: WcetResult = WcetAnalyzer::analyze_pattern(PatternId::Synchronization);
    assert_eq!(SYNC_WCET.worst_case_ticks, 3);
    assert!(SYNC_WCET.is_hot_path);
}

/// Test WCET analyzer for complete tasks
#[test]
fn test_wcet_analyzer_tasks() {
    // Sequence + 2 guards = 6 ticks
    const TASK1: WcetResult = WcetAnalyzer::analyze_task(PatternId::Sequence, 2);
    assert_eq!(TASK1.worst_case_ticks, 6);
    assert!(TASK1.is_hot_path);
    assert!(TASK1.within_budget(8));

    // Parallel split + 3 guards = 8 ticks (at Chatman limit)
    const TASK2: WcetResult = WcetAnalyzer::analyze_task(PatternId::ParallelSplit, 3);
    assert_eq!(TASK2.worst_case_ticks, 8);
    assert!(TASK2.is_hot_path);
    assert!(TASK2.within_budget(CHATMAN_CONSTANT));
}

/// Test WCET sequential composition
#[test]
fn test_wcet_sequential_composition() {
    const PHASES: [WcetPhase; 4] = [
        WcetPhase::new("load", 1, true),
        WcetPhase::new("dispatch", 1, true),
        WcetPhase::new("guard", 2, true),
        WcetPhase::new("pattern", 3, true),
    ];

    const RESULT: WcetResult = WcetAnalyzer::analyze_sequential(PHASES);
    assert_eq!(RESULT.worst_case_ticks, 7);
    assert_eq!(RESULT.best_case_ticks, 7); // All required
}

/// Test WCET parallel composition
#[test]
fn test_wcet_parallel_composition() {
    const BRANCHES: [u64; 4] = [2, 5, 3, 4];
    const RESULT: WcetResult = WcetAnalyzer::analyze_parallel(BRANCHES);

    assert_eq!(RESULT.worst_case_ticks, 5); // Maximum branch
    assert_eq!(RESULT.best_case_ticks, 2); // Minimum branch
    assert!(RESULT.is_hot_path);
}

/// Test WCET conditional analysis
#[test]
fn test_wcet_conditional_analysis() {
    const RESULT: WcetResult = WcetAnalyzer::analyze_conditional(
        1, // condition evaluation
        4, // true branch
        3, // false branch
    );

    assert_eq!(RESULT.worst_case_ticks, 5); // 1 + max(4, 3)
    assert_eq!(RESULT.best_case_ticks, 4); // 1 + min(4, 3)
    assert!(RESULT.is_hot_path);
}

/// Test const generic budgets
#[test]
fn test_const_generic_budgets() {
    let budget = ConstBudget::<8, 0>::new();
    assert_eq!(ConstBudget::<8, 0>::initial(), 8);
    assert_eq!(ConstBudget::<8, 0>::used(), 0);
    assert_eq!(ConstBudget::<8, 0>::remaining(), 8);

    let budget = budget.spend::<3>();
    assert_eq!(ConstBudget::<8, 3>::used(), 3);
    assert_eq!(ConstBudget::<8, 3>::remaining(), 5);

    let budget = budget.spend::<2>();
    assert_eq!(ConstBudget::<8, 5>::used(), 5);
    assert_eq!(ConstBudget::<8, 5>::remaining(), 3);

    // Compile-time enforcement:
    // let _invalid = budget.spend::<5>(); // Would fail: 5 + 5 > 8
}

/// Test Chatman budget specialization
#[test]
fn test_chatman_budget() {
    let budget = ChatmanBudget::<0>::chatman();
    let budget = budget.spend::<3>();
    let budget = budget.spend::<2>();

    assert_eq!(ChatmanBudget::<5>::remaining(), 3);
    assert_eq!(ChatmanBudget::<5>::utilization_percent(), 62); // 5/8 * 100
}

/// Test budget exhaustion detection
#[test]
fn test_budget_exhaustion() {
    assert!(!ConstBudget::<8, 0>::is_exhausted());
    assert!(!ConstBudget::<8, 5>::is_exhausted());
    assert!(!ConstBudget::<8, 7>::is_exhausted());
    assert!(ConstBudget::<8, 8>::is_exhausted());
}

/// Test budget deduction
#[test]
fn test_budget_deduction() {
    const REMAINING1: u64 = deduct_budget(8, 3);
    assert_eq!(REMAINING1, 5);

    const REMAINING2: u64 = deduct_budget(8, 8);
    assert_eq!(REMAINING2, 0);

    assert!(fits_in_budget(5, 8));
    assert!(fits_in_budget(8, 8));
    assert!(!fits_in_budget(9, 8));
}

/// Test budget composition
#[test]
fn test_budget_composition() {
    const COSTS: [u64; 4] = [1, 2, 3, 2];
    const TOTAL: u64 = compose_budgets(COSTS);
    assert_eq!(TOTAL, 8);

    const BRANCHES: [u64; 4] = [2, 5, 3, 4];
    const MAX: u64 = parallel_budgets(BRANCHES);
    assert_eq!(MAX, 5);
}

/// Test budget allocation
#[test]
fn test_budget_allocation() {
    const ALLOC: BudgetAllocation<5> = BudgetAllocation::new([1, 2, 3, 1, 1]);

    assert_eq!(ALLOC.total, 8);
    assert!(ALLOC.is_valid());
    assert_eq!(ALLOC.get_allocation(0).unwrap(), 1);
    assert_eq!(ALLOC.get_allocation(2).unwrap(), 3);
    assert_eq!(ALLOC.get_allocation(4).unwrap(), 1);
}

/// Test budget validation
#[test]
fn test_budget_validation() {
    const VALID_COSTS: [u64; 4] = [1, 2, 3, 2];
    const VALID: bool = validate_budget_sequence(VALID_COSTS, 8);
    assert!(VALID);

    const INVALID_COSTS: [u64; 3] = [5, 5, 5];
    const INVALID: bool = validate_budget_sequence(INVALID_COSTS, 8);
    assert!(!INVALID);
}

/// Test timing proofs
#[test]
fn test_timing_proofs() {
    let proof = TimingProof::<5>::new(ProofStrength::Strong, [1, 2, 3, 4]);

    assert_eq!(TimingProof::<5>::worst_case(), 5);
    assert_eq!(TimingProof::<5>::safety_margin(), 3);
    assert!(proof.verify());

    // Compile-time enforcement:
    // let _invalid = TimingProof::<10>::new(...); // Would fail: 10 > 8
}

/// Test proof strength ordering
#[test]
fn test_proof_strength() {
    let weak = TimingProof::<5>::new(ProofStrength::Weak, [0; 4]);
    let medium = TimingProof::<5>::new(ProofStrength::Medium, [0; 4]);
    let strong = TimingProof::<5>::new(ProofStrength::Strong, [0; 4]);
    let absolute = TimingProof::<5>::new(ProofStrength::Absolute, [0; 4]);

    assert!(weak.is_strong_enough(ProofStrength::Weak));
    assert!(!weak.is_strong_enough(ProofStrength::Medium));

    assert!(medium.is_strong_enough(ProofStrength::Weak));
    assert!(medium.is_strong_enough(ProofStrength::Medium));
    assert!(!medium.is_strong_enough(ProofStrength::Strong));

    assert!(strong.is_strong_enough(ProofStrength::Strong));
    assert!(!strong.is_strong_enough(ProofStrength::Absolute));

    assert!(absolute.is_strong_enough(ProofStrength::Absolute));
}

/// Test timing certificates
#[test]
fn test_timing_certificates() {
    let wcet = WcetResult::new(6, 5, 5);
    let cert = TimingCertificate::new(1, wcet, ProofStrength::Strong, [1, 2, 3, 4], 1000);

    assert!(cert.verify());
    assert!(cert.is_valid(1500, 1000)); // 500 units old, max age 1000
    assert!(!cert.is_valid(2500, 1000)); // 1500 units old, exceeds max age
}

/// Test composition proofs
#[test]
fn test_composition_proofs() {
    let ops = [1u64, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let proof = CompositionProof::<8>::new(ops, 4);

    assert_eq!(CompositionProof::<8>::total_wcet(), 8);
    assert!(proof.verify());
}

/// Test parallel proofs
#[test]
fn test_parallel_proofs() {
    let branches = [2u64, 5, 3, 4, 0, 0, 0, 0];
    let proof = ParallelProof::<5>::new(branches, 4);

    assert_eq!(ParallelProof::<5>::max_wcet(), 5);
    assert!(proof.verify());
}

/// Test loop proofs
#[test]
fn test_loop_proofs() {
    let proof = LoopProof::<4, 2>::new(4, 2);

    assert_eq!(LoopProof::<4, 2>::total_wcet(), 8);
    assert!(proof.verify());

    // Compile-time enforcement:
    // let _invalid = LoopProof::<5, 2>::new(5, 2); // Would fail: 5 * 2 > 8
}

/// Test conditional proofs
#[test]
fn test_conditional_proofs() {
    let proof = ConditionalProof::<1, 4>::new(1, 4, 3);

    assert_eq!(ConditionalProof::<1, 4>::total_wcet(), 5);
    assert!(proof.verify());
}

/// Test proof composition
#[test]
fn test_proof_composition() {
    const COMPOSED1: u64 = compose_timing_proofs(3, 4);
    assert_eq!(COMPOSED1, 7);

    const COMPOSED2: u64 = compose_timing_proofs(1, 2);
    assert_eq!(COMPOSED2, 3);

    const PARALLEL1: u64 = parallel_timing_proofs(5, 3);
    assert_eq!(PARALLEL1, 5);

    const PARALLEL2: u64 = parallel_timing_proofs(2, 4);
    assert_eq!(PARALLEL2, 4);
}

/// Integration test: Full task analysis
#[test]
fn test_full_task_analysis() {
    // Task: Sequence pattern with 2 guards
    const PATTERN_COST: u64 = 1;
    const GUARD_COUNT: u64 = 2;

    // Compute WCET
    const TASK_WCET: WcetResult = WcetAnalyzer::analyze_task(PatternId::Sequence, GUARD_COUNT);

    // Verify within Chatman
    assert_eq!(TASK_WCET.worst_case_ticks, 6);
    assert!(TASK_WCET.is_hot_path);

    // Create proof
    let proof = TimingProof::<6>::new(
        ProofStrength::Strong,
        [TASK_WCET.worst_case_ticks, TASK_WCET.best_case_ticks, 0, 0],
    );

    assert!(proof.verify());
    assert_eq!(TimingProof::<6>::safety_margin(), 2);

    // Create certificate
    let cert = TimingCertificate::new(1, TASK_WCET, ProofStrength::Strong, [1, 2, 3, 4], 1000);

    assert!(cert.verify());
}

/// Integration test: Complex workflow with parallel and sequential
#[test]
fn test_complex_workflow_analysis() {
    // Workflow:
    // 1. Load and dispatch (2 ticks)
    // 2. Parallel branches: [2, 3, 2] (max 3 ticks)
    // 3. Synchronization (1 tick)
    // 4. Receipt (1 tick)
    // Total: 2 + 3 + 1 + 1 = 7 ticks

    const SETUP_COST: u64 = 2;
    const PARALLEL_BRANCHES: [u64; 3] = [2, 3, 2];
    const PARALLEL_COST: u64 = parallel_cost(PARALLEL_BRANCHES);
    const SYNC_COST: u64 = 1;
    const RECEIPT_COST: u64 = 1;

    const TOTAL_WCET: u64 = SETUP_COST + PARALLEL_COST + SYNC_COST + RECEIPT_COST;

    assert_eq!(TOTAL_WCET, 7);
    assert!(TOTAL_WCET <= CHATMAN_CONSTANT);

    // Create proof
    let proof = TimingProof::<7>::new(ProofStrength::Strong, [TOTAL_WCET, 0, 0, 0]);

    assert!(proof.verify());
}

/// Stress test: Maximum complexity within Chatman
#[test]
fn test_maximum_complexity_within_chatman() {
    // Design a task that uses exactly 8 ticks
    const PHASES: [u64; 8] = [1, 1, 1, 1, 1, 1, 1, 1];
    const TOTAL: u64 = total_tick_cost(PHASES);

    assert_eq!(TOTAL, CHATMAN_CONSTANT);

    let proof = TimingProof::<8>::new(ProofStrength::Strong, [TOTAL, 0, 0, 0]);

    assert!(proof.verify());
    assert_eq!(TimingProof::<8>::safety_margin(), 0); // No margin!
}

/// Edge case test: Zero cost operations
#[test]
fn test_zero_cost_operations() {
    const BUDGET: ConstBudget<8, 0> = ConstBudget::new();
    let _budget = BUDGET.spend::<0>(); // Spending 0 is valid

    assert_eq!(ConstBudget::<8, 0>::remaining(), 8);
}

/// Edge case test: Single tick operations
#[test]
fn test_single_tick_operations() {
    const WCET: WcetResult = WcetAnalyzer::analyze_mu_op(0);

    assert_eq!(WCET.worst_case_ticks, 1);
    assert_eq!(WCET.best_case_ticks, 1);
    assert_eq!(WCET.variability(), 0); // Deterministic
}
