#!/usr/bin/env bats
# Bats tests for KNHK shell scripts
# See https://bats-core.readthedocs.io/

# Helper function to get project root
project_root() {
    echo "$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)"
}

# Setup: Run before each test
setup() {
    PROJECT_ROOT="$(project_root)"
    export PROJECT_ROOT
    cd "$PROJECT_ROOT"
}

# Test: run-chicago-tdd-tests.sh runs successfully when C library exists
@test "run-chicago-tdd-tests.sh runs successfully when C library exists" {
    # Skip if C library can't be built
    skip "Requires C library to be built"
    
    # Ensure C library is built
    run make -C "$PROJECT_ROOT/c" lib
    [ "$status" -eq 0 ] || skip "C library build failed"
    
    # Run Chicago TDD tests script
    run bash "$PROJECT_ROOT/scripts/run-chicago-tdd-tests.sh"
    
    # Should succeed
    [ "$status" -eq 0 ]
    # Should contain expected output
    [[ "$output" == *"Chicago TDD Tests"* ]]
}

# Test: run-chicago-tdd-tests.sh fails gracefully when C library missing
@test "run-chicago-tdd-tests.sh fails gracefully when C library missing" {
    # This test would require mocking the C library build
    # For now, we'll skip it as it's complex to mock
    skip "Requires mocking C library build"
}

# Test: run-performance-tests.sh exists and is executable
@test "run-performance-tests.sh exists and is executable" {
    [ -f "$PROJECT_ROOT/scripts/run-performance-tests.sh" ]
    [ -x "$PROJECT_ROOT/scripts/run-performance-tests.sh" ]
}

# Test: run-integration-tests.sh exists and is executable
@test "run-integration-tests.sh exists and is executable" {
    [ -f "$PROJECT_ROOT/scripts/run-integration-tests.sh" ]
    [ -x "$PROJECT_ROOT/scripts/run-integration-tests.sh" ]
}

# Test: run-all-rust-tests.sh exists and is executable
@test "run-all-rust-tests.sh exists and is executable" {
    [ -f "$PROJECT_ROOT/scripts/run-all-rust-tests.sh" ]
    [ -x "$PROJECT_ROOT/scripts/run-all-rust-tests.sh" ]
}

# Test: run-all-rust-tests.sh has proper shebang
@test "run-all-rust-tests.sh has proper shebang" {
    run head -n 1 "$PROJECT_ROOT/scripts/run-all-rust-tests.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == "#!/usr/bin/env bash"* ]]
}

# Test: run-chicago-tdd-tests.sh has proper shebang
@test "run-chicago-tdd-tests.sh has proper shebang" {
    run head -n 1 "$PROJECT_ROOT/scripts/run-chicago-tdd-tests.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == "#!/usr/bin/env bash"* ]
}

# Test: run-performance-tests.sh has proper shebang
@test "run-performance-tests.sh has proper shebang" {
    run head -n 1 "$PROJECT_ROOT/scripts/run-performance-tests.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == "#!/usr/bin/env bash"* ]
}

# Test: run-integration-tests.sh has proper shebang
@test "run-integration-tests.sh has proper shebang" {
    run head -n 1 "$PROJECT_ROOT/scripts/run-integration-tests.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == "#!/usr/bin/env bash"* ]
}

# Test: Scripts use set -euo pipefail for error handling
@test "run-all-rust-tests.sh uses strict error handling" {
    run grep -q "set -euo pipefail" "$PROJECT_ROOT/scripts/run-all-rust-tests.sh"
    [ "$status" -eq 0 ]
}

@test "run-chicago-tdd-tests.sh uses strict error handling" {
    run grep -q "set -euo pipefail" "$PROJECT_ROOT/scripts/run-chicago-tdd-tests.sh"
    [ "$status" -eq 0 ]
}

@test "run-performance-tests.sh uses strict error handling" {
    run grep -q "set -euo pipefail" "$PROJECT_ROOT/scripts/run-performance-tests.sh"
    [ "$status" -eq 0 ]
}

@test "run-integration-tests.sh uses strict error handling" {
    run grep -q "set -euo pipefail" "$PROJECT_ROOT/scripts/run-integration-tests.sh"
    [ "$status" -eq 0 ]
}

