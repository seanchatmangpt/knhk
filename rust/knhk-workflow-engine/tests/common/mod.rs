//! 80/20 Test Harness - Common fixtures and helpers
//!
//! Eliminates 90+ instances of duplicated setup code across 28 test files.
//! Provides:
//! - TestHarness: One-line engine + state store setup
//! - WorkflowBuilder: Fluent API for test workflows
//! - Assertions: Common validation patterns

#![allow(dead_code)]

use knhk_workflow_engine::*;

/// 80/20: Single fixture that eliminates 90+ lines of duplicated setup
pub struct TestHarness {
    pub _temp_dir: tempfile::TempDir,
    pub engine: executor::WorkflowEngine,
    pub parser: parser::WorkflowParser,
}

impl TestHarness {
    /// Parse workflow from Turtle string
    pub fn parse(&mut self, turtle: &str) -> parser::WorkflowSpec {
        self.parser
            .parse_turtle(turtle)
            .expect("Should parse workflow")
    }
}

impl TestHarness {
    /// Create test harness with all components initialized
    pub fn new() -> Self {
        let _temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let state_store =
            state::StateStore::new(_temp_dir.path()).expect("Failed to create state store");
        let engine = executor::WorkflowEngine::new(state_store);
        let parser = parser::WorkflowParser::new().expect("Failed to create parser");

        Self {
            _temp_dir,
            engine,
            parser,
        }
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// 80/20: Fluent builder for test workflows
pub struct WorkflowBuilder {
    turtle: String,
}

impl WorkflowBuilder {
    pub fn new() -> Self {
        Self {
            turtle: String::from(
                r#"@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

"#,
            ),
        }
    }

    /// Add a simple workflow with input/output conditions
    pub fn simple_workflow(mut self, name: &str) -> Self {
        self.turtle.push_str(&format!(
            r#"<http://example.org/workflow> a yawl:Specification ;
    yawl:specName "{}" ;
    yawl:hasInputCondition <http://example.org/input> ;
    yawl:hasOutputCondition <http://example.org/output> .

<http://example.org/input> a yawl:InputCondition ;
    yawl:conditionName "Start" .

<http://example.org/output> a yawl:OutputCondition ;
    yawl:conditionName "End" .

"#,
            name
        ));
        self
    }

    /// Add a task to the workflow
    pub fn with_task(mut self, task_name: &str) -> Self {
        let task_id = task_name.replace(' ', "");
        self.turtle.push_str(&format!(
            r#"<http://example.org/workflow> yawl:hasTask <http://example.org/task_{}> .

<http://example.org/task_{}> a yawl:AtomicTask ;
    yawl:taskName "{}" ;
    yawl:join "XOR" ;
    yawl:split "XOR" .

"#,
            task_id, task_id, task_name
        ));
        self
    }

    /// Add parallel split pattern (Pattern 2)
    pub fn with_parallel_split(mut self) -> Self {
        self.turtle.push_str(
            r#"<http://example.org/parallel_task> a yawl:AtomicTask ;
    yawl:taskName "ParallelSplit" ;
    yawl:split yawl:ControlTypeAnd .

"#,
        );
        self
    }

    /// Add synchronization pattern (Pattern 3)
    pub fn with_synchronization(mut self) -> Self {
        self.turtle.push_str(
            r#"<http://example.org/sync_task> a yawl:AtomicTask ;
    yawl:taskName "Synchronization" ;
    yawl:join yawl:ControlTypeAnd .

"#,
        );
        self
    }

    /// Build the Turtle string
    pub fn build(self) -> String {
        self.turtle
    }
}

impl Default for WorkflowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 80/20: Common assertion helpers
pub mod assertions {
    use knhk_workflow_engine::case::CaseState;
    use knhk_workflow_engine::*;

    /// Assert case completed successfully
    pub async fn assert_case_completed(engine: &executor::WorkflowEngine, case_id: CaseId) {
        let case = engine.get_case(case_id).await.expect("Should get case");
        assert_eq!(
            case.state,
            CaseState::Completed,
            "Case should complete successfully"
        );
    }

    /// Assert XES export is valid XML
    pub fn assert_valid_xes(xes: &str) {
        assert!(
            xes.contains("<?xml version"),
            "XES should have XML declaration"
        );
        assert!(
            xes.contains("<log xes.version="),
            "XES should have log element"
        );
        assert!(xes.contains("</log>"), "XES should close log element");
    }
}

/// 80/20: Test data constants
pub mod data {
    use serde_json::json;

    pub fn simple_case_data() -> serde_json::Value {
        json!({
            "orderAmount": 100.0,
            "customerId": "TEST-001"
        })
    }
}

/// 80/20: Timing helpers for performance tests
pub mod timing {
    use std::time::{Duration, Instant};

    pub struct TimedOperation {
        start: Instant,
    }

    impl TimedOperation {
        pub fn start() -> Self {
            Self {
                start: Instant::now(),
            }
        }

        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }

        pub fn assert_under_ms(&self, max_ms: u64) {
            let elapsed = self.elapsed();
            assert!(
                elapsed.as_millis() < max_ms as u128,
                "Operation took {}ms, expected <{}ms",
                elapsed.as_millis(),
                max_ms
            );
        }

        pub fn assert_under_ticks(&self, max_ticks: u64) {
            // Chatman Constant: 1 tick ≈ 12.5ms
            let max_ms = max_ticks * 12;
            self.assert_under_ms(max_ms);
        }
    }
}
