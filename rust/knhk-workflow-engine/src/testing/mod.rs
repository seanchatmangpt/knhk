//! Chicago TDD testing module
//!
//! Provides Chicago TDD testing framework, generators, coverage analysis,
//! property-based testing, and mutation testing for workflows.

pub mod chicago_tdd;
pub mod coverage;
pub mod generator;
pub mod mutation;
pub mod property;

pub use chicago_tdd::{
    assert_pattern_failure, assert_pattern_has_next_state, assert_pattern_has_variable,
    assert_pattern_success, assert_pattern_variable_equals, create_sequential_workflow,
    create_simple_sequential_workflow, create_test_capability, create_test_context,
    create_test_context_for_workflow, create_test_context_with_vars, create_test_registry,
    create_test_resource, create_test_role, create_test_worklet, ConditionBuilder,
    IntegrationTestHelper, PerformanceTestHelper, TaskBuilder, WorkflowPropertyTester,
    WorkflowSpecBuilder, WorkflowTestFixture,
};
// TestDataBuilder is now in chicago-tdd-tools - import directly:
// use chicago_tdd_tools::builders::TestDataBuilder;
pub use coverage::{CoverageAnalyzer, CoverageReport};
pub use generator::WorkflowTestGenerator;
pub use mutation::{MutationOperator, MutationScore, MutationTester};
pub use property::{
    property_all_workflows_registrable, property_all_workflows_valid_structure,
    property_workflow_execution_terminates, PropertyTestGenerator,
};
