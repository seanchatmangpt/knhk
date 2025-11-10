# Validate GitHub Actions with act

## Overview

Use `act` (https://github.com/nektos/act) to run and validate GitHub Actions workflows locally before pushing to GitHub.

## Installation

### macOS
```bash
brew install act
```

### Linux
```bash
# Using the install script
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

### Windows
See https://github.com/nektos/act#installation

## Usage

### List Available Workflows

```bash
./scripts/validate_github_actions_with_act.sh --list
```

Or directly with act:
```bash
act -l
```

### Validate Specific Workflow

```bash
./scripts/validate_github_actions_with_act.sh --workflow van-der-aalst-validation.yml
```

Or directly with act:
```bash
act -W .github/workflows/van-der-aalst-validation.yml
```

### Dry Run (See What Would Run)

```bash
./scripts/validate_github_actions_with_act.sh --workflow van-der-aalst-validation.yml --dry-run
```

Or directly with act:
```bash
act -W .github/workflows/van-der-aalst-validation.yml --list
```

### Run All Workflows

```bash
./scripts/validate_github_actions_with_act.sh
```

Or directly with act:
```bash
act
```

## Van der Aalst Validation Workflow

### Test Locally

```bash
# List jobs in the workflow
act -W .github/workflows/van-der-aalst-validation.yml --list

# Run the workflow
act -W .github/workflows/van-der-aalst-validation.yml

# Run with specific inputs (workflow_dispatch)
act -W .github/workflows/van-der-aalst-validation.yml \
  --eventpath <(echo '{"inputs":{"spec_id":"test-123","phase":"fitness"}}')
```

### Expected Behavior

The workflow will:
1. ✅ Checkout code
2. ✅ Install Rust toolchain
3. ✅ Cache cargo dependencies
4. ✅ Build workflow engine
5. ✅ Build CLI
6. ✅ Run validation framework tests
7. ⚠️  Run validation (if spec_id provided) - may fail if no workflows registered
8. ⚠️  Upload artifacts - may fail locally (GitHub-specific)
9. ⚠️  Comment on PR - will fail locally (requires GitHub API)

### Limitations

Some features won't work locally:
- ❌ GitHub API calls (PR comments, artifacts)
- ❌ Secrets and environment variables (unless configured)
- ❌ Platform-specific differences (act uses Docker)
- ⚠️  Some actions may not work exactly as on GitHub

### Workarounds

1. **Skip GitHub-specific steps**: Use `continue-on-error: true` for steps that require GitHub API
2. **Mock secrets**: Use `.secrets` file or environment variables
3. **Test core logic**: Focus on building and testing, not GitHub integrations

## Example: Validate Van der Aalst Workflow

```bash
# 1. List what would run
act -W .github/workflows/van-der-aalst-validation.yml --list

# 2. Run the workflow
act -W .github/workflows/van-der-aalst-validation.yml

# 3. Check results
ls -la rust/knhk-cli/tmp/validation/
```

## Integration with Development Workflow

### Pre-commit Validation

Add to `scripts/pre-commit-hook.sh`:

```bash
# Validate GitHub Actions workflows
if git diff --cached --name-only | grep -qE '\.github/workflows/.*\.yml$'; then
    echo "Validating GitHub Actions workflows..."
    ./scripts/validate_github_actions_with_act.sh --workflow $(git diff --cached --name-only | grep '\.github/workflows/.*\.yml$' | head -1 | xargs basename)
fi
```

### Pre-push Validation

Add to `scripts/pre-push-hook.sh`:

```bash
# Validate all GitHub Actions workflows
echo "Validating GitHub Actions workflows..."
./scripts/validate_github_actions_with_act.sh --dry-run
```

## Troubleshooting

### act Not Found

```bash
# Install act
brew install act  # macOS
# or see https://github.com/nektos/act#installation
```

### Docker Not Running

```bash
# Start Docker
docker ps  # Should work without errors
```

### Workflow Fails Locally

Some failures are expected:
- GitHub API calls won't work locally
- Secrets may not be available
- Some actions may behave differently

Focus on:
- ✅ Workflow syntax validation
- ✅ Build steps
- ✅ Test execution
- ✅ Core logic validation

### Platform Differences

act uses Docker, so there may be differences:
- File paths
- Environment variables
- Platform-specific behavior

Test critical paths, but don't expect 100% parity.

## Status

**Status**: ✅ READY - act integration complete

**Next Steps**:
- Add to pre-commit hooks
- Integrate with CI/CD
- Document workflow-specific notes

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ READY - Validate GitHub Actions with act




