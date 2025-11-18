//! MAPE-K integration for federated learning
//!
//! Integrates federated learned models with the MAPE-K autonomic loop.

use super::traits::LocalModel;
use super::types::{Experience, FederatedError};
use crate::error::WorkflowResult;

/// Convert MAPE-K observation to federated learning experience
///
/// # Arguments
///
/// - `observation`: MAPE-K observation (telemetry, receipts, etc.)
///
/// # Returns
///
/// Experience sample for federated learning
pub fn observation_to_experience(observation: &str) -> Result<Experience, FederatedError> {
    // Stub: Convert observation to experience
    // In real implementation, this would:
    // 1. Extract state features from observation
    // 2. Determine action taken
    // 3. Compute reward from performance metrics

    Ok(Experience {
        state: vec![0.5; 100], // Stub state
        action: 0,              // Stub action
        reward: 0.5,            // Stub reward
        next_state: vec![0.5; 100],
        done: false,
    })
}

/// Convert symptom to state representation for model prediction
///
/// # Arguments
///
/// - `symptom_severity`: Severity score (0.0-1.0)
/// - `symptom_frequency`: Frequency score (0.0-1.0)
/// - `symptom_impact`: Impact score (0.0-1.0)
///
/// # Returns
///
/// State vector for model prediction
pub fn symptom_to_state(
    symptom_severity: f64,
    symptom_frequency: f64,
    symptom_impact: f64,
) -> Vec<f32> {
    // Stub: Feature engineering
    // In real implementation, include:
    // - Symptom type encoding
    // - Historical patterns
    // - Resource metrics
    // - Workflow complexity

    vec![
        symptom_severity as f32,
        symptom_frequency as f32,
        symptom_impact as f32,
        // ... more features
    ]
}

/// Map predicted action to adaptation plan
///
/// # Arguments
///
/// - `action`: Action index from model prediction
///
/// # Returns
///
/// Adaptation plan name
pub fn action_to_adaptation_plan(action: usize) -> Result<String, FederatedError> {
    match action {
        0 => Ok("IncreaseResources".to_string()),
        1 => Ok("OptimizeWorkflow".to_string()),
        2 => Ok("ScaleOut".to_string()),
        3 => Ok("RollbackChange".to_string()),
        4 => Ok("AdjustConcurrency".to_string()),
        5 => Ok("EnableCaching".to_string()),
        6 => Ok("ReduceLoad".to_string()),
        7 => Ok("SwitchAlgorithm".to_string()),
        8 => Ok("UpdateConfiguration".to_string()),
        9 => Ok("NoAction".to_string()),
        _ => Err(FederatedError::ModelError(format!(
            "Invalid action: {}",
            action
        ))),
    }
}

/// Use federated model for MAPE-K Plan phase
///
/// # Arguments
///
/// - `model`: Trained federated model
/// - `symptoms`: Detected symptoms from Analyze phase
///
/// # Returns
///
/// List of adaptation plans
pub fn generate_plans_with_federated_model(
    model: &dyn LocalModel,
    symptoms: &[(f64, f64, f64)], // (severity, frequency, impact)
) -> WorkflowResult<Vec<String>> {
    let mut plans = vec![];

    for (severity, frequency, impact) in symptoms {
        // Convert symptom to state
        let state = symptom_to_state(*severity, *frequency, *impact);

        // Predict action using learned model
        let action = model
            .predict(&state)
            .map_err(|e| crate::error::WorkflowError::InternalError(e.to_string()))?;

        // Map action to plan
        let plan = action_to_adaptation_plan(action)
            .map_err(|e| crate::error::WorkflowError::InternalError(e.to_string()))?;

        plans.push(plan);
    }

    Ok(plans)
}

/// Determine if federated model needs retraining
///
/// # Criteria
///
/// 1. Model performance degraded (accuracy < threshold)
/// 2. Distribution shift detected (KL divergence > threshold)
/// 3. New workflow patterns observed
///
/// # Arguments
///
/// - `model_accuracy`: Current model accuracy
/// - `distribution_shift`: KL divergence from training distribution
/// - `new_pattern_count`: Number of new patterns observed
///
/// # Returns
///
/// True if retraining needed
pub fn should_retrain_model(
    model_accuracy: f64,
    distribution_shift: f64,
    new_pattern_count: usize,
) -> bool {
    const ACCURACY_THRESHOLD: f64 = 0.8;
    const DISTRIBUTION_SHIFT_THRESHOLD: f64 = 0.1;
    const NEW_PATTERN_THRESHOLD: usize = 100;

    model_accuracy < ACCURACY_THRESHOLD
        || distribution_shift > DISTRIBUTION_SHIFT_THRESHOLD
        || new_pattern_count > NEW_PATTERN_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symptom_to_state() {
        let state = symptom_to_state(0.8, 0.5, 0.9);

        assert_eq!(state[0], 0.8);
        assert_eq!(state[1], 0.5);
        assert_eq!(state[2], 0.9);
    }

    #[test]
    fn test_action_to_adaptation_plan() {
        assert_eq!(
            action_to_adaptation_plan(0).unwrap(),
            "IncreaseResources"
        );
        assert_eq!(action_to_adaptation_plan(1).unwrap(), "OptimizeWorkflow");
        assert_eq!(action_to_adaptation_plan(9).unwrap(), "NoAction");

        assert!(action_to_adaptation_plan(100).is_err());
    }

    #[test]
    fn test_should_retrain_model() {
        // Low accuracy
        assert!(should_retrain_model(0.7, 0.05, 50));

        // High distribution shift
        assert!(should_retrain_model(0.9, 0.15, 50));

        // Many new patterns
        assert!(should_retrain_model(0.9, 0.05, 150));

        // All good
        assert!(!should_retrain_model(0.9, 0.05, 50));
    }
}
