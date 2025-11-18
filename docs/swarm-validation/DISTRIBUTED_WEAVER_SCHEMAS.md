# Distributed Weaver Validation Schemas for Swarm Coordination

## Overview

This document defines **distributed telemetry schemas** for multi-agent swarm coordination, enabling OpenTelemetry Weaver to validate that distributed consensus, federated learning, and swarm coordination work correctly at runtime.

**DOCTRINE ALIGNMENT:**
- Covenant 6 (Observations Drive Everything): All swarm operations emit distributed telemetry
- Covenant 2 (Invariants Are Law): Weaver validation enforces distributed safety properties
- Covenant 5 (Chatman Constant): All swarm operations ≤8 ticks per agent

---

## Schema Location

Schemas should be added to `/home/user/knhk/registry/swarm/` directory:

```
registry/
├── swarm/
│   ├── swarm-coordination.yaml      # Agent-to-agent coordination
│   ├── swarm-consensus.yaml         # Distributed consensus telemetry
│   ├── swarm-learning.yaml          # Federated learning telemetry
│   ├── swarm-gossip.yaml            # Gossip protocol telemetry
│   └── swarm-metrics.yaml           # Performance metrics
```

---

## swarm-coordination.yaml

```yaml
# OpenTelemetry Semantic Conventions for Swarm Coordination
#
# DOCTRINE ALIGNMENT:
# - Covenant 6: Observations Drive Everything - All agent coordination emits telemetry
# - Covenant 2: Invariants Are Law - Distributed invariants validated via telemetry

groups:
  # Agent Join/Leave Events
  - id: swarm.agent.join
    type: event
    brief: "Agent joins the swarm"
    note: >
      Emitted when an agent successfully joins the swarm and registers with coordinators.
      All agents must emit this event during initialization.
    attributes:
      - id: swarm.agent_id
        type: string
        brief: "Unique agent identifier (UUID)"
        requirement_level: required

      - id: swarm.agent_type
        type: string
        brief: "Agent type (researcher, coder, tester, etc.)"
        examples: ["researcher", "coder", "production-validator"]
        requirement_level: required

      - id: swarm.swarm_id
        type: string
        brief: "Swarm identifier this agent is joining"
        requirement_level: required

      - id: swarm.topology
        type: string
        brief: "Swarm topology"
        examples: ["mesh", "hierarchical", "star", "hybrid"]
        requirement_level: required

      - id: swarm.total_agents
        type: int
        brief: "Total number of agents in swarm"
        requirement_level: required

      - id: swarm.coordinator_id
        type: string
        brief: "Coordinator agent ID"
        requirement_level: required

  - id: swarm.agent.leave
    type: event
    brief: "Agent leaves the swarm"
    note: >
      Emitted when an agent gracefully leaves the swarm or is ejected.
      High frequency indicates instability.
    attributes:
      - ref: swarm.agent_id
      - ref: swarm.swarm_id

      - id: swarm.leave_reason
        type: string
        brief: "Reason for leaving"
        examples: ["graceful_shutdown", "byzantine_detected", "timeout", "network_partition"]
        requirement_level: required

      - id: swarm.byzantine_detected
        type: boolean
        brief: "Whether agent was ejected for Byzantine behavior"
        requirement_level: required

  # Agent-to-Agent Message Passing
  - id: swarm.message.send
    type: span
    brief: "Agent sends message to another agent"
    note: >
      Tracks inter-agent message passing for coordination.
      Performance requirement: ≤5ms local, ≤50ms remote.
    attributes:
      - ref: swarm.agent_id

      - id: swarm.message_id
        type: string
        brief: "Unique message identifier"
        requirement_level: required

      - id: swarm.message_type
        type: string
        brief: "Message type"
        examples: ["task_assignment", "result_share", "consensus_vote", "gossip"]
        requirement_level: required

      - id: swarm.target_agent_id
        type: string
        brief: "Target agent ID"
        requirement_level: required

      - id: swarm.message_size_bytes
        type: int
        brief: "Message payload size in bytes"
        requirement_level: required

      - id: swarm.broadcast
        type: boolean
        brief: "Whether message is broadcast to all agents"
        requirement_level: required

  - id: swarm.message.receive
    type: span
    brief: "Agent receives message from another agent"
    note: >
      Tracks message reception. Parent span should be swarm.message.send for distributed tracing.
    attributes:
      - ref: swarm.agent_id
      - ref: swarm.message_id
      - ref: swarm.message_type

      - id: swarm.sender_agent_id
        type: string
        brief: "Sender agent ID"
        requirement_level: required

      - id: swarm.latency_ms
        type: int
        brief: "Message delivery latency (milliseconds)"
        requirement_level: required

      - id: swarm.message_dropped
        type: boolean
        brief: "Whether message was dropped/lost"
        requirement_level: required

  # Task Coordination
  - id: swarm.task.assign
    type: event
    brief: "Task assigned to agent by coordinator"
    note: >
      Emitted when coordinator assigns work to an agent.
      Enables tracking of load balancing and task distribution.
    attributes:
      - ref: swarm.agent_id
      - ref: swarm.swarm_id

      - id: swarm.task_id
        type: string
        brief: "Unique task identifier"
        requirement_level: required

      - id: swarm.task_type
        type: string
        brief: "Task type"
        examples: ["code_review", "test_generation", "research", "validation"]
        requirement_level: required

      - id: swarm.task_priority
        type: string
        brief: "Task priority"
        examples: ["low", "medium", "high", "critical"]
        requirement_level: required

      - id: swarm.assigned_by
        type: string
        brief: "Coordinator agent ID that assigned this task"
        requirement_level: required

  - id: swarm.task.complete
    type: event
    brief: "Agent completes assigned task"
    note: >
      Emitted when agent finishes task execution.
      Tracks task completion rates and agent throughput.
    attributes:
      - ref: swarm.agent_id
      - ref: swarm.task_id

      - id: swarm.task_duration_ms
        type: int
        brief: "Task execution duration (milliseconds)"
        requirement_level: required

      - id: swarm.task_success
        type: boolean
        brief: "Whether task completed successfully"
        requirement_level: required

      - id: swarm.task_result_size_bytes
        type: int
        brief: "Size of task result in bytes"
        requirement_level: required
```

