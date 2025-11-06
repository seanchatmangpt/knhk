package knhk.performance_budget

# Performance budget policies for KNHK operations
# Enforces 8-tick budget for hot path operations

# Check if ticks exceed budget
violation[msg] {
    input.ticks > 8
    msg := sprintf("Tick count %d exceeds budget (8 ticks)", [input.ticks])
}

# Check if operation is within budget
within_budget {
    input.ticks <= 8
}

# Check if operation exceeds SLO
slo_violation[msg] {
    input.runtime_class == "R1"
    input.latency_ns > 1000  # 1ms SLO for R1
    msg := sprintf("R1 latency %d ns exceeds SLO (1000 ns)", [input.latency_ns])
}

slo_violation[msg] {
    input.runtime_class == "W1"
    input.latency_ns > 1000000  # 1ms SLO for W1
    msg := sprintf("W1 latency %d ns exceeds SLO (1000000 ns)", [input.latency_ns])
}

slo_violation[msg] {
    input.runtime_class == "C1"
    input.latency_ns > 100000000  # 100ms SLO for C1
    msg := sprintf("C1 latency %d ns exceeds SLO (100000000 ns)", [input.latency_ns])
}

# Budget definitions
tick_budget = 8
r1_slo_ns = 1000
w1_slo_ns = 1000000
c1_slo_ns = 100000000

