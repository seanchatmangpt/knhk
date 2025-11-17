# Reference: RevOps Pipeline Data Structures and Decision Tables

**Complete Field Specifications, Data Schemas, and Decision Logic Reference**

---

## Table of Contents

1. [Deal Data Structure](#deal-data-structure)
2. [Workflow Input/Output Schemas](#workflow-inputoutput-schemas)
3. [Decision Tables](#decision-tables)
4. [Approval Authority Matrix](#approval-authority-matrix)
5. [SLO Specifications](#slo-specifications)
6. [Error Codes](#error-codes)

---

## Deal Data Structure

### Core Deal Object

```json
{
  "deal_id": "deal_techcorp_001",
  "deal_name": "TechCorp Enterprise Analytics",
  "deal_status": "active",
  "created_at": "2025-11-17T09:00:00Z",
  "updated_at": "2025-11-17T10:05:00Z",

  "customer": {
    "customer_id": "cust_5893",
    "customer_name": "TechCorp",
    "industry": "IT Consulting",
    "company_size": 5000,
    "location": "San Francisco, CA",
    "existing_customer": false,
    "customer_health_score": 92,
    "segment": "Enterprise"
  },

  "deal_financials": {
    "deal_acv": 600000,
    "deal_value_3yr": 1800000,
    "discount_percent": 0.0,
    "margin_percent": 0.80,
    "payment_terms": "net_30",
    "currency": "USD",
    "approval_required_acv_threshold": 250000
  },

  "deal_metadata": {
    "use_case": "Enterprise Analytics",
    "vertical": "consulting",
    "deal_type": "new",
    "is_expansion": false,
    "expansion_percentage": null,
    "is_renewal": false,
    "renewal_of": null,
    "is_replacement": false,
    "competitor": null
  },

  "sales_info": {
    "sdr_name": "Sarah Chen",
    "sdr_email": "sarah.chen@acme.com",
    "manager_name": "Marcus Thompson",
    "manager_email": "marcus.thompson@acme.com",
    "deal_source": "inbound",
    "deal_source_campaign": "Q4_Enterprise_Outreach"
  },

  "timeline": {
    "target_close_date": "2025-12-15",
    "expected_signature_date": "2025-11-25",
    "expected_start_date": "2025-12-01"
  },

  "workflow_state": {
    "current_workflow": "deal_approval_gate",
    "current_stage": "approval_in_progress",
    "completed_workflows": ["lead_qualification"],
    "pending_workflows": ["contract_processing", "pricing_exception", "revenue_recognition"]
  }
}
```

### Field Reference Table

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `deal_id` | string | Yes | Unique deal identifier | `deal_techcorp_001` |
| `deal_name` | string | Yes | Human-readable deal name | `TechCorp Enterprise Analytics` |
| `customer_id` | string | Yes | Customer identifier | `cust_5893` |
| `customer_name` | string | Yes | Customer company name | `TechCorp` |
| `deal_acv` | number | Yes | Annual contract value in USD | `600000` |
| `deal_value_3yr` | number | Yes | 3-year total value | `1800000` |
| `discount_percent` | number | No | Discount (0-1) | `0.0` or `0.15` |
| `margin_percent` | number | No | Gross margin (0-1) | `0.80` |
| `deal_type` | enum | Yes | Values: `new`, `expansion`, `renewal` | `new` |
| `industry` | string | No | Customer industry | `IT Consulting` |
| `company_size` | number | No | Number of employees | `5000` |
| `sdr_name` | string | Yes | Sales dev rep name | `Sarah Chen` |
| `manager_name` | string | Yes | Sales manager name | `Marcus Thompson` |
| `target_close_date` | ISO8601 | Yes | Expected close date | `2025-12-15` |

---

## Workflow Input/Output Schemas

### Workflow 1: Lead Qualification

**Input Schema**:
```json
{
  "case_id": "lead_techcorp_001",
  "workflow_id": "wf_lead_qualification",
  "data": {
    "company_name": "TechCorp",
    "contact_name": "John Smith",
    "contact_email": "john.smith@techcorp.com",
    "company_size": 5000,
    "industry": "IT Consulting",
    "location": "San Francisco, CA",
    "use_case": "Enterprise Analytics"
  }
}
```

**Output Schema**:
```json
{
  "case_id": "lead_techcorp_001",
  "workflow_id": "wf_lead_qualification",
  "state": "completed",
  "completion_time_minutes": 22,
  "result": {
    "qualified": true,
    "qualification_score": 72,
    "qualification_level": "high",
    "assigned_to": "sarah.chen@acme.com",
    "assigned_to_name": "Sarah Chen",
    "notes": "Strong fit. Enterprise buyer, clear use case.",
    "routing_decision": "assign_to_sdr"
  }
}
```

### Workflow 2: Deal Approval Gate

**Input Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_deal_approval_gate",
  "data": {
    "deal_name": "TechCorp Enterprise Analytics",
    "customer_name": "TechCorp",
    "customer_id": "cust_5893",
    "deal_acv": 600000,
    "deal_value_3yr": 1800000,
    "discount_percent": 0.0,
    "margin_percent": 0.80,
    "sdr_name": "Sarah Chen",
    "manager_name": "Marcus Thompson"
  }
}
```

**Output Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_deal_approval_gate",
  "state": "completed",
  "completion_time_minutes": 145,  // 2h 25m
  "approvals": {
    "manager_approval": {
      "result": "approved",
      "approver": "marcus.thompson@acme.com",
      "approved_at": "2025-11-17T10:30:00Z",
      "comment": "Great fit. TechCorp is in our target market."
    },
    "legal_approval": {
      "result": "approved",
      "approver": "priya.patel@acme.com",
      "approved_at": "2025-11-17T10:40:00Z",
      "changes_requested": false
    },
    "finance_approval": {
      "result": "approved",
      "approver": "james.rodriguez@acme.com",
      "approved_at": "2025-11-17T10:50:00Z",
      "discount_exception": false
    },
    "executive_approval": {
      "result": "approved",
      "approver": "lisa.wong@acme.com",
      "approved_at": "2025-11-17T11:25:00Z",
      "conditions": "none"
    }
  },
  "all_approvals_received": true
}
```

### Workflow 3: Contract Processing

**Input Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_contract_processing",
  "data": {
    "customer_id": "cust_5893",
    "customer_name": "TechCorp",
    "deal_id": "deal_techcorp_001",
    "contract_type": "standard",  // standard | custom | master_service_agreement
    "deal_acv": 600000,
    "deal_terms": "net_30",
    "special_provisions": []
  }
}
```

**Output Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_contract_processing",
  "state": "completed",
  "completion_time_hours": 9,
  "result": {
    "contract_id": "contract_5893_001",
    "contract_type": "standard",
    "signature_status": "signed_by_customer",
    "signed_by_customer_at": "2025-11-17T18:30:00Z",
    "signed_by_acme_at": "2025-11-17T08:00:00Z",
    "contract_start_date": "2025-12-01",
    "contract_end_date": "2026-11-30"
  }
}
```

### Workflow 4: Pricing Exception

**Input Schema** (only if discount > 0):
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_pricing_exception",
  "data": {
    "deal_acv": 600000,
    "requested_discount": 0.0,  // 0-1 range
    "discount_reason": "none",
    "competitor_name": null,
    "finance_justification": ""
  }
}
```

**Output Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_pricing_exception",
  "state": "skipped",  // Skipped because discount_percent == 0
  "reason": "No pricing exception required"
}
```

### Workflow 5: Revenue Recognition

**Input Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_revenue_recognition",
  "data": {
    "deal_id": "deal_techcorp_001",
    "deal_acv": 600000,
    "contract_start_date": "2025-12-01",
    "contract_id": "contract_5893_001",
    "payment_terms": "net_30",
    "payment_method": "invoice"  // invoice | prepay | installment
  }
}
```

**Output Schema**:
```json
{
  "case_id": "deal_techcorp_001",
  "workflow_id": "wf_revenue_recognition",
  "state": "completed",
  "completion_time_hours": 32,
  "result": {
    "revenue_recognized": true,
    "recognition_date": "2025-11-19T14:50:00Z",
    "revenue_amount": 600000,
    "revenue_recognition_method": "date_of_service_commencement",
    "invoice_id": "inv_techcorp_001",
    "invoice_sent_date": "2025-11-19",
    "payment_due_date": "2025-12-19",
    "billing_setup_complete": true
  }
}
```

---

## Decision Tables

### Decision Table 1: Lead Qualification Score

| Company Size | Industry | Use Case Clarity | Budget Identified | Score Range | Routing |
|--------------|----------|-----------------|-------------------|------------|---------|
| <100 | Any | No | No | 0-30 | Archive |
| 100-500 | Non-target | No | No | 31-40 | Manual Review |
| 100-500 | Non-target | Yes | No | 41-50 | Manual Review |
| 100-500 | Non-target | Yes | Yes | 51-65 | Manual Review |
| 100-500 | Target | Yes | Yes | 66-75 | Assign to SDR |
| 500+ | Target | Yes | Yes | 76-100 | Assign to SDR |
| 5000+ | Any | Any | Any | 70+ | Assign to SDR |

### Decision Table 2: Approval Path by Deal Size

| Deal ACV | Manager? | Legal? | Finance? | Executive? | Est. Hours |
|----------|----------|--------|----------|------------|------------|
| $0-50K | Yes | No | Yes | No | 2 |
| $50K-250K | Yes | Yes | Yes | No | 4 |
| $250K-1M | Yes | Yes | Yes | Yes | 6 |
| $1M+ | Yes | Yes | Yes | Yes | 8 |

### Decision Table 3: Contract Processing by Type

| Contract Type | Legal Review Needed? | E-Signature? | Est. Hours | Special Handling |
|---------------|--------------------|--------------|-----------|--------------------|
| Standard | No | Automated | 2 | None |
| Custom | Yes | Manual | 24-48 | Legal exception handling |
| Master Service Agreement | Yes | Manual | 48-72 | Executive review, amendments |
| International | Yes | Manual with local counsel | 72+ | Jurisdiction-specific |

### Decision Table 4: Discount Authority

| Discount % | Manager | Director | CFO | Board | Max Authority |
|-----------|---------|----------|-----|-------|----------------|
| 0-5% | ✓ | - | - | - | Manager |
| 5-10% | - | ✓ | - | - | Director |
| 10-15% | - | - | ✓ | - | CFO |
| 15-25% | - | - | - | ✓ | Board |
| 25%+ | - | - | - | - | CEO only |

### Decision Table 5: Revenue Recognition Timing

| Payment Method | Contract Start | Recognition Timing | Invoice Timing |
|--------------|-----------------|-------------------|-----------------|
| Invoice (Net 30) | Date of service | Date of service commencement | Immediately at start |
| Prepaid | Date of service | Date of prepayment (full) | At time of payment |
| Installment | Date of service | Per installment schedule | Monthly with schedule |
| Deferred | Date of service | Per milestone | Per milestone |

---

## Approval Authority Matrix

### By Role and Deal Characteristic

```yaml
Sales Development Rep (SDR):
  - Submit leads: ✓
  - Submit deals: ✓
  - Approve anything: ✗
  - SLA: 24 hours to qualify lead

Sales Manager:
  - Approve deals up to: $250,000 ACV
  - Discount authority: 0-5%
  - Override SDR qualification: ✓
  - SLA: 24 hours

Director of Sales:
  - Approve deals up to: $1,000,000 ACV
  - Discount authority: 5-10%
  - Escalate to CFO: ✓
  - SLA: 4 hours

Legal Counsel:
  - Review all contracts: ✓
  - Approve standard contracts: ✓
  - Escalate to external counsel: ✓
  - Request amendments: ✓
  - SLA: 24 hours

Finance Manager:
  - Review deal economics: ✓
  - Approve discounts: 0-15%
  - Approve exceptions: ✓ (up to $100K)
  - SLA: 12 hours

CFO:
  - Approve anything: ✓
  - Discount authority: 0-100%
  - Strategic decisions: ✓
  - SLA: 2 hours
```

---

## SLO Specifications

### Lead Qualification SLO

```yaml
metric: "lead_qualification_time"
target: "1 hour"
alert_threshold: "4 hours"
measurement:
  start_event: "lead_created"
  end_event: "lead_routed_to_sdr"
  excludes: ["weekends", "holidays"]
compliance_target: "95%"
consequences:
  breach_1x_per_week: "Review SDR workload"
  breach_2x_per_week: "Investigate root cause"
  breach_5x_per_week: "Escalate to VP Sales"
```

### Manager Approval SLO

```yaml
metric: "manager_approval_time"
target: "4 hours"
alert_threshold: "24 hours"
measurement:
  start_event: "deal_sent_to_manager"
  end_event: "manager_approved_or_rejected"
  excludes: ["nights", "weekends", "holidays"]
compliance_target: "98%"
by_deal_size:
  "<$100K": "2 hours"
  "$100K-$500K": "4 hours"
  ">$500K": "8 hours"
```

### Legal Review SLO

```yaml
metric: "legal_review_time"
target: "4 hours for standard, 24 hours for custom"
alert_threshold: "24 hours / 72 hours"
measurement:
  start_event: "contract_sent_to_legal"
  end_event: "legal_approved_or_requested_changes"
compliance_target: "95%"
by_contract_type:
  standard: "4 hours"
  custom: "24 hours"
  msa: "72 hours"
```

### Finance Review SLO

```yaml
metric: "finance_review_time"
target: "2 hours"
alert_threshold: "12 hours"
measurement:
  start_event: "deal_sent_to_finance"
  end_event: "finance_approved_or_requested_exception"
compliance_target: "98%"
```

### Total Deal Cycle SLO

```yaml
metric: "total_deal_cycle_time"
target: "3 days"
alert_threshold: "14 days"
measurement:
  start_event: "lead_created"
  end_event: "revenue_recognized"
  excludes: ["weekends", "holidays", "customer_delays"]
compliance_target: "90%"
by_deal_type:
  new: "3-5 days"
  renewal: "1-2 days"
  expansion: "2-3 days"
```

---

## Error Codes

### Deal Creation Errors (1000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 1001 | "Invalid customer_id" | Customer doesn't exist | Create customer record first |
| 1002 | "Missing required field: deal_acv" | ACV not provided | Provide deal annual contract value |
| 1003 | "Deal ACV must be > 0" | ACV is zero or negative | Correct the deal amount |
| 1004 | "Customer has active deal" | Duplicate deal restriction | Verify customer needs multiple deals |
| 1005 | "Customer industry not found" | Invalid industry field | Use approved industry codes |

### Workflow State Errors (2000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 2001 | "Workflow not found" | Referenced workflow doesn't exist | Verify workflow ID spelling |
| 2002 | "Case already in progress" | Case already running | Wait for completion or cancel |
| 2003 | "Invalid transition: ACTIVE → COMPLETED" | Can't skip required steps | Complete all required tasks |
| 2004 | "Guard condition failed" | Data doesn't satisfy guard | Check input data against guard conditions |

### Approval Errors (3000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 3001 | "Approver not found" | Assigned approver doesn't exist | Update approver assignment |
| 3002 | "Insufficient authority" | Approver lacks permission | Escalate to higher authority |
| 3003 | "SLA breach: approval overdue" | Approval exceeded timeout | Escalate or reassign |
| 3004 | "Approval already submitted" | Can't re-approve | Contact workflow administrator |

### Pricing Errors (4000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 4001 | "Discount exceeds authority" | Approver can't approve discount | Escalate to CFO |
| 4002 | "Discount would create loss" | Discount below cost basis | Reduce discount or escalate |
| 4003 | "Pricing exception denied" | Exception request rejected | Accept standard pricing or negotiate |

### Contract Errors (5000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 5001 | "Contract signature missing" | Customer hasn't signed | Send to customer for signature |
| 5002 | "Contract amendment required" | Customer requested changes | Negotiate and update contract |
| 5003 | "Contract signature expired" | Signature window closed | Create new contract and resend |
| 5004 | "Legal review incomplete" | Legal hasn't approved | Follow up with legal team |

### Revenue Recognition Errors (6000 series)

| Code | Message | Cause | Resolution |
|------|---------|-------|-----------|
| 6001 | "Contract start date in past" | Start date already passed | Use today's date or correct date |
| 6002 | "Invoice generation failed" | Billing system error | Check billing system status |
| 6003 | "Revenue amount mismatch" | Invoice amount differs from ACV | Reconcile with finance |
| 6004 | "Recognition date not eligible" | Too early for recognition | Wait for contract start date |

---

## Related Documentation

- **How-To Guide**: [Customize and Extend](./REVOPS_HOWTO.md) the RevOps pipeline
- **Explanation**: [Why This Architecture Works](./REVOPS_EXPLANATION.md)
- **Tutorial**: [Running the Complete Pipeline](./REVOPS_TUTORIAL.md)
