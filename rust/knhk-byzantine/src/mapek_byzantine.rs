//! Byzantine-Resistant MAPE-K Integration
//!
//! Integrates Byzantine consensus with MAPE-K autonomic loops for distributed
//! workflow decision-making.

use crate::{
    errors::{ByzantineError, Result},
    network::ByzantineNetwork,
    protocols::{
        hotstuff::HotStuffConsensus,
        pbft::PBFTConsensus,
        Consensus,
    },
    NodeId, WorkflowDecision,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// MAPE-K analysis recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recommendation {
    pub workflow_id: String,
    pub issue: String,
    pub action: WorkflowDecision,
    pub confidence: f64,
    pub timestamp: u64,
}

/// Consensus protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusProtocol {
    PBFT,
    HotStuff,
}

/// Byzantine-resistant MAPE-K implementation
pub struct ByzantineMAPEK {
    #[allow(dead_code)]
    node_id: NodeId,
    #[allow(dead_code)]
    nodes: Vec<NodeId>,
    protocol: ConsensusProtocol,
    network: Arc<ByzantineNetwork>,
    pbft: Option<Arc<PBFTConsensus>>,
    hotstuff: Option<Arc<HotStuffConsensus>>,

    // MAPE-K state
    analysis_results: Arc<RwLock<HashMap<String, Vec<Recommendation>>>>,
    executed_decisions: Arc<RwLock<Vec<WorkflowDecision>>>,
}

