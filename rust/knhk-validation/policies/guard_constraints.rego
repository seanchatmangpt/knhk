package knhk.guard_constraints

# Guard constraint policies for KNHK operations
# Enforces max_run_len ≤ 8 (Chatman Constant)

# Check if run length exceeds maximum allowed
violation[msg] {
    input.run_len > 8
    msg := sprintf("Run length %d exceeds maximum allowed (8)", [input.run_len])
}

# Check if run length is valid
valid {
    input.run_len <= 8
    input.run_len > 0
}

# Get maximum allowed run length
max_run_len = 8

