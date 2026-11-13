use super::softmax;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Model API trait for attack implementations
pub trait ModelAPI {
    /// Make prediction given input
    fn predict(&self, input: &[f32]) -> Vec<f32>;
    /// Compute gradients given input and label
    fn compute_gradients(&self, input: &[f32], label: usize) -> Vec<f32>;
    /// Get model dimension
    fn input_dimension(&self) -> usize;
    /// Get output dimension (number of classes)
    fn output_dimension(&self) -> usize;
}

/// Simple model implementation for testing
#[derive(Debug, Clone)]
pub struct SimpleModel {
    /// weights.
    pub weights: Vec<f32>,
    /// biases.
    pub biases: Vec<f32>,
    /// input dim.
    pub input_dim: usize,
    /// output dim.
    pub output_dim: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimpleModel {
    /// Create new simple linear model
    #[must_use]
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        Self {
            weights: vec![0.01; input_dim * output_dim],
            biases: vec![0.0; output_dim],
            input_dim,
            output_dim,
        }
    }

    /// Set weights from flattened vector
    pub fn set_weights(&mut self, weights: &[f32]) {
        if weights.len() == self.weights.len() {
            self.weights.copy_from_slice(weights);
        }
    }

    /// Get weights as flattened vector
    #[must_use]
    pub fn get_weights(&self) -> Vec<f32> {
        self.weights.clone()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Train on single sample (one SGD step)
    #[allow(clippy::needless_range_loop)]
    pub fn train_step(&mut self, input: &[f32], target_label: usize, lr: f32) {
        let prediction = self.predict(input);

        // Compute gradient for target class
        for j in 0..self.output_dim {
            let target = if j == target_label { 1.0 } else { 0.0 };
            let error = prediction[j] - target;

            // Update bias
            self.biases[j] -= lr * error;

            // Update weights
            for (i, &input_val) in input.iter().enumerate() {
                let idx = i * self.output_dim + j;
                self.weights[idx] -= lr * error * input_val;
            }
        }
    }
}

impl Default for SimpleModel {
    fn default() -> Self {
        Self::new(10, 5)
    }
}

impl ModelAPI for SimpleModel {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[allow(clippy::needless_range_loop)]
    fn predict(&self, input: &[f32]) -> Vec<f32> {
        let mut output = self.biases.clone();

        for (i, &input_val) in input.iter().enumerate() {
            for j in 0..self.output_dim {
                let idx = i * self.output_dim + j;
                if idx < self.weights.len() {
                    output[j] += self.weights[idx] * input_val;
                }
            }
        }

        // Apply softmax
        softmax(&mut output);
        output
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_gradients(&self, input: &[f32], label: usize) -> Vec<f32> {
        let prediction = self.predict(input);
        let mut gradients = vec![0.0; self.weights.len() + self.output_dim];

        // Gradient for biases
        for j in 0..self.output_dim {
            let target = if j == label { 1.0 } else { 0.0 };
            let error = prediction[j] - target;
            gradients[j] = error;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Gradient for weights
        for (i, &input_val) in input.iter().enumerate() {
            for j in 0..self.output_dim {
                let target = if j == label { 1.0 } else { 0.0 };
                let error = prediction[j] - target;
                let idx = self.output_dim + i * self.output_dim + j;
                if idx < gradients.len() {
                    gradients[idx] = error * input_val;
                }
            }
        }

        gradients
    }

    fn input_dimension(&self) -> usize {
        self.input_dim
    }

    fn output_dimension(&self) -> usize {
        self.output_dim
    }
}
