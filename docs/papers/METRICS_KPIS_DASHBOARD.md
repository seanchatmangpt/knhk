# Metrics & KPIs Dashboard

## Sales & Pipeline Metrics

### Dashboard View (Track Weekly)

```
THIS WEEK                          MONTHLY              QUARTERLY
───────────────────────────────────────────────────────────────────
Prospects contacted: [#]           Target: 10-15        Target: 40-60
Response rate: [%]                 Target: 10-15%       Target: 10-15%
Meetings scheduled: [#]            Target: 2-3          Target: 8-12
Qualified leads: [#]               Target: 1            Target: 4-5
Proposals sent: [#]                Target: 0.75         Target: 3-4
Deals closed: [#]                  Target: 0.5          Target: 2-3
Pipeline value: $[amount]          Target: $30-50K      Target: $100K+
Win rate: [%]                      Target: 40-50%       Target: 40-50%
```

### Detailed Metrics

**Prospect Metrics**
```
Metric: Prospects Contacted
  Formula: Number of cold emails sent + calls made
  Target: 10-15 per month (ramp up to 15-20 in Year 2)
  Tracked: CRM, daily outreach log
  Importance: High (directly impacts pipeline)

Metric: Response Rate
  Formula: (Replies to emails + call pickups) / Total outreach attempts
  Target: 10-15% (academia is responsive)
  Tracked: CRM auto-calculates
  Importance: High (indicates messaging quality)
  Action: <10% = revise pitch; >15% = scale outreach
```

**Discovery Metrics**
```
Metric: Meetings Scheduled
  Formula: Number of discovery calls booked
  Target: 20-30% of responses (1 per 5-7 cold emails)
  Tracked: Calendar + CRM
  Importance: High
  Action: Track and improve scheduling success rate

Metric: Discovery-to-Proposal Rate
  Formula: Proposals sent / Meetings completed
  Target: 50-75% (good fit identification)
  Tracked: CRM
  Importance: High
  Action: <50% = improve qualifying; >75% = great insights
```

**Proposal Metrics**
```
Metric: Proposals Sent
  Formula: Number of formal proposals delivered
  Target: 0.75 per month in ramp, 1-2 in scale
  Tracked: Email + CRM log
  Importance: Medium (output, not outcome)

Metric: Proposal Acceptance Rate
  Formula: Accepted proposals / Proposals sent
  Target: 40-50%
  Tracked: CRM (marked as closed/won)
  Importance: High
  Action: <40% = reassess pricing or fit; >50% = increase outreach
```

**Pipeline Metrics**
```
Metric: Total Pipeline Value
  Formula: Sum of all open opportunities at each stage
  Target: $30-50K at any given time (3-4 projects in flight)
  Tracked: CRM pipeline report
  Importance: High (leading indicator of future revenue)
  Breakdown by stage:
    ├─ Prospect stage: 50 prospects × avg value = $1.5M potential
    ├─ Contacted: 10 contacted, maybe 1 deal = $30K potential
    ├─ Qualified: 3 qualified leads = $60-90K potential
    ├─ Proposed: 1-2 proposals = $30-50K potential
    └─ Committed: 0.5 deals in final = $15-25K potential
```

**Win Rate Metrics**
```
Metric: Deal Win Rate
  Formula: Deals closed / Proposals sent
  Target: 40-50%
  Tracked: CRM (proposals vs closed-won)
  Importance: Critical
  Explanation: 50% of proposals → deals means strong qualifying and messaging

Metric: Sales Cycle Length
  Formula: Average days from first contact to contract signature
  Target: 21-28 days (3-4 weeks)
  Tracked: CRM (first contact date → close date)
  Importance: High
  Action: >28 days = process needs acceleration; <21 = possibly not serious enough

Metric: Average Deal Size
  Formula: Total revenue closed / Number of deals
  Target: $30K Year 1, $40K Year 2, $50K+ Year 3
  Tracked: Spreadsheet (price mix)
  Importance: Medium (total revenue = volume × price)
  Goal: Shift from Silver to Gold/Platinum over time
```

---

## Project Delivery Metrics

### Quality Metrics

