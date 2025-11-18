# Neural Integration Algorithms Specification

**Version**: 1.0.0
**Date**: 2025-11-18

This document provides detailed algorithmic specifications for all learning algorithms in Phase 6.

---

## 1. Q-LEARNING

### 1.1 Algorithm

**Temporal Difference Learning (Off-Policy)**

```
Initialize Q(s, a) arbitrarily for all s, a
Initialize ε = 1.0 (exploration rate)

For each episode:
    Initialize state s

    For each step in episode:
        Select action a:
            With probability ε: a = random action (exploration)
            With probability 1-ε: a = argmax_a' Q(s, a') (exploitation)

        Execute action a, observe reward r and next state s'

        Update Q-value:
            Q(s, a) ← Q(s, a) + α[r + γ max_a' Q(s', a') - Q(s, a)]

        s ← s'

        Decay exploration:
            ε ← ε × ε_decay (e.g., 0.995)
            ε ← max(ε, ε_min)  (e.g., 0.01)
```

### 1.2 Hyperparameters

| Parameter | Symbol | Typical Value | Range | Description |
|-----------|--------|---------------|-------|-------------|
| Learning rate | α | 0.1 | 0.01-0.5 | Step size for Q-value updates |
| Discount factor | γ | 0.95 | 0.9-0.99 | Future reward importance |
| Initial epsilon | ε₀ | 1.0 | 0.5-1.0 | Initial exploration rate |
| Epsilon decay | ε_decay | 0.995 | 0.99-0.999 | Exploration decay rate |
| Min epsilon | ε_min | 0.01 | 0.01-0.1 | Minimum exploration rate |

### 1.3 Convergence

**Conditions**:
- Learning rate decays over time: α_t = α₀ / (1 + t)
- All state-action pairs visited infinitely often
- Bounded rewards: |r| ≤ R_max

**Theorem**: Q-learning converges to optimal Q* under these conditions.

### 1.4 Optimizations

**Sparse Q-Table**:
```rust
// Use HashMap instead of 2D array for large state spaces
HashMap<(State, Action), f32>
```

**Eligibility Traces** (Q(λ)):
```
For each s, a:
    e(s, a) ← 0

For each step:
    e(s, a) ← e(s, a) + 1  // Accumulating trace

    δ ← r + γ max_a' Q(s', a') - Q(s, a)  // TD error

    For each s, a:
        Q(s, a) ← Q(s, a) + α δ e(s, a)
        e(s, a) ← γ λ e(s, a)
```

---

## 2. SARSA (On-Policy)

### 2.1 Algorithm

**State-Action-Reward-State-Action**

```
Initialize Q(s, a) arbitrarily for all s, a
Initialize ε = 1.0

For each episode:
    Initialize state s
    Select action a using ε-greedy policy

    For each step in episode:
        Execute action a, observe reward r and next state s'
        Select next action a' using ε-greedy policy

        Update Q-value:
            Q(s, a) ← Q(s, a) + α[r + γ Q(s', a') - Q(s, a)]

        s ← s'
        a ← a'

        Decay ε
```

### 2.2 Difference from Q-Learning

| Aspect | Q-Learning | SARSA |
|--------|------------|-------|
| Policy | Off-policy | On-policy |
| Update | Uses max Q(s', a') | Uses actual Q(s', a') |
| Behavior | More aggressive | More conservative |
| Safety | Can be risky | Safer (learns actual policy) |

### 2.3 When to Use

- **Q-Learning**: When you want optimal policy regardless of exploration
- **SARSA**: When safety matters (learns the policy you actually follow)

**Example**: If exploration causes failures, SARSA learns to avoid risky states even during exploration.

---

## 3. ACTOR-CRITIC

### 3.1 Algorithm

**Policy gradient with value function baseline**

```
Initialize:
    Actor network π(a|s; θ) with parameters θ
    Critic network V(s; φ) with parameters φ

For each episode:
    Initialize state s

    For each step in episode:
        Sample action a ~ π(a|s; θ)
        Execute a, observe reward r and next state s'

        Compute TD error:
            δ ← r + γ V(s'; φ) - V(s; φ)

        Update critic:
            φ ← φ + α_critic δ ∇_φ V(s; φ)

        Update actor:
            θ ← θ + α_actor δ ∇_θ log π(a|s; θ)

        s ← s'
```

