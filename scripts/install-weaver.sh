#!/bin/bash
# Weaver Binary Installation Script
# Installs OpenTelemetry Weaver for telemetry validation
#
# Usage: ./install-weaver.sh [version]
# Example: ./install-weaver.sh v0.1.0

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_VERSION="0.1.0"
WEAVER_BINARY_NAME="weaver"
INSTALL_DIR="${HOME}/.local/bin"
WEAVER_GITHUB_REPO="open-telemetry/opentelemetry-rust"

# Detect OS and architecture
detect_platform() {
    local os
    local arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        MINGW*)     os="windows" ;;
        MSYS*)      os="windows" ;;
        *)          os="unknown" ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="arm64" ;;
        armv7l)         arch="armv7" ;;
        *)              arch="unknown" ;;
    esac
    
    echo "${os}-${arch}"
}

# Check if weaver is already installed
check_weaver_installed() {
    if command -v "${WEAVER_BINARY_NAME}" &> /dev/null; then
        local version
        version=$("${WEAVER_BINARY_NAME}" --version 2>&1 || echo "unknown")
        echo -e "${GREEN}✓ Weaver is already installed: ${version}${NC}"
        return 0
    fi
    return 1
}

# Install via Cargo (preferred method for Rust projects)
install_via_cargo() {
    echo -e "${YELLOW}Attempting to install Weaver via Cargo...${NC}"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found. Skipping Cargo installation.${NC}"
        return 1
    fi
    
    # Check if weaver is available as a cargo binary
    if cargo install --list 2>/dev/null | grep -q "weaver"; then
        echo -e "${YELLOW}Weaver already installed via Cargo.${NC}"
        return 0
    fi
    
    # Try to install weaver via cargo
    # Note: This assumes weaver is published as a crate
    # If not, this will fail gracefully
    if cargo install weaver 2>/dev/null; then
        echo -e "${GREEN}✓ Weaver installed via Cargo${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}Weaver not available as Cargo crate. Trying alternative methods...${NC}"
    return 1
}

# Install via GitHub release
install_via_github() {
    local version="${1:-${DEFAULT_VERSION}}"
    local platform=$(detect_platform)
    local os="${platform%%-*}"
    local arch="${platform##*-}"
    
    echo -e "${YELLOW}Attempting to install Weaver from GitHub releases...${NC}"
    
    # Map architecture names
    case "${arch}" in
        x86_64)  arch="x86_64" ;;
        arm64)   arch="arm64" ;;
        armv7)   arch="armv7" ;;
        *)       echo -e "${RED}✗ Unsupported architecture: ${arch}${NC}"; return 1 ;;
    esac
    
    # Map OS names
    case "${os}" in
        linux)   os="linux" ;;
        darwin)  os="darwin" ;;
        windows) os="windows" ;;
        *)       echo -e "${RED}✗ Unsupported OS: ${os}${NC}"; return 1 ;;
    esac
    
    # Construct download URL
    # Note: Adjust this URL based on actual Weaver release location
    local binary_name="${WEAVER_BINARY_NAME}"
    if [ "${os}" = "windows" ]; then
        binary_name="${binary_name}.exe"
    fi
    
    local download_url="https://github.com/${WEAVER_GITHUB_REPO}/releases/download/v${version}/weaver-${os}-${arch}"
    if [ "${os}" = "windows" ]; then
        download_url="${download_url}.exe"
    fi
    
    echo -e "${YELLOW}Downloading Weaver from: ${download_url}${NC}"
    
    # Create temp directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf ${temp_dir}" EXIT
    
    # Download binary
    if command -v curl &> /dev/null; then
        if ! curl -Lf "${download_url}" -o "${temp_dir}/${binary_name}"; then
            echo -e "${RED}✗ Failed to download Weaver from GitHub${NC}"
            return 1
        fi
    elif command -v wget &> /dev/null; then
        if ! wget -q "${download_url}" -O "${temp_dir}/${binary_name}"; then
            echo -e "${RED}✗ Failed to download Weaver from GitHub${NC}"
            return 1
        fi
    else
        echo -e "${RED}✗ Neither curl nor wget found. Cannot download binary.${NC}"
        return 1
    fi
    
    # Make binary executable (Unix-like systems)
    if [ "${os}" != "windows" ]; then
        chmod +x "${temp_dir}/${binary_name}"
    fi
    
    # Create install directory
    mkdir -p "${INSTALL_DIR}"
    
    # Move binary to install directory
    mv "${temp_dir}/${binary_name}" "${INSTALL_DIR}/${WEAVER_BINARY_NAME}"
    
    echo -e "${GREEN}✓ Weaver downloaded and installed to ${INSTALL_DIR}/${WEAVER_BINARY_NAME}${NC}"
    return 0
}

# Install via Homebrew (macOS)
install_via_homebrew() {
    if [ "$(uname -s)" != "Darwin" ]; then
        return 1
    fi
    
    if ! command -v brew &> /dev/null; then
        return 1
    fi
    
    echo -e "${YELLOW}Attempting to install Weaver via Homebrew...${NC}"
    
    # Check if weaver is available as a Homebrew formula
    if brew search weaver 2>/dev/null | grep -q "weaver"; then
        if brew install weaver 2>/dev/null; then
            echo -e "${GREEN}✓ Weaver installed via Homebrew${NC}"
            return 0
        fi
    fi
    
    return 1
}

