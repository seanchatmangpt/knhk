//! Proof Circuits for Zero-Knowledge Workflow Verification
//!
//! This module contains specialized circuits for proving various workflow properties:
//! - **State Transition**: Prove valid state transitions without revealing state
//! - **Compliance**: Prove regulatory compliance (GDPR, HIPAA) without revealing data
//! - **Policy**: Prove policy adherence without revealing policy details
//! - **Computation**: Prove computation correctness without revealing inputs

pub mod state_transition;
pub mod compliance;
pub mod policy;
pub mod computation;

use super::{PrivateInputs, PublicInputs, ZkError, ZkResult};
