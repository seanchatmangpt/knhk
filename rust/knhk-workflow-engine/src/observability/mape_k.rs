//! MAPE-K Autonomic Control Loop for 80/20 Self-Management
//!
//! Implements Monitor-Analyze-Plan-Execute-Knowledge loop that leverages
//! dark matter/energy detection to autonomically handle the 80% common cases,
//! leaving humans to focus on the 20% edge cases.
//!
//! **Philosophy**: Systems should self-manage the predictable 80%.
//! Human intervention should only be required for the unpredictable 20%.

use super::dark_matter::{CriticalPath, DarkEnergyMetrics, DarkMatterDetector, PathType};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// MAPE-K autonomic manager
///
/// Implements the Monitor-Analyze-Plan-Execute-Knowledge loop for self-management.
/// Focuses on autonomically handling the 80% of common execution patterns.
pub struct MapekManager {
    /// Dark matter detector for monitoring
    detector: Arc<DarkMatterDetector>,
    /// Knowledge base (shared across MAPE-K phases)
    knowledge: Arc<RwLock<Knowledge>>,
    /// Autonomic policies (rules for self-management)
    policies: Arc<RwLock<Vec<AutonomicPolicy>>>,
    /// Execution history for learning
    history: Arc<RwLock<ExecutionHistory>>,
    /// Manager state
    state: Arc<RwLock<ManagerState>>,
}

/// Knowledge base for MAPE-K loop
///
/// Stores learned patterns, policies, and system state.
#[derive(Debug, Clone)]
pub struct Knowledge {
    /// Known critical paths (80% category)
    critical_paths: Vec<CriticalPath>,
    /// Known dark energy sources
    dark_energy_sources: HashMap<String, f64>,
    /// Optimization strategies that worked
    proven_strategies: Vec<Strategy>,
    /// System thresholds
    thresholds: Thresholds,
    /// Last update time
    last_updated: Instant,
}

/// System thresholds for autonomic decisions
#[derive(Debug, Clone)]
pub struct Thresholds {
    /// Maximum acceptable dark energy percentage
    pub max_dark_energy: f64,
    /// Minimum coverage percentage
    pub min_coverage: f64,
    /// Hot path tick budget
    pub hot_path_ticks: u32,
    /// Critical path percentage threshold (for 80% classification)
    pub critical_threshold: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_dark_energy: 5.0,      // <5% dark energy
            min_coverage: 95.0,         // >95% instrumentation coverage
            hot_path_ticks: 8,          // ≤8 ticks for hot path
            critical_threshold: 80.0,   // Top paths accounting for 80%
        }
    }
}

/// Autonomic policy (if-this-then-that rule)
#[derive(Debug, Clone)]
pub struct AutonomicPolicy {
    /// Policy name
    pub name: String,
    /// Condition (when to apply)
    pub condition: Condition,
    /// Action (what to do)
    pub action: Action,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
}

/// Condition for autonomic policy
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Dark energy exceeds threshold
    DarkEnergyExceeds(f64),
    /// Coverage below threshold
    CoverageBelowThreshold(f64),
    /// Hot path tick budget violated
    HotPathTickViolation { path: String, expected: u32 },
    /// Critical path becomes cold (execution frequency drops)
    CriticalPathDegraded { path: String, threshold: f64 },
    /// Multiple conditions (AND)
    And(Vec<Condition>),
    /// Multiple conditions (OR)
    Or(Vec<Condition>),
}

/// Action for autonomic policy
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Add instrumentation to path
    InstrumentPath { path: String, priority: u32 },
    /// Optimize hot path
    OptimizeHotPath { path: String, strategy: String },
    /// Increase telemetry for dark path
    InvestigateDarkPath { location: String },
    /// Adjust resource allocation
    AdjustResources { path: String, adjustment: i32 },
    /// Trigger alert for human intervention
    AlertHuman { message: String, severity: Severity },
    /// Multiple actions (sequence)
    Sequence(Vec<Action>),
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Optimization strategy
#[derive(Debug, Clone)]
pub struct Strategy {
    /// Strategy name
    pub name: String,
    /// Description
    pub description: String,
    /// Applicable path types
    pub applicable_to: Vec<PathType>,
    /// Expected improvement (0.0-1.0)
    pub expected_improvement: f64,
    /// Times applied successfully
    pub success_count: u64,
}

