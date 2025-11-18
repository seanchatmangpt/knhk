# Chaos Engineering Test Plan for KNHK Swarm Systems

## Overview

This document defines comprehensive chaos engineering tests to validate Byzantine fault tolerance, network resilience, and distributed coordination under adversarial conditions.

**DOCTRINE ALIGNMENT:**
- Covenant 2 (Invariants Are Law): System maintains safety under chaos
- Covenant 6 (Observations Drive Everything): All chaos events emit telemetry
- Byzantine tolerance f < n/3 MUST hold under all conditions

---

## Test Categories

### Category 1: Byzantine Agent Attacks

#### Test 1.1: Equivocation Attack

**Scenario**: Malicious agent sends conflicting messages for the same consensus round

```rust
#[test]
fn test_byzantine_equivocation_attack() {
    // Setup: 7 agent PBFT swarm (tolerates 2 Byzantine)
    let config = PBFTConfig::new(7).unwrap();
    let mut agents = create_swarm(7, config.clone());

    // Inject Byzantine agent
    let byzantine_agent = &mut agents[0];

    // Round 1: Byzantine agent proposes value A
    let value_a = b"value_a".to_vec();
    let msg1 = byzantine_agent.pre_prepare(value_a.clone(), &config).unwrap();

    // ATTACK: Same agent proposes conflicting value B for SAME sequence
    let value_b = b"value_b".to_vec();
    let msg2 = byzantine_agent.pre_prepare(value_b.clone(), &config).unwrap();

    // EXPECTED: Honest agents detect equivocation
    let mut detector = ByzantineFaultDetector::new(7);
    detector.detect_equivocation(0, &value_a, &value_b).unwrap();

    // VALIDATION:
    assert_eq!(detector.get_faulty_replicas(), vec![0]);
    assert!(detector.is_system_safe()); // Still safe with 1 Byzantine (< 2)

    // Weaver validation: Ensure equivocation telemetry emitted
    // OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317
    // Check for consensus.byzantine_detected metric with behavior=equivocation
}
```

**Success Criteria**:
- ✅ Equivocation detected within 1 round
- ✅ Byzantine agent blacklisted
- ✅ Consensus continues with remaining honest agents
- ✅ Weaver validates `consensus.byzantine_detected` event emitted

---

#### Test 1.2: Silent Fault Attack

**Scenario**: Agent stops responding (crash or network failure)

```rust
#[test]
fn test_byzantine_silent_fault() {
    let config = PBFTConfig::new(7).unwrap();
    let mut agents = create_swarm(7, config.clone());

    // Round 1: All agents participate
    let round1 = run_consensus_round(&mut agents, b"value1");
    assert!(round1.finalized);

    // ATTACK: Agent 0 stops responding
    agents[0].is_crashed = true; // Simulate crash

    // Round 2: Consensus should still finalize (6 agents >= quorum of 5)
    let round2 = run_consensus_round(&mut agents, b"value2");

    // VALIDATION:
    assert!(round2.finalized);
    assert!(round2.timeout_for_agent(&agents[0])); // Timeout detected

    // Weaver validation: Ensure silent fault telemetry emitted
}
```

**Success Criteria**:
- ✅ Timeout detected after view timeout (5 seconds)
- ✅ Consensus continues without crashed agent
- ✅ System remains safe (6 agents > quorum of 5)
- ✅ Weaver validates `consensus.byzantine_detected` with `behavior=silent_fault`

---

#### Test 1.3: Double-Propose Attack

**Scenario**: Leader proposes different values to different subsets of agents