---

## swarm-consensus.yaml

```yaml
# Distributed Consensus Telemetry for Multi-Agent Swarms
#
# DOCTRINE ALIGNMENT:
# - Covenant 2: Invariants Are Law - Quorum properties validated via telemetry
# - Covenant 5: Chatman Constant - All consensus operations ≤8 ticks

groups:
  # Distributed Consensus Round
  - id: swarm.consensus.round
    type: span
    brief: "Distributed consensus round across N agents"
    note: >
      Tracks a complete distributed consensus round.
      MUST be emitted by ALL participating agents with same round_id.
    attributes:
      - ref: swarm.agent_id

      - id: swarm.consensus.round_id
        type: string
        brief: "Unique round identifier (UUID) - SAME across all agents"
        requirement_level: required

      - id: swarm.consensus.algorithm
        type: string
        brief: "Consensus algorithm"
        examples: ["PBFT", "HotStuff", "Raft", "Gossip"]
        requirement_level: required

      - id: swarm.consensus.proposer_id
        type: string
        brief: "Agent ID of the proposer/leader"
        requirement_level: required

      - id: swarm.consensus.total_agents
        type: int
        brief: "Total number of agents in consensus"
        requirement_level: required

      - id: swarm.consensus.quorum_size
        type: int
        brief: "Required quorum size (2f+1)"
        requirement_level: required

      - id: swarm.consensus.received_votes
        type: int
        brief: "Number of votes this agent received"
        requirement_level: required

      - id: swarm.consensus.voted
        type: boolean
        brief: "Whether this agent voted in this round"
        requirement_level: required

      - id: swarm.consensus.value_hash
        type: string
        brief: "Blake3 hash of proposed value"
        requirement_level: required

  - id: swarm.consensus.finality
    type: event
    brief: "Consensus finality achieved (quorum reached)"
    note: >
      Emitted when consensus finalizes a value.
      CRITICAL: All agents MUST emit this with SAME round_id and value_hash.
    attributes:
      - ref: swarm.consensus.round_id
      - ref: swarm.agent_id

      - id: swarm.consensus.finalized_value_hash
        type: string
        brief: "Blake3 hash of finalized value"
        requirement_level: required

      - id: swarm.consensus.quorum_votes
        type: int
        brief: "Number of votes in quorum (MUST be ≥2f+1)"
        requirement_level: required

      - id: swarm.consensus.latency_ms
        type: int
        brief: "Time from proposal to finality (milliseconds)"
        requirement_level: required

      - id: swarm.consensus.byzantine_detected_count
        type: int
        brief: "Number of Byzantine agents detected in this round"
        requirement_level: required

  # Split-Brain Detection
  - id: swarm.consensus.split_brain
    type: event
    brief: "Split-brain detected (network partition)"
    note: >
      Emitted when multiple quorums finalize different values (CRITICAL ERROR).
      This violates safety and MUST trigger system halt.
    attributes:
      - ref: swarm.consensus.round_id

      - id: swarm.consensus.partition_1_value_hash
        type: string
        brief: "Value hash finalized by partition 1"
        requirement_level: required

      - id: swarm.consensus.partition_2_value_hash
        type: string
        brief: "Value hash finalized by partition 2"
        requirement_level: required

      - id: swarm.consensus.partition_1_agents
        type: string
        brief: "Comma-separated list of agent IDs in partition 1"
        requirement_level: required

      - id: swarm.consensus.partition_2_agents
        type: string
        brief: "Comma-separated list of agent IDs in partition 2"
        requirement_level: required
```

---

## swarm-learning.yaml

