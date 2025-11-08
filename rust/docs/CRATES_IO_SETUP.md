# Crates.io Publishing Setup

## Prerequisites

### 1. Get Crates.io API Token

1. Visit https://crates.io/settings/tokens
2. Click "New Token"
3. Name: `KNHK Publishing Token`
4. Scope: Select `publish-update`
5. Click "Generate"
6. **Copy the token immediately** (you won't see it again)

### 2. Add Token to GitHub Secrets

1. Go to repository settings: `https://github.com/YOUR_ORG/knhk/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `CARGO_TOKEN`
4. Value: Paste the token from step 1
5. Click "Add secret"

### 3. Verify Package Metadata

All crates must have:
- `license` field (MIT/Apache-2.0)
- `description` field
- `repository` field pointing to GitHub
- `readme` field (optional but recommended)

Check with:
```bash
cd rust
./scripts/test-crates-io-install.sh
```

## Publishing Process

### Automated (Recommended)

1. **Update version in all Cargo.toml files**:
   ```bash
   # Edit version in all crates to match (e.g., 1.0.1)
   # Required crates (will be published to crates.io):
   vim rust/knhk-otel/Cargo.toml
   vim rust/knhk-lockchain/Cargo.toml
   vim rust/knhk-hot/Cargo.toml
   vim rust/knhk-etl/Cargo.toml
   vim rust/knhk-warm/Cargo.toml
   vim rust/knhk-config/Cargo.toml
   vim rust/knhk-connectors/Cargo.toml
   vim rust/knhk-cli/Cargo.toml
   ```

2. **Verify versions match**:
   ```bash
   cd rust
   ./scripts/verify-crate-versions.sh
   ```

3. **Test publication dry-run**:
   ```bash
   cd rust
   ./scripts/test-crates-io-install.sh
   ```

4. **Commit version changes**:
   ```bash
   git add rust/*/Cargo.toml
   git commit -m "chore: bump version to 0.1.0"
   git push origin main
   ```

5. **Create and push tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

6. **GitHub Actions will automatically**:
   - Verify version matches tag
   - Publish crates in dependency order
   - Wait for crates.io index updates
   - Verify installation
   - Create GitHub release

### Manual (Fallback)

If you need to publish manually:

```bash
cd rust

# Publish in dependency order
cargo publish --manifest-path knhk-otel/Cargo.toml
sleep 30  # Wait for crates.io index

cargo publish --manifest-path knhk-lockchain/Cargo.toml
sleep 30

cargo publish --manifest-path knhk-hot/Cargo.toml
sleep 30

cargo publish --manifest-path knhk-etl/Cargo.toml
sleep 30

cargo publish --manifest-path knhk-cli/Cargo.toml
```

## Troubleshooting

### "crate not found" errors

**Problem**: Dependent crate hasn't been indexed yet.

**Solution**: Wait 1-2 minutes and retry. Crates.io index updates are not instant.

### Version mismatch errors

**Problem**: Version in Cargo.toml doesn't match git tag.

**Solution**:
```bash
# Check versions
./scripts/verify-crate-versions.sh

# Update if needed
vim knhk-cli/Cargo.toml  # Change version
git commit -am "fix: version mismatch"
git tag -d v0.1.0  # Delete old tag
git tag v0.1.0     # Create new tag
git push origin v0.1.0 --force
```

### "already uploaded" errors

**Problem**: Version already exists on crates.io.

**Solution**: You cannot overwrite published versions. Must bump version:
```bash
# Bump to next version
vim rust/*/Cargo.toml  # Change 0.1.0 → 0.1.1
git commit -am "chore: bump version to 0.1.1"
git tag v0.1.1
git push origin v0.1.1
```

### GitHub Actions failure

**Problem**: Workflow fails during publish.

**Solution**:
1. Check workflow logs in GitHub Actions tab
2. Verify `CARGO_TOKEN` secret is set correctly
3. Run local dry-run to catch issues early:
   ```bash
   ./scripts/test-crates-io-install.sh
   ```

## Pre-Publication Checklist

Before creating a release tag:

- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Version updated in all 5 Cargo.toml files
- [ ] Versions verified: `./scripts/verify-crate-versions.sh`
- [ ] Dry-run succeeds: `./scripts/test-crates-io-install.sh`
- [ ] CHANGELOG.md updated
- [ ] Documentation is current
- [ ] `CARGO_TOKEN` secret is set in GitHub

## Post-Publication Verification

After GitHub Actions completes:

1. **Check crates.io**:
   ```bash
   # Search for your crates
   open https://crates.io/search?q=knhk
   ```

2. **Test installation**:
   ```bash
   cargo install knhk
   knhk --version
   knhk --help
   ```

3. **Verify GitHub release**:
   - Check https://github.com/YOUR_ORG/knhk/releases
   - Ensure release notes are correct

## Version Strategy

Follow semantic versioning:

- **0.1.x**: Initial development, breaking changes allowed
- **0.2.0+**: API stabilizing, minimize breaking changes
- **1.0.0**: Stable API, breaking changes only in major versions

### Version Bumping

```bash
# Patch (0.1.0 → 0.1.1): Bug fixes only
# Minor (0.1.0 → 0.2.0): New features, backwards compatible
# Major (0.1.0 → 1.0.0): Breaking API changes

# Use cargo-edit for easy bumping (optional)
cargo install cargo-edit
cd rust
cargo set-version --workspace 0.2.0
```

## Automation Features

The CI/CD pipeline includes:

1. **Pre-publish validation** (on every PR/push):
   - Metadata checks
   - Dry-run publish
   - Security audit
   - Build and test

2. **Automated publishing** (on git tag):
   - Version verification
   - Sequential publishing with delays
   - Installation verification
   - GitHub release creation

3. **Safety features**:
   - Continue-on-error for already-published crates
   - Index update delays
   - Post-publish verification

## Emergency Procedures

### Yank a bad release

```bash
# Yank specific version (doesn't delete, just hides)
cargo yank --version 0.1.0 knhk-cli

# Undo yank if needed
cargo yank --undo --version 0.1.0 knhk-cli
```

### Publish hotfix

```bash
# Create hotfix branch
git checkout -b hotfix/0.1.1 v0.1.0

# Make fixes
vim rust/knhk-cli/src/main.rs

# Bump version
./scripts/update-versions.sh 0.1.1

# Test
./scripts/test-crates-io-install.sh

# Commit and tag
git commit -am "fix: critical bug in CLI"
git tag v0.1.1
git push origin v0.1.1

# Merge back to main
git checkout main
git merge hotfix/0.1.1
git push origin main
```
