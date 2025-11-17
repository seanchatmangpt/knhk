# Tutorial: Your First Workflow

**A Step-by-Step Guide to Creating and Executing Your First KNHK Workflow**

- **Time to Complete**: ~45 minutes
- **Difficulty Level**: Beginner
- **Prerequisites**: Rust installed, Git cloned, basic understanding of workflows
- **You'll Learn**: Create, register, and execute a simple workflow

---

## Table of Contents

1. [Setup & Installation](#setup--installation)
2. [Understand the Basics](#understand-the-basics)
3. [Create Your First Workflow](#create-your-first-workflow)
4. [Register the Workflow](#register-the-workflow)
5. [Execute and Monitor](#execute-and-monitor)
6. [Troubleshoot Common Issues](#troubleshoot-common-issues)
7. [Next Steps](#next-steps)

---

## Setup & Installation

### Step 1: Clone the Repository

```bash
# Clone KNHK repository
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk

# Navigate to workflow engine
cd rust/knhk-workflow-engine
```

### Step 2: Verify Installation

```bash
# Build the project
cargo build --release

# Verify it works
./target/release/knhk-workflow --help

# Output should show available commands
```

### Step 3: Start the Engine

```bash
# Terminal 1: Start the workflow engine
cargo run --release --bin knhk-workflow

# Output:
# INFO: Server listening on 0.0.0.0:8080
# INFO: OTEL tracing enabled
# INFO: Storage backend: Sled
```

Keep this terminal running for all steps below.

### Step 4: Verify Engine is Running

```bash
# Terminal 2: Check health
curl http://localhost:8080/health

# Output should be:
# {"status":"healthy","version":"1.0.0","timestamp":"2025-11-17T10:30:00Z"}
```

---

## Understand the Basics

Before creating your workflow, let's understand key concepts:

### What is a Workflow?

A **workflow** is a sequence of tasks that execute in a specific order to achieve a goal.

**Example**: Order Processing Workflow
```
1. Receive Order
2. Validate Order
3. Check Inventory
4. Process Payment
5. Ship Order
6. Send Confirmation
```

### Key Concepts

| Term | Meaning |
|------|---------|
| **Workflow** | Template defining how tasks should execute |
| **Case** | Instance of a workflow (one specific order) |
| **Task** | Individual step in the workflow |
| **Pattern** | Reusable structure (sequence, parallel, split, join) |
| **Guard** | Condition that must be true for task to execute |
| **Data** | Information flowing through the workflow |

### Workflow Specification Format

KNHK uses **Turtle/RDF** format to define workflows. This is a W3C standard for semantic data.

**Simple Example**:
```turtle
@prefix workflow: <http://example.org/workflow/> .
@prefix yawl: <http://example.org/yawl/> .

workflow:SimpleOrder
  a workflow:Process ;
  workflow:hasTask workflow:ReceiveOrder ;
  workflow:hasTask workflow:ProcessOrder ;
  workflow:hasTask workflow:Confirm .

workflow:ReceiveOrder a workflow:Task .
workflow:ProcessOrder a workflow:Task .
workflow:Confirm a workflow:Task .
```

---

## Create Your First Workflow

Let's create a simple **Expense Approval Workflow**:

1. **Submit Expense** - Employee submits expense report
2. **Manager Review** - Manager reviews and approves/rejects
3. **Finance Processing** - Finance processes approved expense
4. **Send Confirmation** - Confirm to employee

### Step 1: Create Workflow File

Create a file: `expense-approval.ttl`

```turtle
@prefix : <http://example.org/workflow/> .
@prefix workflow: <http://example.org/workflow/> .
@prefix yawl: <http://example.org/yawl/> .

# Define the workflow process
:ExpenseApprovalWorkflow
  a workflow:Process ;
  workflow:name "Expense Approval" ;
  workflow:description "Simple expense approval workflow" ;
  workflow:entryTask :SubmitExpense ;
  workflow:exitTask :SendConfirmation .

# Task 1: Submit Expense
:SubmitExpense
  a workflow:Task ;
  workflow:taskName "Submit Expense" ;
  workflow:nextTask :ManagerReview ;
  workflow:displayName "Employee submits expense report" .

# Task 2: Manager Review
:ManagerReview
  a workflow:Task ;
  workflow:taskName "Manager Review" ;
  workflow:displayName "Manager reviews and approves" ;
  workflow:hasGuard [
    workflow:guardType "dataCondition" ;
    workflow:guardValue "amount < 5000 OR manager_approved = true"
  ] ;
  workflow:nextTask :ProcessingDecision .

# Task 3: Processing Decision (using split pattern)
:ProcessingDecision
  a workflow:SplitTask ;
  workflow:splitType "OR" ;
  workflow:trueBranch :FinanceProcessing ;
  workflow:falseBranch :Rejection .

# Task 4a: Finance Processing (if approved)
:FinanceProcessing
  a workflow:Task ;
  workflow:taskName "Finance Processing" ;
  workflow:displayName "Finance processes the approved expense" ;
  workflow:nextTask :SendConfirmation .

# Task 4b: Rejection (if not approved)
:Rejection
  a workflow:Task ;
  workflow:taskName "Rejection" ;
  workflow:displayName "Notify employee of rejection" ;
  workflow:nextTask :SendConfirmation .

# Task 5: Send Confirmation
:SendConfirmation
  a workflow:Task ;
  workflow:taskName "Send Confirmation" ;
  workflow:displayName "Send final confirmation email" ;
  workflow:isExitTask true .
```

### Step 2: Validate the Workflow

Before registering, validate it:

```bash
# Create a validation request
curl -X POST http://localhost:8080/api/v1/workflows/validate \
  -H "Content-Type: application/json" \
  -d '{
    "spec": "'"$(cat expense-approval.ttl)"'",
    "check_deadlock": true,
    "check_soundness": true
  }' | jq .

# Output should include:
# "valid": true
# "checks": {
#   "syntax": "passed",
#   "deadlock": "no_deadlock_detected",
#   "soundness": "sound"
# }
```

If you see errors, review the Turtle syntax and try again.

---

## Register the Workflow

### Step 1: Load Workflow File

Create a script `register-workflow.sh`:

```bash
#!/bin/bash

# Read the workflow file
WORKFLOW_SPEC=$(cat expense-approval.ttl)

# Register with the engine
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "expense-approval",
    "description": "Simple expense approval workflow",
    "spec": "'"$WORKFLOW_SPEC"'",
    "tags": ["approval", "expense"]
  }' | jq .
```

### Step 2: Run Registration

```bash
chmod +x register-workflow.sh
./register-workflow.sh

# Output:
# {
#   "id": "wf_7kJ9nM2xQ8pR",
#   "name": "expense-approval",
#   "version": "1.0.0",
#   "created_at": "2025-11-17T12:00:00Z",
#   "patterns": 5
# }
```

**Save the workflow ID** - you'll need it for next steps. Let's call it `WF_ID`.

### Step 3: Verify Registration

```bash
# Get workflow details
curl http://localhost:8080/api/v1/workflows/wf_7kJ9nM2xQ8pR | jq .

# List all workflows
curl http://localhost:8080/api/v1/workflows | jq .
```

---

## Execute and Monitor

### Step 1: Create a Case Instance

A **case** is one instance of the workflow (one specific expense report).

```bash
curl -X POST http://localhost:8080/api/v1/workflows/wf_7kJ9nM2xQ8pR/cases \
  -H "Content-Type: application/json" \
  -d '{
    "case_id": "expense_001",
    "data": {
      "employee_id": "emp_123",
      "amount": 450.00,
      "description": "Client meeting travel",
      "date_submitted": "2025-11-17T12:00:00Z"
    }
  }' | jq .

# Output:
# {
#   "case_id": "expense_001",
#   "workflow_id": "wf_7kJ9nM2xQ8pR",
#   "state": "active",
#   "enabled_tasks": ["SubmitExpense"],
#   "created_at": "2025-11-17T12:00:00Z"
# }
```

**Save the case ID** - `expense_001` in this example.

### Step 2: Check Case Status

```bash
# Get current status
curl http://localhost:8080/api/v1/cases/expense_001 | jq .

# Output shows:
# - Current state: "active"
# - Enabled tasks: which tasks can run now
# - Progress: completed vs. total tasks
# - Data: case information
```

### Step 3: Complete Tasks

Tasks execute in sequence. Complete each enabled task:

```bash
# Task 1: Submit Expense
curl -X POST http://localhost:8080/api/v1/cases/expense_001/tasks/SubmitExpense/complete \
  -H "Content-Type: application/json" \
  -d '{
    "result": "submitted",
    "output_data": {
      "submitted_by": "emp_123",
      "submitted_at": "2025-11-17T12:05:00Z"
    }
  }' | jq .

# Check case status again
curl http://localhost:8080/api/v1/cases/expense_001 | jq .
# Now enabled_tasks should show ["ManagerReview"]
```

### Step 4: Manager Review

```bash
# Task 2: Manager Reviews
curl -X POST http://localhost:8080/api/v1/cases/expense_001/tasks/ManagerReview/complete \
  -H "Content-Type: application/json" \
  -d '{
    "result": "approved",
    "output_data": {
      "manager_id": "mgr_456",
      "manager_approved": true,
      "reviewed_at": "2025-11-17T12:15:00Z"
    }
  }' | jq .
```

### Step 5: Complete Finance Processing

```bash
# Task 3: Finance Processing
curl -X POST http://localhost:8080/api/v1/cases/expense_001/tasks/FinanceProcessing/complete \
  -H "Content-Type: application/json" \
  -d '{
    "result": "processed",
    "output_data": {
      "transaction_id": "txn_789",
      "processed_at": "2025-11-17T12:20:00Z"
    }
  }' | jq .
```

### Step 6: Send Confirmation

```bash
# Task 4: Send Confirmation
curl -X POST http://localhost:8080/api/v1/cases/expense_001/tasks/SendConfirmation/complete \
  -H "Content-Type: application/json" \
  -d '{
    "result": "confirmed",
    "output_data": {
      "confirmation_email": "emp_123@example.com",
      "sent_at": "2025-11-17T12:25:00Z"
    }
  }' | jq .

# Check final case status
curl http://localhost:8080/api/v1/cases/expense_001 | jq .
# State should be: "completed"
```

---

## Monitor Execution

### View Execution History

See all events that occurred during case execution:

```bash
curl http://localhost:8080/api/v1/cases/expense_001/history | jq .

# Output shows timeline of all events:
# [
#   {
#     "timestamp": "2025-11-17T12:00:00Z",
#     "type": "case_created",
#     "task": null
#   },
#   {
#     "timestamp": "2025-11-17T12:05:00Z",
#     "type": "task_completed",
#     "task": "SubmitExpense",
#     "duration_ms": 5000
#   },
#   ...
# ]
```

### View Workflow Metrics

```bash
curl http://localhost:8080/api/v1/workflows/wf_7kJ9nM2xQ8pR/metrics | jq .

# Output:
# {
#   "cases_created": 1,
#   "cases_completed": 1,
#   "avg_duration_ms": 1500,
#   "p95_duration_ms": 2000
# }
```

---

## Troubleshoot Common Issues

### Issue: Workflow Won't Register

**Problem**: Validation fails with "syntax error"

**Solution**:
1. Validate Turtle syntax: `rapper -i turtle -o turtle expense-approval.ttl`
2. Check for missing prefixes
3. Review error line number
4. Compare with examples in documentation

### Issue: Task Won't Complete

**Problem**: Getting "guard violation" error

**Solution**:
1. Check guard condition in workflow definition
2. View case data: `curl http://localhost:8080/api/v1/cases/expense_001`
3. Ensure output data satisfies guard
4. Send required data in complete request

### Issue: Case Gets Stuck

**Problem**: Case state is "active" but no tasks enabled

**Solution**:
1. Check execution history for errors
2. Look for deadlock in workflow definition
3. View case status: `curl http://localhost:8080/api/v1/cases/expense_001`
4. Review logs: `tail -f logs/knhk.log`

### Issue: Engine Not Responding

**Problem**: `curl: (7) Failed to connect`

**Solution**:
1. Verify engine is running: `ps aux | grep knhk`
2. Check port is correct: `netstat -tlnp | grep 8080`
3. Restart: `Ctrl+C` in engine terminal, then `cargo run --release`

---

## Next Steps

### 1. Try More Complex Workflow

Create a workflow with:
- **Parallel tasks** (multiple tasks running simultaneously)
- **Conditional branches** (different paths based on data)
- **Loops** (tasks repeating)

See example: `examples/execute_workflow.rs`

### 2. Add Observability

Monitor your workflow with OpenTelemetry:
- See [How-To: OTEL Observability](../how-to/otel-observability.md)

### 3. Deploy to Kubernetes

Run your workflow in production:
- See [How-To: Kubernetes Deployment](../how-to/kubernetes-deployment.md)

### 4. Integrate with External Systems

Connect your workflow to APIs and databases:
- See [How-To: Custom Hooks](../how-to/custom-hooks.md)

---

## Related Documentation

- [API Endpoints Reference](../reference/api-endpoints.md) - Complete API guide
- [Configuration Guide](../reference/configuration.md) - Fine-tune the engine
- [Error Codes Reference](../reference/error-codes.md) - Understand errors
- [How-To: Troubleshooting](../how-to/troubleshooting.md) - Solve problems
- [Architecture Explanation](../explanations/sigma-pi-mu-mape-k.md) - How it works

---

## Summary

You've successfully:
- ✅ Installed and started the KNHK Workflow Engine
- ✅ Created a workflow in Turtle/RDF format
- ✅ Registered the workflow with the engine
- ✅ Created a case instance
- ✅ Executed tasks and completed the workflow
- ✅ Monitored execution with history and metrics

**Congratulations!** You now understand the basics of KNHK workflows. 🎉

Next time, try creating a more complex workflow with parallel tasks or conditional logic. The examples in the `examples/` directory provide templates for more advanced scenarios.
