#!/bin/bash
# examples/receipt-verification/verify.sh
# Receipt verification example

set -e

echo "=========================================="
echo "Receipt Verification Example"
echo "=========================================="

# List receipts
echo "Listing receipts..."
knhk receipt list

# Get a receipt (if available)
echo ""
echo "Getting receipt..."
if RECEIPT_ID=$(knhk receipt list | grep -o "receipt-[0-9]*" | head -1); then
    echo "Found receipt: $RECEIPT_ID"
    knhk receipt get "$RECEIPT_ID"
    echo ""
    echo "Verifying receipt..."
    knhk receipt verify "$RECEIPT_ID"
    echo ""
    echo "Showing receipt details..."
    knhk receipt show "$RECEIPT_ID"
else
    echo "No receipts found. Run pipeline or hooks first."
fi

echo ""
echo "=========================================="
echo "Receipt verification complete"
echo "=========================================="
