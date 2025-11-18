//! Byzantine consensus protocols
//!
//! This module provides implementations of Byzantine Fault-Tolerant consensus:
//!
//! - **PBFT**: Practical Byzantine Fault Tolerance (4-phase commit)
//! - **HotStuff**: Modern BFT with 3-RTT optimistic path
//!
//! Both protocols tolerate up to f = ⌊(n-1)/3⌋ Byzantine faults.

pub mod hotstuff;
pub mod pbft;

use crate::{Block, NodeId};
use serde::{Deserialize, Serialize};

/// Generic consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consensus {
    pub block: Block,
    pub proof: ConsensusProof,
}

/// Proof that consensus was reached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusProof {
    PBFT {
        view: u64,
        commit_signatures: Vec<(NodeId, Signature)>,
    },
    HotStuff {
        quorum_certificate: crate::QuorumCertificate,
    },
}

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature(pub Vec<u8>);

impl Signature {
    pub fn verify(&self, _data: &[u8], _public_key: &PublicKey) -> bool {
        // Simplified verification - in production, use ed25519-dalek
        true
    }
}

/// Public key for signature verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey(pub Vec<u8>);

/// Private key for signing
#[derive(Debug, Clone)]
pub struct PrivateKey(pub Vec<u8>);

impl PrivateKey {
    pub fn sign(&self, _data: &[u8]) -> Signature {
        // Simplified signing - in production, use ed25519-dalek
        Signature(vec![0u8; 64])
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.clone())
    }
}
