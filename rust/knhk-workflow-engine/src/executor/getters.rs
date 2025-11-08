//! Getter methods for WorkflowEngine

use crate::case::{Case, CaseId};
use crate::integration::fortune5::Fortune5Integration;
use crate::integration::SidecarIntegration;
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternRegistry;
use crate::resource::ResourceAllocator;
use crate::services::timer::TimerService;
use crate::services::{AdmissionGate, EventSidecar, WorkItemService};
use crate::state::StateStore;
use crate::timebase::SysClock;
use crate::worklets::{WorkletExecutor, WorkletRepository};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::engine::WorkflowEngine;

impl WorkflowEngine {
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
    pub fn specs(&self) -> &Arc<RwLock<HashMap<WorkflowSpecId, crate::parser::WorkflowSpec>>> {
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
    pub async fn check_slo_compliance(&self) -> crate::error::WorkflowResult<bool> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.check_slo_compliance().await
        } else {
            Ok(true) // No SLO configured, always compliant
        }
    }

    /// Get SLO metrics (Fortune 5)
    pub async fn get_slo_metrics(&self) -> crate::error::WorkflowResult<serde_json::Value> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.get_slo_metrics().await
        } else {
            Ok(serde_json::json!({}))
        }
    }
}

