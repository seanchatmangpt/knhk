//! SHACL Soundness Validation Tests
//!
//! **Chicago TDD**: Test-first validation of SHACL-based workflow soundness.
//!
//! These tests validate Van der Aalst's soundness criteria using SHACL shapes:
//! 1. Option to Complete: Every workflow can reach output from input
//! 2. Proper Completion: Output is only marked place at completion
//! 3. No Dead Tasks: All tasks reachable in some execution path
//!
//! **80/20 Approach**: Practical soundness validation, not theoretical perfection.

use knhk_workflow_engine::validation::shacl::{ShaclValidator, ValidationSeverity};

// ===========================================================================
// Test Fixtures: Valid and Invalid Workflows
// ===========================================================================

const SOUND_WORKFLOW_SIMPLE: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/task1> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/task1> .

    <http://example.org/task1> a yawl:Task ;
        yawl:flowsInto <http://example.org/flow2> .

    <http://example.org/flow2> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/output1> .

    <http://example.org/output1> a yawl:OutputCondition .
"#;

const SOUND_WORKFLOW_AND_SPLIT: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/split_task> ;
        yawl:hasTask <http://example.org/task_a> ;
        yawl:hasTask <http://example.org/task_b> ;
        yawl:hasTask <http://example.org/join_task> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/split_task> .

    <http://example.org/split_task> a yawl:Task ;
        yawl:hasSplit yawl:ControlTypeAnd ;
        yawl:flowsInto <http://example.org/flow2> ;
        yawl:flowsInto <http://example.org/flow3> .

    <http://example.org/flow2> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/task_a> .

    <http://example.org/flow3> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/task_b> .

    <http://example.org/task_a> a yawl:Task ;
        yawl:flowsInto <http://example.org/flow4> .

    <http://example.org/task_b> a yawl:Task ;
        yawl:flowsInto <http://example.org/flow5> .

    <http://example.org/flow4> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/join_task> .

    <http://example.org/flow5> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/join_task> .

    <http://example.org/join_task> a yawl:Task ;
        yawl:hasJoin yawl:ControlTypeAnd ;
        yawl:flowsInto <http://example.org/flow6> .

    <http://example.org/flow6> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/output1> .

    <http://example.org/output1> a yawl:OutputCondition .
"#;

const UNSOUND_NO_INPUT_CONDITION: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/task1> .

    <http://example.org/task1> a yawl:Task .
    <http://example.org/output1> a yawl:OutputCondition .
"#;

const UNSOUND_NO_OUTPUT_CONDITION: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasTask <http://example.org/task1> .

    <http://example.org/input1> a yawl:InputCondition .
    <http://example.org/task1> a yawl:Task .
"#;

const UNSOUND_UNREACHABLE_TASK: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/connected_task> ;
        yawl:hasTask <http://example.org/orphan_task> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/connected_task> .

    <http://example.org/connected_task> a yawl:Task ;
        yawl:flowsInto <http://example.org/flow2> .

    <http://example.org/flow2> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/output1> .

    <http://example.org/orphan_task> a yawl:Task .
    <http://example.org/output1> a yawl:OutputCondition .
"#;

const UNSOUND_DEAD_END_TASK: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/deadend> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/deadend> .

    <http://example.org/deadend> a yawl:Task .
    <http://example.org/output1> a yawl:OutputCondition .
"#;

const UNSOUND_INPUT_HAS_INCOMING: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/task1> .

    <http://example.org/task1> a yawl:Task ;
        yawl:flowsInto <http://example.org/cycle_flow> .

    <http://example.org/cycle_flow> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/input1> .

    <http://example.org/input1> a yawl:InputCondition .
    <http://example.org/output1> a yawl:OutputCondition .
"#;

const UNSOUND_OUTPUT_HAS_OUTGOING: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> .

    <http://example.org/input1> a yawl:InputCondition .

    <http://example.org/output1> a yawl:OutputCondition ;
        yawl:flowsInto <http://example.org/cycle_flow> .

    <http://example.org/cycle_flow> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/input1> .
"#;

const WARNING_DEGENERATE_XOR_SPLIT: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/xor_task> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/xor_task> .

    <http://example.org/xor_task> a yawl:Task ;
        yawl:hasSplit yawl:ControlTypeXor ;
        yawl:flowsInto <http://example.org/flow2> .

    <http://example.org/flow2> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/output1> .

    <http://example.org/output1> a yawl:OutputCondition .
"#;

const WARNING_DEGENERATE_AND_JOIN: &str = r#"
    @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

    <http://example.org/workflow1> a yawl:Specification ;
        yawl:hasInputCondition <http://example.org/input1> ;
        yawl:hasOutputCondition <http://example.org/output1> ;
        yawl:hasTask <http://example.org/and_task> .

    <http://example.org/input1> a yawl:InputCondition ;
        yawl:flowsInto <http://example.org/flow1> .

    <http://example.org/flow1> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/and_task> .

    <http://example.org/and_task> a yawl:Task ;
        yawl:hasJoin yawl:ControlTypeAnd ;
        yawl:flowsInto <http://example.org/flow2> .

    <http://example.org/flow2> a yawl:FlowsInto ;
        yawl:nextElementRef <http://example.org/output1> .

    <http://example.org/output1> a yawl:OutputCondition .
