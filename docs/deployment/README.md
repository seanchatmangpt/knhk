# KNHK Production Deployment Documentation

This directory contains comprehensive production deployment, validation, and troubleshooting documentation for KNHK.

---

## Quick Navigation

### 🚀 Deployment

- **[Production Deployment Guide](production-deployment-guide.md)** - Complete step-by-step deployment procedures
  - Pre-deployment validation
  - Infrastructure requirements
  - Deployment steps
  - Post-deployment verification
  - Rollback procedures

### ✅ Validation

- **[Weaver Validation Guide](weaver-validation-guide.md)** - Comprehensive Weaver validation guide
  - Why Weaver is the source of truth
  - Schema validation (`weaver registry check`)
  - Runtime validation (`weaver registry live-check`)
  - Continuous validation setup
  - Troubleshooting validation failures

### 🔧 Troubleshooting

- **[Troubleshooting Guide](troubleshooting-guide.md)** - Production issue diagnosis and resolution
  - Diagnostic hierarchy
  - Common issues and solutions
  - Performance troubleshooting
  - Weaver validation failures
  - Telemetry issues
  - Emergency procedures

### 📊 Certification

- **[Production Certification Report](PRODUCTION_CERTIFICATION_REPORT.md)** - Infrastructure certification status
  - Validation infrastructure summary
  - Schema coverage
  - Script capabilities
  - Documentation completeness
  - Next steps for runtime validation

---

## Validation Hierarchy

KNHK follows a strict 3-level validation hierarchy:

```
LEVEL 1: Weaver Validation (Source of Truth)
├─ Schema check: Proves schema is well-formed
└─ Live-check: Proves runtime matches schema
   ↓
LEVEL 2: Compilation & Code Quality (Baseline)
├─ cargo build: Proves code compiles
├─ cargo clippy: Proves code quality
└─ Pattern checks: Proves no unsafe patterns
   ↓
LEVEL 3: Traditional Tests (Supporting Evidence)
├─ cargo test: Proves test logic works
├─ Integration tests: Proves components integrate
└─ Performance tests: Proves performance targets met
```

**Key Principle**: If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.

---

## Quick Start

### 1. Pre-Deployment Validation

```bash
# Run complete production readiness check
./scripts/production-readiness-comprehensive.sh

# Expected: All CRITICAL checks pass
```

### 2. Schema Validation

```bash
# Validate Weaver registry schemas
weaver registry check -r registry/

# Expected: 0 policy violations, all schemas valid
```

### 3. Deploy to Production

```bash
# Follow step-by-step guide
cat docs/deployment/production-deployment-guide.md

# Key steps:
# 1. Prepare environment
# 2. Deploy binaries
# 3. Deploy configuration
# 4. Start services
# 5. Run smoke tests
```

### 4. Runtime Validation

```bash
# Validate runtime telemetry matches schemas
weaver registry live-check --registry /etc/knhk/registry/

# Expected: All declared spans/metrics found, 0 violations
```

---

## Key Files

### Validation Scripts

Located in `/home/user/knhk/scripts/`:

- `weaver-validation-comprehensive.sh` - Complete Weaver validation (schema + runtime)
- `production-readiness-comprehensive.sh` - Full Definition of Done validation
- `run-weaver-check.sh` - Basic Weaver validation with port conflict resolution

### Registry Schemas

Located in `/home/user/knhk/registry/`:

- `registry_manifest.yaml` - Main registry manifest
- `knhk-workflow-engine.yaml` - Workflow telemetry (43 patterns)
- `knhk-operation.yaml` - Hot path operations (R1 ≤8 ticks)
- `knhk-mape-k.yaml` - MAPE-K autonomic loops
- `knhk-guards.yaml` - Guard constraint enforcement
- `knhk-receipts.yaml` - Cryptographic receipts
- `knhk-etl.yaml` - ETL pipeline telemetry
- `knhk-warm.yaml` - Warm path operations (W1)
- `knhk-sidecar.yaml` - gRPC sidecar service

### CI/CD Integration

Located in `/home/user/knhk/.github/workflows/`:

- `weaver-validation.yml` - Automated validation on every push/PR

---

## Documentation Philosophy

This documentation follows KNHK's core principles:

1. **"Only Weaver validation is truth"** - Weaver is the source of truth for production readiness
2. **No false positives** - Tests can lie; telemetry schemas don't
3. **Schema-first validation** - Code must conform to declared schemas
4. **Constitutional validation** - All formal laws must be verified

---

## Getting Help

### Common Issues

See [Troubleshooting Guide](troubleshooting-guide.md) for:
- Service won't start
- High memory/CPU usage
- Chatman Constant violations
- Weaver validation failures
- Telemetry issues
- Workflow execution problems

### Emergency Procedures

See [Troubleshooting Guide - Emergency Procedures](troubleshooting-guide.md#emergency-procedures) for:
- Emergency rollback
- Circuit breaker activation
- Force shutdown

### Collect Diagnostics

```bash
# Run diagnostic collection script (from troubleshooting guide)
./scripts/collect-diagnostics.sh

# Share the generated tarball when reporting issues
```

---

## Documentation Structure

```
docs/deployment/
├── README.md (this file)
├── production-deployment-guide.md (~450 lines)
├── weaver-validation-guide.md (~600 lines)
├── troubleshooting-guide.md (~650 lines)
└── PRODUCTION_CERTIFICATION_REPORT.md (certification status)
```

---

## Validation Infrastructure Status

✅ **COMPLETE** - All infrastructure delivered

- ✅ 11 Weaver registry schemas
- ✅ 3 comprehensive validation scripts
- ✅ 1,700+ lines of deployment documentation
- ✅ CI/CD integration (GitHub Actions)
- ✅ Emergency procedures documented
- ✅ Troubleshooting guide complete

⏸️ **PENDING** - Runtime validation

- Requires deployed application
- Run: `weaver registry live-check --registry registry/`
- Validates actual telemetry matches schemas

---

## References

- **KNHK Project**: /home/user/knhk/
- **CLAUDE.md**: Project configuration and principles
- **FALSE_POSITIVES_REALITY_CHECK.md**: False positive detection
- **REPOSITORY_OVERVIEW.md**: System architecture
- **OpenTelemetry Weaver**: https://github.com/open-telemetry/weaver

---

**Status**: Infrastructure Certified ✅
**Last Updated**: 2025-11-16
**Validator**: Production Validation Agent
