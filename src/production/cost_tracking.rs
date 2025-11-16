// KNHK Cost Tracking - Resource Usage and Cost Accounting
// Phase 5: Production-grade cost tracking for Fortune 500 deployments
// Provides detailed cost breakdowns and ROI calculations

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tracing::{info, debug, instrument};
use dashmap::DashMap;
use crate::autonomic::Receipt;

const COST_AGGREGATION_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
const COST_HISTORY_DAYS: u64 = 90;
const BILLING_PRECISION: f64 = 0.0001; // 4 decimal places

/// Resource usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_core_seconds: f64,
    pub memory_gb_seconds: f64,
    pub storage_gb_hours: f64,
    pub network_gb: f64,
    pub io_operations: u64,
    pub api_calls: u64,
}

/// Cost breakdown by resource type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub compute_cost: f64,
    pub memory_cost: f64,
    pub storage_cost: f64,
    pub network_cost: f64,
    pub io_cost: f64,
    pub api_cost: f64,
    pub total_cost: f64,
}

/// Pricing model for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    pub cpu_per_core_hour: f64,
    pub memory_per_gb_hour: f64,
    pub storage_per_gb_month: f64,
    pub network_per_gb: f64,
    pub io_per_million: f64,
    pub api_per_million: f64,
    pub currency: String,
}

impl Default for PricingModel {
    fn default() -> Self {
        // AWS-like pricing model for reference
        Self {
            cpu_per_core_hour: 0.0416,      // ~$30/month per core
            memory_per_gb_hour: 0.00465,    // ~$3.35/month per GB
            storage_per_gb_month: 0.023,    // S3 standard
            network_per_gb: 0.09,           // Data transfer out
            io_per_million: 0.10,           // I/O operations
            api_per_million: 0.40,          // API Gateway
            currency: "USD".to_string(),
        }
    }
}

/// Cost allocation by department/project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAllocation {
    pub allocation_id: String,
    pub department: String,
    pub project: String,
    pub cost_center: String,
    pub workflow_count: u64,
    pub resource_usage: ResourceUsage,
    pub cost: CostBreakdown,
    pub period_start: SystemTime,
    pub period_end: SystemTime,
}

/// Cost comparison with legacy systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComparison {
    pub period: String,
    pub knhk_cost: f64,
    pub legacy_cost: f64,
    pub savings: f64,
    pub savings_percent: f64,
    pub workflows_processed: u64,
    pub cost_per_workflow_knhk: f64,
    pub cost_per_workflow_legacy: f64,
}

/// ROI calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROICalculation {
    pub investment: f64,
    pub returns: f64,
    pub roi_percent: f64,
    pub payback_period_months: f64,
    pub break_even_date: Option<SystemTime>,
    pub cumulative_savings: f64,
}

/// Budget tracking and alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTracker {
    pub budget_limit: f64,
    pub current_spend: f64,
    pub projected_spend: f64,
    pub remaining_budget: f64,
    pub utilization_percent: f64,
    pub alert_threshold: f64,
    pub alerts_triggered: Vec<BudgetAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub timestamp: SystemTime,
    pub alert_type: BudgetAlertType,
    pub current_spend: f64,
    pub threshold: f64,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetAlertType {
    Threshold50,
    Threshold75,
    Threshold90,
    BudgetExceeded,
}

/// Main cost tracking system
pub struct CostTracker {
    // Pricing configuration
    pricing_model: Arc<RwLock<PricingModel>>,

    // Usage tracking
    workflow_usage: Arc<DashMap<String, ResourceUsage>>,
    aggregated_usage: Arc<RwLock<ResourceUsage>>,

    // Cost calculations
    workflow_costs: Arc<DashMap<String, CostBreakdown>>,
    hourly_costs: Arc<RwLock<VecDeque<HourlyCost>>>,
    daily_costs: Arc<RwLock<VecDeque<DailyCost>>>,

    // Cost allocation
    allocations: Arc<DashMap<String, CostAllocation>>,

    // Legacy comparison
    legacy_baseline: Arc<RwLock<LegacyBaseline>>,
    comparisons: Arc<RwLock<Vec<CostComparison>>>,

