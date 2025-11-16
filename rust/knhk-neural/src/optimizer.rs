// Phase 6: Advanced Optimization Algorithms for Neural Training
// Implements SGD, Adam, AdamW with learning rate schedules
// Supports momentum, Nesterov, bias correction, and weight decay
// All optimizers are Send + Sync for async context safety
// Production-ready with gradient clipping, checkpointing, and full hyperparameter validation

use serde::{Deserialize, Serialize};
use std::f32;

/// Primary optimizer trait for phase 6 neural training
/// Unified interface for all optimizer implementations
pub trait Optimizer: Clone + Send + Sync {
    /// Single optimization step: update parameters based on gradients
    /// Returns the weight deltas for this step
    fn step(&mut self, gradients: &[f32]) -> Vec<f32>;

    /// Update learning rate (for scheduling and adaptive learning)
    fn update_learning_rate(&mut self, new_lr: f32);

    /// Reset optimizer state (for new training epochs or reinitialization)
    fn reset_state(&mut self);

    /// Get current learning rate
    fn get_learning_rate(&self) -> f32;

    /// Checkpoint optimizer state for serialization
    fn state_dict(&self) -> OptimizerCheckpoint;

    /// Restore optimizer from checkpoint
    fn load_state_dict(&mut self, checkpoint: &OptimizerCheckpoint);
}

/// Internal optimizer state trait (kept for backward compatibility)
/// All optimizers implement both Optimizer and OptimizerState
pub trait OptimizerState: Clone + Send + Sync {
    /// Update weights based on gradient and learning rate
    /// Returns updated weight deltas
    fn update(&mut self, gradient: &[f32], lr: f32) -> Vec<f32>;

    /// Get current learning rate
    fn learning_rate(&self) -> f32;

    /// Reset optimizer state (for new training epochs)
    fn reset(&mut self);

    /// Checkpoint optimizer state for serialization (internal)
    fn get_state_dict(&self) -> OptimizerCheckpoint;

    /// Restore optimizer from checkpoint (internal)
    fn set_state_dict(&mut self, checkpoint: &OptimizerCheckpoint);
}

/// Checkpoint for optimizer state persistence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerCheckpoint {
    pub optimizer_type: String,
    pub learning_rate: f32,
    pub momentum: Option<f32>,
    pub velocity: Option<Vec<f32>>,
    pub m_t: Option<Vec<f32>>,
    pub v_t: Option<Vec<f32>>,
    pub t: Option<usize>,
    pub weight_decay: Option<f32>,
}

/// Learning rate schedule for adaptive learning during training
#[derive(Clone, Debug)]
pub enum LRSchedule {
    /// Constant learning rate throughout training
    Constant(f32),

    /// Step decay: lr = initial_lr * decay_rate^(step / step_size)
    StepDecay {
        initial_lr: f32,
        decay_rate: f32,
        step_size: usize,
    },

    /// Exponential decay: lr = initial_lr * decay_rate^step
    ExponentialDecay {
        initial_lr: f32,
        decay_rate: f32,
    },

    /// Cosine annealing: lr follows cosine curve from initial to eta_min
    CosineAnnealing {
        initial_lr: f32,
        t_max: usize,
        eta_min: f32,
    },

    /// Linear warmup followed by decay
    WarmupDecay {
        warmup_steps: usize,
        initial_lr: f32,
        base_lr: f32,
        decay_rate: f32,
    },
}

impl LRSchedule {
    /// Get learning rate for current training step
    pub fn get_lr(&self, step: usize) -> f32 {
        match self {
            LRSchedule::Constant(lr) => *lr,

            LRSchedule::StepDecay {
                initial_lr,
                decay_rate,
                step_size,
            } => {
                let decay_factor = (step / step_size) as i32;
                initial_lr * decay_rate.powi(decay_factor)
            }

            LRSchedule::ExponentialDecay {
                initial_lr,
                decay_rate,
            } => {
                initial_lr * decay_rate.powi(step as i32)
            }

            LRSchedule::CosineAnnealing {
                initial_lr,
                t_max,
                eta_min,
            } => {
                let progress = (step as f32) / (*t_max as f32);
                let cosine_decay =
                    (1.0 + (progress * std::f32::consts::PI).cos()) / 2.0;
                eta_min + (initial_lr - eta_min) * cosine_decay
            }

            LRSchedule::WarmupDecay {
                warmup_steps,
                initial_lr: _,
                base_lr,
                decay_rate,
            } => {
                if step < *warmup_steps {
                    (step as f32 / *warmup_steps as f32) * base_lr
                } else {
                    let decay_step = step - warmup_steps;
                    base_lr * decay_rate.powi(decay_step as i32)
                }
            }
        }
    }