### 3.2 Network Architectures

**Actor** (Policy Network):
```
Input: State features (64 dims)
  ↓
Dense(128) + ReLU
  ↓
Dense(64) + ReLU
  ↓
Dense(action_space_size) + Softmax
  ↓
Output: Action probabilities
```

**Critic** (Value Network):
```
Input: State features (64 dims)
  ↓
Dense(128) + ReLU
  ↓
Dense(64) + ReLU
  ↓
Dense(1)
  ↓
Output: State value V(s)
```

### 3.3 Advantage Actor-Critic (A2C)

**Advantage function**: `A(s, a) = Q(s, a) - V(s)`

```
Compute advantage:
    A ← r + γ V(s'; φ) - V(s; φ)  // Same as TD error

Update actor using advantage:
    θ ← θ + α_actor A ∇_θ log π(a|s; θ)
```

**Why advantage?**: Reduces variance in policy gradient.

### 3.4 Asynchronous Advantage Actor-Critic (A3C)

**Parallel training** with multiple workers:

```
Global networks: π_global, V_global

For each worker i in parallel:
    Local networks: π_i, V_i

    For each episode:
        Copy global → local: π_i ← π_global, V_i ← V_global

        Collect trajectory of T steps

        Compute gradients:
            ∇_θ ← Σ_t A_t ∇_θ log π(a_t|s_t)
            ∇_φ ← Σ_t δ_t ∇_φ V(s_t)

        Update global:
            π_global ← π_global + ∇_θ
            V_global ← V_global + ∇_φ
```

---

## 4. EXPERIENCE REPLAY

### 4.1 Prioritized Experience Replay

**Priority**: TD error magnitude (how surprising the transition was)

```
Initialize:
    Buffer D = []
    Priorities P = []

Add transition:
    D.append((s, a, r, s'))
    P.append(|δ|^α)  // δ = TD error, α = priority exponent

Sample batch:
    For i in batch_size:
        Probability of sampling transition j:
            p_j = P_j^α / Σ_k P_k^α

        Sample transition j according to p_j

        Importance weight:
            w_j = (N × p_j)^(-β)  // β anneals from 0.4 to 1.0

        Add (transition_j, w_j) to batch

Update priorities:
    After learning from batch, update P_j = |δ_j|^α
```

### 4.2 Hyperparameters

| Parameter | Symbol | Value | Description |
|-----------|--------|-------|-------------|
| Priority exponent | α | 0.6 | How much prioritization (0=uniform, 1=full) |
| Importance sampling | β | 0.4→1.0 | Bias correction (anneals during training) |
| Buffer capacity | N | 10,000-1M | Maximum transitions stored |
| Batch size | B | 32-256 | Transitions per training step |

### 4.3 Implementation

```rust
pub struct PrioritizedReplayBuffer {
    buffer: VecDeque<Transition>,
    priorities: VecDeque<f32>,
    capacity: usize,
    alpha: f32,
    beta: f32,
    beta_increment: f32,  // 0.00001 per sample
}

impl PrioritizedReplayBuffer {
    pub fn add(&mut self, transition: Transition, priority: f32) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
            self.priorities.pop_front();
        }

        self.buffer.push_back(transition);
        self.priorities.push_back(priority.powf(self.alpha));
    }

    pub fn sample(&mut self, batch_size: usize) -> Vec<(Transition, f32)> {
        let total: f32 = self.priorities.iter().sum();

        let mut batch = Vec::new();
        for _ in 0..batch_size {
            let idx = self.sample_proportional(total);
            let prob = self.priorities[idx] / total;
            let weight = (self.buffer.len() as f32 * prob).powf(-self.beta);

            batch.push((self.buffer[idx].clone(), weight));
        }

        // Anneal beta
        self.beta = (self.beta + self.beta_increment).min(1.0);

        batch
    }

    fn sample_proportional(&self, total: f32) -> usize {
        let mut rng = fastrand::f32();
        let mut cumulative = 0.0;

        for (i, &priority) in self.priorities.iter().enumerate() {
            cumulative += priority / total;
            if rng < cumulative {
                return i;
            }
        }

        self.priorities.len() - 1
    }
}
```

