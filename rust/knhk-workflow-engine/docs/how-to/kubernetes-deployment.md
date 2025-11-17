# How-To: Deploy KNHK Workflow Engine to Kubernetes

**Production-Ready Kubernetes Deployment Guide**

- **Time to Complete**: ~30 minutes
- **Difficulty Level**: Intermediate
- **Prerequisites**: kubectl, Kubernetes cluster (v1.20+), Helm (optional)
- **You'll Learn**: Deploy engine to Kubernetes with persistence and scaling

---

## Table of Contents

1. [Prerequisites & Setup](#prerequisites--setup)
2. [Create Docker Image](#create-docker-image)
3. [Deploy to Kubernetes](#deploy-to-kubernetes)
4. [Configure Persistence](#configure-persistence)
5. [Enable Monitoring](#enable-monitoring)
6. [Scale & Optimize](#scale--optimize)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites & Setup

### Check Your Environment

```bash
# Verify kubectl access
kubectl version --client

# Check cluster is running
kubectl cluster-info

# Check available resources
kubectl get nodes

# Example output:
# NAME           STATUS   ROLES                  AGE   VERSION
# k8s-master-1   Ready    control-plane,master   5d    v1.25.0
# k8s-worker-1   Ready    <none>                 5d    v1.25.0
```

### Install Required Tools

```bash
# Install Docker (if not already)
docker --version

# Install Helm (optional, for easier deployment)
helm version

# Create namespace for KNHK
kubectl create namespace knhk
kubectl config set-context --current --namespace=knhk
```

---

## Create Docker Image

### Step 1: Create Dockerfile

Create file: `Dockerfile`

```dockerfile
# Multi-stage build for smaller image
FROM rust:1.75 as builder

WORKDIR /app

# Copy source
COPY rust/knhk-workflow-engine ./

# Build release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/knhk-workflow /app/knhk-workflow

# Create config directory
RUN mkdir -p /etc/knhk /var/log/knhk /var/lib/knhk

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080 8081

ENTRYPOINT ["/app/knhk-workflow"]
CMD ["--config", "/etc/knhk/config.toml"]
```

### Step 2: Create .dockerignore

```
target/
.git/
.gitignore
*.md
tests/
benches/
examples/
.cargo/
```

### Step 3: Build Image

```bash
# Build Docker image
docker build -t knhk-workflow-engine:1.0.0 .

# Verify image
docker images | grep knhk

# Test locally
docker run -p 8080:8080 knhk-workflow-engine:1.0.0 &
sleep 5
curl http://localhost:8080/health
docker stop <container_id>
```

### Step 4: Push to Registry

```bash
# Tag for registry
docker tag knhk-workflow-engine:1.0.0 \
  registry.example.com/knhk/workflow-engine:1.0.0

# Push to registry
docker push registry.example.com/knhk/workflow-engine:1.0.0

# Verify
docker pull registry.example.com/knhk/workflow-engine:1.0.0
```

---

## Deploy to Kubernetes

### Step 1: Create ConfigMap for Configuration

Create file: `configmap.yaml`

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: knhk-config
  namespace: knhk
data:
  config.toml: |
    [server]
    port = 8080
    bind_address = "0.0.0.0"
    request_timeout = 30

    [execution]
    worker_threads = 8
    max_concurrent_cases = 5000
    execution_model = "eager"

    [storage]
    backend = "postgres"
    persistence_enabled = true
    compression = true

    [storage.postgres]
    connection_string = "${DB_CONNECTION_STRING}"
    pool_size = 20
    ssl_mode = "require"

    [observability.otel]
    enabled = true
    collector_endpoint = "http://otel-collector:4317"
    sampling_rate = 0.5

    [observability.logging]
    level = "info"
    format = "json"
    file_enabled = true
    file_path = "/var/log/knhk/workflow.log"
```

Apply the config:

```bash
kubectl apply -f configmap.yaml
```

### Step 2: Create Secrets for Sensitive Data

```bash
# Create secret for database password
kubectl create secret generic knhk-db-secret \
  --from-literal=connection-string="postgresql://user:password@postgres:5432/knhk" \
  --namespace=knhk

# Create secret for API keys (optional)
kubectl create secret generic knhk-api-keys \
  --from-literal=api-key-1="sk_prod_abcdef123456" \
  --from-literal=api-key-2="sk_prod_xyz789" \
  --namespace=knhk
```

### Step 3: Create Deployment

Create file: `deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-workflow-engine
  namespace: knhk
  labels:
    app: knhk
    component: workflow-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knhk
      component: workflow-engine
  template:
    metadata:
      labels:
        app: knhk
        component: workflow-engine
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8081"
        prometheus.io/path: "/metrics"
    spec:
      # Pod disruption budget for zero-downtime updates
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - knhk
              topologyKey: kubernetes.io/hostname

      containers:
      - name: knhk-workflow
        image: registry.example.com/knhk/workflow-engine:1.0.0
        imagePullPolicy: IfNotPresent

        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: metrics
          containerPort: 8081
          protocol: TCP

        # Environment variables
        env:
        - name: DB_CONNECTION_STRING
          valueFrom:
            secretKeyRef:
              name: knhk-db-secret
              key: connection-string
        - name: KNHK_OBSERVABILITY_OTEL_ENABLED
          value: "true"
        - name: KNHK_OBSERVABILITY_OTEL_COLLECTOR_ENDPOINT
          value: "http://otel-collector.monitoring:4317"
        - name: KNHK_SECURITY_AUTH_ENABLED
          value: "true"
        - name: KNHK_SECURITY_AUTH_API_KEYS
          valueFrom:
            secretKeyRef:
              name: knhk-api-keys
              key: api-key-1
        - name: RUST_LOG
          value: "info"

        # Volume mounts
        volumeMounts:
        - name: config
          mountPath: /etc/knhk
          readOnly: true
        - name: logs
          mountPath: /var/log/knhk
        - name: data
          mountPath: /var/lib/knhk

        # Resource limits
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"

        # Probes for health checking
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2

        # Security context
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
            add:
            - NET_BIND_SERVICE
          readOnlyRootFilesystem: true

      volumes:
      - name: config
        configMap:
          name: knhk-config
      - name: logs
        emptyDir: {}
      - name: data
        persistentVolumeClaim:
          claimName: knhk-data-pvc

      # Termination grace period for graceful shutdown
      terminationGracePeriodSeconds: 30
```

Apply the deployment:

```bash
kubectl apply -f deployment.yaml

# Check deployment status
kubectl get deployment -n knhk
kubectl describe deployment knhk-workflow-engine -n knhk
```

### Step 4: Create Service

Create file: `service.yaml`

```yaml
apiVersion: v1
kind: Service
metadata:
  name: knhk-workflow-engine
  namespace: knhk
  labels:
    app: knhk
    component: workflow-engine
spec:
  type: ClusterIP
  ports:
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
  - name: metrics
    port: 8081
    targetPort: 8081
    protocol: TCP
  selector:
    app: knhk
    component: workflow-engine
```

Apply the service:

```bash
kubectl apply -f service.yaml

# Get service IP
kubectl get svc -n knhk
```

---

## Configure Persistence

### Step 1: Create PersistentVolumeClaim

Create file: `pvc.yaml`

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: knhk-data-pvc
  namespace: knhk
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd  # Use your storage class
```

Apply the PVC:

```bash
kubectl apply -f pvc.yaml

# Check status
kubectl get pvc -n knhk
```

### Step 2: Setup PostgreSQL

```bash
# Deploy PostgreSQL (using Helm)
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres bitnami/postgresql \
  --namespace knhk \
  --set auth.username=knhk_user \
  --set auth.password=$(openssl rand -base64 32) \
  --set primary.persistence.size=50Gi

# Get database connection string
kubectl get secret postgres-secret -n knhk -o jsonpath='{.data.password}' | base64 -d
```

---

## Enable Monitoring

### Step 1: Add Prometheus Scraping

The deployment already includes Prometheus annotations. Deploy Prometheus:

```yaml
# prometheus-servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: knhk-workflow-engine
  namespace: knhk
spec:
  selector:
    matchLabels:
      app: knhk
  endpoints:
  - port: metrics
    interval: 30s
```

### Step 2: Setup OTEL Collector

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: otel-collector-config
  namespace: monitoring
data:
  otel-collector-config.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317

    exporters:
      jaeger:
        endpoint: jaeger-collector:14250
      prometheus:
        endpoint: 0.0.0.0:8888

    service:
      pipelines:
        traces:
          receivers: [otlp]
          exporters: [jaeger]
        metrics:
          receivers: [otlp]
          exporters: [prometheus]
```

---

## Scale & Optimize

### Step 1: Horizontal Pod Autoscaling

Create file: `hpa.yaml`

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: knhk-workflow-engine-hpa
  namespace: knhk
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: knhk-workflow-engine
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

Apply the HPA:

```bash
kubectl apply -f hpa.yaml

# Monitor scaling
kubectl get hpa -n knhk --watch
```

### Step 2: Pod Disruption Budget

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: knhk-pdb
  namespace: knhk
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: knhk
```

---

## Troubleshooting

### Check Pod Status

```bash
# List pods
kubectl get pods -n knhk

# Describe specific pod
kubectl describe pod knhk-workflow-engine-abc123 -n knhk

# Check logs
kubectl logs knhk-workflow-engine-abc123 -n knhk

# Follow logs
kubectl logs -f knhk-workflow-engine-abc123 -n knhk

# Previous logs (if crashed)
kubectl logs knhk-workflow-engine-abc123 -n knhk --previous
```

### Common Issues

#### Issue: Pod is CrashLoopBackOff

**Solution**:
```bash
# Check logs for errors
kubectl logs <pod-name> -n knhk

# Common causes:
# - Config file has errors
# - Database not accessible
# - Out of memory
```

#### Issue: Pod is Pending

**Solution**:
```bash
# Check events
kubectl describe pod <pod-name> -n knhk

# Check node resources
kubectl top nodes
kubectl top pods -n knhk

# May need to add nodes or reduce resource requests
```

#### Issue: Service not accessible

**Solution**:
```bash
# Check service IP
kubectl get svc -n knhk

# Test connectivity
kubectl run -it --rm debug --image=busybox --restart=Never -- \
  sh -c "wget http://knhk-workflow-engine:8080/health"

# Check ingress (if using)
kubectl get ingress -n knhk
```

### Enable Debug Logging

Update ConfigMap:

```bash
kubectl edit configmap knhk-config -n knhk

# Change logging level:
# [observability.logging]
# level = "debug"

# Restart pods to pick up changes
kubectl rollout restart deployment knhk-workflow-engine -n knhk
```

---

## Complete Deployment Script

Create file: `deploy.sh`

```bash
#!/bin/bash
set -e

NAMESPACE="knhk"
REGISTRY="registry.example.com"
IMAGE="knhk/workflow-engine:1.0.0"

echo "Creating namespace..."
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

echo "Applying ConfigMap..."
kubectl apply -f configmap.yaml

echo "Applying PVC..."
kubectl apply -f pvc.yaml

echo "Applying Deployment..."
kubectl apply -f deployment.yaml

echo "Applying Service..."
kubectl apply -f service.yaml

echo "Applying HPA..."
kubectl apply -f hpa.yaml

echo "Applying PDB..."
kubectl apply -f pdb.yaml

echo "Waiting for deployment..."
kubectl rollout status deployment/knhk-workflow-engine -n $NAMESPACE

echo "Checking service..."
kubectl get svc -n $NAMESPACE

echo "Deployment complete!"
kubectl get all -n $NAMESPACE
```

Run the script:

```bash
chmod +x deploy.sh
./deploy.sh
```

---

## Verification

After deployment:

```bash
# Check all resources
kubectl get all -n knhk

# Port forward for testing
kubectl port-forward svc/knhk-workflow-engine 8080:8080 -n knhk

# In another terminal:
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8081/metrics -n knhk
```

---

## Related Documentation

- [Configuration Guide](../reference/configuration.md) - Fine-tune settings
- [API Endpoints Reference](../reference/api-endpoints.md) - Available endpoints
- [How-To: OTEL Observability](./otel-observability.md) - Setup monitoring
- [How-To: Troubleshooting](./troubleshooting.md) - Solve problems