    /// Create constant schedule
    pub fn constant(lr: f32) -> Self {
        LRSchedule::Constant(lr)
    }

    /// Create step decay schedule
    pub fn step_decay(initial_lr: f32, decay_rate: f32, step_size: usize) -> Self {
        LRSchedule::StepDecay {
            initial_lr,
            decay_rate,
            step_size,
        }
    }

    /// Create exponential decay schedule
    pub fn exponential(initial_lr: f32, decay_rate: f32) -> Self {
        LRSchedule::ExponentialDecay {
            initial_lr,
            decay_rate,
        }
    }

    /// Create cosine annealing schedule
    pub fn cosine(initial_lr: f32, t_max: usize) -> Self {
        LRSchedule::CosineAnnealing {
            initial_lr,
            t_max,
            eta_min: 0.0,
        }
    }
}

/// Configuration for SGD optimizer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGDConfig {
    pub learning_rate: f32,
    pub momentum: f32,
    pub nesterov: bool,
    pub weight_decay: f32,
}

impl SGDConfig {
    /// Create new SGD configuration with validation
    pub fn new(learning_rate: f32) -> Result<Self, String> {
        if learning_rate <= 0.0 || !learning_rate.is_finite() {
            return Err("Learning rate must be positive and finite".to_string());
        }
        Ok(SGDConfig {
            learning_rate,
            momentum: 0.9,
            nesterov: false,
            weight_decay: 0.0,
        })
    }

    /// Set momentum with validation (must be in [0, 1])
    pub fn with_momentum(mut self, momentum: f32) -> Result<Self, String> {
        if momentum < 0.0 || momentum > 1.0 || !momentum.is_finite() {
            return Err("Momentum must be in [0, 1] and finite".to_string());
        }
        self.momentum = momentum;
        Ok(self)
    }

    /// Set weight decay with validation (must be non-negative)
    pub fn with_weight_decay(mut self, weight_decay: f32) -> Result<Self, String> {
        if weight_decay < 0.0 || !weight_decay.is_finite() {
            return Err("Weight decay must be non-negative and finite".to_string());
        }
        self.weight_decay = weight_decay;
        Ok(self)
    }

    /// Enable/disable Nesterov acceleration
    pub fn with_nesterov(mut self, nesterov: bool) -> Self {
        self.nesterov = nesterov;
        self
    }
}

impl Default for SGDConfig {
    fn default() -> Self {
        SGDConfig {
            learning_rate: 0.01,
            momentum: 0.9,
            nesterov: false,
            weight_decay: 0.0,
        }
    }
}

/// Stochastic Gradient Descent with momentum and optional Nesterov acceleration
/// Default momentum: 0.9
/// Production-ready with full hyperparameter validation
#[derive(Clone, Debug)]
pub struct SGDOptimizer {
    learning_rate: f32,
    momentum: f32,
    nesterov: bool,
    velocity: Vec<f32>,
    weight_decay: f32,
}

impl SGDOptimizer {
    /// Create new SGD optimizer with validation
    pub fn new(learning_rate: f32) -> Result<Self, String> {
        let config = SGDConfig::new(learning_rate)?;
        Ok(SGDOptimizer {
            learning_rate: config.learning_rate,
            momentum: config.momentum,
            nesterov: config.nesterov,
            velocity: Vec::new(),
            weight_decay: config.weight_decay,
        })
    }

    /// Create SGD from configuration
    pub fn from_config(config: SGDConfig) -> Self {
        SGDOptimizer {
            learning_rate: config.learning_rate,
            momentum: config.momentum,
            nesterov: config.nesterov,
            velocity: Vec::new(),
            weight_decay: config.weight_decay,
        }
    }

    /// Set momentum coefficient (default 0.9)
    pub fn with_momentum(mut self, momentum: f32) -> Result<Self, String> {
        if momentum < 0.0 || momentum > 1.0 || !momentum.is_finite() {
            return Err("Momentum must be in [0, 1] and finite".to_string());
        }
        self.momentum = momentum;
        Ok(self)
    }

    /// Enable Nesterov acceleration for faster convergence
    pub fn with_nesterov(mut self, nesterov: bool) -> Self {
        self.nesterov = nesterov;
        self
    }