# Ensure install directory is in PATH
ensure_path() {
    local profile_file
    
    # Detect shell profile
    case "${SHELL}" in
        */zsh)  profile_file="${HOME}/.zshrc" ;;
        */bash) profile_file="${HOME}/.bashrc" ;;
        *)      profile_file="${HOME}/.profile" ;;
    esac
    
    # Check if PATH already includes install directory
    if echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        return 0
    fi
    
    # Add to PATH
    echo -e "${YELLOW}Adding ${INSTALL_DIR} to PATH...${NC}"
    echo "" >> "${profile_file}"
    echo "# Weaver binary directory" >> "${profile_file}"
    echo "export PATH=\"\${PATH}:${INSTALL_DIR}\"" >> "${profile_file}"
    
    echo -e "${GREEN}✓ Added ${INSTALL_DIR} to PATH in ${profile_file}${NC}"
    echo -e "${YELLOW}Please run: source ${profile_file}${NC}"
}

# Verify installation
verify_installation() {
    # Check if weaver is in PATH
    if ! command -v "${WEAVER_BINARY_NAME}" &> /dev/null; then
        # Try to add to PATH for current session
        export PATH="${PATH}:${INSTALL_DIR}"
        
        if ! command -v "${WEAVER_BINARY_NAME}" &> /dev/null; then
            echo -e "${RED}✗ Weaver not found in PATH${NC}"
            echo -e "${YELLOW}Please add ${INSTALL_DIR} to your PATH${NC}"
            return 1
        fi
    fi
    
    # Verify weaver works
    if "${WEAVER_BINARY_NAME}" --version &> /dev/null || "${WEAVER_BINARY_NAME}" --help &> /dev/null; then
        echo -e "${GREEN}✓ Weaver installation verified${NC}"
        return 0
    fi
    
    echo -e "${RED}✗ Weaver binary found but not working correctly${NC}"
    return 1
}

# Build from source (fallback)
build_from_source() {
    echo -e "${YELLOW}Attempting to build Weaver from source...${NC}"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Cargo not found. Cannot build from source.${NC}"
        return 1
    fi
    
    if ! command -v git &> /dev/null; then
        echo -e "${RED}✗ Git not found. Cannot clone repository.${NC}"
        return 1
    fi
    
    # Try to find weaver repository
    # This is a placeholder - adjust based on actual repository location
    local repo_url="https://github.com/open-telemetry/opentelemetry-rust"
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf ${temp_dir}" EXIT
    
    echo -e "${YELLOW}Cloning repository...${NC}"
    if ! git clone "${repo_url}" "${temp_dir}/weaver" 2>/dev/null; then
        echo -e "${RED}✗ Failed to clone repository${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}Building Weaver...${NC}"
    cd "${temp_dir}/weaver"
    
    # Look for weaver binary crate
    if [ -d "vendors/weaver" ] || [ -d "crates/weaver-cli" ]; then
        local weaver_path
        weaver_path=$(find . -name "Cargo.toml" -path "*/weaver*" | head -1)
        if [ -n "${weaver_path}" ]; then
            local crate_dir=$(dirname "${weaver_path}")
            cd "${crate_dir}"
            if cargo build --release 2>/dev/null; then
                local binary_path=$(find target/release -name "weaver" -o -name "weaver.exe" | head -1)
                if [ -n "${binary_path}" ]; then
                    mkdir -p "${INSTALL_DIR}"
                    cp "${binary_path}" "${INSTALL_DIR}/${WEAVER_BINARY_NAME}"
                    chmod +x "${INSTALL_DIR}/${WEAVER_BINARY_NAME}"
                    echo -e "${GREEN}✓ Weaver built and installed${NC}"
                    return 0
                fi
            fi
        fi
    fi
    
    echo -e "${RED}✗ Could not find Weaver crate in repository${NC}"
    return 1
}

# Main installation function
main() {
    local version="${1:-${DEFAULT_VERSION}}"
    
    echo -e "${GREEN}=== Weaver Binary Installation ===${NC}"
    echo ""
    
    # Check if already installed
    if check_weaver_installed; then
        echo -e "${GREEN}Weaver is already installed. Exiting.${NC}"
        exit 0
    fi
    
    # Detect platform
    local platform=$(detect_platform)
    echo -e "${YELLOW}Detected platform: ${platform}${NC}"
    echo ""
    
    # Try installation methods in order of preference
    local installed=0
    
    # 1. Try Cargo installation
    if install_via_cargo; then
        installed=1
    fi
    
    # 2. Try Homebrew (macOS)
    if [ "${installed}" -eq 0 ] && install_via_homebrew; then
        installed=1
    fi
    
    # 3. Try GitHub release download
    if [ "${installed}" -eq 0 ] && install_via_github "${version}"; then
        installed=1
    fi
    
    # 4. Try building from source
    if [ "${installed}" -eq 0 ] && build_from_source; then
        installed=1
    fi
    
    if [ "${installed}" -eq 0 ]; then
        echo -e "${RED}✗ Failed to install Weaver using any available method${NC}"
        echo ""
        echo -e "${YELLOW}Manual installation options:${NC}"
        echo "1. Install via Cargo: cargo install weaver"
        echo "2. Download from GitHub releases: https://github.com/${WEAVER_GITHUB_REPO}/releases"
        echo "3. Build from source: git clone ${WEAVER_GITHUB_REPO} && cd weaver && cargo build --release"
        exit 1
    fi
    
    # Ensure PATH is configured
    ensure_path
    
    # Verify installation
    echo ""
    verify_installation
    
    echo ""
    echo -e "${GREEN}=== Installation Complete ===${NC}"
    echo -e "${YELLOW}Weaver binary location: ${INSTALL_DIR}/${WEAVER_BINARY_NAME}${NC}"
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        echo -e "${YELLOW}Please run: export PATH=\"\${PATH}:${INSTALL_DIR}\"${NC}"
    fi
}

# Run main function
main "$@"


