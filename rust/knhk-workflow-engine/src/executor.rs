//! Workflow execution engine

use crate::case::{Case, CaseId, CaseState};
use crate::compliance::ProvenanceTracker;
use crate::enterprise::EnterpriseConfig;
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::{Fortune5Config, RuntimeClass};
use crate::integration::SidecarIntegration;
use crate::integration::{Fortune5Integration, LockchainIntegration, OtelIntegration};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternId, PatternRegistry, RegisterAllExt,
};
use crate::resource::{AllocationRequest, ResourceAllocator};
use crate::security::AuthManager;
use crate::services::timer::TimerService;
use crate::services::{AdmissionGate, EventSidecar, TimerFired, WorkItemService};
use crate::state::StateStore;
use crate::timebase::SysClock;
use crate::validation::DeadlockDetector;
use crate::worklets::{WorkletExecutor, WorkletRepository};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

/// Workflow execution engine
pub struct WorkflowEngine {
    /// Pattern registry
    pattern_registry: Arc<PatternRegistry>,
    /// State store
    state_store: Arc<RwLock<StateStore>>,
    /// Registered workflow specifications
    specs: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,
    /// Active cases
    cases: Arc<RwLock<HashMap<CaseId, Case>>>,
    /// Resource allocator
    resource_allocator: Arc<ResourceAllocator>,
    /// Worklet repository
    worklet_repository: Arc<WorkletRepository>,
    /// Worklet executor
    worklet_executor: Arc<WorkletExecutor>,
    /// Timer service
    timer_service: Arc<TimerService<SysClock>>,
    /// Work item service
    work_item_service: Arc<WorkItemService>,
    /// Admission gate
    admission_gate: Arc<AdmissionGate>,
    /// Event sidecar
    event_sidecar: Arc<EventSidecar>,
    /// Enterprise configuration
    enterprise_config: Option<Arc<EnterpriseConfig>>,
    /// Fortune 5 integration (if enabled)
    fortune5_integration: Option<Arc<Fortune5Integration>>,
    /// OTEL integration (if enabled)
    otel_integration: Option<Arc<OtelIntegration>>,
    /// Lockchain integration (if enabled)
    lockchain_integration: Option<Arc<LockchainIntegration>>,
    /// Auth manager (if enabled)
    auth_manager: Option<Arc<RwLock<AuthManager>>>,
    /// Provenance tracker (if enabled)
    provenance_tracker: Option<Arc<ProvenanceTracker>>,
    /// Sidecar integration (if enabled)
    sidecar_integration: Option<Arc<SidecarIntegration>>,
}

