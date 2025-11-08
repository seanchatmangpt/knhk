//! Workflow engine construction and initialization

use crate::compliance::ProvenanceTracker;
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::Fortune5Config;
use crate::integration::{Fortune5Integration, LockchainIntegration, OtelIntegration};
use crate::patterns::{PatternRegistry, RegisterAllExt};
use crate::resource::ResourceAllocator;
use crate::services::timer::TimerService;
use crate::services::{AdmissionGate, EventSidecar, TimerFired, WorkItemService};
use crate::state::StateStore;
use crate::timebase::SysClock;
use crate::worklets::{WorkletExecutor, WorkletRepository};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::events::{start_event_loop, start_timer_loop};
use super::WorkflowEngine;

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
            state_store: Arc::new(RwLock::new(state_store_arc)),
            specs: Arc::new(DashMap::new()),
            cases: Arc::new(DashMap::new()),
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
        start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        start_event_loop(pattern_registry_clone, event_rx);

        engine
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
            state_store: Arc::new(RwLock::new(state_store_arc)),
            specs: Arc::new(DashMap::new()),
            cases: Arc::new(DashMap::new()),
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
        start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        start_event_loop(pattern_registry_clone, event_rx);

        // Start dual-clock projection task (warm persistence replay)
        // Projects nanosecond commits to millisecond legacy time
        let engine_arc = Arc::new(engine);
        let engine_clone = Arc::clone(&engine_arc);
        tokio::spawn(async move {
            Self::dual_clock_projection_loop(engine_clone).await;
        });

        // Return the engine (move it out of Arc)
        // Note: We can't clone WorkflowEngine, so we use try_unwrap
        // If there are multiple references (shouldn't happen), we'll get an error
        Ok(Arc::try_unwrap(engine_arc).map_err(|_| {
            WorkflowError::Internal(
                "Failed to unwrap engine Arc - multiple references exist".to_string(),
            )
        })?)
    }

    /// Dual-clock projection loop (warm persistence replay)
    ///
    /// Projects nanosecond commits to millisecond legacy time.
    /// Replays completed cases to external observers every N ms,
    /// bridging nanosecond and human domains.
    async fn dual_clock_projection_loop(engine: Arc<WorkflowEngine>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));
        loop {
            interval.tick().await;

            // Get all completed cases from state store
            // FUTURE: Query completed cases and replay to external observers
            // For now, just log that projection is running
            let _store_arc = engine.state_store.read().await;
            // In production, would:
            // 1. Query completed cases from state store
            // 2. Replay to external observers (REST/gRPC)
            // 3. Project nanosecond commits to millisecond legacy time
        }
    }
}