    /// Set L2 weight decay (default 0.0)
    pub fn with_weight_decay(mut self, weight_decay: f32) -> Result<Self, String> {
        if weight_decay < 0.0 || !weight_decay.is_finite() {
            return Err("Weight decay must be non-negative and finite".to_string());
        }
        self.weight_decay = weight_decay;
        Ok(self)
    }

    /// Initialize velocity buffers for given parameter count
    fn initialize(&mut self, param_count: usize) {
        if self.velocity.is_empty() {
            self.velocity = vec![0.0; param_count];
        }
    }

    /// Get momentum value
    pub fn get_momentum(&self) -> f32 {
        self.momentum
    }

    /// Get nesterov flag
    pub fn is_nesterov(&self) -> bool {
        self.nesterov
    }

    /// Get weight decay value
    pub fn get_weight_decay(&self) -> f32 {
        self.weight_decay
    }
}

impl OptimizerState for SGDOptimizer {
    fn update(&mut self, gradient: &[f32], lr: f32) -> Vec<f32> {
        self.initialize(gradient.len());

        let mut updates = Vec::with_capacity(gradient.len());

        for (i, grad) in gradient.iter().enumerate() {
            let grad_with_decay = grad + self.weight_decay;

            self.velocity[i] = self.momentum * self.velocity[i] + grad_with_decay;

            let update = if self.nesterov {
                self.momentum * self.velocity[i] + grad_with_decay
            } else {
                self.velocity[i]
            };

            updates.push(lr * update);
        }

        updates
    }

    fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    fn reset(&mut self) {
        self.velocity.clear();
    }

    fn get_state_dict(&self) -> OptimizerCheckpoint {
        OptimizerCheckpoint {
            optimizer_type: "SGD".to_string(),
            learning_rate: self.learning_rate,
            momentum: Some(self.momentum),
            velocity: Some(self.velocity.clone()),
            m_t: None,
            v_t: None,
            t: None,
            weight_decay: Some(self.weight_decay),
        }
    }

    fn set_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.learning_rate = checkpoint.learning_rate;
        if let Some(momentum) = checkpoint.momentum {
            self.momentum = momentum;
        }
        if let Some(velocity) = &checkpoint.velocity {
            self.velocity = velocity.clone();
        }
        if let Some(wd) = checkpoint.weight_decay {
            self.weight_decay = wd;
        }
    }
}

impl Optimizer for SGDOptimizer {
    fn step(&mut self, gradients: &[f32]) -> Vec<f32> {
        self.update(gradients, self.learning_rate)
    }

    fn update_learning_rate(&mut self, new_lr: f32) {
        if new_lr > 0.0 && new_lr.is_finite() {
            self.learning_rate = new_lr;
        }
    }

    fn reset_state(&mut self) {
        self.reset();
    }

    fn get_learning_rate(&self) -> f32 {
        self.learning_rate()
    }

    fn state_dict(&self) -> OptimizerCheckpoint {
        self.get_state_dict()
    }

    fn load_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.set_state_dict(checkpoint)
    }
}

/// Configuration for Adam optimizer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdamConfig {
    pub learning_rate: f32,
    pub beta1: f32,
    pub beta2: f32,
    pub epsilon: f32,
    pub weight_decay: f32,
}

impl AdamConfig {
    /// Create new Adam configuration with validation
    pub fn new(learning_rate: f32) -> Result<Self, String> {
        if learning_rate <= 0.0 || !learning_rate.is_finite() {
            return Err("Learning rate must be positive and finite".to_string());
        }
        Ok(AdamConfig {
            learning_rate,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            weight_decay: 0.0,
        })
    }

    /// Set beta1 with validation (must be in [0, 1))
    pub fn with_beta1(mut self, beta1: f32) -> Result<Self, String> {
        if beta1 < 0.0 || beta1 >= 1.0 || !beta1.is_finite() {
            return Err("Beta1 must be in [0, 1) and finite".to_string());
        }
        self.beta1 = beta1;
        Ok(self)
    }

    /// Set beta2 with validation (must be in [0, 1))
    pub fn with_beta2(mut self, beta2: f32) -> Result<Self, String> {
        if beta2 < 0.0 || beta2 >= 1.0 || !beta2.is_finite() {
            return Err("Beta2 must be in [0, 1) and finite".to_string());
        }
        self.beta2 = beta2;
        Ok(self)
    }

