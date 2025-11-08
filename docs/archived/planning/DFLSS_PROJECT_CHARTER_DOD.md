# Definition of Done — DFLSS Project Charter

**State at sign-off = Ready to enter Measure. Evidence must be linkable.**

---

## 1. Approvals

* [ ] Sponsor, Process Owner, Black Belt signed and dated.

* [ ] RACI published for Define, Measure, Analyze, Design, Verify.

**Evidence**: Signed charter document with dates; RACI matrix published and linked.

---

## 2. Problem/VOC/Baseline

* [ ] Problem statement with quantified pain (latency, defects, audit hours, KLOC, services count).

* [ ] VOC sources listed and stored.

* [ ] Baseline metrics defined with data sources and time window.

**Evidence**: Problem statement document with metrics; VOC repository link; Baseline measurement plan with data sources.

---

## 3. CTQs (SMART)

* [ ] Targets stated and testable:

  * Hot-path guard ≤2 ns (τ ≤ 8 ticks).

  * ≥80% validations on hot path.

  * p95 decision latency ≤10 ms E2E.

  * Violations ≤10 ppm; rollback ≤0.5%.

  * 100% receipts coverage; audit prep −80%.

  * Validation code −70%; middleware −50%.

* [ ] Metrics owner + collection method per CTQ.

**Evidence**: CTQ document with SMART criteria; Metrics ownership matrix; Collection method documentation per CTQ.

---

## 4. Scope

* [ ] In/Out lists by domain, systems, geos.

* [ ] Phase-1 pilot named; canary plan defined.

* [ ] Exit criteria to scale specified.

**Evidence**: Scope document with In/Out matrix; Pilot selection document; Canary deployment plan; Scale exit criteria.

---

## 5. Financials

* [ ] OOM benefits and costs with assumptions.

* [ ] Measurement plan for benefits realization.

* [ ] Finance sign-off.

**Evidence**: Financial model with OOM estimates; Benefits realization measurement plan; Finance approval signature.

---

## 6. Architecture Readiness (summary, not build)

* [ ] Hot path: branchless C guards (ASK/COUNT/COMPARE/UNIQUE/VALIDATE) called out.

* [ ] Warm path: Rust AOT, MPHF, preload strategy documented.

* [ ] Cold path: Erlang reasoning boundary defined.

* [ ] Sidecar pattern chosen; mTLS, retries, CB noted.

* [ ] OTEL weaver live-check plan.

* [ ] Lockchain receipts plan (hash(A)=hash(μ(O)), Π as ⊕-monoid).

**Evidence**: Architecture summary document; Hot/Warm/Cold path specifications; Sidecar design document; OTEL integration plan; Lockchain receipts specification.

---

## 7. Ontology & Policy Inputs

* [ ] Ontology inventory (e.g., FIBO, GoodRelations, FHIR, SCOR, Enterprise).

* [ ] Guard catalog mapped to invariants Q (typing, cardinality, uniqueness, ranges).

* [ ] Hook generation plan (LLM+Tera) and approval workflow.

**Evidence**: Ontology inventory document; Guard catalog with invariant mappings; Hook generation workflow document.

---

## 8. Risk Register

* [ ] Data locality/cache-miss risk with AOT/MPHF mitigations.

* [ ] Over-blocking risk with shadow-mode plan.

* [ ] Partner/protocol risk with adapters/contract tests.

* [ ] Operational overhead risk with sampling budgets.

**Evidence**: Risk register document; Mitigation plans for each risk; Risk owner assignments.

---

## 9. Governance & Controls

* [ ] Versioning for policy packs; change control defined.

* [ ] Security posture: mTLS, key mgmt, PII handling.

* [ ] DR/HA SLOs and error budgets.

**Evidence**: Governance document; Policy pack versioning strategy; Security controls document; DR/HA SLOs and error budgets.

---

## 10. Dashboards & Evidence Paths

* [ ] Live links stubbed for: hot-path hit-rate, decision latency, violations ppm, receipts verification, code/middleware reductions, availability.

* [ ] Repo locations for charter artifacts, SIPOC, CTQ tree, risk log.

**Evidence**: Dashboard links (stubbed or live); Repository structure document; Artifact location index.

---

## 11. Communications & Training

* [ ] Stakeholder map; cadence calendar.

* [ ] Pilot runbook outline (shadow → enforce).

**Evidence**: Stakeholder map document; Communication calendar; Pilot runbook outline.

---

## 12. Acceptance Checklist (all must be "Yes")

* [ ] Problem/VOC/baseline complete and reproducible.

* [ ] CTQs traceable to metrics and owners.

* [ ] Scope/financials/risks approved.

* [ ] Architecture and ontology inputs documented to the level needed for Measure.

* [ ] Governance, security, and observability plans in place.

* [ ] Dashboards provisioned; links resolve.

* [ ] Kickoff for Measure scheduled with agenda.

**Evidence**: Acceptance checklist signed; All items verified; Measure kickoff scheduled.

---

## KGC Invariants Captured in Charter

The charter must explicitly reference or incorporate these formal invariants:

* **Law**: **A = μ(O)** - Actions are deterministic projections of observations

* **Idempotence**: **μ∘μ = μ** - Safe retry semantics without coordination

* **Typing**: **O ⊨ Σ** - Operations satisfy schema constraints

* **Order**: **Λ total** - Deterministic ordering maintained

* **Time bound**: **τ ≤ 8** (hot path) - Epoch containment for hot path guards

* **Merge**: **Π, ⊕** - Receipt composition forms monoid

* **Invariants**: **preserve(Q)** - Invariant preservation enforced

* **Shard**: **μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ)** - Shard distributivity for parallel evaluation

* **Provenance**: **hash(A) = hash(μ(O))** - Cryptographic verification enabled

**Evidence**: Charter document references formal foundations; Invariants documented in architecture section.

---

## Formal Foundation Reference

All charter documentation must align with KNHK's formal mathematical foundations:

* See [Formal Mathematical Foundations](formal-foundations.md) for complete treatment of all 17 laws.

* See [DFLSS Project Charter](DFLSS_PROJECT_CHARTER.md) for project structure.

* See [Reflex Enterprise Press Release](REFLEX_ENTERPRISE_PRESS_RELEASE.md) for product context.

---

## Done Criteria

**Done = all items above signed, linked, and auditable.**

Each checklist item must have:
1. **Evidence**: Linkable document or artifact
2. **Owner**: Assigned responsibility
3. **Verification**: Sign-off or approval
4. **Traceability**: Link to CTQ or requirement

---

## Sign-Off

**Charter Ready for Measure Phase**

Sponsor: __________  Date: ____

Process Owner: _____ Date: ____

Black Belt: ________ Date: ____

Quality Assurance: __ Date: ____

---

**Next Phase**: [Measure Phase Kickoff](DFLSS_MEASURE_PHASE.md) (to be created)

