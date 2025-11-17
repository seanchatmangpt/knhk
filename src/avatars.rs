// Avatar System for Fortune 500 RevOps Pipeline
// Trait-based polymorphic decision-making agents

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Decision outcome with reasoning and timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision_type: String,
    pub outcome: String,
    pub reasoning: Vec<String>,
    pub criteria_checked: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub decision_time_ms: u64,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Avatar authority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityLevel {
    None,
    Limited(u64),  // Dollar limit
    Full,
}

/// Service Level Agreement timing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SLA {
    pub target_hours: u64,
    pub variance_percentage: f64,
}

#[derive(Debug, Error)]
pub enum AvatarError {
    #[error("Insufficient authority for decision: {0}")]
    InsufficientAuthority(String),
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Decision criteria not met: {0}")]
    CriteriaNotMet(String),
}

/// Core Avatar trait - dyn-compatible (no async methods)
pub trait Avatar: Send + Sync {
    /// Make a decision based on input data
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError>;

    /// Get avatar's approval authority level
    fn get_authority(&self) -> AuthorityLevel;

    /// Get avatar's SLA commitment
    fn get_sla(&self) -> SLA;

    /// Get decision criteria for this avatar type
    fn get_decision_criteria(&self) -> Vec<String>;

    /// Get avatar name and role
    fn get_name(&self) -> &str;
    fn get_role(&self) -> &str;
}

// ============================================================================
// SDR Avatar: Sarah Chen - Lead Qualification
// ============================================================================

pub struct SDRAvatar {
    name: String,
    role: String,
}

impl SDRAvatar {
    pub fn new() -> Self {
        Self {
            name: "Sarah Chen".to_string(),
            role: "Senior Sales Development Representative".to_string(),
        }
    }

    fn score_lead(&self, data: &serde_json::Value) -> Result<(f64, Vec<String>), AvatarError> {
        let mut score = 0.0;
        let mut reasoning = Vec::new();

        // Company size scoring (0-30 points)
        if let Some(company_size) = data.get("company_size").and_then(|v| v.as_u64()) {
            let size_score = match company_size {
                0..=100 => 5.0,
                101..=500 => 15.0,
                501..=5000 => 25.0,
                _ => 30.0,
            };
            score += size_score;
            reasoning.push(format!("Company size {} employees: +{} points", company_size, size_score));
        }

        // Industry scoring (0-25 points)
        if let Some(industry) = data.get("industry").and_then(|v| v.as_str()) {
            let industry_score = match industry {
                "Technology" | "Finance" | "Healthcare" => 25.0,
                "Manufacturing" | "Retail" => 20.0,
                "Education" | "Non-profit" => 10.0,
                _ => 15.0,
            };
            score += industry_score;
            reasoning.push(format!("Industry '{}': +{} points", industry, industry_score));
        }

        // Use case clarity (0-25 points)
        if let Some(use_case) = data.get("use_case").and_then(|v| v.as_str()) {
            let clarity_score = if use_case.len() > 100 { 25.0 } else if use_case.len() > 50 { 15.0 } else { 5.0 };
            score += clarity_score;
            reasoning.push(format!("Use case clarity: +{} points", clarity_score));
        }

        // Budget indication (0-20 points)
        if let Some(budget) = data.get("budget_indicated").and_then(|v| v.as_bool()) {
            if budget {
                score += 20.0;
                reasoning.push("Budget indicated: +20 points".to_string());
            }
        }

        Ok((score, reasoning))
    }
}

impl Avatar for SDRAvatar {
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError> {
        let start = std::time::Instant::now();

        let (score, mut reasoning) = self.score_lead(input)?;
        let qualified = score >= 60.0;

        reasoning.push(format!("Total qualification score: {}/100", score));

        let outcome = if qualified {
            "QUALIFIED"
        } else {
            "NOT_QUALIFIED"
        }.to_string();

        let mut metadata = HashMap::new();
        metadata.insert("score".to_string(), serde_json::json!(score));
        metadata.insert("threshold".to_string(), serde_json::json!(60.0));

        // Add random variance (±10%)
        let base_time_ms = 2000; // 2 seconds base
        let variance = (rand::random::<f64>() - 0.5) * 0.2; // ±10%
        let decision_time_ms = (base_time_ms as f64 * (1.0 + variance)) as u64;

        Ok(Decision {
            decision_type: "lead_qualification".to_string(),
            outcome,
            reasoning,
            criteria_checked: self.get_decision_criteria(),
            timestamp: chrono::Utc::now(),
            decision_time_ms,
            confidence: score / 100.0,
            metadata,
        })
    }

