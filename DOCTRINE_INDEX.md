# Doctrine Index: Navigation Map for DOCTRINE_2027

**Purpose**: This document maps DOCTRINE_2027 and DOCTRINE_COVENANT.md to all artifacts, use cases, and audiences. Use this to navigate the entire system and know exactly where to point people.

---

## Quick Navigation by Audience

### 📚 For Book / Whitepaper Authors
**Goal**: Compelling narrative that proves the design is inevitable

**Start Here**:
1. `DOCTRINE_2027.md` - **The Core Statement** (read first 3 sections)
   - "For fifty years, the work has been the same..."
   - "The Core Cycle Across Eras" table
   - "Why the Design Looks Inevitable"

2. `DOCTRINE_COVENANT.md` - **The Validation Bridge** (Covenant enforcement section)
   - Shows how doctrine maps to executable code

3. `SELF_EXECUTING_WORKFLOWS.md` - **Implementation Deep Dive**
   - Real example: "Minimal Example: Define Your Workflow"
   - Shows Turtle → System Understands → Validates → Generates → Executes flow

**Key Quotes to Use**:
- "For 50 years we modeled reality carefully, enforced quality rigorously, and closed feedback loops as fast as our tools allowed. The Chatman Equation Platform is those same habits, finally running at hardware speed."
- "The autonomous ontology system is not a departure from TAI's promise. It is that promise delivered at scale and proven by math, telemetry, and receipts."
- "Seen in this light, the technical design is not exotic; it is forced by the constraints accumulated over five decades."

**Chapter 1 Opening**: DOCTRINE_2027.md "The Core Statement" + Historical Arc
**Chapter 2 Deep Dive**: SELF_EXECUTING_WORKFLOWS.md + MAPE-K_AUTONOMIC_INTEGRATION.md

---

### 🛍️ For Marketplace / Product Team
**Goal**: How the design enables vertical stacks and scalable templates

**Start Here**:
1. `DOCTRINE_2027.md` - Section 4: **"One Contract at a Time" → One Sector at a Time**
   - Shows how ggen Marketplace vertical stacks flow from doctrine

2. `DOCTRINE_COVENANT.md` - **Covenant 1: Turtle Is Definition and Cause**
   - Explains why RDF/Turtle is the single source of truth
   - No drift between code, docs, APIs

3. `ggen-marketplace/knhk-yawl-workflows/` - **Example Vertical Stack**
   - `template/yawl-workflow-pure.ttl.j2` - How pure templates work
   - `queries/extract_*.sparql` - How extraction is mechanical
   - `README.md` - Market-ready documentation

**Key Messaging**:
- "Every vertical has a curated Σ slice (ontology), validated templates, sector-specific invariants Q, and MAPE-K loops tuned to its risk profile."
- "Vertical expertise becomes installable, verifiable knowledge assets."
- "ggen exists because hand-syncing these surfaces failed."

**For New Vertical Creation**:
1. Read `DOCTRINE_COVENANT.md` Covenant 4 (Patterns Expressible via Permutations)
2. Create sector-specific ontology in `ontology/{sector}/`
3. Create SPARQL queries for extraction in `queries/`
4. Create Π projections (templates) in `templates/`
5. Define Q constraints (Guards, latency SLOs) for your vertical
6. Document in marketplace README with doctrine alignment

---

### 🏗️ For Architecture & System Design
**Goal**: Understanding why each component exists and how they interconnect

**Start Here**:
1. `DOCTRINE_2027.md` - Section 2: **How the System Embodies the Pattern**
   - Maps O, Σ, Q, Π, MAPE-K to implementation

2. `DOCTRINE_COVENANT.md` - **All 6 covenants**
   - Covenant 1: Why Turtle is definition
   - Covenant 2: Why invariants are enforced
   - Covenant 3: Why MAPE-K is embedded
   - Covenant 4: Why patterns are permutations
   - Covenant 5: Why Chatman constant matters
   - Covenant 6: Why observations are first-class

3. `YAWL_TURTLE_ANALYSIS.md` - **Technical Architecture Analysis**
   - Shows complete technical mapping
   - Identifies pattern coverage

4. `SELF_EXECUTING_WORKFLOWS.md` - **Execution Pipeline**
   - Detailed 80/20 principle explanation
   - Shows how schema drives everything

**Architecture Components**:
- **O (Observation)**: `ontology/`, `registry/` (OpenTelemetry schemas), SPARQL results
- **Σ (Ontology)**: `yawl-extended.ttl`, `mape-k-autonomic.ttl`, sector-specific ontologies
- **Q (Invariants)**: `yawl-pattern-permutations.ttl`, Chicago TDD harness, latency bounds
- **Π (Projections)**: `templates/`, `ggen/` code generation
- **MAPE-K Loop**: `mape-k-*.sparql` queries, autonomic hooks, knowledge store

---

### 👨‍💻 For Swarm Agents & Developers
**Goal**: Understand the doctrine alignment checklist before coding