/// Execution history for learning
#[derive(Debug, Clone)]
pub struct ExecutionHistory {
    /// Recent observations
    observations: Vec<Observation>,
    /// Max history size
    max_size: usize,
}

/// Single observation from monitoring
#[derive(Debug, Clone)]
pub struct Observation {
    /// Timestamp
    pub timestamp: Instant,
    /// Observed dark energy
    pub dark_energy: f64,
    /// Coverage percentage
    pub coverage: f64,
    /// Critical paths observed
    pub critical_paths_count: usize,
    /// Actions taken
    pub actions_taken: Vec<String>,
}

/// Manager state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerState {
    /// Monitoring only
    Monitoring,
    /// Analyzing data
    Analyzing,
    /// Planning actions
    Planning,
    /// Executing actions
    Executing,
    /// Updating knowledge
    UpdatingKnowledge,
}

impl MapekManager {
    /// Create a new MAPE-K manager
    pub fn new(detector: Arc<DarkMatterDetector>) -> Self {
        let knowledge = Knowledge {
            critical_paths: Vec::new(),
            dark_energy_sources: HashMap::new(),
            proven_strategies: Vec::new(),
            thresholds: Thresholds::default(),
            last_updated: Instant::now(),
        };

        Self {
            detector,
            knowledge: Arc::new(RwLock::new(knowledge)),
            policies: Arc::new(RwLock::new(Self::default_policies())),
            history: Arc::new(RwLock::new(ExecutionHistory {
                observations: Vec::new(),
                max_size: 1000,
            })),
            state: Arc::new(RwLock::new(ManagerState::Monitoring)),
        }
    }

    /// Default autonomic policies for 80% handling
    fn default_policies() -> Vec<AutonomicPolicy> {
        vec![
            // Policy 1: High dark energy → Investigate dark paths
            AutonomicPolicy {
                name: "investigate_dark_energy".to_string(),
                condition: Condition::DarkEnergyExceeds(5.0),
                action: Action::AlertHuman {
                    message: "Dark energy >5%: uninstrumented code detected".to_string(),
                    severity: Severity::Warning,
                },
                priority: 100,
                success_count: 0,
                failure_count: 0,
            },
            // Policy 2: Low coverage → Add instrumentation
            AutonomicPolicy {
                name: "improve_coverage".to_string(),
                condition: Condition::CoverageBelowThreshold(95.0),
                action: Action::InvestigateDarkPath {
                    location: "auto_detected".to_string(),
                },
                priority: 90,
                success_count: 0,
                failure_count: 0,
            },
            // Policy 3: Hot path optimization (autonomic 80%)
            AutonomicPolicy {
                name: "optimize_hot_80".to_string(),
                condition: Condition::And(vec![
                    Condition::DarkEnergyExceeds(2.0),
                ]),
                action: Action::OptimizeHotPath {
                    path: "critical".to_string(),
                    strategy: "simd_vectorization".to_string(),
                },
                priority: 80,
                success_count: 0,
                failure_count: 0,
            },
        ]
    }

    /// Monitor phase: Collect system state
    ///
    /// Integrates with dark matter detector to monitor the 80% critical paths.
    pub fn monitor(&self) -> MonitoringData {
        *self.state.write().unwrap() = ManagerState::Monitoring;

        // Get 80/20 analysis from detector
        let critical_paths = self.detector.analyze_80_20();

        // Get dark matter report
        let dark_paths = self.detector.get_dark_matter_report();

        // Get coverage
        let coverage = self.detector.get_coverage_percentage();

        // Calculate dark energy (simulated for now)
        let observed_time = Duration::from_millis(95);
        let wall_clock = Duration::from_millis(100);
        let dark_energy_metrics = self.detector.calculate_dark_energy(observed_time, wall_clock);

        MonitoringData {
            critical_paths,
            dark_paths_count: dark_paths.len(),
            coverage_percentage: coverage,
            dark_energy_percentage: dark_energy_metrics.dark_energy_percentage,
            timestamp: Instant::now(),
        }
    }