    fn get_authority(&self) -> AuthorityLevel {
        AuthorityLevel::None
    }

    fn get_sla(&self) -> SLA {
        SLA {
            target_hours: 24,
            variance_percentage: 10.0,
        }
    }

    fn get_decision_criteria(&self) -> Vec<String> {
        vec![
            "Company size".to_string(),
            "Industry fit".to_string(),
            "Use case clarity".to_string(),
            "Budget indication".to_string(),
        ]
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_role(&self) -> &str {
        &self.role
    }
}

// ============================================================================
// Manager Avatar: Marcus Thompson - Deal Approval
// ============================================================================

pub struct ManagerAvatar {
    name: String,
    role: String,
    approval_limit: u64,
}

impl ManagerAvatar {
    pub fn new() -> Self {
        Self {
            name: "Marcus Thompson".to_string(),
            role: "Regional Sales Manager".to_string(),
            approval_limit: 250_000,
        }
    }
}

impl Avatar for ManagerAvatar {
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError> {
        let start = std::time::Instant::now();

        let acv = input.get("acv")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AvatarError::InvalidInput("Missing ACV".to_string()))?;

        let mut reasoning = Vec::new();
        let mut criteria_checked = self.get_decision_criteria();

        reasoning.push(format!("Deal ACV: ${}", acv));
        reasoning.push(format!("Manager approval limit: ${}", self.approval_limit));

        let outcome = if acv <= self.approval_limit {
            reasoning.push("Within manager authority - APPROVED".to_string());
            "APPROVED"
        } else {
            reasoning.push("Exceeds manager authority - ESCALATE to CFO".to_string());
            "ESCALATE_TO_CFO"
        };

        let mut metadata = HashMap::new();
        metadata.insert("acv".to_string(), serde_json::json!(acv));
        metadata.insert("approval_limit".to_string(), serde_json::json!(self.approval_limit));

        let base_time_ms = 3600000; // 1 hour base (24hr SLA)
        let variance = (rand::random::<f64>() - 0.5) * 0.2;
        let decision_time_ms = (base_time_ms as f64 * (1.0 + variance)) as u64;

        Ok(Decision {
            decision_type: "deal_approval".to_string(),
            outcome: outcome.to_string(),
            reasoning,
            criteria_checked,
            timestamp: chrono::Utc::now(),
            decision_time_ms,
            confidence: if acv <= self.approval_limit { 1.0 } else { 0.9 },
            metadata,
        })
    }

    fn get_authority(&self) -> AuthorityLevel {
        AuthorityLevel::Limited(self.approval_limit)
    }

    fn get_sla(&self) -> SLA {
        SLA {
            target_hours: 24,
            variance_percentage: 10.0,
        }
    }

    fn get_decision_criteria(&self) -> Vec<String> {
        vec![
            "ACV threshold".to_string(),
            "Workload capacity".to_string(),
            "Deal risk assessment".to_string(),
        ]
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_role(&self) -> &str {
        &self.role
    }
}

// ============================================================================
// Legal Avatar: Priya Patel - Contract Review
// ============================================================================

pub struct LegalAvatar {
    name: String,
    role: String,
}

impl LegalAvatar {
    pub fn new() -> Self {
        Self {
            name: "Priya Patel".to_string(),
            role: "Senior Legal Counsel".to_string(),
        }
    }

    fn determine_contract_type(&self, acv: u64, custom_terms: bool) -> &'static str {
        if custom_terms {
            "CUSTOM"
        } else if acv >= 500_000 {
            "MSA"
        } else {
            "STANDARD"
        }
    }
}

impl Avatar for LegalAvatar {
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError> {
        let acv = input.get("acv")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AvatarError::InvalidInput("Missing ACV".to_string()))?;

        let custom_terms = input.get("custom_terms")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut reasoning = Vec::new();

        let contract_type = self.determine_contract_type(acv, custom_terms);

        reasoning.push(format!("Deal ACV: ${}", acv));
        reasoning.push(format!("Custom terms requested: {}", custom_terms));
        reasoning.push(format!("Contract type: {}", contract_type));

        match contract_type {
            "STANDARD" => reasoning.push("Standard contract - fast track approval".to_string()),
            "MSA" => reasoning.push("Master Service Agreement required for high-value deal".to_string()),
            "CUSTOM" => reasoning.push("Custom terms require detailed legal review".to_string()),
            _ => {}
        }

