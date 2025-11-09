# KNHK v1.0 Operations Guide

**Version**: 1.0.0  
**Status**: Production Operations Guide  
**Last Updated**: 2025-11-09

---

## Overview

This guide provides operational procedures for running and maintaining KNHK v1.0 in production environments.

**Critical Principle**: "Never trust the text, only trust test results" - All operational procedures must be verified through tests and OTEL validation.

---

## Daily Operations

### Health Monitoring

#### Service Health Check

```bash
# Check service status
systemctl status knhk

# Check health endpoint
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics
```

#### Key Metrics to Monitor

1. **Workflow Execution Rate**
   ```promql
   rate(knhk_workflow_cases_total[5m])
   ```

2. **Task Completion Rate**
   ```promql
   rate(knhk_workflow_tasks_completed_total[5m])
   ```

3. **Error Rate**
   ```promql
   rate(knhk_workflow_errors_total[5m])
   ```

4. **Hot Path Tick Distribution**
   ```promql
   histogram_quantile(0.99, knhk_hot_path_ticks_bucket)
   ```

### Log Monitoring

#### View Recent Logs

```bash
# Systemd service logs
journalctl -u knhk -f

# Last 100 lines
journalctl -u knhk -n 100

# Logs since today
journalctl -u knhk --since today

# Docker logs (if using containers)
docker logs knhk -f
```

#### Key Log Patterns

1. **Errors**
   ```bash
   journalctl -u knhk | grep -i error
   ```

2. **Warnings**
   ```bash
   journalctl -u knhk | grep -i warn
   ```

3. **Performance Issues**
   ```bash
   journalctl -u knhk | grep -i "tick\|performance\|slow"
   ```

---

## Weekly Operations

### Performance Review

#### Hot Path Performance

```bash
# Run hot path benchmark
cargo test --test hot_path_performance --release

# Verify tick budget compliance
knhk performance benchmark
```

#### Resource Usage

```bash
# Check memory usage
ps aux | grep knhk

# Check disk usage
du -sh /var/lib/knhk

# Check CPU usage
top -p $(pgrep knhk)
```

### State Store Maintenance

#### Backup State Store

```bash
# Create backup
tar -czf knhk-state-backup-$(date +%Y%m%d).tar.gz /var/lib/knhk

# Verify backup
tar -tzf knhk-state-backup-*.tar.gz | head -20
```

#### Cleanup Old Data

```bash
# List old cases (older than 90 days)
knhk workflow list --older-than 90d

# Archive old cases
knhk workflow archive --older-than 90d
```

---

## Monthly Operations

### Security Audit

#### Run Security Scan

```bash
# Cargo audit
cargo audit

# Secret scanning
grep -r "password\|secret\|key" --exclude-dir=target

# Dependency check
cargo tree | grep -i "vulnerable\|deprecated"
```

### Performance Optimization

#### Review Performance Metrics

```bash
# Generate performance report
knhk performance report --output performance-report.json

# Analyze hot path performance
knhk performance analyze --hot-path
```

#### Optimize Configuration

```bash
# Review current configuration
knhk config show

# Tune performance parameters
knhk config set hot_path_max_ticks 8
knhk config set warm_path_timeout_ms 100
```

---

## Incident Response

### Service Outage

#### Immediate Actions

1. **Check Service Status**
   ```bash
   systemctl status knhk
   ```

2. **Check Logs**
   ```bash
   journalctl -u knhk -n 100 --no-pager
   ```

3. **Check Resource Usage**
   ```bash
   top -p $(pgrep knhk)
   df -h /var/lib/knhk
   ```

#### Recovery Procedures

1. **Restart Service**
   ```bash
   sudo systemctl restart knhk
   ```

2. **Verify Recovery**
   ```bash
   curl http://localhost:8080/health
   knhk workflow list
   ```

3. **Check OTEL**
   ```bash
   # Verify spans are being generated
   tail -f /var/log/otel-collector.log | grep knhk
   ```

### Performance Degradation

#### Investigation Steps

1. **Check Hot Path Performance**
   ```bash
   knhk performance benchmark
   ```

2. **Check Resource Usage**
   ```bash
   top -p $(pgrep knhk)
   iostat -x 1
   ```

3. **Check State Store**
   ```bash
   du -sh /var/lib/knhk
   ls -lh /var/lib/knhk
   ```

#### Remediation

1. **Scale Resources**
   ```bash
   # Increase memory limit
   systemctl edit knhk
   # Add: LimitMEMLOCK=4G
   ```

2. **Optimize Configuration**
   ```bash
   knhk config set cache_size_mb 1024
   ```

3. **Restart Service**
   ```bash
   sudo systemctl restart knhk
   ```

### Data Corruption

#### Detection

```bash
# Verify state store integrity
knhk state verify

# Check for corrupted cases
knhk workflow list --verify
```

#### Recovery

1. **Restore from Backup**
   ```bash
   sudo systemctl stop knhk
   sudo rm -rf /var/lib/knhk/*
   tar -xzf knhk-state-backup-*.tar.gz -C /
   sudo systemctl start knhk
   ```

2. **Verify Recovery**
   ```bash
   knhk workflow list
   knhk state verify
   ```

---

## Maintenance Windows

### Planned Maintenance

