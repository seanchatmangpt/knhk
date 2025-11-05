#!/bin/bash
# Receipt Verification Example
# Demonstrates receipt operations

set -e

echo "=== Receipt Verification Example ==="
echo ""

# Create and execute hook to generate receipt
echo "Creating hook to generate receipt..."
knhk hook create test-hook ASK_SP 0xC0FFEE 0 8

echo ""
echo "Executing hook to generate receipt..."
knhk hook eval test-hook

# Get receipt (example ID - replace with actual receipt ID from hook execution)
echo ""
echo "Getting receipt..."
# Note: Replace with actual receipt ID from hook execution
knhk receipt list

# Verify receipt
echo ""
echo "Verifying receipt..."
# Note: Replace with actual receipt ID
# knhk receipt verify receipt_1234567890abcdef

# List all receipts
echo ""
echo "Listing all receipts..."
knhk receipt list

echo ""
echo "=== Example Complete ==="

