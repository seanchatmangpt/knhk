# ADR-003: Neural Learning Integration (Phase 6)

**Status**: Proposed
**Date**: 2025-11-18
**Deciders**: System Architecture Team, DOCTRINE Compliance Board
**Technical Story**: Integrate reinforcement learning into MAPE-K Analyze stage

---

## Context and Problem Statement

The MAPE-K Analyze stage currently uses statistical methods and rule-based anomaly detection. As workflow complexity increases, we need adaptive learning to discover optimal configurations automatically. How do we integrate neural learning while maintaining:
- Chatman constant (≤8 ticks hot path)
- Weaver validation (source of truth)
- Deterministic behavior (for Byzantine consensus)

---

## Decision Drivers

- **DOCTRINE Alignment**: MAPE-K (Covenant 3 - Feedback Loops at Machine Speed)
- **Performance**: Prediction must be ≤8 ticks (hot path)
- **Scalability**: Must work with distributed consensus (Phase 8)
- **Validation**: Weaver must validate all learning telemetry
- **Simplicity**: Avoid deep learning complexity where tabular RL suffices

---

## Considered Options

### Option 1: Deep Neural Networks (DNNs)
**Description**: Multi-layer perceptrons with backpropagation.

**Pros**:
- ✅ Handle high-dimensional state spaces
- ✅ Function approximation for continuous actions
- ✅ Industry-standard approach

**Cons**:
- ❌ Training is slow (seconds to minutes)
- ❌ Inference can exceed 8 ticks for large models
- ❌ Non-deterministic (floating-point rounding)
- ❌ Difficult to validate with Weaver

**Verdict**: REJECTED (violates Chatman constant)

### Option 2: Tabular Reinforcement Learning (Q-Learning, SARSA)
**Description**: Discrete state/action tables with value updates.

**Pros**:
- ✅ Prediction is O(1) table lookup (≤8 ticks)
- ✅ Deterministic (no floating-point instability)
- ✅ Simple to validate (Weaver can log Q-values)
- ✅ Works well for discrete workflow configurations

**Cons**:
- ❌ Doesn't scale to high-dimensional state spaces
- ❌ Requires discretization of continuous states
- ❌ Table size grows exponentially with state dimensions

**Verdict**: ACCEPTED (for discrete workflow optimization)

### Option 3: Policy Gradient Methods (Actor-Critic)
**Description**: Neural networks for continuous action spaces.

**Pros**:
- ✅ Handles continuous actions (resource allocation)
- ✅ Can run prediction in <8 ticks with small networks
- ✅ Compatible with distributed learning (Phase 8)

**Cons**:
- ❌ Training requires many samples
- ❌ Variance in gradients (slower convergence)
- ❌ More complex than tabular RL

**Verdict**: ACCEPTED (for continuous resource optimization)

---

## Decision Outcome

**Chosen option**: **Hybrid approach (Tabular RL + Actor-Critic)**

### Implementation Strategy

1. **Discrete Actions** (pattern selection, queue size):
   - Use Q-Learning or SARSA
   - State: `(active_tasks, avg_latency, cpu_util, mem_util, pattern_id, queue_depth)`
   - Action: `ScaleUp | ScaleDown | SwitchPattern | ResizeQueue`
   - Prediction: O(1) table lookup (≤8 ticks)

2. **Continuous Actions** (resource allocation, parallelism):
   - Use Actor-Critic
   - State: Same as above (6-dimensional vector)
   - Action: Continuous values (0.0-1.0 for resource fraction)
   - Prediction: Small MLP (2 layers, 32 neurons) ≈ 5 ticks

3. **Training** (Off Critical Path):
   - Async background thread
   - Experience replay with priority sampling
   - Batch updates every 60 seconds
   - No hot path impact

### MAPE-K Integration

```
Monitor → Analyze → Plan → Execute → Knowledge
             ↓
    Neural Predictor
         ↓
    State → Action (≤8 ticks)
         ↓
    Experience Replay (async training)
```

---

## Positive Consequences

- ✅ Adaptive workflow optimization (learns from execution history)
- ✅ Maintains Chatman constant (≤8 ticks for prediction)
- ✅ Compatible with Byzantine consensus (deterministic inference)
- ✅ Weaver validation for all learning telemetry
- ✅ Graceful degradation (fallback to rule-based if ML fails)

---

## Negative Consequences

- ⚠️ Requires experience replay buffer (~1 MB memory per 1K experiences)
- ⚠️ Learning convergence may take 1000+ episodes
- ⚠️ Non-stationary environments may cause divergence
- ⚠️ Discretization of continuous states loses precision

---

## Mitigation Strategies

1. **Convergence**: Use target networks (DQN) for stability
2. **Non-Stationarity**: Adaptive learning rate decay
3. **Memory**: Circular buffer with max 10K experiences
4. **Discretization**: Fine-grained bins (100 bins per dimension)

---

## Validation

### Weaver Schema
```yaml
spans:
  - span_name: neural.predict
    attributes:
      - name: model.type
        type: string
      - name: prediction.latency_ticks
        type: int
        brief: "Must be ≤8 ticks"
```

### Chicago TDD Test
```rust
#[test]
fn test_prediction_latency() {
    let agent = QLearningAgent::new(config);
    let start = Instant::now();
    agent.predict(&state).unwrap();
    let ticks = start.elapsed().as_nanos() as u32;
    assert!(ticks <= 8, "Violated Chatman constant");
}
```

---

## Compliance

- **DOCTRINE Principle**: MAPE-K (Analyze stage learns optimal policies)
- **Covenant**: Covenant 3 (Feedback Loops Run at Machine Speed)
- **Chatman Constant**: Prediction ≤8 ticks (validated in chicago-tdd)
- **Weaver Validation**: All learning metrics observable

---

## References

- [Sutton & Barto (2018) - Reinforcement Learning: An Introduction](http://incompleteideas.net/book/the-book.html)
- [Mnih et al. (2015) - DQN Paper](https://www.nature.com/articles/nature14236)
- `PHASE_6_NEURAL_SPECIFICATION.md`
- `DOCTRINE_COVENANT.md` - Covenant 3

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 2025-11-18 | Initial ADR | System Architecture Team |
