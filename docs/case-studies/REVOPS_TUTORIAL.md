# Tutorial: Running the RevOps Pipeline - A Complete Walkthrough

**Time to Complete**: 2 hours (walkthrough) + 30 min (hands-on)
**Difficulty**: Intermediate
**Prerequisites**: KNHK engine running, basic workflow understanding
**Goal**: Understand how to operationalize the RevOps pipeline end-to-end

---

## Part 1: Understanding the RevOps Scenario (30 minutes)

### The Business Context

Acme Enterprise Software processes 2,000 deals per month worth $500M+ in bookings. Each deal must go through:

1. **Lead Qualification**: Is this a real opportunity?
2. **Deal Approval**: Do managers, legal, finance agree?
3. **Contract Processing**: Is the contract signed?
4. **Pricing Exception**: Is the discount justified?
5. **Revenue Recognition**: Book the deal, set up billing

**Without automation**: 19-35 days average
**With automation**: 2-5 days average

### Key Innovation: Parallel Approvals

Traditional approach (sequential):
```
Manager Review (2-4 days) →
Legal Review (2-3 days) →
Finance Check (1-2 days) →
Result: 5-9 days
```

Automated approach (parallel):
```
Manager Review ─┬─ (concurrent)
Legal Review   ─┤ (concurrent)
Finance Check  ─┤ (concurrent)
Result: 1-2 hours
```

### The Five Workflows

| WF# | Name | Pattern | Key Decision | Typical Time |
|-----|------|---------|--------------|--------------|
| 1 | Lead Qualification | OR-split | Auto/Manual/Archive | 15-30 min |
| 2 | Deal Approval Gate | AND-split + Sync | Parallel approvals | 1-8 hours |
| 3 | Contract Processing | Advanced branching | Standard/Custom routes | 24-96 hours |
| 4 | Pricing Exception | OR-join + Loop | Justify discount | 2-24 hours |
| 5 | Revenue Recognition | Deferred choice | Invoice vs. Prepay | Immediate-60 days |

---

## Part 2: Setting Up the Pipeline (30 minutes)

### Step 1: Register All Five Workflows

In KNHK, register each workflow:

```bash
# Download workflow definitions
curl -s https://github.com/.../revops-workflows.tar.gz | tar xz

# Register WF1: Lead Qualification
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d @workflows/lead-qualification.json

# Register WF2: Deal Approval Gate
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d @workflows/deal-approval-gate.json

# Register WF3: Contract Processing
curl -X POST http://localhost:8080/api/v1/workflows \
  -d @workflows/contract-processing.json

# Register WF4: Pricing Exception
curl -X POST http://localhost:8080/api/v1/workflows \
  -d @workflows/pricing-exception.json

# Register WF5: Revenue Recognition
curl -X POST http://localhost:8080/api/v1/workflows \
  -d @workflows/revenue-recognition.json

# Verify all registered
curl http://localhost:8080/api/v1/workflows | jq '.workflows | length'
# Output: 5
```

### Step 2: Configure Roles & Approvers

Create configuration for each persona:

```yaml
# config/revops-roles.yaml

roles:
  sdr:
    display_name: "Sales Development Rep"
    permissions:
      - submit_leads
      - submit_deals
      - comment_on_deals

  sales_manager:
    display_name: "Sales Manager"
    permissions:
      - approve_deals_up_to: 250000
      - comment_on_deals
      - escalate_to_vp_sales
    sla_hours: 24

  legal_counsel:
    display_name: "Legal Counsel"
    permissions:
      - review_contracts
      - request_amendments
      - approve_contracts_up_to: 5000000
    sla_hours: 24

  finance_manager:
    display_name: "Finance Manager"
    permissions:
      - review_deal_economics
      - approve_discounts_up_to: 0.15
      - approve_exceptions_up_to: 100000
    sla_hours: 12

  cfo:
    display_name: "Chief Financial Officer"
    permissions:
      - approve_all_deals
      - approve_all_exceptions
      - approve_pricing_strategies
    sla_hours: 2

users:
  sarah.chen@acme.com:
    role: sdr
    team: "West Region"

  marcus.thompson@acme.com:
    role: sales_manager
    team: "West Region"
    reports: [sarah.chen@acme.com, david.park@acme.com]

  priya.patel@acme.com:
    role: legal_counsel
    team: "Legal"

  james.rodriguez@acme.com:
    role: finance_manager
    team: "Finance - Deals"

  lisa.wong@acme.com:
    role: cfo
    team: "Executive"
```

