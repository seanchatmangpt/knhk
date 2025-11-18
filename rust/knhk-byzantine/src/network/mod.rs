//! Byzantine network simulation
//!
//! Provides network layer for Byzantine consensus with:
//! - Message broadcast and unicast
//! - Byzantine failure injection (message loss, delay, corruption)
//! - Node state tracking
//! - Message buffering and delivery

pub mod broadcast;

use crate::{errors::Result, NodeId};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Network message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub from: NodeId,
    pub to: Option<NodeId>, // None for broadcast
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

/// Node state in the network
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeState {
    Active,
    Suspected,
    Byzantine,
    Offline,
}

/// Byzantine network simulator
pub struct ByzantineNetwork {
    nodes: Vec<NodeId>,
    node_states: Arc<DashMap<NodeId, NodeState>>,
    message_queues: Arc<DashMap<NodeId, Arc<RwLock<VecDeque<Envelope>>>>>,
    byzantine_config: Arc<RwLock<ByzantineConfig>>,
}

/// Configuration for Byzantine behavior simulation
#[derive(Debug, Clone)]
pub struct ByzantineConfig {
    /// Probability of message loss (0.0 - 1.0)
    pub message_loss_rate: f64,
    /// Message delay range
    pub message_delay: Option<Duration>,
    /// Probability of message corruption
    pub corruption_rate: f64,
    /// Maximum messages in queue per node
    pub max_queue_size: usize,
}

impl Default for ByzantineConfig {
    fn default() -> Self {
        Self {
            message_loss_rate: 0.0,
            message_delay: None,
            corruption_rate: 0.0,
            max_queue_size: 10000,
        }
    }
}

impl ByzantineNetwork {
    /// Create new Byzantine network
    pub fn new(nodes: Vec<NodeId>) -> Self {
        let node_states = Arc::new(DashMap::new());
        let message_queues = Arc::new(DashMap::new());

        for &node in &nodes {
            node_states.insert(node, NodeState::Active);
            message_queues.insert(node, Arc::new(RwLock::new(VecDeque::new())));
        }

        Self {
            nodes,
            node_states,
            message_queues,
            byzantine_config: Arc::new(RwLock::new(ByzantineConfig::default())),
        }
    }

    /// Broadcast message to all nodes
    pub async fn broadcast(&self, payload: Vec<u8>) -> Result<()> {
        debug!("Broadcasting message to {} nodes", self.nodes.len());

        for &node in &self.nodes {
            if self.is_active(node) {
                self.send_to(node, payload.clone()).await?;
            }
        }

        Ok(())
    }

    /// Send message to specific node
    pub async fn send_to(&self, to: NodeId, payload: Vec<u8>) -> Result<()> {
        let config = self.byzantine_config.read().await;

        // Simulate message loss
        if config.message_loss_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < config.message_loss_rate {
                warn!("Message lost to node {}", to);
                return Ok(());
            }
        }

        // Simulate message delay
        if let Some(delay) = config.message_delay {
            tokio::time::sleep(delay).await;
        }

        // Simulate message corruption
        let payload = if config.corruption_rate > 0.0 {
            use rand::Rng;
            if rand::thread_rng().gen::<f64>() < config.corruption_rate {
                warn!("Message corrupted to node {}", to);
                vec![0u8; payload.len()] // Corrupt message
            } else {
                payload
            }
        } else {
            payload
        };

        drop(config);

        // Enqueue message
        if let Some(queue_lock) = self.message_queues.get(&to) {
            let mut queue = queue_lock.write().await;

            let config = self.byzantine_config.read().await;
            if queue.len() >= config.max_queue_size {
                warn!("Message queue full for node {}", to);
                return Ok(());
            }
            drop(config);

            queue.push_back(Envelope {
                from: NodeId(0), // Simplified sender
                to: Some(to),
                payload,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            });
            debug!("Message queued for node {}", to);
        }

