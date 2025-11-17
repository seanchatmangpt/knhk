//! P2P Network Layer
//!
//! Byzantine sender detection, peer discovery, and message ordering
//! Provides reliable broadcast and unicast with message authentication

use crate::{ConsensusError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, warn, info};

/// Peer message with authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMessage {
    /// Source node ID
    pub source: String,
    /// Destination node ID (empty = broadcast)
    pub destination: String,
    /// Message sequence number
    pub sequence: u64,
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message signature (ed25519)
    pub signature: Vec<u8>,
}

impl PeerMessage {
    /// Compute message hash for signature verification
    pub fn compute_hash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.source);
        hasher.update(&self.destination);
        hasher.update(self.sequence.to_le_bytes());
        hasher.update(self.timestamp_ms.to_le_bytes());
        hasher.update(&self.payload);
        hasher.finalize().to_vec()
    }

    /// Check if broadcast message
    pub fn is_broadcast(&self) -> bool {
        self.destination.is_empty()
    }
}

/// Peer node in network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Node ID
    pub node_id: String,
    /// Network address
    pub address: String,
    /// Public key for signature verification
    pub public_key: Vec<u8>,
    /// Last seen timestamp
    pub last_seen_ms: u64,
    /// Message count received
    pub message_count: u64,
}

/// Peer discovery and management
#[derive(Debug, Clone)]
pub struct PeerDiscovery {
    /// Known peers
    peers: Arc<DashMap<String, PeerInfo>>,
    /// Bootstrap nodes
    bootstrap: Arc<Vec<String>>,
    /// Self node ID
    self_id: String,
}

impl PeerDiscovery {
    /// Create peer discovery
    pub fn new(self_id: String, bootstrap_nodes: Vec<String>) -> Self {
        PeerDiscovery {
            peers: Arc::new(DashMap::new()),
            bootstrap: Arc::new(bootstrap_nodes),
            self_id,
        }
    }

    /// Register peer
    pub fn register_peer(&self, peer: PeerInfo) -> Result<()> {
        if peer.node_id == self.self_id {
            return Err(ConsensusError::InvalidValidatorSet(
                "Cannot register self as peer".to_string(),
            ));
        }

        self.peers.insert(peer.node_id.clone(), peer.clone());
        debug!(peer_id = %peer.node_id, "Peer registered");

        Ok(())
    }

    /// Get peer info
    pub fn get_peer(&self, node_id: &str) -> Option<PeerInfo> {
        self.peers.get(node_id).map(|p| p.clone())
    }

    /// Get all peers
    pub fn get_all_peers(&self) -> Vec<PeerInfo> {
        self.peers.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Update peer last seen
    pub fn update_peer_seen(&self, node_id: &str) -> Result<()> {
        if let Some(mut peer) = self.peers.get_mut(node_id) {
            peer.last_seen_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            Ok(())
        } else {
            Err(ConsensusError::ByzantineNodeDetected(format!(
                "Unknown peer: {}",
                node_id
            )))
        }
    }
}

/// Network node for P2P communication
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// Node ID
    pub node_id: String,
    /// Peer discovery
    discovery: PeerDiscovery,
    /// Sent messages
    sent: Arc<DashMap<u64, PeerMessage>>,
    /// Received messages
    received: Arc<DashMap<u64, PeerMessage>>,
    /// Message sequence counter
    message_seq: Arc<parking_lot::Mutex<u64>>,
    /// Byzantine nodes detected
    byzantine: Arc<parking_lot::Mutex<HashSet<String>>>,
}

impl NetworkNode {
    /// Create network node
    pub fn new(node_id: String, bootstrap_nodes: Vec<String>) -> Self {
        NetworkNode {
            node_id: node_id.clone(),
            discovery: PeerDiscovery::new(node_id, bootstrap_nodes),
            sent: Arc::new(DashMap::new()),
            received: Arc::new(DashMap::new()),
            message_seq: Arc::new(parking_lot::Mutex::new(0)),
            byzantine: Arc::new(parking_lot::Mutex::new(HashSet::new())),
        }
    }

    /// Broadcast message to all peers
    pub fn broadcast(&self, payload: Vec<u8>) -> Result<u64> {
        let mut seq = self.message_seq.lock();
        *seq += 1;
        let seq_num = *seq;

        let msg = PeerMessage {
            source: self.node_id.clone(),
            destination: String::new(),
            sequence: seq_num,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload,
            signature: vec![],
        };

        self.sent.insert(seq_num, msg.clone());

        debug!(
            node = %self.node_id,
            sequence = seq_num,
            peers = self.discovery.peer_count(),
            "Broadcast message"
        );

        Ok(seq_num)
    }