---

## 5. GENETIC ALGORITHM (Descriptor Evolution)

### 5.1 Algorithm

```
Initialize population P of N descriptors randomly

For each generation:
    // 1. Evaluation
    For each descriptor d in P:
        fitness(d) ← evaluate(d, execution_history)

    // 2. Selection
    Sort P by fitness (descending)
    elites ← top 20% of P

    // 3. Crossover + Mutation
    offspring ← []
    For i in (N - elite_count):
        parent1 ← tournament_select(P, k=3)
        parent2 ← tournament_select(P, k=3)

        If random() < crossover_rate:
            child ← crossover(parent1, parent2)
        Else:
            child ← parent1

        child ← mutate(child, mutation_rate)
        offspring.append(child)

    // 4. Replacement
    P ← elites + offspring

    generation ← generation + 1
```

### 5.2 Genetic Operators

**Mutation** (Gaussian for continuous genes):
```rust
fn mutate_continuous(value: f32, mutation_rate: f32, sigma: f32) -> f32 {
    if fastrand::f32() < mutation_rate {
        let noise = gaussian_random(0.0, sigma);  // N(0, σ)
        value + noise
    } else {
        value
    }
}
```

**Mutation** (Discrete for integer genes):
```rust
fn mutate_discrete(value: u8, mutation_rate: f32, delta: i8) -> u8 {
    if fastrand::f32() < mutation_rate {
        (value as i8 + fastrand::i8(-delta..=delta)).clamp(0, 255) as u8
    } else {
        value
    }
}
```

**Crossover** (Single-point):
```rust
fn crossover(parent1: &Genome, parent2: &Genome) -> Genome {
    let mut child = parent1.clone();
    let crossover_point = fastrand::usize(0..parent1.genes.len());

    for i in crossover_point..parent1.genes.len() {
        child.genes[i] = parent2.genes[i];
    }

    child
}
```

**Crossover** (Uniform):
```rust
fn crossover_uniform(parent1: &Genome, parent2: &Genome) -> Genome {
    let mut child = parent1.clone();

    for i in 0..parent1.genes.len() {
        if fastrand::bool() {
            child.genes[i] = parent2.genes[i];
        }
    }

    child
}
```

### 5.3 Selection Strategies

**Tournament Selection**:
```rust
fn tournament_select(population: &[Genome], k: usize) -> &Genome {
    let mut best: Option<&Genome> = None;

    for _ in 0..k {
        let candidate = &population[fastrand::usize(0..population.len())];
        if best.map_or(true, |b| candidate.fitness > b.fitness) {
            best = Some(candidate);
        }
    }

    best.unwrap()
}
```

**Roulette Wheel Selection**:
```rust
fn roulette_select(population: &[Genome]) -> &Genome {
    let total_fitness: f32 = population.iter().map(|g| g.fitness).sum();
    let mut rng = fastrand::f32() * total_fitness;

    for genome in population {
        rng -= genome.fitness;
        if rng <= 0.0 {
            return genome;
        }
    }

    &population[population.len() - 1]
}
```

### 5.4 Fitness Function

**Multi-objective optimization**:

```rust
fn fitness(genome: &Genome, history: &ExecutionHistory) -> f32 {
    let executions = history.get_for_genome(genome);

    if executions.is_empty() {
        return 0.0;
    }

    // Compute metrics
    let avg_duration = mean(executions.iter().map(|e| e.duration_ms));
    let success_rate = executions.iter().filter(|e| e.success).count() as f32
                       / executions.len() as f32;
    let avg_cost = mean(executions.iter().map(|e| e.cost));
    let p99_latency = percentile(executions.iter().map(|e| e.duration_ms), 0.99);

    // Normalize (0-1)
    let duration_score = 1.0 - (avg_duration / history.max_duration());
    let success_score = success_rate;
    let cost_score = 1.0 - (avg_cost / history.max_cost());
    let latency_score = 1.0 - (p99_latency / history.max_p99());

    // Weighted sum
    0.3 * duration_score
        + 0.4 * success_score
        + 0.2 * cost_score
        + 0.1 * latency_score
}
```