"#;

// ===========================================================================
// Tests: VR-S001 - Input Condition Required
// ===========================================================================

#[test]
fn test_vr_s001_sound_workflow_has_input_condition() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(SOUND_WORKFLOW_SIMPLE).unwrap();

    assert!(
        report.conforms,
        "Sound workflow should have input condition"
    );
    assert_eq!(
        report.count_by_severity(ValidationSeverity::Violation),
        0,
        "Should have no violations"
    );
}

#[test]
fn test_vr_s001_detects_missing_input_condition() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(UNSOUND_NO_INPUT_CONDITION)
        .unwrap();

    assert!(!report.conforms, "Should detect missing input condition");
    assert!(report.has_violations(), "Should have violations");

    let vr_s001_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S001")
        .collect();

    assert_eq!(
        vr_s001_violations.len(),
        1,
        "Should have exactly one VR-S001 violation"
    );
    assert!(
        vr_s001_violations[0].message.contains("input condition"),
        "Violation message should mention input condition"
    );
}

// ===========================================================================
// Tests: VR-S002 - Output Condition Required
// ===========================================================================

#[test]
fn test_vr_s002_sound_workflow_has_output_condition() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(SOUND_WORKFLOW_SIMPLE).unwrap();

    assert!(
        report.conforms,
        "Sound workflow should have output condition"
    );
}

#[test]
fn test_vr_s002_detects_missing_output_condition() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(UNSOUND_NO_OUTPUT_CONDITION)
        .unwrap();

    assert!(!report.conforms, "Should detect missing output condition");
    assert!(report.has_violations());

    let vr_s002_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S002")
        .collect();

    assert_eq!(
        vr_s002_violations.len(),
        1,
        "Should have exactly one VR-S002 violation"
    );
    assert!(
        vr_s002_violations[0].message.contains("output condition"),
        "Violation message should mention output condition"
    );
}

// ===========================================================================
// Tests: VR-S003 - All Tasks Reachable
// ===========================================================================

#[test]
fn test_vr_s003_sound_workflow_all_tasks_reachable() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(SOUND_WORKFLOW_AND_SPLIT)
        .unwrap();

    assert!(
        report.conforms,
        "Sound workflow should have all tasks reachable"
    );
}

#[test]
fn test_vr_s003_detects_unreachable_task() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(UNSOUND_UNREACHABLE_TASK)
        .unwrap();

    assert!(!report.conforms, "Should detect unreachable task");
    assert!(report.has_violations());

    let vr_s003_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S003")
        .collect();

    assert_eq!(
        vr_s003_violations.len(),
        1,
        "Should have exactly one VR-S003 violation for orphan task"
    );
    assert!(
        vr_s003_violations[0].message.contains("unreachable"),
        "Violation message should mention unreachable"
    );
}

// ===========================================================================
// Tests: VR-S004 - All Tasks Have Outgoing Flows
// ===========================================================================

#[test]
fn test_vr_s004_sound_workflow_all_tasks_have_outgoing() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(SOUND_WORKFLOW_SIMPLE).unwrap();

    assert!(
        report.conforms,
        "Sound workflow should have all tasks with outgoing flows"
    );
}

#[test]
fn test_vr_s004_detects_dead_end_task() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(UNSOUND_DEAD_END_TASK).unwrap();

    assert!(!report.conforms, "Should detect dead end task");
    assert!(report.has_violations());

    let vr_s004_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S004")
        .collect();

    assert_eq!(
        vr_s004_violations.len(),
        1,
        "Should have exactly one VR-S004 violation for dead end"
    );
    assert!(
        vr_s004_violations[0].message.contains("dead end"),
        "Violation message should mention dead end"
    );
}

// ===========================================================================
// Tests: VR-S005 - XOR Split Multiple Outgoing
// ===========================================================================

#[test]
fn test_vr_s005_warns_on_degenerate_xor_split() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(WARNING_DEGENERATE_XOR_SPLIT)
        .unwrap();

    assert!(!report.conforms, "Should detect degenerate XOR split");
    assert!(report.has_warnings(), "Should have warnings");

    let vr_s005_warnings: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S005" && v.severity == ValidationSeverity::Warning)
        .collect();

    assert!(
        !vr_s005_warnings.is_empty(),
        "Should have VR-S005 warning for degenerate XOR split"
    );
}

// ===========================================================================
// Tests: VR-S007 - AND Join Multiple Incoming
// ===========================================================================

