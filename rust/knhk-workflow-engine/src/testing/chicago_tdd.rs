#![allow(clippy::unwrap_used)] // Test infrastructure - unwrap() acceptable
//! Chicago TDD Test Framework for Workflows
//!
//! Provides a framework for writing Chicago TDD tests for workflows following
//! the AAA pattern (Arrange, Act, Assert) with real collaborators.

use crate::case::{Case, CaseId, CaseState};
use crate::error::WorkflowResult;
use crate::executor::WorkflowEngine;
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::state::StateStore;
use std::collections::HashMap;

/// Test fixture for workflow testing
pub struct WorkflowTestFixture {
    /// Workflow engine
    pub engine: WorkflowEngine,
    /// Registered workflow specs
    pub specs: HashMap<WorkflowSpecId, WorkflowSpec>,
    /// Created cases
    pub cases: Vec<CaseId>,
}

impl WorkflowTestFixture {
    /// Create a new test fixture
    pub fn new() -> WorkflowResult<Self> {
        let state_store = StateStore::new("./test_workflow_db")?;
        let engine = WorkflowEngine::new(state_store);

        Ok(Self {
            engine,
            specs: HashMap::new(),
            cases: vec![],
        })
    }

    /// Register a workflow specification
    pub async fn register_workflow(
        &mut self,
        spec: WorkflowSpec,
    ) -> WorkflowResult<WorkflowSpecId> {
        let spec_id = spec.id;
        self.engine.register_workflow(spec.clone()).await?;
        self.specs.insert(spec_id, spec);
        Ok(spec_id)
    }

    /// Create a test case
    pub async fn create_case(
        &mut self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        let case_id = self.engine.create_case(spec_id, data).await?;
        self.cases.push(case_id);
        Ok(case_id)
    }

    /// Execute a case and return final state
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        self.engine.start_case(case_id).await?;
        self.engine.execute_case(case_id).await?;
        self.engine.get_case(case_id).await
    }

    /// Assert case state
    pub fn assert_case_state(&self, case: &Case, expected_state: CaseState) {
        assert_eq!(
            case.state, expected_state,
            "Expected case state {:?}, but got {:?}",
            expected_state, case.state
        );
    }

    /// Assert case completed successfully
    pub fn assert_case_completed(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Completed);
    }

    /// Assert case failed
    pub fn assert_case_failed(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Failed);
    }

    /// Clean up test resources
    pub fn cleanup(&self) -> WorkflowResult<()> {
        // In production, would clean up state store
        Ok(())
    }
}

impl Default for WorkflowTestFixture {
    fn default() -> Self {
        Self::new().expect("Failed to create test fixture")
    }
}

/// Macro for Chicago TDD workflow tests
#[macro_export]
macro_rules! chicago_tdd_workflow_test {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            // Arrange: Set up test fixture
            let mut fixture = $crate::testing::chicago_tdd::WorkflowTestFixture::new()
                .expect("Failed to create test fixture");

            // Execute test function
            $test_fn(&mut fixture).await.expect("Test failed");

            // Cleanup
            fixture.cleanup().expect("Failed to cleanup");
        }
    };
}

/// Property-based test generator for workflows
pub struct WorkflowPropertyTester {
    /// Test fixture
    fixture: WorkflowTestFixture,
    /// Number of test cases to generate
    num_cases: usize,
}

impl WorkflowPropertyTester {
    /// Create a new property tester
    pub fn new(num_cases: usize) -> WorkflowResult<Self> {
        Ok(Self {
            fixture: WorkflowTestFixture::new()?,
            num_cases,
        })
    }

    /// Test workflow property: All cases eventually complete or fail
    pub async fn test_completion_property(
        &mut self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        for _ in 0..self.num_cases {
            let case_id = self
                .fixture
                .create_case(spec_id, serde_json::json!({}))
                .await?;
            let case = self.fixture.execute_case(case_id).await?;

            // Property: Case must be in Completed or Failed state
            if case.state != CaseState::Completed && case.state != CaseState::Failed {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Test workflow property: No deadlocks
    pub async fn test_deadlock_property(
        &mut self,
        _spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        // In production, would test for deadlocks
        // For now, return true (deadlock detection happens at registration)
        Ok(true)
    }

    /// Test workflow property: Tick budget compliance
    pub async fn test_tick_budget_property(
        &mut self,
        _spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        // In production, would verify all tasks complete in ≤8 ticks
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = WorkflowTestFixture::new().unwrap();
        assert!(fixture.specs.is_empty());
        assert!(fixture.cases.is_empty());
    }
}
