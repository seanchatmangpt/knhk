# KNHK Phases 6-10: Deployment Models

**Status**: 🔵 DESIGN | **Version**: 1.0.0 | **Date**: 2025-11-18

---

## Overview

This document specifies deployment models for KNHK Phases 6-10 across different infrastructure scenarios: single-node development, multi-region production, hybrid cloud, and edge deployment.

---

## Deployment Model 1: Single-Node Development

**Use Case**: Local development, testing, small-scale deployments

```
┌─────────────────────────────────────────────────────────┐
│           Single-Node KNHK Instance                      │
│                                                           │
│  License: Free / Pro                                    │
│  Hardware: CPU + SIMD (optional GPU for Pro)            │
│  Consensus: None (Free) / Local only (Pro)              │
│  Storage: SQLite (local)                                 │
│                                                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │  KNHK Workflow Engine                            │   │
│  │                                                   │   │
│  │  - Phase 6: Q-Learning (tabular, CPU)           │   │
│  │  - Phase 7: Classical signatures (Ed25519)      │   │
│  │  - Phase 8: No consensus (single node)          │   │
│  │  - Phase 9: CPU + SIMD (if available)           │   │
│  │  - Phase 10: Free/Pro license                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Storage                                         │   │
│  │  - SQLite: workflow definitions                 │   │
│  │  - sled: experience replay buffer               │   │
│  │  - File: audit log                              │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘

Performance:
- Latency: <10ms (local SQLite)
- Throughput: 1,000 workflows/sec
- Neural learning: CPU-only (slow convergence)
- No Byzantine fault tolerance
```

**Configuration**:
```yaml
# config/single-node.yaml
license:
  tier: pro
  max_workflows: 100
  max_concurrent: 10

neural:
  enabled: true
  model: qlearning
  backend: cpu

crypto:
  mode: classical  # Ed25519

consensus:
  enabled: false  # Single node

hardware:
  accelerator: simd  # AVX-512 if available
  fallback: cpu

storage:
  engine: sqlite
  path: /var/lib/knhk/workflows.db
```

---

## Deployment Model 2: Multi-Region Production (3 Regions)

**Use Case**: Enterprise production with Byzantine fault tolerance

```
┌────────────── Region: US-EAST ──────────────┐
│  Node 1 (Leader)                            │
│  - Enterprise License                       │
│  - GPU: NVIDIA A100                         │
│  - PBFT Primary                             │
│  - Neural: Actor-Critic (GPU)               │
│  - Crypto: Hybrid (Ed25519+Dilithium3)     │
└─────────────────────────────────────────────┘
                  ↓ ↑
            (Consensus Messages)
                  ↓ ↑
┌────────────── Region: EU-WEST ──────────────┐
│  Node 2 (Replica)                           │
│  - Enterprise License                       │
│  - GPU: NVIDIA A100                         │
│  - PBFT Replica                             │
│  - Neural: Actor-Critic (GPU)               │
│  - Crypto: Hybrid (Ed25519+Dilithium3)     │
└─────────────────────────────────────────────┘
                  ↓ ↑
            (Consensus Messages)
                  ↓ ↑
┌────────────── Region: AP-SOUTH ─────────────┐
│  Node 3 (Replica)                           │
│  - Enterprise License                       │
│  - GPU: NVIDIA A100                         │
│  - PBFT Replica                             │
│  - Neural: Actor-Critic (GPU)               │
│  - Crypto: Hybrid (Ed25519+Dilithium3)     │
└─────────────────────────────────────────────┘

Consensus Protocol: PBFT (f=1, n=3)
- Tolerates 1 Byzantine node
- Requires 2 nodes for commit (2f+1=2)
- Global consensus latency: ~250ms
- Local operations: ≤8 ticks

Performance:
- Latency: 250ms (global consensus), <1ms (local)
- Throughput: 100,000 workflows/sec (GPU-accelerated)
- Neural learning: GPU-accelerated (fast convergence)
- Byzantine fault tolerance: f=1 (33% nodes can fail)
```