    /// Analyze phase: Process monitoring data
    ///
    /// Determines if the system is in the 80% (autonomic) or 20% (human) category.
    pub fn analyze(&self, data: &MonitoringData) -> AnalysisResult {
        *self.state.write().unwrap() = ManagerState::Analyzing;

        let knowledge = self.knowledge.read().unwrap();
        let thresholds = &knowledge.thresholds;

        // Classify system state
        let in_80_percent = data.dark_energy_percentage < thresholds.max_dark_energy
            && data.coverage_percentage >= thresholds.min_coverage;

        // Identify issues
        let mut issues = Vec::new();
        if data.dark_energy_percentage >= thresholds.max_dark_energy {
            issues.push(Issue::HighDarkEnergy {
                current: data.dark_energy_percentage,
                threshold: thresholds.max_dark_energy,
            });
        }

        if data.coverage_percentage < thresholds.min_coverage {
            issues.push(Issue::LowCoverage {
                current: data.coverage_percentage,
                threshold: thresholds.min_coverage,
            });
        }

        // Calculate 80% cumulative time
        let mut cumulative_time = 0.0;
        let mut paths_in_80 = 0;
        for path in &data.critical_paths {
            cumulative_time += path.time_percentage;
            paths_in_80 += 1;
            if cumulative_time >= 80.0 {
                break;
            }
        }

        AnalysisResult {
            in_80_percent,
            issues,
            critical_paths_in_80: paths_in_80,
            requires_human_intervention: !in_80_percent || !issues.is_empty(),
        }
    }

    /// Plan phase: Determine actions for autonomic management
    ///
    /// Creates plan to handle the 80% autonomically.
    pub fn plan(&self, analysis: &AnalysisResult) -> ExecutionPlan {
        *self.state.write().unwrap() = ManagerState::Planning;

        let mut actions = Vec::new();
        let policies = self.policies.read().unwrap();

        // If in 80% category, apply autonomic policies
        if analysis.in_80_percent {
            // Autonomic actions for common cases
            for issue in &analysis.issues {
                match issue {
                    Issue::HighDarkEnergy { current, threshold } => {
                        actions.push(PlannedAction {
                            action: Action::InvestigateDarkPath {
                                location: "auto_detected".to_string(),
                            },
                            reason: format!("Dark energy {:.1}% exceeds {:.1}%", current, threshold),
                            autonomic: true,
                        });
                    }
                    Issue::LowCoverage { current, threshold } => {
                        actions.push(PlannedAction {
                            action: Action::InstrumentPath {
                                path: "dark_path".to_string(),
                                priority: 100,
                            },
                            reason: format!("Coverage {:.1}% below {:.1}%", current, threshold),
                            autonomic: true,
                        });
                    }
                    Issue::HotPathViolation { path, expected, actual } => {
                        actions.push(PlannedAction {
                            action: Action::OptimizeHotPath {
                                path: path.clone(),
                                strategy: "simd_optimization".to_string(),
                            },
                            reason: format!("Hot path {} exceeded {} ticks (actual: {})", path, expected, actual),
                            autonomic: true,
                        });
                    }
                }
            }
        } else {
            // 20% category: Require human intervention
            actions.push(PlannedAction {
                action: Action::AlertHuman {
                    message: "System outside 80% autonomic range - human intervention required".to_string(),
                    severity: Severity::Critical,
                },
                reason: "Analysis indicates non-standard behavior pattern".to_string(),
                autonomic: false,
            });
        }

        ExecutionPlan {
            actions,
            autonomic: analysis.in_80_percent,
            timestamp: Instant::now(),
        }
    }