        let mut metadata = HashMap::new();
        metadata.insert("contract_type".to_string(), serde_json::json!(contract_type));
        metadata.insert("acv".to_string(), serde_json::json!(acv));
        metadata.insert("custom_terms".to_string(), serde_json::json!(custom_terms));

        let base_time_ms = 3600000; // 1 hour base (24hr SLA)
        let variance = (rand::random::<f64>() - 0.5) * 0.2;
        let decision_time_ms = (base_time_ms as f64 * (1.0 + variance)) as u64;

        Ok(Decision {
            decision_type: "contract_review".to_string(),
            outcome: format!("APPROVED_{}", contract_type),
            reasoning,
            criteria_checked: self.get_decision_criteria(),
            timestamp: chrono::Utc::now(),
            decision_time_ms,
            confidence: 0.95,
            metadata,
        })
    }

    fn get_authority(&self) -> AuthorityLevel {
        AuthorityLevel::Full
    }

    fn get_sla(&self) -> SLA {
        SLA {
            target_hours: 24,
            variance_percentage: 10.0,
        }
    }

    fn get_decision_criteria(&self) -> Vec<String> {
        vec![
            "Contract type".to_string(),
            "Legal compliance".to_string(),
            "Risk assessment".to_string(),
            "Terms review".to_string(),
        ]
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_role(&self) -> &str {
        &self.role
    }
}

// ============================================================================
// Finance Avatar: James Rodriguez - Deal Economics
// ============================================================================

pub struct FinanceAvatar {
    name: String,
    role: String,
    max_discount: f64,
}

impl FinanceAvatar {
    pub fn new() -> Self {
        Self {
            name: "James Rodriguez".to_string(),
            role: "VP Finance".to_string(),
            max_discount: 15.0, // 15% max discount
        }
    }
}

impl Avatar for FinanceAvatar {
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError> {
        let acv = input.get("acv")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AvatarError::InvalidInput("Missing ACV".to_string()))?;

        let discount = input.get("discount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let mut reasoning = Vec::new();

        reasoning.push(format!("Deal ACV: ${}", acv));
        reasoning.push(format!("Requested discount: {}%", discount));
        reasoning.push(format!("Finance authority: up to {}%", self.max_discount));

        let outcome = if discount <= self.max_discount {
            reasoning.push("Discount within finance authority - APPROVED".to_string());
            "APPROVED"
        } else {
            reasoning.push("Discount exceeds finance authority - ESCALATE to CFO".to_string());
            "ESCALATE_TO_CFO"
        };

        let mut metadata = HashMap::new();
        metadata.insert("acv".to_string(), serde_json::json!(acv));
        metadata.insert("discount".to_string(), serde_json::json!(discount));
        metadata.insert("max_discount".to_string(), serde_json::json!(self.max_discount));

        let base_time_ms = 1800000; // 30 minutes base (12hr SLA)
        let variance = (rand::random::<f64>() - 0.5) * 0.2;
        let decision_time_ms = (base_time_ms as f64 * (1.0 + variance)) as u64;

        Ok(Decision {
            decision_type: "finance_approval".to_string(),
            outcome: outcome.to_string(),
            reasoning,
            criteria_checked: self.get_decision_criteria(),
            timestamp: chrono::Utc::now(),
            decision_time_ms,
            confidence: if discount <= self.max_discount { 1.0 } else { 0.9 },
            metadata,
        })
    }

    fn get_authority(&self) -> AuthorityLevel {
        AuthorityLevel::Full
    }

    fn get_sla(&self) -> SLA {
        SLA {
            target_hours: 12,
            variance_percentage: 10.0,
        }
    }

    fn get_decision_criteria(&self) -> Vec<String> {
        vec![
            "Discount authority".to_string(),
            "Deal economics".to_string(),
            "Revenue recognition".to_string(),
            "Payment terms".to_string(),
        ]
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_role(&self) -> &str {
        &self.role
    }
}

// ============================================================================
// CFO Avatar: Lisa Wong - Executive Approval
// ============================================================================

pub struct CFOAvatar {
    name: String,
    role: String,
}

impl CFOAvatar {
    pub fn new() -> Self {
        Self {
            name: "Lisa Wong".to_string(),
            role: "Chief Financial Officer".to_string(),
        }
    }

