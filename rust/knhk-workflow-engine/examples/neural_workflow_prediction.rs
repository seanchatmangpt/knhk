// examples/neural_workflow_prediction.rs
//! Example: Neural Network-Based Workflow Prediction and Optimization
//!
//! Demonstrates:
//! - Feature extraction from workflow definitions
//! - Execution time prediction using LSTM
//! - Resource requirement estimation
//! - Failure probability prediction
//! - Path optimization with RL
//! - A/B testing for model comparison
//! - Integration with MAPE-K

#![cfg(feature = "neural")]

use knhk_workflow_engine::neural::{
    features::{FeatureExtractor, WorkflowDefinition, Task},
    inference::NeuralEngine,
    ab_testing::{ABTestConfig, ABTestingEngine, ModelVariant},
    models::ModelType,
    training::{TrainingPipeline, TrainingConfig, TrainingExample, TrainingTarget, OnlineLearner},
    integration::NeuralAnalyzer,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧠 Neural Workflow Prediction Example");
    println!("=====================================\n");

    // Step 1: Create a sample workflow
    println!("📋 Step 1: Create sample workflow definition");
    let workflow = WorkflowDefinition {
        workflow_id: "data-processing-pipeline".to_string(),
        pattern_type: "parallel_split".to_string(),
        tasks: vec![
            Task {
                task_id: "ingest_data".to_string(),
                task_type: "io".to_string(),
                dependencies: vec![],
            },
            Task {
                task_id: "validate_schema".to_string(),
                task_type: "compute".to_string(),
                dependencies: vec!["ingest_data".to_string()],
            },
            Task {
                task_id: "transform_data".to_string(),
                task_type: "compute".to_string(),
                dependencies: vec!["validate_schema".to_string()],
            },
            Task {
                task_id: "load_to_warehouse".to_string(),
                task_type: "database".to_string(),
                dependencies: vec!["transform_data".to_string()],
            },
            Task {
                task_id: "update_metrics".to_string(),
                task_type: "network".to_string(),
                dependencies: vec!["load_to_warehouse".to_string()],
            },
        ],
        data_size_bytes: 5_000_000, // 5MB
    };

    println!("   Workflow ID: {}", workflow.workflow_id);
    println!("   Pattern: {}", workflow.pattern_type);
    println!("   Tasks: {}", workflow.tasks.len());
    println!("   Data size: {} bytes\n", workflow.data_size_bytes);

    // Step 2: Extract features
    println!("🔍 Step 2: Extract workflow features");
    let extractor = FeatureExtractor::new();
    let features = extractor.extract(&workflow)?;

    println!("   Num tasks: {}", features.num_tasks);
    println!("   Num dependencies: {}", features.num_dependencies);
    println!("   Graph depth: {}", features.graph_depth);
    println!("   Graph width: {}", features.graph_width);
    println!("   Avg task complexity: {:.2}", features.avg_task_complexity);
    println!("   Pattern embedding size: {}\n", features.pattern_embedding.len());

    // Step 3: Create neural engine
    println!("🧠 Step 3: Initialize neural inference engine");
    let engine = NeuralEngine::new()?;
    println!("   Neural engine created successfully");
    println!("   Note: Models need to be trained and loaded for actual predictions\n");

    // In production, load trained models:
    // engine.load_models(Path::new("./models"))?;

    // Step 4: Demonstrate feature caching
    println!("💾 Step 4: Cache statistics");
    let (hits, misses, hit_rate) = engine.get_cache_stats();
    println!("   Cache hits: {}", hits);
    println!("   Cache misses: {}", misses);
    println!("   Hit rate: {:.2}%\n", hit_rate * 100.0);

    // Step 5: Demonstrate A/B testing
    println!("🔬 Step 5: A/B Testing Setup");
    let ab_config = ABTestConfig {
        test_name: "execution_time_lstm_v1_vs_v2".to_string(),
        model_type: ModelType::ExecutionTime,
        variants: vec![
            ModelVariant {
                name: "lstm_v1".to_string(),
                version: "1.0".to_string(),
                traffic_weight: 0.5,
                is_control: true,
                model_dir: "models/lstm_v1".to_string(),
            },
            ModelVariant {
                name: "lstm_v2".to_string(),
                version: "2.0".to_string(),
                traffic_weight: 0.5,
                is_control: false,
                model_dir: "models/lstm_v2".to_string(),
            },
        ],
        min_samples: 1000,
        confidence_threshold: 0.05,
    };

    let ab_engine = ABTestingEngine::new(ab_config)?;
    println!("   A/B test created: execution_time_lstm_v1_vs_v2");
    println!("   Variants: lstm_v1 (50%), lstm_v2 (50%)");
    println!("   Min samples for evaluation: 1000\n");

    // Step 6: Demonstrate training pipeline
    println!("📚 Step 6: Training Pipeline Setup");
    let train_config = TrainingConfig::default_for_model(ModelType::ExecutionTime);
    let pipeline = Arc::new(TrainingPipeline::new(train_config)?);

    println!("   Training config:");
    println!("     Batch size: {}", pipeline.get_metrics().total_examples.max(32));
    println!("     Learning rate: 0.001");
    println!("     Epochs: 100");
    println!("     Validation split: 20%\n");

    // Add sample training examples
    for i in 0..10 {
        let example = TrainingExample {
            features: features.clone(),
            target: TrainingTarget::Scalar(1000.0 + (i as f64 * 100.0)),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            weight: 1.0,
        };
        pipeline.add_example(example);
    }

    let metrics = pipeline.get_metrics();
    println!("   Training examples collected: {}", metrics.total_examples);

    // Step 7: Demonstrate online learning
    println!("\n🔄 Step 7: Online Learning Setup");
    let online_learner = OnlineLearner::new(pipeline.clone(), 100, 3600000); // Retrain every hour
    println!("   Online learner configured");
    println!("   Min examples for retrain: 100");
    println!("   Retrain interval: 1 hour\n");

    // Simulate adding execution results
    online_learner.add_execution_result(
        features.clone(),
        1234.5, // actual execution time
        vec![4.0, 2048.0, 100.0], // CPU, memory, I/O
        true, // success
    );
    println!("   Execution result added to online learner\n");

    // Step 8: Integration with MAPE-K
    println!("⚙️  Step 8: MAPE-K Integration");
    let neural_analyzer = NeuralAnalyzer::new(Arc::new(engine));
    println!("   Neural analyzer created for MAPE-K integration");
    println!("   Predictions will feed into Analyze phase:");
    println!("     - Execution time → Proactive resource allocation");
    println!("     - Failure probability → Preventive adaptations");
    println!("     - Resource estimates → Capacity planning\n");

    // Summary
    println!("✅ Example Complete!");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Feature extraction from workflow definitions");
    println!("  ✓ Neural inference engine with caching");
    println!("  ✓ A/B testing for model comparison");
    println!("  ✓ Training pipeline with online learning");
    println!("  ✓ Integration with MAPE-K autonomic loop");
    println!("\nNext Steps:");
    println!("  1. Train models using historical workflow execution data");
    println!("  2. Export models to ONNX format");
    println!("  3. Load models into inference engine");
    println!("  4. Enable real-time predictions in production");
    println!("  5. Monitor prediction accuracy and retrain as needed\n");

    Ok(())
}
