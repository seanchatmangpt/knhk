// Results Capture and JSON Output System
// Comprehensive execution tracking and analysis output

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    pub events: Vec<TimelineEvent>,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl ExecutionTimeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: chrono::Utc::now(),
        }
    }

    pub fn add_event(&mut self, event_type: &str, data: &serde_json::Value) {
        self.events.push(TimelineEvent {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now(),
            data: data.clone(),
        });
    }

    pub fn total_duration_ms(&self) -> u64 {
        if self.events.is_empty() {
            return 0;
        }

        let last_event = &self.events[self.events.len() - 1];
        let duration = last_event.timestamp - self.start_time;
        duration.num_milliseconds() as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub timeline: ExecutionTimeline,
    pub decisions: Vec<serde_json::Value>,
    pub workflow_stages: Vec<String>,
    pub metrics: HashMap<String, serde_json::Value>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOCompliance {
    pub target_hours: u64,
    pub actual_hours: f64,
    pub compliant: bool,
    pub variance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveResults {
    pub scenario: ScenarioResult,
    pub slo_compliance: Vec<SLOCompliance>,
    pub automation_percentage: f64,
    pub total_cycle_time_hours: f64,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub execution_summary: ExecutionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_decisions: usize,
    pub successful_decisions: usize,
    pub escalations: usize,
    pub parallel_executions: usize,
    pub avatar_participation: HashMap<String, usize>,
}

impl ComprehensiveResults {
    pub fn from_scenario(scenario: ScenarioResult) -> Self {
        let total_cycle_time_hours = scenario.metrics
            .get("total_cycle_time_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Calculate SLO compliance
        let mut slo_compliance = Vec::new();

        // Example: Calculate SLO for each stage
        for decision in &scenario.decisions {
            if let Some(stage) = decision.get("stage").and_then(|s| s.as_str()) {
                if let Some(decision_data) = decision.get("decision") {
                    if let Some(decision_time_ms) = decision_data.get("decision_time_ms").and_then(|v| v.as_u64()) {
                        let actual_hours = decision_time_ms as f64 / 3600000.0;

                        // Default SLA targets (can be customized per stage)
                        let target_hours = match stage {
                            "CFO Approval" | "CFO Discount Approval" => 2,
                            "Finance Review" => 12,
                            _ => 24,
                        };

                        let variance = ((actual_hours - target_hours as f64) / target_hours as f64) * 100.0;
                        let compliant = actual_hours <= (target_hours as f64 * 1.1); // 10% buffer

                        slo_compliance.push(SLOCompliance {
                            target_hours,
                            actual_hours,
                            compliant,
                            variance_percentage: variance,
                        });
                    }
                }
            }
        }

        // Calculate execution summary
        let mut avatar_participation = HashMap::new();
        let mut escalations = 0;
        let mut successful_decisions = 0;

        for decision in &scenario.decisions {
            if let Some(avatar_name) = decision.get("avatar").and_then(|a| a.as_str()) {
                *avatar_participation.entry(avatar_name.to_string()).or_insert(0) += 1;
            }

            if let Some(decision_data) = decision.get("decision") {
                if let Some(outcome) = decision_data.get("outcome").and_then(|o| o.as_str()) {
                    if outcome.contains("ESCALATE") {
                        escalations += 1;
                    }
                    if outcome.contains("APPROVED") || outcome.contains("QUALIFIED") {
                        successful_decisions += 1;
                    }
                }
            }
        }

        // Count parallel executions (Legal + Finance)
        let parallel_executions = if scenario.workflow_stages.contains(&"Legal Review".to_string())
            && scenario.workflow_stages.contains(&"Finance Review".to_string()) {
            1
        } else {
            0
        };

        let execution_summary = ExecutionSummary {
            total_decisions: scenario.decisions.len(),
            successful_decisions,
            escalations,
            parallel_executions,
            avatar_participation,
        };

        // Calculate automation percentage (decisions automated vs manual)
        let automation_percentage = if scenario.decisions.len() > 0 {
            (successful_decisions as f64 / scenario.decisions.len() as f64) * 100.0
        } else {
            0.0
        };

        Self {
            scenario,
            slo_compliance,
            automation_percentage,
            total_cycle_time_hours,
            generated_at: chrono::Utc::now(),
            execution_summary,
        }
    }

    /// Save results to JSON file
    pub async fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(path, json).await?;

        Ok(())
    }

    /// Load results from JSON file
    pub async fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(path).await?;
        let results: Self = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(results)
    }

    /// Generate human-readable summary
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("=== {} ===\n\n", self.scenario.scenario_name));
        summary.push_str(&format!("Execution Status: {}\n", if self.scenario.success { "SUCCESS" } else { "FAILED" }));
        summary.push_str(&format!("Total Cycle Time: {:.2} hours\n", self.total_cycle_time_hours));
        summary.push_str(&format!("Automation Rate: {:.1}%\n", self.automation_percentage));
        summary.push_str(&format!("Generated: {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        summary.push_str("=== Execution Summary ===\n");
        summary.push_str(&format!("Total Decisions: {}\n", self.execution_summary.total_decisions));
        summary.push_str(&format!("Successful: {}\n", self.execution_summary.successful_decisions));
        summary.push_str(&format!("Escalations: {}\n", self.execution_summary.escalations));
        summary.push_str(&format!("Parallel Executions: {}\n\n", self.execution_summary.parallel_executions));

        summary.push_str("=== Avatar Participation ===\n");
        for (avatar, count) in &self.execution_summary.avatar_participation {
            summary.push_str(&format!("{}: {} decision(s)\n", avatar, count));
        }
        summary.push_str("\n");

        summary.push_str("=== SLO Compliance ===\n");
        let compliant_count = self.slo_compliance.iter().filter(|s| s.compliant).count();
        let total_slos = self.slo_compliance.len();
        summary.push_str(&format!("Compliance Rate: {}/{} ({:.1}%)\n",
            compliant_count, total_slos,
            if total_slos > 0 { (compliant_count as f64 / total_slos as f64) * 100.0 } else { 0.0 }
        ));

        for (i, slo) in self.slo_compliance.iter().enumerate() {
            summary.push_str(&format!("  Stage {}: Target {}h, Actual {:.2}h, {} ({:+.1}%)\n",
                i + 1,
                slo.target_hours,
                slo.actual_hours,
                if slo.compliant { "✓" } else { "✗" },
                slo.variance_percentage
            ));
        }
        summary.push_str("\n");

        summary.push_str("=== Workflow Stages ===\n");
        for (i, stage) in self.scenario.workflow_stages.iter().enumerate() {
            summary.push_str(&format!("{}. {}\n", i + 1, stage));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline() {
        let mut timeline = ExecutionTimeline::new();
        timeline.add_event("test", &serde_json::json!({"data": "value"}));

        assert_eq!(timeline.events.len(), 1);
        assert_eq!(timeline.events[0].event_type, "test");
    }

    #[tokio::test]
    async fn test_save_load() {
        let timeline = ExecutionTimeline::new();
        let mut metrics = HashMap::new();
        metrics.insert("test".to_string(), serde_json::json!(123));

        let scenario = ScenarioResult {
            scenario_name: "Test".to_string(),
            timeline,
            decisions: vec![],
            workflow_stages: vec![],
            metrics,
            success: true,
        };

        let results = ComprehensiveResults::from_scenario(scenario);

        let path = Path::new("/tmp/test_results.json");
        results.save_to_file(path).await.unwrap();

        let loaded = ComprehensiveResults::load_from_file(path).await.unwrap();
        assert_eq!(loaded.scenario.scenario_name, "Test");
    }
}