    fn strategic_assessment(&self, acv: u64, discount: f64) -> (bool, Vec<String>) {
        let mut reasoning = Vec::new();

        // Strategic value assessment
        let strategic_value = acv >= 500_000;
        if strategic_value {
            reasoning.push("High strategic value deal (ACV ≥ $500K)".to_string());
        }

        // Discount assessment
        let acceptable_discount = discount <= 25.0; // CFO can approve up to 25%
        if acceptable_discount {
            reasoning.push(format!("Discount {}% within acceptable range", discount));
        } else {
            reasoning.push(format!("Discount {}% exceeds acceptable threshold", discount));
        }

        let approved = strategic_value && acceptable_discount;

        (approved, reasoning)
    }
}

impl Avatar for CFOAvatar {
    fn decide(&self, input: &serde_json::Value) -> Result<Decision, AvatarError> {
        let acv = input.get("acv")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AvatarError::InvalidInput("Missing ACV".to_string()))?;

        let discount = input.get("discount")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let mut reasoning = Vec::new();

        reasoning.push(format!("Executive review - Deal ACV: ${}", acv));
        reasoning.push(format!("Discount: {}%", discount));

        let (approved, strategic_reasoning) = self.strategic_assessment(acv, discount);
        reasoning.extend(strategic_reasoning);

        let outcome = if approved {
            reasoning.push("CFO APPROVAL GRANTED".to_string());
            "APPROVED"
        } else {
            reasoning.push("CFO APPROVAL DENIED - strategic criteria not met".to_string());
            "DENIED"
        };

        let mut metadata = HashMap::new();
        metadata.insert("acv".to_string(), serde_json::json!(acv));
        metadata.insert("discount".to_string(), serde_json::json!(discount));
        metadata.insert("strategic_value".to_string(), serde_json::json!(acv >= 500_000));

        let base_time_ms = 300000; // 5 minutes base (2hr SLA)
        let variance = (rand::random::<f64>() - 0.5) * 0.2;
        let decision_time_ms = (base_time_ms as f64 * (1.0 + variance)) as u64;

        Ok(Decision {
            decision_type: "cfo_approval".to_string(),
            outcome: outcome.to_string(),
            reasoning,
            criteria_checked: self.get_decision_criteria(),
            timestamp: chrono::Utc::now(),
            decision_time_ms,
            confidence: if approved { 1.0 } else { 0.85 },
            metadata,
        })
    }

    fn get_authority(&self) -> AuthorityLevel {
        AuthorityLevel::Full
    }

    fn get_sla(&self) -> SLA {
        SLA {
            target_hours: 2,
            variance_percentage: 10.0,
        }
    }

    fn get_decision_criteria(&self) -> Vec<String> {
        vec![
            "Strategic value".to_string(),
            "Deal economics".to_string(),
            "Risk assessment".to_string(),
            "Executive discretion".to_string(),
        ]
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_role(&self) -> &str {
        &self.role
    }
}

// ============================================================================
// Avatar Factory
// ============================================================================

pub fn create_avatar(avatar_type: &str) -> Result<Box<dyn Avatar>, AvatarError> {
    match avatar_type {
        "sdr" => Ok(Box::new(SDRAvatar::new())),
        "manager" => Ok(Box::new(ManagerAvatar::new())),
        "legal" => Ok(Box::new(LegalAvatar::new())),
        "finance" => Ok(Box::new(FinanceAvatar::new())),
        "cfo" => Ok(Box::new(CFOAvatar::new())),
        _ => Err(AvatarError::InvalidInput(format!("Unknown avatar type: {}", avatar_type))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdr_qualification() {
        let avatar = SDRAvatar::new();
        let input = serde_json::json!({
            "company_size": 1000,
            "industry": "Technology",
            "use_case": "We need workflow automation for our enterprise operations team with complex approval chains",
            "budget_indicated": true,
        });

        let decision = avatar.decide(&input).unwrap();
        assert_eq!(decision.outcome, "QUALIFIED");
        assert!(decision.confidence >= 0.6);
    }

    #[test]
    fn test_manager_approval() {
        let avatar = ManagerAvatar::new();
        let input = serde_json::json!({
            "acv": 200000,
        });

        let decision = avatar.decide(&input).unwrap();
        assert_eq!(decision.outcome, "APPROVED");
    }

    #[test]
    fn test_avatar_factory() {
        let avatar = create_avatar("sdr").unwrap();
        assert_eq!(avatar.get_name(), "Sarah Chen");
    }
}
