#!/bin/bash
# Automated Self-Validation Script
# "Eating our own dog food" - Uses KNHKS to validate itself continuously
#
# This script runs continuous self-validation using KNHKS to validate KNHKS operations.
# It generates receipts, validates with Weaver, and produces reports.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/evidence/self_validation}"
INTERVAL="${INTERVAL:-300}"  # Default: 5 minutes
WEAVER_ENABLED="${WEAVER_ENABLED:-true}"
RECEIPTS_ENABLED="${RECEIPTS_ENABLED:-true}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

# Check if CLI binary exists
check_cli_binary() {
    local cli_paths=(
        "$PROJECT_ROOT/rust/target/release/knhk"
        "$PROJECT_ROOT/rust/target/debug/knhk"
        "target/release/knhk"
        "target/debug/knhk"
    )

    for path in "${cli_paths[@]}"; do
        if [ -f "$path" ] && [ -x "$path" ]; then
            echo "$path"
            return 0
        fi
    done

    return 1
}

# Build CLI if needed
build_cli() {
    log "Building KNHK CLI..."
    cd "$PROJECT_ROOT/rust"
    if ! cargo build --package knhk-cli --release 2>&1 | tail -20; then
        error "Failed to build CLI"
        return 1
    fi
    cd "$PROJECT_ROOT"
}

# Setup output directory
setup_output_dir() {
    mkdir -p "$OUTPUT_DIR"
    log "Output directory: $OUTPUT_DIR"
}

# Run one-time validation
run_validation() {
    local cli_bin="$1"
    local output_file="$2"
    local iteration="$3"

    log "Running validation iteration $iteration..."

    local weaver_flag=""
    if [ "$WEAVER_ENABLED" = "true" ]; then
        weaver_flag="--weaver"
    fi

    local receipts_flag=""
    if [ "$RECEIPTS_ENABLED" = "true" ]; then
        receipts_flag="--receipts"
    fi

    if "$cli_bin" validate self-validate $weaver_flag $receipts_flag --output "$output_file" 2>&1; then
        log "Validation iteration $iteration completed successfully"
        return 0
    else
        error "Validation iteration $iteration failed"
        return 1
    fi
}

# Run daemon mode (continuous validation)
run_daemon() {
    local cli_bin="$1"
    local interval="$2"

    log "Starting self-validation daemon (interval: ${interval}s)"
    log "Press Ctrl+C to stop"

    local iteration=0
    while true; do
        iteration=$((iteration + 1))
        local timestamp=$(date +%Y%m%d_%H%M%S)
        local output_file="$OUTPUT_DIR/validation_${timestamp}_${iteration}.json"

        if run_validation "$cli_bin" "$output_file" "$iteration"; then
            log "Sleeping for ${interval}s until next validation..."
        else
            warn "Validation failed, continuing anyway..."
        fi

        sleep "$interval"
    done
}

# Main function
main() {
    log "KNHK Automated Self-Validation"
    log "=============================="
    log "Output directory: $OUTPUT_DIR"
    log "Interval: ${INTERVAL}s"
    log "Weaver enabled: $WEAVER_ENABLED"
    log "Receipts enabled: $RECEIPTS_ENABLED"
    log ""

    # Setup output directory
    setup_output_dir

    # Find or build CLI binary
    local cli_bin
    if cli_bin=$(check_cli_binary); then
        log "Found CLI binary: $cli_bin"
    else
        warn "CLI binary not found, building..."
        build_cli
        if ! cli_bin=$(check_cli_binary); then
            error "Failed to find or build CLI binary"
            exit 1
        fi
        log "Built CLI binary: $cli_bin"
    fi

    # Check if running in daemon mode or one-time
    if [ "${1:-}" = "--daemon" ] || [ "${1:-}" = "-d" ]; then
        run_daemon "$cli_bin" "$INTERVAL"
    else
        # One-time validation
        local timestamp=$(date +%Y%m%d_%H%M%S)
        local output_file="$OUTPUT_DIR/validation_${timestamp}.json"
        if run_validation "$cli_bin" "$output_file" 1; then
            log "Validation completed successfully"
            log "Report saved to: $output_file"
            exit 0
        else
            error "Validation failed"
            exit 1
        fi
    fi
}

# Run main function
main "$@"





