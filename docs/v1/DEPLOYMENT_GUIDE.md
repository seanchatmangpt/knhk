# KNHK v1.0 Deployment Guide

**Version**: 1.0.0  
**Status**: Production Deployment Guide  
**Last Updated**: 2025-11-09

---

## Overview

This guide provides detailed deployment procedures for KNHK v1.0 in production environments, including Fortune 5 enterprise deployments.

**Critical Principle**: "Never trust the text, only trust test results" - All deployments must be verified through tests and OTEL validation.

---

## Pre-Deployment Checklist

### System Requirements

- [ ] **Operating System**: Linux (x86_64, ARM64) or macOS (x86_64, ARM64)
- [ ] **Rust**: 1.75+ (stable) installed and verified
- [ ] **Memory**: Minimum 4GB RAM, 8GB+ recommended
- [ ] **Storage**: 10GB+ free space for build artifacts
- [ ] **Network**: Outbound connectivity for OTEL, KMS, SPIFFE (if enabled)
- [ ] **Permissions**: Write access to state store directory

### Dependencies

- [ ] **Build Tools**: `make`, `gcc`/`clang` installed
- [ ] **System Libraries**: OpenSSL, zlib installed
- [ ] **Optional**: Docker (for containerized deployment)

### Fortune 5 Features (if enabled)

- [ ] **SPIFFE/SPIRE**: SPIRE agent installed and configured
- [ ] **KMS**: AWS KMS access configured (or other KMS provider)
- [ ] **Multi-Region**: Network connectivity to all regions configured
- [ ] **SLO Monitoring**: Prometheus/Grafana stack configured

---

## Deployment Methods

### Method 1: Direct Binary Deployment

#### Step 1: Build Production Binary

```bash
cd rust
cargo build --release --features fortune5
```

#### Step 2: Install Binary

```bash
sudo cp target/release/knhk /usr/local/bin/
sudo chmod +x /usr/local/bin/knhk
sudo chown knhk:knhk /usr/local/bin/knhk
```

#### Step 3: Create State Store Directory

```bash
sudo mkdir -p /var/lib/knhk
sudo chown knhk:knhk /var/lib/knhk
sudo chmod 755 /var/lib/knhk
```

#### Step 4: Create Systemd Service

Create `/etc/systemd/system/knhk.service`:

```ini
[Unit]
Description=KNHK Workflow Engine
After=network.target

[Service]
Type=simple
User=knhk
Group=knhk
WorkingDirectory=/var/lib/knhk
ExecStart=/usr/local/bin/knhk
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Environment variables
Environment="KNHK_LOG_LEVEL=info"
Environment="KNHK_STATE_STORE_PATH=/var/lib/knhk"
Environment="KNHK_WORKFLOW_DB_PATH=/var/lib/knhk/workflow_db"

# Fortune 5 features (if enabled)
Environment="KNHK_FORTUNE5_ENABLED=true"
Environment="KNHK_SPIFFE_ENABLED=true"
Environment="KNHK_KMS_ENABLED=true"
Environment="KNHK_MULTI_REGION_ENABLED=true"

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

#### Step 5: Start Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable knhk
sudo systemctl start knhk
sudo systemctl status knhk
```

### Method 2: Container Deployment

#### Step 1: Build Container Image

```bash
docker build -t knhk:v1.0.0 -f Dockerfile .
```

#### Step 2: Run Container

```bash
docker run -d \
  --name knhk \
  --restart unless-stopped \
  -p 8080:8080 \
  -v /var/lib/knhk:/var/lib/knhk \
  -e KNHK_LOG_LEVEL=info \
  -e KNHK_FORTUNE5_ENABLED=true \
  knhk:v1.0.0
```

#### Step 3: Verify Deployment

```bash
docker logs knhk
docker exec knhk knhk workflow list
curl http://localhost:8080/health
```

### Method 3: Kubernetes Deployment

#### Step 1: Create ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: knhk-config
data:
  KNHK_LOG_LEVEL: "info"
  KNHK_STATE_STORE_PATH: "/var/lib/knhk"
  KNHK_FORTUNE5_ENABLED: "true"
```

#### Step 2: Create Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk
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
      containers:
      - name: knhk
        image: knhk:v1.0.0
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: knhk-config
        volumeMounts:
        - name: state-store
          mountPath: /var/lib/knhk
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
      volumes:
      - name: state-store
        persistentVolumeClaim:
          claimName: knhk-state-store
```

#### Step 3: Create Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: knhk
spec:
  selector:
    app: knhk
  ports:
  - port: 8080
    targetPort: 8080
  type: LoadBalancer