**Configuration**:
```yaml
# config/multi-region.yaml
license:
  tier: enterprise
  max_workflows: unlimited
  max_concurrent: unlimited

neural:
  enabled: true
  model: actor_critic
  backend: gpu
  distributed: true

crypto:
  mode: hybrid  # Ed25519 + Dilithium3

consensus:
  enabled: true
  protocol: pbft
  nodes:
    - id: 1
      region: us-east
      endpoint: https://us-east.knhk.example.com:8443
    - id: 2
      region: eu-west
      endpoint: https://eu-west.knhk.example.com:8443
    - id: 3
      region: ap-south
      endpoint: https://ap-south.knhk.example.com:8443
  max_byzantine: 1
  timeout: 5s

hardware:
  accelerator: gpu
  device: cuda
  fallback: simd

storage:
  engine: postgresql
  replicas: 3
  consensus_log: true
```

**Network Requirements**:
- Bandwidth: 1 Gbps minimum between regions
- Latency: <100ms between regions (for <250ms consensus)
- Encryption: TLS 1.3 with Kyber KEM (quantum-safe)

---

## Deployment Model 3: Hybrid Cloud (On-Prem + Cloud)

**Use Case**: Enterprise with on-prem FPGA + cloud GPU fleet

```
┌─────────── On-Premises ─────────────┐
│                                      │
│  ┌────────────────────────────────┐ │
│  │  KNHK Core Node (FPGA)         │ │
│  │                                 │ │
│  │  - Enterprise License          │ │
│  │  - FPGA: Xilinx Alveo U280     │ │
│  │  - Hardware: FPGA (ultra-low   │ │
│  │    latency hot path)           │ │
│  │  - Consensus: HotStuff (fast)  │ │
│  │  - Neural: Inference only      │ │
│  │  - Crypto: Hybrid signatures   │ │
│  └────────────────────────────────┘ │
│                                      │
└──────────────────────────────────────┘
            ↓ ↑
     (Hybrid Connection)
            ↓ ↑
┌─────────── Cloud (AWS/Azure) ───────┐
│                                      │
│  ┌────────────────────────────────┐ │
│  │  GPU Fleet (Training)          │ │
│  │                                 │ │
│  │  - 10x NVIDIA A100 GPUs        │ │
│  │  - Neural: Distributed training│ │
│  │  - Model updates → On-prem     │ │
│  │  - S3: Experience replay       │ │
│  └────────────────────────────────┘ │
│                                      │
│  ┌────────────────────────────────┐ │
│  │  Consensus Replicas (2 nodes)  │ │
│  │                                 │ │
│  │  - HotStuff replicas           │ │
│  │  - Backup for on-prem FPGA     │ │
│  └────────────────────────────────┘ │
└──────────────────────────────────────┘

Hybrid Architecture:
- On-prem FPGA: Hot path execution (≤1μs latency)
- Cloud GPU: Neural network training (100x faster)
- Consensus: 1 on-prem + 2 cloud (f=1, n=3)
- Data: Sensitive data on-prem, training data in cloud

Performance:
- Latency: <1μs (FPGA hot path), ~10ms (cloud replica)
- Throughput: 1,000,000 workflows/sec (FPGA)
- Neural learning: GPU fleet (1000x faster training)
- Hybrid consensus: ~50ms (on-prem + cloud)
```

**Configuration**:
```yaml
# config/hybrid-cloud.yaml
license:
  tier: enterprise

hardware:
  primary: fpga
  fpga:
    device: /dev/xdma0
    bitstream: /opt/knhk/fpga/workflow_engine.bit
  gpu:
    fleet: cloud
    endpoint: https://gpu-fleet.knhk.example.com

neural:
  inference:
    backend: fpga
    latency: 1us
  training:
    backend: cloud_gpu_fleet
    replicas: 10

consensus:
  protocol: hotstuff
  nodes:
    - id: 1
      location: on_prem
      endpoint: fpga://local
    - id: 2
      location: cloud
      endpoint: https://cloud-1.knhk.example.com
    - id: 3
      location: cloud
      endpoint: https://cloud-2.knhk.example.com

storage:
  on_prem:
    engine: postgres
    encryption: true
  cloud:
    engine: s3
    bucket: knhk-experience-replay
    region: us-east-1
```

---

## Deployment Model 4: Edge Deployment (IoT/5G)

**Use Case**: Low-latency edge computing with minimal hardware

