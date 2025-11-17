//! Andon & Gemba Integration - Complete Monitoring System
//!
//! Demonstrates how Andon (visual control) and Gemba (real-time observation)
//! work together to provide comprehensive workflow monitoring.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::monitoring::{AndonSystem, AndonConfig, GembaWalker, ObservationPoint, ObservationContext, ObservationMetrics};
//! use std::sync::Arc;
//!
//! // Create Andon system with custom config
//! let andon_config = AndonConfig {
//!     max_latency_us: 500_000,      // 500ms warning
//!     critical_latency_us: 1_000_000, // 1s critical
//!     max_error_rate: 0.1,          // 10% error rate
//!     max_tick_budget: 8,           // Chatman Constant
//!     auto_stop_on_red: true,        // Auto-stop on critical issues
//!     alert_retention_secs: 3600,    // 1 hour retention
//! };
//!
//! let andon = Arc::new(AndonSystem::new(andon_config));
//!
//! // Create Gemba walker (observes execution)
//! let gemba = GembaWalker::new(andon.clone(), 1000);
//!
//! // During workflow execution, observe at key points
//! gemba.observe(
//!     ObservationPoint::TaskExecution,
//!     workflow_id,
//!     ObservationContext {
//!         case_state: Some(CaseState::Running),
//!         variables: HashMap::new(),
//!         stack_depth: 1,
//!         active_tasks: vec!["task1".to_string()],
//!         metadata: HashMap::new(),
//!     },
//!     ObservationMetrics {
//!         latency_us: 250_000,  // 250ms
//!         ticks_used: 5,
//!         memory_bytes: None,
//!         cpu_percent: None,
//!     },
//! ).await?;
//!
//! // Check Andon state
//! let state = andon.state().await;
//! match state {
//!     AndonState::Green => println!("All systems operational"),
//!     AndonState::Yellow => println!("Warning conditions present"),
//!     AndonState::Red => {
//!         println!("Critical issue - workflows stopped");
//!         // Handle critical situation
//!     }
//! }
//!
//! // Get performance summary
//! let summary = gemba.get_performance_summary().await;
//! println!("Average latency: {}μs", summary.avg_latency_us);
//! println!("Average ticks: {}", summary.avg_ticks);
//! ```

use crate::monitoring::{AndonSystem, GembaWalker};
use std::sync::Arc;

/// Integrated monitoring system combining Andon and Gemba
pub struct IntegratedMonitoringSystem {
    /// Andon visual control system
    pub andon: Arc<AndonSystem>,
    /// Gemba real-time observer
    pub gemba: GembaWalker,
}

impl IntegratedMonitoringSystem {
    /// Create new integrated monitoring system
    pub fn new(andon: Arc<AndonSystem>, max_observations: usize) -> Self {
        let gemba = GembaWalker::new(andon.clone(), max_observations);
        Self { andon, gemba }
    }

    /// Get comprehensive system health status
    pub async fn get_health_status(&self) -> SystemHealthStatus {
        let andon_state = self.andon.state().await;
        let performance = self.gemba.get_performance_summary().await;
        let alerts = self.andon.get_alerts().await;

        SystemHealthStatus {
            andon_state,
            performance,
            active_alerts: alerts.len(),
            critical_alerts: alerts
                .iter()
                .filter(|a| a.severity == crate::monitoring::AndonState::Red)
                .count(),
            warnings: alerts
                .iter()
                .filter(|a| a.severity == crate::monitoring::AndonState::Yellow)
                .count(),
        }
    }
}

/// System health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemHealthStatus {
    /// Current Andon state
    pub andon_state: crate::monitoring::AndonState,
    /// Performance summary
    pub performance: crate::monitoring::PerformanceSummary,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Number of critical (red) alerts
    pub critical_alerts: usize,
    /// Number of warning (yellow) alerts
    pub warnings: usize,
}
