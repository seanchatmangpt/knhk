# How-To: Debug and Fix Workflow Deadlocks

**Advanced Guide to Detecting, Understanding, and Fixing Workflow Deadlocks**

- **Time to Complete**: ~60 minutes
- **Difficulty Level**: Intermediate to Advanced
- **Prerequisites**: Understanding of YAWL patterns, basic workflow knowledge
- **You'll Learn**: Deadlock detection, analysis, and prevention strategies

---

## Table of Contents

1. [Understanding Deadlocks](#understanding-deadlocks)
2. [Design-Time Detection](#design-time-detection)
3. [Runtime Detection](#runtime-detection)
4. [Deadlock Analysis](#deadlock-analysis)
5. [Fix Strategies](#fix-strategies)
6. [Prevention Best Practices](#prevention-best-practices)

---

## Understanding Deadlocks

### What is a Deadlock?

A **deadlock** occurs when a workflow reaches a state where no tasks can execute, yet the workflow isn't complete. The case is permanently stuck.

**Visual Example**:

```
Entry Task
    ↓
  Split
    ├─→ Task A ──→┐
    │             ├─→ Join (BLOCKED!)
    └─→ Task B ──→┘

Problem: Join is waiting for both paths
         But path A leads to a dead end
         Path B never starts
         Result: DEADLOCK
```

### Types of Deadlocks

#### 1. **Structural Deadlock** (Design-Time)

Deadlock is inherent to the workflow design.

**Example: Mismatched Split-Join**

```turtle
# ❌ WRONG: Sync join with missing split
:split1 yawl:splitType "AND" ;
        yawl:child :task1 ;
        yawl:child :task2 .

# Task 1 leads to join
:task1 yawl:next [
  yawl:join :join1
] .

# Task 2 dead end - no path to join
:task2 yawl:next :end .
# Result: join1 waits forever for task2 path
```

**Detection**: Caught by deadlock analyzer at registration time.

#### 2. **Behavioral Deadlock** (Runtime)

Deadlock emerges from specific runtime conditions or data values.

**Example: Guard Prevents All Paths**

```turtle
# Design looks sound
:split1 yawl:splitType "OR" ;
        yawl:child [
          :guard "amount < 1000" ;
          :next :approve_fast
        ] ;
        yawl:child [
          :guard "amount >= 1000" ;
          :next :approve_slow
        ] .

# At runtime: amount = 500, so first guard true, approve_fast runs
# But what if external service is down?
# approve_fast never completes, join waits forever
```

**Detection**: Only visible during execution monitoring.

#### 3. **Circular Dependency**

Tasks waiting for each other.

```turtle
# ❌ WRONG: Circular reference
:task1 workflow:next :task2 .
:task2 workflow:next :task3 .
:task3 workflow:next :task1 .  # Back to task1!
```

**Detection**: Caught by graph analysis.

---

## Design-Time Detection

### Step 1: Enable Deadlock Checking

```bash
# Validate workflow before registration
curl -X POST http://localhost:8080/api/v1/workflows/validate \
  -H "Content-Type: application/json" \
  -d '{
    "spec": "'"$(cat workflow.ttl)"'",
    "check_deadlock": true,
    "check_soundness": true
  }' | jq .
```

### Step 2: Interpret Results

**Good Result**:
```json
{
  "valid": true,
  "checks": {
    "deadlock": "no_deadlock_detected",
    "soundness": "sound"
  }
}
```

**Deadlock Detected**:
```json
{
  "valid": false,
  "errors": [
    {
      "type": "deadlock_detected",
      "message": "Sync join without matching split",
      "location": "task:approve",
      "suggestion": "Check split/join nesting levels"
    }
  ]
}
```

### Step 3: Understand Error Location

Error includes:
- **Location**: Which task/node has the issue
- **Message**: What deadlock pattern was found
- **Suggestion**: How to fix it

### Common Deadlock Patterns

#### Pattern 1: Sync Join Without Split

```turtle
# ❌ WRONG
:split1 [
  yawl:splitType "AND" ;
  yawl:child :task1 ;
  yawl:child :task2
] .

:task1 yawl:next :join1 .
:task2 yawl:next :final_task .  # Wrong! Missing path to join
:join1 yawl:next :final_task .
```

**Fix**: Ensure all branches lead to corresponding join

```turtle
# ✅ CORRECT
:split1 [
  yawl:splitType "AND" ;
  yawl:child :task1 ;
  yawl:child :task2
] .

:task1 yawl:next :join1 .
:task2 yawl:next :join1 .      # Both to join
:join1 yawl:next :final_task .
```

#### Pattern 2: Unbalanced Nesting

```turtle
# ❌ WRONG: Nested splits without matching joins
:outer_split [
  yawl:splitType "AND" ;
  yawl:child :inner_split [
    yawl:splitType "AND" ;
    yawl:child :task1 ;
    yawl:child :task2
  ] ;
  yawl:child :task3
] .

# Problem: inner join nests inside outer split
# But task3 doesn't enter inner split
# Levels don't match!
```

**Fix**: Match nesting levels

```turtle
# ✅ CORRECT: Balanced nesting
:outer_split [
  yawl:splitType "AND" ;
  yawl:child [
    yawl:inner_split [
      yawl:splitType "AND" ;
      yawl:child :task1 ;
      yawl:child :task2
    ] ;
    yawl:join :inner_join
  ] ;
  yawl:child :task3
] .
:outer_join [ ... ] .
```

#### Pattern 3: Circular Reference

```turtle
# ❌ WRONG: Circular
:task1 yawl:next :task2 .
:task2 yawl:next :task3 .
:task3 yawl:next :task1 .  # Loop back!
```

**Fix**: Ensure linear flow or explicit loop structure

```turtle
# ✅ CORRECT: Linear with optional repeat
:task1 yawl:next :task2 .
:task2 yawl:next :task3 .
:task3 yawl:next [
  yawl:condition [
    yawl:trueBranch :task1 ;   # Explicit repeat
    yawl:falseBranch :exit
  ]
] .
```

---

## Runtime Detection

### Step 1: Monitor Case Progress

Even well-designed workflows can deadlock at runtime due to:
- External service failures
- Unexpected data conditions
- Race conditions

**Monitoring Script**:

```bash
#!/bin/bash

CASE_ID=$1
CHECK_INTERVAL=10
STALL_THRESHOLD=300  # 5 minutes

LAST_PROGRESS=0
LAST_CHECK=$(date +%s)

while true; do
  # Get current case status
  RESPONSE=$(curl -s http://localhost:8080/api/v1/cases/$CASE_ID)

  CURRENT_PROGRESS=$(echo $RESPONSE | jq '.progress.percentage')
  CURRENT_STATE=$(echo $RESPONSE | jq -r '.state')

  echo "Progress: $CURRENT_PROGRESS%, State: $CURRENT_STATE"

  # Check if progress stalled
  if [ "$CURRENT_PROGRESS" = "$LAST_PROGRESS" ] && [ "$CURRENT_STATE" = "active" ]; then
    ELAPSED=$(($(date +%s) - $LAST_CHECK))

    if [ $ELAPSED -gt $STALL_THRESHOLD ]; then
      echo "⚠️  DEADLOCK DETECTED: No progress for $ELAPSED seconds"
      echo "Case details:"
      curl -s http://localhost:8080/api/v1/cases/$CASE_ID | jq .
      exit 1
    fi
  else
    LAST_CHECK=$(date +%s)
  fi

  LAST_PROGRESS=$CURRENT_PROGRESS

  sleep $CHECK_INTERVAL
done
```

Run the monitor:

```bash
chmod +x monitor-case.sh
./monitor-case.sh case_001
```

### Step 2: Analyze Stuck Cases

When a case appears stuck:

```bash
# Get detailed case information
curl http://localhost:8080/api/v1/cases/case_001 | jq .

# Key fields to check:
# - state: Should be "active", not "suspended" or "failed"
# - enabled_tasks: Should have tasks that can execute
# - data: Check if guards might be blocking
# - execution_time: How long has it been running?

# Get execution history
curl http://localhost:8080/api/v1/cases/case_001/history | jq .

# Look for:
# - Last completed task
# - When did it stop progressing?
# - Any error events?
```

### Step 3: Check for External Blocks

```bash
# Get enabled tasks
ENABLED=$(curl -s http://localhost:8080/api/v1/cases/case_001 | jq '.enabled_tasks')
echo "Enabled tasks: $ENABLED"

# For each enabled task, try to complete manually
curl -X POST http://localhost:8080/api/v1/cases/case_001/tasks/task_name/complete \
  -d '{"result": "manual_completion", "output_data": {}}'

# If completes successfully → external block was the issue
# If fails with guard violation → data issue
# If fails with error → task implementation issue
```

---

## Deadlock Analysis

### Using Petri Net Analysis

KNHK workflows map to Petri nets. Use specialized tools:

**Export Workflow**:

```bash
# Export as PNML (Petri Net Markup Language)
curl http://localhost:8080/api/v1/workflows/wf_001/export?format=pnml > workflow.pnml

# Open in ProM (Process Mining Tool)
# http://promtools.org/
```

**Analyze in ProM**:

1. Open ProM Lite
2. Import PNML file
3. Run "Deadlock Analyzer" plugin
4. Review detected deadlock patterns

### Using Graph Analysis

Create a dependency graph:

```bash
#!/bin/bash

# Extract task dependencies from Turtle
grep "yawl:next\|workflow:next\|yawl:child" workflow.ttl | \
  grep -oE ':[a-zA-Z0-9_]+' | \
  paste - - > edges.txt

# Analyze for cycles
# If A→B→C→A exists: CYCLE (potential deadlock)

# Check for unreachable nodes
# If node has no incoming edges: potentially unreachable
```

### Manual Walkthrough

Trace execution paths manually:

```
1. Start at entry task
2. For each enabled task:
   - Can it complete?
   - What task becomes enabled after?
3. Continue until exit or stuck
4. If stuck: that's the deadlock point
```

---

## Fix Strategies

### Strategy 1: Fix Workflow Design

If deadlock is structural, fix the design:

```turtle
# ❌ Original (deadlock)
:split1 [
  yawl:splitType "AND" ;
  yawl:child :task_a ;
  yawl:child :task_b
] .

:task_a yawl:next :join1 .
:task_b yawl:next :error_handler .  # Wrong! Missing join

# ✅ Fixed version
:split1 [
  yawl:splitType "AND" ;
  yawl:child :task_a ;
  yawl:child :task_b
] .

:task_a yawl:next :join1 .
:task_b yawl:next :join1 .          # Both to join
:join1 yawl:next :next_task .
```

**Deploy Fixed Version**:

```bash
# Register new workflow version
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "order-processing",
    "version": "2.0.0",
    "spec": "'"$(cat fixed-workflow.ttl)"'"
  }'

# Migrate in-progress cases (if possible)
# Or create new cases with fixed workflow
```

### Strategy 2: Add Timeout Protection

If deadlock is behavioral (external service failure):

```turtle
# Add timeout task
:task_with_timeout [
  yawl:timeout "PT30S" ;  # 30 second timeout
  yawl:onTimeout :timeout_handler ;
  yawl:task :external_call
] .

:timeout_handler yawl:next :fallback_task .
:fallback_task yawl:next :join1 .
```

### Strategy 3: Add Guard Conditions

If deadlock from data issues:

```turtle
# Add guards to prevent impossible conditions
:split_task [
  yawl:child [
    yawl:guard "amount < 1000" ;
    yawl:next :fast_approval
  ] ;
  yawl:child [
    yawl:guard "amount >= 1000" ;
    yawl:next :slow_approval
  ] ;
  yawl:child [
    yawl:guard "true" ;  # Fallback for edge cases
    yawl:next :manual_review
  ]
] .
```

### Strategy 4: Add Explicit Error Handling

```turtle
# Handle task failures explicitly
:risky_task [
  yawl:task :external_api_call ;
  yawl:onError :error_handler ;
  yawl:onSuccess :success_handler
] .

:error_handler yawl:next :join1 .
:success_handler yawl:next :join1 .
```

---

## Recovery from Runtime Deadlock

### Option 1: Manual Task Completion

If external task completes externally:

```bash
# Complete the blocked task manually
curl -X POST http://localhost:8080/api/v1/cases/case_001/tasks/blocked_task/complete \
  -d '{
    "result": "completed_externally",
    "output_data": {"status": "success"}
  }'

# This may unblock subsequent joins
```

### Option 2: Cancel and Restart

If case is unrecoverable:

```bash
# Cancel stuck case
curl -X POST http://localhost:8080/api/v1/cases/case_001/cancel \
  -d '{"reason": "Deadlock - restarting with fixed workflow"}'

# Create new case with fixed workflow
curl -X POST http://localhost:8080/api/v1/workflows/wf_001_fixed/cases \
  -d '{"case_id": "case_001_retry", "data": {...}}'
```

### Option 3: Intervention by Administrator

```bash
# Enable a waiting task to proceed
curl -X POST http://localhost:8080/api/v1/cases/case_001/tasks/waiting_task/enable

# Update case data to satisfy guards
curl -X PATCH http://localhost:8080/api/v1/cases/case_001/data \
  -d '{
    "updates": {
      "field": "new_value"
    }
  }'
```

---

## Prevention Best Practices

### 1. Design for Clarity

```turtle
# ✅ GOOD: Clear, explicit structure
:split1 [ :splitType "AND" ;
  :child [
    :task1 ;
    :join1
  ] ;
  :child [
    :task2 ;
    :join1
  ]
] .

# ❌ AVOID: Implicit, complex nesting
:complex [
  :nested [
    :deeper [
      :task1
    ]
  ] ;
  :join [ :nested [ :deeper [ :task2 ] ] ]
] .
```

### 2. Always Use Soundness Checking

```bash
# Validate before registration
curl -X POST http://localhost:8080/api/v1/workflows/validate \
  -d '{
    "spec": "...",
    "check_soundness": true,
    "check_deadlock": true
  }'
```

### 3. Test Common Paths

Create test cases for:
- Normal flow (all conditions met)
- Edge cases (boundary conditions)
- Error scenarios (external failures)

```bash
# Test normal flow
curl -X POST http://localhost:8080/api/v1/workflows/wf/cases \
  -d '{"data": {"amount": 500}}'  # Normal amount

# Test edge case
curl -X POST http://localhost:8080/api/v1/workflows/wf/cases \
  -d '{"data": {"amount": 0}}'    # Boundary

# Test error
curl -X POST http://localhost:8080/api/v1/workflows/wf/cases \
  -d '{"data": {"amount": 999999}}'  # Large amount
```

### 4. Implement Monitoring

```bash
# Use the monitoring script from Step 1
./monitor-case.sh case_001

# Set alerts for:
# - No progress for 5+ minutes
# - Error rate > 1%
# - Queue depth growth
```

### 5. Document Assumptions

```turtle
# Add comments explaining design
# Example workflow with assumptions

# Assumption 1: Only two outcomes from approval
# Either approved (fast path) or rejected (admin path)
:approval_decision [
  :child [ :guard "approved = true" ; :next :fast_process ] ;
  :child [ :guard "approved = false" ; :next :admin_review ]
] .

# Assumption 2: Both paths rejoin at same point
:fast_process :next :final_join .
:admin_review :next :final_join .
```

---

## Checklist for Deadlock-Free Workflows

Before deploying:

- [ ] Run deadlock checker: `check_deadlock = true`
- [ ] Run soundness checker: `check_soundness = true`
- [ ] Review all split-join pairs for balance
- [ ] Verify no circular references exist
- [ ] Test all guard conditions with sample data
- [ ] Test with invalid/edge case data
- [ ] Implement timeouts for external calls
- [ ] Add error handlers for failure paths
- [ ] Document assumptions about data
- [ ] Set up case progress monitoring
- [ ] Plan recovery if deadlock occurs

---

## Related Documentation

- [How-To: Troubleshooting](./troubleshooting.md) - General troubleshooting
- [Configuration Guide](../reference/configuration.md) - Validation settings
- [Error Codes Reference](../reference/error-codes.md) - Deadlock errors
- [Explanation: Event Sourcing](../explanations/event-sourcing.md) - State tracking
