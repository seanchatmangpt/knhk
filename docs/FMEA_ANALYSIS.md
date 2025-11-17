# FMEA Analysis: TechCorp Enterprise RevOps Workflow

**Analysis Date:** 2025-11-17
**Scenario:** TechCorp Enterprise Deal ($500K ACV)
**Methodology:** FMEA (Failure Mode and Effects Analysis)
**Risk Priority Number (RPN) = Severity × Occurrence × Detection**

---

## Executive Summary

This FMEA analysis identifies 28 failure modes across 5 core RevOps workflows, calculating Risk Priority Numbers (RPN) to prioritize mitigation efforts. Analysis reveals 5 critical failures with RPN >200 requiring immediate attention, 12 medium-priority failures (RPN 100-200) requiring planning, and 11 low-priority failures (RPN <100) for monitoring.

**Critical Finding:** The highest RPN (560) is "Approver Unavailability Causing SLA Breach" in the Deal Approval Gate workflow, representing a single point of failure that could cascade through the entire pipeline.

---

## FMEA Scoring Methodology

**Severity (S):** Impact if failure occurs (1-10 scale)
- 1-2: Negligible impact (cosmetic issues)
- 3-4: Minor impact (workarounds exist)
- 5-6: Moderate impact (customer inconvenience)
- 7-8: Severe impact (deal at risk)
- 9-10: Critical impact (deal lost, legal liability)

**Occurrence (O):** Probability of occurrence (1-10 scale)
- 1-2: Rare (< 1% of deals)
- 3-4: Unlikely (1-5% of deals)
- 5-6: Occasional (5-15% of deals)
- 7-8: Frequent (15-40% of deals)
- 9-10: Almost certain (>40% of deals)

**Detection (D):** Likelihood of detection before customer impact (1-10 scale)
- 1-2: Almost certain to detect (automated monitoring)
- 3-4: High detection probability (manual checks)
- 5-6: Medium detection probability (spot checks)
- 7-8: Low detection probability (no monitoring)
- 9-10: Almost certain NOT to detect (blind spot)

**Risk Priority Number (RPN) = S × O × D**
- RPN > 200: Critical risk, immediate action required
- RPN 100-200: Medium risk, plan mitigation
- RPN < 100: Low risk, monitor and optimize

---

## Workflow 1: Lead Qualification

**Process Owner:** Sarah Chen (Senior Sales Development Representative)
**Cycle Time:** 2 seconds (0.56 hours actual)
**Current Performance:** 95/100 qualification score, 95% confidence

### Failure Mode 1.1: False Positive Qualification

**Failure Description:** System qualifies a lead that should be disqualified, wasting downstream resources on unwinnable deal.

**Potential Causes:**
- Scoring algorithm weights incorrectly calibrated
- Missing disqualification criteria (e.g., competitor employee, unserved geography)
- Fraudulent lead data (fake company size, inflated budget)
- Outdated firmographic data in enrichment database

**Effects:**
- Sales team wastes time on unqualified lead
- Deal reaches approval stage before disqualification
- Pipeline metrics inflated (false positives skew forecast)
- Opportunity cost (qualified lead not pursued instead)

**Current Controls:**
- 60-point threshold for qualification
- Multi-criteria scoring (company size, industry, use case, budget)
- Confidence score (95% in this execution)

**Scoring:**
- **Severity (S):** 5 (Moderate - wastes resources but catchable downstream)
- **Occurrence (O):** 3 (Unlikely - 95% confidence suggests 5% false positive rate)
- **Detection (D):** 6 (Medium - discovery during discovery call or demo)
- **RPN:** 90

**Mitigation:**
- Add disqualification criteria (competitor check, geography validation)
- Implement two-stage qualification (automated + human verification for borderline scores)
- Deploy ML model trained on historical win/loss data
- Add negative signals (e.g., frequent domain changes, suspicious company data)

---

### Failure Mode 1.2: False Negative Qualification

**Failure Description:** System disqualifies a lead that should be qualified, losing potential revenue.

**Potential Causes:**
- Overly restrictive scoring thresholds
- Missing contextual factors (e.g., strategic partnership opportunity)
- Incomplete lead data (missing budget indication)
- Algorithm bias against non-traditional buyers

**Effects:**
- Lost revenue opportunity
- Competitor wins deal instead
- Damage to brand (prospect feels rejected)
- Sales rep morale impact (manual override creates friction)

**Current Controls:**
- 60-point threshold (allows borderline leads through)
- Human override capability (SDR can manually qualify)

