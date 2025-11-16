#!/usr/bin/env bash
# Validate ggen marketplace template completeness and correctness

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_ROOT="$(dirname "$SCRIPT_DIR")"

ERRORS=0
WARNINGS=0

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_error() {
  echo -e "${RED}✗ ERROR: $1${NC}" >&2
  ((ERRORS++))
}

log_warning() {
  echo -e "${YELLOW}⚠ WARNING: $1${NC}" >&2
  ((WARNINGS++))
}

log_success() {
  echo -e "${GREEN}✓ $1${NC}"
}

log_info() {
  echo -e "${NC}ℹ $1${NC}"
}

echo "=== KNHK YAWL Template Validation ==="
echo "Template Root: $TEMPLATE_ROOT"
echo ""

# 1. Check required files
echo "Checking required files..."
required_files=(
  "ggen.yaml"
  "README.md"
  "template/yawl-workflow.ttl.j2"
  "template/yawl-workflow.json.j2"
  "queries/extract_workflows.sparql"
  "queries/extract_tasks.sparql"
  "queries/extract_conditions.sparql"
  "queries/extract_flows.sparql"
  "queries/extract_patterns.sparql"
  "queries/extract_metadata.sparql"
  "docs/USAGE.md"
  "docs/ARCHITECTURE.md"
  "docs/EXAMPLES.md"
  "examples/simple-sequence.ttl"
  "examples/parallel-split.ttl"
  "examples/exclusive-choice.ttl"
)

for file in "${required_files[@]}"; do
  if [ -f "$TEMPLATE_ROOT/$file" ]; then
    log_success "Found: $file"
  else
    log_error "Missing: $file"
  fi
done

echo ""

# 2. Validate YAML metadata
echo "Validating ggen.yaml..."
if command -v yamllint &> /dev/null; then
  if yamllint "$TEMPLATE_ROOT/ggen.yaml" &> /dev/null; then
    log_success "ggen.yaml syntax valid"
  else
    log_warning "ggen.yaml has YAML syntax issues"
  fi
else
  log_info "yamllint not installed (optional)"
fi

# Check required YAML fields
if grep -q "^id:" "$TEMPLATE_ROOT/ggen.yaml"; then
  log_success "ggen.yaml contains 'id' field"
else
  log_error "ggen.yaml missing 'id' field"
fi

if grep -q "^version:" "$TEMPLATE_ROOT/ggen.yaml"; then
  log_success "ggen.yaml contains 'version' field"
else
  log_error "ggen.yaml missing 'version' field"
fi

echo ""

# 3. Validate SPARQL queries
echo "Validating SPARQL queries..."
sparql_files=("$TEMPLATE_ROOT"/queries/*.sparql)

for sparql_file in "${sparql_files[@]}"; do
  if [ -f "$sparql_file" ]; then
    filename=$(basename "$sparql_file")

    # Check for SELECT keyword (basic validation)
    if grep -q "^SELECT" "$sparql_file"; then
      log_success "Valid SPARQL query: $filename"
    else
      log_warning "SPARQL query may be incomplete: $filename"
    fi

    # Check for PREFIX declarations
    if grep -q "^PREFIX" "$sparql_file"; then
      log_success "SPARQL query has PREFIX declarations: $filename"
    else
      log_warning "SPARQL query missing PREFIX declarations: $filename"
    fi
  fi
done

echo ""

# 4. Validate Jinja2 templates
echo "Validating Jinja2 templates..."
for template_file in "$TEMPLATE_ROOT"/template/*.j2; do
  if [ -f "$template_file" ]; then
    filename=$(basename "$template_file")

    # Check for template directives
    if grep -q "{%" "$template_file"; then
      log_success "Template contains Jinja2 directives: $filename"
    else
      log_warning "Template may be missing Jinja2 directives: $filename"
    fi

    # Check for variable interpolation
    if grep -q "{{" "$template_file"; then
      log_success "Template contains variable interpolation: $filename"
    else
      log_warning "Template may be missing variable interpolation: $filename"
    fi

    # Check for common filters
    if grep -q "|" "$template_file"; then
      log_success "Template uses Jinja2 filters: $filename"
    else
      log_info "Template does not use filters: $filename"
    fi
  fi
done

echo ""

# 5. Validate Turtle example files
echo "Validating Turtle example files..."
for ttl_file in "$TEMPLATE_ROOT"/examples/*.ttl; do
  if [ -f "$ttl_file" ]; then
    filename=$(basename "$ttl_file")

    # Check for @prefix declarations
    if grep -q "^@prefix" "$ttl_file"; then
      log_success "Turtle file has @prefix declarations: $filename"
    else
      log_error "Turtle file missing @prefix declarations: $filename"
    fi

    # Check for rdf:type declarations
    if grep -q "a.*:" "$ttl_file"; then
      log_success "Turtle file has type declarations: $filename"
    else
      log_error "Turtle file may be missing type declarations: $filename"
    fi
  fi
done

echo ""

# 6. Validate documentation
echo "Validating documentation..."
for doc_file in "$TEMPLATE_ROOT"/docs/*.md; do
  if [ -f "$doc_file" ]; then
    filename=$(basename "$doc_file")

    # Check for heading
    if grep -q "^#" "$doc_file"; then
      log_success "Documentation has headings: $filename"
    else
      log_warning "Documentation may lack proper headings: $filename"
    fi

    # Check for code blocks
    if grep -q "^\`\`\`" "$doc_file"; then
      log_success "Documentation has code examples: $filename"
    else
      log_info "Documentation may lack code examples: $filename"
    fi
  fi
done

echo ""

# 7. Summary
echo "=== Validation Summary ==="
echo "Errors: $ERRORS"
echo "Warnings: $WARNINGS"

if [ $ERRORS -eq 0 ]; then
  log_success "All validation checks passed!"
  exit 0
else
  log_error "$ERRORS validation errors found"
  exit 1
fi
