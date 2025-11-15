# How-to Guide 1: Set Up Your Development Environment

## Goal

Quickly set up a complete KNHK development environment with all necessary tools, dependencies, and configurations to develop, test, and validate features.

**Time Estimate**: 15-30 minutes
**Prerequisites**: Linux/macOS, basic command-line familiarity
**Outcomes**: Ready to develop, test, and validate KNHK features

---

## Prerequisites

Before starting, ensure you have:
- [ ] Git installed (`git --version`)
- [ ] About 5GB free disk space
- [ ] Command-line terminal access
- [ ] Administrator/sudo access (for system package installation)

---

## Step 1: Install Rust and Cargo

### On Linux (Debian/Ubuntu)

```bash
# Install from official installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose "1) Proceed with standard installation" (default)

# Load Rust environment
source $HOME/.cargo/env

# Verify installation
rustc --version   # Should be 1.70 or later
cargo --version   # Should be 1.70 or later
```

### On macOS

```bash
# Using curl (preferred)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or using Homebrew
brew install rustup-init
rustup-init

# Verify
rustc --version
cargo --version
```

### On Windows

```powershell
# Download and run installer from https://rustup.rs/
# Or using Chocolatey:
choco install rust

# Verify in PowerShell or Command Prompt
rustc --version
cargo --version
```

✅ **Checkpoint**: `rustc --version` returns version ≥1.70

---

## Step 2: Install System Dependencies

### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  git \
  make \
  clang \
  llvm
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew packages
brew install \
  pkg-config \
  openssl \
  make \
  llvm
```

### Verify Installation

```bash
# Check that make is available
make --version    # Should be 4.0 or later

# Check C compiler
gcc --version
# or
clang --version
```

✅ **Checkpoint**: All tools report versions without errors

---

## Step 3: Install Cargo Tools

```bash
# Faster test runner (recommended)
cargo install cargo-nextest

# Test coverage (optional)
cargo install cargo-tarpaulin

# Documentation builder
cargo install cargo-doc

# Clippy is usually included with Rust
cargo install clippy
```

### Verify Tools

```bash
cargo nextest --version
```

✅ **Checkpoint**: `cargo nextest` is available

---

## Step 4: Clone the Repository

```bash
# Clone KNHK repository
git clone https://github.com/seanchatmangpt/knhk.git

# Navigate to project
cd knhk

# Verify you're on the right branch
git status
```

### Important: Create Feature Branch

```bash
# Create your feature branch (required for development)
git checkout -b feature/your-feature-name

# Verify
git branch
# Should show: * feature/your-feature-name
```

✅ **Checkpoint**: Repository cloned and you're on a feature branch

---

## Step 5: Install Project Dependencies

```bash
# From knhk directory
cd knhk

# Download and compile dependencies
cargo build --workspace

# This may take 5-15 minutes on first run
```

### Expected Output

```
Compiling knhk v0.1.0 (...)
Finished dev [unoptimized + debuginfo] target(s) in XXs
```

✅ **Checkpoint**: `cargo build --workspace` completes successfully

---

## Step 6: Verify Build Artifacts

```bash
# Check that binaries were created
ls -la target/debug/

# Look for knhk executable
./target/debug/knhk --help
```

### Expected Help Output

```
KNHK - Knowledge Hooks Framework

USAGE:
    knhk [OPTIONS] <COMMAND>

COMMANDS:
    help       Print help message
    ...
```

✅ **Checkpoint**: `knhk --help` displays without errors

---

## Step 7: Run Initial Tests

```bash
# Run all workspace tests
cargo test --workspace

# Or with the faster nextest runner
cargo nextest run --workspace
```

### Expected Results

```
test result: ok. XX passed; 0 failed; 0 ignored; 0 measured
```

✅ **Checkpoint**: All tests pass

---

## Step 8: Configure Your Editor

### Visual Studio Code (Recommended)

```bash
# Install extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension mitmih.markdown-table
```

### Create `.vscode/settings.json`

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--all-targets",
    "--all-features",
    "--"
  ],
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### Vim/Neovim

```bash
# Install coc-rust-analyzer for Vim/Neovim
# https://github.com/fannheyward/coc-rust-analyzer
```

✅ **Checkpoint**: Editor configured for Rust development

---

## Step 9: Optional but Recommended Tools

### Install Additional Development Tools

```bash
# Weaver for schema validation (required for advanced validation)
# Installation depends on your system
# See https://github.com/open-telemetry/weaver

