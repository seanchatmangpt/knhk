//! gRPC API service

// TODO: Implement gRPC service using tonic
// This is a placeholder for the gRPC API implementation

use crate::executor::WorkflowEngine;
use std::sync::Arc;

/// gRPC service for workflow engine
pub struct GrpcService {
    engine: Arc<WorkflowEngine>,
}

impl GrpcService {
    /// Create a new gRPC service
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }
}

// TODO: Implement gRPC service methods
// - RegisterWorkflow
// - GetWorkflow
// - CreateCase
// - GetCase
// - CancelCase
// - ExecuteCase

