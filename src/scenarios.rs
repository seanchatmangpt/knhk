// TechCorp Scenario Definition
// Complete Fortune 500 RevOps pipeline execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::avatars::{Avatar, Decision, create_avatar};
use crate::knhk_client::MockKnhkClient;
use crate::results::{ScenarioResult, ExecutionTimeline, TimelineEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealScenario {
    pub name: String,
    pub company: String,
    pub acv: u64,
    pub discount: f64,
    pub custom_terms: bool,
    pub company_size: u64,
    pub industry: String,
    pub use_case: String,
    pub budget_indicated: bool,
}

impl DealScenario {
    /// Create the TechCorp scenario
    pub fn techcorp() -> Self {
        Self {
            name: "TechCorp Enterprise Deal".to_string(),
            company: "TechCorp".to_string(),
            acv: 500_000,
            discount: 12.0,
            custom_terms: false,
            company_size: 5000,
            industry: "Technology".to_string(),
            use_case: "Enterprise workflow automation for complex approval processes across 15 departments with compliance requirements".to_string(),
            budget_indicated: true,
        }
    }

    /// Execute the complete workflow
    pub async fn execute(&self) -> Result<ScenarioResult, Box<dyn std::error::Error>> {
        let mut timeline = ExecutionTimeline::new();
        let mut decisions = Vec::new();
        let mut workflow_stages = Vec::new();

        timeline.add_event("scenario_start", &serde_json::json!({
            "scenario": self.name,
            "company": self.company,
            "acv": self.acv,
        }));

        // Stage 1: Lead Qualification (SDR)
        tracing::info!("Starting Stage 1: Lead Qualification");
        let sdr = create_avatar("sdr")?;
        let lead_data = serde_json::json!({
            "company_size": self.company_size,
            "industry": self.industry,
            "use_case": self.use_case,
            "budget_indicated": self.budget_indicated,
        });

        let sdr_decision = sdr.decide(&lead_data)?;
        timeline.add_event("lead_qualification", &serde_json::json!({
            "avatar": sdr.get_name(),
            "outcome": sdr_decision.outcome,
            "decision_time_ms": sdr_decision.decision_time_ms,
        }));

        workflow_stages.push("Lead Qualification".to_string());
        decisions.push(serde_json::json!({
            "stage": "Lead Qualification",
            "avatar": sdr.get_name(),
            "decision": sdr_decision,
        }));

        if sdr_decision.outcome != "QUALIFIED" {
            return Ok(ScenarioResult {
                scenario_name: self.name.clone(),
                timeline,
                decisions,
                workflow_stages,
                metrics: HashMap::new(),
                success: false,
            });
        }

        // Stage 2: Deal Approval (Manager)
        tracing::info!("Starting Stage 2: Deal Approval");
        let manager = create_avatar("manager")?;
        let deal_data = serde_json::json!({
            "acv": self.acv,
        });

        let manager_decision = manager.decide(&deal_data)?;
        timeline.add_event("deal_approval", &serde_json::json!({
            "avatar": manager.get_name(),
            "outcome": manager_decision.outcome,
            "decision_time_ms": manager_decision.decision_time_ms,
        }));

        workflow_stages.push("Deal Approval".to_string());
        decisions.push(serde_json::json!({
            "stage": "Deal Approval",
            "avatar": manager.get_name(),
            "decision": manager_decision.clone(),
        }));

        // If escalated, get CFO approval
        let mut cfo_approved = false;
        if manager_decision.outcome == "ESCALATE_TO_CFO" {
            tracing::info!("Escalating to CFO for approval");
            let cfo = create_avatar("cfo")?;
            let cfo_data = serde_json::json!({
                "acv": self.acv,
                "discount": self.discount,
            });

            let cfo_decision = cfo.decide(&cfo_data)?;
            timeline.add_event("cfo_approval", &serde_json::json!({
                "avatar": cfo.get_name(),
                "outcome": cfo_decision.outcome,
                "decision_time_ms": cfo_decision.decision_time_ms,
            }));

            workflow_stages.push("CFO Approval".to_string());
            decisions.push(serde_json::json!({
                "stage": "CFO Approval",
                "avatar": cfo.get_name(),
                "decision": cfo_decision.clone(),
            }));

            if cfo_decision.outcome == "APPROVED" {
                cfo_approved = true;
            } else {
                return Ok(ScenarioResult {
                    scenario_name: self.name.clone(),
                    timeline,
                    decisions,
                    workflow_stages,
                    metrics: HashMap::new(),
                    success: false,
                });
            }
        }

        // Stage 3 & 4: Parallel Legal and Finance Review
        tracing::info!("Starting Stages 3 & 4: Parallel Legal and Finance Review");

        let legal = create_avatar("legal")?;
        let finance = create_avatar("finance")?;

        let legal_data = serde_json::json!({
            "acv": self.acv,
            "custom_terms": self.custom_terms,
        });

        let finance_data = serde_json::json!({
            "acv": self.acv,
            "discount": self.discount,
        });

        // Execute in parallel
        let legal_handle = {
            let legal = legal.clone();
            let data = legal_data.clone();
            tokio::spawn(async move {
                legal.decide(&data)
            })
        };

        let finance_handle = {
            let finance = finance.clone();
            let data = finance_data.clone();
            tokio::spawn(async move {
                finance.decide(&data)
            })
        };

        let legal_decision = legal_handle.await??;
        let finance_decision = finance_handle.await??;

        timeline.add_event("legal_review", &serde_json::json!({
            "avatar": legal.get_name(),
            "outcome": legal_decision.outcome,
            "decision_time_ms": legal_decision.decision_time_ms,
        }));

        timeline.add_event("finance_review", &serde_json::json!({
            "avatar": finance.get_name(),
            "outcome": finance_decision.outcome,
            "decision_time_ms": finance_decision.decision_time_ms,
        }));

        workflow_stages.push("Legal Review".to_string());
        workflow_stages.push("Finance Review".to_string());

        decisions.push(serde_json::json!({
            "stage": "Legal Review",
            "avatar": legal.get_name(),
            "decision": legal_decision,
        }));

        decisions.push(serde_json::json!({
            "stage": "Finance Review",
            "avatar": finance.get_name(),
            "decision": finance_decision.clone(),
        }));

        // Check if finance escalated to CFO (and CFO hasn't already approved)
        if finance_decision.outcome == "ESCALATE_TO_CFO" && !cfo_approved {
            tracing::info!("Finance escalating to CFO for discount approval");
            let cfo = create_avatar("cfo")?;
            let cfo_data = serde_json::json!({
                "acv": self.acv,
                "discount": self.discount,
            });

            let cfo_decision = cfo.decide(&cfo_data)?;
            timeline.add_event("cfo_discount_approval", &serde_json::json!({
                "avatar": cfo.get_name(),
                "outcome": cfo_decision.outcome,
                "decision_time_ms": cfo_decision.decision_time_ms,
            }));

            workflow_stages.push("CFO Discount Approval".to_string());
            decisions.push(serde_json::json!({
                "stage": "CFO Discount Approval",
                "avatar": cfo.get_name(),
                "decision": cfo_decision.clone(),
            }));

            if cfo_decision.outcome != "APPROVED" {
                return Ok(ScenarioResult {
                    scenario_name: self.name.clone(),
                    timeline,
                    decisions,
                    workflow_stages,
                    metrics: HashMap::new(),
                    success: false,
                });
            }
        }

        // Stage 5: Revenue Recognition
        tracing::info!("Starting Stage 5: Revenue Recognition");
        timeline.add_event("revenue_recognition", &serde_json::json!({
            "acv": self.acv,
            "status": "booked",
        }));
        workflow_stages.push("Revenue Recognition".to_string());

        // Calculate metrics
        let mut metrics = HashMap::new();
        let total_time_ms: u64 = timeline.events.iter()
            .map(|e| {
                e.data.get("decision_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            })
            .sum();

        let total_time_hours = total_time_ms as f64 / 3600000.0;
        metrics.insert("total_cycle_time_hours".to_string(), serde_json::json!(total_time_hours));
        metrics.insert("num_stages".to_string(), serde_json::json!(workflow_stages.len()));
        metrics.insert("num_decisions".to_string(), serde_json::json!(decisions.len()));
        metrics.insert("acv".to_string(), serde_json::json!(self.acv));
        metrics.insert("discount".to_string(), serde_json::json!(self.discount));

        timeline.add_event("scenario_complete", &serde_json::json!({
            "total_time_hours": total_time_hours,
            "success": true,
        }));

        Ok(ScenarioResult {
            scenario_name: self.name.clone(),
            timeline,
            decisions,
            workflow_stages,
            metrics,
            success: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_techcorp_scenario() {
        tracing_subscriber::fmt::init();

        let scenario = DealScenario::techcorp();
        let result = scenario.execute().await.unwrap();

        assert!(result.success);
        assert!(result.workflow_stages.len() >= 5);
        assert!(!result.decisions.is_empty());
    }
}