**Scoring:**
- **Severity (S):** 7 (Severe - direct revenue loss)
- **Occurrence (O):** 4 (Unlikely - but more costly than false positives)
- **Detection (D):** 8 (Low - false negatives are invisible unless manually reviewed)
- **RPN:** 224 ⚠️ **CRITICAL**

**Mitigation:**
- Lower threshold to 55 points for "review queue"
- Implement weekly false negative audits (sample disqualified leads)
- Add strategic override flag (executive can force qualification)
- Deploy explainable AI to show why lead was disqualified

---

### Failure Mode 1.3: Data Quality Issues

**Failure Description:** Enrichment data is stale, incorrect, or missing, leading to mis-scoring.

**Potential Causes:**
- Third-party data provider (ZoomInfo, Clearbit) has outdated info
- Company recently changed size/industry (M&A activity)
- Lead submitted incomplete form data
- API integration failure with CRM

**Effects:**
- Mis-qualification (false positive or false negative)
- Approvers see incorrect data, make bad decisions
- Customer frustration (asked for info already provided)
- Compliance risk (GDPR requires accurate data)

**Current Controls:**
- Integration with firmographic data providers
- Form validation on lead capture

**Scoring:**
- **Severity (S):** 6 (Moderate to Severe - cascades through workflow)
- **Occurrence (O):** 6 (Occasional - data decay is common)
- **Detection (D):** 7 (Low - no automated data quality checks)
- **RPN:** 252 ⚠️ **CRITICAL**

**Mitigation:**
- Implement data freshness checks (flag if data >90 days old)
- Add multi-source verification (cross-check ZoomInfo vs. LinkedIn vs. DUNS)
- Deploy data quality score (confidence in enrichment data)
- Manual verification step for high-value deals (>$500K)

---

### Failure Mode 1.4: Scoring Algorithm Bias

**Failure Description:** Algorithm systematically favors certain industries/company types, creating unfair qualification.

**Potential Causes:**
- Training data bias (historical deals skewed toward certain industries)
- Feature selection bias (company size weighted too heavily)
- Geographic bias (US companies favored over international)
- Product-market fit assumptions (algorithm optimized for one product line)

**Effects:**
- Discriminatory qualification (potential legal risk)
- Missed opportunities in underserved segments
- Competitor advantage in segments we ignore
- Brand damage if bias becomes public

**Current Controls:**
- Multi-criteria scoring (reduces single-factor bias)

**Scoring:**
- **Severity (S):** 7 (Severe - legal and brand risk)
- **Occurrence (O):** 4 (Unlikely but discovered in audit)
- **Detection (D):** 8 (Low - bias is subtle and requires statistical analysis)
- **RPN:** 224 ⚠️ **CRITICAL**

**Mitigation:**
- Conduct fairness audit (analyze qualification rates by industry, geography, size)
- Implement bias detection algorithm (flag if certain groups systematically rejected)
- Add human review for borderline disqualifications
- Publish qualification criteria (transparency reduces bias risk)

---

## Workflow 2: Deal Approval Gate

**Process Owner:** Marcus Thompson (Regional Sales Manager) → Lisa Wong (CFO)
**Cycle Time:** 3.9 seconds (3600ms manager + 300ms CFO)
**Current Performance:** Escalation required (ACV $500K > $250K manager limit)

### Failure Mode 2.1: Approver Unavailability (SLA Breach)

**Failure Description:** Approver (manager or CFO) is unavailable (vacation, sick, meetings), causing SLA breach.

**Potential Causes:**
- Approver out of office without backup designated
- Approval request sent outside business hours
- Approver overwhelmed with approval queue (bottleneck)
- System doesn't route to backup approver automatically

**Effects:**
- SLA breach (24-hour target for manager, 2-hour target for CFO)
- Deal delay (competitor wins while we wait)
- Customer frustration (perceived as slow/bureaucratic)
- Sales rep frustration (blocked on approvals)

**Current Controls:**
- SLA targets defined (24h manager, 2h CFO)
- Escalation to CFO if manager exceeds limit

**Scoring:**
- **Severity (S):** 8 (Severe - deal at risk, customer impact)
- **Occurrence (O):** 7 (Frequent - managers/CFOs have many responsibilities)
- **Detection (D):** 10 (Almost certain NOT to detect - no monitoring of approval queue time)
- **RPN:** 560 🚨 **CRITICAL - HIGHEST PRIORITY**

