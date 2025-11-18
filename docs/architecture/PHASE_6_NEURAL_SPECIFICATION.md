# Phase 6: Neural Integration Layer - Detailed Specification

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18
**Phase Duration**: 8 weeks | **LOC Estimate**: ~12,000 lines

---

## DOCTRINE Alignment

**Principle**: MAPE-K (Analyze Stage) - "Plan → Do → Review → Adjust" at machine speed
**Covenant**: Covenant 3 (Feedback Loops Run at Machine Speed)
**Why This Matters**: Neural learning accelerates the autonomic cycle by learning optimal policies from workflow execution history.

**What This Means**:
The Analyze stage of MAPE-K currently uses statistical methods and rule-based anomaly detection. Phase 6 adds reinforcement learning algorithms that learn optimal workflow configurations, resource allocation policies, and execution strategies from accumulated observations (O).

**Anti-Patterns to Avoid**:
- ❌ Neural models blocking the hot path (must be async)
- ❌ Training without telemetry validation (Weaver must verify)
- ❌ Unbounded training loops (must respect Chatman constant)
- ❌ Models without CPU fallback (degradation required)
- ❌ Learning without validation (false positive trap)

---

## Architecture Overview

### Core Components

```
┌────────────────────────────────────────────────────────────────┐
│                  Phase 6: Neural Integration                    │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │             Neural Model Trait Hierarchy                 │   │
│  │                                                           │   │
│  │  trait NeuralModel<I, O> {                              │   │
│  │    type Error;                                          │   │
│  │    fn predict(&self, input: I) -> Result<O, Error>;    │   │
│  │    fn train(&mut self, batch: &[Experience<I, O>]);    │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Q-Learning  │  │    SARSA     │  │ Actor-Critic │         │
│  │              │  │              │  │              │         │
│  │ • Discrete   │  │ • On-policy  │  │ • Continuous │         │
│  │ • Off-policy │  │ • TD(0)      │  │ • Policy grad│         │
│  │ • ε-greedy   │  │ • Safe       │  │ • Advantage  │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Experience Replay Buffer                    │   │
│  │                                                           │   │
│  │  • Priority sampling (TD-error weighted)                │   │
│  │  • Circular buffer (fixed capacity)                     │   │
│  │  • Thread-safe (RwLock)                                 │   │
│  │  • Persistence (sled database)                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │         Multi-Agent Coordination (Optional)              │   │
│  │                                                           │   │
│  │  • Shared reward signals                                │   │
│  │  • Consensus-backed policy sync (Phase 8)               │   │
│  │  • Distributed training (Parameter server)              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │          MAPE-K Integration (Analyze Stage)              │   │
│  │                                                           │   │
│  │  Monitor → Analyze (Neural) → Plan → Execute → Knowledge│   │
│  │              ↓                                            │   │
│  │      State: Workflow metrics                             │   │
│  │      Action: Config change                               │   │
│  │      Reward: Performance improvement                     │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

---

## Core Trait Definitions

### 1. NeuralModel Trait (GAT-based)

```rust
/// Generic neural model trait with flexible input/output types
///
/// Uses Generic Associated Types (GATs) to support:
/// - Lifetime-dependent borrowing
/// - Zero-copy prediction
/// - Type-safe state/action spaces
pub trait NeuralModel {
    /// Input type (state representation)
    type Input<'a> where Self: 'a;

    /// Output type (action or Q-value)
    type Output;

    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Configuration type
    type Config: Default + Clone;

    /// Predict output from input (inference)
    ///
    /// Latency: MUST be ≤8 ticks for hot path
    /// Telemetry: Emits "neural.predict" span
    fn predict<'a>(&'a self, input: Self::Input<'a>)
        -> Result<Self::Output, Self::Error>;