impl ByzantineMAPEK {
    /// Create new Byzantine MAPE-K instance with PBFT
    pub fn new_pbft(
        node_id: NodeId,
        nodes: Vec<NodeId>,
        timeout: Duration,
        network: Arc<ByzantineNetwork>,
    ) -> Self {
        let pbft = Arc::new(PBFTConsensus::new(
            node_id,
            nodes.clone(),
            timeout,
            network.clone(),
        ));

        info!("Byzantine MAPE-K initialized with PBFT: node={}", node_id);

        Self {
            node_id,
            nodes,
            protocol: ConsensusProtocol::PBFT,
            network,
            pbft: Some(pbft),
            hotstuff: None,
            analysis_results: Arc::new(RwLock::new(HashMap::new())),
            executed_decisions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create new Byzantine MAPE-K instance with HotStuff
    pub fn new_hotstuff(
        node_id: NodeId,
        nodes: Vec<NodeId>,
        timeout: Duration,
        network: Arc<ByzantineNetwork>,
    ) -> Self {
        let hotstuff = Arc::new(HotStuffConsensus::new(
            node_id,
            nodes.clone(),
            timeout,
            network.clone(),
        ));

        info!("Byzantine MAPE-K initialized with HotStuff: node={}", node_id);

        Self {
            node_id,
            nodes,
            protocol: ConsensusProtocol::HotStuff,
            network,
            pbft: None,
            hotstuff: Some(hotstuff),
            analysis_results: Arc::new(RwLock::new(HashMap::new())),
            executed_decisions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Monitor: Collect workflow metrics (simplified)
    pub async fn monitor(&self, workflow_id: &str) -> Result<WorkflowMetrics> {
        debug!("Monitoring workflow {}", workflow_id);

        // Simplified monitoring - in production, collect real metrics
        Ok(WorkflowMetrics {
            workflow_id: workflow_id.to_string(),
            tasks_completed: 10,
            tasks_failed: 2,
            avg_duration_ms: 150,
            resource_usage: 0.75,
        })
    }

    /// Analyze: Analyze workflow and generate recommendations
    pub async fn analyze(&self, workflow_id: &str) -> Result<Vec<Recommendation>> {
        info!("Analyzing workflow {}", workflow_id);

        let metrics = self.monitor(workflow_id).await?;

        let mut recommendations = Vec::new();

        // Analyze failure rate
        let failure_rate = metrics.tasks_failed as f64 / metrics.tasks_completed as f64;
        if failure_rate > 0.1 {
            recommendations.push(Recommendation {
                workflow_id: workflow_id.to_string(),
                issue: "High failure rate detected".to_string(),
                action: WorkflowDecision {
                    workflow_id: workflow_id.to_string(),
                    action: crate::DecisionAction::Reconfigure {
                        new_config: "increase-retries".to_string(),
                    },
                    timestamp: current_timestamp(),
                },
                confidence: 0.85,
                timestamp: current_timestamp(),
            });
        }

        // Analyze resource usage
        if metrics.resource_usage > 0.9 {
            recommendations.push(Recommendation {
                workflow_id: workflow_id.to_string(),
                issue: "High resource usage".to_string(),
                action: WorkflowDecision {
                    workflow_id: workflow_id.to_string(),
                    action: crate::DecisionAction::Delay {
                        until: current_timestamp() + 60,
                    },
                    timestamp: current_timestamp(),
                },
                confidence: 0.90,
                timestamp: current_timestamp(),
            });
        }

        // Store analysis results
        self.analysis_results
            .write()
            .await
            .insert(workflow_id.to_string(), recommendations.clone());

        Ok(recommendations)
    }

    /// Analyze with Byzantine consensus
    pub async fn analyze_with_consensus(&self, workflow_id: &str) -> Result<Vec<Recommendation>> {
        let recommendations = self.analyze(workflow_id).await?;

        if recommendations.is_empty() {
            return Ok(vec![]);
        }

        // Convert recommendations to workflow decisions
        let decisions: Vec<_> = recommendations
            .iter()
            .map(|r| r.action.clone())
            .collect();

        // Run consensus on decisions
        let consensus = self.reach_consensus(decisions).await?;

        // Return recommendations for decisions that reached consensus
        let consensus_decisions: std::collections::HashSet<_> = consensus
            .block
            .decisions
            .iter()
            .map(|d| d.workflow_id.clone())
            .collect();

        let agreed_recommendations: Vec<_> = recommendations
            .into_iter()
            .filter(|r| consensus_decisions.contains(&r.workflow_id))
            .collect();

        info!(
            "Consensus reached on {}/{} recommendations",
            agreed_recommendations.len(),
            consensus_decisions.len()
        );

        Ok(agreed_recommendations)
    }

    /// Plan: Create execution plan (simplified)
    pub async fn plan(&self, recommendations: &[Recommendation]) -> Result<ExecutionPlan> {
        debug!("Planning execution for {} recommendations", recommendations.len());

        let steps = recommendations
            .iter()
            .map(|r| ExecutionStep {
                workflow_id: r.workflow_id.clone(),
                decision: r.action.clone(),
                order: 0,
            })
            .collect();

        Ok(ExecutionPlan { steps })
    }

    /// Execute: Execute consensus decision
    pub async fn execute_consensus_decision(&self, decision: &WorkflowDecision) -> Result<()> {
        info!("Executing decision for workflow {}", decision.workflow_id);

        // Simulate execution
        match &decision.action {
            crate::DecisionAction::Execute => {
                debug!("Executing workflow {}", decision.workflow_id);
            }
            crate::DecisionAction::Reject => {
                debug!("Rejecting workflow {}", decision.workflow_id);
            }
            crate::DecisionAction::Delay { until } => {
                debug!("Delaying workflow {} until {}", decision.workflow_id, until);
            }
            crate::DecisionAction::Reconfigure { new_config } => {
                debug!("Reconfiguring workflow {} with {}", decision.workflow_id, new_config);
            }
        }

        // Store executed decision
        self.executed_decisions.write().await.push(decision.clone());

        Ok(())
    }

    /// Knowledge: Update knowledge base (simplified)
    pub async fn update_knowledge(&self, _decision: &WorkflowDecision, _outcome: bool) {
        debug!("Updating knowledge base");
        // In production, update ML models, statistics, etc.
    }

    /// Reach consensus on workflow decisions
    async fn reach_consensus(&self, decisions: Vec<WorkflowDecision>) -> Result<Consensus> {
        match self.protocol {
            ConsensusProtocol::PBFT => {
                let pbft = self.pbft.as_ref().ok_or_else(|| {
                    ByzantineError::Configuration("PBFT not initialized".to_string())
                })?;
                pbft.propose(decisions).await
            }
            ConsensusProtocol::HotStuff => {
                let hotstuff = self.hotstuff.as_ref().ok_or_else(|| {
                    ByzantineError::Configuration("HotStuff not initialized".to_string())
                })?;
                hotstuff.propose(decisions).await
            }
        }
    }

    /// Detect Byzantine nodes based on conflicting decisions
    pub async fn detect_byzantine_nodes(&self) -> Vec<NodeId> {
        debug!("Detecting Byzantine nodes");

        let byzantine = self.network.byzantine_nodes();

        if !byzantine.is_empty() {
            warn!("Detected {} Byzantine nodes", byzantine.len());
        }

        byzantine
    }

    /// Get executed decisions
    pub async fn executed_decisions(&self) -> Vec<WorkflowDecision> {
        self.executed_decisions.read().await.clone()
    }

    /// Get analysis results
    pub async fn analysis_results(&self, workflow_id: &str) -> Option<Vec<Recommendation>> {
        self.analysis_results.read().await.get(workflow_id).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub workflow_id: String,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub avg_duration_ms: u64,
    pub resource_usage: f64,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub workflow_id: String,
    pub decision: WorkflowDecision,
    pub order: usize,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_byzantine_mapek_creation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let mapek = ByzantineMAPEK::new_pbft(
            NodeId(0),
            nodes,
            Duration::from_secs(5),
            network,
        );

        assert_eq!(mapek.protocol, ConsensusProtocol::PBFT);
    }

    #[tokio::test]
    async fn test_monitor() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let mapek = ByzantineMAPEK::new_pbft(
            NodeId(0),
            nodes,
            Duration::from_secs(5),
            network,
        );

        let metrics = mapek.monitor("workflow-1").await.unwrap();
        assert_eq!(metrics.workflow_id, "workflow-1");
    }

    #[tokio::test]
    async fn test_analyze() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let mapek = ByzantineMAPEK::new_pbft(
            NodeId(0),
            nodes,
            Duration::from_secs(5),
            network,
        );

        let recommendations = mapek.analyze("workflow-1").await.unwrap();
        assert!(!recommendations.is_empty());
    }
}