**Start Here**:
1. `CLAUDE.md` - Section: **"Doctrine North Star"**
   - Your briefing starts with doctrine alignment

2. `DOCTRINE_COVENANT.md` - **6 Binding Covenants**
   - Find which covenant applies to your work
   - Identify anti-patterns you must avoid
   - Know validation method for your covenant

3. `CLAUDE.md` - Section: **"Agent Briefing Template for Swarm Execution"**
   - Use this template for every task
   - Fills in: Principle, Covenant, What This Means, Anti-Patterns, Validation

**Before Writing Any Code**:
1. Read the relevant covenant (usually 1-2 sections, ~10 min read)
2. Fill in the briefing template (5 minutes)
3. Check that your code will satisfy the checklist
4. Know that covenant violations are BLOCKING

**Example Checklist for Covenant 1 (Turtle Is Definition)**:
- [ ] Does this add logic to templates? → VIOLATION (rewrite as ontology)
- [ ] Does this add implicit assumptions? → VIOLATION (state in Turtle)
- [ ] Could SPARQL extract this from existing Turtle? → YES (use SPARQL)
- [ ] Is this definition-driven or logic-driven? → DEFINITION-DRIVEN (good)

---

### ✅ For Code Review & Quality Gates
**Goal**: Every review uses covenants as the rubric

**Gate Checklist**:
Before any code is merged:

1. **Covenant Alignment** (5 min read)
   - Which covenant applies? (stated in commit message)
   - Does code violate it? If yes → REQUEST CHANGES

2. **Anti-Pattern Check** (see DOCTRINE_COVENANT.md)
   - Does code contain any listed anti-patterns?
   - If yes → REQUEST CHANGES

3. **Validation** (see DOCTRINE_COVENANT.md)
   - Does the covenant have a validation method?
   - Is that validation passing? (Weaver, Chicago TDD, tests)
   - If no → REQUEST CHANGES

4. **Documentation**
   - Does commit message cite the covenant?
   - Can future developers understand covenant alignment?

**In PR Comments**:
```
This PR must satisfy [Covenant Name].

See: DOCTRINE_COVENANT.md, Covenant [N]

What this means: [Brief explanation]
Anti-patterns to check: [List 2-3]
Validation required: [Weaver/Chicago/Integration]

Status: [Pass/Fail]
```

---

### 📊 For Validation & Testing
**Goal**: Every test ties back to a covenant

**Validation Hierarchy**:

```
DOCTRINE_2027
    ↓
DOCTRINE_COVENANT.md (6 covenants)
    ↓
VALIDATION METHODS:
    ├─ Weaver (O ⊨ Σ): weaver registry check && live-check
    ├─ Chicago TDD (Q ⊨ Implementation): cargo test + performance checks
    ├─ Pattern Matrix (Σ ⊨ Completeness): validate against permutations
    ├─ Integration (Full Cycle): O → Σ → μ → O'
    └─ Code Review (Anti-Pattern Check): DOCTRINE_COVENANT.md checklist
```

**Test Organization**:
- Tests for **Covenant 1** (Turtle is definition): Compare input → extraction → output
- Tests for **Covenant 2** (Invariants are law): Verify all Q constraints satisfied
- Tests for **Covenant 3** (MAPE-K at machine speed): Latency and loop closure
- Tests for **Covenant 4** (Patterns expressible): Validate against permutation matrix
- Tests for **Covenant 5** (Chatman constant): Latency ≤ 8 ticks for hot path
- Tests for **Covenant 6** (Observations drive): Verify all behaviors observable

---

## File Navigation Tree