```

---

## Fortune 5 Enterprise Features

### SPIFFE/SPIRE Integration

#### Prerequisites

1. **Install SPIRE**
   ```bash
   # Follow SPIRE installation guide
   # https://spiffe.io/docs/latest/spire-about/getting-started/
   ```

2. **Configure SPIRE Agent**
   ```bash
   export KNHK_SPIFFE_SOCKET=/tmp/spire-agent.sock
   export KNHK_SPIFFE_TRUST_DOMAIN=example.org
   ```

3. **Verify SPIFFE**
   ```bash
   knhk workflow list --spiffe-enabled
   ```

### KMS Integration

#### AWS KMS Setup

1. **Create KMS Key**
   ```bash
   aws kms create-key --description "KNHK Workflow Engine Key"
   ```

2. **Configure KMS**
   ```bash
   export KNHK_KMS_PROVIDER=aws
   export KNHK_KMS_REGION=us-east-1
   export KNHK_KMS_KEY_ID=arn:aws:kms:us-east-1:123456789012:key/abc123
   ```

3. **Test KMS**
   ```bash
   knhk fortune5 kms rotate
   ```

### Multi-Region Support

1. **Configure Regions**
   ```bash
   export KNHK_REGIONS=us-east-1,us-west-2,eu-west-1
   export KNHK_PRIMARY_REGION=us-east-1
   ```

2. **Test Multi-Region**
   ```bash
   knhk fortune5 multi-region sync
   ```

---

## Post-Deployment Validation

### Health Checks

1. **Service Health**
   ```bash
   curl http://localhost:8080/health
   ```

2. **Workflow Engine Status**
   ```bash
   knhk workflow list
   ```

3. **REST API Endpoints**
   ```bash
   # Health check
   curl http://localhost:8080/health
   
   # Register workflow
   curl -X POST http://localhost:8080/workflows -H "Content-Type: application/json" -d '{"spec": {...}}'
   
   # Create case
   curl -X POST http://localhost:8080/cases -H "Content-Type: application/json" -d '{"spec_id": "...", "data": {...}}'
   ```

4. **OTEL Instrumentation**
   ```bash
   # Check OTEL collector logs
   tail -f /var/log/otel-collector.log
   ```

### Performance Validation

1. **Hot Path Performance**
   ```bash
   cargo test --test hot_path_performance
   ```

2. **Tick Budget Verification**
   ```bash
   # Verify ≤8 ticks for hot path operations
   knhk performance benchmark
   ```

### Security Validation

1. **Security Audit**
   ```bash
   cargo audit
   ```

2. **Secret Scanning**
   ```bash
   # Verify no hardcoded secrets
   grep -r "password\|secret\|key" --exclude-dir=target
   ```

---

## Monitoring & Observability

### OTEL Configuration

1. **OTEL Collector Config**
   ```yaml
   receivers:
     otlp:
       protocols:
         grpc:
           endpoint: 0.0.0.0:4317
   
   exporters:
     logging:
       loglevel: info
     prometheus:
       endpoint: "0.0.0.0:8889"
   
   service:
     pipelines:
       traces:
         receivers: [otlp]
         exporters: [logging, prometheus]
   ```

2. **Verify Spans**
   ```bash
   # Check OTEL collector logs for spans
   tail -f /var/log/otel-collector.log | grep knhk
   ```

### Prometheus Metrics

1. **Metrics Endpoint**
   ```bash
   curl http://localhost:8080/metrics
   ```

2. **Key Metrics**
   - `knhk_workflow_cases_total`
   - `knhk_workflow_tasks_completed_total`
   - `knhk_workflow_execution_duration_seconds`
   - `knhk_hot_path_ticks`

### Grafana Dashboards

1. **Import Dashboard**
   ```bash
   # Import KNHK dashboard JSON
   grafana-cli admin import-dashboard knhk-dashboard.json
   ```

2. **Key Panels**
   - Workflow execution rate
   - Task completion rate
   - Hot path tick distribution
   - Error rate

---

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   # Check logs
   journalctl -u knhk -f
   
   # Check permissions
   ls -la /var/lib/knhk
   
   # Check configuration
   knhk config validate
   ```

2. **OTEL Not Working**
   ```bash
   # Check OTEL collector status
   systemctl status otel-collector
   
   # Check network connectivity
   telnet localhost 4317
   
   # Review collector logs
   journalctl -u otel-collector -f
   ```

3. **Performance Issues**
   ```bash
   # Check hot path tick count
   knhk performance benchmark
   
   # Check memory usage
   top -p $(pgrep knhk)
   
   # Check disk I/O
   iostat -x 1
   ```

4. **Fortune 5 Features Not Working**
   ```bash
   # Verify SPIFFE
   knhk fortune5 spiffe status
   
   # Verify KMS
   knhk fortune5 kms status
   
   # Verify Multi-Region
   knhk fortune5 multi-region status
   ```

---

## Rollback Procedures

### Quick Rollback

1. **Stop Service**
   ```bash
   sudo systemctl stop knhk
   ```

2. **Restore Previous Version**
   ```bash
   git checkout <previous-version>
   cargo build --release
   sudo cp target/release/knhk /usr/local/bin/
   ```

3. **Start Service**
   ```bash
   sudo systemctl start knhk
   ```

### Data Rollback

1. **Backup State Store**
   ```bash
   tar -czf knhk-state-backup.tar.gz /var/lib/knhk
   ```

2. **Restore State Store**
   ```bash
   sudo systemctl stop knhk
   sudo rm -rf /var/lib/knhk/*
   tar -xzf knhk-state-backup.tar.gz -C /
   sudo systemctl start knhk
   ```

---

## Related Documentation

- [Implementation Guide](./IMPLEMENTATION_GUIDE.md)
- [Architecture Guide](./ARCHITECTURE_GUIDE.md)
- [Operations Guide](./OPERATIONS_GUIDE.md)
- [Definition of Done](./definition-of-done/fortune5-production.md)
- [Release Checklist](./certification/release-checklist.md)

---

## Support

For issues or questions:
- **Documentation**: See [Main Documentation Index](../INDEX.md)
- **Evidence**: See [Evidence Directory](../evidence/)
- **Architecture**: See [Architecture Documentation](../ARCHITECTURE.md)

---

## Notes

- All deployments must pass pre-push validation gates
- DoD criteria must be met before production deployment
- OTEL validation is the source of truth for telemetry
- Test results are authoritative over documentation claims

