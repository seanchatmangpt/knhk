# Shell Script Tests

This directory contains bats tests for KNHK shell scripts.

## Prerequisites

Install bats:

```bash
# macOS
brew install bats-core

# Linux (Ubuntu/Debian)
sudo apt-get install bats

# Linux (Fedora)
sudo dnf install bats
```

## Running Tests

```bash
# Run all shell script tests
make test-shell

# Or directly
bats tests/shell/*.bats

# Run specific test file
bats tests/shell/scripts.bats
```

## Test Coverage

Currently tests:
- `scripts/run-chicago-tdd-tests.sh`
- `scripts/run-performance-tests.sh`
- `scripts/run-integration-tests.sh`
- `scripts/run-all-rust-tests.sh`

Each test validates:
- Script exists and is executable
- Proper shebang (`#!/usr/bin/env bash`)
- Strict error handling (`set -euo pipefail`)

## Adding New Tests

Create a new `.bats` file in this directory:

```bash
#!/usr/bin/env bats

@test "my-script.sh exists and is executable" {
    [ -f "$PROJECT_ROOT/scripts/my-script.sh" ]
    [ -x "$PROJECT_ROOT/scripts/my-script.sh" ]
}
```

See [bats documentation](https://bats-core.readthedocs.io/) for more examples.