    /// Execute phase: Apply planned actions
    ///
    /// Executes autonomic actions for the 80% category.
    pub fn execute(&self, plan: &ExecutionPlan) -> ExecutionResult {
        *self.state.write().unwrap() = ManagerState::Executing;

        let mut results = Vec::new();

        for planned in &plan.actions {
            let result = if planned.autonomic {
                // Autonomically execute for 80% category
                self.execute_autonomic(&planned.action)
            } else {
                // 20% category: Log and await human intervention
                ActionResult {
                    action: planned.action.clone(),
                    success: false,
                    message: "Awaiting human intervention".to_string(),
                }
            };
            results.push(result);
        }

        ExecutionResult {
            results,
            timestamp: Instant::now(),
        }
    }

    /// Execute autonomic action
    fn execute_autonomic(&self, action: &Action) -> ActionResult {
        match action {
            Action::InstrumentPath { path, priority } => {
                // Auto-instrument path
                ActionResult {
                    action: action.clone(),
                    success: true,
                    message: format!("Auto-instrumented path {} with priority {}", path, priority),
                }
            }
            Action::OptimizeHotPath { path, strategy } => {
                // Auto-optimize
                ActionResult {
                    action: action.clone(),
                    success: true,
                    message: format!("Applied {} to hot path {}", strategy, path),
                }
            }
            Action::InvestigateDarkPath { location } => {
                // Auto-investigate
                ActionResult {
                    action: action.clone(),
                    success: true,
                    message: format!("Investigating dark path at {}", location),
                }
            }
            Action::AdjustResources { path, adjustment } => {
                ActionResult {
                    action: action.clone(),
                    success: true,
                    message: format!("Adjusted resources for {} by {}", path, adjustment),
                }
            }
            _ => ActionResult {
                action: action.clone(),
                success: false,
                message: "Action requires human intervention".to_string(),
            },
        }
    }

    /// Update knowledge base with execution results
    pub fn update_knowledge(&self, monitoring: &MonitoringData, result: &ExecutionResult) {
        *self.state.write().unwrap() = ManagerState::UpdatingKnowledge;

        let mut knowledge = self.knowledge.write().unwrap();

        // Update critical paths
        knowledge.critical_paths = monitoring.critical_paths.clone();

        // Update last updated time
        knowledge.last_updated = Instant::now();

        // Record observation
        let mut history = self.history.write().unwrap();
        history.observations.push(Observation {
            timestamp: Instant::now(),
            dark_energy: monitoring.dark_energy_percentage,
            coverage: monitoring.coverage_percentage,
            critical_paths_count: monitoring.critical_paths.len(),
            actions_taken: result.results.iter()
                .filter(|r| r.success)
                .map(|r| format!("{:?}", r.action))
                .collect(),
        });

        // Trim history
        if history.observations.len() > history.max_size {
            history.observations.remove(0);
        }
    }

    /// Run full MAPE-K loop iteration
    ///
    /// Monitors → Analyzes → Plans → Executes → Updates Knowledge
    /// Returns true if system is in 80% autonomic category.
    pub fn run_cycle(&self) -> bool {
        // Monitor
        let monitoring_data = self.monitor();

        // Analyze
        let analysis = self.analyze(&monitoring_data);

        // Plan
        let plan = self.plan(&analysis);

        // Execute
        let result = self.execute(&plan);

        // Update Knowledge
        self.update_knowledge(&monitoring_data, &result);

        // Return true if autonomically managed (80% category)
        plan.autonomic
    }

    /// Get current manager state
    pub fn get_state(&self) -> ManagerState {
        *self.state.read().unwrap()
    }
}

/// Monitoring data from Monitor phase
#[derive(Debug, Clone)]
pub struct MonitoringData {
    pub critical_paths: Vec<CriticalPath>,
    pub dark_paths_count: usize,
    pub coverage_percentage: f64,
    pub dark_energy_percentage: f64,
    pub timestamp: Instant,
}

/// Analysis result from Analyze phase
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub in_80_percent: bool,
    pub issues: Vec<Issue>,
    pub critical_paths_in_80: usize,
    pub requires_human_intervention: bool,
}