```yaml
# Federated Learning Telemetry for Distributed Swarms
#
# DOCTRINE ALIGNMENT:
# - Covenant 6: All learning operations observable
# - Covenant 2: Byzantine-robust aggregation enforced

groups:
  # Federated Learning Round
  - id: swarm.learning.round
    type: span
    brief: "Federated learning round (local training + aggregation)"
    note: >
      Tracks a complete federated learning round.
      All agents MUST emit this with same round_id.
    attributes:
      - ref: swarm.agent_id

      - id: swarm.learning.round_id
        type: string
        brief: "Unique learning round identifier (UUID)"
        requirement_level: required

      - id: swarm.learning.total_agents
        type: int
        brief: "Total agents participating in federated learning"
        requirement_level: required

      - id: swarm.learning.model_version
        type: int
        brief: "Global model version number"
        requirement_level: required

      - id: swarm.learning.local_samples
        type: int
        brief: "Number of samples used for local training"
        requirement_level: required

      - id: swarm.learning.local_loss
        type: number
        brief: "Local model loss after training"
        requirement_level: required

      - id: swarm.learning.local_accuracy
        type: number
        brief: "Local model accuracy (0.0-1.0)"
        requirement_level: required

  # Gradient Aggregation
  - id: swarm.learning.aggregate
    type: span
    brief: "Aggregate gradients from all agents (Byzantine-robust)"
    note: >
      Tracks gradient aggregation. MUST use median-based aggregation to reject Byzantine gradients.
    attributes:
      - ref: swarm.learning.round_id

      - id: swarm.learning.aggregator_id
        type: string
        brief: "Agent ID of the aggregator"
        requirement_level: required

      - id: swarm.learning.gradients_received
        type: int
        brief: "Number of gradient updates received"
        requirement_level: required

      - id: swarm.learning.gradients_rejected
        type: int
        brief: "Number of gradients rejected as Byzantine"
        requirement_level: required

      - id: swarm.learning.aggregation_method
        type: string
        brief: "Aggregation method"
        examples: ["FedAvg", "MedianAvg", "Krum", "Trimmed-Mean"]
        requirement_level: required

      - id: swarm.learning.aggregated_model_hash
        type: string
        brief: "Blake3 hash of aggregated model weights"
        requirement_level: required

  # Convergence Validation
  - id: swarm.learning.convergence
    type: event
    brief: "Federated learning converged (all agents same policy)"
    note: >
      Emitted when all agents converge to same policy (KL divergence < threshold).
    attributes:
      - ref: swarm.learning.round_id

      - id: swarm.learning.kl_divergence
        type: number
        brief: "KL divergence between agent policies"
        requirement_level: required

      - id: swarm.learning.convergence_threshold
        type: number
        brief: "KL divergence threshold for convergence"
        requirement_level: required

      - id: swarm.learning.rounds_to_convergence
        type: int
        brief: "Number of rounds to reach convergence"
        requirement_level: required
```

---

## Validation Commands

### Schema Validation (Must Pass)

```bash
# Validate swarm schemas
weaver registry check -r registry/swarm/

# Expected output: All schemas valid
```

### Distributed Live-Check (Production Requirement)

```bash
# Run distributed live-check across ALL swarm agents
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317 \
  weaver registry live-check \
    --registry registry/swarm/ \
    --distributed \
    --min-agents 10 \
    --timeout 60s

# This validates:
# 1. All agents emit telemetry
# 2. Consensus quorums match across agents
# 3. No split-brain (all agents same value_hash)
# 4. Message delivery ≥99.9%
# 5. Learning convergence (KL divergence < 0.01)
```

### Invariant Validation

```bash
# Verify distributed invariants via telemetry
weaver registry live-check \
  --invariant "swarm.consensus.quorum_votes >= 2*f+1" \
  --invariant "swarm.consensus.split_brain count == 0" \
  --invariant "swarm.message.receive latency_ms <= 50" \
  --invariant "swarm.learning.kl_divergence < 0.01"
```

---

## Integration with Existing Schemas

These distributed swarm schemas **extend** the existing consensus schemas:

```yaml
# registry/registry_manifest.yaml
groups:
  - id: swarm
    brief: "Distributed swarm coordination and learning"
    schemas:
      - swarm/swarm-coordination.yaml
      - swarm/swarm-consensus.yaml
      - swarm/swarm-learning.yaml
      - swarm/swarm-gossip.yaml
      - swarm/swarm-metrics.yaml
```

---

## Next Steps

1. **Create Schema Files**: Add YAML files to `/home/user/knhk/registry/swarm/`
2. **Implement Telemetry Emission**: Update consensus/learning code to emit these spans/events
3. **Deploy OpenTelemetry Collector**: Set up distributed telemetry aggregation
4. **Run Distributed Live-Check**: Validate swarm behavior at runtime
5. **CI/CD Integration**: Add Weaver validation to deployment pipeline

---

**Schema Version**: 1.0
**DOCTRINE Compliance**: Covenants 2, 5, 6
**Status**: ⚠️ **DEFINED** (Implementation Required)