### Step 3: Configure SLOs & Monitoring

```yaml
# config/revops-slos.yaml

slos:
  lead_qualification_time:
    target_hours: 1
    alert_threshold: 4

  manager_approval_time:
    target_hours: 4
    alert_threshold: 24

  legal_review_time:
    target_hours: 4
    alert_threshold: 24

  finance_review_time:
    target_hours: 2
    alert_threshold: 12

  executive_approval_time:
    target_hours: 2
    alert_threshold: 2

  total_deal_cycle_time:
    target_days: 3
    alert_threshold: 14

notifications:
  missed_slo:
    channels: [slack, email]
    recipients:
      - role: sales_manager
      - role: cfo
```

---

## Part 3: Creating a Test Deal (30 minutes)

### Scenario: TechCorp Deal

Let's walk through the complete TechCorp deal step-by-step.

### Step 1: Lead Creation

```bash
# Create a lead in Salesforce/CRM
# Then submit to KNHK

curl -X POST http://localhost:8080/api/v1/workflows/lead-qualification/cases \
  -H "Content-Type: application/json" \
  -d '{
    "case_id": "lead_techcorp_001",
    "data": {
      "company_name": "TechCorp",
      "contact_name": "John Smith",
      "contact_email": "john.smith@techcorp.com",
      "company_size": 5000,
      "industry": "IT Consulting",
      "location": "San Francisco, CA",
      "use_case": "Enterprise Analytics"
    }
  }'

# Response:
# {
#   "case_id": "lead_techcorp_001",
#   "workflow_id": "wf_lead_qualification",
#   "state": "active",
#   "enabled_tasks": ["ReceiveMarketingLead"],
#   "created_at": "2025-11-17T09:00:00Z"
# }
```

### Step 2: Complete Qualification Tasks

```bash
# Task 1: Receive Marketing Lead
curl -X POST http://localhost:8080/api/v1/cases/lead_techcorp_001/tasks/ReceiveMarketingLead/complete \
  -d '{"result": "received"}'

# Task 2: Validate Contact Info
curl -X POST http://localhost:8080/api/v1/cases/lead_techcorp_001/tasks/ValidateContactInfo/complete \
  -d '{
    "result": "valid",
    "output_data": {
      "email_valid": true,
      "phone_valid": true,
      "company_verified": true
    }
  }'

# Task 3: Scoring Decision (auto-calculated)
# System calculates: 72 point score
# Guard: score (72) > 65? → YES
# Routes to: AssignToSDR

curl http://localhost:8080/api/v1/cases/lead_techcorp_001 | jq '.enabled_tasks'
# Output: ["AssignToSDR"]

# Task 4: AssignToSDR (auto-execution)
# System loads all SDRs, finds least loaded
# Assigns to: Emily Garcia
# WF1 completes successfully

curl http://localhost:8080/api/v1/cases/lead_techcorp_001 | jq '.state'
# Output: "completed"
```

### Step 3: Create Deal & Submit to WF2

```bash
# Sarah Chen creates deal record in Salesforce
# Automatically triggers WF2

curl -X POST http://localhost:8080/api/v1/workflows/deal-approval-gate/cases \
  -H "Content-Type: application/json" \
  -d '{
    "case_id": "deal_techcorp_001",
    "data": {
      "deal_name": "TechCorp Enterprise Analytics",
      "customer_name": "TechCorp",
      "customer_id": "cust_5893",
      "deal_acv": 600000,
      "deal_value_3yr": 1800000,
      "discount_percent": 0,
      "margin_percent": 0.80,
      "sdr_name": "Sarah Chen",
      "manager_name": "Marcus Thompson"
    }
  }'

# Response starts WF2 with 4 parallel approval paths
# {
#   "case_id": "deal_techcorp_001",
#   "workflow_id": "wf_deal_approval_gate",
#   "state": "active",
#   "enabled_tasks": [
#     "SendToSalesManager",
#     "SendToLegal",
#     "SendToFinance",
#     "ExecutiveCheck"
#   ],
#   "created_at": "2025-11-17T10:05:00Z"
# }
```

### Step 4: Manager Approval

