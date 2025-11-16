//! Core Phase Trait and Types
//!
//! Implements HKT-style phase system using phantom types for type-level programming.

use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::WorkflowEngine;

/// Phase execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus {
    /// Phase passed all checks
    Pass,
    /// Phase failed validation
    Fail,
    /// Phase completed with warnings
    Warning,
    /// Phase was skipped
    Skipped,
}

/// Phase execution result
#[derive(Debug, Clone)]
pub struct PhaseResult<T: Clone + Debug> {
    /// Phase name
    pub name: String,
    /// Phase status
    pub status: PhaseStatus,
    /// Number of checks passed
    pub passed: usize,
    /// Number of checks failed
    pub failed: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Execution duration
    pub duration: Duration,
    /// Phase-specific metrics
    pub metrics: HashMap<String, f64>,
    /// Phase-specific output data
    pub data: T,
    /// Detailed messages
    pub messages: Vec<String>,
}

impl<T: Clone + Debug> PhaseResult<T> {
    /// Create a new phase result
    pub fn new(name: impl Into<String>, status: PhaseStatus, data: T) -> Self {
        Self {
            name: name.into(),
            status,
            passed: 0,
            failed: 0,
            warnings: 0,
            duration: Duration::from_secs(0),
            metrics: HashMap::new(),
            data,
            messages: Vec::new(),
        }
    }

    /// Set execution duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add a metric
    pub fn add_metric(&mut self, key: impl Into<String>, value: f64) {
        self.metrics.insert(key.into(), value);
    }

    /// Add a message
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }

    /// Set counts
    pub fn with_counts(mut self, passed: usize, failed: usize, warnings: usize) -> Self {
        self.passed = passed;
        self.failed = failed;
        self.warnings = warnings;
        self
    }
}

/// Phase execution context
#[derive(Clone)]
pub struct PhaseContext {
    /// Workflow engine reference
    pub engine: Arc<WorkflowEngine>,
    /// Spec ID being validated
    pub spec_id: WorkflowSpecId,
    /// Context-specific data from previous phases
    pub context_data: Arc<HashMap<String, Vec<u8>>>,
}

impl PhaseContext {
    /// Create a new phase context
    pub fn new(engine: Arc<WorkflowEngine>, spec_id: WorkflowSpecId) -> Self {
        Self {
            engine,
            spec_id,
            context_data: Arc::new(HashMap::new()),
        }
    }

    /// Create context with shared data
    pub fn with_data(mut self, data: HashMap<String, Vec<u8>>) -> Self {
        self.context_data = Arc::new(data);
        self
    }
}

/// Phase metadata (compile-time information)
#[derive(Debug, Clone)]
pub struct PhaseMetadata {
    /// Phase name
    pub name: &'static str,
    /// Phase description
    pub description: &'static str,
    /// Phase version
    pub version: &'static str,
    /// Required dependencies (other phase names)
    pub dependencies: &'static [&'static str],
    /// Whether phase can run in parallel
    pub parallel: bool,
}

/// Core Phase trait - represents a validation phase
///
/// Uses phantom types for HKT-style composition:
/// - T: Phase-specific output type
/// - M: Phase metadata marker (const generic alternative)
///
/// # Example
/// ```ignore
/// struct MyPhase<M>(PhantomData<M>);
///
/// impl<M> Phase<MyOutputData, M> for MyPhase<M> {
///     fn metadata() -> PhaseMetadata { ... }
///     async fn execute(&self, ctx: PhaseContext) -> WorkflowResult<PhaseResult<MyOutputData>> { ... }
/// }
/// ```
pub trait Phase<T: Clone + Debug, M = ()>: Send + Sync {
    /// Get phase metadata (compile-time)
    fn metadata() -> PhaseMetadata
    where
        Self: Sized;

    /// Execute the phase
    fn execute(
        &self,
        ctx: PhaseContext,
    ) -> impl Future<Output = WorkflowResult<PhaseResult<T>>> + Send;

    /// Pre-execution hook (optional)
    fn pre_execute(&self, _ctx: &PhaseContext) -> impl Future<Output = WorkflowResult<()>> + Send {
        async { Ok(()) }
    }

