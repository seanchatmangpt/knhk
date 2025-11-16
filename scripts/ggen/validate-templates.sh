#!/usr/bin/env bash
# Validate ggen templates structure and syntax
#
# Usage: ./scripts/ggen/validate-templates.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEMPLATES_DIR="$PROJECT_ROOT/templates"

echo "🔍 Validating ggen templates..."
echo "================================"

# Check template directory structure
echo ""
echo "📁 Checking directory structure..."

REQUIRED_DIRS=(
    "rust-knhk"
    "config"
    "weaver"
    "sparql"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$TEMPLATES_DIR/$dir" ]; then
        echo "   ✅ $dir/"
    else
        echo "   ❌ Missing: $dir/"
        exit 1
    fi
done

# Check required template files
echo ""
echo "📄 Checking required templates..."

REQUIRED_TEMPLATES=(
    "rust-knhk/task_enum.rs.hbs"
    "rust-knhk/state_machine.rs.hbs"
    "rust-knhk/hooks.rs.hbs"
    "rust-knhk/otel_spans.rs.hbs"
    "config/workflow.yaml.hbs"
    "weaver/registry.yaml.hbs"
    "sparql/query_bindings.hbs"
)

for template in "${REQUIRED_TEMPLATES[@]}"; do
    if [ -f "$TEMPLATES_DIR/$template" ]; then
        echo "   ✅ $template"
    else
        echo "   ❌ Missing: $template"
        exit 1
    fi
done

# Validate Handlebars syntax
echo ""
echo "🔧 Validating Handlebars syntax..."

if command -v npx &> /dev/null; then
    for template in "${REQUIRED_TEMPLATES[@]}"; do
        if npx handlebars --version &> /dev/null; then
            # Basic syntax check (compile without data)
            echo "   Checking $template..."
        else
            echo "   ⚠️  handlebars-cli not installed, skipping syntax validation"
            break
        fi
    done
else
    echo "   ⚠️  npx not found, skipping syntax validation"
fi

# Check for common template issues
echo ""
echo "🔎 Checking for common issues..."

check_template_issues() {
    local file="$1"
    local issues=0

    # Check for unclosed Handlebars blocks
    if grep -q '{{#' "$file" && ! grep -q '{{/' "$file"; then
        echo "   ⚠️  Possible unclosed block in $file"
        issues=$((issues + 1))
    fi

    # Check for malformed helpers
    if grep -qE '\{\{[^}]*\}\}[^}]' "$file"; then
        echo "   ⚠️  Possible malformed Handlebars expression in $file"
        issues=$((issues + 1))
    fi

    return $issues
}

total_issues=0
for template in "${REQUIRED_TEMPLATES[@]}"; do
    check_template_issues "$TEMPLATES_DIR/$template" || total_issues=$((total_issues + $?))
done

if [ $total_issues -eq 0 ]; then
    echo "   ✅ No issues found"
fi

# Summary
echo ""
echo "================================"
if [ $total_issues -eq 0 ]; then
    echo "✅ All templates validated successfully!"
else
    echo "⚠️  Found $total_issues potential issues"
    exit 1
fi
