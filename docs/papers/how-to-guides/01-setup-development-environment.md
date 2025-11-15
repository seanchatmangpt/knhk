# How-to Guide: Set Up Your Development Environment

**Goal**: Complete development environment setup for KNHK
**Time**: 10-15 minutes
**Difficulty**: Beginner

## Prerequisites

- Linux, macOS, or WSL2 on Windows
- Administrator/sudo access (for package installation)
- ~2GB free disk space
- Internet connection

## Step 1: Install Rust

KNHK is written in Rust. Install the latest stable toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Select option 1 (default installation).

Verify installation:
```bash
rustc --version
cargo --version
```

## Step 2: Install Build Dependencies

### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  git
```

### macOS
```bash
brew install pkg-config openssl git
```

### Verify
```bash
gcc --version
git --version
```

## Step 3: Clone the Repository

```bash
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk
```

Verify you're on the main branch:
```bash
git status
```

Expected output:
```
On branch main
Your branch is up to date with 'origin/main'.
nothing to commit, working tree clean
```

## Step 4: Install Required Tools

### Cargo build tools (if needed)
```bash
rustup update
cargo update
```

### Weaver (for schema validation)
```bash
cargo install opentelemetry-weaver
```

Verify:
```bash
weaver --version
```

### Optional: LaTeX (for building papers)
```bash
# Ubuntu/Debian
sudo apt-get install -y texlive-full

# macOS
brew install --cask mactex
```

## Step 5: Configure Git (Important!)

Set up git for commits:

```bash
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

Verify:
```bash
git config --list | grep user
```

## Step 6: Build the Project

Build in release mode (optimized):
```bash
cargo build --release
```

This may take 3-5 minutes on first run.

Verify success:
```bash
echo "Build successful!"
```

## Step 7: Verify Installation

Run the full test suite:

```bash
make test-chicago-v04
```

All tests should pass. Expected output ends with:
```
test result: ok. XX passed; 0 failed
```

## Step 8: Check Your Environment

Run the diagnostic command:
```bash
cargo --version && rustc --version && weaver --version
```

Expected:
```
cargo 1.X.X
rustc 1.X.X
Weaver X.X.X
```

## Step 9: Set Up IDE (Optional but Recommended)

### VS Code Setup

1. Install VS Code from https://code.visualstudio.com/
2. Install the "Rust Analyzer" extension (ID: rust-lang.rust-analyzer)
3. Open KNHK folder in VS Code
4. Create `.vscode/settings.json`:

```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

### Other IDEs

- **IntelliJ IDEA**: Install Rust plugin
- **Vim/Neovim**: Install rust.vim + rust-analyzer
- **Sublime Text**: Install LSP and rust-analyzer

## Step 10: Verify Everything Works

Run this command:
```bash
cargo test --lib --release && \
make test-performance-v04 && \
weaver registry check -r registry/ && \
echo "✅ All systems operational!"
```

If this succeeds, your environment is ready!

## Troubleshooting

### Cargo fails: "linker not found"
```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential

# macOS
xcode-select --install
```

### Weaver installation fails
```bash
cargo install --force opentelemetry-weaver
```

### Permission denied on scripts
```bash
chmod +x scripts/*.sh
```

### Tests timeout
- Close other applications
- Run with: `cargo test --lib --release -- --test-threads=1`

### Git authentication fails
```bash
# Use SSH instead
git remote set-url origin git@github.com:seanchatmangpt/knhk.git
```

### Out of memory during build
```bash
# Use fewer parallel jobs
cargo build --release -j 2
```

## Environment Variables (Optional)

For more detailed logging during development:

```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

Add to your shell's rc file (`.bashrc`, `.zshrc`, etc.) to make permanent:
```bash
echo 'export RUST_LOG=debug' >> ~/.bashrc
source ~/.bashrc
```

## Next Steps

After successful setup:

1. **Learn the basics**: [Getting Started with KNHK](../tutorials/01-getting-started-with-knhk.md)
2. **Understand telemetry**: [Understanding Telemetry](../tutorials/02-understanding-telemetry.md)
3. **Run tests**: See [How to Run Tests Efficiently](02-run-tests-efficiently.md)
4. **Start developing**: See [How to Add New Features](04-add-new-features.md)

## Verification Checklist

- [ ] Rust installed and updated
- [ ] Build dependencies installed
- [ ] Repository cloned
- [ ] Project builds successfully (`cargo build --release`)
- [ ] Tests pass (`make test-chicago-v04`)
- [ ] Weaver validation works (`weaver registry check -r registry/`)
- [ ] IDE configured (optional)
- [ ] Git configured with user info

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Beginner
**Related**: Running Tests, Adding Features
