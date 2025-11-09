# KNHK v1.0 Troubleshooting Guide

**Version**: 1.0.0  
**Status**: Production Troubleshooting Guide  
**Last Updated**: 2025-11-09

---

## Overview

This guide provides troubleshooting procedures for common issues encountered when running KNHK v1.0 in production.

**Critical Principle**: "Never trust the text, only trust test results" - All troubleshooting must be verified through tests and OTEL validation.

---

## Quick Diagnosis

### Health Check Commands

```bash
# Service status
systemctl status knhk

# Health endpoint
curl http://localhost:8080/health

# Workflow list
knhk workflow list

# REST API endpoints
curl http://localhost:8080/health
curl -X POST http://localhost:8080/workflows -H "Content-Type: application/json" -d '{"spec": {...}}'
curl -X POST http://localhost:8080/cases -H "Content-Type: application/json" -d '{"spec_id": "...", "data": {...}}'
curl http://localhost:8080/cases/{id}
curl -X POST http://localhost:8080/cases/{id}/execute
curl http://localhost:8080/cases/{id}/history

# Performance benchmark
knhk performance benchmark

# State verification
knhk state verify
```

---

## Common Issues

### Issue 1: Service Won't Start

#### Symptoms

- `systemctl status knhk` shows `failed`
- Service exits immediately after start
- No logs generated

#### Diagnosis

```bash
# Check service status
systemctl status knhk

# Check logs
journalctl -u knhk -n 100 --no-pager

# Check permissions
ls -la /var/lib/knhk
ls -la /usr/local/bin/knhk

# Check configuration
knhk config validate
```

#### Common Causes

1. **Permission Issues**
   ```bash
   # Fix permissions
   sudo chown -R knhk:knhk /var/lib/knhk
   sudo chmod 755 /var/lib/knhk
   ```

2. **Missing Dependencies**
   ```bash
   # Check Rust installation
   rustc --version
   cargo --version
   
   # Check system libraries
   ldd /usr/local/bin/knhk
   ```

3. **Configuration Errors**
   ```bash
   # Validate configuration
   knhk config validate
   
   # Check environment variables
   systemctl show knhk | grep Environment
   ```

#### Resolution

```bash
# Fix permissions
sudo chown -R knhk:knhk /var/lib/knhk

# Fix configuration
knhk config fix

# Restart service
sudo systemctl daemon-reload
sudo systemctl restart knhk
```

---

### Issue 2: Performance Degradation

#### Symptoms

- Slow workflow execution
- High CPU usage
- High memory usage
- Timeout errors

#### Diagnosis

```bash
# Check hot path performance
knhk performance benchmark

# Check resource usage
top -p $(pgrep knhk)
ps aux | grep knhk

# Check state store size
du -sh /var/lib/knhk
ls -lh /var/lib/knhk

# Check disk I/O
iostat -x 1

# Check network
netstat -an | grep 8080
```

#### Common Causes

1. **State Store Too Large**
   ```bash
   # Check state store size
   du -sh /var/lib/knhk
   
   # Archive old cases
   knhk workflow archive --older-than 90d
   ```

2. **Memory Leak**
   ```bash
   # Check memory usage over time
   watch -n 1 'ps aux | grep knhk'
   
   # Check for memory leaks
   valgrind --leak-check=full ./target/release/knhk
   ```

3. **Hot Path Violations**
   ```bash
   # Check hot path tick count
   knhk performance benchmark
   
   # Verify tick budget compliance
   cargo test --test hot_path_performance
   ```

#### Resolution

```bash
# Optimize configuration
knhk config set cache_size_mb 2048
knhk config set hot_path_max_ticks 8

# Clean up old data
knhk workflow archive --older-than 90d

# Restart service
sudo systemctl restart knhk
```

---

### Issue 3: OTEL Not Working

#### Symptoms

- No spans in OTEL collector
- Missing metrics
- No traces in Jaeger