```rust
#[test]
fn test_byzantine_double_propose() {
    let config = PBFTConfig::new(7).unwrap();
    let mut agents = create_swarm(7, config.clone());
    let leader = &mut agents[0];

    // ATTACK: Leader proposes value_a to agents 1-3
    let value_a = b"value_a".to_vec();
    let msg_a = leader.pre_prepare(value_a.clone(), &config).unwrap();
    broadcast_to_subset(&msg_a, &agents[1..4]);

    // ATTACK: Leader proposes value_b to agents 4-6 (DIFFERENT VALUE)
    let value_b = b"value_b".to_vec();
    let msg_b = leader.pre_prepare(value_b.clone(), &config).unwrap();
    broadcast_to_subset(&msg_b, &agents[4..7]);

    // EXPECTED: Agents detect conflicting pre-prepare messages
    let detector = run_byzantine_detection(&agents);

    // VALIDATION:
    assert!(detector.detected_equivocation(leader.id));
    assert_eq!(detector.faulty_replicas().len(), 1);
    assert!(!consensus_reached(&agents)); // No finality due to conflict
}
```

**Success Criteria**:
- ✅ Conflict detected when agents share pre-prepare messages
- ✅ Leader blacklisted for Byzantine behavior
- ✅ View change triggered to rotate leader
- ✅ New leader elected and consensus resumes

---

### Category 2: Network Partition Attacks

#### Test 2.1: Split-Brain Scenario

**Scenario**: Network partition creates two isolated quorums

```rust
#[test]
fn test_network_partition_split_brain() {
    let config = PBFTConfig::new(10).unwrap(); // 10 agents, quorum=7, f=3
    let mut agents = create_swarm(10, config.clone());

    // ATTACK: Create network partition
    // Partition 1: Agents 0-5 (6 agents, cannot reach quorum of 7)
    // Partition 2: Agents 6-9 (4 agents, cannot reach quorum of 7)
    let partition1 = &mut agents[0..6];
    let partition2 = &mut agents[6..10];

    // Simulate network partition (agents cannot communicate across partitions)
    block_communication_between(partition1, partition2);

    // Try to reach consensus in partition 1
    let round1 = run_consensus_round_isolated(partition1, b"value1");

    // Try to reach consensus in partition 2
    let round2 = run_consensus_round_isolated(partition2, b"value2");

    // VALIDATION:
    assert!(!round1.finalized); // Cannot reach quorum (6 < 7)
    assert!(!round2.finalized); // Cannot reach quorum (4 < 7)

    // CRITICAL: No split-brain (no two quorums with different values)
    // Weaver validation: Ensure NO consensus.finality events emitted
    // Ensure NO swarm.consensus.split_brain events emitted
}
```

**Success Criteria**:
- ✅ Neither partition reaches quorum
- ✅ No finality events emitted (liveness failure, but safety preserved)
- ✅ NO split-brain (two different finalized values)
- ✅ Weaver validates `swarm.consensus.split_brain` count == 0

---

#### Test 2.2: Partition Healing

**Scenario**: Network partition heals, agents must reconcile state

```rust
#[test]
fn test_partition_healing_reconciliation() {
    let config = HotStuffConfig::new(7).unwrap();
    let mut agents = create_swarm(7, config.clone());

    // Phase 1: Normal operation
    let round1 = run_consensus_round(&mut agents, b"value1");
    assert!(round1.finalized);

    // Phase 2: Network partition (agents 0-3 vs 4-6)
    let partition1 = &mut agents[0..4];
    let partition2 = &mut agents[4..7];
    block_communication_between(partition1, partition2);

    // Neither partition can reach quorum (4 < 5, 3 < 5)
    let round2_p1 = run_consensus_round_isolated(partition1, b"value2");
    let round2_p2 = run_consensus_round_isolated(partition2, b"value2");
    assert!(!round2_p1.finalized);
    assert!(!round2_p2.finalized);

    // Phase 3: Partition heals
    unblock_communication_between(partition1, partition2);

    // Phase 4: Agents reconcile state and resume consensus
    let round3 = run_consensus_round(&mut agents, b"value3");

    // VALIDATION:
    assert!(round3.finalized);
    assert_eq!(round3.quorum_votes, 5); // Quorum reached
    // All agents must have same committed state
    verify_state_consistency(&agents);
}
```

