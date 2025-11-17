# How-To: Customize and Extend the RevOps Pipeline

**A Practical Guide to Adapting the RevOps Pipeline for Your Organization**

- **Difficulty**: Intermediate
- **Time to Complete**: 2-3 hours per customization
- **Prerequisites**: Completed the Tutorial, understand basic workflow concepts

---

## Table of Contents

1. [Common Customization Scenarios](#common-customization-scenarios)
2. [Extending with New Approval Gates](#extending-with-new-approval-gates)
3. [Adding Custom Decision Logic](#adding-custom-decision-logic)
4. [Modifying SLO Targets](#modifying-slo-targets)
5. [Integrating External Systems](#integrating-external-systems)
6. [Handling Special Cases](#handling-special-cases)

---

## Common Customization Scenarios

### Scenario 1: Your Company Has Different Approval Thresholds

**Problem**: Acme's thresholds ($250K manager, $5M legal) don't match your company's structure

**Solution**: Customize the deal-approval-gate workflow

```yaml
# Your custom approval hierarchy
# config/approval-thresholds.yaml

approval_tiers:
  tier_1:
    approver_role: account_executive
    max_acv: 50000
    sla_hours: 2
    parallel: true

  tier_2:
    approver_role: sales_director
    max_acv: 250000
    sla_hours: 4
    requires: [tier_1]  # Sequence: must pass tier 1 first

  tier_3:
    approver_role: vp_sales
    max_acv: 1000000
    sla_hours: 24
    requires: [tier_1, tier_2]

  tier_4:
    approver_role: cfo
    max_acv: null  # Unlimited
    sla_hours: 2
    requires: [tier_1, tier_2, tier_3]
```

**Modify the workflow**:

```turtle
# Modified WF2: Deal Approval Gate (Custom Thresholds)
:deal_approval_gate [
  :splitType "OR" ;

  # Route 1: Small deal ($0-50K) - Fast track
  :child [
    :guard "deal_acv <= 50000" ;
    :next :tier_1_approval
  ] ;

  # Route 2: Mid deal ($50K-250K) - Manager approval
  :child [
    :guard "deal_acv > 50000 AND deal_acv <= 250000" ;
    :next :tier_2_approval
  ] ;

  # Route 3: Large deal ($250K-1M) - Director approval
  :child [
    :guard "deal_acv > 250000 AND deal_acv <= 1000000" ;
    :next :tier_3_approval
  ] ;

  # Route 4: Enterprise ($1M+) - Executive approval
  :child [
    :guard "deal_acv > 1000000" ;
    :next :tier_4_approval
  ]
] .

# Tier 1 always required
:tier_1_approval [
  :task :ae_review ;
  :next :sync_all_approvals
] .

:ae_review [
  :timeout "PT2H" ;
  :onTimeout :escalate_to_director
] .

# Tier 2: Only for mid-size deals
:tier_2_approval [
  :task :director_review ;
  :next :sync_all_approvals
] .

:director_review [
  :timeout "PT4H" ;
  :onTimeout :escalate_to_vp
] .

# ... similar for tier 3 and 4 ...
```

**Deploy**:

```bash
# Test your custom thresholds
curl -X POST http://localhost:8080/api/v1/workflows/deal-approval-gate-custom/cases \
  -d '{
    "case_id": "test_small",
    "data": {"deal_acv": 30000}  # Should go to AE only
  }'

curl -X POST http://localhost:8080/api/v1/workflows/deal-approval-gate-custom/cases \
  -d '{
    "case_id": "test_large",
    "data": {"deal_acv": 500000}  # Should go to Director
  }'

curl -X POST http://localhost:8080/api/v1/workflows/deal-approval-gate-custom/cases \
  -d '{
    "case_id": "test_enterprise",
    "data": {"deal_acv": 2000000}  # Should go to CFO
  }'
```

### Scenario 2: Add a New Approval Step (Compliance Check)

**Problem**: Your company needs compliance review for certain industries

**Solution**: Insert a new workflow step into the pipeline

```turtle
# Insert Compliance Check into WF2

# Before: Deal Approval → Contract Processing
# After:  Deal Approval → Compliance Check → Contract Processing

:deal_approval_completion [
  :splitType "OR" ;

  # Route 1: Non-regulated industry (direct to contract)
  :child [
    :guard "industry NOT IN ['healthcare', 'financial', 'government']" ;
    :next :contract_processing_entry
  ] ;

  # Route 2: Regulated industry (compliance check required)
  :child [
    :guard "industry IN ['healthcare', 'financial', 'government']" ;
    :next :compliance_review
  ]
] .

:compliance_review [
  :task :regulatory_officer_review ;
  :timeout "PT24H" ;
  :onSuccess :contract_processing_entry ;
  :onTimeout :escalate_cfo ;
  :onError :regulatory_escalation
] .

:regulatory_officer_review [
  :role "Compliance Officer" ;
  :data {
    :industry ?industry ;
    :jurisdiction ?customer_jurisdiction ;
    :contract_terms ?contract_terms
  } ;
  :output {
    :compliance_approved :boolean ;
    :required_clauses [:list] ;
    :restrictions [:list]
  }
] .
```

### Scenario 3: Skip Certain Workflows for Specific Deals

**Problem**: Internal deals shouldn't need legal review

**Solution**: Add bypass logic to skip workflows

```yaml
# config/workflow-bypass-rules.yaml

bypass_rules:
  - condition: "customer_type == 'internal'"
    skip_workflows: ["contract_processing", "legal_review"]
    reason: "Internal deals exempt from contract processing"

  - condition: "deal_acv < 10000"
    skip_workflows: ["legal_review"]
    reason: "Small deals don't require legal"
    requires_approval: "vp_sales"

  - condition: "customer_id IN renewal_customers AND deal_type == 'renewal'"
    skip_workflows: ["lead_qualification"]
    reason: "Renewal deals bypass qualification"
```

Implement bypass in workflow:

```turtle
:pipeline_entry [
  :splitType "OR" ;

  # Route 1: Bypass qualification for renewals
  :child [
    :guard "deal_type == 'renewal' AND customer_id IN known_customers" ;
    :next :deal_approval_gate
  ] ;

  # Route 2: Normal flow
  :child [
    :guard "deal_type == 'new'" ;
    :next :lead_qualification
  ]
] .
```

---

## Extending with New Approval Gates

### Add Procurement Approval

For B2B SaaS selling to enterprises with procurement departments:

```turtle
:procurement_gate [
  :task :procurement_submission ;
  :next :procurement_review
] .

:procurement_submission [
  :automated "true" ;
  :sends_to "procurement@customer.com" ;
  :template "procurement-package.pdf"
] .

:procurement_review [
  :timeout "PT72H" ;  # 72 hours typical procurement review
  :onTimeout :escalate_to_sponsor ;
  :role "Procurement Manager (Customer)"
] .

# Route back to finance if rejected
:procurement_review [
  :onError :procurement_changes_needed ;
] .

:procurement_changes_needed [
  :task :request_changes ;
  :loop_back_to :procurement_submission ;
  :max_iterations 3
] .
```

### Add Sponsor Approval (for large deals)

```turtle
:sponsor_approval [
  :task :executive_sponsor_review ;
  :role "C-Level Sponsor (Customer)" ;
  :timeout "PT48H" ;
  :onTimeout :sales_outreach
] .

:executive_sponsor_review [
  :data {
    :business_value ?deal_justification ;
    :strategic_fit ?fit_with_strategy ;
    :budget_availability ?budget_approved
  } ;
  :output {
    :sponsor_approved :boolean ;
    :deployment_timeline ?timeline
  }
] .
```

---

## Adding Custom Decision Logic

### Example 1: Net Revenue Retention (NRR) Bonus Decision

```turtle
# Calculate if deal qualifies for faster approval based on NRR impact

:nrr_calculation [
  :automated "true" ;
  :formula """
    annual_expansion = (new_contract_value - old_contract_value) / old_contract_value
    nrr_impact = annual_expansion / company_annual_revenue

    IF nrr_impact > 0.05 THEN high_value = true ELSE high_value = false
  """ ;
  :next :nrr_decision
] .

:nrr_decision [
  :splitType "OR" ;

  # High NRR: Fast approval
  :child [
    :guard "high_value == true" ;
    :next :fasttrack_approval
  ] ;

  # Normal NRR: Standard approval
  :child [
    :guard "high_value == false" ;
    :next :standard_approval
  ]
] .

:fasttrack_approval [
  :timeout "PT4H" ;  # 4 hours vs normal 24
  :approver "sales_director"
] .
```

### Example 2: Geographic Risk Decision

```turtle
# Different approval paths based on customer location and sanctions

:geographic_check [
  :automated "true" ;
  :lookup_table "countries-sanctioned.csv" ;
  :check_field "customer_country"
] .

:geographic_decision [
  :splitType "OR" ;

  # Sanctioned country: Requires legal + compliance
  :child [
    :guard "customer_country IN sanctioned_list" ;
    :next :sanctions_review
  ] ;

  # High-risk country: Compliance review only
  :child [
    :guard "customer_country IN high_risk_list" ;
    :next :compliance_review
  ] ;

  # Safe country: Normal approval
  :child [
    :guard "customer_country IN safe_list" ;
    :next :standard_approval
  ]
] .

:sanctions_review [
  :requires_approval ["legal", "compliance", "cfo"] ;
  :parallel "false"  # Sequential, not parallel
] .
```

---

## Modifying SLO Targets

### Adjust for Your Organization's Speed

```yaml
# config/revops-slos-custom.yaml

# Default Acme SLOs (reference)
acme_defaults:
  lead_qualification_time: 1 hour
  manager_approval_time: 4 hours
  legal_review_time: 4 hours
  finance_review_time: 2 hours
  executive_approval_time: 2 hours
  total_deal_cycle_time: 3 days

# Your custom SLOs (conservative - longer times)
your_company:
  lead_qualification_time: 4 hours    # Your team needs more time to qualify
  manager_approval_time: 8 hours      # Manager not always available
  legal_review_time: 8 hours          # Your legal team is smaller
  finance_review_time: 4 hours        # More complex deals
  executive_approval_time: 4 hours    # More deliberate
  total_deal_cycle_time: 10 days      # More realistic for your org

# Monitor different SLOs for different deal sizes
deal_size_slos:
  small_deals:    # <$50K
    cycle_time_target: 2 days
    alert_threshold: 5 days

  medium_deals:   # $50K-$500K
    cycle_time_target: 7 days
    alert_threshold: 14 days

  large_deals:    # >$500K
    cycle_time_target: 30 days
    alert_threshold: 60 days

# By region
regional_slos:
  apac:
    timezone: "Asia/Singapore"
    total_deal_cycle_time: 14 days      # Longer approval chains in region
    manager_approval_time: 24 hours     # Different working hours

  emea:
    timezone: "Europe/London"
    total_deal_cycle_time: 10 days
    manager_approval_time: 8 hours

  americas:
    timezone: "US/Eastern"
    total_deal_cycle_time: 7 days
    manager_approval_time: 4 hours
```

**Apply custom SLOs**:

```bash
# Update SLOs for your organization
curl -X PATCH http://localhost:8080/api/v1/config/slos \
  -H "Content-Type: application/json" \
  -d @config/revops-slos-custom.yaml

# Verify new SLOs
curl http://localhost:8080/api/v1/config/slos | jq '.slos'
```

---

## Integrating External Systems

### Connect to Salesforce

```bash
# Webhook to receive deal updates from Salesforce

curl -X POST http://localhost:8080/api/v1/integrations/webhooks \
  -d '{
    "name": "salesforce-deal-updates",
    "source": "salesforce",
    "event": "deal.updated",
    "webhook_url": "https://your-knhk-instance/webhooks/salesforce",
    "triggers": [
      "deal_created",
      "deal_amount_changed",
      "deal_stage_changed"
    ]
  }'
```

### Connect to HubSpot

```bash
# Create HubSpot integration
curl -X POST http://localhost:8080/api/v1/integrations \
  -d '{
    "type": "hubspot",
    "api_key": "YOUR_HUBSPOT_API_KEY",
    "features": [
      "sync_deals",
      "sync_contacts",
      "receive_webhooks"
    ]
  }'

# Map HubSpot deal properties to KNHK workflow fields
curl -X POST http://localhost:8080/api/v1/integrations/hubspot/mappings \
  -d '{
    "mappings": {
      "hubspot.dealstage": "knhk.workflow_stage",
      "hubspot.dealname": "knhk.deal_name",
      "hubspot.amount": "knhk.deal_acv",
      "hubspot.closedate": "knhk.target_close_date"
    }
  }'
```

### Connect to Slack for Notifications

```bash
# Post workflow events to Slack

curl -X POST http://localhost:8080/api/v1/integrations/slack \
  -d '{
    "workspace": "YOUR_WORKSPACE",
    "bot_token": "xoxb-YOUR-TOKEN",
    "channels": {
      "deal_approvals": "C_APPROVALS_CHANNEL",
      "bottlenecks": "C_OPS_CHANNEL",
      "slo_breaches": "C_ALERTS_CHANNEL"
    }
  }'
```

### Receive Updates via Webhook

```bash
# Your system sends deal events to KNHK
# Example from Salesforce:

curl -X POST http://localhost:8080/api/v1/webhooks/salesforce \
  -H "Content-Type: application/json" \
  -H "X-Salesforce-Signature: YOUR_SIGNATURE" \
  -d '{
    "event": "deal.created",
    "data": {
      "deal_id": "006f4000000K3QAAU",
      "deal_name": "Acme Corp Renewal",
      "deal_amount": 250000,
      "account_id": "001f4000000K9GEAA",
      "owner_id": "005f4000000K9GEAA",
      "close_date": "2025-12-31"
    }
  }'
```

---

## Handling Special Cases

### Case 1: Deals That Need to Skip Approval Steps

**Scenario**: Executive made a verbal commitment, need fast-track

```bash
# Manually advance deal to later stage
curl -X POST http://localhost:8080/api/v1/cases/deal_002/skip-to-stage \
  -d '{
    "target_stage": "contract_processing",
    "reason": "Executive fast-track approval",
    "approved_by": "lisa.wong@acme.com",
    "audit_note": "Authorized by CFO verbally on 11/17 at 2pm"
  }'
```

### Case 2: Deal That Needs Re-Approval (Deal Changed)

**Scenario**: Customer changed requirements mid-deal, need to re-approve

```bash
# Rewind deal to approval stage for re-approval
curl -X POST http://localhost:8080/api/v1/cases/deal_002/rewind \
  -d '{
    "target_stage": "deal_approval_gate",
    "reason": "Customer requirements changed",
    "change_description": "Added 3 additional licenses and expanded user count",
    "new_deal_acv": 850000
  }'
```

### Case 3: Deal with Unusual Legal Terms

**Scenario**: Customer's standard contract is outside normal bounds

```bash
# Escalate contract to senior legal counsel
curl -X POST http://localhost:8080/api/v1/cases/deal_002/tasks/contract-review/escalate \
  -d '{
    "escalate_to": "senior_legal_counsel",
    "reason": "Non-standard payment terms (50/50 split)",
    "complexity_score": 8,
    "required_expertise": ["payment_terms", "international_law"]
  }'
```

### Case 4: Competitor Intel Affects Deal

**Scenario**: Competitor released new product, customer asking for discount

```bash
# Re-open pricing negotiation
curl -X POST http://localhost:8080/api/v1/cases/deal_002/tasks/pricing-exception/reopen \
  -d '{
    "reason": "Competitive threat detected",
    "market_intel": "Competitor released equivalent product at 30% lower price",
    "requested_discount": 0.20,
    "approver": "finance_manager"
  }'
```

---

## Common Customizations Checklist

Before modifying your RevOps pipeline:

- [ ] Document your current approval process (as-is)
- [ ] Map to desired future state (to-be)
- [ ] Identify which workflows need changes
- [ ] Test changes in non-production environment
- [ ] Train your team on new process
- [ ] Monitor SLO compliance for 2 weeks
- [ ] Gather feedback from approvers
- [ ] Refine based on real-world execution
- [ ] Document the customizations for future team members

---

## Need More Help?

- **Can't implement a customization?** See [Reference: Decision Matrices](./REVOPS_REFERENCE.md)
- **Want to understand the design choices?** See [Explanation: Architecture Rationale](./REVOPS_EXPLANATION.md)
- **Confused about workflow patterns?** See the main [Tutorial](./REVOPS_TUTORIAL.md)
- **Looking for specific field names?** See [Reference: Deal Data Structure](./REVOPS_REFERENCE.md)