```
┌──────────── Edge Node ────────────┐
│  (Factory Floor / 5G Edge Server) │
│                                    │
│  License: Pro                     │
│  Hardware: ARM64 + Neon SIMD      │
│  Consensus: Raft (crash-fault)    │
│  Storage: Embedded (sled)         │
│                                    │
│  ┌──────────────────────────────┐ │
│  │  KNHK Lite Engine            │ │
│  │                               │ │
│  │  - Phase 6: Q-Learning (CPU) │ │
│  │  - Phase 7: Classical crypto │ │
│  │  - Phase 8: Raft (3 nodes)   │ │
│  │  - Phase 9: Neon SIMD (ARM)  │ │
│  │  - Phase 10: Pro license     │ │
│  └──────────────────────────────┘ │
└────────────────────────────────────┘
            ↓ ↑
    (Low-bandwidth uplink)
            ↓ ↑
┌──────────── Cloud (Central) ──────┐
│  - Aggregate telemetry            │
│  - Model updates                  │
│  - Centralized training           │
└────────────────────────────────────┘

Edge Characteristics:
- Low latency: <5ms (local decisions)
- Bandwidth-limited: <1 Mbps uplink
- Intermittent connectivity: Eventual consistency
- Resource-constrained: ARM CPU, 4GB RAM

Performance:
- Latency: <5ms (edge), ~500ms (cloud)
- Throughput: 100 workflows/sec (edge)
- Neural learning: Periodic model sync from cloud
- Consensus: Raft (crash-fault only, simpler than PBFT)
```

**Configuration**:
```yaml
# config/edge.yaml
license:
  tier: pro

hardware:
  accelerator: simd
  simd: neon  # ARM64

neural:
  enabled: true
  model: qlearning
  backend: cpu
  sync:
    enabled: true
    interval: 3600s  # Hourly sync with cloud
    endpoint: https://cloud.knhk.example.com

crypto:
  mode: classical  # Ed25519 (smaller sigs for bandwidth)

consensus:
  protocol: raft
  nodes: 3
  election_timeout: 1s

storage:
  engine: sled
  path: /var/lib/knhk/edge.db
  max_size: 1GB  # Embedded storage

telemetry:
  batch_size: 1000
  upload_interval: 300s  # Every 5 min
```

---

## Comparison Matrix

| Model | License | Hardware | Consensus | Latency | Throughput | Cost |
|-------|---------|----------|-----------|---------|------------|------|
| Single-Node | Free/Pro | CPU+SIMD | None | <10ms | 1K/s | $ |
| Multi-Region | Enterprise | 3× GPU | PBFT (f=1) | ~250ms | 100K/s | $$$$ |
| Hybrid Cloud | Enterprise | FPGA+GPU | HotStuff | <1μs | 1M/s | $$$$$ |
| Edge | Pro | ARM+Neon | Raft | <5ms | 100/s | $$ |

---

## Infrastructure Requirements

### Single-Node
```
CPU: 4 cores (8+ recommended)
RAM: 8GB (16GB+ for neural learning)
Disk: 50GB SSD
Network: 100 Mbps
OS: Linux (Ubuntu 22.04+)
```

### Multi-Region (per node)
```
CPU: 32 cores
RAM: 128GB
GPU: NVIDIA A100 (80GB)
Disk: 1TB NVMe SSD
Network: 10 Gbps
OS: Linux (Ubuntu 22.04)
CUDA: 12.0+
```

### Hybrid Cloud (on-prem node)
```
CPU: 64 cores
RAM: 256GB
FPGA: Xilinx Alveo U280
Disk: 2TB NVMe SSD
Network: 40 Gbps
OS: Linux (Ubuntu 22.04)
Xilinx Runtime: 2023.2+
```

### Edge
```
CPU: ARM64 (4 cores, e.g., Raspberry Pi 4)
RAM: 4GB
Disk: 32GB eMMC
Network: 1 Gbps LAN, <1 Mbps WAN
OS: Linux (Raspberry Pi OS / Ubuntu 22.04 ARM)
```

---

## Deployment Automation

### Docker Compose (Single-Node)

```yaml
# docker-compose.yaml
version: '3.8'

services:
  knhk-engine:
    image: knhk/workflow-engine:latest
    environment:
      - LICENSE_TIER=pro
      - NEURAL_ENABLED=true
      - HARDWARE_ACCELERATOR=simd
    volumes:
      - ./data:/var/lib/knhk
      - ./config:/etc/knhk
    ports:
      - "8080:8080"  # HTTP API
      - "8443:8443"  # gRPC
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
```

