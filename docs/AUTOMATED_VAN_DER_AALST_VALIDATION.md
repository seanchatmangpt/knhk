# Automated Van der Aalst Validation

## Overview

Automated validation for the Van der Aalst End-to-End Validation Framework, integrated with CI/CD pipelines, git hooks, and development workflows.

## Automation Scripts

### 1. `scripts/automate_van_der_aalst_validation.sh`

Main automation script for running the complete validation framework.

**Usage**:
```bash
./scripts/automate_van_der_aalst_validation.sh \
  --spec-id <spec_id> \
  [--phase <phase>] \
  [--format <format>] \
  [--output-dir <dir>]
```

**Options**:
- `--spec-id <spec_id>`: Workflow specification ID (required)
- `--phase <phase>`: Run specific phase (fitness, precision, generalization, process_mining, formal)
- `--format <format>`: Report format (markdown, json, html) - default: markdown
- `--output-dir <dir>`: Output directory - default: `./tmp/van_der_aalst_validation`

**Examples**:
```bash
# Run complete validation
./scripts/automate_van_der_aalst_validation.sh --spec-id abc-123-def

# Run specific phase
./scripts/automate_van_der_aalst_validation.sh --spec-id abc-123-def --phase fitness

# Generate JSON report
./scripts/automate_van_der_aalst_validation.sh --spec-id abc-123-def --format json
```

### 2. `scripts/automate_validation_wip.sh`

Automated WIP (Work In Progress) validation for git hooks and development workflows.

**Usage**:
```bash
./scripts/automate_validation_wip.sh [--check-changes] [--validate-all]
```

**Options**:
- `--check-changes`: Check for changed workflow files and validate them
- `--validate-all`: Validate all registered workflows

**Examples**:
```bash
# Check changed workflow files
./scripts/automate_validation_wip.sh --check-changes

# Validate all registered workflows
./scripts/automate_validation_wip.sh --validate-all
```

## CI/CD Integration

### GitHub Actions Workflow

**File**: `.github/workflows/van-der-aalst-validation.yml`

**Triggers**:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual workflow dispatch

**Features**:
- Automatic validation on workflow engine changes
- Validation report artifacts
- PR comments with validation results
- Caching for faster builds

**Usage**:
```yaml
# Manual dispatch with options
workflow_dispatch:
  inputs:
    spec_id: 'abc-123-def'
    phase: 'fitness'
```

### Integration with Existing CI

Add to existing CI workflows:

```yaml
- name: Van der Aalst Validation
  run: |
    ./scripts/automate_van_der_aalst_validation.sh \
      --spec-id $WORKFLOW_SPEC_ID \
      --format json \
      --output-dir ./validation_results
```

## Git Hooks Integration

### Pre-commit Hook

Add to `scripts/pre-commit-hook.sh`:

```bash
# Van der Aalst validation for changed workflows
if git diff --cached --name-only | grep -qE '\.(ttl|turtle)$'; then
    echo "Validating changed workflows..."
    ./scripts/automate_validation_wip.sh --check-changes
fi
```

### Pre-push Hook

Add to `scripts/pre-push-hook.sh`:

```bash
# Run validation framework tests before push
echo "Running validation framework tests..."
cd rust/knhk-workflow-engine
cargo test --test van_der_aalst_framework
```

## Scheduled Validation

### Cron Job

Run validation daily:

```bash
# Add to crontab
0 0 * * * cd /path/to/knhk && ./scripts/automate_van_der_aalst_validation.sh --spec-id <spec_id> --format json
```

### GitHub Actions Scheduled

Add to `.github/workflows/van-der-aalst-validation.yml`:

```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
```

## Validation Status Tracking

### WIP Status

Track validation status in WIP:

```bash
# Check validation status
./scripts/automate_validation_wip.sh --check-changes

# Validate all workflows
./scripts/automate_validation_wip.sh --validate-all
```

### Status Reports

Generate status reports:

```bash
# Generate JSON status report
./scripts/automate_van_der_aalst_validation.sh \
  --spec-id <spec_id> \
  --format json \
  --output-dir ./status_reports

# Parse status
jq '.summary.overall_status' ./status_reports/validation_report.json
```

## Integration Examples

### CI/CD Pipeline

```yaml
name: Validation Pipeline

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build CLI
        run: |
          cd rust/knhk-cli
          cargo build --release
      
      - name: Run Validation
        run: |
          ./scripts/automate_van_der_aalst_validation.sh \
            --spec-id ${{ secrets.WORKFLOW_SPEC_ID }} \
            --format json
      
      - name: Check Validation Status
        run: |
          STATUS=$(jq -r '.summary.overall_status' tmp/van_der_aalst_validation/validation_report.json)
          if [ "$STATUS" != "Pass" ]; then
            echo "Validation failed!"
            exit 1
          fi
```

### Development Workflow

```bash
# Before committing workflow changes
./scripts/automate_validation_wip.sh --check-changes

# Before pushing
./scripts/automate_validation_wip.sh --validate-all

# Manual validation
./scripts/automate_van_der_aalst_validation.sh --spec-id <spec_id>
```

## Status

**Status**: ✅ COMPLETE - Automation scripts and CI/CD integration ready

**Next Steps**:
- Integrate with existing git hooks
- Add scheduled validation
- Expand validation coverage

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ COMPLETE - Automated Van der Aalst Validation

