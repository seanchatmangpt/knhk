// KNHK Phase 6: Advanced Neural Integration
// Hyper-advanced Rust with Generic Associated Types (GATs) and neural networks

pub mod model;
pub mod reinforcement;
pub mod optimizer;
pub mod training;
pub mod workflow;

pub use model::{NeuralModel, Layer, DenseLayer};
pub use reinforcement::{QLearning, SARSAAgent, WorkflowState, WorkflowAction, Agent};
pub use training::Trainer;
pub use workflow::{
    SelfLearningWorkflow, AdaptiveWorkflowExecutor, EpisodeResult, LearningMetrics,
    ModelVersion, WorkflowConfig, WorkflowMetrics, WorkflowExecutor, PerformanceTracker,
    PerformanceImprovements,
};

/// Prelude for Phase 6 neural features
pub mod prelude {
    pub use crate::model::{NeuralModel, Layer, DenseLayer};
    pub use crate::reinforcement::{QLearning, SARSAAgent, Agent};
    pub use crate::training::{Trainer, TrainingConfig};
    pub use crate::workflow::SelfLearningWorkflow;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_model_creation() {
        // Test basic neural layer creation
        let layer: DenseLayer<10, 20> = DenseLayer::new();
        assert_eq!(layer.input_size(), 10);
        assert_eq!(layer.output_size(), 20);
    }
}
