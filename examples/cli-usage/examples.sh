#!/bin/bash
# examples/cli-usage/examples.sh
# CLI usage examples

set -e

echo "=========================================="
echo "KNHK CLI Usage Examples"
echo "=========================================="

# Boot
echo "=== Boot Commands ==="
echo "knhk boot init schema.ttl invariants.sparql"
echo ""

# Connect
echo "=== Connect Commands ==="
echo "knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples"
echo "knhk connect list"
echo ""

# Cover
echo "=== Cover Commands ==="
echo "knhk cover define \"SELECT ?s ?p ?o WHERE { ?s ?p ?o }\" \"max_run_len 8\""
echo "knhk cover list"
echo ""

# Admit
echo "=== Admit Commands ==="
echo "knhk admit delta delta.json"
echo ""

# Reflex
echo "=== Reflex Commands ==="
echo "knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8"
echo "knhk reflex list"
echo ""

# Epoch
echo "=== Epoch Commands ==="
echo "knhk epoch create epoch1 8 \"reflex1,reflex2\""
echo "knhk epoch run epoch1"
echo "knhk epoch list"
echo ""

# Route
echo "=== Route Commands ==="
echo "knhk route install webhook1 webhook https://api.example.com/webhook"
echo "knhk route list"
echo ""

# Receipt
echo "=== Receipt Commands ==="
echo "knhk receipt get receipt-123"
echo "knhk receipt merge receipt-1,receipt-2,receipt-3"
echo "knhk receipt list"
echo "knhk receipt verify receipt-123"
echo "knhk receipt show receipt-123"
echo ""

# Pipeline
echo "=== Pipeline Commands ==="
echo "knhk pipeline run --connectors kafka-prod"
echo "knhk pipeline status"
echo ""

# Metrics
echo "=== Metrics Commands ==="
echo "knhk metrics get"
echo ""

# Coverage
echo "=== Coverage Commands ==="
echo "knhk coverage get"
echo ""

# Hook
echo "=== Hook Commands ==="
echo "knhk hook create test-ask ASK_SP 0xC0FFEE 0 8 0x1234"
echo "knhk hook list"
echo "knhk hook eval test-ask"
echo "knhk hook show test-ask"
echo ""

# Context
echo "=== Context Commands ==="
echo "knhk context create prod \"Production\" urn:knhk:schema:enterprise"
echo "knhk context list"
echo "knhk context switch prod"
echo "knhk context current"
echo ""

echo "=========================================="
echo "Examples complete"
echo "=========================================="
echo ""
echo "Note: These are example commands. Modify as needed for your use case."
