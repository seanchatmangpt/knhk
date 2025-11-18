# KNHK Agent Swarm Framework

A distributed multi-agent system for KNHK enabling collective intelligence through Byzantine consensus, neural learning, and quantum-safe communication.

## Architecture

The swarm framework implements a hierarchical multi-agent architecture:

- **QueenAgent**: Orchestrates swarm, assigns tasks, resolves conflicts
- **WorkerAgent**: Executes tasks, reports results, proposes optimizations
- **ScoutAgent**: Gathers information, detects patterns, monitors environment
- **GuardianAgent**: Validates decisions, enforces invariants, detects Byzantine behavior
- **LearnerAgent**: Runs neural models, discovers patterns, provides recommendations

## DOCTRINE Alignment

- **Covenant 2**: Byzantine-safe consensus ensures invariants are law
- **Covenant 3**: Full MAPE-K loop integration across swarm operations
- **Covenant 6**: Complete observability of all swarm activities
- **New Covenant**: "Swarm > Individual" - Collective wisdom over single-agent decisions

## Components

### Core Swarm Engine
- Agent lifecycle management
- Task assignment and tracking
- Health monitoring
- Swarm-wide decision making

### Byzantine Consensus
- 2f+1 agreement protocol
- Byzantine fault tolerance
- Quorum-based decisions
- Safety and liveness guarantees

### Communication Framework
- Priority-based message queuing
- Pub/sub messaging patterns
- Ordered delivery guarantees
- Audit trail logging

### Federated Learning
- Distributed model training
- Consensus on model updates
- Byzantine-resilient aggregation
- Privacy-preserving learning

### Distributed Storage
- RDF dataset replication
- Merkle tree verification
- Consistency checking
- Byzantine-resilient storage

## Usage

```rust
use knhk_swarm::{AgentSwarm, AgentConfig, SwarmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let swarm = AgentSwarm::new(SwarmConfig::default()).await?;

    // Spawn worker agents
    let worker_id = swarm.spawn_worker_agent(AgentConfig::default()).await?;

    // Make swarm decision with Byzantine consensus
    let decision = swarm.make_swarm_decision(workflow_decision).await?;

    // Check swarm health
    let health = swarm.health_check().await;
    println!("Swarm health: {:.2}%", health.overall_health * 100.0);

    Ok(())
}
```

## Validation

All swarm operations are validated using OpenTelemetry Weaver:

```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

## Testing

```bash
cargo test --package knhk-swarm
cargo bench --package knhk-swarm
```

## Performance Targets

- Consensus latency: ≤8 ticks (Chatman Constant)
- Message delivery: In-order, exactly-once
- Agent spawn time: ≤100ms
- Health check: ≤50ms
- Swarm coordination overhead: ≤10%