### 5.5 Hyperparameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Population size | 100 | Number of genomes |
| Elite count | 20 | Top 20% preserved |
| Crossover rate | 0.8 | Probability of crossover |
| Mutation rate | 0.1 | Probability per gene |
| Tournament size | 3 | For tournament selection |
| Generations | 100+ | Evolution iterations |

---

## 6. NEURAL NETWORK TRAINING

### 6.1 Gradient Descent

**Batch Gradient Descent**:
```
For each epoch:
    For each batch (x_i, y_i) in dataset:
        // Forward pass
        ŷ_i = model(x_i)

        // Compute loss
        L = loss_function(ŷ_i, y_i)

        // Backward pass
        ∇L = gradient(L, model.parameters)

        // Update parameters
        θ ← θ - α ∇L
```

**Adam Optimizer** (recommended):
```
Initialize:
    m = 0  // First moment (mean)
    v = 0  // Second moment (variance)
    t = 0  // Time step

For each gradient ∇L:
    t ← t + 1

    // Update moments
    m ← β₁ m + (1 - β₁) ∇L
    v ← β₂ v + (1 - β₂) ∇L²

    // Bias correction
    m̂ ← m / (1 - β₁^t)
    v̂ ← v / (1 - β₂^t)

    // Update parameters
    θ ← θ - α m̂ / (√v̂ + ε)
```

**Hyperparameters**:
- α = 0.001 (learning rate)
- β₁ = 0.9 (momentum)
- β₂ = 0.999 (RMSprop)
- ε = 1e-8 (numerical stability)

### 6.2 Loss Functions

**Regression** (Duration Prediction):
```
MSE = (1/N) Σ (ŷ_i - y_i)²
```

**Binary Classification** (Success Prediction):
```
Binary Cross-Entropy = -(1/N) Σ [y_i log(ŷ_i) + (1 - y_i) log(1 - ŷ_i)]
```

**Autoencoder** (Anomaly Detection):
```
Reconstruction Loss = MSE(x, decoder(encoder(x)))
```

### 6.3 Regularization

**L2 Regularization** (Weight Decay):
```
L_total = L_data + λ Σ θ²

Gradient: ∇L_total = ∇L_data + 2λθ
```

**Dropout** (During Training):
```
For each layer:
    With probability p, set neuron output to 0
    Scale remaining outputs by 1/(1-p)
```

**Early Stopping**:
```
Monitor validation loss
If no improvement for N epochs:
    Stop training
    Restore best model
```

---

## 7. ANOMALY DETECTION (Autoencoder)

### 7.1 Training

```
Initialize encoder and decoder networks

For each epoch:
    For each normal sample x:
        // Encode
        z = encoder(x)  // Bottleneck representation

        // Decode
        x̂ = decoder(z)  // Reconstruction

        // Loss: Reconstruction error
        L = MSE(x, x̂)

        // Backprop through decoder then encoder
        gradients = ∇L
        update(decoder, gradients)
        update(encoder, gradients)
```

### 7.2 Detection

```
For each new sample x:
    // Reconstruct
    x̂ = decoder(encoder(x))

    // Compute error
    error = MSE(x, x̂)

    // Compare to threshold
    is_anomaly = (error > threshold)
```

### 7.3 Threshold Selection

**Method 1**: 95th percentile of training errors
```
errors = [MSE(x, decoder(encoder(x))) for x in training_set]
threshold = percentile(errors, 0.95)
```

**Method 2**: Mean + 3 standard deviations
```
threshold = mean(errors) + 3 × std(errors)
```

---

## 8. FEDERATED LEARNING

### 8.1 FedAvg Algorithm

```
// Server
Initialize global model θ_global

For each round t:
    // Sample clients
    clients ← random_sample(all_clients, fraction=0.1)

    // Send global model to clients
    For each client k in clients:
        send(θ_global) to client k

    // Clients train locally
    For each client k in parallel:
        θ_k ← local_train(θ_global, local_data_k)
        send(θ_k) back to server

    // Aggregate models
    θ_global ← Σ_k (n_k / n_total) θ_k
    // n_k = number of samples at client k
```

