//! Reliable broadcast protocol for Byzantine networks
//!
//! Implements reliable broadcast primitives:
//! - Best-effort broadcast
//! - Reliable broadcast (RB)
//! - Byzantine reliable broadcast (BRB)

use crate::{
    errors::{ByzantineError, Result},
    network::{ByzantineNetwork, Envelope},
    NodeId,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Broadcast message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub id: u64,
    pub sender: NodeId,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

/// Byzantine Reliable Broadcast protocol
pub struct ByzantineReliableBroadcast {
    node_id: NodeId,
    nodes: Vec<NodeId>,
    f: usize,
    network: Arc<ByzantineNetwork>,

    // Tracking state
    received: Arc<DashMap<u64, BroadcastMessage>>,
    echoed: Arc<DashMap<u64, HashSet<NodeId>>>,
    ready: Arc<DashMap<u64, HashSet<NodeId>>>,
    delivered: Arc<RwLock<HashSet<u64>>>,

    message_counter: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum BRBMessage {
    Send(BroadcastMessage),
    Echo {
        msg_id: u64,
        sender: NodeId,
        node_id: NodeId,
    },
    Ready {
        msg_id: u64,
        sender: NodeId,
        node_id: NodeId,
    },
}

impl ByzantineReliableBroadcast {
    /// Create new BRB instance
    pub fn new(
        node_id: NodeId,
        nodes: Vec<NodeId>,
        network: Arc<ByzantineNetwork>,
    ) -> Self {
        let n = nodes.len();
        let f = (n - 1) / 3;

        info!(
            "BRB initialized: node={}, n={}, f={}",
            node_id, n, f
        );

        Self {
            node_id,
            nodes,
            f,
            network,
            received: Arc::new(DashMap::new()),
            echoed: Arc::new(DashMap::new()),
            ready: Arc::new(DashMap::new()),
            delivered: Arc::new(RwLock::new(HashSet::new())),
            message_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Broadcast message with Byzantine reliability
    pub async fn broadcast(&self, payload: Vec<u8>) -> Result<u64> {
        let mut counter = self.message_counter.write().await;
        *counter += 1;
        let msg_id = *counter;
        drop(counter);

        let msg = BroadcastMessage {
            id: msg_id,
            sender: self.node_id,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        info!("Broadcasting message {}", msg_id);

        // Send to all nodes
        let brb_msg = BRBMessage::Send(msg.clone());
        self.network
            .broadcast(bincode::serialize(&brb_msg)?)
            .await?;

        // Store locally
        self.received.insert(msg_id, msg);

        Ok(msg_id)
    }

    /// Process incoming BRB message
    pub async fn handle_message(&self, envelope: Envelope) -> Result<()> {
        let brb_msg: BRBMessage = bincode::deserialize(&envelope.payload)?;

        match brb_msg {
            BRBMessage::Send(msg) => self.handle_send(msg).await,
            BRBMessage::Echo { msg_id, sender, node_id } => {
                self.handle_echo(msg_id, sender, node_id).await
            }
            BRBMessage::Ready { msg_id, sender, node_id } => {
                self.handle_ready(msg_id, sender, node_id).await
            }
        }
    }

    async fn handle_send(&self, msg: BroadcastMessage) -> Result<()> {
        let msg_id = msg.id;

        if self.received.contains_key(&msg_id) {
            return Ok(()); // Already received
        }

        debug!("Received message {} from {}", msg_id, msg.sender);
        self.received.insert(msg_id, msg.clone());

        // Send ECHO
        let echo = BRBMessage::Echo {
            msg_id,
            sender: msg.sender,
            node_id: self.node_id,
        };

        self.network
            .broadcast(bincode::serialize(&echo)?)
            .await?;

        Ok(())
    }

    async fn handle_echo(&self, msg_id: u64, sender: NodeId, node_id: NodeId) -> Result<()> {
        // Track echo
        self.echoed
            .entry(msg_id)
            .or_default()
            .insert(node_id);

        let echo_count = self.echoed.get(&msg_id).map(|s| s.len()).unwrap_or(0);

        // If received (n+f)/2 + 1 ECHOs, send READY
        let threshold = (self.nodes.len() + self.f) / 2 + 1;
        if echo_count >= threshold {
            let ready_sent = self.ready.get(&msg_id).map(|s| s.contains(&self.node_id)).unwrap_or(false);

            if !ready_sent {
                debug!("Sending READY for message {}", msg_id);

                let ready = BRBMessage::Ready {
                    msg_id,
                    sender,
                    node_id: self.node_id,
                };

                self.network
                    .broadcast(bincode::serialize(&ready)?)
                    .await?;

                self.ready
                    .entry(msg_id)
                    .or_default()
                    .insert(self.node_id);
            }
        }

        Ok(())
    }

    async fn handle_ready(&self, msg_id: u64, sender: NodeId, node_id: NodeId) -> Result<()> {
        // Track ready
        self.ready
            .entry(msg_id)
            .or_default()
            .insert(node_id);

        let ready_count = self.ready.get(&msg_id).map(|s| s.len()).unwrap_or(0);

        // If received f+1 READYs and haven't sent READY yet, send READY
        if ready_count > self.f {
            let ready_sent = self.ready.get(&msg_id).map(|s| s.contains(&self.node_id)).unwrap_or(false);

            if !ready_sent {
                debug!("Amplifying READY for message {}", msg_id);

                let ready = BRBMessage::Ready {
                    msg_id,
                    sender,
                    node_id: self.node_id,
                };

                self.network
                    .broadcast(bincode::serialize(&ready)?)
                    .await?;

                self.ready
                    .entry(msg_id)
                    .or_default()
                    .insert(self.node_id);
            }
        }

        // If received 2f+1 READYs, deliver message
        if ready_count > 2 * self.f {
            let mut delivered = self.delivered.write().await;

            if !delivered.contains(&msg_id) {
                info!("Delivering message {}", msg_id);
                delivered.insert(msg_id);

                // Message is now reliably delivered
            }
        }

        Ok(())
    }

    /// Check if message was delivered
    pub async fn is_delivered(&self, msg_id: u64) -> bool {
        self.delivered.read().await.contains(&msg_id)
    }

    /// Get delivered messages
    pub async fn delivered_messages(&self) -> Vec<BroadcastMessage> {
        let delivered = self.delivered.read().await;
        self.received
            .iter()
            .filter(|entry| delivered.contains(entry.key()))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Wait for message delivery
    pub async fn wait_for_delivery(&self, msg_id: u64, timeout: Duration) -> Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            if tokio::time::Instant::now() > deadline {
                return Err(ByzantineError::ConsensusTimeout {
                    timeout_ms: timeout.as_millis() as u64,
                });
            }

            if self.is_delivered(msg_id).await {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_brb_creation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let brb = ByzantineReliableBroadcast::new(NodeId(0), nodes, network);

        assert_eq!(brb.f, 1);
    }

    #[tokio::test]
    async fn test_brb_broadcast() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let brb = ByzantineReliableBroadcast::new(NodeId(0), nodes, network);

        let msg_id = brb.broadcast(vec![1, 2, 3]).await.unwrap();
        assert_eq!(msg_id, 1);
    }
}