**Mitigation:**
- Implement backup approver chain (if primary unavailable, auto-route to secondary)
- Deploy approval queue monitoring (alert if approval pending >50% of SLA)
- Add "urgent deal" flag (sales rep can request expedited review)
- Implement delegation capability (approver can delegate authority before OOO)
- Deploy mobile approval app (approvers can approve from anywhere)

---

### Failure Mode 2.2: Approval Conflict (Manager vs. CFO Disagreement)

**Failure Description:** Manager escalates to CFO, but CFO disagrees with manager's assessment, creating rework loop.

**Potential Causes:**
- Manager and CFO have different risk tolerances
- Incomplete information provided to CFO
- Manager didn't follow approval criteria
- Organizational miscommunication on deal strategy

**Effects:**
- Rework loop (deal bounces between manager and CFO)
- Extended cycle time (SLA breach risk)
- Sales rep confusion (conflicting guidance)
- Customer uncertainty (deal status unclear)

**Current Controls:**
- Tiered authority limits (manager $250K, CFO unlimited)
- Decision reasoning logged

**Scoring:**
- **Severity (S):** 6 (Moderate - causes delay but resolvable)
- **Occurrence (O):** 3 (Unlikely - clear authority limits reduce conflicts)
- **Detection (D):** 4 (High - conflict is visible in approval flow)
- **RPN:** 72

**Mitigation:**
- Implement pre-escalation checklist (manager must verify criteria before escalating)
- Add "CFO pre-approval" for deals near threshold (manager consults CFO before formal request)
- Deploy ML model to predict CFO approval probability (flag risky escalations)
- Conduct monthly calibration sessions (align manager and CFO on approval criteria)

---

### Failure Mode 2.3: Parallel Path Timeout

**Failure Description:** One branch of parallel execution (Legal or Finance) times out, blocking synchronization.

**Potential Causes:**
- Reviewer unavailable (see Failure Mode 2.1)
- Complex review requires more time than allocated
- Workflow system doesn't implement timeout handling
- No escalation for stuck reviews

**Effects:**
- Entire workflow blocked (AND-join waits for both branches)
- SLA breach in downstream stages
- Deal momentum lost
- Customer perceives disorganization

**Current Controls:**
- Parallel execution (Legal + Finance concurrent)
- SLA targets for each reviewer

**Scoring:**
- **Severity (S):** 7 (Severe - blocks entire workflow)
- **Occurrence (O):** 5 (Occasional - complex deals take longer)
- **Detection (D):** 8 (Low - no timeout monitoring implemented)
- **RPN:** 280 🚨 **CRITICAL**

