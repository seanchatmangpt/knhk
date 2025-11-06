# KNHK Kubernetes Deployment

This directory contains Kubernetes deployment manifests for the KNHK platform.

## Files

- `namespace.yaml` - Namespace definition (`kgc`)
- `configmap.yaml` - Configuration for all components
- `service-sidecar.yaml` - Service for KGC Sidecar (ClusterIP)
- `service-warm.yaml` - Service for Warm Orchestrator (ClusterIP)
- `service-cold.yaml` - Service for Cold Reasoner (ClusterIP)
- `app-sidecar-pod.yaml` - Example pod with Enterprise App + KGC Sidecar
- `knhk-core-pod.yaml` - Pod with Warm + Hot + Cold containers
- `deployment.yaml` - Deployment for knhk-core (3 replicas)
- `daemonset-sidecar.yaml` - DaemonSet for sidecar (runs on all nodes)

## Deployment Order

1. Create namespace:
   ```bash
   kubectl apply -f namespace.yaml
   ```

2. Create ConfigMap:
   ```bash
   kubectl apply -f configmap.yaml
   ```

3. Create services:
   ```bash
   kubectl apply -f service-warm.yaml
   kubectl apply -f service-cold.yaml
   kubectl apply -f service-sidecar.yaml
   ```

4. Deploy core components:
   ```bash
   kubectl apply -f deployment.yaml
   ```

5. Deploy sidecar (if using DaemonSet):
   ```bash
   kubectl apply -f daemonset-sidecar.yaml
   ```

6. Deploy example app with sidecar:
   ```bash
   kubectl apply -f app-sidecar-pod.yaml
   ```

## Architecture

### Pod: app+sidecar
- **Enterprise App**: Your application container
- **KGC Sidecar**: Local proxy (gRPC on localhost:50051)

### Pod: knhk-core
- **Warm Orchestrator**: ETL, AOT, scheduling (gRPC on :50052)
- **Hot Path**: C library (.so) shared via volume
- **Cold Reasoner**: SPARQL/SHACL (RPC on :50053)

## Configuration

All configuration is managed via ConfigMap (`knhk-config`):

- `SIDECAR_LISTEN_ADDR`: Sidecar listen address (default: `0.0.0.0:50051`)
- `SIDECAR_BATCH_SIZE`: Batch size (default: `8`, guard: ≤8)
- `SIDECAR_BATCH_TIMEOUT_MS`: Batch timeout (default: `100ms`)
- `WARM_ORCHESTRATOR_ENDPOINT`: Warm orchestrator service endpoint
- `HOT_PATH_LIB_PATH`: Hot path library path
- `COLD_REASONER_ENDPOINT`: Cold reasoner service endpoint
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OTEL collector endpoint

## Resource Limits

### Sidecar
- Requests: 128Mi memory, 50m CPU
- Limits: 256Mi memory, 200m CPU

### Warm Orchestrator
- Requests: 512Mi memory, 200m CPU
- Limits: 1Gi memory, 1000m CPU

### Cold Reasoner
- Requests: 1Gi memory, 500m CPU
- Limits: 2Gi memory, 2000m CPU

## Health Checks

- **Sidecar**: HTTP GET `/health` on port 50051
- **Warm**: TCP socket on port 50052
- **Cold**: TCP socket on port 50053

## Notes

- Hot path library is shared via `emptyDir` volume
- All components use OTEL for observability
- Circuit breaker protects warm orchestrator connection
- Batching accumulates requests (batch_size=8, timeout=100ms)
- Retry logic uses exponential backoff (idempotent operations)

