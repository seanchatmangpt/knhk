# Mesh Network Deployment Topologies

**Version**: 1.0.0 | **Date**: 2025-11-18

---

## Overview

This document describes concrete deployment topologies for KNHK mesh networking across different scales, from small development teams to enterprise-scale production deployments.

## Topology Selection Matrix

| Agents | Topology | Gossip Fanout (k) | Regions | Convergence | Cost |
|--------|----------|------------------|---------|-------------|------|
| 10-100 | Flat Mesh | 5 | 1 | <50ms | $$ |
| 100-1k | Single-Region | 10 | 1 | <100ms | $$$ |
| 1k-10k | Multi-Region | 10-20 | 2-3 | <300ms | $$$$ |
| 10k-100k | Multi-Region + Leaders | 20-50 | 3-5 | <500ms | $$$$$ |
| 100k-1M | Hierarchical | 50-100 | 5-10 | <1s | $$$$$$ |

## Topology 1: Development Flat Mesh

**Use Case**: Local development, testing, small teams

### Configuration

```yaml
topology: flat_mesh
agent_count: 10-100
regions:
  - name: local
    location: localhost
    agents: 10-100

gossip:
  fanout: 5
  interval_ms: 100
  convergence_threshold: 0.95

network:
  transport: TCP
  tls: false  # Dev only!
  port_range: 8000-8100

bootstrap:
  seeds:
    - localhost:8000
    - localhost:8001
```

### Deployment Diagram

```
┌─────────────────────────────────────┐
│     Development Laptop              │
│                                     │
│  Agent1:8000 ←→ Agent2:8001         │
│       ↕             ↕               │
│  Agent3:8002 ←→ Agent4:8003         │
│       ↕             ↕               │
│  Agent5:8004 ←→ ... ←→ Agent10:8009 │
│                                     │
└─────────────────────────────────────┘
```

### Performance Characteristics

- **Latency**: <1ms (loopback)
- **Convergence**: 3-5 rounds = 30-50ms
- **Bandwidth**: <1 MB/s
- **Cost**: Free (local)

### Use Cases

✅ Local testing and debugging
✅ Unit tests and integration tests
✅ Proof-of-concept demonstrations
✅ Learning and experimentation

❌ NOT for production
❌ NO security (TLS disabled)

## Topology 2: Single-Region Production Mesh

**Use Case**: Production deployments in single cloud region

### Configuration

```yaml
topology: single_region_mesh
agent_count: 100-1000
regions:
  - name: us-east-1
    location: AWS us-east-1
    zones:
      - us-east-1a
      - us-east-1b
      - us-east-1c
    agents: 1000

gossip:
  fanout: 10
  interval_ms: 100
  convergence_threshold: 0.99

network:
  transport: QUIC
  tls: true
  cert_rotation_hours: 24
  port: 9000

bootstrap:
  seeds:
    - mesh-seed-1.us-east-1.internal:9000
    - mesh-seed-2.us-east-1.internal:9000
    - mesh-seed-3.us-east-1.internal:9000

security:
  mtls: true
  signature_algo: ed25519
  rate_limit_per_peer: 1000
  byzantine_threshold: 0.5

observability:
  otel_endpoint: otel-collector.internal:4317
  trace_sample_rate: 0.01
  metrics_interval_sec: 10
```

### Deployment Diagram

```
┌───────────────────────────────────────────────┐
│         AWS Region: us-east-1                 │
│                                               │
│  ┌──────────────┐  ┌──────────────┐          │
│  │   AZ: 1a     │  │   AZ: 1b     │          │
│  │ Agents 1-333 │  │ Agents 334-  │          │
│  │              │←→│   666        │          │
│  └──────────────┘  └──────────────┘          │
│         ↕                 ↕                   │
│  ┌──────────────┐  ┌──────────────┐          │
│  │   AZ: 1c     │  │  Bootstrap   │          │
│  │ Agents 667-  │  │  Seeds (3)   │          │
│  │   1000       │←→│              │          │
│  └──────────────┘  └──────────────┘          │
│                                               │
└───────────────────────────────────────────────┘
```

### Infrastructure

**Compute**:
- EC2 instances: c6i.xlarge (4 vCPU, 8GB RAM)
- Auto-scaling: 100-1000 agents
- Spot instances for cost optimization

**Networking**:
- VPC with private subnets
- NAT gateway for outbound
- Security groups: port 9000 intra-VPC only

**Observability**:
- OpenTelemetry Collector
- Prometheus for metrics
- Jaeger for distributed tracing
- Grafana dashboards

### Performance Characteristics

- **Latency**: <10ms intra-region
- **Convergence**: 5-7 rounds = 50-70ms
- **Bandwidth**: ~10 MB/s total
- **Cost**: ~$500/month (1000 agents)

### Use Cases

✅ Production SaaS applications
✅ Enterprise internal tools
✅ High-throughput workflows
✅ Real-time collaboration

## Topology 3: Multi-Region with Leaders

**Use Case**: Global production, multi-region redundancy

### Configuration