    /// Set epsilon with validation (must be very small positive)
    pub fn with_epsilon(mut self, epsilon: f32) -> Result<Self, String> {
        if epsilon <= 0.0 || epsilon >= 0.01 || !epsilon.is_finite() {
            return Err("Epsilon must be in (0, 0.01) and finite".to_string());
        }
        self.epsilon = epsilon;
        Ok(self)
    }

    /// Set weight decay with validation
    pub fn with_weight_decay(mut self, weight_decay: f32) -> Result<Self, String> {
        if weight_decay < 0.0 || !weight_decay.is_finite() {
            return Err("Weight decay must be non-negative and finite".to_string());
        }
        self.weight_decay = weight_decay;
        Ok(self)
    }
}

impl Default for AdamConfig {
    fn default() -> Self {
        AdamConfig {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            weight_decay: 0.0,
        }
    }
}

/// Adam optimizer: Adaptive Moment Estimation
/// Combines momentum (1st moment) and RMSprop (2nd moment)
/// Default: lr=0.001, beta1=0.9, beta2=0.999, epsilon=1e-8
/// Production-ready with full hyperparameter validation
#[derive(Clone, Debug)]
pub struct AdamOptimizer {
    learning_rate: f32,
    beta1: f32,
    beta2: f32,
    epsilon: f32,
    m_t: Vec<f32>,
    v_t: Vec<f32>,
    t: usize,
    weight_decay: f32,
}

impl AdamOptimizer {
    /// Create new Adam optimizer with validation
    pub fn new(learning_rate: f32) -> Result<Self, String> {
        let config = AdamConfig::new(learning_rate)?;
        Ok(AdamOptimizer {
            learning_rate: config.learning_rate,
            beta1: config.beta1,
            beta2: config.beta2,
            epsilon: config.epsilon,
            m_t: Vec::new(),
            v_t: Vec::new(),
            t: 0,
            weight_decay: config.weight_decay,
        })
    }

    /// Create Adam from configuration
    pub fn from_config(config: AdamConfig) -> Self {
        AdamOptimizer {
            learning_rate: config.learning_rate,
            beta1: config.beta1,
            beta2: config.beta2,
            epsilon: config.epsilon,
            m_t: Vec::new(),
            v_t: Vec::new(),
            t: 0,
            weight_decay: config.weight_decay,
        }
    }

    /// Set beta1 for 1st moment decay (default 0.9)
    pub fn with_beta1(mut self, beta1: f32) -> Result<Self, String> {
        if beta1 < 0.0 || beta1 >= 1.0 || !beta1.is_finite() {
            return Err("Beta1 must be in [0, 1) and finite".to_string());
        }
        self.beta1 = beta1;
        Ok(self)
    }

    /// Set beta2 for 2nd moment decay (default 0.999)
    pub fn with_beta2(mut self, beta2: f32) -> Result<Self, String> {
        if beta2 < 0.0 || beta2 >= 1.0 || !beta2.is_finite() {
            return Err("Beta2 must be in [0, 1) and finite".to_string());
        }
        self.beta2 = beta2;
        Ok(self)
    }

    /// Set epsilon for numerical stability (default 1e-8)
    pub fn with_epsilon(mut self, epsilon: f32) -> Result<Self, String> {
        if epsilon <= 0.0 || epsilon >= 0.01 || !epsilon.is_finite() {
            return Err("Epsilon must be in (0, 0.01) and finite".to_string());
        }
        self.epsilon = epsilon;
        Ok(self)
    }

    /// Set L2 weight decay (default 0.0)
    pub fn with_weight_decay(mut self, weight_decay: f32) -> Result<Self, String> {
        if weight_decay < 0.0 || !weight_decay.is_finite() {
            return Err("Weight decay must be non-negative and finite".to_string());
        }
        self.weight_decay = weight_decay;
        Ok(self)
    }

    /// Get timestep count
    pub fn get_timestep(&self) -> usize {
        self.t
    }

    /// Get beta1 value
    pub fn get_beta1(&self) -> f32 {
        self.beta1
    }

    /// Get beta2 value
    pub fn get_beta2(&self) -> f32 {
        self.beta2
    }

    /// Get epsilon value
    pub fn get_epsilon(&self) -> f32 {
        self.epsilon
    }

    /// Initialize moment buffers
    fn initialize(&mut self, param_count: usize) {
        if self.m_t.is_empty() {
            self.m_t = vec![0.0; param_count];
            self.v_t = vec![0.0; param_count];
        }
    }
}