#### Diagnosis

```bash
# Check OTEL collector status
systemctl status otel-collector

# Check network connectivity
telnet localhost 4317
nc -zv localhost 4317

# Review collector logs
journalctl -u otel-collector -f

# Check KNHK OTEL configuration
knhk config show | grep OTEL

# Test OTEL endpoint
curl http://localhost:4317/health
```

#### Common Causes

1. **OTEL Collector Not Running**
   ```bash
   # Start OTEL collector
   sudo systemctl start otel-collector
   ```

2. **Network Connectivity Issues**
   ```bash
   # Check firewall
   sudo iptables -L | grep 4317
   
   # Check network configuration
   ip addr show
   ```

3. **Configuration Mismatch**
   ```bash
   # Verify OTEL endpoint
   knhk config show | grep OTEL_ENDPOINT
   
   # Check collector configuration
   cat /etc/otel-collector/config.yaml
   ```

#### Resolution

```bash
# Restart OTEL collector
sudo systemctl restart otel-collector

# Verify KNHK OTEL configuration
export KNHK_OTEL_ENDPOINT=http://localhost:4317
knhk config set otel_endpoint http://localhost:4317

# Restart KNHK service
sudo systemctl restart knhk

# Verify spans
tail -f /var/log/otel-collector.log | grep knhk
```

---

### Issue 4: Fortune 5 Features Not Working

#### Symptoms

- SPIFFE authentication fails
- KMS operations fail
- Multi-region sync fails

#### Diagnosis

```bash
# Check SPIFFE status
knhk fortune5 spiffe status

# Check KMS status
knhk fortune5 kms status

# Check multi-region status
knhk fortune5 multi-region status

# Check environment variables
env | grep KNHK
```

#### Common Causes

1. **SPIFFE Not Configured**
   ```bash
   # Check SPIRE agent status
   systemctl status spire-agent
   
   # Check SPIFFE socket
   ls -la /tmp/spire-agent.sock
   ```

2. **KMS Access Denied**
   ```bash
   # Check AWS credentials
   aws sts get-caller-identity
   
   # Check KMS key permissions
   aws kms describe-key --key-id <key-id>
   ```

3. **Multi-Region Network Issues**
   ```bash
   # Check network connectivity
   ping <region-endpoint>
   
   # Check DNS resolution
   nslookup <region-endpoint>
   ```

#### Resolution

```bash
# Fix SPIFFE
export KNHK_SPIFFE_SOCKET=/tmp/spire-agent.sock
export KNHK_SPIFFE_TRUST_DOMAIN=example.org

# Fix KMS
export KNHK_KMS_PROVIDER=aws
export KNHK_KMS_REGION=us-east-1
export KNHK_KMS_KEY_ID=arn:aws:kms:us-east-1:123456789012:key/abc123

# Fix multi-region
export KNHK_REGIONS=us-east-1,us-west-2,eu-west-1
export KNHK_PRIMARY_REGION=us-east-1

# Restart service
sudo systemctl restart knhk
```

---

### Issue 5: State Store Corruption

#### Symptoms

- Cases missing or corrupted
- State verification fails
- Workflow execution errors

#### Diagnosis

```bash
# Verify state store integrity
knhk state verify

# Check for corrupted cases
knhk workflow list --verify

# Check state store files
ls -lh /var/lib/knhk
find /var/lib/knhk -type f -name "*.corrupt"
```

#### Common Causes

1. **Disk Full**
   ```bash
   # Check disk space
   df -h /var/lib/knhk
   ```

2. **File System Errors**
   ```bash
   # Check file system
   fsck /dev/<device>
   ```

3. **Power Failure**
   ```bash
   # Check for incomplete writes
   find /var/lib/knhk -name "*.tmp"
   ```

#### Resolution