### Kubernetes (Multi-Region)

```yaml
# k8s/knhk-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: knhk-consensus
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knhk
  template:
    metadata:
      labels:
        app: knhk
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchLabels:
                app: knhk
            topologyKey: topology.kubernetes.io/region
      containers:
      - name: knhk-engine
        image: knhk/workflow-engine:enterprise
        env:
        - name: LICENSE_TIER
          value: "enterprise"
        - name: CONSENSUS_PROTOCOL
          value: "pbft"
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        resources:
          requests:
            nvidia.com/gpu: 1
          limits:
            nvidia.com/gpu: 1
            memory: 128Gi
            cpu: "32"
```

---

## Monitoring & Observability

All deployment models export telemetry to:

```yaml
observability:
  otlp:
    endpoint: https://otel-collector.example.com:4317
    protocol: grpc

  metrics:
    prometheus:
      enabled: true
      port: 9090

  tracing:
    jaeger:
      enabled: true
      endpoint: https://jaeger.example.com:14250

  logs:
    loki:
      enabled: true
      endpoint: https://loki.example.com:3100
```

**Weaver Validation** (all models):
```bash
# Static schema validation
weaver registry check -r registry/phases_6_10/

# Live runtime validation
weaver registry live-check \
  --registry registry/phases_6_10/ \
  --endpoint http://localhost:8080/telemetry
```

---

## Migration Paths

### Free → Pro
```
1. Install Pro license key
2. Restart KNHK engine
3. Enable neural learning (optional)
4. Enable SIMD acceleration (automatic)
5. Verify: weaver registry live-check
```

### Pro → Enterprise
```
1. Install Enterprise license key
2. Add GPU nodes (2 additional regions)
3. Enable Byzantine consensus (PBFT/HotStuff)
4. Enable hybrid signatures (Ed25519+Dilithium3)
5. Enable GPU acceleration
6. Verify: Full consensus test
```

### Single-Node → Multi-Region
```
1. Deploy 2 additional nodes (different regions)
2. Configure consensus (PBFT or HotStuff)
3. Replicate storage (PostgreSQL streaming replication)
4. Enable Byzantine fault tolerance
5. Migrate workflows gradually (blue-green deployment)
```

---

## Disaster Recovery

### Backup Strategy

**Single-Node**:
```bash
# Daily SQLite backup
sqlite3 /var/lib/knhk/workflows.db ".backup '/backup/knhk-$(date +%Y%m%d).db'"
```

**Multi-Region**:
```bash
# Consensus log replication (automatic)
# PostgreSQL streaming replication to 3 regions
# Point-in-time recovery (PITR) enabled
```

### Recovery Procedures

**Node Failure (Multi-Region)**:
1. Detect failure (health check timeout)
2. Promote replica to leader (automatic, HotStuff)
3. Consensus continues with n-1 nodes
4. Replace failed node when available

**Total Failure (All Regions)**:
1. Restore from latest checkpoint (consensus log)
2. Replay transactions since checkpoint
3. Verify state with Weaver validation
4. Resume operations

---

## Security Hardening

### Network Security
- TLS 1.3 for all inter-node communication
- Quantum-safe KEM (Kyber) for key exchange
- mTLS with hybrid certificates (Ed25519+Dilithium3)
- Firewall rules (allow only consensus ports)

### Access Control
- RBAC for API access
- ABAC for workflow permissions (Phase 5)
- License-based feature gates (Phase 10)
- Audit logging for all operations

### Data Protection
- Encryption at rest (AES-256-GCM)
- Encryption in transit (TLS 1.3)
- Secret key zeroization (Zeroize trait)
- Memory protection (mlock for keys)

---

## Related Documents

- `PHASES_6-10_ARCHITECTURE_OVERVIEW.md`
- `PHASE_INTEGRATION_ARCHITECTURE.md`
- `DOCTRINE_COVENANT.md`
- Each phase specification document

**Conclusion**: KNHK Phases 6-10 support flexible deployment from edge to enterprise multi-region, all maintaining DOCTRINE compliance and performance constraints.