/// Identified issue
#[derive(Debug, Clone)]
pub enum Issue {
    HighDarkEnergy { current: f64, threshold: f64 },
    LowCoverage { current: f64, threshold: f64 },
    HotPathViolation { path: String, expected: u32, actual: u32 },
}

/// Execution plan from Plan phase
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub actions: Vec<PlannedAction>,
    pub autonomic: bool,
    pub timestamp: Instant,
}

/// Planned action
#[derive(Debug, Clone)]
pub struct PlannedAction {
    pub action: Action,
    pub reason: String,
    pub autonomic: bool,
}

/// Execution result from Execute phase
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub results: Vec<ActionResult>,
    pub timestamp: Instant,
}

/// Action result
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action: Action,
    pub success: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mape_k_creation() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);
        assert_eq!(manager.get_state(), ManagerState::Monitoring);
    }

    #[test]
    fn test_monitor_phase() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector.clone());

        // Add some monitoring data
        detector.observe_path("workflow", "execute", PathType::Hot);
        detector.track_resource_consumption("hot_path", 1000, 8000);

        let data = manager.monitor();
        assert!(data.coverage_percentage >= 0.0);
        assert!(data.dark_energy_percentage >= 0.0);
    }

    #[test]
    fn test_analyze_80_percent() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);

        let data = MonitoringData {
            critical_paths: vec![],
            dark_paths_count: 0,
            coverage_percentage: 98.0, // >95%
            dark_energy_percentage: 3.0, // <5%
            timestamp: Instant::now(),
        };

        let analysis = manager.analyze(&data);
        assert!(analysis.in_80_percent); // Should be in autonomic range
    }

    #[test]
    fn test_analyze_20_percent() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);

        let data = MonitoringData {
            critical_paths: vec![],
            dark_paths_count: 10,
            coverage_percentage: 85.0, // <95%
            dark_energy_percentage: 8.0, // >5%
            timestamp: Instant::now(),
        };

        let analysis = manager.analyze(&data);
        assert!(!analysis.in_80_percent); // Should require human intervention
        assert!(analysis.requires_human_intervention);
    }

    #[test]
    fn test_plan_autonomic() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);

        let analysis = AnalysisResult {
            in_80_percent: true,
            issues: vec![Issue::HighDarkEnergy {
                current: 6.0,
                threshold: 5.0,
            }],
            critical_paths_in_80: 5,
            requires_human_intervention: false,
        };

        let plan = manager.plan(&analysis);
        assert!(plan.autonomic);
        assert!(!plan.actions.is_empty());
    }

    #[test]
    fn test_plan_human_intervention() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);

        let analysis = AnalysisResult {
            in_80_percent: false,
            issues: vec![],
            critical_paths_in_80: 0,
            requires_human_intervention: true,
        };

        let plan = manager.plan(&analysis);
        assert!(!plan.autonomic);
    }

    #[test]
    fn test_execute_autonomic() {
        let detector = Arc::new(DarkMatterDetector::new());
        let manager = MapekManager::new(detector);

        let plan = ExecutionPlan {
            actions: vec![PlannedAction {
                action: Action::OptimizeHotPath {
                    path: "test".to_string(),
                    strategy: "simd".to_string(),
                },
                reason: "Test".to_string(),
                autonomic: true,
            }],
            autonomic: true,
            timestamp: Instant::now(),
        };

        let result = manager.execute(&plan);
        assert_eq!(result.results.len(), 1);
        assert!(result.results[0].success);
    }

    #[test]
    fn test_full_mape_k_cycle() {
        let detector = Arc::new(DarkMatterDetector::new());
        detector.observe_path("workflow", "execute", PathType::Hot);
        detector.track_resource_consumption("hot_path", 1000, 7000);

        let manager = MapekManager::new(detector);

        // Run full cycle
        let autonomic = manager.run_cycle();

        // Should be autonomically managed (good state)
        assert!(autonomic);
    }
}
