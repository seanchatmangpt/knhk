#!/bin/bash
# Single-Piece Flow Agent
# Eliminates batching waste by enforcing one-task-at-a-time flow
#
# Usage: ./scripts/flow-agent.sh "task description"
#
# Flow Efficiency Target: 66% (from 12.5% baseline)
# Waste Reduction: 4.2 hours batching waste eliminated

set -euo pipefail

TASK="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Timestamp for metrics
START_TIME=$(date +%s)

log_info() {
    echo -e "${BLUE}[FLOW]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Validate task provided
if [ -z "$TASK" ]; then
    log_error "No task provided"
    echo "Usage: $0 \"task description\""
    exit 1
fi

# Check WIP limit (max 1-2 tasks)
check_wip_limit() {
    log_info "Checking WIP limit..."

    # Count current git branches (excluding main/master)
    BRANCH_COUNT=$(git branch 2>/dev/null | grep -v -E '(main|master|\*)' | wc -l | tr -d ' ')

    if [ "$BRANCH_COUNT" -gt 2 ]; then
        log_error "WIP limit exceeded: $BRANCH_COUNT active branches (max 2)"
        log_error "Complete existing tasks before starting new ones"
        git branch | grep -v -E '(main|master)'
        exit 1
    fi

    log_success "WIP limit OK: $BRANCH_COUNT active tasks"
}

# Gate 0 validation (before starting)
gate_0_pre() {
    log_info "Running Gate 0 pre-validation..."

    if [ -f "$SCRIPT_DIR/gate-0-validation.sh" ]; then
        bash "$SCRIPT_DIR/gate-0-validation.sh"
    else
        log_warn "gate-0-validation.sh not found, skipping pre-validation"
    fi
}

# Gate 0 validation (after completion)
gate_0_post() {
    log_info "Running Gate 0 post-validation..."

    if [ -f "$SCRIPT_DIR/gate-0-validation.sh" ]; then
        bash "$SCRIPT_DIR/gate-0-validation.sh"
    else
        log_warn "gate-0-validation.sh not found, skipping post-validation"
    fi
}

# Execute single-piece flow
execute_flow() {
    log_info "========================================="
    log_info "Single-Piece Flow Execution"
    log_info "Task: $TASK"
    log_info "========================================="

    # Step 1: WIP limit check
    check_wip_limit

    # Step 2: Pre-validation
    gate_0_pre

    # Step 3: Record start
    log_info "Starting task: $TASK"
    echo "$TASK" > /tmp/knhk-current-task.txt

    # Step 4: Task execution guidance
    cat <<PROTOCOL

${GREEN}Single-Piece Flow Protocol:${NC}

1. ${BLUE}Implement${NC} - Write code for this task ONLY
   - Focus on one feature/fix at a time
   - No partial implementations
   - No "TODO" comments

2. ${BLUE}Test${NC} - Verify implementation immediately
   - Write tests for this task ONLY
   - Run tests until all pass
   - No deferring test writing

3. ${BLUE}Document${NC} - Update docs for this task ONLY
   - Document new behavior
   - Update relevant READMEs
   - Keep docs in sync with code

4. ${BLUE}Verify${NC} - Complete validation before moving on
   - All tests pass
   - Gate 0 validation passes
   - No broken functionality

5. ${BLUE}Commit${NC} - Complete handoff to next task
   - Atomic commit for this task
   - Clear commit message
   - Push to remote

${YELLOW}Remember:${NC} Do NOT start next task until this one is COMPLETE

PROTOCOL

    # Wait for user to indicate completion
    read -p "Press ENTER when task implementation is complete..."

    # Step 5: Post-validation
    log_info "Validating task completion..."
    gate_0_post

    # Step 6: Record completion
    log_success "Task complete: $TASK"
    rm -f /tmp/knhk-current-task.txt

    # Step 7: Calculate metrics
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    MINUTES=$((DURATION / 60))
    SECONDS=$((DURATION % 60))

    log_info "Task duration: ${MINUTES}m ${SECONDS}s"

    # Step 8: Store metrics in database
    if [ -f "$PROJECT_ROOT/.swarm/memory.db" ]; then
        log_info "Storing flow metrics in .swarm/memory.db..."
        sqlite3 "$PROJECT_ROOT/.swarm/memory.db" <<SQL
CREATE TABLE IF NOT EXISTS memory (
    key TEXT PRIMARY KEY,
    value TEXT,
    timestamp INTEGER
);
INSERT OR REPLACE INTO memory (key, value, timestamp) VALUES ('dflss/flow/last-task', '$TASK', $(date +%s));
INSERT OR REPLACE INTO memory (key, value, timestamp) VALUES ('dflss/flow/last-duration', '${DURATION}s', $(date +%s));
SQL
    fi
}

# Main execution
main() {
    cd "$PROJECT_ROOT"
    execute_flow

    log_success "========================================="
    log_success "Single-Piece Flow Complete"
    log_success "Ready for next task"
    log_success "========================================="
}

main