```yaml
topology: multi_region_leaders
agent_count: 1000-100000
regions:
  - name: us-east-1
    location: AWS us-east-1
    agents: 30000
    leader_count: 3

  - name: eu-west-1
    location: AWS eu-west-1
    agents: 30000
    leader_count: 3

  - name: ap-southeast-1
    location: AWS ap-southeast-1
    agents: 40000
    leader_count: 3

gossip:
  intra_region:
    fanout: 20
    interval_ms: 100

  inter_region:
    fanout: 3  # Leaders only
    interval_ms: 500

network:
  transport: QUIC
  tls: true
  global_accelerator: true  # AWS Global Accelerator

leaders:
  election: raft
  heartbeat_ms: 100
  failover_timeout_ms: 1000

bootstrap:
  seeds_per_region: 5
  global_seeds:
    - mesh-seed-global-1.example.com:9000
    - mesh-seed-global-2.example.com:9000

security:
  mtls: true
  signature_algo: ed25519
  rate_limit_per_peer: 5000
  byzantine_threshold: 0.6
  encryption: AES-256-GCM

observability:
  otel_endpoints:
    - us-east-1: otel-us.internal:4317
    - eu-west-1: otel-eu.internal:4317
    - ap-southeast-1: otel-ap.internal:4317
  trace_sample_rate: 0.001
  metrics_interval_sec: 30
```

### Deployment Diagram

```
┌─────────────────────────────────────────────────┐
│                                                 │
│         AWS Global Accelerator                  │
│                                                 │
└──────────────┬──────────────────────────────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
    ↓          ↓          ↓
┌────────┐ ┌────────┐ ┌────────┐
│US-East │ │EU-West │ │AP-SE   │
│30k     │ │30k     │ │40k     │
│agents  │ │agents  │ │agents  │
│        │ │        │ │        │
│Leader  │←│Leader  │←│Leader  │
│  L1,L2,│→│  L4,L5,│→│  L7,L8,│
│  L3    │ │  L6    │ │  L9    │
└────────┘ └────────┘ └────────┘
```

### Regional Mesh

Each region runs independent gossip mesh:

```
Region: us-east-1 (30k agents)

  ┌─────────────────────────────────┐
  │  Gossip Mesh (30k agents)       │
  │  Fanout: k=20                   │
  │  Convergence: ~8 rounds         │
  └─────────────────────────────────┘
              ↕
  ┌─────────────────────────────────┐
  │  Leaders (L1, L2, L3)           │
  │  - Elected via Raft             │
  │  - Gossip to EU/APAC leaders    │
  └─────────────────────────────────┘
```

### Inter-Region Gossip

Leaders gossip between regions:

```
L1 (US) ←→ L4 (EU) ←→ L7 (APAC)
   ↕         ↕         ↕
L2 (US) ←→ L5 (EU) ←→ L8 (APAC)
   ↕         ↕         ↕
L3 (US) ←→ L6 (EU) ←→ L9 (APAC)

Latency: 50-100ms inter-region
Convergence: ~10 rounds = 5 seconds
```

### Performance Characteristics

- **Intra-region latency**: <10ms
- **Inter-region latency**: 50-100ms
- **Global convergence**: <5s
- **Bandwidth**: ~100 MB/s total
- **Cost**: ~$50,000/month (100k agents)

### Use Cases

✅ Global SaaS platforms
✅ Multi-region disaster recovery
✅ Low-latency worldwide access
✅ Compliance (data residency)

## Topology 4: Hierarchical (1M agents)

**Use Case**: Massive enterprise deployments

### Configuration

```yaml
topology: hierarchical
agent_count: 100000-1000000

hierarchy:
  levels: 3

  level1_global:
    coordinators: 10
    regions: 10

  level2_regional:
    aggregators_per_region: 5
    agents_per_aggregator: 20000

  level3_edge:
    edge_nodes_per_aggregator: 10
    agents_per_edge: 2000

gossip:
  edge_to_edge: 10
  edge_to_aggregator: 5
  aggregator_to_coordinator: 3

  intervals:
    edge_ms: 100
    aggregator_ms: 500
    global_ms: 1000

network:
  transport: QUIC
  tls: true
  compression: zstd
  batching: true
  batch_size: 100

bootstrap:
  global_coordinators:
    - coord-1.global.mesh.example.com:9000
    - coord-2.global.mesh.example.com:9000
    - coord-3.global.mesh.example.com:9000

security:
  mtls: true
  signature_algo: ed25519
  rate_limit_per_peer: 10000
  byzantine_threshold: 0.7
  encryption: AES-256-GCM
  key_rotation_hours: 12

observability:
  otel_collector_per_region: true
  aggregation: true
  trace_sample_rate: 0.0001
  metrics_interval_sec: 60
  dashboards:
    - global_overview
    - regional_health
    - edge_performance
```

### Deployment Diagram