        Ok(())
    }

    /// Receive message for a node
    pub async fn receive(&self, node: NodeId) -> Option<Envelope> {
        if let Some(queue_lock) = self.message_queues.get(&node) {
            let mut queue = queue_lock.write().await;
            let envelope = queue.pop_front();
            if envelope.is_some() {
                debug!("Message received by node {}", node);
            }
            envelope
        } else {
            None
        }
    }

    /// Mark node as Byzantine/malicious
    pub async fn handle_byzantine_node(&self, node_id: NodeId) {
        warn!("Marking node {} as Byzantine", node_id);
        self.node_states.insert(node_id, NodeState::Byzantine);
    }

    /// Mark node as suspected
    pub fn suspect_node(&self, node_id: NodeId) {
        debug!("Suspecting node {}", node_id);
        self.node_states.insert(node_id, NodeState::Suspected);
    }

    /// Mark node as offline
    pub fn mark_offline(&self, node_id: NodeId) {
        debug!("Marking node {} offline", node_id);
        self.node_states.insert(node_id, NodeState::Offline);
    }

    /// Mark node as active
    pub fn mark_active(&self, node_id: NodeId) {
        debug!("Marking node {} active", node_id);
        self.node_states.insert(node_id, NodeState::Active);
    }

    /// Check if node is active
    pub fn is_active(&self, node_id: NodeId) -> bool {
        self.node_states
            .get(&node_id)
            .map(|s| *s == NodeState::Active)
            .unwrap_or(false)
    }

    /// Get node state
    pub fn get_state(&self, node_id: NodeId) -> Option<NodeState> {
        self.node_states.get(&node_id).map(|s| s.clone())
    }

    /// Get all Byzantine nodes
    pub fn byzantine_nodes(&self) -> Vec<NodeId> {
        self.node_states
            .iter()
            .filter(|entry| *entry.value() == NodeState::Byzantine)
            .map(|entry| *entry.key())
            .collect()
    }

    /// Get all suspected nodes
    pub fn suspected_nodes(&self) -> Vec<NodeId> {
        self.node_states
            .iter()
            .filter(|entry| *entry.value() == NodeState::Suspected)
            .map(|entry| *entry.key())
            .collect()
    }

    /// Get active nodes count
    pub fn active_count(&self) -> usize {
        self.node_states
            .iter()
            .filter(|entry| *entry.value() == NodeState::Active)
            .count()
    }

    /// Set Byzantine configuration
    pub async fn set_config(&self, config: ByzantineConfig) {
        *self.byzantine_config.write().await = config;
    }

    /// Get Byzantine configuration
    pub async fn get_config(&self) -> ByzantineConfig {
        self.byzantine_config.read().await.clone()
    }

    /// Get pending message count for a node
    pub async fn pending_count(&self, node_id: NodeId) -> usize {
        if let Some(queue_lock) = self.message_queues.get(&node_id) {
            queue_lock.read().await.len()
        } else {
            0
        }
    }

    /// Clear all messages for a node
    pub async fn clear_queue(&self, node_id: NodeId) {
        if let Some(queue_lock) = self.message_queues.get(&node_id) {
            queue_lock.write().await.clear();
            debug!("Cleared message queue for node {}", node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        let network = ByzantineNetwork::new(nodes.clone());

        assert_eq!(network.active_count(), 3);
        for &node in &nodes {
            assert_eq!(network.get_state(node), Some(NodeState::Active));
        }
    }

    #[tokio::test]
    async fn test_broadcast() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        let network = ByzantineNetwork::new(nodes.clone());

        let msg = vec![1, 2, 3, 4];
        network.broadcast(msg.clone()).await.unwrap();

        for &node in &nodes {
            assert_eq!(network.pending_count(node).await, 1);
        }
    }

    #[tokio::test]
    async fn test_byzantine_node_marking() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        let network = ByzantineNetwork::new(nodes);

        network.handle_byzantine_node(NodeId(1)).await;

        assert_eq!(network.get_state(NodeId(1)), Some(NodeState::Byzantine));
        assert_eq!(network.byzantine_nodes().len(), 1);
        assert_eq!(network.active_count(), 2);
    }

    #[tokio::test]
    async fn test_message_loss_simulation() {
        let nodes = vec![NodeId(0), NodeId(1)];
        let network = ByzantineNetwork::new(nodes);

        // Set 100% message loss
        network
            .set_config(ByzantineConfig {
                message_loss_rate: 1.0,
                ..Default::default()
            })
            .await;

        network.broadcast(vec![1, 2, 3]).await.unwrap();

        // No messages should be delivered
        assert_eq!(network.pending_count(NodeId(0)).await, 0);
        assert_eq!(network.pending_count(NodeId(1)).await, 0);
    }
}
