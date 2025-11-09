# Fortune 5 Deployment Guide

**Status**: Production-Ready

---

## Quick Start

### Prerequisites

- Kubernetes 1.24+ with RBAC
- State store (Sled local or Redis/etcd distributed)
- OTEL Collector (optional)

### Configuration

```bash
# Essential environment variables
WORKFLOW_STATE_PATH=/var/lib/workflow-engine
WORKFLOW_API_HOST=0.0.0.0
WORKFLOW_API_PORT=8080
WORKFLOW_FORTUNE5_ENABLED=true
WORKFLOW_OTEL_ENDPOINT=http://otel-collector:4317
WORKFLOW_AUTH_ENABLED=true
```

---

## Kubernetes Deployment

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

---

## Health Checks

```bash
# Liveness
curl http://localhost:8080/health/live
# Returns: {"alive": true}

# Readiness
curl http://localhost:8080/health/ready
# Returns: {"ready": true}

# Health
curl http://localhost:8080/health
# Returns: {"status": "Healthy", "service": "knhk-workflow-engine"}
```

---

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

---

## Features

### Enterprise APIs
- REST API with OpenAPI/Swagger
- gRPC API (planned)
- Health checks: `/health`, `/health/ready`, `/health/live`

### Security
- JWT authentication
- RBAC authorization
- Audit logging
- TLS/mTLS support

### Observability
- OTEL integration
- Prometheus metrics (planned)
- Structured logging

### Resilience
- Circuit breakers
- Rate limiting
- Retry logic
- Dead letter queue

### Scalability
- Horizontal scaling
- Load balancing
- Multi-region support (planned)

---

## Monitoring

### Metrics
Prometheus metrics available at `/metrics` (planned).

### Tracing
OTEL traces sent to configured collector endpoint.

### Logging
Structured logs with context: `workflow_id`, `case_id`, `operation`, `duration_ms`.

---

## Troubleshooting

### Engine Not Ready
```bash
curl http://localhost:8080/health/ready
```

### High Latency
Check OTEL traces for bottlenecks in pattern execution or state store.

### Authentication Failures
Verify JWT token format:
```bash
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/v1/workflows
```

---

## Performance

- Hot path: ≤8 ticks (Chatman Constant)
- Caching: Workflow specs and pattern executors cached
- Rate limiting: 100 req/s default (configurable)

---

**License**: MIT
