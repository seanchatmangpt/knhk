// Phase 6: Neural Model with Generic Associated Types (GATs)
// Hyper-advanced Rust: Lifetime-dependent traits for learned models

use ndarray::{Array1, Array2};

/// Generic Associated Types enable lifetime-dependent trait methods
/// This allows us to return references to internal state without lifetime issues
pub trait NeuralModel: Clone + Send + Sync {
    /// Input type (can depend on lifetime 'a)
    type Input<'a>: Clone where Self: 'a;

    /// Output type (can depend on lifetime 'a)
    type Output<'a> where Self: 'a;

    /// Gradient type for backprop
    type Gradient<'a> where Self: 'a;

    /// Forward pass: compute output from input
    fn forward(&self, input: Self::Input<'_>) -> Self::Output<'_>;

    /// Backward pass: update weights based on gradient
    fn backward(&mut self, gradient: Self::Gradient<'_>) -> f32;

    /// Get learnable parameters
    fn parameters(&self) -> Vec<Array1<f32>>;

    /// Update parameters
    fn update_parameters(&mut self, deltas: Vec<Array1<f32>>);
}

/// Const generic dense layer: IN inputs → OUT outputs
#[derive(Clone)]
pub struct DenseLayer<const IN: usize, const OUT: usize> {
    // Weights: [OUT x IN] matrix
    weights: Array2<f32>,
    // Biases: [OUT] vector
    biases: Array1<f32>,
    // Last input for backprop
    last_input: Option<Array1<f32>>,
    // Learning rate
    learning_rate: f32,
}

impl<const IN: usize, const OUT: usize> DenseLayer<IN, OUT> {
    pub fn new() -> Self {
        Self::with_learning_rate(0.001)
    }

    pub fn with_learning_rate(lr: f32) -> Self {
        DenseLayer {
            weights: Array2::zeros((OUT, IN)),
            biases: Array1::zeros(OUT),
            last_input: None,
            learning_rate: lr,
        }
    }

    pub fn initialize_xavier(&mut self) {
        // Xavier initialization: scale by 1/sqrt(input_size)
        let scale = 1.0 / (IN as f32).sqrt();
        self.weights = Array2::from_shape_fn((OUT, IN), |_| {
            (rand::random::<f32>() - 0.5) * 2.0 * scale
        });
        self.biases.fill(0.0);
    }

    pub fn input_size(&self) -> usize { IN }
    pub fn output_size(&self) -> usize { OUT }

    /// ReLU activation
    fn relu(x: f32) -> f32 {
        if x > 0.0 { x } else { 0.0 }
    }

    /// ReLU derivative
    fn relu_prime(x: f32) -> f32 {
        if x > 0.0 { 1.0 } else { 0.0 }
    }
}

impl<const IN: usize, const OUT: usize> Default for DenseLayer<IN, OUT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const IN: usize, const OUT: usize> NeuralModel for DenseLayer<IN, OUT> {
    // Input: borrowed array (can depend on lifetime)
    type Input<'a> = &'a Array1<f32>;

    // Output: array owned by the lifetime (computed)
    type Output<'a> = Array1<f32>;

    // Gradient: borrowed
    type Gradient<'a> = &'a Array1<f32>;

    fn forward(&self, input: Self::Input<'_>) -> Self::Output<'_> {
        // Linear: y = Wx + b
        let mut output = self.weights.dot(input) + &self.biases;

        // ReLU activation
        output.mapv_inplace(Self::relu);

        output
    }

    fn backward(&mut self, gradient: Self::Gradient<'_>) -> f32 {
        if let Some(input) = &self.last_input {
            // Compute weight gradient: ∇W = gradient ⊗ input
            let weight_gradient = gradient.view().into_shape((OUT, 1)).unwrap()
                .dot(&input.view().into_shape((1, IN)).unwrap());

            // Update weights: W ← W - learning_rate * ∇W
            self.weights -= &(weight_gradient * self.learning_rate);

            // Update biases: b ← b - learning_rate * gradient
            self.biases -= &(gradient * self.learning_rate);

            // Compute loss (L2 norm of gradient)
            let loss = gradient.dot(gradient).sqrt();
            loss
        } else {
            0.0
        }
    }

    fn parameters(&self) -> Vec<Array1<f32>> {
        vec![
            self.weights.view().into_shape(IN * OUT).unwrap().to_owned(),
            self.biases.clone(),
        ]
    }

    fn update_parameters(&mut self, deltas: Vec<Array1<f32>>) {
        if !deltas.is_empty() {
            let w_delta = deltas[0].view().into_shape((OUT, IN)).unwrap();
            self.weights -= &w_delta;
        }
        if deltas.len() > 1 {
            self.biases -= &deltas[1];
        }
    }
}

/// Sequential model: stack of dense layers
/// Note: Sequential composition is better handled with concrete types or recursion
/// rather than trait objects, due to GAT limitations with dyn dispatch
pub struct SequentialModel<const L1: usize, const L2: usize, const L3: usize> {
    layer1: DenseLayer<L1, L2>,
    layer2: DenseLayer<L2, L3>,
}

impl<const L1: usize, const L2: usize, const L3: usize> SequentialModel<L1, L2, L3> {
    pub fn new() -> Self {
        SequentialModel {
            layer1: DenseLayer::new(),
            layer2: DenseLayer::new(),
        }
    }

    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        let intermediate = Layer::forward(&self.layer1, input);
        Layer::forward(&self.layer2, &intermediate)
    }
}

impl<const L1: usize, const L2: usize, const L3: usize> Default for SequentialModel<L1, L2, L3> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for layers (simpler than NeuralModel)
pub trait Layer: Clone + Send + Sync {
    fn forward(&self, input: &Array1<f32>) -> Array1<f32>;
    fn backward(&mut self, gradient: &Array1<f32>) -> f32;
}

impl<const IN: usize, const OUT: usize> Layer for DenseLayer<IN, OUT> {
    fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        NeuralModel::forward(self, input)
    }

    fn backward(&mut self, gradient: &Array1<f32>) -> f32 {
        NeuralModel::backward(self, gradient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_layer_forward() {
        let layer: DenseLayer<5, 3> = DenseLayer::new();
        let input = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        let output = NeuralModel::forward(&layer, &input);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_dense_layer_xavier_init() {
        let mut layer: DenseLayer<10, 20> = DenseLayer::new();
        layer.initialize_xavier();

        // Weights should be initialized (non-zero with high probability)
        let sum: f32 = layer.weights.iter().sum();
        assert!(sum.abs() > 0.1, "Xavier init should create non-trivial weights");
    }

    #[test]
    fn test_parameter_retrieval() {
        let layer: DenseLayer<5, 3> = DenseLayer::new();
        let params = layer.parameters();
        assert_eq!(params.len(), 2); // weights + biases
    }
}