impl OptimizerState for AdamOptimizer {
    fn update(&mut self, gradient: &[f32], lr: f32) -> Vec<f32> {
        self.initialize(gradient.len());
        self.t += 1;

        let mut updates = Vec::with_capacity(gradient.len());

        let bias_correction1 = 1.0 - self.beta1.powi(self.t as i32);
        let bias_correction2 = 1.0 - self.beta2.powi(self.t as i32);

        for (i, grad) in gradient.iter().enumerate() {
            let grad_with_decay = grad + self.weight_decay;

            self.m_t[i] = self.beta1 * self.m_t[i]
                + (1.0 - self.beta1) * grad_with_decay;

            self.v_t[i] = self.beta2 * self.v_t[i]
                + (1.0 - self.beta2) * grad_with_decay * grad_with_decay;

            let m_hat = self.m_t[i] / bias_correction1;
            let v_hat = self.v_t[i] / bias_correction2;

            let denominator = v_hat.sqrt() + self.epsilon;
            let update = lr * m_hat / denominator;

            updates.push(update);
        }

        updates
    }

    fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    fn reset(&mut self) {
        self.m_t.clear();
        self.v_t.clear();
        self.t = 0;
    }

    fn get_state_dict(&self) -> OptimizerCheckpoint {
        OptimizerCheckpoint {
            optimizer_type: "Adam".to_string(),
            learning_rate: self.learning_rate,
            momentum: Some(self.beta1),
            velocity: Some(self.v_t.clone()),
            m_t: Some(self.m_t.clone()),
            v_t: Some(self.v_t.clone()),
            t: Some(self.t),
            weight_decay: Some(self.weight_decay),
        }
    }

    fn set_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.learning_rate = checkpoint.learning_rate;
        if let Some(beta1) = checkpoint.momentum {
            self.beta1 = beta1;
        }
        if let Some(m_t) = &checkpoint.m_t {
            self.m_t = m_t.clone();
        }
        if let Some(v_t) = &checkpoint.v_t {
            self.v_t = v_t.clone();
        }
        if let Some(t) = checkpoint.t {
            self.t = t;
        }
        if let Some(wd) = checkpoint.weight_decay {
            self.weight_decay = wd;
        }
    }
}

impl Optimizer for AdamOptimizer {
    fn step(&mut self, gradients: &[f32]) -> Vec<f32> {
        self.update(gradients, self.learning_rate)
    }

    fn update_learning_rate(&mut self, new_lr: f32) {
        if new_lr > 0.0 && new_lr.is_finite() {
            self.learning_rate = new_lr;
        }
    }

    fn reset_state(&mut self) {
        self.reset();
    }

    fn get_learning_rate(&self) -> f32 {
        self.learning_rate()
    }

    fn state_dict(&self) -> OptimizerCheckpoint {
        self.get_state_dict()
    }

    fn load_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.set_state_dict(checkpoint)
    }
}

/// AdamW: Adam with decoupled weight decay
/// Separates weight decay from gradient-based update
/// Production-ready with decoupled weight decay regularization
#[derive(Clone, Debug)]
pub struct AdamWOptimizer {
    adam: AdamOptimizer,
    weight_decay: f32,
}

impl AdamWOptimizer {
    /// Create new AdamW optimizer with validation
    pub fn new(learning_rate: f32) -> Result<Self, String> {
        let adam = AdamOptimizer::new(learning_rate)?;
        Ok(AdamWOptimizer {
            adam,
            weight_decay: 0.01,
        })
    }

    /// Create AdamW from config
    pub fn from_config(config: AdamConfig) -> Self {
        AdamWOptimizer {
            adam: AdamOptimizer::from_config(config),
            weight_decay: 0.01,
        }
    }

    /// Set weight decay coefficient with validation
    pub fn with_weight_decay(mut self, weight_decay: f32) -> Result<Self, String> {
        if weight_decay < 0.0 || !weight_decay.is_finite() {
            return Err("Weight decay must be non-negative and finite".to_string());
        }
        self.weight_decay = weight_decay;
        self.adam.weight_decay = 0.0;  // Ensure decoupling
        Ok(self)
    }

    /// Set beta1 with delegation to Adam
    pub fn with_beta1(mut self, beta1: f32) -> Result<Self, String> {
        self.adam = self.adam.with_beta1(beta1)?;
        Ok(self)
    }

    /// Set beta2 with delegation to Adam
    pub fn with_beta2(mut self, beta2: f32) -> Result<Self, String> {
        self.adam = self.adam.with_beta2(beta2)?;
        Ok(self)
    }

    /// Get weight decay value
    pub fn get_weight_decay(&self) -> f32 {
        self.weight_decay
    }

