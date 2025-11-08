//! Workflow engine core structure

use crate::case::{Case, CaseId};
use crate::compliance::ProvenanceTracker;
use crate::enterprise::EnterpriseConfig;
use crate::integration::fortune5::Fortune5Integration;
use crate::integration::{LockchainIntegration, OtelIntegration, SidecarIntegration};
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternRegistry;
use crate::resource::ResourceAllocator;
use crate::security::AuthManager;
use crate::services::timer::TimerService;
use crate::services::{AdmissionGate, EventSidecar, WorkItemService};
use crate::state::StateStore;
use crate::timebase::SysClock;
use crate::worklets::{WorkletExecutor, WorkletRepository};
use crate::WorkflowSpec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Workflow execution engine
pub struct WorkflowEngine {
    /// Pattern registry
    pub(crate) pattern_registry: Arc<PatternRegistry>,
    /// State store
    pub(crate) state_store: Arc<RwLock<StateStore>>,
    /// Registered workflow specifications
    pub(crate) specs: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,
    /// Active cases
    pub(crate) cases: Arc<RwLock<HashMap<CaseId, Case>>>,
    /// Resource allocator
    pub(crate) resource_allocator: Arc<ResourceAllocator>,
    /// Worklet repository
    pub(crate) worklet_repository: Arc<WorkletRepository>,
    /// Worklet executor
    pub(crate) worklet_executor: Arc<WorkletExecutor>,
    /// Timer service
    pub(crate) timer_service: Arc<TimerService<SysClock>>,
    /// Work item service
    pub(crate) work_item_service: Arc<WorkItemService>,
    /// Admission gate
    pub(crate) admission_gate: Arc<AdmissionGate>,
    /// Event sidecar
    pub(crate) event_sidecar: Arc<EventSidecar>,
    /// Enterprise configuration
    pub(crate) enterprise_config: Option<Arc<EnterpriseConfig>>,
    /// Fortune 5 integration (if enabled)
    pub(crate) fortune5_integration: Option<Arc<Fortune5Integration>>,
    /// OTEL integration (if enabled)
    pub(crate) otel_integration: Option<Arc<OtelIntegration>>,
    /// Lockchain integration (if enabled)
    pub(crate) lockchain_integration: Option<Arc<LockchainIntegration>>,
    /// Auth manager (if enabled)
    pub(crate) auth_manager: Option<Arc<RwLock<AuthManager>>>,
    /// Provenance tracker (if enabled)
    pub(crate) provenance_tracker: Option<Arc<ProvenanceTracker>>,
    /// Sidecar integration (if enabled)
    pub(crate) sidecar_integration: Option<Arc<SidecarIntegration>>,
}

