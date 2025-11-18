// Example: Neural Network for Duration Prediction
//
// This example demonstrates how to train a simple feedforward
// neural network to predict workflow task duration.

use std::f32;

// Dense layer with ReLU activation
pub struct DenseLayer {
    weights: Vec<Vec<f32>>,  // [output_size][input_size]
    biases: Vec<f32>,        // [output_size]
    last_input: Option<Vec<f32>>,
    last_output: Option<Vec<f32>>,
}

impl DenseLayer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        // Xavier initialization
        let limit = (6.0 / (input_size + output_size) as f32).sqrt();

        let mut weights = Vec::with_capacity(output_size);
        for _ in 0..output_size {
            let row: Vec<f32> = (0..input_size)
                .map(|_| fastrand::f32() * 2.0 * limit - limit)
                .collect();
            weights.push(row);
        }

        let biases = vec![0.0; output_size];

        Self {
            weights,
            biases,
            last_input: None,
            last_output: None,
        }
    }

    pub fn forward(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = self.biases.clone();

        for (i, row) in self.weights.iter().enumerate() {
            let mut sum = 0.0;
            for (j, &w) in row.iter().enumerate() {
                sum += w * input[j];
            }
            output[i] += sum;
        }

        // Cache for backprop
        self.last_input = Some(input.to_vec());
        self.last_output = Some(output.clone());

        output
    }

    pub fn backward(&mut self, grad_output: &[f32], learning_rate: f32) -> Vec<f32> {
        let input = self.last_input.as_ref().unwrap();
        let mut grad_input = vec![0.0; input.len()];

        // Update weights and biases
        for (i, row) in self.weights.iter_mut().enumerate() {
            for (j, w) in row.iter_mut().enumerate() {
                // ∂L/∂w = ∂L/∂out × input
                let grad_w = grad_output[i] * input[j];
                *w -= learning_rate * grad_w;

                // Accumulate gradient w.r.t. input
                grad_input[j] += grad_output[i] * *w;
            }

            // Update bias
            self.biases[i] -= learning_rate * grad_output[i];
        }

        grad_input
    }
}

// ReLU activation
pub fn relu(x: &mut [f32]) {
    for val in x.iter_mut() {
        *val = val.max(0.0);
    }
}

// ReLU derivative
pub fn relu_derivative(x: &[f32]) -> Vec<f32> {
    x.iter().map(|&v| if v > 0.0 { 1.0 } else { 0.0 }).collect()
}

// Simple feedforward network
pub struct DurationPredictor {
    layer1: DenseLayer,  // 10 -> 16
    layer2: DenseLayer,  // 16 -> 8
    layer3: DenseLayer,  // 8 -> 1
}

impl DurationPredictor {
    pub fn new() -> Self {
        Self {
            layer1: DenseLayer::new(10, 16),
            layer2: DenseLayer::new(16, 8),
            layer3: DenseLayer::new(8, 1),
        }
    }

    /// Predict duration (forward pass)
    pub fn predict(&mut self, features: &[f32; 10]) -> f32 {
        // Layer 1
        let mut h1 = self.layer1.forward(features);
        relu(&mut h1);

        // Layer 2
        let mut h2 = self.layer2.forward(&h1);
        relu(&mut h2);

        // Layer 3 (output)
        let output = self.layer3.forward(&h2);

        output[0]
    }

    /// Train on single sample (SGD)
    pub fn train_step(&mut self, features: &[f32; 10], target: f32, learning_rate: f32) -> f32 {
        // Forward pass
        let mut h1 = self.layer1.forward(features);
        let h1_pre_relu = h1.clone();
        relu(&mut h1);

        let mut h2 = self.layer2.forward(&h1);
        let h2_pre_relu = h2.clone();
        relu(&mut h2);

        let output = self.layer3.forward(&h2);
        let prediction = output[0];

        // Loss: MSE
        let error = prediction - target;
        let loss = error * error;

        // Backward pass
        // ∂L/∂output = 2 × (prediction - target)
        let grad_output = vec![2.0 * error];

        // Backprop through layer 3
        let grad_h2 = self.layer3.backward(&grad_output, learning_rate);

        // Apply ReLU derivative
        let grad_h2_pre_relu: Vec<f32> = grad_h2.iter()
            .zip(&h2_pre_relu)
            .map(|(g, h)| if *h > 0.0 { *g } else { 0.0 })
            .collect();

        // Backprop through layer 2
        let grad_h1 = self.layer2.backward(&grad_h2_pre_relu, learning_rate);

        // Apply ReLU derivative
        let grad_h1_pre_relu: Vec<f32> = grad_h1.iter()
            .zip(&h1_pre_relu)
            .map(|(g, h)| if *h > 0.0 { *g } else { 0.0 })
            .collect();

        // Backprop through layer 1
        self.layer1.backward(&grad_h1_pre_relu, learning_rate);

        loss
    }

    /// Train on batch
    pub fn train(&mut self, data: &[([f32; 10], f32)], epochs: usize, learning_rate: f32) {
        for epoch in 0..epochs {
            let mut total_loss = 0.0;

            for (features, target) in data {
                let loss = self.train_step(features, *target, learning_rate);
                total_loss += loss;
            }

            let avg_loss = total_loss / data.len() as f32;

            if epoch % 100 == 0 {
                println!("Epoch {}: Loss = {:.4}", epoch, avg_loss);
            }
        }
    }

    /// Evaluate on test set
    pub fn evaluate(&mut self, data: &[([f32; 10], f32)]) -> f32 {
        let mut total_error = 0.0;

        for (features, target) in data {
            let prediction = self.predict(features);
            let error = (prediction - target).abs() / target;
            total_error += error;
        }

        total_error / data.len() as f32
    }
}

// Example usage
fn main() {
    println!("Neural Network Duration Predictor Example\n");

    // Generate synthetic training data
    let mut training_data = Vec::new();
    for _ in 0..1000 {
        let features: [f32; 10] = std::array::from_fn(|_| fastrand::f32());

        // Simulate duration: weighted sum of features + noise
        let duration = features.iter().sum::<f32>() * 100.0 + fastrand::f32() * 10.0;

        training_data.push((features, duration));
    }

    // Split train/test (80/20)
    let split = (training_data.len() as f32 * 0.8) as usize;
    let (train, test) = training_data.split_at(split);

    // Create and train model
    let mut model = DurationPredictor::new();
    println!("Training...");
    model.train(train, 1000, 0.001);

    // Evaluate
    println!("\nEvaluating on test set...");
    let avg_error = model.evaluate(test);
    println!("Average error: {:.2}%", avg_error * 100.0);

    // Test prediction
    println!("\nExample predictions:");
    for i in 0..5 {
        let (features, actual) = &test[i];
        let predicted = model.predict(features);
        println!("Actual: {:.2}, Predicted: {:.2}, Error: {:.2}%",
                 actual, predicted, ((predicted - actual).abs() / actual) * 100.0);
    }
}