```
Metric: On-Time Delivery Rate
  Formula: % of projects delivered within agreed timeline
  Target: 100%
  Tracked: Spreadsheet (delivery date vs commit date)
  Importance: Critical
  How: Week-by-week tracking, weekly client calls
  Red flag: Missing week 3-4 milestone = escalate

Metric: Test Coverage
  Formula: % of code covered by automated tests
  Target: >90% (Gold/Platinum), 80%+ (Silver)
  Tracked: Coverage.io or similar tool
  Importance: High
  Action: <80% = add more tests; >95% = acceptable trade-off

Metric: Performance Certification
  Formula: % of code meeting DoD ≤8 tick performance requirement
  Target: 100%
  Tracked: DoD Validator output
  Importance: Critical (differentiation)
  How: Run validation in week 3, optimize in week 3-4

Metric: Documentation Quality
  Formula: Developer assessment + client satisfaction survey
  Target: 9/10 or higher
  Tracked: Post-project survey + self-assessment
  Importance: Medium-High
  Components:
    ├─ README clarity (5 pt scale)
    ├─ API doc completeness (5 pt scale)
    ├─ Example usefulness (5 pt scale)
    ├─ Troubleshooting helpfulness (5 pt scale)
    └─ Overall (5 pt scale)

Metric: Client Satisfaction
  Formula: Average score from post-project survey
  Target: 4.8/5.0 or higher
  Tracked: Post-project survey (email after delivery)
  Importance: High (leads to referrals and repeats)
  Questions:
    ├─ Code quality (1-5)
    ├─ Communication (1-5)
    ├─ Timeliness (1-5)
    ├─ Support (1-5)
    ├─ Overall value (1-5)
    └─ Likelihood to recommend (1-5)

Metric: Repeat Customer Rate
  Formula: % of customers doing 2+ projects with you
  Target: 20%+
  Tracked: CRM (customer repeat flag)
  Importance: High (low CAC, high LTV)
  Why: Strong indicator of satisfaction and trust
```

### Productivity Metrics

```
Metric: Projects Completed Per Month
  Formula: Number of deliveries / Month
  Target: 0.5 per month Y1, 1 per month Y2, 1+ per month Y3
  Tracked: Delivery calendar
  Importance: Medium (output metric)

Metric: Revenue Per Project
  Formula: Project revenue / Hours spent
  Target: Aim for projects >$30K average
  Tracked: Spreadsheet
  Importance: Medium-High (pricing/scoping)

Metric: Effective Hourly Rate
  Formula: Project revenue / Estimated hours
  Target: $150-200/hour effective rate
  Tracked: Time tracking + invoice
  Importance: Medium (financial health)
  How: (Project revenue) / (design hours + dev hours + testing hours + docs)

Metric: Utilization Rate
  Formula: % of time spent on billable work vs admin/business dev
  Target: 70-80% billable time, 20-30% non-billable
  Tracked: Time tracking app
  Importance: High (profitability driver)
  Breakdown:
    ├─ Project work (billable): 70-80%
    ├─ Sales/business dev: 10-15%
    ├─ Admin/overhead: 5-10%
    └─ Learning/optimization: 5%
```

---

## Financial Metrics

### Revenue Metrics

```
Metric: Monthly Recurring Revenue (MRR)
  Formula: Sum of all recurring revenue sources
  Target: $0K Y1, $2.5K/month Y2, $8K/month Y3
  Tracked: Spreadsheet (marketplace + support contracts)
  Importance: High (subscription/recurring base)
  Includes:
    ├─ Marketplace licensing revenue (50% share)
    ├─ Support contracts (if offered)
    └─ Consulting retainers (if any)

Metric: Annual Contract Value (ACV) - Marketplace
  Formula: Average annual revenue per implementation in marketplace
  Target: $3K Y1, $5K Y2, $7K Y3
  Tracked: Marketplace analytics dashboard
  Importance: Medium (early indicator of marketplace value)

Metric: Customer Acquisition Cost (CAC)
  Formula: Marketing + Sales spend / New customers acquired
  Target: <$5K per customer (focus on relationships, not ads)
  Tracked: Spreadsheet (marketing spend / deals closed)
  Importance: High (sustainable growth indicator)
  Why: Important for determining payback period

Metric: Lifetime Value (LTV)
  Formula: Total revenue from customer over relationship / CAC
  Target: 5:1 ratio or higher
  Tracked: Spreadsheet (customer total revenue / acquisition cost)
  Importance: Critical (determines business model viability)
  Calculation:
    └─ If CAC=$2K and customer generates $10K over lifetime, LTV:CAC = 5:1 ✓

Metric: Gross Margin (Services)
  Formula: (Revenue - COGS) / Revenue
  Target: 80%+
  Tracked: Monthly P&L
  Importance: High (pricing power)
  COGS = Your labor + direct costs (tools, infrastructure)
```

### Profitability Metrics

```
Metric: Gross Profit
  Formula: Revenue - Cost of Goods Sold
  Target: 80% of revenue in gross profit
  Tracked: Monthly P&L
  Importance: High

Metric: Net Profit (Bottom Line)
  Formula: Revenue - All costs (COGS + OpEx)
  Target: 50%+ in Year 1, 65%+ in Year 2, 75%+ in Year 3
  Tracked: Monthly P&L
  Importance: Critical (actual profitability)
  Why: High margins because service business with low overhead

Metric: Profit Margin %
  Formula: Net profit / Revenue
  Target: 55% Y1, 65% Y2, 75% Y3
  Tracked: Monthly P&L
  Importance: High (financial health)

Metric: Cash Position
  Formula: Bank balance
  Target: Maintain 3-month operating expense reserve
  Tracked: Bank account balance
  Importance: Critical (survival metric)
  Why: Protects against lumpy revenue/large unexpected costs
```