    /// Get inner Adam optimizer reference
    pub fn adam(&self) -> &AdamOptimizer {
        &self.adam
    }
}

impl OptimizerState for AdamWOptimizer {
    fn update(&mut self, gradient: &[f32], lr: f32) -> Vec<f32> {
        let adam_updates = self.adam.update(gradient, lr);

        let mut updates = Vec::with_capacity(adam_updates.len());
        for update in adam_updates {
            let weight_decay_penalty = lr * self.weight_decay;
            updates.push(update + weight_decay_penalty);
        }

        updates
    }

    fn learning_rate(&self) -> f32 {
        self.adam.learning_rate
    }

    fn reset(&mut self) {
        self.adam.reset();
    }

    fn get_state_dict(&self) -> OptimizerCheckpoint {
        let mut checkpoint = self.adam.get_state_dict();
        checkpoint.optimizer_type = "AdamW".to_string();
        checkpoint.weight_decay = Some(self.weight_decay);
        checkpoint
    }

    fn set_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.adam.set_state_dict(checkpoint);
        if let Some(wd) = checkpoint.weight_decay {
            self.weight_decay = wd;
        }
    }
}

impl Optimizer for AdamWOptimizer {
    fn step(&mut self, gradients: &[f32]) -> Vec<f32> {
        self.update(gradients, self.adam.learning_rate)
    }

    fn update_learning_rate(&mut self, new_lr: f32) {
        if new_lr > 0.0 && new_lr.is_finite() {
            self.adam.learning_rate = new_lr;
        }
    }

    fn reset_state(&mut self) {
        self.reset();
    }

    fn get_learning_rate(&self) -> f32 {
        self.learning_rate()
    }

    fn state_dict(&self) -> OptimizerCheckpoint {
        self.get_state_dict()
    }

    fn load_state_dict(&mut self, checkpoint: &OptimizerCheckpoint) {
        self.set_state_dict(checkpoint)
    }
}

/// Gradient clipping configuration for training stability
/// Supports both element-wise value clipping and L2 norm clipping
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GradientClipping {
    pub max_norm: Option<f32>,
    pub max_value: Option<f32>,
}

impl GradientClipping {
    /// Create new gradient clipping config
    pub fn new() -> Self {
        GradientClipping {
            max_norm: None,
            max_value: None,
        }
    }

    /// Set maximum L2 norm for gradient vector with validation
    pub fn with_max_norm(mut self, max_norm: f32) -> Result<Self, String> {
        if max_norm <= 0.0 || !max_norm.is_finite() {
            return Err("Max norm must be positive and finite".to_string());
        }
        self.max_norm = Some(max_norm);
        Ok(self)
    }

    /// Set maximum value for individual gradient elements with validation
    pub fn with_max_value(mut self, max_value: f32) -> Result<Self, String> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err("Max value must be positive and finite".to_string());
        }
        self.max_value = Some(max_value);
        Ok(self)
    }

    /// Apply clipping to gradient (element-wise then norm)
    pub fn clip(&self, gradient: &mut [f32]) {
        // Element-wise value clipping
        if let Some(max_value) = self.max_value {
            for g in gradient.iter_mut() {
                if g.abs() > max_value {
                    *g = max_value * g.signum();
                }
            }
        }

        // L2 norm clipping
        if let Some(max_norm) = self.max_norm {
            let norm: f32 = gradient.iter().map(|g| g * g).sum::<f32>().sqrt();
            if norm > max_norm && norm > 0.0 {
                let scale = max_norm / norm;
                for g in gradient.iter_mut() {
                    *g *= scale;
                }
            }
        }
    }

    /// Get L2 norm of gradient
    pub fn compute_norm(gradient: &[f32]) -> f32 {
        gradient.iter().map(|g| g * g).sum::<f32>().sqrt()
    }

    /// Get max absolute value in gradient
    pub fn compute_max_abs(gradient: &[f32]) -> f32 {
        gradient.iter().map(|g| g.abs()).fold(0.0, f32::max)
    }

    /// Check if clipping was applied
    pub fn would_clip(&self, gradient: &[f32]) -> bool {
        if let Some(max_value) = self.max_value {
            if Self::compute_max_abs(gradient) > max_value {
                return true;
            }
        }
        if let Some(max_norm) = self.max_norm {
            if Self::compute_norm(gradient) > max_norm {
                return true;
            }
        }
        false
    }
}

