#!/usr/bin/env bash
# Test all example workflows for correctness

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_ROOT="$(dirname "$SCRIPT_DIR")"
EXAMPLES_DIR="$TEMPLATE_ROOT/examples"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_success() { echo -e "${GREEN}✓ $1${NC}"; }
log_error() { echo -e "${RED}✗ $1${NC}"; exit 1; }
log_info() { echo -e "${YELLOW}ℹ $1${NC}"; }

echo "=== Testing KNHK YAWL Examples ==="
echo ""

# Check if ggen is installed
if ! command -v ggen &> /dev/null; then
  log_info "ggen not installed - install from https://github.com/seanchatmangpt/ggen"
  log_info "Skipping execution tests"
  exit 0
fi

# Test each example
for example in "$EXAMPLES_DIR"/*.ttl; do
  filename=$(basename "$example" .ttl)
  echo "Testing: $filename"

  # Validate Turtle syntax
  if ggen graph load "$example" &> /dev/null; then
    log_success "Valid Turtle: $filename.ttl"
  else
    log_error "Invalid Turtle syntax: $filename.ttl"
  fi

  # Test SPARQL queries
  log_info "Validating SPARQL queries..."

  # Extract workflows
  if ggen graph query --ontology "$example" --sparql \
    'PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
     SELECT ?workflow WHERE { ?workflow a yawl:WorkflowSpecification }' \
    &> /dev/null; then
    log_success "Workflow extraction query valid"
  else
    log_error "Workflow extraction query failed"
  fi

  # Extract tasks
  if ggen graph query --ontology "$example" --sparql \
    'PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
     SELECT ?task WHERE { ?task a yawl:Task }' \
    &> /dev/null; then
    log_success "Task extraction query valid"
  else
    log_error "Task extraction query failed"
  fi

  echo ""
done

log_success "All example tests passed!"