---

## Marketplace Metrics (Year 2+)

### Adoption Metrics

```
Metric: Implementations in Marketplace
  Formula: Number of published implementations
  Target: 10+ by end of Y1, 20+ by end of Y2, 30+ by Y3
  Tracked: Marketplace admin panel
  Importance: Medium-High

Metric: Downloads Per Implementation
  Formula: Total downloads / Number of implementations
  Target: 500+ per implementation per year (by Y2)
  Tracked: Marketplace analytics
  Importance: High (adoption indicator)
  Action: <500 = improve marketing; >500 = accelerate publishing

Metric: Active Users
  Formula: Number of distinct organizations using implementations
  Target: 100+ by Y2, 300+ by Y3
  Tracked: Marketplace + download tracking
  Importance: High

Metric: Deployment Rate
  Formula: Deployments per month per implementation
  Target: 50+ per implementation per month (Y2)
  Tracked: Deployment analytics
  Importance: High
  Why: Shows active use, not just download
```

### Revenue Metrics (Marketplace)

```
Metric: Marketplace Revenue Per Implementation
  Formula: Total license revenue / Number of implementations
  Target: $3K average Y2, $5K average Y3
  Tracked: Revenue report per implementation
  Importance: High

Metric: Marketplace Revenue Growth
  Formula: MoM growth rate of marketplace revenue
  Target: 5-10% MoM growth (compounding)
  Tracked: Month-over-month comparison
  Importance: High

Metric: Revenue Per User
  Formula: Marketplace revenue / Active users
  Target: $30-50 per user per year
  Tracked: Calculated from metrics above
  Importance: Medium-High
```

---

## Personal/Operational Metrics

### Work-Life Balance

```
Metric: Hours Worked Per Week
  Formula: Total billable + non-billable hours
  Target: 40-50 hours/week (full-time when scaled)
  Tracked: Time tracking app
  Importance: High (burnout prevention)

Metric: Time Off Taken
  Formula: Vacation/PTO days taken
  Target: 15-20 days per year
  Tracked: Calendar
  Importance: High (sustainability)

Metric: Skill Development
  Formula: Hours spent on learning/growth
  Target: 2-4 hours per week
  Tracked: Learning log
  Importance: Medium (competitive advantage)
```

### Learning Metrics

```
Metric: Papers Read
  Formula: Academic papers reviewed for implementations
  Target: 2-3 per month
  Tracked: Reading list
  Importance: Low-Medium

Metric: Technical Improvements
  Formula: Updates to ggen/KNHK tools
  Target: Minor updates monthly, major updates quarterly
  Tracked: GitHub commit log
  Importance: Medium (tooling improvement)
```

---

## Dashboard Template (Spreadsheet/Tool)

Create a simple Google Sheet with:

```
WEEKLY DASHBOARD
════════════════════════════════════════════

Sales Pipeline:
  Prospects contacted this week: ___
  Meetings scheduled: ___
  Proposals sent: ___
  Deals closed: ___
  Pipeline value: $______

Current Projects:
  Active projects: ___
  On track: Yes/No
  Potential risks: ___

Revenue:
  YTD revenue: $______
  This month target: $______
  Progress to target: ___%

Costs & Profit:
  YTD costs: $______
  YTD profit: $______
  Margin: ___%

Metrics Status:
  Response rate: __% (Target: 10-15%)
  Win rate: __% (Target: 40-50%)
  Client satisfaction: __._ / 5 (Target: 4.8+)
  On-time delivery: __% (Target: 100%)

Notes:
  This week: ___
  Next week priorities: ___
  Blockers: ___
```

---

## Review Cadence

```
Weekly Review (Every Friday)
  ├─ Update sales metrics
  ├─ Check project progress
  ├─ Note blockers/risks
  └─ Plan next week (1 hour)

Monthly Review (1st of month)
  ├─ Reconcile actual vs projected revenue
  ├─ Reconcile actual vs projected costs
  ├─ Calculate profit/margin
  ├─ Update all KPIs
  ├─ Plan adjustments for next month
  └─ 2-3 hours

Quarterly Review (Start of Q2/Q3/Q4)
  ├─ Review 3-month progress vs annual plan
  ├─ Analyze wins/losses (what worked, what didn't)
  ├─ Update financial projections
  ├─ Plan next quarter actions
  ├─ Assess if pricing/packages need adjustment
  └─ 4-5 hours

Annual Review (End of year)
  ├─ Full year financial reconciliation
  ├─ Success metrics analysis (client satisfaction, repeats, etc.)
  ├─ Lessons learned (what to do differently)
  ├─ Plan for next year
  ├─ Assess hiring needs
  ├─ Plan product/service improvements
  └─ 1-2 days
```