impl Default for GradientClipping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SGD Optimizer Tests
    #[test]
    fn test_sgd_creation() {
        let sgd = SGDOptimizer::new(0.01).unwrap();
        assert_eq!(sgd.get_learning_rate(), 0.01);
    }

    #[test]
    fn test_sgd_invalid_learning_rate() {
        assert!(SGDOptimizer::new(0.0).is_err());
        assert!(SGDOptimizer::new(-0.01).is_err());
        assert!(SGDOptimizer::new(f32::NAN).is_err());
    }

    #[test]
    fn test_sgd_momentum_builder() {
        let sgd = SGDOptimizer::new(0.01)
            .unwrap()
            .with_momentum(0.95)
            .unwrap();
        assert_eq!(sgd.get_momentum(), 0.95);
    }

    #[test]
    fn test_sgd_invalid_momentum() {
        let sgd = SGDOptimizer::new(0.01).unwrap();
        assert!(sgd.with_momentum(-0.1).is_err());
        assert!(sgd.with_momentum(1.1).is_err());
    }

    #[test]
    fn test_sgd_nesterov() {
        let sgd = SGDOptimizer::new(0.01).unwrap().with_nesterov(true);
        assert!(sgd.is_nesterov());
    }

    #[test]
    fn test_sgd_weight_decay() {
        let sgd = SGDOptimizer::new(0.01)
            .unwrap()
            .with_weight_decay(0.0001)
            .unwrap();
        assert_eq!(sgd.get_weight_decay(), 0.0001);
    }

    #[test]
    fn test_sgd_step() {
        let mut sgd = SGDOptimizer::new(0.01).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];
        let updates = sgd.step(&gradient);
        assert_eq!(updates.len(), 3);
        assert!(updates[0] != 0.0);
    }

    #[test]
    fn test_sgd_update_learning_rate() {
        let mut sgd = SGDOptimizer::new(0.01).unwrap();
        sgd.update_learning_rate(0.001);
        assert_eq!(sgd.get_learning_rate(), 0.001);
    }

    #[test]
    fn test_sgd_reset_state() {
        let mut sgd = SGDOptimizer::new(0.01).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];
        sgd.step(&gradient);
        sgd.reset_state();
        assert_eq!(sgd.velocity.len(), 0);
    }

    // Adam Optimizer Tests
    #[test]
    fn test_adam_creation() {
        let adam = AdamOptimizer::new(0.001).unwrap();
        assert_eq!(adam.get_learning_rate(), 0.001);
        assert_eq!(adam.get_beta1(), 0.9);
        assert_eq!(adam.get_beta2(), 0.999);
    }

    #[test]
    fn test_adam_custom_betas() {
        let adam = AdamOptimizer::new(0.001)
            .unwrap()
            .with_beta1(0.95)
            .unwrap()
            .with_beta2(0.99)
            .unwrap();
        assert_eq!(adam.get_beta1(), 0.95);
        assert_eq!(adam.get_beta2(), 0.99);
    }

    #[test]
    fn test_adam_invalid_betas() {
        let adam = AdamOptimizer::new(0.001).unwrap();
        assert!(adam.with_beta1(1.0).is_err());
        assert!(adam.with_beta2(1.0).is_err());
    }

    #[test]
    fn test_adam_epsilon() {
        let adam = AdamOptimizer::new(0.001)
            .unwrap()
            .with_epsilon(1e-7)
            .unwrap();
        assert_eq!(adam.get_epsilon(), 1e-7);
    }

    #[test]
    fn test_adam_step() {
        let mut adam = AdamOptimizer::new(0.001).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];
        let updates = adam.step(&gradient);
        assert_eq!(updates.len(), 3);
        assert_eq!(adam.get_timestep(), 1);
    }

    #[test]
    fn test_adam_multiple_steps() {
        let mut adam = AdamOptimizer::new(0.001).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];
        adam.step(&gradient);
        adam.step(&gradient);
        adam.step(&gradient);
        assert_eq!(adam.get_timestep(), 3);
    }

    // AdamW Optimizer Tests
    #[test]
    fn test_adamw_creation() {
        let adamw = AdamWOptimizer::new(0.001).unwrap();
        assert_eq!(adamw.get_learning_rate(), 0.001);
    }

    #[test]
    fn test_adamw_weight_decay() {
        let adamw = AdamWOptimizer::new(0.001)
            .unwrap()
            .with_weight_decay(0.01)
            .unwrap();
        assert_eq!(adamw.get_weight_decay(), 0.01);
    }

    #[test]
    fn test_adamw_step() {
        let mut adamw = AdamWOptimizer::new(0.001).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];
        let updates = adamw.step(&gradient);
        assert_eq!(updates.len(), 3);
    }

    // Learning Rate Schedule Tests
    #[test]
    fn test_lr_schedule_constant() {
        let schedule = LRSchedule::constant(0.01);
        assert_eq!(schedule.get_lr(0), 0.01);
        assert_eq!(schedule.get_lr(100), 0.01);
        assert_eq!(schedule.get_lr(1000), 0.01);
    }

    #[test]
    fn test_lr_schedule_step_decay() {
        let schedule = LRSchedule::step_decay(0.1, 0.5, 10);
        assert_eq!(schedule.get_lr(0), 0.1);
        assert_eq!(schedule.get_lr(9), 0.1);
        assert!(schedule.get_lr(10) < 0.1); // Should decay
    }

    #[test]
    fn test_lr_schedule_cosine() {
        let schedule = LRSchedule::cosine(0.1, 100);
        let lr_0 = schedule.get_lr(0);
        let lr_50 = schedule.get_lr(50);
        let lr_100 = schedule.get_lr(100);
        assert!(lr_50 < lr_0);
        assert!(lr_100 < lr_50);
    }

    // Gradient Clipping Tests
    #[test]
    fn test_gradient_clipping_value() {
        let clipping = GradientClipping::new()
            .with_max_value(0.5)
            .unwrap();
        let mut grad = vec![0.3, 0.7, 0.2];
        clipping.clip(&mut grad);
        assert_eq!(grad[0], 0.3);
        assert_eq!(grad[1], 0.5);
        assert_eq!(grad[2], 0.2);
    }

    #[test]
    fn test_gradient_clipping_norm() {
        let clipping = GradientClipping::new()
            .with_max_norm(1.0)
            .unwrap();
        let mut grad = vec![0.6, 0.8]; // norm = 1.0
        clipping.clip(&mut grad);
        assert_eq!(grad[0], 0.6);
        assert_eq!(grad[1], 0.8);

        let mut grad2 = vec![1.0, 1.0]; // norm = sqrt(2) > 1.0
        clipping.clip(&mut grad2);
        let norm_after = GradientClipping::compute_norm(&grad2);
        assert!((norm_after - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gradient_clipping_would_clip() {
        let clipping = GradientClipping::new()
            .with_max_value(0.5)
            .unwrap();
        let grad = vec![0.3, 0.7, 0.2];
        assert!(clipping.would_clip(&grad));

        let grad_ok = vec![0.1, 0.2, 0.3];
        assert!(!clipping.would_clip(&grad_ok));
    }

    #[test]
    fn test_gradient_clipping_compute_norm() {
        let grad = vec![3.0, 4.0];
        let norm = GradientClipping::compute_norm(&grad);
        assert_eq!(norm, 5.0);
    }

    // Checkpoint/State Persistence Tests
    #[test]
    fn test_sgd_checkpoint() {
        let sgd = SGDOptimizer::new(0.01).unwrap();
        let checkpoint = sgd.state_dict();
        assert_eq!(checkpoint.optimizer_type, "SGD");
        assert_eq!(checkpoint.learning_rate, 0.01);
    }

    #[test]
    fn test_adam_checkpoint() {
        let adam = AdamOptimizer::new(0.001).unwrap();
        let checkpoint = adam.state_dict();
        assert_eq!(checkpoint.optimizer_type, "Adam");
        assert_eq!(checkpoint.learning_rate, 0.001);
    }

    #[test]
    fn test_adamw_checkpoint() {
        let adamw = AdamWOptimizer::new(0.001)
            .unwrap()
            .with_weight_decay(0.01)
            .unwrap();
        let checkpoint = adamw.state_dict();
        assert_eq!(checkpoint.optimizer_type, "AdamW");
        assert_eq!(checkpoint.weight_decay, Some(0.01));
    }

    #[test]
    fn test_optimizer_trait_implementation() {
        let mut sgd = SGDOptimizer::new(0.01).unwrap();
        let gradient = vec![0.1, 0.2, 0.3];

        // Test all Optimizer trait methods
        let updates = sgd.step(&gradient);
        assert_eq!(updates.len(), 3);

        sgd.update_learning_rate(0.005);
        assert_eq!(sgd.get_learning_rate(), 0.005);

        sgd.reset_state();
        assert_eq!(sgd.velocity.len(), 0);

        let checkpoint = sgd.state_dict();
        sgd.load_state_dict(&checkpoint);
    }
}
