//! Core Gossip Protocol Implementation
//!
//! Implements epidemic dissemination with Byzantine-robust voting for massive swarms.

use super::config::GossipConfig;
use super::merkle::{generate_merkle_proof, StateProof};
use super::state::{StateValue, VersionedState};
use super::topology::PeerSampler;
use super::{AgentId, Result, RoundNumber};
use blake3::Hash as Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

/// Gossip message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Push: Send my state to peer
    Push {
        state: VersionedState,
        proof: Option<StateProof>,
    },
    /// Pull: Request peer's state
    Pull { version: u64 },
    /// Push-Pull: Exchange states
    PushPull {
        my_state: VersionedState,
        proof: Option<StateProof>,
    },
    /// Ack: Acknowledge message receipt
    Ack { version: u64 },
}

/// Gossip protocol implementation
pub struct GossipProtocol {
    /// Configuration
    config: GossipConfig,
    /// My current state
    my_state: Arc<RwLock<VersionedState>>,
    /// Peer sampler
    peer_sampler: Arc<RwLock<PeerSampler>>,
    /// Received states from peers (for majority voting)
    peer_states: Arc<RwLock<HashMap<AgentId, VersionedState>>>,
    /// Current gossip round
    current_round: Arc<RwLock<RoundNumber>>,
    /// Round start time
    round_start: Arc<RwLock<Option<Instant>>>,
}