#### Pre-Maintenance Checklist

- [ ] Notify stakeholders
- [ ] Create backup
- [ ] Document current state
- [ ] Prepare rollback plan

#### Maintenance Steps

1. **Stop Service**
   ```bash
   sudo systemctl stop knhk
   ```

2. **Perform Maintenance**
   ```bash
   # Update binary
   cargo build --release
   sudo cp target/release/knhk /usr/local/bin/
   
   # Update configuration
   sudo systemctl edit knhk
   ```

3. **Start Service**
   ```bash
   sudo systemctl start knhk
   ```

4. **Verify Service**
   ```bash
   curl http://localhost:8080/health
   knhk workflow list
   ```

#### Post-Maintenance Verification

- [ ] Service health check passed
- [ ] Workflow execution verified
- [ ] OTEL instrumentation verified
- [ ] Performance benchmarks passed

---

## Backup & Recovery

### Backup Procedures

#### Automated Backups

```bash
# Create backup script
cat > /usr/local/bin/knhk-backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=/var/backups/knhk
DATE=$(date +%Y%m%d_%H%M%S)
tar -czf $BACKUP_DIR/knhk-state-$DATE.tar.gz /var/lib/knhk
# Keep only last 30 days
find $BACKUP_DIR -name "knhk-state-*.tar.gz" -mtime +30 -delete
EOF

chmod +x /usr/local/bin/knhk-backup.sh

# Add to crontab (daily at 2 AM)
echo "0 2 * * * /usr/local/bin/knhk-backup.sh" | crontab -
```

#### Manual Backups

```bash
# Create backup
tar -czf knhk-state-backup-$(date +%Y%m%d).tar.gz /var/lib/knhk

# Verify backup
tar -tzf knhk-state-backup-*.tar.gz | head -20
```

### Recovery Procedures

#### Full Recovery

1. **Stop Service**
   ```bash
   sudo systemctl stop knhk
   ```

2. **Restore Backup**
   ```bash
   sudo rm -rf /var/lib/knhk/*
   tar -xzf knhk-state-backup-*.tar.gz -C /
   ```

3. **Start Service**
   ```bash
   sudo systemctl start knhk
   ```

4. **Verify Recovery**
   ```bash
   knhk workflow list
   knhk state verify
   ```

#### Partial Recovery

1. **Identify Corrupted Data**
   ```bash
   knhk state verify
   ```

2. **Restore Specific Cases**
   ```bash
   knhk workflow restore --case-id <case_id> --from-backup <backup_file>
   ```

---

## Capacity Planning

### Resource Requirements

#### Minimum Requirements

- **CPU**: 2 cores
- **Memory**: 4GB RAM
- **Storage**: 10GB (state store)
- **Network**: 100Mbps

#### Recommended Requirements

- **CPU**: 4+ cores
- **Memory**: 8GB+ RAM
- **Storage**: 50GB+ (state store)
- **Network**: 1Gbps

### Scaling Guidelines

#### Vertical Scaling

```bash
# Increase memory limit
systemctl edit knhk
# Add: LimitMEMLOCK=8G

# Increase CPU limit
systemctl edit knhk
# Add: CPUQuota=400%
```

#### Horizontal Scaling

```bash
# Deploy multiple instances
# Use load balancer
# Configure shared state store
```

---

## Troubleshooting

### Common Issues

#### Service Won't Start

**Symptoms**:
- `systemctl status knhk` shows failed
- Logs show startup errors

**Diagnosis**:
```bash
# Check logs
journalctl -u knhk -n 50

# Check permissions
ls -la /var/lib/knhk

# Check configuration
knhk config validate
```

**Resolution**:
```bash
# Fix permissions
sudo chown -R knhk:knhk /var/lib/knhk

# Fix configuration
knhk config fix

# Restart service
sudo systemctl restart knhk
```

#### Performance Degradation

**Symptoms**:
- Slow workflow execution
- High CPU usage
- High memory usage

**Diagnosis**:
```bash
# Check hot path performance
knhk performance benchmark

# Check resource usage
top -p $(pgrep knhk)

# Check state store size
du -sh /var/lib/knhk
```

**Resolution**:
```bash
# Optimize configuration
knhk config set cache_size_mb 2048

# Clean up old data
knhk workflow archive --older-than 90d

# Restart service
sudo systemctl restart knhk
```

#### OTEL Not Working

**Symptoms**:
- No spans in OTEL collector
- Missing metrics

**Diagnosis**:
```bash
# Check OTEL collector status
systemctl status otel-collector

# Check network connectivity
telnet localhost 4317

# Review collector logs
journalctl -u otel-collector -f
```

**Resolution**:
```bash
# Restart OTEL collector
sudo systemctl restart otel-collector

# Verify KNHK OTEL configuration
knhk config show | grep OTEL

# Restart KNHK service
sudo systemctl restart knhk
```

---

## Related Documentation

- [Implementation Guide](./IMPLEMENTATION_GUIDE.md)
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Architecture Guide](./ARCHITECTURE_GUIDE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING_GUIDE.md)

---

## Notes

- All operational procedures must be verified through tests
- OTEL validation is the source of truth for telemetry
- Regular backups are critical for disaster recovery
- Performance monitoring is essential for capacity planning

