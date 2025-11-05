#!/bin/bash
# examples/basic-hook/run.sh
# Basic hook execution example

set -e

echo "=========================================="
echo "KNHK Basic Hook Example"
echo "=========================================="

# Create test data
cat > hook.ttl <<EOF
@prefix ex: <http://example.org/> .

ex:alice ex:role ex:admin .
ex:bob ex:role ex:user .
ex:charlie ex:role ex:guest .
EOF

cat > invariants.sparql <<EOF
# Basic invariants
ASK WHERE { ?s ?p ?o }
EOF

echo "✓ Created test data files"

# Initialize system
echo "Initializing system..."
knhk boot init hook.ttl invariants.sparql || echo "⚠ System already initialized"

# Create hook
echo "Creating hook..."
knhk hook create check-exists ASK_SP 0xC0FFEE 0 8 || echo "⚠ Hook may already exist"

# Show hook
echo "Hook details:"
knhk hook show check-exists

echo ""
echo "=========================================="
echo "Example complete!"
echo "=========================================="