# Docker (for running tests in containers)
# macOS: brew install docker
# Linux: https://docs.docker.com/engine/install/

# LaTeX (if modifying research papers)
sudo apt-get install texlive-full  # Ubuntu/Debian
brew install --cask mactex           # macOS
```

---

## Step 10: Verify Complete Setup

Run this comprehensive check:

```bash
#!/bin/bash
# Save as check-setup.sh and run: bash check-setup.sh

echo "Checking KNHK development setup..."
echo ""

# Check Rust
echo "✓ Rust version:"
rustc --version

# Check Cargo
echo "✓ Cargo version:"
cargo --version

# Check Make
echo "✓ Make version:"
make --version | head -1

# Check Git
echo "✓ Git version:"
git --version

# Check repository
echo "✓ Repository status:"
git status | head -3

# Check build
echo "✓ Building project..."
cargo build --workspace > /dev/null 2>&1 && echo "Build successful" || echo "Build failed"

# Check tests
echo "✓ Running tests..."
cargo test --workspace > /dev/null 2>&1 && echo "Tests passed" || echo "Tests failed"

echo ""
echo "Setup verification complete!"
```

### Expected Output

```
✓ Rust version: rustc 1.70.0 (...)
✓ Cargo version: cargo 1.70.0 (...)
✓ Make version: GNU Make 4.3
✓ Git version: git version 2.40.0
✓ Repository status: On branch feature/your-feature-name
✓ Build successful
✓ Tests passed

Setup verification complete!
```

✅ **Complete**: Development environment fully configured

---

## Troubleshooting

### Issue: "Rust not found"

**Solution**:
```bash
source $HOME/.cargo/env
rustc --version
```

### Issue: "cargo build" fails

**Solution**:
```bash
cargo clean
cargo update
cargo build --workspace
```

### Issue: Permission denied when installing tools

**Solution**:
```bash
# Use sudo (careful with system packages)
sudo apt-get install build-essential

# Or install to user directory
cargo install --path /home/user/bin cargo-nextest
```

### Issue: "Could not find working C compiler"

**Solution** (Ubuntu/Debian):
```bash
sudo apt-get install build-essential gcc clang
```

**Solution** (macOS):
```bash
xcode-select --install
```

---

## Environment Variables

### Optional Configuration

```bash
# Add to ~/.bashrc or ~/.zshrc for persistent configuration
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# For faster compilation (optional)
export CARGO_BUILD_JOBS=4  # Adjust based on CPU cores
```

### Verify Environment

```bash
echo $CARGO_HOME
echo $PATH | grep cargo
```

---

## Quick Reference

| Tool | Command | Purpose |
|------|---------|---------|
| Build | `cargo build --workspace` | Compile all code |
| Test | `cargo test --workspace` | Run all tests |
| Check | `cargo check --all` | Fast syntax check |
| Clippy | `cargo clippy --all` | Lint and warnings |
| Format | `cargo fmt --all` | Code formatting |
| Clean | `cargo clean` | Remove build artifacts |
| Chicago Tests | `make test-chicago-v04` | Integration testing |
| Performance | `make test-performance-v04` | Performance validation |

---

## Next Steps

After setting up your environment:

1. **Run the Getting Started Tutorial**
   - [Tutorial 1: Your First KNHK Workflow](../tutorials/01-getting-started.md)

2. **Build Your First Feature**
   - [How-to: Add New Features](02-add-new-features.md) (coming soon)

3. **Fix Issues**
   - [How-to: Debug Failing Tests](03-debug-failing-tests.md) (coming soon)

4. **Understand the Architecture**
   - [Explanation: KNHK Architecture](../explanation/formal-foundations.md)

---

## Support

If you encounter issues:

1. **Check the FAQ** - See Troubleshooting section above
2. **Review CLAUDE.md** - `/CLAUDE.md` has development guidelines
3. **Search Issues** - https://github.com/seanchatmangpt/knhk/issues
4. **Ask for Help** - Open a new issue with your error message

---

**Created**: 2025-11-15
**Updated**: 2025-11-15
**Status**: Complete
**Difficulty**: Beginner
**Next**: [How-to: Run Tests Efficiently](02-run-tests-efficiently.md) (coming soon)
