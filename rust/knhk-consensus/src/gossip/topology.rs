//! Peer Sampling and Topology Optimization
//!
//! Implements random peer sampling for epidemic dissemination and
//! latency-aware topology optimization for faster convergence.

use super::{AgentId, Result};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::trace;

/// Peer information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer agent ID
    pub id: AgentId,
    /// Network latency to this peer (milliseconds)
    pub latency_ms: u64,
    /// Peer is alive/reachable
    pub alive: bool,
    /// Last contact timestamp
    pub last_contact: u64,
}

/// Peer sampler (selects k random peers per round)
pub struct PeerSampler {
    /// My agent ID
    agent_id: AgentId,
    /// All known peers
    peers: HashMap<AgentId, PeerInfo>,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
}

impl PeerSampler {
    /// Create new peer sampler
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            peers: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    /// Add peer to list
    pub fn add_peer(&mut self, peer_id: AgentId, latency_ms: u64) {
        let peer = PeerInfo {
            id: peer_id,
            latency_ms,
            alive: true,
            last_contact: current_timestamp_ms(),
        };
        self.peers.insert(peer_id, peer);
    }

    /// Remove peer from list
    pub fn remove_peer(&mut self, peer_id: AgentId) {
        self.peers.remove(&peer_id);
    }

    /// Mark peer as alive
    pub fn mark_alive(&mut self, peer_id: AgentId) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.alive = true;
            peer.last_contact = current_timestamp_ms();
        }
    }

    /// Mark peer as dead
    pub fn mark_dead(&mut self, peer_id: AgentId) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.alive = false;
        }
    }

    /// Sample k random peers (uniform random)
    pub fn sample_random(&mut self, k: usize) -> Vec<AgentId> {
        let alive_peers: Vec<AgentId> = self
            .peers
            .values()
            .filter(|p| p.alive && p.id != self.agent_id)
            .map(|p| p.id)
            .collect();

        if alive_peers.is_empty() {
            return vec![];
        }

        let sample_size = k.min(alive_peers.len());
        alive_peers
            .choose_multiple(&mut self.rng, sample_size)
            .copied()
            .collect()
    }

    /// Sample k peers with latency-aware optimization (prefer low-latency peers)
    pub fn sample_optimized(&mut self, k: usize) -> Vec<AgentId> {
        let mut alive_peers: Vec<&PeerInfo> = self
            .peers
            .values()
            .filter(|p| p.alive && p.id != self.agent_id)
            .collect();

        if alive_peers.is_empty() {
            return vec![];
        }

        // Sort by latency (ascending)
        alive_peers.sort_by_key(|p| p.latency_ms);

        // Take k lowest-latency peers with some randomization
        let sample_size = k.min(alive_peers.len());
        let top_k = (sample_size * 2).min(alive_peers.len()); // Consider top 2k
        let candidates: Vec<AgentId> = alive_peers.iter().take(top_k).map(|p| p.id).collect();

        candidates
            .choose_multiple(&mut self.rng, sample_size)
            .copied()
            .collect()
    }

    /// Get total number of alive peers
    pub fn alive_count(&self) -> usize {
        self.peers.values().filter(|p| p.alive).count()
    }

    /// Get all peers
    pub fn all_peers(&self) -> Vec<AgentId> {
        self.peers.keys().copied().collect()
    }
}

/// Topology optimizer (learns best peer topology over time)
pub struct TopologyOptimizer {
    /// My agent ID
    agent_id: AgentId,
    /// Preferred peers (learned from successful gossip rounds)
    preferred_peers: HashSet<AgentId>,
    /// Peer success rates (gossip rounds succeeded / attempted)
    success_rates: HashMap<AgentId, (usize, usize)>, // (success, total)
    /// Minimum success rate threshold (0.0 - 1.0)
    min_success_rate: f64,
}

impl TopologyOptimizer {
    /// Create new topology optimizer
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            preferred_peers: HashSet::new(),
            success_rates: HashMap::new(),
            min_success_rate: 0.7,
        }
    }

    /// Record successful gossip with peer
    pub fn record_success(&mut self, peer_id: AgentId) {
        let entry = self.success_rates.entry(peer_id).or_insert((0, 0));
        entry.0 += 1; // Success
        entry.1 += 1; // Total

        // Add to preferred if success rate high enough
        if self.get_success_rate(peer_id) >= self.min_success_rate {
            self.preferred_peers.insert(peer_id);
        }
    }

    /// Record failed gossip with peer
    pub fn record_failure(&mut self, peer_id: AgentId) {
        let entry = self.success_rates.entry(peer_id).or_insert((0, 0));
        entry.1 += 1; // Total

        // Remove from preferred if success rate too low
        if self.get_success_rate(peer_id) < self.min_success_rate {
            self.preferred_peers.remove(&peer_id);
        }
    }

    /// Get success rate for peer
    pub fn get_success_rate(&self, peer_id: AgentId) -> f64 {
        self.success_rates
            .get(&peer_id)
            .map(|(success, total)| {
                if *total == 0 {
                    0.0
                } else {
                    *success as f64 / *total as f64
                }
            })
            .unwrap_or(0.0)
    }

    /// Get preferred peers
    pub fn preferred_peers(&self) -> Vec<AgentId> {
        self.preferred_peers.iter().copied().collect()
    }

    /// Reset optimizer
    pub fn reset(&mut self) {
        self.preferred_peers.clear();
        self.success_rates.clear();
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_sampler_creation() {
        let sampler = PeerSampler::new(1);
        assert_eq!(sampler.alive_count(), 0);
    }

    #[test]
    fn test_peer_sampler_add_remove() {
        let mut sampler = PeerSampler::new(1);
        sampler.add_peer(2, 10);
        sampler.add_peer(3, 20);
        assert_eq!(sampler.alive_count(), 2);

        sampler.remove_peer(2);
        assert_eq!(sampler.alive_count(), 1);
    }

    #[test]
    fn test_peer_sampler_random_sampling() {
        let mut sampler = PeerSampler::new(1);
        for i in 2..12 {
            sampler.add_peer(i, i * 10);
        }

        let sample = sampler.sample_random(5);
        assert_eq!(sample.len(), 5);
        assert!(!sample.contains(&1)); // Should not include self
    }

    #[test]
    fn test_peer_sampler_optimized_sampling() {
        let mut sampler = PeerSampler::new(1);
        sampler.add_peer(2, 100); // High latency
        sampler.add_peer(3, 10); // Low latency
        sampler.add_peer(4, 50); // Medium latency

        let sample = sampler.sample_optimized(2);
        assert_eq!(sample.len(), 2);
        // Should prefer lower latency peers (3 and 4 over 2)
    }

    #[test]
    fn test_topology_optimizer() {
        let mut optimizer = TopologyOptimizer::new(1);

        // Record successes
        optimizer.record_success(2);
        optimizer.record_success(2);
        optimizer.record_success(2);

        // Record failures
        optimizer.record_failure(3);
        optimizer.record_failure(3);
        optimizer.record_failure(3);

        assert!(optimizer.get_success_rate(2) >= 0.7);
        assert!(optimizer.get_success_rate(3) < 0.7);
        assert!(optimizer.preferred_peers().contains(&2));
        assert!(!optimizer.preferred_peers().contains(&3));
    }
}