```
                    ┌──────────────────┐
                    │  Global Coords   │
                    │  (GC1...GC10)    │
                    └────────┬─────────┘
                             │
      ┌──────────────────────┼──────────────────────┐
      │                      │                      │
      ↓                      ↓                      ↓
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ Region 1    │      │ Region 2    │      │ Region 10   │
│ Aggregators │      │ Aggregators │      │ Aggregators │
│ (RA1...RA5) │      │ (RA6...RA10)│ ...  │ (RA46..RA50)│
└──────┬──────┘      └──────┬──────┘      └──────┬──────┘
       │                    │                    │
  ┌────┼────┐          ┌────┼────┐          ┌────┼────┐
  ↓    ↓    ↓          ↓    ↓    ↓          ↓    ↓    ↓
Edge Edge Edge      Edge Edge Edge      Edge Edge Edge
Nodes Nodes Nodes  Nodes Nodes Nodes  Nodes Nodes Nodes
(2k) (2k) (2k)     (2k) (2k) (2k)     (2k) (2k) (2k)

Total: 10 regions × 5 aggregators × 10 edges × 2k agents
     = 1,000,000 agents
```

### Hierarchical Gossip Flow

**Level 3 (Edge Nodes)**:
```
2000 agents per edge node
↓
Gossip within edge (k=10, ~7 rounds)
↓
Aggregated state to Level 2
```

**Level 2 (Regional Aggregators)**:
```
Receive states from 10 edge nodes (20k agents)
↓
Merge via CRDT
↓
Gossip aggregated state to Global Coordinators
```

**Level 1 (Global Coordinators)**:
```
Receive states from 50 aggregators (1M agents)
↓
Merge via CRDT
↓
Global state available
↓
Gossip back down hierarchy (updates)
```

### State Aggregation Example

```yaml
# Edge Node E1 (2000 agents)
state:
  workflow_executions: 5000
  active_tasks: 1200
  completion_rate: 0.95
  avg_latency_ms: 45

# Regional Aggregator RA1 (20k agents from 10 edges)
aggregated_state:
  workflow_executions: 50000
  active_tasks: 12000
  completion_rate: 0.94  # Weighted average
  avg_latency_ms: 47     # Weighted average
  edge_nodes: [E1, E2, ..., E10]

# Global Coordinator GC1 (1M agents from 50 aggregators)
global_state:
  workflow_executions: 2500000
  active_tasks: 600000
  completion_rate: 0.93
  avg_latency_ms: 50
  regions: 10
  aggregators: 50
  edges: 500
```

### Performance Characteristics

- **Edge latency**: <10ms (intra-edge)
- **Regional latency**: <100ms (edge → aggregator)
- **Global latency**: <1s (aggregator → global)
- **Total convergence**: ~12-15 rounds = 5-10 seconds
- **Bandwidth**: ~1 GB/s total
- **Cost**: ~$500,000/month (1M agents)

### Infrastructure

**Compute**:
- Edge nodes: c6i.2xlarge (8 vCPU, 16GB)
- Aggregators: c6i.4xlarge (16 vCPU, 32GB)
- Global coords: c6i.8xlarge (32 vCPU, 64GB)

**Networking**:
- AWS Transit Gateway for inter-region
- Direct Connect for low latency
- Global Accelerator for edge access

**Storage**:
- DynamoDB for state persistence
- S3 for telemetry archives
- ElastiCache for hot state

**Observability**:
- Managed Prometheus
- AWS X-Ray for tracing
- CloudWatch for aggregated metrics
- Custom Grafana dashboards

### Use Cases

✅ Fortune 500 enterprise
✅ Global supply chain coordination
✅ Massive simulation systems
✅ National-scale infrastructure

## Topology Comparison Matrix

| Feature | Flat Mesh | Single-Region | Multi-Region | Hierarchical |
|---------|-----------|---------------|--------------|--------------|
| **Max agents** | 100 | 1,000 | 100,000 | 1,000,000 |
| **Regions** | 1 | 1 | 2-5 | 5-10 |
| **Convergence** | <50ms | <100ms | <5s | <10s |
| **Latency SLA** | N/A | 99.9% | 99.5% | 99% |
| **Cost/month** | $0 | $500 | $50k | $500k |
| **Setup time** | 5 min | 1 hour | 1 day | 1 week |
| **Ops complexity** | Low | Medium | High | Very High |

## Migration Paths

### Small → Single-Region

1. Deploy bootstrap seeds in target region
2. Start agents with new config
3. Gradually migrate workflows
4. Decommission local mesh

### Single-Region → Multi-Region

1. Deploy regional seeds in new regions
2. Elect regional leaders
3. Configure inter-region gossip
4. Enable geographic routing
5. Validate convergence metrics

### Multi-Region → Hierarchical

1. Deploy regional aggregators
2. Group edge nodes (2k agents each)
3. Deploy global coordinators
4. Configure hierarchical gossip
5. Enable state aggregation
6. Validate global convergence

## Related Documents

- `ADR-001-MESH-NETWORK-ARCHITECTURE.md` - Architecture decisions
- `SYSTEM-ARCHITECTURE.md` - Component design
- `WEAVER-SCHEMA.yaml` - Observability schema
- `RUNBOOK.md` - Operations and troubleshooting
