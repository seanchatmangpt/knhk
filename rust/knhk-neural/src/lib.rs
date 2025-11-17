// KNHK Phase 6: Advanced Neural Integration
// Hyper-advanced Rust with Generic Associated Types (GATs) and neural networks

pub mod model;
pub mod optimizer;
pub mod reinforcement;
pub mod training;
pub mod workflow;

pub use model::{DenseLayer, Layer, NeuralModel};
pub use reinforcement::{Agent, QLearning, SARSAAgent, WorkflowAction, WorkflowState};
pub use training::Trainer;
pub use workflow::{
    AdaptiveWorkflowExecutor, EpisodeResult, LearningMetrics, ModelVersion,
    PerformanceImprovements, PerformanceTracker, SelfLearningWorkflow, WorkflowConfig,
    WorkflowExecutor, WorkflowMetrics,
};

/// Prelude for Phase 6 neural features
pub mod prelude {
    pub use crate::model::{DenseLayer, Layer, NeuralModel};
    pub use crate::reinforcement::{Agent, QLearning, SARSAAgent};
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