impl GossipProtocol {
    /// Create new gossip protocol instance
    pub fn new(config: GossipConfig, initial_state: VersionedState) -> Self {
        let peer_sampler = PeerSampler::new(config.agent_id);

        Self {
            config,
            my_state: Arc::new(RwLock::new(initial_state)),
            peer_sampler: Arc::new(RwLock::new(peer_sampler)),
            peer_states: Arc::new(RwLock::new(HashMap::new())),
            current_round: Arc::new(RwLock::new(0)),
            round_start: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize peer list
    pub async fn init_peers(&self, peer_ids: Vec<AgentId>) {
        let mut sampler = self.peer_sampler.write().await;
        for peer_id in peer_ids {
            if peer_id != self.config.agent_id {
                sampler.add_peer(peer_id, 10); // Default 10ms latency
            }
        }
    }

    /// Execute one gossip round (push-pull epidemic dissemination)
    pub async fn execute_round(&self) -> Result<GossipRoundStats> {
        let round_start = Instant::now();
        *self.round_start.write().await = Some(round_start);

        let mut round = *self.current_round.read().await;
        round += 1;
        *self.current_round.write().await = round;

        info!(round = round, "Starting gossip round");

        // 1. Select k random peers
        let peers = {
            let mut sampler = self.peer_sampler.write().await;
            if self.config.topology_optimization {
                sampler.sample_optimized(self.config.peer_sample_size)
            } else {
                sampler.sample_random(self.config.peer_sample_size)
            }
        };

        if peers.is_empty() {
            warn!("No peers available for gossip");
            return Ok(GossipRoundStats {
                round,
                messages_sent: 0,
                messages_received: 0,
                states_merged: 0,
                byzantine_detected: 0,
                duration: round_start.elapsed(),
            });
        }

        trace!(round = round, peer_count = peers.len(), "Selected peers");

        // 2. Push my state to peers (simulated - would use network in real impl)
        let my_state = self.my_state.read().await.clone();
        let messages_sent = peers.len();

        // 3. Pull peer states (simulated - would receive from network)
        // In real implementation, this would be async network calls
        let mut messages_received = 0;
        let mut states_merged = 0;
        let mut byzantine_detected = 0;

        // Simulate receiving states from peers (placeholder for real network)
        // In production: await tokio::spawn for each peer communication
        for peer_id in &peers {
            // This would be: let peer_state = receive_from_peer(*peer_id).await?;
            // For now, just record that we would receive from peer
            messages_received += 1;
        }

        // 4. Merge received states
        let merged = self.merge_peer_states().await?;
        states_merged = merged.states_merged;
        byzantine_detected = merged.byzantine_detected;

        // 5. Majority voting (Byzantine robustness)
        self.apply_majority_vote().await?;

        let duration = round_start.elapsed();
        debug!(
            round = round,
            messages_sent,
            messages_received,
            states_merged,
            byzantine_detected,
            duration_ms = duration.as_millis(),
            "Gossip round complete"
        );

        Ok(GossipRoundStats {
            round,
            messages_sent,
            messages_received,
            states_merged,
            byzantine_detected,
            duration,
        })
    }

    /// Merge peer states (conflict resolution: higher version wins)
    async fn merge_peer_states(&self) -> Result<MergeStats> {
        let peer_states = self.peer_states.read().await;
        let mut my_state = self.my_state.write().await;

        let mut states_merged = 0;
        let mut byzantine_detected = 0;

        for (peer_id, peer_state) in peer_states.iter() {
            // Verify state hash (Byzantine detection)
            if !peer_state.verify_hash() {
                warn!(peer_id = peer_id, "Byzantine state detected: hash mismatch");
                byzantine_detected += 1;
                continue;
            }

            // Merge if newer
            if my_state.merge(peer_state.clone()) {
                trace!(
                    peer_id = peer_id,
                    old_version = my_state.version,
                    new_version = peer_state.version,
                    "Merged peer state"
                );
                states_merged += 1;
            }
        }

        Ok(MergeStats {
            states_merged,
            byzantine_detected,
        })
    }

    /// Apply majority voting (Byzantine robustness: adopt value if >2f+1 peers have it)
    async fn apply_majority_vote(&self) -> Result<()> {
        let peer_states = self.peer_states.read().await;
        if peer_states.is_empty() {
            return Ok(());
        }

        // Count votes for each state version
        let mut version_votes: HashMap<u64, usize> = HashMap::new();
        let mut version_states: HashMap<u64, VersionedState> = HashMap::new();

        for state in peer_states.values() {
            *version_votes.entry(state.version).or_insert(0) += 1;
            version_states.entry(state.version).or_insert(state.clone());
        }

        // Find majority (>2f+1 where f = max_byzantine_faults)
        let quorum = 2 * self.config.max_byzantine_faults + 1;
        for (version, votes) in version_votes {
            if votes >= quorum {
                if let Some(state) = version_states.get(&version) {
                    let mut my_state = self.my_state.write().await;
                    if state.version > my_state.version {
                        debug!(
                            old_version = my_state.version,
                            new_version = version,
                            votes,
                            quorum,
                            "Adopting majority state"
                        );
                        *my_state = state.clone();
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle incoming gossip message
    pub async fn handle_message(&self, from: AgentId, message: GossipMessage) -> Result<()> {
        match message {
            GossipMessage::Push { state, proof } => {
                // Verify proof if Byzantine detection enabled
                if self.config.byzantine_detection {
                    if let Some(proof) = proof {
                        // Verify Merkle proof (would check against known root)
                        // For now, just verify hash
                        if !state.verify_hash() {
                            warn!(from = from, "Rejected Byzantine state: invalid hash");
                            return Ok(());
                        }
                    }
                }

                // Store peer state for later merging
                self.peer_states.write().await.insert(from, state);
            }
            GossipMessage::Pull { version } => {
                // Respond with my state if I have newer version
                let my_state = self.my_state.read().await;
                if my_state.version > version {
                    // Would send Push message back to peer
                    trace!(from = from, "Responding to pull with newer state");
                }
            }
            GossipMessage::PushPull { my_state: peer_state, proof } => {
                // Exchange states
                if self.config.byzantine_detection {
                    if !peer_state.verify_hash() {
                        warn!(from = from, "Rejected Byzantine state in push-pull");
                        return Ok(());
                    }
                }

                self.peer_states.write().await.insert(from, peer_state);
                // Would send my state back
            }
            GossipMessage::Ack { version } => {
                // Acknowledgment received
                trace!(from = from, version = version, "Received ack");
            }
        }

        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> VersionedState {
        self.my_state.read().await.clone()
    }

    /// Get current round number
    pub async fn current_round(&self) -> RoundNumber {
        *self.current_round.read().await
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peer_sampler.read().await.alive_count()
    }
}

/// Gossip round statistics
#[derive(Clone, Debug)]
pub struct GossipRoundStats {
    /// Round number
    pub round: RoundNumber,
    /// Messages sent
    pub messages_sent: usize,
    /// Messages received
    pub messages_received: usize,
    /// States merged
    pub states_merged: usize,
    /// Byzantine states detected
    pub byzantine_detected: usize,
    /// Round duration
    pub duration: Duration,
}

/// Merge statistics
#[derive(Clone, Debug)]
struct MergeStats {
    states_merged: usize,
    byzantine_detected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_creation() {
        let config = GossipConfig::new(1, 10);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
        let protocol = GossipProtocol::new(config, initial_state);

        assert_eq!(protocol.current_round().await, 0);
        assert_eq!(protocol.get_state().await.version, 0);
    }

    #[tokio::test]
    async fn test_protocol_init_peers() {
        let config = GossipConfig::new(1, 10);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
        let protocol = GossipProtocol::new(config, initial_state);

        protocol.init_peers(vec![2, 3, 4, 5]).await;
        assert_eq!(protocol.peer_count().await, 4);
    }

    #[tokio::test]
    async fn test_protocol_execute_round() {
        let config = GossipConfig::new(1, 10);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
        let protocol = GossipProtocol::new(config, initial_state);

        protocol.init_peers(vec![2, 3, 4, 5]).await;

        let stats = protocol.execute_round().await.unwrap();
        assert_eq!(stats.round, 1);
        assert!(stats.messages_sent > 0);
    }

    #[tokio::test]
    async fn test_protocol_handle_push_message() {
        let config = GossipConfig::new(1, 10);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 1);
        let protocol = GossipProtocol::new(config, initial_state);

        let peer_state = VersionedState::new(1, StateValue::Number(42), 2);
        let message = GossipMessage::Push {
            state: peer_state.clone(),
            proof: None,
        };

        protocol.handle_message(2, message).await.unwrap();

        // Verify peer state was stored
        let peer_states = protocol.peer_states.read().await;
        assert!(peer_states.contains_key(&2));
    }
}