    // Budget tracking
    budget_tracker: Arc<RwLock<BudgetTracker>>,

    // ROI tracking
    roi_tracker: Arc<RwLock<ROICalculation>>,

    // Statistics
    total_workflows_tracked: Arc<AtomicU64>,
    total_cost_calculated: Arc<AtomicU64>,
    total_savings_achieved: Arc<AtomicU64>,

    // Chargeback
    chargeback_records: Arc<DashMap<String, ChargebackRecord>>,
}

#[derive(Debug, Clone)]
struct HourlyCost {
    hour: u64,
    usage: ResourceUsage,
    cost: CostBreakdown,
    workflow_count: u64,
}

#[derive(Debug, Clone)]
struct DailyCost {
    date: String,
    usage: ResourceUsage,
    cost: CostBreakdown,
    workflow_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyBaseline {
    avg_cost_per_workflow: f64,
    avg_processing_time_ms: f64,
    infrastructure_cost_monthly: f64,
    operational_cost_monthly: f64,
    total_cost_monthly: f64,
}

impl Default for LegacyBaseline {
    fn default() -> Self {
        // Typical enterprise legacy system costs
        Self {
            avg_cost_per_workflow: 0.50,
            avg_processing_time_ms: 5000.0,
            infrastructure_cost_monthly: 50000.0,
            operational_cost_monthly: 25000.0,
            total_cost_monthly: 75000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChargebackRecord {
    record_id: String,
    department: String,
    period: String,
    total_cost: f64,
    workflow_count: u64,
    status: ChargebackStatus,
    invoice_date: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ChargebackStatus {
    Pending,
    Invoiced,
    Paid,
    Disputed,
}

impl CostTracker {
    /// Initialize cost tracker
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing cost tracker");

        let budget_tracker = BudgetTracker {
            budget_limit: 100000.0, // $100k monthly budget
            current_spend: 0.0,
            projected_spend: 0.0,
            remaining_budget: 100000.0,
            utilization_percent: 0.0,
            alert_threshold: 0.75,
            alerts_triggered: Vec::new(),
        };

        let roi_tracker = ROICalculation {
            investment: 500000.0, // Initial investment
            returns: 0.0,
            roi_percent: 0.0,
            payback_period_months: 0.0,
            break_even_date: None,
            cumulative_savings: 0.0,
        };

        Ok(Self {
            pricing_model: Arc::new(RwLock::new(PricingModel::default())),
            workflow_usage: Arc::new(DashMap::new()),
            aggregated_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            workflow_costs: Arc::new(DashMap::new()),
            hourly_costs: Arc::new(RwLock::new(VecDeque::new())),
            daily_costs: Arc::new(RwLock::new(VecDeque::new())),
            allocations: Arc::new(DashMap::new()),
            legacy_baseline: Arc::new(RwLock::new(LegacyBaseline::default())),
            comparisons: Arc::new(RwLock::new(Vec::new())),
            budget_tracker: Arc::new(RwLock::new(budget_tracker)),
            roi_tracker: Arc::new(RwLock::new(roi_tracker)),
            total_workflows_tracked: Arc::new(AtomicU64::new(0)),
            total_cost_calculated: Arc::new(AtomicU64::new(0)),
            total_savings_achieved: Arc::new(AtomicU64::new(0)),
            chargeback_records: Arc::new(DashMap::new()),
        })
    }

    /// Start cost tracking services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting cost tracking services");

        // Start cost aggregator
        self.start_cost_aggregator();

        // Start comparison calculator
        self.start_comparison_calculator();

        // Start budget monitor
        self.start_budget_monitor();

        info!("Cost tracking services started");
        Ok(())
    }

    /// Calculate workflow cost
    #[instrument(skip(self, receipts))]
    pub async fn calculate_workflow_cost(
        &self,
        workflow_id: &str,
        receipts: &[Receipt],
        duration: Duration,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        self.total_workflows_tracked.fetch_add(1, Ordering::Relaxed);

        // Calculate resource usage
        let usage = self.calculate_resource_usage(receipts, duration);

        // Store usage
        self.workflow_usage.insert(workflow_id.to_string(), usage.clone());

        // Update aggregated usage
        {
            let mut agg = self.aggregated_usage.write().unwrap();
            agg.cpu_core_seconds += usage.cpu_core_seconds;
            agg.memory_gb_seconds += usage.memory_gb_seconds;
            agg.storage_gb_hours += usage.storage_gb_hours;
            agg.network_gb += usage.network_gb;
            agg.io_operations += usage.io_operations;
            agg.api_calls += usage.api_calls;
        }

        // Calculate cost breakdown
        let cost = self.calculate_cost_breakdown(&usage);

        // Store cost
        self.workflow_costs.insert(workflow_id.to_string(), cost.clone());

        // Update budget tracker
        {
            let mut budget = self.budget_tracker.write().unwrap();
            budget.current_spend += cost.total_cost;
            budget.remaining_budget = budget.budget_limit - budget.current_spend;
            budget.utilization_percent = (budget.current_spend / budget.budget_limit) * 100.0;

            // Check budget alerts
            self.check_budget_alerts(&mut budget, cost.total_cost);
        }

        // Calculate and track savings vs legacy
        let legacy_cost = self.calculate_legacy_cost(workflow_id).await?;
        let savings = legacy_cost - cost.total_cost;

        if savings > 0.0 {
            self.total_savings_achieved.fetch_add((savings * 100.0) as u64, Ordering::Relaxed);

            // Update ROI
            let mut roi = self.roi_tracker.write().unwrap();
            roi.returns += savings;
            roi.cumulative_savings += savings;
            roi.roi_percent = (roi.returns / roi.investment) * 100.0;

            // Calculate payback period
            let monthly_savings = savings * 30.0 * 24.0 * 3600.0 / duration.as_secs() as f64;
            if monthly_savings > 0.0 {
                roi.payback_period_months = roi.investment / monthly_savings;

                // Calculate break-even date
                if roi.cumulative_savings >= roi.investment && roi.break_even_date.is_none() {
                    roi.break_even_date = Some(SystemTime::now());
                    info!("KNHK reached break-even point! Cumulative savings: ${:.2}", roi.cumulative_savings);
                }
            }
        }

        self.total_cost_calculated.fetch_add((cost.total_cost * 100.0) as u64, Ordering::Relaxed);

        debug!("Workflow {} cost: ${:.4} (saved ${:.4} vs legacy)",
            workflow_id, cost.total_cost, savings);

        Ok(cost.total_cost)
    }

    /// Calculate resource usage from receipts
    fn calculate_resource_usage(&self, _receipts: &[Receipt], duration: Duration) -> ResourceUsage {
        // Estimate resource usage based on workflow execution
        // In production, this would read actual metrics

        let seconds = duration.as_secs_f64();

        ResourceUsage {
            cpu_core_seconds: seconds * 0.5, // Assume 0.5 cores average
            memory_gb_seconds: seconds * 2.0, // Assume 2GB average
            storage_gb_hours: 0.001,         // Minimal storage
            network_gb: 0.01,                // 10MB network
            io_operations: 100,              // 100 I/O ops
            api_calls: 10,                   // 10 API calls
        }
    }

    /// Calculate cost breakdown
    fn calculate_cost_breakdown(&self, usage: &ResourceUsage) -> CostBreakdown {
        let pricing = self.pricing_model.read().unwrap();

        let compute_cost = (usage.cpu_core_seconds / 3600.0) * pricing.cpu_per_core_hour;
        let memory_cost = (usage.memory_gb_seconds / 3600.0) * pricing.memory_per_gb_hour;
        let storage_cost = (usage.storage_gb_hours * 30.0 * 24.0) * pricing.storage_per_gb_month / (30.0 * 24.0);
        let network_cost = usage.network_gb * pricing.network_per_gb;
        let io_cost = (usage.io_operations as f64 / 1_000_000.0) * pricing.io_per_million;
        let api_cost = (usage.api_calls as f64 / 1_000_000.0) * pricing.api_per_million;

        let total_cost = compute_cost + memory_cost + storage_cost + network_cost + io_cost + api_cost;

        CostBreakdown {
            compute_cost: (compute_cost * 10000.0).round() / 10000.0,
            memory_cost: (memory_cost * 10000.0).round() / 10000.0,
            storage_cost: (storage_cost * 10000.0).round() / 10000.0,
            network_cost: (network_cost * 10000.0).round() / 10000.0,
            io_cost: (io_cost * 10000.0).round() / 10000.0,
            api_cost: (api_cost * 10000.0).round() / 10000.0,
            total_cost: (total_cost * 10000.0).round() / 10000.0,
        }
    }

    /// Calculate equivalent legacy system cost
    async fn calculate_legacy_cost(&self, _workflow_id: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let baseline = self.legacy_baseline.read().unwrap();
        Ok(baseline.avg_cost_per_workflow)
    }

    /// Check and trigger budget alerts
    fn check_budget_alerts(&self, budget: &mut BudgetTracker, _new_cost: f64) {
        let utilization = budget.utilization_percent;

        let trigger_alert = |alert_type: BudgetAlertType, threshold: f64| -> Option<BudgetAlert> {
            if utilization >= threshold {
                Some(BudgetAlert {
                    timestamp: SystemTime::now(),
                    alert_type,
                    current_spend: budget.current_spend,
                    threshold: budget.budget_limit * (threshold / 100.0),
                    message: format!("Budget utilization reached {:.1}%", utilization),
                })
            } else {
                None
            }
        };

        if let Some(alert) = trigger_alert(BudgetAlertType::Threshold90, 90.0) {
            budget.alerts_triggered.push(alert);
        } else if let Some(alert) = trigger_alert(BudgetAlertType::Threshold75, 75.0) {
            budget.alerts_triggered.push(alert);
        } else if let Some(alert) = trigger_alert(BudgetAlertType::Threshold50, 50.0) {
            budget.alerts_triggered.push(alert);
        }

        if utilization > 100.0 {
            budget.alerts_triggered.push(BudgetAlert {
                timestamp: SystemTime::now(),
                alert_type: BudgetAlertType::BudgetExceeded,
                current_spend: budget.current_spend,
                threshold: budget.budget_limit,
                message: format!("Budget exceeded! Current spend: ${:.2}", budget.current_spend),
            });
        }
    }

    /// Generate cost allocation report
    pub async fn generate_allocation_report(
        &self,
        department: &str,
        period_start: SystemTime,
        period_end: SystemTime,
    ) -> CostAllocation {
        let mut total_usage = ResourceUsage::default();
        let mut workflow_count = 0u64;

        // Aggregate usage for department
        for entry in self.workflow_usage.iter() {
            // In production, would filter by department
            let usage = entry.value();
            total_usage.cpu_core_seconds += usage.cpu_core_seconds;
            total_usage.memory_gb_seconds += usage.memory_gb_seconds;
            total_usage.storage_gb_hours += usage.storage_gb_hours;
            total_usage.network_gb += usage.network_gb;
            total_usage.io_operations += usage.io_operations;
            total_usage.api_calls += usage.api_calls;
            workflow_count += 1;
        }

        let cost = self.calculate_cost_breakdown(&total_usage);

        CostAllocation {
            allocation_id: format!("alloc-{}", uuid::Uuid::new_v4()),
            department: department.to_string(),
            project: "KNHK Migration".to_string(),
            cost_center: "IT Operations".to_string(),
            workflow_count,
            resource_usage: total_usage,
            cost,
            period_start,
            period_end,
        }
    }

    /// Start cost aggregator
    fn start_cost_aggregator(&self) {
        let usage = self.aggregated_usage.clone();
        let hourly = self.hourly_costs.clone();
        let daily = self.daily_costs.clone();
        let workflow_count = self.total_workflows_tracked.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(COST_AGGREGATION_INTERVAL);

            loop {
                ticker.tick().await;

                // Aggregate hourly costs
                let current_usage = usage.read().unwrap().clone();
                let count = workflow_count.load(Ordering::Relaxed);

                let hour = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() / 3600;

                // Calculate hourly cost
                // Would be more sophisticated in production

                let mut hourly_costs = hourly.write().unwrap();
                hourly_costs.push_back(HourlyCost {
                    hour,
                    usage: current_usage,
                    cost: CostBreakdown::default(),
                    workflow_count: count,
                });

                // Keep limited history
                if hourly_costs.len() > 24 * 7 { // 7 days
                    hourly_costs.pop_front();
                }

                info!("Hourly cost aggregation complete");
            }
        });
    }

