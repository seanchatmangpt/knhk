#!/usr/bin/env bash
# Generate workflows from YAWL ontology using ggen templates
#
# Usage:
#   ./scripts/ggen/generate-workflows.sh <ontology.ttl> [output_dir]
#
# This script:
# 1. Parses YAWL ontology (Turtle/RDF)
# 2. Extracts workflow structure via SPARQL queries
# 3. Generates Rust code using Handlebars templates
# 4. Generates config files (YAML)
# 5. Generates Weaver registry (OTEL schema)
# 6. Validates generated code

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Paths
TEMPLATES_DIR="$PROJECT_ROOT/templates"
ONTOLOGY_DIR="$PROJECT_ROOT/ontology"
OUTPUT_DIR="${2:-$PROJECT_ROOT/target/generated}"

# Input validation
if [ $# -lt 1 ]; then
    echo "Usage: $0 <ontology.ttl> [output_dir]"
    echo ""
    echo "Examples:"
    echo "  $0 ontology/workflows/approval.ttl"
    echo "  $0 ontology/workflows/approval.ttl target/generated"
    exit 1
fi

ONTOLOGY_FILE="$1"

if [ ! -f "$ONTOLOGY_FILE" ]; then
    echo "Error: Ontology file not found: $ONTOLOGY_FILE"
    exit 1
fi

echo "🚀 KNHK ggen Code Generation"
echo "============================"
echo "Ontology: $ONTOLOGY_FILE"
echo "Templates: $TEMPLATES_DIR"
echo "Output: $OUTPUT_DIR"
echo ""

# Create output directories
mkdir -p "$OUTPUT_DIR"/{src,config,weaver,tests,docs}

echo "📊 Step 1: Extracting workflow metadata from ontology..."

# Extract workflow name using SPARQL
WORKFLOW_NAME=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?name WHERE { ?wf a yawl:Workflow ; rdfs:label ?name } LIMIT 1' \
    --results-format json | jq -r '.results.bindings[0].name.value // "unknown"')

echo "   Workflow name: $WORKFLOW_NAME"

# Extract workflow ID
WORKFLOW_ID=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?id WHERE { ?wf a yawl:Workflow ; yawl:workflowId ?id } LIMIT 1' \
    --results-format json | jq -r '.results.bindings[0].id.value // "unknown"')

echo "   Workflow ID: $WORKFLOW_ID"

echo ""
echo "📝 Step 2: Extracting tasks..."

# Extract tasks
TASKS_JSON=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?id ?name ?type ?pattern WHERE {
        ?task a yawl:Task ;
              yawl:taskId ?id ;
              rdfs:label ?name ;
              yawl:taskType ?type ;
              yawl:pattern ?pattern .
    }' \
    --results-format json)

TASK_COUNT=$(echo "$TASKS_JSON" | jq '.results.bindings | length')
echo "   Found $TASK_COUNT tasks"

echo ""
echo "🔄 Step 3: Extracting state machine..."

# Extract states
STATES_JSON=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?name ?type ?isFinal WHERE {
        ?state a yawl:State ;
               rdfs:label ?name ;
               yawl:stateType ?type .
        OPTIONAL { ?state yawl:isFinal ?isFinal }
    }' \
    --results-format json)

STATE_COUNT=$(echo "$STATES_JSON" | jq '.results.bindings | length')
echo "   Found $STATE_COUNT states"

# Extract transitions
TRANSITIONS_JSON=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?from ?to ?event WHERE {
        ?trans a yawl:Transition ;
               yawl:fromState ?fromState ;
               yawl:toState ?toState ;
               yawl:event ?event .
        ?fromState rdfs:label ?from .
        ?toState rdfs:label ?to .
    }' \
    --results-format json)

TRANSITION_COUNT=$(echo "$TRANSITIONS_JSON" | jq '.results.bindings | length')
echo "   Found $TRANSITION_COUNT transitions"

echo ""
echo "🪝 Step 4: Extracting hooks..."

# Extract hooks
HOOKS_JSON=$(npx oxigraph query \
    --file "$ONTOLOGY_FILE" \
    --query 'SELECT ?name ?trigger ?type WHERE {
        ?hook a knhk:Hook ;
              rdfs:label ?name ;
              knhk:trigger ?trigger ;
              knhk:hookType ?type .
    }' \
    --results-format json)

HOOK_COUNT=$(echo "$HOOKS_JSON" | jq '.results.bindings | length')
echo "   Found $HOOK_COUNT hooks"

echo ""
echo "🔧 Step 5: Generating Rust code..."

# Create template context JSON
CONTEXT_FILE="$OUTPUT_DIR/context.json"
cat > "$CONTEXT_FILE" <<EOF
{
  "workflow_name": "$WORKFLOW_NAME",
  "workflow_id": "$WORKFLOW_ID",
  "workflow_version": "1.0.0",
  "generation_timestamp": "$(date -Iseconds)",
  "ontology_path": "$ONTOLOGY_FILE",
  "tasks": $(echo "$TASKS_JSON" | jq '.results.bindings'),
  "states": $(echo "$STATES_JSON" | jq '.results.bindings'),
  "transitions": $(echo "$TRANSITIONS_JSON" | jq '.results.bindings'),
  "hooks": $(echo "$HOOKS_JSON" | jq '.results.bindings')
}
EOF

echo "   Context file: $CONTEXT_FILE"

# Generate Rust files using handlebars-cli
echo "   Generating task_enum.rs..."
npx handlebars \
    --input "$TEMPLATES_DIR/rust-knhk/task_enum.rs.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/src/task_enum.rs"

echo "   Generating state_machine.rs..."
npx handlebars \
    --input "$TEMPLATES_DIR/rust-knhk/state_machine.rs.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/src/state_machine.rs"

echo "   Generating hooks.rs..."
npx handlebars \
    --input "$TEMPLATES_DIR/rust-knhk/hooks.rs.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/src/hooks.rs"

echo "   Generating otel_spans.rs..."
npx handlebars \
    --input "$TEMPLATES_DIR/rust-knhk/otel_spans.rs.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/src/otel_spans.rs"

echo ""
echo "⚙️  Step 6: Generating configuration..."

echo "   Generating workflow.yaml..."
npx handlebars \
    --input "$TEMPLATES_DIR/config/workflow.yaml.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/config/workflow.yaml"

echo ""
echo "📊 Step 7: Generating Weaver registry..."

echo "   Generating registry.yaml..."
npx handlebars \
    --input "$TEMPLATES_DIR/weaver/registry.yaml.hbs" \
    --data "$CONTEXT_FILE" \
    --output "$OUTPUT_DIR/weaver/registry.yaml"

echo ""
echo "✅ Step 8: Validating generated code..."

# Validate Rust syntax
if command -v rustfmt &> /dev/null; then
    echo "   Running rustfmt..."
    rustfmt --check "$OUTPUT_DIR/src"/*.rs || {
        echo "   ⚠️  Formatting issues found, auto-formatting..."
        rustfmt "$OUTPUT_DIR/src"/*.rs
    }
else
    echo "   ⚠️  rustfmt not found, skipping format check"
fi

# Validate YAML syntax
if command -v yamllint &> /dev/null; then
    echo "   Running yamllint..."
    yamllint "$OUTPUT_DIR/config"/*.yaml || echo "   ⚠️  YAML validation warnings"
else
    echo "   ⚠️  yamllint not found, skipping YAML validation"
fi

# Validate Weaver registry
if command -v weaver &> /dev/null; then
    echo "   Running Weaver registry check..."
    weaver registry check -r "$OUTPUT_DIR/weaver/" || {
        echo "   ❌ Weaver validation failed"
        exit 1
    }
else
    echo "   ⚠️  weaver not found, skipping registry validation"
fi

echo ""
echo "✨ Code generation complete!"
echo ""
echo "Generated files:"
echo "  Rust code:   $OUTPUT_DIR/src/"
echo "  Config:      $OUTPUT_DIR/config/"
echo "  Weaver:      $OUTPUT_DIR/weaver/"
echo ""
echo "Next steps:"
echo "  1. Review generated code in $OUTPUT_DIR"
echo "  2. Integrate with your project:"
echo "     cp -r $OUTPUT_DIR/src/* rust/your-crate/src/"
echo "  3. Build and test:"
echo "     cargo build && cargo test"
echo "  4. Validate with Weaver:"
echo "     weaver registry check -r $OUTPUT_DIR/weaver/"
echo ""