    /// Post-execution hook (optional)
    fn post_execute(
        &self,
        _ctx: &PhaseContext,
        _result: &PhaseResult<T>,
    ) -> impl Future<Output = WorkflowResult<()>> + Send {
        async { Ok(()) }
    }
}

/// Type-level phase composition via phantom types
pub struct ComposedPhase<P1, P2, T1, T2, M1 = (), M2 = ()>
where
    P1: Phase<T1, M1>,
    P2: Phase<T2, M2>,
    T1: Clone + Debug,
    T2: Clone + Debug,
{
    phase1: P1,
    phase2: P2,
    _phantom: PhantomData<(T1, T2, M1, M2)>,
}

impl<P1, P2, T1, T2, M1, M2> ComposedPhase<P1, P2, T1, T2, M1, M2>
where
    P1: Phase<T1, M1>,
    P2: Phase<T2, M2>,
    T1: Clone + Debug,
    T2: Clone + Debug,
{
    /// Create a new composed phase
    pub fn new(phase1: P1, phase2: P2) -> Self {
        Self {
            phase1,
            phase2,
            _phantom: PhantomData,
        }
    }
}

/// Composed phase output
#[derive(Debug, Clone)]
pub struct ComposedOutput<T1: Clone + Debug, T2: Clone + Debug> {
    pub first: T1,
    pub second: T2,
}

// Implement Phase for ComposedPhase (sequential execution)
impl<P1, P2, T1, T2, M1, M2> Phase<ComposedOutput<T1, T2>> for ComposedPhase<P1, P2, T1, T2, M1, M2>
where
    P1: Phase<T1, M1> + Send + Sync,
    P2: Phase<T2, M2> + Send + Sync,
    T1: Clone + Debug + Send + Sync,
    T2: Clone + Debug + Send + Sync,
    M1: Send + Sync,
    M2: Send + Sync,
{
    fn metadata() -> PhaseMetadata {
        PhaseMetadata {
            name: "composed_phase",
            description: "Composed phase executing two phases sequentially",
            version: "1.0.0",
            dependencies: &[],
            parallel: false,
        }
    }

    async fn execute(
        &self,
        ctx: PhaseContext,
    ) -> WorkflowResult<PhaseResult<ComposedOutput<T1, T2>>> {
        use std::time::Instant;

        let start = Instant::now();

        // Execute phase 1
        let result1 = self.phase1.execute(ctx.clone()).await?;

        // Execute phase 2
        let result2 = self.phase2.execute(ctx).await?;

        let duration = start.elapsed();

        let status = match (result1.status, result2.status) {
            (PhaseStatus::Pass, PhaseStatus::Pass) => PhaseStatus::Pass,
            (PhaseStatus::Fail, _) | (_, PhaseStatus::Fail) => PhaseStatus::Fail,
            _ => PhaseStatus::Warning,
        };

        Ok(PhaseResult::new(
            "composed_phase",
            status,
            ComposedOutput {
                first: result1.data,
                second: result2.data,
            },
        )
        .with_duration(duration)
        .with_counts(
            result1.passed + result2.passed,
            result1.failed + result2.failed,
            result1.warnings + result2.warnings,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestData {
        value: i32,
    }

    struct TestPhase;

    impl Phase<TestData> for TestPhase {
        fn metadata() -> PhaseMetadata {
            PhaseMetadata {
                name: "test_phase",
                description: "Test phase",
                version: "1.0.0",
                dependencies: &[],
                parallel: true,
            }
        }

        async fn execute(&self, _ctx: PhaseContext) -> WorkflowResult<PhaseResult<TestData>> {
            Ok(PhaseResult::new(
                "test_phase",
                PhaseStatus::Pass,
                TestData { value: 42 },
            ))
        }
    }

    #[test]
    fn test_phase_result_creation() {
        let result = PhaseResult::new("test", PhaseStatus::Pass, TestData { value: 42 });
        assert_eq!(result.name, "test");
        assert_eq!(result.status, PhaseStatus::Pass);
        assert_eq!(result.data.value, 42);
    }
}
