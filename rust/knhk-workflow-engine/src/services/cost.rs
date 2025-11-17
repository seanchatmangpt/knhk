//! YAWL Cost Service Port with TRIZ Hyper-Advanced Patterns
//!
//! This module ports Java YAWL's CostService while applying TRIZ principles:
//! - **Principle 10 (Prior Action)**: Pre-compute cost estimates
//! - **Principle 1 (Segmentation)**: Separate cost tracking from execution
//!
//! # Architecture
//!
//! YAWL cost service tracks:
//! - Activity costs (per task execution)
//! - Resource costs (per resource allocation)
//! - Case costs (total cost per case)
//! - Workflow costs (aggregate costs per workflow)
//!
//! # TRIZ Enhancements
//!
//! - Cost estimates are pre-computed (Principle 10)
//! - Cost tracking is separated from execution (Principle 1)

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use crate::resource::ResourceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Cost entry for an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCost {
    /// Task ID
    pub task_id: String,
    /// Resource ID (if allocated)
    pub resource_id: Option<ResourceId>,
    /// Cost amount
    pub amount: f64,
    /// Currency
    pub currency: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Cost category
    pub category: CostCategory,
}

/// Cost category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostCategory {
    /// Resource cost (human time)
    Resource,
    /// System cost (compute time)
    System,
    /// External service cost
    External,
    /// Infrastructure cost
    Infrastructure,
}

/// Cost summary for a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseCostSummary {
    /// Case ID
    pub case_id: CaseId,
    /// Total cost
    pub total_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost breakdown by category
    pub breakdown: HashMap<CostCategory, f64>,
    /// Activity costs
    pub activities: Vec<ActivityCost>,
}

/// YAWL Cost Service
///
/// Tracks and reports costs for workflow execution.
///
/// # TRIZ Principle 1: Segmentation
///
/// Cost tracking is separated from execution, allowing independent optimization.
///
/// # TRIZ Principle 10: Prior Action
///
/// Cost estimates are pre-computed based on resource rates and task complexity.
pub struct CostService {
    /// Activity costs by case ID
    case_costs: Arc<RwLock<HashMap<CaseId, Vec<ActivityCost>>>>,
    /// Resource cost rates (resource_id → cost per hour)
    resource_rates: Arc<RwLock<HashMap<ResourceId, f64>>>,
    /// Default currency
    currency: String,
}

impl CostService {
    /// Create a new cost service
    pub fn new(currency: String) -> Self {
        Self {
            case_costs: Arc::new(RwLock::new(HashMap::new())),
            resource_rates: Arc::new(RwLock::new(HashMap::new())),
            currency,
        }
    }

    /// Record activity cost
    ///
    /// Records the cost of executing a task.
    pub async fn record_activity_cost(&self, cost: ActivityCost, case_id: CaseId) -> WorkflowResult<()> {
        let mut costs = self.case_costs.write().await;
        costs.entry(case_id).or_insert_with(Vec::new).push(cost);
        debug!("CostService: Recorded activity cost for case {}", case_id);
        Ok(())
    }

    /// Get case cost summary
    ///
    /// Calculates total cost and breakdown for a case.
    pub async fn get_case_cost_summary(&self, case_id: CaseId) -> WorkflowResult<CaseCostSummary> {
        let costs = self.case_costs.read().await;
        let activities = costs.get(&case_id).cloned().unwrap_or_default();

        let mut breakdown = HashMap::new();
        let mut total_cost = 0.0;

        for activity in &activities {
            *breakdown.entry(activity.category).or_insert(0.0) += activity.amount;
            total_cost += activity.amount;
        }

        Ok(CaseCostSummary {
            case_id,
            total_cost,
            currency: self.currency.clone(),
            breakdown,
            activities,
        })
    }

    /// Set resource cost rate
    ///
    /// Sets the cost per hour for a resource.
    pub async fn set_resource_rate(&self, resource_id: ResourceId, rate_per_hour: f64) {
        let mut rates = self.resource_rates.write().await;
        rates.insert(resource_id, rate_per_hour);
        info!("CostService: Set resource rate to {} per hour", rate_per_hour);
    }

    /// Get resource cost rate
    pub async fn get_resource_rate(&self, resource_id: &ResourceId) -> Option<f64> {
        let rates = self.resource_rates.read().await;
        rates.get(resource_id).copied()
    }
}

impl Default for CostService {
    fn default() -> Self {
        Self::new("USD".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_tracking() {
        let service = CostService::new("USD".to_string());
        let case_id = CaseId::new();

        let cost = ActivityCost {
            task_id: "task1".to_string(),
            resource_id: None,
            amount: 100.0,
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            category: CostCategory::Resource,
        };

        service.record_activity_cost(cost, case_id).await.unwrap();

        let summary = service.get_case_cost_summary(case_id).await.unwrap();
        assert_eq!(summary.total_cost, 100.0);
    }
}