    /// Start comparison calculator
    fn start_comparison_calculator(&self) {
        let comparisons = self.comparisons.clone();
        let budget = self.budget_tracker.clone();
        let legacy = self.legacy_baseline.clone();
        let workflows = self.total_workflows_tracked.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(86400)); // Daily

            loop {
                ticker.tick().await;

                // Calculate daily comparison
                let budget = budget.read().unwrap();
                let legacy = legacy.read().unwrap();
                let workflow_count = workflows.load(Ordering::Relaxed);

                let knhk_cost = budget.current_spend;
                let legacy_cost = workflow_count as f64 * legacy.avg_cost_per_workflow;
                let savings = legacy_cost - knhk_cost;
                let savings_percent = if legacy_cost > 0.0 {
                    (savings / legacy_cost) * 100.0
                } else {
                    0.0
                };

                let comparison = CostComparison {
                    period: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    knhk_cost,
                    legacy_cost,
                    savings,
                    savings_percent,
                    workflows_processed: workflow_count,
                    cost_per_workflow_knhk: if workflow_count > 0 {
                        knhk_cost / workflow_count as f64
                    } else {
                        0.0
                    },
                    cost_per_workflow_legacy: legacy.avg_cost_per_workflow,
                };

                comparisons.write().unwrap().push(comparison);

                info!("Daily cost comparison: KNHK saved ${:.2} ({:.1}%)",
                    savings, savings_percent);
            }
        });
    }

    /// Start budget monitor
    fn start_budget_monitor(&self) {
        let budget = self.budget_tracker.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(3600)); // Hourly

            loop {
                ticker.tick().await;

                let budget = budget.read().unwrap();

                // Project spend for rest of month
                let days_elapsed = 15.0; // Would calculate actual days
                let days_remaining = 30.0 - days_elapsed;
                let daily_rate = budget.current_spend / days_elapsed;
                let projected = budget.current_spend + (daily_rate * days_remaining);

                drop(budget);

                let mut budget = budget.write().unwrap();
                budget.projected_spend = projected;

                if projected > budget.budget_limit {
                    warn!("Projected spend ${:.2} exceeds budget ${:.2}",
                        projected, budget.budget_limit);
                }
            }
        });
    }

    /// Get cost statistics
    pub fn get_stats(&self) -> CostStats {
        let budget = self.budget_tracker.read().unwrap();
        let roi = self.roi_tracker.read().unwrap();

        CostStats {
            total_workflows: self.total_workflows_tracked.load(Ordering::Relaxed),
            total_cost: budget.current_spend,
            average_cost_per_workflow: if self.total_workflows_tracked.load(Ordering::Relaxed) > 0 {
                budget.current_spend / self.total_workflows_tracked.load(Ordering::Relaxed) as f64
            } else {
                0.0
            },
            total_savings: roi.cumulative_savings,
            roi_percent: roi.roi_percent,
            budget_utilization: budget.utilization_percent,
            projected_monthly_cost: budget.projected_spend,
        }
    }

    /// Shutdown cost tracker
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down cost tracker");

        let stats = self.get_stats();
        info!("Final cost statistics:");
        info!("  Total workflows: {}", stats.total_workflows);
        info!("  Total cost: ${:.2}", stats.total_cost);
        info!("  Average per workflow: ${:.4}", stats.average_cost_per_workflow);
        info!("  Total savings: ${:.2}", stats.total_savings);
        info!("  ROI: {:.1}%", stats.roi_percent);

        info!("Cost tracker shutdown complete");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    pub total_workflows: u64,
    pub total_cost: f64,
    pub average_cost_per_workflow: f64,
    pub total_savings: f64,
    pub roi_percent: f64,
    pub budget_utilization: f64,
    pub projected_monthly_cost: f64,
}