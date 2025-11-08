# Fortune 5 Deployment Guide

## Overview

This guide provides instructions for deploying the KNHK Workflow Engine in Fortune 5 enterprise environments with production-grade features.

## Features

### Enterprise-Grade APIs
- **REST API**: Full REST API with OpenAPI/Swagger documentation
- **gRPC API**: High-performance gRPC service for internal communication
- **Health Checks**: `/health`, `/health/ready`, `/health/live` endpoints
- **OpenAPI**: `/openapi.json` and `/swagger` endpoints

### Security
- **Authentication**: JWT token validation (FUTURE: SPIFFE/SPIRE integration)
- **Authorization**: RBAC-based access control
- **Audit Logging**: Comprehensive audit trail for all operations
- **TLS/mTLS**: End-to-end encryption support

### Observability
- **OTEL Integration**: Distributed tracing with OpenTelemetry
- **Metrics**: Prometheus-compatible metrics
- **Logging**: Structured logging with context
- **Health Monitoring**: Component-level health checks

### Resilience
- **Circuit Breakers**: Automatic failure detection and recovery
- **Rate Limiting**: Per-client rate limiting
- **Retry Logic**: Exponential backoff retry
- **Dead Letter Queue**: Failed message handling

### Scalability
- **Horizontal Scaling**: Multi-instance deployment
- **State Management**: Distributed state store (FUTURE: Redis/etcd)
- **Load Balancing**: Round-robin, least-connections, consistent hashing
- **Multi-Region**: Cross-region replication support

## Deployment

### Prerequisites

1. **Kubernetes Cluster**: 1.24+ with RBAC enabled
2. **State Store**: Sled (local) or Redis/etcd (distributed)
3. **OTEL Collector**: For observability (optional)
4. **Load Balancer**: For external access

### Configuration

#### Environment Variables

```bash
# State Store
WORKFLOW_STATE_PATH=/var/lib/workflow-engine

# API Server
WORKFLOW_API_HOST=0.0.0.0
WORKFLOW_API_PORT=8080

# Fortune 5 Features
WORKFLOW_FORTUNE5_ENABLED=true
WORKFLOW_OTEL_ENDPOINT=http://otel-collector:4317
WORKFLOW_AUDIT_ENABLED=true
WORKFLOW_RATE_LIMIT_ENABLED=true

# Security
WORKFLOW_AUTH_ENABLED=true
WORKFLOW_SPIFFE_ENABLED=false  # FUTURE
WORKFLOW_TRUST_DOMAIN=spiffe://knhk.org

# Scalability
WORKFLOW_MULTI_REGION=false
WORKFLOW_REGION=us-east-1
WORKFLOW_INSTANCE_ID=workflow-engine-1
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-workflow-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knhk-workflow-engine
  template:
    metadata:
      labels:
        app: knhk-workflow-engine
    spec:
      containers:
      - name: workflow-engine
        image: knhk/workflow-engine:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: WORKFLOW_STATE_PATH
          value: /var/lib/workflow-engine
        - name: WORKFLOW_FORTUNE5_ENABLED
          value: "true"
        - name: WORKFLOW_OTEL_ENDPOINT
          value: "http://otel-collector:4317"
        volumeMounts:
        - name: workflow-state
          mountPath: /var/lib/workflow-engine
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "500m"
      volumes:
      - name: workflow-state
        persistentVolumeClaim:
          claimName: workflow-state-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: knhk-workflow-engine
spec:
  selector:
    app: knhk-workflow-engine
  ports:
  - port: 8080
    targetPort: 8080
    name: http
  type: LoadBalancer
```

### Health Checks

#### Liveness Probe
```bash
curl http://localhost:8080/health/live
# Returns: {"alive": true}
```

#### Readiness Probe
```bash
curl http://localhost:8080/health/ready
# Returns: {"ready": true}
```

#### Health Check
```bash
curl http://localhost:8080/health
# Returns: {
#   "status": "Healthy",
#   "service": "knhk-workflow-engine",
#   "version": "1.0.0"
# }
```

## API Usage

### Register Workflow

```bash
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "spec": {
      "id": "workflow-123",
      "name": "Example Workflow",
      "tasks": [],
      "conditions": []
    }
  }'
```

### Create Case

```bash
curl -X POST http://localhost:8080/api/v1/cases \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "spec_id": "workflow-123",
    "data": {"input": "value"}
  }'
```

### Execute Pattern

```bash
curl -X POST http://localhost:8080/api/v1/patterns/1/execute \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "variables": {"test": "value"}
  }'
```

## Monitoring

### Metrics

Prometheus metrics are available at `/metrics` (FUTURE: implement metrics endpoint).

### Tracing

OTEL traces are sent to the configured OTEL collector endpoint.

### Logging

Structured logs are emitted with context:
- `workflow_id`: Workflow identifier
- `case_id`: Case identifier
- `pattern_id`: Pattern identifier
- `operation`: Operation name
- `duration_ms`: Operation duration

## Security

### Authentication

Currently supports JWT token validation. SPIFFE/SPIRE integration is planned for future releases.

### Authorization

RBAC policies control access to workflows, cases, and patterns.

### Audit Logging

All API operations are logged with:
- User identity
- Operation type
- Resource accessed
- Success/failure status
- Timestamp

## Performance

### Hot Path Optimization

Patterns 1-5 (basic control flow) are optimized for ≤8 tick execution (≤2ns per operation).

### Caching

Workflow specifications and pattern executors are cached in memory.

### Rate Limiting

Per-client rate limiting prevents abuse:
- Default: 100 requests/second
- Configurable per client

## Troubleshooting

### Engine Not Ready

Check health endpoint:
```bash
curl http://localhost:8080/health/ready
```

### High Latency

Check metrics and traces for bottlenecks:
- Pattern execution time
- State store latency
- Network latency

### Authentication Failures

Verify token format:
```bash
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/v1/workflows
```

## Future Enhancements

1. **SPIFFE/SPIRE Integration**: mTLS authentication
2. **Distributed State**: Redis/etcd backend
3. **Multi-Region Replication**: Cross-region state sync
4. **Advanced Metrics**: Custom Prometheus metrics
5. **Webhook Support**: Event-driven workflows
6. **GraphQL API**: Alternative query interface

