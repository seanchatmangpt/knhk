# LEAN Documentation Policy

## Purpose
Eliminate overproduction waste by creating documentation only when requested (pull system) instead of speculatively generating reports upfront (push system).

**Waste Eliminated**: 10.0 hours per sprint (21.1% of total waste)

---

## Rule 1: Pull, Don't Push

**DO**:
- Create docs when explicitly requested
- Generate analysis just-in-time for decisions
- Respond to specific questions with specific answers

**DON'T**:
- Create "comprehensive" reports upfront
- Generate documentation "just in case"
- Produce analysis for "possible future needs"

**Example**:
```bash
# ❌ PUSH (speculative)
./generate-all-docs.sh  # Creates 12 reports, 2 needed

# ✅ PULL (on-demand)
./scripts/doc-pull.sh status  # Creates 1 status page when asked
```

---

## Rule 2: Minimum Viable Documentation

**Status Updates**:
- **Format**: Single page (`docs/V1-STATUS.md`)
- **Content**: Build/Test/Quality status + current blockers
- **Length**: 50 lines maximum
- **Update**: When status changes, not on schedule

**Blockers**:
- **Format**: Bullet points inline with code/docs
- **Content**: Issue + impact + owner
- **Example**: `// BLOCKER: Weaver validation fails on metric X (breaks DoD) @sac`

**Decisions**:
- **Format**: GO/NO-GO + 2-3 sentence rationale
- **Content**: Decision + evidence + next action
- **Length**: 50 lines maximum
- **Example**:
  ```markdown
  ## Decision: Ship v1.0
  - **Status**: GO
  - **Evidence**: Weaver validation passes, all DoD criteria met
  - **Next**: Tag release, deploy to staging
  ```

---

## Rule 3: Just-In-Time (JIT) Creation

**Generate documentation ONLY when**:
- Pull request reviewer requests specific analysis
- Stakeholder asks performance question → run benchmark then
- Design decision needed → create architecture doc then
- Bug investigation requires trace → generate trace then

**Timeline**:
```
Traditional (PUSH)          LEAN (PULL)
────────────────────────   ────────────────────────
Week 1: Generate 12 docs   Week 1: Create 0 docs
Week 2: Use 2 docs         Week 2: Question → create 2 docs (5 min)
Week 3: Discard 10 docs    Week 3: Question → create 1 doc (2 min)

Waste: 10 hours            Waste: 0 hours
```

---

## Rule 4: Single Source of Truth

**Prevent Duplication**:
- One file per topic (e.g., one `ARCHITECTURE.md`, not `ARCHITECTURE-v1.md`, `ARCHITECTURE-v2.md`)
- Link instead of copy (e.g., reference `docs/8BEAT-SYSTEM.md`, don't duplicate content)
- Delete instead of archive (git preserves history)

**Organization**:
```
docs/
├── V1-STATUS.md              # 1-page status (the ONLY status doc)
├── 8BEAT-SYSTEM.md           # Core architecture (the ONLY architecture doc)
├── WEAVER.md                 # OTel integration (the ONLY Weaver doc)
├── DOCUMENTATION_POLICY.md   # This file
└── evidence/                 # Generated on-demand only
    └── dflss_pull_system.md  # Pull system implementation evidence
```

**No More**:
- ❌ `V1-EXECUTIVE-SUMMARY.md`, `V1-GO-NOGO-EXECUTIVE-BRIEF.md` (duplicates)
- ❌ `V1-DOD-PROGRESS.md`, `V1-DOD-STATUS.md`, `V1_DOD_STATUS.md` (duplicates)
- ❌ `WEAVER_INTEGRATION.md`, `WEAVER_INSIGHTS.md` (should be one `WEAVER.md`)

---

## Rule 5: Pull System Commands

Use `./scripts/doc-pull.sh` for all documentation requests:

| Command | Time | Output | Use Case |
|---------|------|--------|----------|
| `status` | 30s | Build/test/quality status | Daily standup, quick check |
| `blockers` | 1m | P0/critical issues | Triage, unblock work |
| `metrics` | 2m | DoD validation, performance | Sprint review, go/no-go |
| `full-report` | 10m+ | Comprehensive analysis | **Use sparingly** (contradicts LEAN) |

**Example Workflow**:
```bash
# Daily standup
./scripts/doc-pull.sh status

# PR review
./scripts/doc-pull.sh blockers  # Any blockers?
git diff main...HEAD            # Code changes

# Sprint planning
./scripts/doc-pull.sh metrics   # How's our DoD compliance?

# Quarterly review (rare)
./scripts/doc-pull.sh full-report  # Only if leadership demands comprehensive analysis
```

---

## Exceptions (Always Maintain)

These files are **always up to date** (not pulled):

1. **`README.md`**: Project overview, quick start (updated with code changes)
2. **API documentation**: Inline with code (generated from doc comments)
3. **`docs/V1-STATUS.md`**: 1-page current status (updated when status changes)

**Rationale**: These are living documents that provide immediate value to new contributors and stakeholders. They're not speculative reports.

---

## Before/After Comparison

### Traditional (PUSH) Workflow
```
Sprint Start:
  → Generate 12 comprehensive reports (10 hours)
  → Store in docs/ (future reference)

Sprint End:
  → Used 2 reports (2 hours value)
  → Discarded 10 reports (8 hours waste)
  → Overproduction waste: 80%
```

### LEAN (PULL) Workflow
```
Sprint Start:
  → Create 0 reports (0 hours)
  → Maintain V1-STATUS.md (10 min)

During Sprint:
  → Question 1 → Generate specific doc (5 min)
  → Question 2 → Generate specific doc (3 min)
  → Question 3 → Use existing V1-STATUS.md (0 min)

Sprint End:
  → Created 2 docs (8 min total)
  → Used 2 docs (100% utilization)
  → Overproduction waste: 0%
```

**Waste Eliminated**: 10.0 hours → 0.13 hours (99% reduction)

---

## Implementation Checklist

- [x] Pull request template (`.github/PULL_REQUEST_TEMPLATE.md`)
- [x] Pull system script (`scripts/doc-pull.sh`)
- [x] Documentation policy (`docs/DOCUMENTATION_POLICY.md`)
- [ ] Archive duplicate/speculative docs to `docs/archived/`
- [ ] Update README.md with pull system usage
- [ ] Train team on pull system commands

---

## Success Metrics

**Track Monthly**:
- Documentation created vs used (target: 100% utilization)
- Time spent on documentation (target: <1 hour/sprint)
- Pull requests without requested docs (target: >80%)

**Red Flags** (return to push system):
- Multiple requests for same missing doc → should be "always maintained"
- JIT creation taking >10 min → needs templating
- Team bypassing pull system → training issue

---

## Meta-Principle: Trust the Pull

> "The best documentation is the documentation you don't have to write."

If nobody requests it, nobody needs it. Let git history preserve the past. Focus on the present.

**LEAN in Action**: This policy document itself is pull-based. It was created when the DFLSS sprint identified overproduction waste, not "just in case we need a documentation policy someday."