**Success Criteria**:
- ✅ Partition prevents consensus (liveness failure)
- ✅ Healing restores connectivity
- ✅ Agents reconcile state via view synchronization
- ✅ Consensus resumes with full quorum

---

### Category 3: Timing Attacks

#### Test 3.1: Slow Agent Attack

**Scenario**: Malicious agent delays messages to cause timeouts

```rust
#[test]
fn test_slow_agent_timing_attack() {
    let config = PBFTConfig::new(7).unwrap();
    let mut agents = create_swarm(7, config.clone());

    // ATTACK: Agent 0 delays all messages by 10 seconds
    agents[0].message_delay_ms = 10_000;

    // Run consensus round with timeout of 5 seconds
    let start = Instant::now();
    let round = run_consensus_round_with_timeout(&mut agents, b"value1", Duration::from_secs(5));
    let duration = start.elapsed();

    // VALIDATION:
    assert!(round.finalized); // Consensus succeeds without slow agent
    assert!(duration < Duration::from_secs(6)); // Completed before 6s
    assert!(round.timed_out_agents.contains(&agents[0].id)); // Slow agent detected

    // Weaver validation: Ensure consensus.latency_ms <= 6000
}
```

**Success Criteria**:
- ✅ Slow agent excluded from quorum after timeout
- ✅ Consensus completes with remaining agents
- ✅ Slow agent marked as suspected Byzantine
- ✅ Latency within acceptable bounds

---

### Category 4: Federated Learning Attacks

#### Test 4.1: Gradient Poisoning Attack

**Scenario**: Byzantine agent sends malicious gradients to corrupt global model

```rust
#[test]
fn test_gradient_poisoning_attack() {
    let config = FederatedLearningConfig::new(10).unwrap();
    let mut agents = create_learning_swarm(10, config.clone());

    // Train local models on honest agents
    for agent in &mut agents[1..10] {
        agent.train_local_model(honest_dataset());
    }

    // ATTACK: Agent 0 sends poisoned gradients (100x larger than normal)
    agents[0].poison_gradients(scale_factor = 100.0);

    // Collect gradients from all agents
    let mut gradients = vec![];
    for agent in &agents {
        gradients.push(agent.get_local_gradients());
    }

    // Aggregate using Byzantine-robust median-based FedAvg
    let aggregator = MedianAggregator::new();
    let global_model = aggregator.aggregate(gradients);

    // VALIDATION:
    assert!(aggregator.rejected_gradients.contains(&agents[0].id));
    assert_eq!(aggregator.rejected_gradients.len(), 1); // Only 1 Byzantine

    // Model accuracy should not degrade significantly
    let accuracy_without_attack = 0.95;
    let accuracy_with_attack = evaluate_model(&global_model);
    assert!(accuracy_with_attack >= accuracy_without_attack - 0.05); // <5% degradation
}
```

**Success Criteria**:
- ✅ Poisoned gradients detected by median aggregation
- ✅ Byzantine agent's gradients rejected
- ✅ Global model accuracy remains high (>90%)
- ✅ Weaver validates `swarm.learning.gradients_rejected` metric

---

### Category 5: Chaos at Scale

#### Test 5.1: Massive Agent Churn

**Scenario**: 30% of agents crash and rejoin repeatedly

```rust
#[test]
fn test_massive_agent_churn() {
    let config = HotStuffConfig::new(100).unwrap();
    let mut agents = create_swarm(100, config.clone());

    // Run consensus for 100 rounds with continuous churn
    for round in 0..100 {
        // Randomly crash 30 agents
        let crashed = randomly_select(30, &agents);
        for agent in crashed {
            agent.crash();
        }

        // Remaining 70 agents attempt consensus
        let result = run_consensus_round(&mut agents, format!("value{}", round).as_bytes());

        // VALIDATION: Consensus still succeeds (70 > quorum of 67)
        assert!(result.finalized);

        // Restart crashed agents
        for agent in crashed {
            agent.restart();
            agent.sync_state(&agents); // Catch up on missed rounds
        }
    }

    // Weaver validation: Ensure all 100 rounds finalized
    // Ensure swarm.agent.leave events == swarm.agent.join events (all agents rejoined)
}
```