```bash
# Marcus Thompson receives notification
# Reviews deal in KNHK UI or via API

curl http://localhost:8080/api/v1/cases/deal_techcorp_001 | jq '.data'

# After review, approves:
curl -X POST http://localhost:8080/api/v1/cases/deal_techcorp_001/tasks/SendToSalesManager/complete \
  -d '{
    "result": "approved",
    "output_data": {
      "manager_approval": "APPROVED",
      "manager_comment": "Great fit. TechCorp is in our target market."
    }
  }'
```

### Step 5: Parallel Approvals Complete

```bash
# Legal approval
curl -X POST http://localhost:8080/api/v1/cases/deal_techcorp_001/tasks/SendToLegal/complete \
  -d '{
    "result": "approved",
    "output_data": {
      "legal_approval": "APPROVED",
      "legal_changes_requested": false
    }
  }'

# Finance approval
curl -X POST http://localhost:8080/api/v1/cases/deal_techcorp_001/tasks/SendToFinance/complete \
  -d '{
    "result": "approved",
    "output_data": {
      "finance_approval": "APPROVED",
      "discount_exception": false
    }
  }'

# Executive approval (auto-triggered for ACV > 250K)
curl -X POST http://localhost:8080/api/v1/cases/deal_techcorp_001/tasks/ExecutiveCheck/complete \
  -d '{
    "result": "approved",
    "output_data": {
      "executive_approval": "APPROVED",
      "conditions": "none"
    }
  }'

# Check: All approvals complete?
curl http://localhost:8080/api/v1/cases/deal_techcorp_001 | jq '.enabled_tasks'
# Output: ["AllApprovalsComplete"]

# WF2 completes, transitions to WF3
```

### Step 6: Continue Through WF3-WF5

The pipeline automatically transitions through:
- WF3: Contract prepared, signed by customer (automated e-signature)
- WF4: Skipped (no pricing exception)
- WF5: Invoice sent, payment received, revenue booked

```bash
# Check final status
curl http://localhost:8080/api/v1/cases/deal_techcorp_001/history | jq '.events | length'
# Shows all events: CaseCreated, TaskEnabled, TaskCompleted, etc.
```

---

## Part 4: Monitoring & Dashboards (30 minutes)

### Check SLO Compliance

```bash
# Get SLO status
curl http://localhost:8080/api/v1/workflows/deal-approval-gate/metrics?time_range=24h | jq .

# Response:
# {
#   "workflow_id": "deal-approval-gate",
#   "time_range": "24h",
#   "metrics": {
#     "cases_created": 47,
#     "cases_completed": 45,
#     "cases_failed": 0,
#     "avg_duration_hours": 2.3,
#     "p95_duration_hours": 8.1,
#     "slo_target_hours": 8,
#     "slo_compliance_percent": 98.3
#   }
# }
```

### View MAPE-K Recommendations

```bash
# See autonomous improvements
curl http://localhost:8080/api/v1/workflows/deal-approval-gate/mape-k/recommendations | jq .

# Example response:
# {
#   "recommendations": [
#     {
#       "id": "rec_1",
#       "type": "timing_optimization",
#       "description": "Send e-signatures at 2pm PT for 2h faster response",
#       "confidence": 0.92,
#       "estimated_improvement": "2 hours saved per deal",
#       "status": "ready_to_apply"
#     }
#   ]
# }
```

---

## Part 5: Hands-On Exercise (You Do It)

Now try it yourself:

1. **Create a deal**: Submit a deal for a fictional company
2. **Drive approvals**: Simulate manager, legal, finance approvals
3. **Track metrics**: Check SLO compliance
4. **View results**: See the complete audit trail

**Time**: 30 minutes

**Success criteria**:
- [ ] Deal submitted to WF2
- [ ] All 4 parallel approvals completed
- [ ] Deal transitioned to WF3
- [ ] SLO compliance tracked
- [ ] MAPE-K found at least 1 improvement recommendation

---

## Conclusion

You've now walked through:
1. The business scenario (RevOps pipeline)
2. Setting up the 5 workflows
3. Creating and completing a deal
4. Monitoring SLOs and metrics
5. Seeing MAPE-K in action

**Next steps**:
- See [How-To: Extend RevOps Pipeline](./REVOPS_HOWTO.md) to customize for your company
- See [Reference: Deal Data Structure](./REVOPS_REFERENCE.md) for complete field specifications
- See [Explanation: Why This Architecture Works](./REVOPS_EXPLANATION.md) for the design rationale