```
DOCTRINE LAYER (your north star):
├── DOCTRINE_2027.md .......................... Narrative foundation (1000+ lines)
├── DOCTRINE_COVENANT.md ...................... Binding enforcement (600+ lines)
├── DOCTRINE_INDEX.md (this file) ............ Navigation map

IMPLEMENTATION LAYER (systems built from doctrine):
├── YAWL Turtle Workflows (Phase 1)
│   ├── ontology/yawl-extended.ttl ............ Extended execution semantics
│   ├── ontology/yawl-pattern-permutations.ttl Proof of all 43+ patterns
│   ├── ontology/workflows/examples/
│   │   ├── autonomous-work-definition.ttl ... Complex pattern example
│   │   ├── simple-sequence.ttl
│   │   ├── parallel-split.ttl
│   │   └── exclusive-choice.ttl
│   ├── SELF_EXECUTING_WORKFLOWS.md .......... Phase 1 implementation guide
│   └── YAWL_TURTLE_ANALYSIS.md ............. Architecture analysis
│
├── MAPE-K Autonomic Control (Phase 2)
│   ├── ontology/mape-k-autonomic.ttl ........ Complete feedback loop (1000+ lines)
│   ├── ontology/workflows/examples/
│   │   └── autonomic-self-healing-workflow.ttl Self-healing payment processor
│   ├── ggen-marketplace/knhk-yawl-workflows/queries/
│   │   ├── mape-k-monitor.sparql ........... Continuous observation
│   │   ├── mape-k-analyze.sparql ........... Anomaly detection & analysis
│   │   ├── mape-k-plan.sparql .............. Policy-driven action selection
│   │   └── mape-k-knowledge.sparql ......... Persistent learning
│   └── MAPE-K_AUTONOMIC_INTEGRATION.md ..... Phase 2 implementation guide
│
├── Marketplace Vertical Stack (Example)
│   ├── ggen-marketplace/knhk-yawl-workflows/
│   │   ├── template/
│   │   │   ├── yawl-workflow-pure.ttl.j2 .. Pure passthrough template
│   │   │   └── yawl-workflow.json.j2
│   │   ├── queries/
│   │   │   ├── extract_tasks_extended.sparql
│   │   │   ├── extract_data_flow.sparql
│   │   │   ├── extract_events.sparql
│   │   │   ├── extract_constraints.sparql
│   │   │   └── ... (8 extraction queries total)
│   │   └── README.md ........................ Vertical stack documentation
│
├── Validation & Enforcement
│   ├── chicago-tdd/harness/ ................ Latency enforcement (Q3)
│   ├── registry/ ........................... OpenTelemetry schema (O)
│   └── SHACL validators (in progress) ...... Type checking (Q2)
│
└── Internal Guidance
    ├── CLAUDE.md ........................... Development configuration
    ├── CHATMAN_EQUATION_SPEC.md ............ Formal Q definition
    └── README.md ........................... Project overview
```

---

## Cross-Reference Matrix

| Audience | Primary Doc | Secondary Docs | Use Case |
|----------|------------|-----------------|----------|
| **Book Author** | DOCTRINE_2027.md | SELF_EXECUTING_WORKFLOWS.md, MAPE-K guide | Narrative & proof |
| **Product Manager** | DOCTRINE_2027 S4 | ggen-marketplace/README | Vertical stacks |
| **Architect** | All covenants | YAWL_TURTLE_ANALYSIS.md | Design review |
| **Developer** | CLAUDE.md + Covenant | Code files | Implementation |
| **Code Reviewer** | DOCTRINE_COVENANT.md | Validation section | Merge gate |
| **QA Engineer** | Covenant validation | Integration tests | Test plan |
| **Marketplace Partner** | DOCTRINE_2027 S4 | ggen-marketplace docs | Stack creation |

---

## How to Point People

### "I want to understand the vision"
→ `DOCTRINE_2027.md` - Read "The Core Statement" and "The 2027 Statement"

### "I want to build a new vertical stack"
→ `DOCTRINE_2027.md` S4 + `ggen-marketplace/knhk-yawl-workflows/` example + `DOCTRINE_COVENANT.md` Covenants 1, 4

### "I want to understand why the design is this way"
→ `DOCTRINE_2027.md` - "Why the Design Looks Inevitable" + Covenant explanations

### "I need to validate a code change"
→ `DOCTRINE_COVENANT.md` - Find relevant covenant + validation section

### "I'm writing tests. What should I test?"
→ `DOCTRINE_COVENANT.md` - Find your covenant + validation method

### "I need to brief swarm agents"
→ `CLAUDE.md` - Agent Briefing Template + relevant covenant

### "We're writing marketing copy"
→ `DOCTRINE_2027.md` - "Key Quotes to Use" in this index + "The 2027 Statement"

### "I want to understand MAPE-K"
→ `MAPE-K_AUTONOMIC_INTEGRATION.md` (full detail) OR `DOCTRINE_COVENANT.md` Covenant 3

### "What does 'Turtle is definition and cause' mean?"
→ `DOCTRINE_COVENANT.md` - Covenant 1 + implementation references

### "Why is max_run_length ≤ 8 ticks important?"
→ `DOCTRINE_2027.md` - "The Constant That Ties It Together" + `DOCTRINE_COVENANT.md` Covenant 5

---

## Doctrine Versioning

All doctrine documents are versioned together:

| Version | Date | Status | Change |
|---------|------|--------|--------|
| 1.0.0 | 2025-11-16 | CANONICAL | Initial release |

**Never commit doctrine changes without updating version numbers.**

**When to Update Doctrine**:
- New industry standard that affects principles (RARE)
- Fundamental discovery about system design (VERY RARE)
- Clarification of existing principle (MINOR bump)

**When NOT to Update Doctrine**:
- Feature additions (add to implementation guides instead)
- Bug fixes (document in changelog, not doctrine)
- Marketplace verticals (each has own documentation)

---

## Status

✅ **Doctrine Foundation Complete and Committed**
- DOCTRINE_2027.md (1.0.0)
- DOCTRINE_COVENANT.md (1.0.0)
- CLAUDE.md updated with doctrine north star
- ggen-marketplace/knhk-yawl-workflows/ - Example vertical stack ready to ship
- All implementation files (17 files, 5000+ lines) aligned with doctrine

**Ready for**:
- Product deck
- Book/whitepaper
- Marketplace documentation
- Swarm execution
- Internal training