**Success Criteria**:
- ✅ Consensus succeeds despite 30% churn
- ✅ Crashed agents successfully rejoin and sync state
- ✅ No data loss or inconsistency
- ✅ Performance degradation <20%

---

## Test Execution Matrix

| Test | Swarm Size | Byzantine Agents | Expected Result | Weaver Validation |
|------|------------|------------------|-----------------|-------------------|
| 1.1 Equivocation | 7 | 1 | Detected, safe | `byzantine_detected` emitted |
| 1.2 Silent Fault | 7 | 1 | Timeout, safe | `byzantine_detected` (silent) |
| 1.3 Double-Propose | 7 | 1 (leader) | View change | Leader rotation |
| 2.1 Split-Brain | 10 | 0 | No finality | `split_brain count == 0` |
| 2.2 Partition Heal | 7 | 0 | Reconciled | State consistency |
| 3.1 Slow Agent | 7 | 1 | Excluded, safe | Latency <= 6s |
| 4.1 Gradient Poison | 10 | 1 | Rejected | `gradients_rejected == 1` |
| 5.1 Massive Churn | 100 | 30 (crashed) | Finalized | All rounds succeed |

---

## Continuous Chaos Testing (CI/CD Integration)

### GitHub Actions Workflow

```yaml
name: Chaos Engineering Tests

on:
  push:
    branches: [main, develop]
  schedule:
    - cron: '0 2 * * *'  # Run nightly

jobs:
  chaos-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup OpenTelemetry Collector
        run: |
          docker run -d -p 4317:4317 -p 4318:4318 \\
            otel/opentelemetry-collector:latest

      - name: Run Byzantine Attack Tests
        run: |
          cargo test --package knhk-consensus --test chaos_byzantine

      - name: Run Network Partition Tests
        run: |
          cargo test --package knhk-consensus --test chaos_network

      - name: Run Federated Learning Attack Tests
        run: |
          cargo test --package knhk-learning --test chaos_learning

      - name: Validate Weaver Schemas
        run: |
          weaver registry check -r registry/swarm/

      - name: Distributed Live-Check
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4317
        run: |
          weaver registry live-check \\
            --registry registry/swarm/ \\
            --distributed \\
            --min-agents 10
```

---

## Success Metrics

### Safety (MUST NEVER FAIL)

- ✅ **No split-brain**: `swarm.consensus.split_brain` count == 0
- ✅ **Byzantine tolerance**: System safe with f < n/3 Byzantine agents
- ✅ **No data loss**: All finalized values consistent across honest agents
- ✅ **Invariants preserved**: Quorum properties maintained

### Liveness (SHOULD RECOVER)

- ✅ **Consensus resumes**: After partition healing within 3 view changes
- ✅ **Agent recovery**: Crashed agents rejoin within 30 seconds
- ✅ **Performance degradation**: <20% latency increase under chaos

### Observability (WEAVER VALIDATES)

- ✅ **All events emitted**: 100% telemetry coverage
- ✅ **Distributed tracing**: Message paths traceable across agents
- ✅ **Metrics accuracy**: Latency, throughput, error rates accurate

---

## Implementation Status

**Current**: ❌ **NOT IMPLEMENTED**

**Required Work:**
1. Create chaos test suite (`tests/chaos_engineering/`)
2. Implement network partition simulator
3. Implement Byzantine agent injection
4. Set up distributed OpenTelemetry collector
5. Create Weaver validation assertions
6. Integrate with CI/CD pipeline

**Effort Estimate**: 3-4 weeks

---

**Document Version**: 1.0
**Status**: ⚠️ **SPECIFICATION ONLY** (Tests Not Implemented)
**Next Steps**: Implement test harness, integrate with Weaver validation
