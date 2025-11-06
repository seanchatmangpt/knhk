# Weaver Binary Installation

## Overview

The `install-weaver.sh` script provides automated installation of the OpenTelemetry Weaver binary for telemetry validation. Weaver is used for live-check validation of OpenTelemetry semantic conventions.

## Usage

```bash
# Install latest version
./scripts/install-weaver.sh

# Install specific version
./scripts/install-weaver.sh v0.1.0
```

## Installation Methods

The script attempts installation in the following order:

1. **Cargo Installation** (preferred)
   - Installs via `cargo install weaver`
   - Requires Rust toolchain

2. **Homebrew Installation** (macOS only)
   - Installs via `brew install weaver`
   - Requires Homebrew

3. **GitHub Release Download**
   - Downloads pre-built binary from GitHub releases
   - Supports Linux, macOS, and Windows

4. **Build from Source** (fallback)
   - Clones repository and builds using Cargo
   - Requires Git and Rust toolchain

## Platform Support

- **Linux**: x86_64, arm64, armv7
- **macOS**: x86_64 (Intel), arm64 (Apple Silicon)
- **Windows**: amd64, 386

## Installation Location

- **Unix-like systems**: `~/.local/bin/weaver`
- **Windows**: `%USERPROFILE%\.local\bin\weaver.exe`

## PATH Configuration

The script automatically adds the installation directory to your shell profile:

- **zsh**: `~/.zshrc`
- **bash**: `~/.bashrc`
- **Other**: `~/.profile`

After installation, run:
```bash
source ~/.zshrc  # or ~/.bashrc
```

## Verification

After installation, verify Weaver is installed:

```bash
weaver --version
# or
weaver --help
```

## Manual Installation

If automated installation fails, you can install manually:

### Via Cargo
```bash
cargo install weaver
```

### Via Homebrew (macOS)
```bash
brew install weaver
```

### From GitHub Releases
1. Visit: https://github.com/open-telemetry/opentelemetry-rust/releases
2. Download appropriate binary for your platform
3. Make executable: `chmod +x weaver`
4. Move to PATH: `sudo mv weaver /usr/local/bin/`

### Build from Source
```bash
git clone https://github.com/open-telemetry/opentelemetry-rust.git
cd opentelemetry-rust/vendors/weaver  # or appropriate path
cargo build --release
cp target/release/weaver ~/.local/bin/
```

## Troubleshooting

### Weaver not found after installation

1. Check if binary exists:
   ```bash
   ls -la ~/.local/bin/weaver
   ```

2. Add to PATH manually:
   ```bash
   export PATH="${PATH}:${HOME}/.local/bin"
   ```

3. Verify in current shell:
   ```bash
   which weaver
   ```

### Permission denied

If you get permission errors:

```bash
chmod +x ~/.local/bin/weaver
```

### macOS Security Warning

If macOS blocks the binary:

```bash
xattr -d com.apple.quarantine ~/.local/bin/weaver
```

## Integration with KNHK

Weaver is used by KNHK for telemetry validation:

```bash
# Start Weaver live-check
knhk metrics weaver-start --otlp-port 4317 --admin-port 8080

# Validate telemetry
knhk metrics weaver-validate --timeout 10

# Stop Weaver
knhk metrics weaver-stop --admin-port 8080
```

## Requirements

- **curl** or **wget** (for GitHub downloads)
- **Git** (for source builds)
- **Rust/Cargo** (for Cargo installation or source builds)
- **Homebrew** (optional, for macOS Homebrew installation)

## Notes

- The script checks for existing Weaver installations before proceeding
- Installation directory is created if it doesn't exist
- PATH is automatically configured for the user's shell
- Script exits gracefully if Weaver is already installed