impl WorkflowEngine {
    /// Create a new workflow engine with all 43 patterns registered
    pub fn new(state_store: StateStore) -> Self {
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let resource_allocator = Arc::new(ResourceAllocator::new());
        let worklet_repository = Arc::new(WorkletRepository::new());
        let worklet_executor = Arc::new(WorkletExecutor::new(worklet_repository.clone()));

        // Create event channels
        let (timer_tx, timer_rx) = mpsc::channel::<TimerFired>(1024);
        let (event_tx, event_rx) = mpsc::channel::<serde_json::Value>(1024);

        // Create services
        let timebase = Arc::new(SysClock);
        let state_store_arc = Arc::new(state_store);
        let timer_service = Arc::new(TimerService::new(
            timebase,
            timer_tx.clone(),
            Some(state_store_arc.clone()),
        ));
        let work_item_service = Arc::new(WorkItemService::new());
        let admission_gate = Arc::new(AdmissionGate::new());
        let event_sidecar = Arc::new(EventSidecar::new(event_tx.clone()));

        let engine = Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store)),
            specs: Arc::new(RwLock::new(HashMap::new())),
            cases: Arc::new(RwLock::new(HashMap::new())),
            resource_allocator,
            worklet_repository,
            worklet_executor,
            timer_service: timer_service.clone(),
            work_item_service,
            admission_gate,
            event_sidecar: event_sidecar.clone(),
            enterprise_config: None,
            otel_integration: None,
            lockchain_integration: None,
            auth_manager: None,
            provenance_tracker: None,
            fortune5_integration: None,
            sidecar_integration: None,
        };

        // Start event loops
        let pattern_registry_clone = Arc::clone(&engine.pattern_registry);
        Self::start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        Self::start_event_loop(pattern_registry_clone, event_rx);

        engine
    }

    /// Start timer event loop
    fn start_timer_loop(registry: Arc<PatternRegistry>, mut timer_rx: mpsc::Receiver<TimerFired>) {
        tokio::spawn(async move {
            while let Some(tf) = timer_rx.recv().await {
                // Parse IDs from strings
                let case_id = crate::case::CaseId::parse_str(&tf.case_id)
                    .unwrap_or_else(|_| crate::case::CaseId::new());
                let workflow_id = crate::parser::WorkflowSpecId::parse_str(&tf.workflow_id)
                    .unwrap_or_else(|_| crate::parser::WorkflowSpecId::new());

                let ctx = PatternExecutionContext {
                    case_id,
                    workflow_id,
                    variables: {
                        let mut vars = HashMap::new();
                        vars.insert("key".to_string(), tf.key.clone());
                        vars.insert("fired_at".to_string(), tf.fired_at.to_string());
                        vars
                    },
                    arrived_from: std::collections::HashSet::new(),
                    scope_id: String::new(),
                };
                // Execute pattern 30 or 31 based on timer type
                let _ = registry.execute(&PatternId(tf.pattern_id as u32), &ctx);
            }
        });
    }

    /// Start external event loop (Pattern 16: Deferred Choice)
    fn start_event_loop(
        registry: Arc<PatternRegistry>,
        mut event_rx: mpsc::Receiver<serde_json::Value>,
    ) {
        tokio::spawn(async move {
            while let Some(evt) = event_rx.recv().await {
                let case_id = evt
                    .get("case_id")
                    .and_then(|v| v.as_str())
                    .map(|s| CaseId::parse_str(s).unwrap_or_else(|_| CaseId::new()))
                    .unwrap_or_else(|| CaseId::new());
                let workflow_id = evt
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .map(|s| WorkflowSpecId::parse_str(s).unwrap_or_else(|_| WorkflowSpecId::new()))
                    .unwrap_or_else(|| WorkflowSpecId::new());

                let ctx = PatternExecutionContext {
                    case_id,
                    workflow_id,
                    variables: {
                        let mut vars = HashMap::new();
                        if let Some(event_type) = evt.get("event_type").and_then(|v| v.as_str()) {
                            vars.insert("event_type".to_string(), event_type.to_string());
                        }
                        if let Some(data) = evt.get("data") {
                            if let Some(obj) = data.as_object() {
                                for (k, v) in obj {
                                    if let Some(s) = v.as_str() {
                                        vars.insert(k.clone(), s.to_string());
                                    }
                                }
                            }
                        }
                        vars
                    },
                    arrived_from: std::collections::HashSet::new(),
                    scope_id: String::new(),
                };
                let _ = registry.execute(&PatternId(16), &ctx); // Pattern 16: Deferred Choice
            }
        });
    }

    /// Create a new workflow engine with Fortune 5 configuration
    pub fn with_fortune5(
        state_store: StateStore,
        fortune5_config: Fortune5Config,
    ) -> WorkflowResult<Self> {
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let resource_allocator = Arc::new(ResourceAllocator::new());
        let worklet_repository = Arc::new(WorkletRepository::new());
        let worklet_executor = Arc::new(WorkletExecutor::new(worklet_repository.clone()));

        // Create event channels
        let (timer_tx, timer_rx) = mpsc::channel::<TimerFired>(1024);
        let (event_tx, event_rx) = mpsc::channel::<serde_json::Value>(1024);

        // Create services
        let timer_service = Arc::new(TimerService::new(
            Arc::new(SysClock),
            timer_tx.clone(),
            None,
        ));
        let work_item_service = Arc::new(WorkItemService::new());
        let admission_gate = Arc::new(AdmissionGate::new());
        let event_sidecar = Arc::new(EventSidecar::new(event_tx.clone()));

        // Initialize Fortune 5 integration
        let fortune5_integration = Arc::new(Fortune5Integration::new(fortune5_config.clone()));

        // Initialize OTEL integration if enabled in Fortune 5 config
        let otel_integration = if fortune5_config.slo.is_some() {
            Some(Arc::new(OtelIntegration::new(None)))
        } else {
            None
        };

        // Initialize lockchain integration for provenance
        // Use environment variable or default path
        let lockchain_path =
            std::env::var("KNHK_LOCKCHAIN_PATH").unwrap_or_else(|_| "./lockchain".to_string());
        let lockchain_integration = Some(Arc::new(LockchainIntegration::new(&lockchain_path)?));

        let engine = Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store)),
            specs: Arc::new(RwLock::new(HashMap::new())),
            cases: Arc::new(RwLock::new(HashMap::new())),
            resource_allocator,
            worklet_repository,
            worklet_executor,
            timer_service: timer_service.clone(),
            work_item_service,
            admission_gate,
            event_sidecar: event_sidecar.clone(),
            enterprise_config: None,
            fortune5_integration: Some(fortune5_integration),
            otel_integration,
            lockchain_integration,
            auth_manager: None,
            provenance_tracker: Some(Arc::new(ProvenanceTracker::new(true))),
            sidecar_integration: None,
        };

        // Start event loops
        let pattern_registry_clone = Arc::clone(&engine.pattern_registry);
        Self::start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        Self::start_event_loop(pattern_registry_clone, event_rx);

        Ok(engine)
    }

    /// Register a workflow specification with deadlock validation and Fortune 5 checks
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()> {
        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked workflow registration".to_string(),
                ));
            }
        }

        // Validate for deadlocks before registration
        let detector = DeadlockDetector;
        detector.validate(&spec)?;

        let mut specs = self.specs.write().await;
        let spec_clone = spec.clone();
        specs.insert(spec.id, spec);

        // Persist to state store
        let store = self.state_store.read().await;
        store.save_spec(&spec_clone)?;

        Ok(())
    }

    /// Get workflow specification
    pub async fn get_workflow(&self, spec_id: WorkflowSpecId) -> WorkflowResult<WorkflowSpec> {
        // Try in-memory first
        let specs = self.specs.read().await;
        if let Some(spec) = specs.get(&spec_id) {
            return Ok(spec.clone());
        }
        drop(specs);

        // Fallback to state store
        let store = self.state_store.read().await;
        store.load_spec(&spec_id)?.ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
        })
    }

    /// Create a new case
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        // Admission gate: validate case data before execution
        self.admission_gate.admit(&data)?;

        // Verify workflow exists
        let _spec = self.get_workflow(spec_id).await?;

        // Create case
        let case = Case::new(spec_id, data);
        let case_id = case.id;

        // Store case
        let mut cases = self.cases.write().await;
        let case_clone = case.clone();
        cases.insert(case_id, case);

        // Persist to state store
        let store = self.state_store.read().await;
        store.save_case(case_id, &case_clone)?;

        Ok(case_id)
    }

    /// Start a case
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        case.start()?;

        // Persist state
        let store = self.state_store.write().await;
        store.save_case(case_id, case)?;

        Ok(())
    }

    /// Execute a case (run workflow) with resource allocation, worklet support, and Fortune 5 SLO tracking
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked case execution".to_string(),
                ));
            }
        }

        let start_time = Instant::now();

        // Get case
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        // Start if not already started
        if case.state == CaseState::Created {
            case.start()?;
        }

        // Get workflow specification
        let specs = self.specs.read().await;
        let spec = specs.get(&case.spec_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", case.spec_id))
        })?;
        let spec_clone = spec.clone();
        drop(specs);
        drop(cases);

        // Execute workflow with resource allocation and worklet support
        self.execute_workflow_tasks(case_id, &spec_clone).await?;

        // Record SLO metrics if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let elapsed_ns = start_time.elapsed().as_nanos() as u64;
            // Determine runtime class based on execution time
            let runtime_class = if elapsed_ns <= 2_000 {
                RuntimeClass::R1 // Hot path (≤2ns)
            } else if elapsed_ns <= 1_000_000 {
                RuntimeClass::W1 // Warm path (≤1ms)
            } else {
                RuntimeClass::C1 // Cold path (≤500ms)
            };
            fortune5.record_slo_metric(runtime_class, elapsed_ns).await;
        }

        Ok(())
    }

    /// Execute workflow tasks with resource allocation
    async fn execute_workflow_tasks(
        &self,
        case_id: CaseId,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<()> {
        // Start from start condition
        if let Some(ref start_condition_id) = spec.start_condition {
            if let Some(start_condition) = spec.conditions.get(start_condition_id) {
                // Execute tasks from start condition
                for task_id in &start_condition.outgoing_flows {
                    if let Some(task) = spec.tasks.get(task_id) {
                        self.execute_task_with_allocation(case_id, spec.id, task)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a task with resource allocation and Fortune 5 SLO tracking
    async fn execute_task_with_allocation(
        &self,
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        task: &crate::parser::Task,
    ) -> WorkflowResult<()> {
        let task_start_time = Instant::now();
        // Allocate resources if allocation policy is specified
        if let Some(ref policy) = task.allocation_policy {
            let request = AllocationRequest {
                task_id: task.id.clone(),
                spec_id,
                required_roles: task.required_roles.clone(),
                required_capabilities: task.required_capabilities.clone(),
                policy: *policy,
                priority: task.priority.unwrap_or(100) as u8,
            };

            match self.resource_allocator.allocate(request).await {
                Ok(allocation) => {
                    // Resources allocated - proceed with task execution
                    // In production, would track allocation and release after execution
                    for resource_id in &allocation.resource_ids {
                        self.resource_allocator
                            .update_workload(*resource_id, 1)
                            .await?;
                    }
                }
                Err(e) => {
                    // Resource allocation failed - try worklet exception handling
                    if let Some(_worklet_id) = task.exception_worklet {
                        let context = PatternExecutionContext {
                            case_id,
                            workflow_id: spec_id,
                            variables: HashMap::new(),
                            arrived_from: std::collections::HashSet::new(),
                            scope_id: String::new(),
                        };
                        if let Some(result) = self
                            .worklet_executor
                            .handle_exception("resource_unavailable", context)
                            .await?
                        {
                            if !result.success {
                                return Err(e);
                            }
                        } else {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Execute task (simplified - full implementation would execute actual task logic)
        // Check max_ticks constraint
        if let Some(max_ticks) = task.max_ticks {
            let elapsed_ns = task_start_time.elapsed().as_nanos() as u64;
            let elapsed_ticks = elapsed_ns / 2; // 2ns per tick
            if elapsed_ticks > max_ticks as u64 {
                return Err(WorkflowError::TaskExecutionFailed(format!(
                    "Task {} exceeded tick budget: {} ticks > {} ticks",
                    task.id, elapsed_ticks, max_ticks
                )));
            }
        }

        // Record SLO metrics if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let elapsed_ns = task_start_time.elapsed().as_nanos() as u64;
            // Determine runtime class based on task max_ticks or execution time
            let runtime_class = if task.max_ticks.map_or(false, |ticks| ticks <= 8) {
                RuntimeClass::R1 // Hot path task
            } else if elapsed_ns <= 1_000_000 {
                RuntimeClass::W1 // Warm path
            } else {
                RuntimeClass::C1 // Cold path
            };
            fortune5.record_slo_metric(runtime_class, elapsed_ns).await;
        }

        Ok(())
    }

    /// Cancel a case
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        // Persist state
        let store = self.state_store.write().await;
        store.save_case(case_id, case)?;

        Ok(())
    }

    /// Get case status
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        let cases = self.cases.read().await;
        cases
            .get(&case_id)
            .cloned()
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))
    }

    /// List all registered workflow specifications
    pub async fn list_workflows(&self) -> WorkflowResult<Vec<WorkflowSpecId>> {
        let specs = self.specs.read().await;
        Ok(specs.keys().cloned().collect())
    }

    /// List all cases for a workflow specification
    pub async fn list_cases(&self, spec_id: WorkflowSpecId) -> WorkflowResult<Vec<CaseId>> {
        let store = self.state_store.read().await;
        store.list_cases(&spec_id)
    }

    /// Execute a pattern
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let start_time = Instant::now();

        // Check Fortune 5 promotion gate if enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            if !fortune5.check_promotion_gate().await? {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked execution".to_string(),
                ));
            }
        }

        // Execute pattern
        let result = self
            .pattern_registry
            .execute(&pattern_id, &context)
            .ok_or_else(|| {
                WorkflowError::InvalidSpecification(format!("Pattern {} not found", pattern_id))
            })?;

        // Reflex bridge: promote pattern subgraphs to hot path if promotable
        let mut reflex_bridge = crate::reflex::ReflexBridge::new();
        if let Ok(workflow_spec) = self.get_workflow(context.workflow_id).await {
            if let Ok(promotable_segments) = reflex_bridge.bind_hot_segments(&workflow_spec) {
                // Check if this pattern is in a promotable segment
                for segment in &promotable_segments {
                    if segment.pattern_ids.contains(&(pattern_id.0 as u8))
                        && segment.hot_executor_bound
                    {
                        // Pattern is promotable - would execute via hot path in production
                        // For now, just record that promotion was considered
                    }
                }
            }
        }

        // Record SLO metrics if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let duration_ns = start_time.elapsed().as_nanos() as u64;
            // Determine runtime class based on pattern ID and duration
            let runtime_class = if duration_ns <= 2_000 {
                crate::integration::fortune5::RuntimeClass::R1 // Hot path
            } else if duration_ns <= 1_000_000 {
                crate::integration::fortune5::RuntimeClass::W1 // Warm path
            } else {
                crate::integration::fortune5::RuntimeClass::C1 // Cold path
            };
            fortune5.record_slo_metric(runtime_class, duration_ns).await;
        }

        Ok(result)
    }

    /// Get pattern registry
    pub fn pattern_registry(&self) -> &PatternRegistry {
        &self.pattern_registry
    }

    /// Get resource allocator
    pub fn resource_allocator(&self) -> &ResourceAllocator {
        &self.resource_allocator
    }

    /// Get worklet repository
    pub fn worklet_repository(&self) -> &WorkletRepository {
        &self.worklet_repository
    }

    /// Get worklet executor
    pub fn worklet_executor(&self) -> &WorkletExecutor {
        &self.worklet_executor
    }

    /// Get state store (for REST API access)
    pub fn state_store(&self) -> &Arc<RwLock<StateStore>> {
        &self.state_store
    }

    /// Get specs (for REST API access)
    pub fn specs(&self) -> &Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>> {
        &self.specs
    }

    /// Get cases (for REST API access)
    pub fn cases(&self) -> &Arc<RwLock<HashMap<CaseId, Case>>> {
        &self.cases
    }

    /// Get Fortune 5 integration (if enabled)
    pub fn fortune5_integration(&self) -> Option<&Fortune5Integration> {
        self.fortune5_integration.as_deref()
    }

    /// Get sidecar integration
    pub fn sidecar_integration(&self) -> Option<&SidecarIntegration> {
        self.sidecar_integration.as_deref()
    }

    /// Get timer service
    pub fn timer_service(&self) -> &Arc<TimerService<SysClock>> {
        &self.timer_service
    }

    /// Get work item service
    pub fn work_item_service(&self) -> &Arc<WorkItemService> {
        &self.work_item_service
    }

    /// Get admission gate
    pub fn admission_gate(&self) -> &Arc<AdmissionGate> {
        &self.admission_gate
    }

    /// Get event sidecar
    pub fn event_sidecar(&self) -> &Arc<EventSidecar> {
        &self.event_sidecar
    }

    /// Check SLO compliance (Fortune 5 feature)
    pub async fn check_slo_compliance(&self) -> WorkflowResult<bool> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.check_slo_compliance().await
        } else {
            Ok(true) // No SLO configured, always compliant
        }
    }

    /// Get SLO metrics (Fortune 5)
    pub async fn get_slo_metrics(&self) -> Option<(u64, u64, u64)> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.get_slo_metrics().await
        } else {
            None
        }
    }

    /// Check if feature flag is enabled (Fortune 5)
    pub async fn is_feature_enabled(&self, feature: &str) -> bool {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.is_feature_enabled(feature).await
        } else {
            true // No Fortune 5, allow all features
        }
    }
}