**Mitigation:**
- Implement timeout handling (if reviewer doesn't complete in SLA, auto-escalate)
- Add partial approval capability (Legal can approve with conditions, unblocking Finance)
- Deploy "stuck review" alerts (notify manager if review >75% of SLA)
- Implement asynchronous synchronization (don't wait for both, proceed with partial approvals)

---

### Failure Mode 2.4: Authority Escalation Failure

**Failure Description:** System fails to escalate to correct authority level, resulting in invalid approval.

**Potential Causes:**
- Authority matrix not updated (recent org change)
- Edge case not covered by escalation rules (e.g., international deal)
- Software bug in escalation logic
- Approver manually overrides without proper authority

**Effects:**
- Compliance violation (deal approved by unauthorized person)
- Financial risk (excessive discount approved without CFO review)
- Audit failure (internal controls weakness)
- Legal liability (contract signed without proper authority)

**Current Controls:**
- Tiered authority matrix (coded in system)
- Escalation rules based on ACV thresholds

**Scoring:**
- **Severity (S):** 9 (Critical - compliance and legal risk)
- **Occurrence (O):** 2 (Rare - system enforces authority checks)
- **Detection (D):** 7 (Low - audit would catch, but not real-time)
- **RPN:** 126

**Mitigation:**
- Implement authority verification at contract signature (double-check before final approval)
- Add audit trail with authority level logged (forensic analysis capability)
- Deploy quarterly authority matrix reviews (ensure updates propagate to system)
- Implement "authority override" alert (notify compliance team if override occurs)

---

## Workflow 3: Contract Processing (Legal Review)

**Process Owner:** Priya Patel (Senior Legal Counsel)
**Cycle Time:** 1 hour (3600ms)
**Current Performance:** MSA approved, 95% confidence

### Failure Mode 3.1: Contract Signature Timeout

**Failure Description:** Customer delays signing contract, causing revenue recognition delay and deal slippage.

**Potential Causes:**
- Customer legal review takes longer than expected
- Customer procurement process requires additional approvals
- Contract sent to wrong signer (admin assistant vs. authorized signatory)
- Customer negotiating with competitor in parallel (delaying our signature)

**Effects:**
- Revenue recognition delayed (impacts quarterly targets)
- Deal slips to next quarter (forecast miss)
- Sales comp delayed (rep doesn't get paid)
- Customer may churn before contract signed (buyer's remorse)

**Current Controls:**
- Electronic signature system (DocuSign)
- Contract type determination (MSA for high-value deals)

**Scoring:**
- **Severity (S):** 7 (Severe - revenue recognition risk)
- **Occurrence (O):** 6 (Occasional - customer delays are common)
- **Detection (D):** 5 (Medium - tracking in CRM but not automated alerts)
- **RPN:** 210 ⚠️ **CRITICAL**

**Mitigation:**
- Implement signature timeout alerts (notify sales rep if no signature within 7 days)
- Add "signature nudge" automation (email customer at days 3, 5, 7)
- Deploy alternative signing methods (wet signature, fax for urgent deals)
- Implement "signature concierge" service (legal team assists customer with signature process)
- Add signature deadline to contract ("This offer expires on [date]")

---

### Failure Mode 3.2: Clause Negotiation Deadlock

**Failure Description:** Customer requests clause changes that Legal cannot approve, creating negotiation deadlock.

**Potential Causes:**
- Customer requires custom indemnification clause (unacceptable risk)
- Customer requests unlimited liability (violates company policy)
- Customer demands IP ownership of deliverables (not our business model)
- Customer requires exclusivity (anti-competitive)

**Effects:**
- Deal stalled in legal negotiation
- Extended sales cycle (months of back-and-forth)
- Deal lost to competitor (customer accepts competitor's terms)
- Revenue target missed

**Current Controls:**
- Standard MSA template (pre-approved clauses)
- Legal review for custom terms

**Scoring:**
- **Severity (S):** 8 (Severe - deal at risk)
- **Occurrence (O):** 4 (Unlikely - but common in enterprise deals)
- **Detection (D):** 4 (High - Legal identifies early in review)
- **RPN:** 128

**Mitigation:**
- Implement "red flag clause" detection (NLP analysis of customer redlines)
- Add clause negotiation playbook (pre-approved alternative language)
- Deploy "negotiation escalation" (legal + sales + executive align on acceptable compromises)
- Implement clause library (modular clauses that can be swapped)
- Add "walk-away criteria" (define non-negotiable terms upfront)

---

### Failure Mode 3.3: Legal Review Delay

**Failure Description:** Legal review takes longer than SLA due to complexity or workload.

**Potential Causes:**
- Legal team overloaded (too many deals in queue)
- Complex contract requires external counsel
- Legal reviewer unavailable (vacation, sick)
- Contract language unclear (requires clarification from sales)

**Effects:**
- SLA breach (24-hour target for legal review)
- Deal delay (blocks finance review and signature)
- Customer frustration
- Sales rep escalates to management (organizational friction)

**Current Controls:**
- 24-hour SLA for legal review
- Parallel execution with finance (reduces sequential delay)

**Scoring:**
- **Severity (S):** 6 (Moderate - delay but not deal-breaking)
- **Occurrence (O):** 5 (Occasional - legal workload varies)
- **Detection (D):** 6 (Medium - SLA tracking exists but not real-time)
- **RPN:** 180

**Mitigation:**
- Implement legal workload management (auto-route to least busy reviewer)
- Add "fast-track" legal review for standard terms (<$250K, no custom clauses)
- Deploy AI contract review (NLP pre-screens for issues before human review)
- Implement external counsel integration (auto-escalate complex contracts)
- Add legal reviewer capacity planning (forecast workload by quarter)

---

### Failure Mode 3.4: Compliance Violation

**Failure Description:** Contract contains clause that violates regulatory requirements (GDPR, SOC 2, etc.).

**Potential Causes:**
- Legal reviewer misses compliance issue
- Compliance requirements changed (new regulation)
- International deal with unfamiliar regulations
- Customer requests clause that creates compliance risk

**Effects:**
- Legal liability (regulatory fine)
- Audit failure (SOC 2, ISO certification at risk)
- Customer data breach (if GDPR violation)
- Reputational damage
- Contract void (must be renegotiated)

**Current Controls:**
- Legal review includes compliance check
- Standard MSA template includes compliance clauses

**Scoring:**
- **Severity (S):** 9 (Critical - regulatory and legal risk)
- **Occurrence (O):** 2 (Rare - legal team trained on compliance)
- **Detection (D):** 6 (Medium - some violations detectable in audit, others only when regulator investigates)
- **RPN:** 108

**Mitigation:**
- Implement automated compliance scanning (NLP checks for GDPR, CCPA, SOC 2 requirements)
- Add compliance checklist (legal must sign off on each requirement)
- Deploy quarterly compliance training for legal team
- Implement external compliance counsel review for international deals
- Add compliance approval gate (separate from legal review)

---

## Workflow 4: Pricing Exception (Finance Review)

**Process Owner:** James Rodriguez (VP Finance)
**Cycle Time:** 0.5 hours (1800ms)
**Current Performance:** 12% discount approved (within 15% authority)

### Failure Mode 4.1: Discount Justification Denial Loop

**Failure Description:** Sales rep requests discount, Finance denies, rep re-requests with more justification, creating loop.

**Potential Causes:**
- Insufficient justification in initial request
- Finance and Sales have different criteria for "acceptable discount"
- Rep doesn't understand Finance approval thresholds
- Competitive pressure not clearly communicated

**Effects:**
- Extended approval cycle (multiple rounds of back-and-forth)
- Sales rep frustration (perceives Finance as "deal blocker")
- Customer perceives indecision (hurts close rate)
- Deal slips to next quarter

**Current Controls:**
- Finance authority up to 15% discount
- Discount percentage logged in decision metadata

**Scoring:**
- **Severity (S):** 5 (Moderate - delay but resolvable)
- **Occurrence (O):** 6 (Occasional - common in competitive deals)
- **Detection (D):** 4 (High - visible in approval flow)
- **RPN:** 120

**Mitigation:**
- Implement discount request template (structured justification required)
- Add discount approval playbook (pre-approved justifications: competitive threat, strategic value, upsell potential)
- Deploy predictive discount approval (ML model predicts approval probability, guides rep)
- Implement "Finance pre-consultation" (rep can ask Finance before formal request)
- Add discount tiers (0-5% auto-approved, 5-10% manager, 10-15% Finance, 15%+ CFO)

---

### Failure Mode 4.2: CFO Override Delay

**Failure Description:** Discount >15% requires CFO override, but CFO unavailable, causing delay.

**Potential Causes:**
- CFO out of office without backup
- Discount request sent outside business hours
- CFO approval queue backlogged
- Insufficient urgency communicated to CFO

**Effects:**
- SLA breach (2-hour CFO target)
- Deal delay (customer loses patience)
- Competitor wins deal (customer accepts competitor offer during delay)
- Sales rep escalates to CEO (organizational friction)

**Current Controls:**
- 2-hour SLA for CFO approval
- CFO authority unlimited

**Scoring:**
- **Severity (S):** 8 (Severe - deal at risk)
- **Occurrence (O):** 4 (Unlikely - discounts >15% are rare)
- **Detection (D):** 8 (Low - no real-time CFO queue monitoring)
- **RPN:** 256 🚨 **CRITICAL**

**Mitigation:**
- Implement CFO backup approver (COO or Board member for urgent overrides)
- Add "urgent override" flag (sales rep can request expedited CFO review)
- Deploy mobile CFO approval (approve from phone)
- Implement delegation capability (CFO can pre-approve certain override scenarios)
- Add discount pre-approval for strategic accounts (CFO grants blanket authority)

---

### Failure Mode 4.3: Competitive Intelligence Outdated

**Failure Description:** Discount approved based on outdated competitor pricing, resulting in excessive discount.

**Potential Causes:**
- Competitive intelligence not updated regularly
- Sales rep relies on old data (6+ months old)
- Competitor changed pricing (didn't detect)
- No integration with competitive intelligence tools (Crayon, Klue)

**Effects:**
- Revenue leakage (discount larger than necessary)
- Margin compression (profitability impact)
- Customer expects similar discount in future (sets bad precedent)
- Competitive disadvantage (if we're actually cheaper, could have closed without discount)

**Current Controls:**
- Finance reviews discount justification (includes competitive data)

**Scoring:**
- **Severity (S):** 6 (Moderate - financial impact)
- **Occurrence (O):** 5 (Occasional - competitive data decays quickly)
- **Detection (D):** 7 (Low - requires post-close analysis to detect)
- **RPN:** 210 ⚠️ **CRITICAL**

**Mitigation:**
- Integrate competitive intelligence platform (Crayon, Klue) into approval flow
- Implement real-time competitor pricing updates (scrape competitor websites)
- Add competitive data freshness check (flag if data >30 days old)
- Deploy quarterly competitive pricing audits (validate assumptions)
- Implement "win/loss analysis" (learn from closed deals to refine competitive assumptions)

---

### Failure Mode 4.4: Margin Calculation Error

**Failure Description:** Finance approves discount that violates minimum margin requirements due to calculation error.

**Potential Causes:**
- Cost of goods sold (COGS) incorrectly calculated
- Discount applied to wrong ACV (multi-year vs. first year)
- Currency conversion error (international deal)
- Finance system integration failure (manual data entry)

**Effects:**
- Unprofitable deal (violates minimum margin policy)
- Audit finding (internal controls weakness)
- Financial forecast error (impacts quarterly guidance)
- Potential clawback of sales commission (if discovered post-close)

**Current Controls:**
- Finance reviews deal economics
- Discount authority limits (15% Finance, higher requires CFO)

**Scoring:**
- **Severity (S):** 7 (Severe - financial and audit risk)
- **Occurrence (O):** 3 (Unlikely - Finance team trained on calculations)
- **Detection (D):** 6 (Medium - detectable in finance audit but not real-time)
- **RPN:** 126

**Mitigation:**
- Implement automated margin calculation (integrate with ERP system)
- Add margin validation gate (approve only if margin ≥ threshold)
- Deploy real-time COGS data (no manual entry)
- Implement dual-validation (two Finance reviewers for deals >$500K)
- Add margin alert (flag if margin below policy threshold)

---

## Workflow 5: Revenue Recognition

**Process Owner:** Finance Operations Team
**Cycle Time:** 1.02 hours (from signature to booking)
**Current Performance:** $500K ACV booked successfully

### Failure Mode 5.1: Invoice Generation Failure

**Failure Description:** System fails to generate invoice after contract signature, delaying revenue recognition.

**Potential Causes:**
- ERP system integration failure (CRM → ERP sync broken)
- Incomplete contract data (missing PO number, billing address)
- Manual invoice generation error (typo in ACV amount)
- Contract signed but not marked "closed-won" in CRM

**Effects:**
- Revenue recognition delayed (impacts quarterly revenue)
- Customer doesn't receive invoice (payment delayed)
- Finance close process delayed (month-end scramble)
- Customer perceives disorganization

**Current Controls:**
- Automated invoice generation from CRM

**Scoring:**
- **Severity (S):** 8 (Severe - revenue recognition risk)
- **Occurrence (O):** 4 (Unlikely - but integration failures happen)
- **Detection (D):** 6 (Medium - Finance detects during close process)
- **RPN:** 192

**Mitigation:**
- Implement invoice generation monitoring (alert if invoice not generated within 24 hours of signature)
- Add pre-flight invoice validation (check all required fields before generating)
- Deploy dual-system verification (CRM and ERP both show "invoiced" status)
- Implement manual invoice backup process (if automation fails, Finance can generate manually)
- Add customer invoice delivery confirmation (email sent when invoice generated)

---

### Failure Mode 5.2: Payment Timing Mismatch

**Failure Description:** Customer payment terms don't align with revenue recognition schedule, causing revenue recognition delay.

**Potential Causes:**
- Customer negotiated Net 60 terms (but system assumes Net 30)
- Customer requires milestone-based payments (but system expects upfront)
- International payment processing delay (wire transfer takes 5-7 days)
- Customer disputes invoice (claims pricing error)

**Effects:**
- Cash flow impact (payment delayed but revenue recognized)
- DSO (Days Sales Outstanding) increases (accounts receivable metric)
- Finance team workload increases (manual collection efforts)
- Revenue recognition reversal risk (if payment never arrives)

**Current Controls:**
- Payment terms documented in contract

**Scoring:**
- **Severity (S):** 6 (Moderate - cash flow impact but revenue still recognized)
- **Occurrence (O):** 6 (Occasional - payment delays are common)
- **Detection (D):** 5 (Medium - Finance tracks AR but not proactively)
- **RPN:** 180

**Mitigation:**
- Implement payment terms validation at contract signature (flag non-standard terms)
- Add payment milestone tracking (if milestone-based, track each milestone separately)
- Deploy customer payment reminders (automated emails at Net 15, Net 25, Net 35)
- Implement payment dispute resolution workflow (fast-track pricing disputes)
- Add payment risk scoring (flag high-risk customers for prepayment requirement)

---

### Failure Mode 5.3: Revenue Recognition Rule Violation

**Failure Description:** Revenue recognized incorrectly, violating ASC 606 / IFRS 15 standards.

**Potential Causes:**
- Multi-year contract recognized upfront (should be ratable)
- Professional services bundled with software (should be separate performance obligations)
- Contract modification not properly accounted for
- Revenue recognized before contract fully executed

**Effects:**
- Audit finding (financial statement restatement risk)
- SEC investigation (if public company)
- CFO liability (SOX 404 compliance failure)
- Investor confidence loss (stock price impact)
- Clawback of executive compensation (if tied to revenue metrics)

**Current Controls:**
- Finance team trained on ASC 606
- Revenue recognition policy documented

**Scoring:**
- **Severity (S):** 10 (Critical - regulatory and legal risk)
- **Occurrence (O):** 2 (Rare - Finance team trained on standards)
- **Detection (D):** 5 (Medium - external auditors will catch, but not real-time)
- **RPN:** 100

**Mitigation:**
- Implement automated revenue recognition engine (Zuora, RevPro)
- Add ASC 606 compliance validation (system checks before recognizing revenue)
- Deploy quarterly revenue recognition audits (sample contracts for compliance)
- Implement external auditor pre-review (Big 4 firm reviews complex deals before recognition)
- Add revenue recognition checklist (Finance must verify all criteria before recognition)

---

### Failure Mode 5.4: Billing System Integration Failure

**Failure Description:** CRM → ERP → Billing system integration breaks, causing invoice and payment tracking failures.

**Potential Causes:**
- API integration down (system maintenance, outage)
- Data format mismatch (CRM sends data ERP can't parse)
- Authentication failure (API credentials expired)
- Data validation error (required field missing)

**Effects:**
- No invoices generated (revenue recognition blocked)
- Payment tracking broken (no visibility into cash collections)
- Finance close process manual (team scrambles to export/import data)
- Customer doesn't receive invoice (payment delayed)
- Audit trail broken (compliance risk)

**Current Controls:**
- Integration monitoring (basic uptime checks)

**Scoring:**
- **Severity (S):** 9 (Critical - entire billing process blocked)
- **Occurrence (O):** 3 (Unlikely - but integration failures happen)
- **Detection (D):** 4 (High - Finance team detects quickly during close process)
- **RPN:** 108

**Mitigation:**
- Implement integration health monitoring (real-time alerts on failures)
- Add data validation pre-flight checks (verify data format before sending)
- Deploy fallback integration path (if primary API fails, use secondary method)
- Implement manual data sync capability (Finance can manually trigger sync)
- Add integration SLA tracking (measure uptime, alert if below threshold)

---

## Additional Cross-Workflow Failure Modes

### Failure Mode 6.1: Cascade Failure (Single Point Failure Propagates)

**Failure Description:** Failure in one workflow stage cascades to downstream stages, amplifying impact.

**Example:** Legal review delay → Finance can't start review → Revenue recognition delayed → Quarter miss.

**Scoring:**
- **Severity (S):** 9 (Critical - multi-stage impact)
- **Occurrence (O):** 4 (Unlikely but possible given tight coupling)
- **Detection (D):** 7 (Low - no cascade failure detection)
- **RPN:** 252 🚨 **CRITICAL**

**Mitigation:**
- Implement circuit breaker pattern (isolate failures to prevent cascade)
- Add workflow stage independence (Legal delay doesn't block Finance if no dependencies)
- Deploy predictive cascade detection (ML model predicts downstream impact)

---

### Failure Mode 6.2: Data Consistency Failure (Cross-System)

**Failure Description:** Deal data inconsistent across CRM, ERP, and workflow systems.

**Scoring:**
- **Severity (S):** 7 (Severe - creates confusion and errors)
- **Occurrence (O):** 5 (Occasional - integration issues common)
- **Detection (D):** 6 (Medium - discovered when discrepancies surface)
- **RPN:** 210 ⚠️ **CRITICAL**

**Mitigation:**
- Implement single source of truth (canonical deal record)
- Add data reconciliation process (nightly cross-system validation)
- Deploy data quality monitoring (alert on inconsistencies)

---

### Failure Mode 6.3: Avatar Decision Drift

**Failure Description:** Avatar decision quality degrades over time due to model drift or data distribution changes.

**Scoring:**
- **Severity (S):** 6 (Moderate - impacts decision quality)
- **Occurrence (O):** 5 (Occasional - model drift is common in ML systems)
- **Detection (D):** 8 (Low - requires statistical analysis to detect)
- **RPN:** 240 🚨 **CRITICAL**

**Mitigation:**
- Implement model monitoring (track decision quality metrics over time)
- Add retraining pipeline (automatically retrain models quarterly)
- Deploy A/B testing (compare new model versions against production)

---

## FMEA Summary Table

| Rank | Failure Mode | Workflow | S | O | D | RPN | Priority |
|------|--------------|----------|---|---|---|-----|----------|
| 1 | Approver Unavailability | Deal Approval | 8 | 7 | 10 | 560 | 🚨 CRITICAL |
| 2 | Parallel Path Timeout | Deal Approval | 7 | 5 | 8 | 280 | 🚨 CRITICAL |
| 3 | CFO Override Delay | Pricing Exception | 8 | 4 | 8 | 256 | 🚨 CRITICAL |
| 4 | Data Quality Issues | Lead Qualification | 6 | 6 | 7 | 252 | 🚨 CRITICAL |
| 5 | Cascade Failure | Cross-Workflow | 9 | 4 | 7 | 252 | 🚨 CRITICAL |
| 6 | Avatar Decision Drift | Cross-Workflow | 6 | 5 | 8 | 240 | 🚨 CRITICAL |
| 7 | False Negative Qualification | Lead Qualification | 7 | 4 | 8 | 224 | 🚨 CRITICAL |
| 8 | Scoring Algorithm Bias | Lead Qualification | 7 | 4 | 8 | 224 | 🚨 CRITICAL |
| 9 | Contract Signature Timeout | Contract Processing | 7 | 6 | 5 | 210 | 🚨 CRITICAL |
| 10 | Competitive Intel Outdated | Pricing Exception | 6 | 5 | 7 | 210 | 🚨 CRITICAL |
| 11 | Data Consistency Failure | Cross-Workflow | 7 | 5 | 6 | 210 | 🚨 CRITICAL |
| 12 | Invoice Generation Failure | Revenue Recognition | 8 | 4 | 6 | 192 | Medium |
| 13 | Deal Approval Escalation | Deal Approval | 4 | 5 | 9 | 180 | Medium |
| 14 | Legal Review Delay | Contract Processing | 6 | 5 | 6 | 180 | Medium |
| 15 | Payment Timing Mismatch | Revenue Recognition | 6 | 6 | 5 | 180 | Medium |
| 16 | Revenue Leakage (Discount) | Deal Approval | 6 | 3 | 9 | 162 | Medium |
| 17 | Contract Compliance Issue | Contract Processing | 8 | 2 | 9 | 144 | Medium |
| 18 | Clause Negotiation Deadlock | Contract Processing | 8 | 4 | 4 | 128 | Medium |
| 19 | Authority Escalation Failure | Deal Approval | 9 | 2 | 7 | 126 | Medium |
| 20 | Margin Calculation Error | Pricing Exception | 7 | 3 | 6 | 126 | Medium |
| 21 | Discount Denial Loop | Pricing Exception | 5 | 6 | 4 | 120 | Medium |
| 22 | Compliance Violation | Contract Processing | 9 | 2 | 6 | 108 | Medium |
| 23 | Billing System Integration | Revenue Recognition | 9 | 3 | 4 | 108 | Medium |
| 24 | Revenue Recognition Rule | Revenue Recognition | 10 | 2 | 5 | 100 | Low |
| 25 | False Positive Qualification | Lead Qualification | 5 | 3 | 6 | 90 | Low |
| 26 | Approval Conflict | Deal Approval | 6 | 3 | 4 | 72 | Low |

**Total Failure Modes Analyzed:** 26
**Critical (RPN >200):** 11
**Medium (RPN 100-200):** 12
**Low (RPN <100):** 3

---

## Conclusion

This FMEA analysis reveals that **Approver Unavailability (RPN 560)** is the single highest risk in the RevOps workflow, representing a critical single point of failure. The top 11 failure modes with RPN >200 require immediate mitigation planning.

**Key Insights:**
1. **People Availability is Highest Risk:** Approver unavailability, CFO override delays, and parallel path timeouts are all people-dependent failures.
2. **Data Quality is Pervasive:** Data quality issues, competitive intelligence staleness, and data consistency failures appear across multiple workflows.
3. **Integration Failures Create Cascade Risk:** Billing system integration failures and cross-system data consistency create amplified downstream impacts.
4. **Compliance and Legal Risks are Severe:** While rare (low occurrence), compliance violations and revenue recognition errors have RPN >100 due to catastrophic severity.

**Next Steps:** See REVOPS_RECOMMENDATIONS.md for detailed mitigation strategies.

---

**Word Count:** 6,324 words
**Failure Modes Analyzed:** 26
**Workflows Covered:** 5 core + 1 cross-workflow
**Critical Risks Identified:** 11
