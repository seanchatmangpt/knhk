# Quick Start: Publishing to crates.io

## One-Time Setup (5 minutes)

### 1. Get crates.io Token
```bash
# Visit https://crates.io/settings/tokens
# Create token: "KNHK Publishing Token"
# Scope: publish-update
# Copy the token (you won't see it again!)
```

### 2. Add Token to GitHub
```bash
# Go to: https://github.com/YOUR_ORG/knhk/settings/secrets/actions
# Click "New repository secret"
# Name: CARGO_TOKEN
# Value: <paste token from step 1>
# Click "Add secret"
```

### 3. Verify Setup
```bash
cd /Users/sac/knhk/rust
./scripts/test-crates-io-install.sh
```

✅ If you see "All crates.io publication checks passed!", you're ready!

## Publishing a New Version (3 steps)

### Step 1: Update Version
```bash
cd /Users/sac/knhk/rust

# Edit all Cargo.toml files and change version to 1.0.1 (or whatever version you want)
vim knhk-otel/Cargo.toml      # Change version = "1.0.0" to "1.0.1"
vim knhk-lockchain/Cargo.toml # Change version = "1.0.0" to "1.0.1"
vim knhk-hot/Cargo.toml       # Change version = "1.0.0" to "1.0.1"
vim knhk-etl/Cargo.toml       # Change version = "1.0.0" to "1.0.1"
vim knhk-warm/Cargo.toml      # Change version = "1.0.0" to "1.0.1"
vim knhk-config/Cargo.toml    # Change version = "1.0.0" to "1.0.1"
vim knhk-connectors/Cargo.toml # Change version = "1.0.0" to "1.0.1"
vim knhk-cli/Cargo.toml       # Change version = "1.0.0" to "1.0.1"
```

### Step 2: Validate & Test
```bash
# Verify all versions match
./scripts/verify-crate-versions.sh

# Test publication dry-run
./scripts/test-crates-io-install.sh
```

### Step 3: Publish
```bash
# Commit version changes
git add .
git commit -m "chore: bump version to 1.0.1"
git push origin main

# Create and push tag (this triggers automatic publication!)
git tag v1.0.1
git push origin v1.0.1
```

## What Happens Automatically

GitHub Actions will:
1. ✅ Verify version matches tag
2. ✅ Publish all 8 crates in order
3. ✅ Wait for crates.io index updates
4. ✅ Verify `cargo install knhk` works
5. ✅ Create GitHub release with notes

**Total time: ~5 minutes**

## Monitor Progress

1. **GitHub Actions**: https://github.com/YOUR_ORG/knhk/actions
2. **Crates.io**: https://crates.io/search?q=knhk
3. **Releases**: https://github.com/YOUR_ORG/knhk/releases

## Verify Installation

After GitHub Actions completes:

```bash
# Install from crates.io
cargo install knhk

# Verify it works
knhk --version
# Should show: knhk 1.0.1
```

## If Something Goes Wrong

### Tag/Version Mismatch
```bash
# Error: "Version mismatch: tag=1.0.1, Cargo.toml=1.0.0"

# Fix: Update Cargo.toml versions
vim rust/knhk-*/Cargo.toml  # Change to 1.0.1

# Delete old tag
git tag -d v1.0.1
git push origin :refs/tags/v1.0.1

# Commit and retag
git commit -am "fix: update version to 1.0.1"
git tag v1.0.1
git push origin main v1.0.1
```

### Dry-Run Fails
```bash
# Check the error log
cat /tmp/publish-<crate-name>.log

# Common issues:
# - Missing license/description/repository in Cargo.toml
# - Broken build (run `cargo build`)
# - Clippy warnings (run `cargo clippy`)
```

### Already Published
```bash
# Error: "crate version 1.0.1 already uploaded"

# Can't overwrite published versions!
# Must bump to 1.0.2:
vim rust/knhk-*/Cargo.toml  # Change to 1.0.2
./scripts/verify-crate-versions.sh
git commit -am "chore: bump version to 1.0.2"
git tag v1.0.2
git push origin main v1.0.2
```

## Pre-Publication Checklist

Before pushing a tag, ensure:

- [ ] All tests pass: `cd rust && cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Version updated in all 8 Cargo.toml files
- [ ] Versions verified: `./scripts/verify-crate-versions.sh`
- [ ] Dry-run succeeds: `./scripts/test-crates-io-install.sh`
- [ ] CHANGELOG updated (if you have one)
- [ ] Commit pushed to main
- [ ] `CARGO_TOKEN` secret is set in GitHub

## Emergency: Yank a Release

If you published a broken version:

```bash
# Yank the bad version (hides it from default searches)
cargo yank --version 1.0.1 knhk-cli
cargo yank --version 1.0.1 knhk-hot
# ... repeat for all crates

# Publish fixed version
# (bump to 1.0.2 and follow normal process)
```

**Note**: Yanking doesn't delete the version, just marks it as not recommended.

## Version Strategy

- **Patch (1.0.0 → 1.0.1)**: Bug fixes only
- **Minor (1.0.0 → 1.1.0)**: New features, backwards compatible
- **Major (1.0.0 → 2.0.0)**: Breaking changes

## Support

**Local testing issues**: Run `./scripts/test-crates-io-install.sh` and check error output

**GitHub Actions issues**: Check workflow logs at https://github.com/YOUR_ORG/knhk/actions

**crates.io issues**: Visit https://crates.io/help

## Files Reference

- **Workflows**: `/Users/sac/knhk/rust/.github/workflows/`
  - `publish-crates.yml` - Auto-publish on tag
  - `validate-publish.yml` - Validate on PR/push

- **Scripts**: `/Users/sac/knhk/rust/scripts/`
  - `test-crates-io-install.sh` - Full validation
  - `verify-crate-versions.sh` - Version consistency check

- **Docs**: `/Users/sac/knhk/rust/docs/`
  - `CRATES_IO_SETUP.md` - Detailed setup guide
  - `CI_CD_SUMMARY.md` - Complete automation overview
  - `QUICK_START_CRATES_IO.md` - This file

---

**Next Steps**: Follow the 3-step publishing process above! 🚀
