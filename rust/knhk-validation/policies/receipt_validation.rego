package knhk.receipt_validation

# Receipt validation policies for KNHK operations
# Validates receipt structure, hash, and provenance

# Check if receipt ID is valid (non-empty)
violation[msg] {
    input.receipt_id == ""
    msg := "Receipt ID cannot be empty"
}

# Check if receipt hash is valid (32 bytes = 64 hex chars)
violation[msg] {
    count(input.receipt_hash) != 32
    msg := sprintf("Receipt hash must be 32 bytes, got %d", [count(input.receipt_hash)])
}

# Check if receipt ticks are within budget
violation[msg] {
    input.ticks > 8
    msg := sprintf("Receipt ticks %d exceed budget (8)", [input.ticks])
}

# Check if receipt has valid timestamp
violation[msg] {
    input.timestamp_ms == 0
    msg := "Receipt timestamp cannot be zero"
}

# Validate receipt structure
valid {
    input.receipt_id != ""
    count(input.receipt_hash) == 32
    input.ticks <= 8
    input.timestamp_ms > 0
}