#[test]
fn test_vr_s007_warns_on_degenerate_and_join() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(WARNING_DEGENERATE_AND_JOIN)
        .unwrap();

    assert!(!report.conforms, "Should detect degenerate AND join");
    assert!(report.has_warnings(), "Should have warnings");

    let vr_s007_warnings: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S007" && v.severity == ValidationSeverity::Warning)
        .collect();

    assert!(
        !vr_s007_warnings.is_empty(),
        "Should have VR-S007 warning for degenerate AND join"
    );
}

// ===========================================================================
// Tests: VR-S009 - Input Condition No Incoming
// ===========================================================================

#[test]
fn test_vr_s009_detects_input_condition_with_incoming_flow() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(UNSOUND_INPUT_HAS_INCOMING)
        .unwrap();

    assert!(
        !report.conforms,
        "Should detect input condition with incoming flow"
    );
    assert!(report.has_violations());

    let vr_s009_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S009")
        .collect();

    assert!(
        !vr_s009_violations.is_empty(),
        "Should have VR-S009 violation for input condition with incoming flow"
    );
}

// ===========================================================================
// Tests: VR-S010 - Output Condition No Outgoing
// ===========================================================================

#[test]
fn test_vr_s010_detects_output_condition_with_outgoing_flow() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(UNSOUND_OUTPUT_HAS_OUTGOING)
        .unwrap();

    assert!(
        !report.conforms,
        "Should detect output condition with outgoing flow"
    );
    assert!(report.has_violations());

    let vr_s010_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule_id == "VR-S010")
        .collect();

    assert!(
        !vr_s010_violations.is_empty(),
        "Should have VR-S010 violation for output condition with outgoing flow"
    );
}

// ===========================================================================
// Integration Tests: Complete Soundness Validation
// ===========================================================================

#[test]
fn test_complete_soundness_validation_sound_workflow() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(SOUND_WORKFLOW_SIMPLE).unwrap();

    assert!(report.conforms, "Sound workflow should pass all validation");
    assert_eq!(report.violations.len(), 0, "Should have zero violations");
    assert!(!report.has_violations(), "Should not have violations");
    assert!(!report.has_warnings(), "Should not have warnings");
}

#[test]
fn test_complete_soundness_validation_and_split_workflow() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(SOUND_WORKFLOW_AND_SPLIT)
        .unwrap();

    assert!(
        report.conforms,
        "AND split/join workflow should pass all validation"
    );
    assert_eq!(report.violations.len(), 0, "Should have zero violations");
}

#[test]
fn test_validation_report_severity_counts() {
    let validator = ShaclValidator::new().unwrap();
    let report = validator
        .validate_soundness(WARNING_DEGENERATE_XOR_SPLIT)
        .unwrap();

    assert_eq!(
        report.count_by_severity(ValidationSeverity::Violation),
        0,
        "Degenerate XOR should have no violations"
    );
    assert!(
        report.count_by_severity(ValidationSeverity::Warning) > 0,
        "Degenerate XOR should have warnings"
    );
}

#[test]
fn test_multiple_violations_detected() {
    // Workflow with both no input and no output
    let bad_workflow = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

        <http://example.org/workflow1> a yawl:Specification ;
            yawl:hasTask <http://example.org/task1> .

        <http://example.org/task1> a yawl:Task .
    "#;

    let validator = ShaclValidator::new().unwrap();
    let report = validator.validate_soundness(bad_workflow).unwrap();

    assert!(!report.conforms, "Bad workflow should not conform");
    assert!(
        report.violations.len() >= 2,
        "Should detect multiple violations (no input + no output)"
    );

    let has_s001 = report.violations.iter().any(|v| v.rule_id == "VR-S001");
    let has_s002 = report.violations.iter().any(|v| v.rule_id == "VR-S002");

    assert!(has_s001, "Should detect missing input condition");
    assert!(has_s002, "Should detect missing output condition");
}

// ===========================================================================
// Performance Tests: Validation Should Be Fast
// ===========================================================================

#[test]
fn test_soundness_validation_performance() {
    use std::time::Instant;

    let validator = ShaclValidator::new().unwrap();

    let start = Instant::now();
    let report = validator
        .validate_soundness(SOUND_WORKFLOW_AND_SPLIT)
        .unwrap();
    let duration = start.elapsed();

    assert!(report.conforms, "Validation should succeed");
    assert!(
        duration.as_millis() < 100,
        "Soundness validation should complete in <100ms, took {:?}",
        duration
    );
}

#[test]
fn test_validator_reuse() {
    // Validator should be reusable across multiple validations
    let validator = ShaclValidator::new().unwrap();

    let report1 = validator.validate_soundness(SOUND_WORKFLOW_SIMPLE).unwrap();
    let report2 = validator
        .validate_soundness(SOUND_WORKFLOW_AND_SPLIT)
        .unwrap();
    let report3 = validator
        .validate_soundness(UNSOUND_NO_INPUT_CONDITION)
        .unwrap();

    assert!(report1.conforms);
    assert!(report2.conforms);
    assert!(!report3.conforms);
}