### 8.2 Aggregation for Q-Learning

```
// Each workflow learns locally
For each workflow w:
    Q_local_w ← learn from executions

// Periodically synchronize
If sync_interval reached:
    // Aggregate Q-tables
    For each (state, action) pair:
        Q_global(s, a) ← mean([Q_local_w(s, a) for all w])

    // Broadcast back
    For each workflow w:
        Q_local_w ← Q_global
```

---

## 9. PERFORMANCE OPTIMIZATIONS

### 9.1 Batch Inference

```rust
// Instead of:
for input in inputs {
    output = model.forward(input);  // 1000 calls
}

// Do this:
outputs = model.forward_batch(inputs);  // 1 call, vectorized
```

### 9.2 Model Quantization

**Float32 → Int8**:
```
scale = (max_weight - min_weight) / 255
zero_point = round(-min_weight / scale)

quantized_weight = round(weight / scale) + zero_point

// Dequantize for inference
weight ≈ scale × (quantized_weight - zero_point)
```

**Benefits**: 4x memory reduction, faster inference

### 9.3 Caching

```rust
pub struct CachedPredictor {
    model: DurationPredictor,
    cache: LruCache<WorkflowFeatures, f32>,
}

impl CachedPredictor {
    pub fn predict(&mut self, features: &WorkflowFeatures) -> f32 {
        if let Some(&cached) = self.cache.get(features) {
            return cached;
        }

        let prediction = self.model.predict(features);
        self.cache.put(features.clone(), prediction);
        prediction
    }
}
```

---

## COMPLEXITY ANALYSIS

### Time Complexity

| Algorithm | Training (per step) | Inference | Memory |
|-----------|-------------------|-----------|--------|
| Q-Learning | O(1) | O(1) | O(\|S\| × \|A\|) |
| Neural Network | O(n × m) | O(n × m) | O(n × m) |
| Genetic Algorithm | O(N × f) | O(f) | O(N × g) |
| Experience Replay | O(log N) | O(1) | O(N) |

Where:
- |S| = state space size
- |A| = action space size
- n, m = network layer sizes
- N = population/buffer size
- f = fitness evaluation time
- g = genome size

### Space Complexity

**Q-Table**: Sparse storage for large state spaces
- Worst: O(10^12) entries (impractical)
- Actual: O(10^6) entries (only visited states)

**Neural Networks**: Dense storage
- 3-layer (64→128→64→1): ~16K parameters = 64KB

**Experience Replay**: Fixed-size circular buffer
- 1M transitions × 256 bytes = 256MB

---

## CONVERGENCE GUARANTEES

### Q-Learning

**Theorem** (Watkins & Dayan, 1992):
- If all state-action pairs visited infinitely often
- Learning rate decays: Σ α_t = ∞, Σ α_t² < ∞
- Then: Q_t → Q* with probability 1

### Genetic Algorithm

**Schema Theorem** (Holland, 1975):
- Short, low-order, above-average schemas (building blocks)
  receive exponentially increasing trials over time

**No Free Lunch** (Wolpert & Macready, 1997):
- No single algorithm optimal for all problems
- Tailoring to problem domain is essential

### Neural Networks

**Universal Approximation Theorem**:
- A feedforward network with ≥1 hidden layer
  can approximate any continuous function
  to arbitrary precision (given enough neurons)

---

## CONCLUSION

These algorithms form the foundation of Phase 6's learning capabilities. Each algorithm has been chosen for specific properties:

- **Q-Learning**: Simple, proven, efficient
- **SARSA**: Safe, on-policy learning
- **Actor-Critic**: Continuous action spaces
- **Genetic Algorithms**: Descriptor optimization
- **Autoencoders**: Unsupervised anomaly detection

All algorithms respect the **Chatman constant** (≤8 ticks) by:
1. Performing learning **offline** (background threads)
2. Using **cached inference** for hot path
3. **Batching** updates for efficiency

Validation via **Weaver schemas** ensures all learning decisions are observable, maintaining alignment with **DOCTRINE_2027**.
