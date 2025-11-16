//! Usage-based billing and cost management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MarketplaceError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub updated_at: DateTime<Utc>,
}

impl UsageMetrics {
    pub fn new(name: String, unit: String) -> Self {
        Self {
            name,
            value: 0.0,
            unit,
            updated_at: Utc::now(),
        }
    }

    pub fn add(&mut self, amount: f64) {
        self.value += amount;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub name: String,
    pub base_cost: f64,
    pub per_execution: f64,
    pub per_compute_gb_hour: f64,
    pub per_data_transfer_gb: f64,
    pub included_executions: u32,
    pub included_compute: f64,
    pub included_transfer: f64,
}

impl PricingTier {
    pub fn community() -> Self {
        Self {
            name: "Community".to_string(),
            base_cost: 0.0,
            per_execution: 0.00,
            per_compute_gb_hour: 0.00,
            per_data_transfer_gb: 0.00,
            included_executions: 1_000,
            included_compute: 10.0,
            included_transfer: 5.0,
        }
    }

    pub fn professional() -> Self {
        Self {
            name: "Professional".to_string(),
            base_cost: 99.0,
            per_execution: 0.01,
            per_compute_gb_hour: 0.05,
            per_data_transfer_gb: 0.10,
            included_executions: 100_000,
            included_compute: 500.0,
            included_transfer: 100.0,
        }
    }
}

pub struct BillingEngine {
    tiers: std::collections::HashMap<String, PricingTier>,
}

impl BillingEngine {
    pub fn new() -> Self {
        let mut tiers = std::collections::HashMap::new();
        tiers.insert("Community".to_string(), PricingTier::community());
        tiers.insert("Professional".to_string(), PricingTier::professional());

        Self { tiers }
    }

    pub fn estimate_cost(&self, tier_name: &str, executions: u32) -> Result<f64> {
        let tier = self
            .tiers
            .get(tier_name)
            .ok_or_else(|| MarketplaceError::Billing("Tier not found".to_string()))?;

        let excess_executions = executions.saturating_sub(tier.included_executions);
        Ok(tier.base_cost + (excess_executions as f64 * tier.per_execution))
    }
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyBill {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost: f64,
    pub tier_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_tiers() {
        let community = PricingTier::community();
        assert_eq!(community.base_cost, 0.0);

        let professional = PricingTier::professional();
        assert_eq!(professional.base_cost, 99.0);
    }

    #[test]
    fn test_cost_estimation() {
        let engine = BillingEngine::new();
        let cost = engine.estimate_cost("Community", 500).unwrap();
        assert_eq!(cost, 0.0);
    }
}