```bash
# Stop service
sudo systemctl stop knhk

# Restore from backup
sudo rm -rf /var/lib/knhk/*
tar -xzf knhk-state-backup-*.tar.gz -C /

# Verify restoration
knhk state verify

# Start service
sudo systemctl start knhk
```

---

## Advanced Troubleshooting

### Debug Mode

#### Enable Debug Logging

```bash
# Set debug log level
export KNHK_LOG_LEVEL=debug

# Restart service
sudo systemctl restart knhk

# Monitor debug logs
journalctl -u knhk -f | grep -i debug
```

#### Enable Trace Logging

```bash
# Set trace log level
export KNHK_LOG_LEVEL=trace

# Restart service
sudo systemctl restart knhk

# Monitor trace logs
journalctl -u knhk -f | grep -i trace
```

### Performance Profiling

#### CPU Profiling

```bash
# Install perf
sudo apt-get install linux-perf

# Profile KNHK
sudo perf record -p $(pgrep knhk) -g sleep 60
sudo perf report
```

#### Memory Profiling

```bash
# Install valgrind
sudo apt-get install valgrind

# Profile memory
valgrind --leak-check=full --show-leak-kinds=all ./target/release/knhk
```

### Network Troubleshooting

#### Check Network Connectivity

```bash
# Check OTEL endpoint
curl http://localhost:4317/health

# Check REST API
curl http://localhost:8080/health

# Check gRPC
grpc_health_probe -addr=localhost:50051
```

#### Check Firewall Rules

```bash
# Check iptables
sudo iptables -L -n | grep 8080
sudo iptables -L -n | grep 4317

# Check firewalld
sudo firewall-cmd --list-all
```

---

## Diagnostic Tools

### State Store Analysis

```bash
# Analyze state store
knhk state analyze --output state-analysis.json

# Check case distribution
knhk state stats --cases

# Check event distribution
knhk state stats --events
```

### Performance Analysis

```bash
# Generate performance report
knhk performance report --output performance-report.json

# Analyze hot path
knhk performance analyze --hot-path

# Analyze warm path
knhk performance analyze --warm-path
```

### Validation Analysis

```bash
# Run Van der Aalst validation
./scripts/van_der_aalst_validation.sh

# Run XES conformance validation
./scripts/xes_validation.sh

# Run DoD validation
./scripts/dod_validation.sh
```

---

## Emergency Procedures

### Service Outage

1. **Immediate Actions**
   ```bash
   # Check service status
   systemctl status knhk
   
   # Check logs
   journalctl -u knhk -n 100
   
   # Check resource usage
   top -p $(pgrep knhk)
   ```

2. **Quick Recovery**
   ```bash
   # Restart service
   sudo systemctl restart knhk
   
   # Verify recovery
   curl http://localhost:8080/health
   ```

3. **Full Recovery**
   ```bash
   # Stop service
   sudo systemctl stop knhk
   
   # Restore from backup
   tar -xzf knhk-state-backup-*.tar.gz -C /
   
   # Start service
   sudo systemctl start knhk
   ```

### Data Loss

1. **Immediate Actions**
   ```bash
   # Stop service
   sudo systemctl stop knhk
   
   # Create emergency backup
   tar -czf knhk-emergency-backup-$(date +%Y%m%d_%H%M%S).tar.gz /var/lib/knhk
   ```

2. **Recovery**
   ```bash
   # Restore from latest backup
   tar -xzf knhk-state-backup-*.tar.gz -C /
   
   # Verify restoration
   knhk state verify
   
   # Start service
   sudo systemctl start knhk
   ```

---

## Related Documentation

- [Operations Guide](./OPERATIONS_GUIDE.md)
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Architecture Guide](./ARCHITECTURE_GUIDE.md)
- [Implementation Guide](./IMPLEMENTATION_GUIDE.md)

---

## Notes

- All troubleshooting must be verified through tests
- OTEL validation is the source of truth for telemetry
- Regular backups are critical for disaster recovery
- Document all troubleshooting steps for future reference