    /// Train model from experience batch (off critical path)
    ///
    /// Latency: Async, runs in background
    /// Telemetry: Emits "neural.train" span with loss metrics
    fn train(&mut self, batch: &[Experience<Self::Input<'_>, Self::Output>])
        -> Result<TrainingMetrics, Self::Error>;

    /// Export model for inference (ONNX format)
    fn export(&self) -> Result<Vec<u8>, Self::Error>;

    /// Load model from checkpoint
    fn load(data: &[u8], config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
```

### 2. Experience Replay

```rust
/// Experience tuple for reinforcement learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience<S, A> {
    /// Current state
    pub state: S,
    /// Action taken
    pub action: A,
    /// Reward received
    pub reward: f64,
    /// Next state
    pub next_state: S,
    /// Episode terminated
    pub done: bool,
    /// TD-error (for priority sampling)
    pub td_error: f64,
}

/// Priority replay buffer with thread-safe access
pub struct ReplayBuffer<S, A> {
    /// Circular buffer
    buffer: RwLock<VecDeque<Experience<S, A>>>,
    /// Max capacity
    capacity: usize,
    /// Persistent storage
    db: Option<sled::Db>,
    /// Priority sampling enabled
    prioritized: bool,
}

impl<S, A> ReplayBuffer<S, A>
where
    S: Serialize + DeserializeOwned + Clone,
    A: Serialize + DeserializeOwned + Clone,
{
    /// Sample batch with priority weighting
    ///
    /// Latency: O(n log k) where k = batch_size
    /// Telemetry: "replay.sample" span
    pub fn sample(&self, batch_size: usize) -> Vec<Experience<S, A>> {
        // Implementation: Priority sampling based on TD-error
        unimplemented!("Priority sampling with TD-error weights")
    }

    /// Add experience to buffer
    ///
    /// Latency: O(1) amortized
    /// Telemetry: "replay.push" event
    pub fn push(&self, experience: Experience<S, A>) {
        unimplemented!("Circular buffer insertion")
    }
}
```

### 3. Reinforcement Learning Algorithms

#### Q-Learning (Off-Policy, Discrete Actions)

```rust
/// Q-Learning agent for discrete action spaces
///
/// Algorithm: Q(s,a) ← Q(s,a) + α[r + γ max Q(s',a') - Q(s,a)]
/// Properties:
/// - Off-policy (learns from any experience)
/// - Discrete actions only
/// - ε-greedy exploration
pub struct QLearningAgent<S> {
    /// Q-table: State × Action → Value
    q_table: HashMap<StateActionPair<S>, f64>,
    /// Learning rate (α)
    alpha: f64,
    /// Discount factor (γ)
    gamma: f64,
    /// Exploration rate (ε)
    epsilon: f64,
    /// Replay buffer
    replay: ReplayBuffer<S, usize>,
}

impl<S> NeuralModel for QLearningAgent<S>
where
    S: Hash + Eq + Clone + Serialize + DeserializeOwned,
{
    type Input<'a> = &'a S where S: 'a;
    type Output = usize; // Action index
    type Error = NeuralError;
    type Config = QLearningConfig;

    #[instrument(skip(self))]
    fn predict<'a>(&'a self, state: &'a S) -> Result<usize, NeuralError> {
        // ε-greedy action selection
        if rand::random::<f64>() < self.epsilon {
            // Explore: random action
            Ok(rand::random())
        } else {
            // Exploit: best Q-value
            let q_values = self.get_q_values(state);
            Ok(argmax(&q_values))
        }
    }

    #[instrument(skip(self, batch))]
    fn train(&mut self, batch: &[Experience<Self::Input<'_>, usize>])
        -> Result<TrainingMetrics, NeuralError>
    {
        let mut total_loss = 0.0;

        for exp in batch {
            // Q-learning update
            let current_q = self.q_table
                .get(&StateActionPair(exp.state.clone(), exp.action))
                .copied()
                .unwrap_or(0.0);

            let max_next_q = self.get_q_values(&exp.next_state)
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            let target = exp.reward + self.gamma * max_next_q * (1.0 - exp.done as u8 as f64);
            let td_error = target - current_q;

            // Update Q-value
            self.q_table.insert(
                StateActionPair(exp.state.clone(), exp.action),
                current_q + self.alpha * td_error,
            );

            total_loss += td_error.powi(2);
        }

        Ok(TrainingMetrics {
            loss: total_loss / batch.len() as f64,
            episodes: batch.len(),
            convergence: self.check_convergence(),
        })
    }
}
```

#### SARSA (On-Policy, Safe Learning)

```rust
/// SARSA agent for safe on-policy learning
///
/// Algorithm: Q(s,a) ← Q(s,a) + α[r + γ Q(s',a') - Q(s,a)]
/// Properties:
/// - On-policy (learns from actual behavior)
/// - Conservative (doesn't assume optimal future)
/// - Better for safety-critical workflows
pub struct SARSAAgent<S> {
    /// Q-table: State × Action → Value
    q_table: HashMap<StateActionPair<S>, f64>,
    /// Learning rate (α)
    alpha: f64,
    /// Discount factor (γ)
    gamma: f64,
    /// Current policy (softmax)
    temperature: f64,
}

// Implementation similar to Q-Learning but uses actual next action
// instead of max Q-value
```

#### Actor-Critic (Continuous Actions, Policy Gradient)

```rust
/// Actor-Critic agent for continuous action spaces
///
/// Architecture:
/// - Actor: Maps states → action probabilities (policy network)
/// - Critic: Maps states → value estimates (value network)
///
/// Algorithm:
/// - Actor: ∇J = E[∇log π(a|s) * A(s,a)]
/// - Critic: TD-error = r + γV(s') - V(s)
/// - Advantage: A(s,a) = Q(s,a) - V(s) ≈ TD-error
pub struct ActorCriticAgent {
    /// Policy network (actor)
    actor: PolicyNetwork,
    /// Value network (critic)
    critic: ValueNetwork,
    /// Actor learning rate
    actor_lr: f64,
    /// Critic learning rate
    critic_lr: f64,
    /// Entropy coefficient (exploration)
    entropy_coef: f64,
}

/// Policy network (multi-layer perceptron)
struct PolicyNetwork {
    layers: Vec<Layer>,
}

/// Value network (multi-layer perceptron)
struct ValueNetwork {
    layers: Vec<Layer>,
}

impl NeuralModel for ActorCriticAgent {
    type Input<'a> = &'a [f64]; // State vector
    type Output = Vec<f64>; // Action probabilities
    type Error = NeuralError;
    type Config = ActorCriticConfig;

    #[instrument(skip(self))]
    fn predict<'a>(&'a self, state: &'a [f64]) -> Result<Vec<f64>, NeuralError> {
        // Forward pass through actor network
        let action_logits = self.actor.forward(state)?;

        // Softmax to get probabilities
        let action_probs = softmax(&action_logits);

        Ok(action_probs)
    }

    #[instrument(skip(self, batch))]
    fn train(&mut self, batch: &[Experience<Self::Input<'_>, Vec<f64>>])
        -> Result<TrainingMetrics, NeuralError>
    {
        // Compute advantages using TD-error
        let mut advantages = Vec::with_capacity(batch.len());
        for exp in batch {
            let value = self.critic.forward(exp.state)?[0];
            let next_value = self.critic.forward(&exp.next_state)?[0];
            let td_error = exp.reward + self.gamma * next_value - value;
            advantages.push(td_error);
        }

        // Update critic (value network) using MSE
        let critic_loss = self.update_critic(batch, &advantages)?;

        // Update actor (policy network) using policy gradient
        let actor_loss = self.update_actor(batch, &advantages)?;

        Ok(TrainingMetrics {
            loss: actor_loss + critic_loss,
            episodes: batch.len(),
            convergence: self.check_convergence(),
        })
    }
}
```

---

## MAPE-K Integration

### Workflow State Representation

```rust
/// State representation for workflow optimization
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Number of active tasks
    pub active_tasks: u32,
    /// Average latency (in ticks)
    pub avg_latency: u32,
    /// Resource utilization (0-100)
    pub cpu_util: u8,
    pub mem_util: u8,
    /// Current pattern being executed
    pub pattern: PatternId,
    /// Queue depth
    pub queue_depth: u32,
}

/// Action representation for workflow configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WorkflowAction {
    /// Increase parallelism
    ScaleUp(u8),
    /// Decrease parallelism
    ScaleDown(u8),
    /// Change execution strategy
    SwitchPattern(PatternId),
    /// Adjust queue size
    ResizeQueue(u32),
    /// Enable hardware acceleration
    EnableAccelerator(AcceleratorType),
}

/// Reward function for workflow optimization
pub fn compute_reward(
    prev_state: &WorkflowState,
    action: &WorkflowAction,
    next_state: &WorkflowState,
) -> f64 {
    // Reward components:
    // 1. Latency reduction (primary objective)
    let latency_improvement = (prev_state.avg_latency as f64
        - next_state.avg_latency as f64) / prev_state.avg_latency as f64;

    // 2. Resource efficiency (secondary objective)
    let resource_penalty = (next_state.cpu_util as f64 / 100.0).powi(2) * 0.1;

    // 3. Chatman constant compliance (hard constraint)
    let chatman_penalty = if next_state.avg_latency > 8 {
        -10.0 // Severe penalty for exceeding limit
    } else {
        0.0
    };

    latency_improvement - resource_penalty + chatman_penalty
}
```

### Integration with Analyze Stage

```rust
/// Neural-enhanced MAPE-K Analyze component
pub struct NeuralAnalyzer<M: NeuralModel> {
    /// Neural model (Q-Learning, SARSA, or Actor-Critic)
    model: Arc<Mutex<M>>,
    /// Experience buffer
    replay: Arc<ReplayBuffer<WorkflowState, M::Output>>,
    /// Training thread
    trainer: Option<JoinHandle<()>>,
}

impl<M: NeuralModel> NeuralAnalyzer<M>
where
    M::Input<'_>: From<&WorkflowState>,
    M::Output: Into<WorkflowAction>,
{
    /// Analyze workflow state and recommend action
    ///
    /// Latency: ≤8 ticks (prediction only, training is async)
    /// Telemetry: "mape_k.analyze.neural" span
    #[instrument(skip(self))]
    pub async fn analyze(&self, observations: &[Observation])
        -> Result<WorkflowAction, AnalyzeError>
    {
        // Extract state from observations
        let state = WorkflowState::from_observations(observations)?;

        // Predict action using neural model
        let model = self.model.lock().await;
        let output = model.predict(state.into())?;
        let action = output.into();

        // Record for future training
        self.record_experience(state, action).await;

        Ok(action)
    }

    /// Background training loop
    async fn training_loop(
        model: Arc<Mutex<M>>,
        replay: Arc<ReplayBuffer<WorkflowState, M::Output>>,
    ) {
        loop {
            // Wait for enough experiences
            tokio::time::sleep(Duration::from_secs(60)).await;

            if replay.len() < 64 {
                continue; // Need more data
            }

            // Sample batch
            let batch = replay.sample(32);

            // Train model
            let mut model = model.lock().await;
            match model.train(&batch) {
                Ok(metrics) => {
                    tracing::info!(
                        loss = metrics.loss,
                        episodes = metrics.episodes,
                        "Neural training completed"
                    );
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Neural training failed");
                }
            }
        }
    }
}
```

---

## Performance Constraints

### Latency Budgets

| Operation | Latency | Path | Validation |
|-----------|---------|------|------------|
| `predict()` | ≤8 ticks | Hot | chicago-tdd |
| `sample()` | ≤100 μs | Warm | replay-bench |
| `train()` | Async | Cold | background thread |
| `export()` | ≤1 ms | Warm | export-bench |

### Memory Constraints

- Replay buffer: Max 10,000 experiences (~1 MB per 1K experiences)
- Q-table: Max 100,000 entries (~800 KB for f64 values)
- Neural networks: Max 10 MB per model (ONNX format)

### Chatman Constant Compliance

```rust
#[cfg(test)]
mod chatman_tests {
    use super::*;

    #[test]
    fn test_prediction_latency() {
        let agent = QLearningAgent::new(config);
        let state = WorkflowState::default();

        let start = Instant::now();
        let _action = agent.predict(&state).unwrap();
        let elapsed_ticks = start.elapsed().as_nanos() as u32;

        assert!(
            elapsed_ticks <= 8,
            "Prediction exceeded Chatman constant: {} ticks",
            elapsed_ticks
        );
    }
}
```

---

## OpenTelemetry Schema

All neural operations emit structured telemetry validated by Weaver.

```yaml
# registry/phases_6_10/neural.yaml
spans:
  - span_name: neural.predict
    attributes:
      - name: model.type
        type: string
        requirement_level: required
        brief: "Type of neural model (qlearning, sarsa, actor_critic)"
      - name: state.size
        type: int
        requirement_level: required
        brief: "Size of input state vector"
      - name: prediction.latency_ticks
        type: int
        requirement_level: required
        brief: "Prediction latency in CPU ticks"

  - span_name: neural.train
    attributes:
      - name: model.type
        type: string
        requirement_level: required
      - name: batch.size
        type: int
        requirement_level: required
      - name: loss
        type: double
        requirement_level: required
        brief: "Training loss (TD-error MSE)"
      - name: convergence
        type: boolean
        requirement_level: required
        brief: "Whether model has converged"

metrics:
  - metric_name: neural.prediction.latency
    instrument: histogram
    unit: ticks
    description: "Histogram of prediction latencies"

  - metric_name: neural.training.loss
    instrument: gauge
    unit: 1
    description: "Current training loss"

  - metric_name: neural.replay.size
    instrument: gauge
    unit: experiences
    description: "Number of experiences in replay buffer"
```

---

## Testing Strategy

### Unit Tests (Chicago TDD)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qlearning_convergence() {
        // Simple grid world environment
        let mut agent = QLearningAgent::new(QLearningConfig::default());

        // Train for 1000 episodes
        for _ in 0..1000 {
            let trajectory = collect_episode(&agent);
            agent.train(&trajectory).unwrap();
        }

        // Verify convergence
        assert!(agent.has_converged());

        // Verify optimal policy
        let state = State::start();
        let action = agent.predict(&state).unwrap();
        assert_eq!(action, OPTIMAL_ACTION);
    }

    #[test]
    fn test_experience_replay_priority() {
        let buffer = ReplayBuffer::new(1000, true);

        // Add experiences with varying TD-errors
        buffer.push(Experience { td_error: 0.1, ..default() });
        buffer.push(Experience { td_error: 5.0, ..default() }); // High priority
        buffer.push(Experience { td_error: 0.5, ..default() });

        // Sample should prioritize high TD-error
        let batch = buffer.sample(10);
        assert!(batch.iter().any(|e| e.td_error > 1.0));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_mape_k_neural_integration() {
    // Setup workflow engine
    let engine = WorkflowEngine::new().await;

    // Setup neural analyzer
    let analyzer = NeuralAnalyzer::<QLearningAgent>::new(
        QLearningConfig::default()
    );

    // Execute workflow with neural optimization
    let workflow = create_test_workflow();
    let case_id = engine.start_case(&workflow).await.unwrap();

    // Monitor executions
    for _ in 0..100 {
        let observations = engine.get_observations(case_id).await.unwrap();
        let action = analyzer.analyze(&observations).await.unwrap();
        engine.apply_action(case_id, action).await.unwrap();
    }

    // Verify performance improvement
    let initial_latency = observations[0].latency;
    let final_latency = observations[99].latency;
    assert!(final_latency < initial_latency * 0.8); // 20% improvement
}
```

---

## Migration Path

### Phase 1: Core Traits (Week 1-2)
- Define `NeuralModel` trait with GATs
- Implement `ReplayBuffer` with priority sampling
- Setup telemetry schema

### Phase 2: Q-Learning (Week 3-4)
- Implement `QLearningAgent`
- Discrete action spaces only
- Integration with MAPE-K

### Phase 3: SARSA & Actor-Critic (Week 5-6)
- Implement `SARSAAgent` (on-policy)
- Implement `ActorCriticAgent` (continuous actions)
- Neural network layers

### Phase 4: Multi-Agent (Week 7-8)
- Distributed training
- Consensus-backed policy sync (Phase 8 dependency)
- Benchmarks and validation

---

## Success Criteria

- [ ] All neural predictions ≤8 ticks (Chatman constant)
- [ ] Q-Learning converges on grid world in <1000 episodes
- [ ] Actor-Critic learns continuous control in <5000 episodes
- [ ] Replay buffer supports 10K experiences with <100μs sampling
- [ ] Weaver validation passes for all neural telemetry
- [ ] Integration with MAPE-K Analyze stage
- [ ] 20% workflow performance improvement in benchmarks

---

## Related Documents

- `PHASES_6-10_ARCHITECTURE_OVERVIEW.md` - Overall architecture
- `TYPE_LEVEL_DESIGN_PATTERNS.md` - Rust patterns used
- `PHASE_INTEGRATION_ARCHITECTURE.md` - How Phase 6 integrates with others
- `DOCTRINE_COVENANT.md` - Covenant 3 (Feedback Loops)

**Next**: See `PHASE_7_QUANTUM_CRYPTO_SPECIFICATION.md` for cryptographic layer.