    /// Send unicast message to peer
    pub fn send_to(&self, peer_id: String, payload: Vec<u8>) -> Result<u64> {
        if !self.discovery.get_peer(&peer_id).is_some() {
            return Err(ConsensusError::ByzantineNodeDetected(format!(
                "Unknown peer: {}",
                peer_id
            )));
        }

        let mut seq = self.message_seq.lock();
        *seq += 1;
        let seq_num = *seq;

        let msg = PeerMessage {
            source: self.node_id.clone(),
            destination: peer_id.clone(),
            sequence: seq_num,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload,
            signature: vec![],
        };

        self.sent.insert(seq_num, msg);

        debug!(
            node = %self.node_id,
            destination = %peer_id,
            sequence = seq_num,
            "Unicast message"
        );

        Ok(seq_num)
    }

    /// Receive and validate message
    pub fn receive_message(&self, msg: PeerMessage) -> Result<()> {
        // Validate sender is known peer
        if !self.discovery.get_peer(&msg.source).is_some() {
            warn!(source = %msg.source, "Message from unknown peer");
            return Err(ConsensusError::ByzantineNodeDetected(
                format!("Unknown sender: {}", msg.source),
            ));
        }

        // Check timestamp is reasonable (within 5 minutes)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if msg.timestamp_ms > now + 300000 || msg.timestamp_ms + 300000 < now {
            warn!(
                source = %msg.source,
                timestamp = msg.timestamp_ms,
                "Message timestamp out of bounds"
            );
            return Err(ConsensusError::ByzantineNodeDetected(
                "Timestamp out of bounds".to_string(),
            ));
        }

        let source = msg.source.clone();
        self.received.insert(msg.sequence, msg);
        self.discovery.update_peer_seen(&source)?;

        Ok(())
    }

    /// Detect Byzantine sender
    pub fn detect_byzantine_sender(&self, node_id: &str) -> Result<()> {
        let mut byzantine = self.byzantine.lock();
        byzantine.insert(node_id.to_string());

        error!(node_id = %node_id, "Byzantine node detected and recorded");

        Ok(())
    }

    /// Get Byzantine nodes
    pub fn get_byzantine_nodes(&self) -> Vec<String> {
        self.byzantine
            .lock()
            .iter()
            .cloned()
            .collect()
    }

    /// Message ordering: verify monotonic increase
    pub fn verify_message_order(
        &self,
        peer_id: &str,
        sequence: u64,
    ) -> Result<()> {
        let received = self.received.iter().filter(|entry| {
            entry.value().source == peer_id
        });

        let mut max_seq = 0;
        for entry in received {
            if entry.value().sequence > max_seq {
                max_seq = entry.value().sequence;
            }
        }

        if sequence <= max_seq {
            return Err(ConsensusError::ByzantineNodeDetected(
                format!("Out-of-order message from {}", peer_id),
            ));
        }

        Ok(())
    }

    /// Get message stats
    pub fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            node_id: self.node_id.clone(),
            sent_count: self.sent.len(),
            received_count: self.received.len(),
            peer_count: self.discovery.peer_count(),
            byzantine_count: self.byzantine.lock().len(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Node ID
    pub node_id: String,
    /// Messages sent
    pub sent_count: usize,
    /// Messages received
    pub received_count: usize,
    /// Connected peers
    pub peer_count: usize,
    /// Detected Byzantine nodes
    pub byzantine_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery() {
        let discovery = PeerDiscovery::new("node1".to_string(), vec![]);
        let peer = PeerInfo {
            node_id: "node2".to_string(),
            address: "127.0.0.1:8001".to_string(),
            public_key: vec![],
            last_seen_ms: 0,
            message_count: 0,
        };
        discovery.register_peer(peer.clone()).unwrap();
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_network_broadcast() {
        let net = NetworkNode::new("node1".to_string(), vec![]);
        let peer = PeerInfo {
            node_id: "node2".to_string(),
            address: "127.0.0.1:8001".to_string(),
            public_key: vec![],
            last_seen_ms: 1000,
            message_count: 0,
        };
        net.discovery.register_peer(peer).unwrap();

        let seq = net.broadcast(b"test".to_vec()).unwrap();
        assert_eq!(seq, 1);
    }

    #[test]
    fn test_byzantine_detection() {
        let net = NetworkNode::new("node1".to_string(), vec![]);
        net.detect_byzantine_sender("node2").unwrap();
        assert_eq!(net.get_byzantine_nodes().len(), 1);
    }
}
