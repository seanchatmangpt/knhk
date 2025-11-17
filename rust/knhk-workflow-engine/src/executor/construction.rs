//! Workflow engine construction and initialization

use crate::compliance::ProvenanceTracker;
use crate::error::WorkflowResult;
use crate::integration::fortune5::Fortune5Config;
use crate::integration::ConnectorIntegration;
use crate::integration::{Fortune5Integration, LockchainIntegration, OtelIntegration};
use crate::patterns::{PatternRegistry, RegisterAllExt};
use crate::resource::ResourceAllocator;
use crate::services::timer::TimerService;
use crate::services::{AdmissionGate, EventSidecar, TimerFired, WorkItemService};
use crate::state::manager::StateManager;
#[cfg(feature = "storage")]
use crate::state::StateStore;
use crate::timebase::SysClock;
use crate::worklets::{WorkletExecutor, WorkletRepository};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::events::{start_event_loop, start_timer_loop};
use super::WorkflowEngine;
#[cfg(feature = "rdf")]
use oxigraph::store::Store;
use std::collections::HashMap;

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
        let state_manager = Arc::new(StateManager::new(state_store_arc.clone()));
        let timer_service = Arc::new(TimerService::new(
            timebase,
            timer_tx.clone(),
            Some(state_store_arc.clone()),
        ));
        let work_item_service = Arc::new(WorkItemService::new());
        let admission_gate = Arc::new(AdmissionGate::new());
        let event_sidecar = Arc::new(EventSidecar::new(event_tx.clone()));

        // Create RDF stores for runtime queries
        #[cfg(feature = "rdf")]
        let spec_rdf_store = Arc::new(RwLock::new(
            Store::new().unwrap_or_else(|_| panic!("Failed to create spec RDF store")),
        ));
        #[cfg(not(feature = "rdf"))]
        let spec_rdf_store = Arc::new(RwLock::new(()));
        #[cfg(feature = "rdf")]
        let pattern_metadata_store =
            Arc::new(RwLock::new(Store::new().unwrap_or_else(|_| {
                panic!("Failed to create pattern metadata RDF store")
            })));
        #[cfg(not(feature = "rdf"))]
        let pattern_metadata_store = Arc::new(RwLock::new(()));
        let case_rdf_stores = Arc::new(RwLock::new(HashMap::new()));

        let engine = Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store_arc)),
            state_manager,
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
            connector_integration: Some(Arc::new(tokio::sync::Mutex::new(
                ConnectorIntegration::new(),
            ))),
            spec_rdf_store,
            pattern_metadata_store,
            case_rdf_stores,
        };

        // Note: Pattern metadata loading must be done by the caller after engine creation
        // because WorkflowEngine::new is synchronous. Call engine.load_pattern_metadata_rdf().await
        // after creating the engine in async context.

        // Start event loops
        let pattern_registry_clone = Arc::clone(&engine.pattern_registry);
        start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        start_event_loop(pattern_registry_clone, event_rx);

        // Recover timers from durable storage on startup
        if let Err(e) = engine.timer_service.recover_timers().await {
            tracing::warn!("Failed to recover timers on startup: {}", e);
        }

        // Start compaction scheduler at fixed tick epochs
        let state_store_clone = Arc::clone(&engine.state_store);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let store = state_store_clone.read().await;
                if let Err(e) = (*store).compact().await {
                    tracing::warn!("Compaction failed: {}", e);
                } else {
                    tracing::debug!("Compaction completed successfully");
                }
            }
        });

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
        let state_manager = Arc::new(StateManager::new(state_store_arc.clone()));
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

        // Create RDF stores for runtime queries
        #[cfg(feature = "rdf")]
        let spec_rdf_store = Arc::new(RwLock::new(
            Store::new().unwrap_or_else(|_| panic!("Failed to create spec RDF store")),
        ));
        #[cfg(not(feature = "rdf"))]
        let spec_rdf_store = Arc::new(RwLock::new(()));
        #[cfg(feature = "rdf")]
        let pattern_metadata_store =
            Arc::new(RwLock::new(Store::new().unwrap_or_else(|_| {
                panic!("Failed to create pattern metadata RDF store")
            })));
        #[cfg(not(feature = "rdf"))]
        let pattern_metadata_store = Arc::new(RwLock::new(()));
        let case_rdf_stores = Arc::new(RwLock::new(HashMap::new()));

        let engine = Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store_arc)),
            state_manager,
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
            connector_integration: Some(Arc::new(tokio::sync::Mutex::new(
                ConnectorIntegration::new(),
            ))),
            spec_rdf_store,
            pattern_metadata_store,
            case_rdf_stores,
        };

        // Note: Pattern metadata loading must be done by the caller after engine creation
        // because WorkflowEngine::new is synchronous. Call engine.load_pattern_metadata_rdf().await
        // after creating the engine in async context.

        // Start event loops
        let pattern_registry_clone = Arc::clone(&engine.pattern_registry);
        start_timer_loop(pattern_registry_clone.clone(), timer_rx);
        start_event_loop(pattern_registry_clone, event_rx);

        // Start dual-clock projection task (warm persistence replay)
        // Projects nanosecond commits to millisecond legacy time
        // Note: WorkflowEngine contains LockchainIntegration which is not Send,
        // so we can't spawn a task with the entire engine. Instead, we'll skip
        // the background task for now. In production, this would be handled
        // by a separate service that owns the state store.
        // FUTURE: Extract state_store into a separate Send-safe type

        Ok(engine)
    }

    /// Set OTEL integration on the engine
    ///
    /// This method creates a new engine instance with OTEL integration set.
    /// Since WorkflowEngine is Clone, this is efficient due to Arc sharing.
    pub fn with_otel(mut self, otel_integration: Arc<OtelIntegration>) -> Self {
        self.otel_integration = Some(otel_integration);
        self
    }

    /// Dual-clock projection loop (warm persistence replay)
    ///
    /// Projects nanosecond commits to millisecond legacy time.
    /// Replays completed cases to external observers every N ms,
    /// bridging nanosecond and human domains.
    ///
    /// NOTE: This method is not currently used because WorkflowEngine contains
    /// non-Send types (LockchainIntegration). In production, this would be handled
    /// by a separate service that owns the state store.
    #[allow(dead_code)]
    async fn dual_clock_projection_loop(_engine: Arc<WorkflowEngine>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));
        loop {
            interval.tick().await;

            // Get all completed cases from state store
            // FUTURE: Query completed cases and replay to external observers
            // For now, just log that projection is running
            // In production, would:
            // 1. Query completed cases from state store
            // 2. Replay to external observers (REST/gRPC)
            // 3. Project nanosecond commits to millisecond legacy time
            tracing::debug!("Dual-clock projection loop running (not implemented)");
        }
    }
}
