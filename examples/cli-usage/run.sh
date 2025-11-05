#!/bin/bash
# CLI Usage Examples
# Demonstrates common CLI usage patterns

set -e

echo "=== CLI Usage Examples ==="
echo ""

echo "1. Basic Commands"
echo "-------------------"
echo "Initializing system..."
knhk boot init schema.ttl invariants.sparql

echo ""
echo "Registering connector..."
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

echo ""
echo "Listing connectors..."
knhk connect list

echo ""
echo "2. Hook Management"
echo "-------------------"
echo "Creating hook..."
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8

echo ""
echo "Listing hooks..."
knhk hook list

echo ""
echo "3. Context Management"
echo "-------------------"
echo "Creating context..."
knhk context create prod1 Production urn:knhk:schema:enterprise

echo ""
echo "Listing contexts..."
knhk context list

echo ""
echo "4. Receipt Operations"
echo "-------------------"
echo "Listing receipts..."
knhk receipt list

echo ""
echo "=== Examples Complete ==="

