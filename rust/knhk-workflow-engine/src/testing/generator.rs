#![allow(clippy::unwrap_used)] // Test infrastructure - unwrap() acceptable
//! Workflow test generator
//!
//! Generates Chicago TDD tests from workflow specifications.

use crate::error::WorkflowResult;
use crate::parser::WorkflowSpec;
use std::collections::HashMap;

/// Test generator for workflows
pub struct WorkflowTestGenerator {
    /// Generated test code
    tests: Vec<String>,
}

impl WorkflowTestGenerator {
    /// Create a new test generator
    pub fn new() -> Self {
        Self { tests: vec![] }
    }

    /// Generate Chicago TDD tests for a workflow specification
    pub fn generate_tests(&mut self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let mut test_code = String::from("//! Generated Chicago TDD tests for workflow\n");
        test_code.push_str("//! Generated from workflow specification\n\n");
        test_code
            .push_str("use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;\n");
        test_code.push_str("use knhk_workflow_engine::case::CaseState;\n\n");

        // Generate test for workflow registration
        test_code.push_str(&self.generate_registration_test(spec)?);

        // Generate test for case creation
        test_code.push_str(&self.generate_case_creation_test(spec)?);

        // Generate test for case execution
        test_code.push_str(&self.generate_execution_test(spec)?);

        // Generate tests for each task
        for (task_id, task) in &spec.tasks {
            test_code.push_str(&self.generate_task_test(spec, task_id, task)?);
        }

        Ok(test_code)
    }

    /// Generate registration test
    fn generate_registration_test(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let test_name = format!("test_workflow_{}_registration", sanitize_name(&spec.name));
        Ok(format!(
            r#"
#[tokio::test]
async fn {}() {{
    // Arrange: Set up test fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Act: Register workflow
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Assert: Workflow is registered
    assert!(fixture.specs.contains_key(&spec_id));
}}
"#,
            test_name
        ))
    }

    /// Generate case creation test
    fn generate_case_creation_test(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let test_name = format!("test_workflow_{}_case_creation", sanitize_name(&spec.name));
        Ok(format!(
            r#"
#[tokio::test]
async fn {}() {{
    // Arrange: Set up test fixture and register workflow
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Create case
    let case_id = fixture.create_case(spec_id, serde_json::json!({{}})).await.unwrap();
    
    // Assert: Case is created
    assert!(fixture.cases.contains(&case_id));
}}
"#,
            test_name
        ))
    }

    /// Generate execution test
    fn generate_execution_test(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let test_name = format!("test_workflow_{}_execution", sanitize_name(&spec.name));
        Ok(format!(
            r#"
#[tokio::test]
async fn {}() {{
    // Arrange: Set up test fixture and create case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({{}})).await.unwrap();
    
    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
}}
"#,
            test_name
        ))
    }

    /// Generate task test
    fn generate_task_test(
        &self,
        spec: &WorkflowSpec,
        task_id: &str,
        task: &crate::parser::Task,
    ) -> WorkflowResult<String> {
        let test_name = format!(
            "test_workflow_{}_task_{}",
            sanitize_name(&spec.name),
            sanitize_name(task_id)
        );
        Ok(format!(
            r#"
#[tokio::test]
async fn {}() {{
    // Arrange: Set up test fixture and create case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({{}})).await.unwrap();
    
    // Act: Execute case (task {} will be executed)
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
}}
"#,
            test_name, task.name
        ))
    }
}

/// Sanitize name for use in test function names
fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "_")
        .replace('-', "_")
        .replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

impl Default for WorkflowTestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My Workflow"), "my_workflow");
        assert_eq!(sanitize_name("test-workflow.v1"), "test_workflow_v1");
    }
}
