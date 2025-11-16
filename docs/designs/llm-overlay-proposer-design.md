# LLM-Based Overlay Proposer System Design

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Design Specification (Pre-Implementation)
**Authors**: Backend API Developer Agent
**Related Documents**:
- [Autonomous Ontology System Design](../autonomous-ontology-system-design.md)
- [Autonomous Ontology Index](../autonomous-ontology-index.md)
- [ADR-005: Overlays for Experimental Changes](../autonomous-ontology-adr.md#adr-005)

---

## Executive Summary

This document specifies the design of the **LLM-Based Overlay Proposer**, a constraint-aware system that autonomously generates ontology change proposals (ΔΣ) using large language models. The system must respect hard invariants (Q1-Q5), organizational doctrines, performance budgets, and guard constraints while proposing improvements to the KNHK ontology based on observed patterns.

The key innovation is **constraint-aware generation**: the LLM operates within a bounded space defined by invariants and doctrines, ensuring all proposals are valid before promotion. This prevents the LLM from suggesting changes that violate system guarantees (e.g., >8 tick operations, retrocausal data flows, or broken approval chains).

**Design Goals**:
- Autonomous ontology evolution guided by observations
- Hard constraint satisfaction (Q1-Q5) enforced at generation time
- Doctrine compliance validated before promotion
- Performance budget preservation (≤8 ticks for hot path)
- Learning from accepted/rejected proposals
- Sector-specific adaptation (finance, healthcare, manufacturing, logistics)

**Non-Goals** (Deferred to Future Phases):
- Multi-region distributed LLM coordination
- Real-time streaming proposals
- LLM fine-tuning on proprietary data (Phase 2)
- Automated promotion without human review (requires Phase 5 security)

---

## 1. System Architecture

### 1.1 High-Level Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     OBSERVATION PLANE (O)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Pattern      │  │ Event        │  │ Anomaly      │          │
│  │ Detection    │  │ Correlation  │  │ Detection    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         └─────────────────┴─────────────────┘                   │
│                           │                                     │
│                  Detected Patterns                              │
└───────────────────────────┼─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  LLM OVERLAY PROPOSER                            │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 1. Pattern Analysis                                       │  │
│  │    - Extract semantic meaning from observation patterns   │  │
│  │    - Identify relevant sector (finance, healthcare, etc.) │  │
│  └───────────────────────────────────────────────────────────┘  │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 2. Constraint Loading                                     │  │
│  │    - Load applicable doctrines (sector + org policies)    │  │
│  │    - Load hard invariants (Q1-Q5)                         │  │
│  │    - Load guard profiles (immutable boundaries)           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 3. Prompt Engineering                                      │  │
│  │    - Construct constraint-aware prompt                    │  │
│  │    - Include few-shot examples (good/bad proposals)       │  │
│  │    - Specify output format (ΔΣ structure)                 │  │
│  └───────────────────────────────────────────────────────────┘  │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 4. LLM Generation (Constrained)                           │  │
│  │    - Strategy A: Prompt-based constraints (baseline)      │  │
│  │    - Strategy B: Guided decoding (LMQL) for critical Q    │  │
│  │    - Strategy C: Post-hoc filtering (safety net)          │  │
│  │    - Strategy D: Hybrid (recommended)                     │  │
│  └───────────────────────────────────────────────────────────┘  │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 5. Proposal Extraction                                     │  │
│  │    - Parse LLM output to ΔΣ format                        │  │
│  │    - Extract reasoning and confidence                     │  │
│  │    - Estimate performance impact (ticks)                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                            │                                    │
│                     Proposal (ΔΣ)                               │
└────────────────────────────┼────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  VALIDATION PIPELINE                             │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Static   │→ │ Invariant│→ │ Doctrine │→ │ Guard    │       │
│  │ Check    │  │ Check    │  │ Check    │  │ Check    │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│                                    │                             │
│                                    ▼                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │Performance│→ │ Rollback │→ │Compat.   │                      │
│  │ Check    │  │ Check    │  │ Check    │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
└──────────────────────────────────┼──────────────────────────────┘
                                   │
                    Valid ✓ / Invalid ✗
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     CHANGE PLANE (ΔΣ+Q)                          │
│                                                                  │
│  If Valid:   Promote to Σ snapshot → Atomic swap               │
│  If Invalid: Reject + Log failure + Learn from mistake         │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Component Responsibilities

#### Pattern Detection (Observation Plane)
- **Inputs**: RDF triples, OTEL spans, workflow executions
- **Outputs**: `DetectedPattern` with semantic description and confidence
- **Examples**:
  - "15 different account types observed in finance sector"
  - "Diabetes treatment protocol updated in medical literature"
  - "Route optimization showing 20% inefficiency in logistics"

#### LLM Proposer (New Component)
- **Inputs**: `DetectedPattern`, `DoctrineSet`, `HardInvariants`, `GuardProfiles`
- **Outputs**: `Proposal` containing ΔΣ and metadata
- **Responsibilities**:
  - Understand pattern semantics
  - Generate valid ontology extensions
  - Respect all constraints
  - Provide reasoning and confidence scores

#### Validation Pipeline (Change Plane)
- **Inputs**: `Proposal`
- **Outputs**: `ValidationReport` (pass/fail + reasons)
- **Responsibilities**:
  - Enforce Q1-Q5 invariants
  - Validate against doctrines
  - Check performance impact
  - Verify rollback capability

#### Learning System (New Component)
- **Inputs**: Accepted/rejected proposals with feedback
- **Outputs**: Updated few-shot examples, refined prompts
- **Responsibilities**:
  - Build corpus of good/bad proposals
  - Adapt prompts based on success/failure patterns
  - Track constraint violation patterns

---

## 2. Constraint Types

### 2.1 Hard Invariants (Q1-Q5)

These are **immutable** system guarantees that no proposal can violate:

| Invariant | Description | Validation Method | Example Violation |
|-----------|-------------|-------------------|-------------------|
| **Q1: No Retrocausation** | Time flows forward, no temporal cycles | Graph cycle detection | Adding `causes` relation where A→B and B→A |
| **Q2: Type Soundness** | All properties have valid domain/range | SHACL shape validation | Property with domain=Person, range=Integer on String value |
| **Q3: Guard Preservation** | `max_run_len ≤ 8` for hot path | Static analysis of C codegen | Adding 10 new required properties to hot path struct |
| **Q4: SLO Compliance** | Hot path operations ≤8 ticks | Performance profiling simulation | Proposal adds nested validation requiring 12 ticks |
| **Q5: Performance Bounds** | No regression >10% on any benchmark | Comparative benchmark execution | New overlay causes 15% slowdown in `ingest_triples()` |

**Enforcement**:
- Q1-Q2: Validated by SHACL engine
- Q3-Q4: Estimated during proposal generation, validated during codegen
- Q5: Requires actual benchmark execution (post-promotion validation with rollback)

### 2.2 Doctrines (Organizational Policies)

Doctrines are **sector-specific** rules that can be updated but must be respected:

**Finance Sector Example**:
```yaml
doctrines:
  - id: FIN-001
    name: "Approval Chain Requirement"
    rule: "All account modifications require two-step approval"
    affected_classes: [Account, Transaction]
    validation: "Check for approval_required=true property"

  - id: FIN-002
    name: "Audit Trail Immutability"
    rule: "Transaction history cannot be modified retroactively"
    affected_classes: [Transaction, AuditLog]
    validation: "Ensure no DELETE operations on historical data"

  - id: FIN-003
    name: "Regulatory Reporting"
    rule: "All transactions must have reporting_category property"
    affected_classes: [Transaction]
    validation: "SHACL shape requires reporting_category"
```

**Healthcare Sector Example**:
```yaml
doctrines:
  - id: HEALTH-001
    name: "HIPAA Privacy"
    rule: "Patient data requires encryption and access logging"
    affected_classes: [Patient, MedicalRecord]
    validation: "Check for encryption_required=true and audit_access=true"

  - id: HEALTH-002
    name: "Clinical Protocol Validation"
    rule: "Treatment protocols require clinical review before deployment"
    affected_classes: [TreatmentProtocol, Medication]
    validation: "Check for clinical_review_status=approved"
```

**LLM Interaction**:
- Doctrines are injected into LLM prompt as constraints
- LLM must generate proposals that satisfy all applicable doctrines
- Validation pipeline checks doctrine compliance post-generation

### 2.3 Schema Constraints

All proposals must conform to the **meta-ontology (Σ²)**:

- All classes must be subclasses of `owl:Thing` or existing domain classes
- All properties must have declared `rdfs:domain` and `rdfs:range`
- No orphaned nodes (all entities must be reachable from root ontology)
- Namespace conventions must be followed (e.g., `knhk:`, `finance:`, `health:`)

### 2.4 Performance Budget Constraints

Proposals must not exceed allocated computational budgets:

| Operation | Budget | Enforcement |
|-----------|--------|-------------|
| Hot path execution | ≤8 ticks | Static analysis of C codegen |
| Validation pipeline | ≤100ms | Timeout during validation |
| Code generation | ≤1s | Timeout during codegen |
| Snapshot promotion | ≤1ns | Atomic operation guarantee |

**LLM Guidance**: Prompt includes estimated tick counts for different operation types to guide proposal scope.

### 2.5 Guard Constraints (Immutable Boundaries)

Guard profiles define **immutable** system boundaries that proposals cannot modify:

```rust
struct GuardProfile {
    id: String,
    name: String,
    protected_classes: Vec<String>,    // Cannot be removed
    protected_properties: Vec<String>, // Cannot be removed
    max_run_len: usize,               // Cannot exceed (default: 8)
    performance_tier: PerformanceTier, // Cannot downgrade
}

enum PerformanceTier {
    HotPath,   // ≤8 ticks (branchless C)
    WarmPath,  // ≤1ms (Rust)
    ColdPath,  // ≤100ms (Python/validation)
}
```

**Example Guard Profile (Finance)**:
```yaml
guard_profile:
  id: "FINANCE_CORE_GUARD"
  protected_classes:
    - Account
    - Transaction
    - ApprovalChain
  protected_properties:
    - account_id
    - transaction_id
    - timestamp
  max_run_len: 8
  performance_tier: HotPath
```

**LLM Constraint**: Cannot propose removal of protected classes/properties, cannot propose changes that exceed `max_run_len`.

### 2.6 Immutability Constraints

- **No Retroactive Changes**: Proposals cannot modify historical data (Q1 enforcement)
- **Snapshot Immutability**: Once a snapshot is promoted, it cannot be mutated (only superseded)
- **Receipt Immutability**: Validation receipts are append-only, cryptographically signed

---

## 3. LLM Prompt Engineering for Constraints

### 3.1 Constraint Encoding Strategy

The prompt must **explicitly encode** all constraints so the LLM understands boundaries:

```
You are an ontology evolution assistant for the KNHK knowledge graph system.
Your task is to propose ontology changes (ΔΣ) based on observed patterns.

CRITICAL CONSTRAINTS (MUST NEVER VIOLATE):

1. Hard Invariants (Q1-Q5):
   - Q1: No temporal cycles (time flows forward only)
   - Q2: All properties must have valid domain/range types
   - Q3: Hot path operations must have ≤8 execution steps
   - Q4: Hot path execution time must be ≤8 CPU ticks
   - Q5: No performance regression >10% on existing benchmarks

2. Doctrines (Sector: {sector}):
   {doctrine_list}

3. Guard Constraints:
   - Protected classes (CANNOT REMOVE): {protected_classes}
   - Protected properties (CANNOT REMOVE): {protected_properties}
   - Maximum run length: {max_run_len} steps

4. Performance Budget:
   - Adding a new class: ~1 tick overhead
   - Adding a new property: ~0.5 ticks overhead
   - Adding a nested validation: ~3 ticks overhead
   - TOTAL BUDGET FOR THIS PROPOSAL: {remaining_ticks} ticks

OBSERVED PATTERN:
{pattern_description}

CURRENT ONTOLOGY SNAPSHOT:
{relevant_ontology_excerpt}

TASK:
Propose a minimal ontology change (ΔΣ) that addresses the observed pattern
while satisfying ALL constraints above.

OUTPUT FORMAT (JSON):
{
  "reasoning": "Why this change is needed and how it satisfies constraints",
  "confidence": 0.0-1.0,
  "estimated_ticks": integer,
  "delta_sigma": {
    "added_classes": [
      {"uri": "knhk:NewClass", "label": "...", "subclass_of": "..."}
    ],
    "added_properties": [
      {"uri": "knhk:newProperty", "domain": "...", "range": "..."}
    ],
    "removed_classes": [],
    "removed_properties": []
  },
  "doctrines_satisfied": ["FIN-001", "FIN-003"],
  "invariants_checked": ["Q1", "Q2", "Q3", "Q4"]
}

EXAMPLES:
{few_shot_examples}
```

### 3.2 Few-Shot Examples

#### Good Proposal Example (Finance)

**Pattern**: "Observed 15 different account types, current ontology only has 3 generic types"

**LLM Output**:
```json
{
  "reasoning": "The finance sector requires specialized account types for regulatory compliance. Adding RetirementAccount and HSAAccount as subclasses of Account satisfies the observed need while maintaining approval chain requirements (FIN-001) and audit trails (FIN-002). Each new class adds only 0.5 ticks overhead, well within the 8-tick budget.",
  "confidence": 0.85,
  "estimated_ticks": 1,
  "delta_sigma": {
    "added_classes": [
      {
        "uri": "finance:RetirementAccount",
        "label": "Retirement Account (401k, IRA)",
        "subclass_of": "finance:Account",
        "properties_required": ["account_id", "owner", "tax_status"]
      },
      {
        "uri": "finance:HSAAccount",
        "label": "Health Savings Account",
        "subclass_of": "finance:Account",
        "properties_required": ["account_id", "owner", "medical_purpose"]
      }
    ],
    "added_properties": [
      {
        "uri": "finance:tax_status",
        "label": "Tax Treatment Status",
        "domain": "finance:RetirementAccount",
        "range": "xsd:string",
        "required": true
      }
    ],
    "removed_classes": [],
    "removed_properties": []
  },
  "doctrines_satisfied": ["FIN-001", "FIN-002", "FIN-003"],
  "invariants_checked": ["Q1", "Q2", "Q3", "Q4"]
}
```

**Validation Result**: ✅ PASS (all constraints satisfied)

#### Bad Proposal Example (Violates Q3)

**Pattern**: "Need more detailed transaction validation"

**LLM Output**:
```json
{
  "reasoning": "Add comprehensive transaction validation with 10-step verification process",
  "confidence": 0.7,
  "estimated_ticks": 12,  // ❌ EXCEEDS 8-TICK LIMIT
  "delta_sigma": {
    "added_properties": [
      {"uri": "finance:validation_step_1", ...},
      {"uri": "finance:validation_step_2", ...},
      // ... 10 properties total
    ]
  }
}
```

**Validation Result**: ❌ FAIL (violates Q3: max_run_len ≤ 8)

**Feedback to LLM**: "Proposal rejected: exceeds performance budget (12 ticks > 8 ticks). Consider batching validations or moving to warm path."

#### Bad Proposal Example (Violates Doctrine)

**Pattern**: "Streamline account creation process"

**LLM Output**:
```json
{
  "reasoning": "Remove approval requirement to speed up account creation",
  "confidence": 0.6,
  "delta_sigma": {
    "removed_properties": ["finance:approval_required"]  // ❌ VIOLATES FIN-001
  }
}
```

**Validation Result**: ❌ FAIL (violates doctrine FIN-001: "All account modifications require two-step approval")

**Feedback to LLM**: "Proposal rejected: violates doctrine FIN-001. Approval chains are mandatory for regulatory compliance."

### 3.3 Prompt Adaptation by Sector

Different sectors require different prompt strategies:

**Finance**: Emphasize regulatory compliance, audit trails, approval workflows
**Healthcare**: Emphasize patient safety, privacy (HIPAA), clinical validation
**Manufacturing**: Emphasize safety interlocks, quality standards, equipment certification
**Logistics**: Emphasize delivery SLAs, route optimization, inventory accuracy

---

## 4. Proposal Format

### 4.1 Core Data Structures

```rust
/// A pattern detected in the observation plane that may require ontology evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub id: PatternId,
    pub description: String,
    pub sector: Sector,
    pub confidence: f64,  // 0.0-1.0
    pub observations: Vec<ObservationId>,
    pub timestamp: DateTime<Utc>,
}

/// Request to LLM to propose an ontology change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalRequest {
    pub pattern: DetectedPattern,
    pub current_snapshot_id: SnapshotId,
    pub doctrines: Vec<DoctrineRule>,
    pub invariants: HardInvariants,  // Q1-Q5 status
    pub guard_constraints: GuardProfile,
    pub performance_budget: PerformanceBudget,
}

/// LLM-generated proposal for ontology change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub pattern_id: PatternId,
    pub llm_prompt: String,              // Full prompt sent to LLM
    pub llm_response: String,            // Raw LLM output
    pub delta_sigma: SigmaDiff,          // Parsed ontology change
    pub reasoning: String,               // LLM's explanation
    pub confidence: f64,                 // LLM's confidence score
    pub estimated_ticks: u32,            // Predicted execution time
    pub doctrines_satisfied: Vec<String>, // Doctrine IDs claimed
    pub invariants_satisfied: Vec<String>, // Q1-Q5 claimed
    pub can_rollback: bool,              // Is this reversible?
    pub timestamp: DateTime<Utc>,
}

/// Ontology change specification (ΔΣ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaDiff {
    pub added_classes: Vec<ClassDefinition>,
    pub removed_classes: Vec<String>,  // URIs
    pub added_properties: Vec<PropertyDefinition>,
    pub removed_properties: Vec<String>,  // URIs
    pub modified_shapes: Vec<ShapeDefinition>,  // SHACL updates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub uri: String,
    pub label: String,
    pub subclass_of: String,
    pub properties_required: Vec<String>,
    pub properties_optional: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub uri: String,
    pub label: String,
    pub domain: String,  // Class URI
    pub range: String,   // Datatype or Class URI
    pub required: bool,
    pub cardinality: Cardinality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cardinality {
    One,           // Exactly 1
    ZeroOrOne,     // Optional
    ZeroOrMore,    // List
    OneOrMore,     // Non-empty list
}
```

### 4.2 Performance Budget Tracking

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    pub max_ticks: u32,           // e.g., 8 for hot path
    pub consumed_ticks: u32,      // Already used by existing ontology
    pub remaining_ticks: u32,     // Available for new proposal
    pub cost_per_class: f64,      // ~1 tick per class
    pub cost_per_property: f64,   // ~0.5 ticks per property
    pub cost_per_validation: f64, // ~3 ticks per SHACL validation
}

impl PerformanceBudget {
    pub fn estimate_cost(&self, diff: &SigmaDiff) -> u32 {
        let class_cost = (diff.added_classes.len() as f64 * self.cost_per_class).ceil() as u32;
        let prop_cost = (diff.added_properties.len() as f64 * self.cost_per_property).ceil() as u32;
        let validation_cost = (diff.modified_shapes.len() as f64 * self.cost_per_validation).ceil() as u32;

        class_cost + prop_cost + validation_cost
    }

    pub fn can_afford(&self, diff: &SigmaDiff) -> bool {
        self.estimate_cost(diff) <= self.remaining_ticks
    }
}
```

---

## 5. Constraint Enforcement Strategies

### 5.1 Strategy A: Prompt Engineering (Baseline)

**Approach**: Encode all constraints in the prompt and rely on LLM to follow instructions.

**Implementation**:
```rust
async fn generate_proposal_prompt_based(
    &self,
    request: &ProposalRequest,
) -> Result<Proposal> {
    let prompt = self.build_constrained_prompt(request)?;
    let llm_response = self.llm_client.complete(&prompt).await?;
    let proposal = self.parse_proposal_response(&llm_response)?;
    Ok(proposal)
}

fn build_constrained_prompt(&self, request: &ProposalRequest) -> String {
    format!(
        "HARD INVARIANTS:\n{}\n\nDOCTRINES:\n{}\n\nGUARD CONSTRAINTS:\n{}\n\nPATTERN:\n{}\n\nPropose ΔΣ:",
        format_invariants(&request.invariants),
        format_doctrines(&request.doctrines),
        format_guards(&request.guard_constraints),
        request.pattern.description
    )
}
```

**Pros**:
- Simple to implement
- Works with any LLM API
- Fast (single LLM call)
- Flexible (easy to update constraints)

**Cons**:
- LLM can still violate constraints (hallucination)
- No hard guarantees
- Requires post-hoc validation
- Waste of compute if proposal is invalid

**Use Cases**: Initial prototyping, non-critical proposals, high-confidence LLMs

### 5.2 Strategy B: Constrained Generation (Strong Guarantees)

**Approach**: Use guided decoding (e.g., LMQL, Guidance) to force LLM to generate tokens that satisfy constraints.

**Implementation** (Pseudocode with LMQL):
```python
import lmql

@lmql.query
async def generate_constrained_proposal(
    pattern: str,
    doctrines: List[str],
    invariants: Dict[str, bool],
    max_ticks: int
):
    '''lmql
    """PATTERN: {pattern}

    CONSTRAINTS:
    - Invariants: {invariants}
    - Doctrines: {doctrines}
    - Max ticks: {max_ticks}

    Propose ΔΣ:
    {
        "added_classes": [CLASSES],
        "estimated_ticks": [TICKS where TICKS <= max_ticks]
    }
    """
    '''
```

**Pros**:
- Strong guarantees (constraints enforced at token level)
- Reduces invalid proposals
- Can enforce schema structure

**Cons**:
- Slower (constrained decoding overhead)
- More complex implementation
- Requires specialized libraries (LMQL, Guidance)
- Limited to constraints expressible as token masks

**Use Cases**: Critical proposals, production deployments, high-stakes sectors (healthcare, finance)

### 5.3 Strategy C: Post-hoc Filtering (Safety Net)

**Approach**: Generate proposals freely, then filter out any that violate constraints.

**Implementation**:
```rust
async fn generate_proposal_with_filtering(
    &self,
    request: &ProposalRequest,
) -> Result<Proposal> {
    // Generate multiple proposals
    let mut proposals = vec![];
    for _ in 0..5 {  // Generate 5 candidates
        let prompt = self.build_prompt(request)?;
        let response = self.llm_client.complete(&prompt).await?;
        if let Ok(proposal) = self.parse_proposal(&response) {
            proposals.push(proposal);
        }
    }

    // Filter by constraints
    let valid_proposals: Vec<_> = proposals
        .into_iter()
        .filter(|p| self.validate_constraints(p, request).is_ok())
        .collect();

    // Return highest confidence valid proposal
    valid_proposals
        .into_iter()
        .max_by_key(|p| (p.confidence * 1000.0) as u64)
        .ok_or_else(|| anyhow!("No valid proposals generated"))
}
```

**Pros**:
- Simple to implement
- Works with any LLM
- Can catch all violations
- No changes to LLM pipeline

**Cons**:
- Wasteful (generates invalid proposals)
- No guarantee of finding valid proposal
- Slow (multiple LLM calls)
- May reject good proposals due to parse errors

**Use Cases**: Exploration, research, low-stakes prototyping

### 5.4 Strategy D: Hybrid (Recommended)

**Approach**: Combine all three strategies for defense-in-depth.

**Implementation**:
```rust
async fn generate_proposal_hybrid(
    &self,
    request: &ProposalRequest,
) -> Result<Proposal> {
    // LAYER 1: Prompt-based constraints (Strategy A)
    let prompt = self.build_constrained_prompt(request)?;

    // LAYER 2: Constrained generation for critical invariants (Strategy B)
    // Use LMQL to enforce Q3 (max_run_len) and Q4 (max_ticks) at token level
    let llm_response = if request.guard_constraints.performance_tier == PerformanceTier::HotPath {
        self.llm_client_constrained
            .complete_with_bounds(&prompt, request.performance_budget.max_ticks)
            .await?
    } else {
        self.llm_client.complete(&prompt).await?
    };

    let proposal = self.parse_proposal_response(&llm_response)?;

    // LAYER 3: Post-hoc validation (Strategy C)
    self.validate_all_constraints(&proposal, request)?;

    Ok(proposal)
}

fn validate_all_constraints(
    &self,
    proposal: &Proposal,
    request: &ProposalRequest,
) -> Result<()> {
    // Static checks
    self.validate_schema(&proposal.delta_sigma)?;

    // Invariant checks
    for invariant in ["Q1", "Q2", "Q3", "Q4", "Q5"] {
        self.check_invariant(invariant, proposal, request)?;
    }

    // Doctrine checks
    for doctrine in &request.doctrines {
        self.check_doctrine(doctrine, proposal)?;
    }

    // Guard checks
    self.check_guard_constraints(&proposal.delta_sigma, &request.guard_constraints)?;

    // Performance check
    let estimated_cost = request.performance_budget.estimate_cost(&proposal.delta_sigma);
    ensure!(
        estimated_cost <= request.performance_budget.remaining_ticks,
        "Performance budget exceeded: {} > {}",
        estimated_cost,
        request.performance_budget.remaining_ticks
    );

    Ok(())
}
```

**Pros**:
- Best of all strategies
- Defense-in-depth
- High confidence in outputs
- Adapts to criticality (hot path vs warm path)

**Cons**:
- Most complex to implement
- Slower than Strategy A alone
- Requires multiple libraries/tools

**Recommendation**: Use hybrid strategy for production deployment.

---

## 6. Validation Pipeline

The validation pipeline runs **after** LLM generation and **before** promotion to snapshot.

### 6.1 Seven-Stage Validation

```rust
pub struct ValidationPipeline {
    schema_validator: SchemaValidator,
    invariant_checkers: Vec<Box<dyn InvariantChecker>>,
    doctrine_validator: DoctrineValidator,
    guard_validator: GuardValidator,
    performance_estimator: PerformanceEstimator,
    rollback_analyzer: RollbackAnalyzer,
    compatibility_checker: CompatibilityChecker,
}

impl ValidationPipeline {
    pub async fn validate(&self, proposal: &Proposal) -> Result<ValidationReport> {
        let mut report = ValidationReport::new(proposal.id.clone());

        // STAGE 1: Static Schema Check
        // Does the ΔΣ conform to meta-ontology (Σ²)?
        match self.schema_validator.validate(&proposal.delta_sigma) {
            Ok(_) => report.add_pass("static_check"),
            Err(e) => {
                report.add_fail("static_check", e.to_string());
                return Ok(report);  // Fail fast
            }
        }

        // STAGE 2: Invariant Check (Q1-Q5)
        // Do hard invariants still hold after applying ΔΣ?
        for checker in &self.invariant_checkers {
            match checker.check(proposal) {
                Ok(_) => report.add_pass(&format!("invariant_{}", checker.name())),
                Err(e) => {
                    report.add_fail(&format!("invariant_{}", checker.name()), e.to_string());
                    return Ok(report);  // Fail fast
                }
            }
        }

        // STAGE 3: Doctrine Check
        // Do all applicable doctrines hold?
        match self.doctrine_validator.validate(proposal) {
            Ok(_) => report.add_pass("doctrine_check"),
            Err(e) => {
                report.add_fail("doctrine_check", e.to_string());
                return Ok(report);  // Fail fast
            }
        }

        // STAGE 4: Guard Check
        // Are all guard boundaries preserved?
        match self.guard_validator.validate(proposal) {
            Ok(_) => report.add_pass("guard_check"),
            Err(e) => {
                report.add_fail("guard_check", e.to_string());
                return Ok(report);
            }
        }

        // STAGE 5: Performance Check
        // Is estimated execution time within budget?
        let estimated_ticks = self.performance_estimator.estimate(&proposal.delta_sigma)?;
        if estimated_ticks <= 8 {
            report.add_pass("performance_check");
        } else {
            report.add_fail(
                "performance_check",
                format!("Estimated {} ticks, max is 8", estimated_ticks)
            );
            return Ok(report);
        }

        // STAGE 6: Rollback Check
        // Can we safely revert this change if needed?
        match self.rollback_analyzer.analyze(&proposal.delta_sigma) {
            Ok(true) => report.add_pass("rollback_check"),
            Ok(false) => {
                report.add_fail("rollback_check", "Change is not reversible".to_string());
            }
            Err(e) => {
                report.add_fail("rollback_check", e.to_string());
            }
        }

        // STAGE 7: Compatibility Check
        // Is this backward compatible with existing data?
        match self.compatibility_checker.check(&proposal.delta_sigma).await {
            Ok(_) => report.add_pass("compatibility_check"),
            Err(e) => {
                report.add_fail("compatibility_check", e.to_string());
            }
        }

        Ok(report)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub proposal_id: ProposalId,
    pub passed: bool,
    pub stages: Vec<ValidationStage>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStage {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}
```

### 6.2 Invariant Checker Implementations

Each hard invariant (Q1-Q5) has a dedicated checker:

```rust
/// Q1: No Retrocausation - Time flows forward only
pub struct Q1NoRetrocausationChecker;

impl InvariantChecker for Q1NoRetrocausationChecker {
    fn name(&self) -> &str { "Q1" }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Check for temporal cycles in causal relationships
        let mut graph = CausalGraph::new();

        // Add existing ontology edges
        graph.load_from_snapshot(&proposal.current_snapshot)?;

        // Add proposed edges
        for prop in &proposal.delta_sigma.added_properties {
            if is_causal_property(prop) {
                graph.add_edge(&prop.domain, &prop.range);
            }
        }

        // Detect cycles
        if let Some(cycle) = graph.find_cycle() {
            bail!("Q1 violation: temporal cycle detected: {:?}", cycle);
        }

        Ok(())
    }
}

/// Q3: Guard Preservation - max_run_len ≤ 8
pub struct Q3GuardPreservationChecker;

impl InvariantChecker for Q3GuardPreservationChecker {
    fn name(&self) -> &str { "Q3" }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Estimate execution steps for hot path after applying ΔΣ
        let current_steps = estimate_hot_path_steps(&proposal.current_snapshot)?;
        let delta_steps = estimate_delta_steps(&proposal.delta_sigma)?;
        let total_steps = current_steps + delta_steps;

        ensure!(
            total_steps <= 8,
            "Q3 violation: hot path would require {} steps (max 8)",
            total_steps
        );

        Ok(())
    }
}

/// Q5: Performance Bounds - no regression >10%
pub struct Q5PerformanceBoundsChecker {
    benchmark_runner: Arc<BenchmarkRunner>,
}

impl InvariantChecker for Q5PerformanceBoundsChecker {
    fn name(&self) -> &str { "Q5" }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // This requires actual benchmark execution, so we estimate here
        // and validate post-promotion with rollback capability
        let estimated_regression = estimate_performance_impact(&proposal.delta_sigma)?;

        ensure!(
            estimated_regression <= 0.10,
            "Q5 violation: estimated performance regression {}% (max 10%)",
            (estimated_regression * 100.0)
        );

        // TODO: Post-promotion validation with actual benchmarks
        Ok(())
    }
}
```

---

## 7. Learning from Feedback

### 7.1 Feedback Loop Architecture

```rust
pub struct ProposalLearningSystem {
    corpus: ProposalCorpus,
    prompt_adapter: PromptAdapter,
    pattern_analyzer: PatternAnalyzer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalCorpus {
    pub accepted_proposals: Vec<(Proposal, ValidationReport)>,
    pub rejected_proposals: Vec<(Proposal, ValidationReport)>,
    pub constraint_violations: HashMap<String, Vec<ProposalId>>,  // Which proposals violated which constraints
}

impl ProposalLearningSystem {
    /// Record a proposal outcome (accepted or rejected)
    pub fn record_outcome(
        &mut self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()> {
        if report.passed {
            self.corpus.accepted_proposals.push((proposal.clone(), report));
            tracing::info!(proposal_id = %proposal.id, "Accepted proposal recorded for learning");
        } else {
            self.corpus.rejected_proposals.push((proposal.clone(), report.clone()));

            // Track which constraints were violated
            for stage in &report.stages {
                if !stage.passed {
                    self.corpus
                        .constraint_violations
                        .entry(stage.name.clone())
                        .or_default()
                        .push(proposal.id.clone());
                }
            }

            tracing::warn!(
                proposal_id = %proposal.id,
                violations = ?report.stages.iter().filter(|s| !s.passed).map(|s| &s.name).collect::<Vec<_>>(),
                "Rejected proposal recorded for learning"
            );
        }

        // Adapt prompts based on patterns
        self.adapt_prompts_from_feedback()?;

        Ok(())
    }

    /// Adapt prompts based on acceptance/rejection patterns
    fn adapt_prompts_from_feedback(&mut self) -> Result<()> {
        // Analyze common violation patterns
        let violation_analysis = self.pattern_analyzer.analyze_violations(&self.corpus)?;

        // If Q3 is frequently violated, emphasize performance budget more strongly
        if violation_analysis.frequent_violations.contains(&"Q3".to_string()) {
            self.prompt_adapter.increase_emphasis("performance_budget");
            tracing::info!("Increasing prompt emphasis on performance budget due to frequent Q3 violations");
        }

        // If doctrine violations are common, add more few-shot examples
        if violation_analysis.frequent_violations.contains(&"doctrine_check".to_string()) {
            self.prompt_adapter.add_doctrine_examples(&self.corpus.accepted_proposals);
            tracing::info!("Adding doctrine-compliant examples to few-shot set");
        }

        Ok(())
    }

    /// Generate few-shot examples from corpus
    pub fn get_few_shot_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample> {
        self.corpus
            .accepted_proposals
            .iter()
            .filter(|(p, _)| p.pattern.sector == *sector)
            .take(count)
            .map(|(p, r)| FewShotExample {
                pattern: p.pattern.description.clone(),
                proposal: p.delta_sigma.clone(),
                reasoning: p.reasoning.clone(),
                validation_result: r.clone(),
            })
            .collect()
    }
}
```

### 7.2 Prompt Adaptation Strategies

```rust
pub struct PromptAdapter {
    base_template: String,
    emphasis_weights: HashMap<String, f64>,  // Which sections to emphasize
    few_shot_examples: HashMap<Sector, Vec<FewShotExample>>,
}

impl PromptAdapter {
    pub fn build_prompt(&self, request: &ProposalRequest) -> String {
        let mut prompt = self.base_template.clone();

        // Adjust emphasis based on learned weights
        if self.emphasis_weights.get("performance_budget").unwrap_or(&1.0) > &1.5 {
            prompt = prompt.replace(
                "PERFORMANCE BUDGET:",
                "⚠️ CRITICAL PERFORMANCE BUDGET (FREQUENTLY VIOLATED):"
            );
        }

        // Add sector-specific few-shot examples
        let examples = self.few_shot_examples
            .get(&request.pattern.sector)
            .cloned()
            .unwrap_or_default();

        let example_text = examples
            .iter()
            .map(|ex| format!(
                "EXAMPLE:\nPattern: {}\nProposal: {:?}\nResult: ✅ ACCEPTED\n",
                ex.pattern, ex.proposal
            ))
            .collect::<Vec<_>>()
            .join("\n");

        prompt = prompt.replace("{few_shot_examples}", &example_text);

        prompt
    }

    pub fn increase_emphasis(&mut self, section: &str) {
        *self.emphasis_weights.entry(section.to_string()).or_insert(1.0) += 0.5;
    }
}
```

### 7.3 Continuous Improvement Metrics

Track these metrics to measure learning effectiveness:

| Metric | Target | Description |
|--------|--------|-------------|
| Acceptance Rate | >70% | Percentage of proposals that pass validation |
| First-Try Acceptance | >50% | Proposals that pass without needing regeneration |
| Q3 Violation Rate | <10% | Proposals rejected due to performance budget |
| Doctrine Violation Rate | <15% | Proposals rejected due to doctrine violations |
| Average Confidence (Accepted) | >0.75 | LLM confidence for accepted proposals |
| Average Confidence (Rejected) | <0.60 | LLM confidence for rejected proposals (should be lower) |

---

## 8. Multi-Constraint Interaction

### 8.1 Constraint Conflicts

Some proposals may satisfy individual constraints but violate combinations:

**Example Conflict**: Finance sector

- **Doctrine FIN-001**: "All account modifications require approval"
- **Doctrine FIN-004**: "Real-time fraud detection must respond in <100ms"
- **Proposal**: Add new fraud detection property to Account class

**Conflict**: Adding approval requirement to fraud detection violates the 100ms SLA (approval requires human interaction, taking seconds/minutes).

**Resolution Strategies**:

#### Strategy 1: Explicit Ranking
```yaml
doctrine_priority:
  - id: FIN-004  # Real-time fraud detection (highest priority)
    rank: 1
  - id: FIN-001  # Approval chains (lower priority)
    rank: 2

resolution: "When FIN-004 and FIN-001 conflict, FIN-004 takes precedence"
```

#### Strategy 2: Negotiation
```rust
pub struct ConflictResolver {
    negotiation_strategies: Vec<Box<dyn NegotiationStrategy>>,
}

impl ConflictResolver {
    pub fn resolve(&self, conflicts: Vec<DoctrineConflict>) -> Result<Resolution> {
        for strategy in &self.negotiation_strategies {
            if let Some(resolution) = strategy.attempt_resolution(&conflicts)? {
                return Ok(resolution);
            }
        }
        Err(anyhow!("Could not resolve doctrine conflicts"))
    }
}

// Example: Compromise strategy
pub struct CompromiseStrategy;

impl NegotiationStrategy for CompromiseStrategy {
    fn attempt_resolution(&self, conflicts: &[DoctrineConflict]) -> Result<Option<Resolution>> {
        // For FIN-001 vs FIN-004 conflict:
        // - Apply approval requirement to batch operations (not real-time)
        // - Apply fraud detection to all operations (including real-time)
        // - Result: Real-time ops get fraud detection without approval (satisfies FIN-004)
        //           Batch ops get both fraud detection AND approval (satisfies FIN-001)

        Ok(Some(Resolution {
            approach: "Split operation types: real-time exempt from approval".to_string(),
            modified_proposal: /* ... */,
        }))
    }
}
```

#### Strategy 3: Rejection
```rust
// If no resolution possible, reject proposal with clear explanation
ValidationReport {
    passed: false,
    stages: vec![
        ValidationStage {
            name: "doctrine_conflict".to_string(),
            passed: false,
            message: Some(
                "Cannot satisfy both FIN-001 (approval required) and FIN-004 (real-time SLA). \
                 Consider splitting into two proposals: (1) real-time fraud detection exempt from approval, \
                 (2) batch fraud detection with approval."
            ),
        }
    ],
}
```

### 8.2 Constraint Composition

Some constraints must be satisfied **together**, not independently:

**Example**: Healthcare sector

- **Q2**: Type soundness (all properties have valid types)
- **HEALTH-001**: Patient data must be encrypted
- **HEALTH-002**: Treatment protocols require clinical review

**Composed Constraint**: Any proposal that adds a new patient property must:
1. Have valid domain/range (Q2)
2. Mark encryption_required=true (HEALTH-001)
3. NOT modify treatment protocols without clinical_review_status (HEALTH-002)

```rust
pub struct ComposedConstraint {
    pub id: String,
    pub name: String,
    pub component_constraints: Vec<String>,  // Q2, HEALTH-001, HEALTH-002
    pub composition_rule: CompositionRule,
}

pub enum CompositionRule {
    AllOf(Vec<String>),        // All constraints must hold
    AnyOf(Vec<String>),        // At least one must hold
    ExactlyOneOf(Vec<String>), // Exactly one must hold
    IfThen { if_constraint: String, then_constraint: String },  // Conditional
}

impl ComposedConstraint {
    pub fn validate(&self, proposal: &Proposal) -> Result<()> {
        match &self.composition_rule {
            CompositionRule::AllOf(constraints) => {
                for constraint_id in constraints {
                    self.validate_single(constraint_id, proposal)?;
                }
                Ok(())
            }
            CompositionRule::IfThen { if_constraint, then_constraint } => {
                if self.is_satisfied(if_constraint, proposal) {
                    self.validate_single(then_constraint, proposal)?;
                }
                Ok(())
            }
            // ... other rules
        }
    }
}
```

---

## 9. Sector-Specific Prompting

### 9.1 Finance Sector

**Key Concerns**: Regulatory compliance, audit trails, approval chains, fraud prevention

**Prompt Additions**:
```
FINANCE SECTOR REQUIREMENTS:

1. All financial data must have audit trails (append-only, timestamped)
2. Account modifications require two-step approval (FIN-001)
3. Transaction history is immutable (cannot delete/modify)
4. All transactions must have reporting_category for regulatory compliance
5. Real-time fraud detection must complete in <100ms

COMMON PATTERNS:
- New account types (e.g., Retirement, HSA, Trust)
- Payment methods (e.g., ACH, Wire, Crypto)
- Risk assessment categories
- Compliance reporting fields

AVOID:
- Removing approval requirements
- Modifying historical transaction data
- Adding slow validations to hot path
```

### 9.2 Healthcare Sector

**Key Concerns**: Patient safety, HIPAA privacy, clinical validation, medical accuracy

**Prompt Additions**:
```
HEALTHCARE SECTOR REQUIREMENTS:

1. All patient data requires encryption (HEALTH-001)
2. Treatment protocols require clinical review before deployment (HEALTH-002)
3. Medical records must preserve version history (no destructive updates)
4. All diagnoses must reference ICD-10 codes
5. Medication data must include contraindications and side effects

COMMON PATTERNS:
- New treatment protocols (e.g., updated diabetes care guidelines)
- Medication formulations (e.g., new dosage forms)
- Diagnostic criteria updates (based on clinical research)
- Patient consent management

AVOID:
- Deploying clinical changes without review
- Removing patient safety checks
- Breaking HIPAA compliance
- Modifying medical records retroactively
```

### 9.3 Manufacturing Sector

**Key Concerns**: Equipment safety, quality standards, regulatory certification, maintenance schedules

**Prompt Additions**:
```
MANUFACTURING SECTOR REQUIREMENTS:

1. All equipment modifications require safety certification
2. Quality control checks must maintain audit trails
3. Maintenance schedules are immutable once equipment is in production
4. All changes must preserve equipment safety interlocks
5. Production line changes require simulation validation

COMMON PATTERNS:
- New equipment types (e.g., robotic assembly, quality sensors)
- Maintenance procedures (e.g., preventive schedules)
- Quality metrics (e.g., tolerance thresholds)
- Safety protocols (e.g., emergency shutdowns)

AVOID:
- Removing safety interlocks
- Modifying production equipment without certification
- Breaking quality control procedures
```

### 9.4 Logistics Sector

**Key Concerns**: Delivery SLAs, route optimization, inventory accuracy, real-time tracking

**Prompt Additions**:
```
LOGISTICS SECTOR REQUIREMENTS:

1. Delivery SLAs must be preserved (cannot relax timing constraints)
2. Route optimization must complete in <1 second for real-time decisions
3. Inventory counts must be reconciled daily (no silent failures)
4. All shipments must have tracking identifiers
5. Warehouse operations must preserve FIFO/LIFO constraints

COMMON PATTERNS:
- New shipping methods (e.g., drone delivery, same-day)
- Route optimization constraints (e.g., vehicle capacity, traffic)
- Inventory tracking (e.g., perishables, hazardous materials)
- Delivery zones (e.g., urban, rural, international)

AVOID:
- Relaxing SLAs without customer approval
- Adding slow computations to route planning
- Breaking inventory reconciliation procedures
```

---

## 10. Example Proposal Workflows

### 10.1 Example 1: Finance New Account Type (Success)

**Pattern**:
```json
{
  "description": "Observed 15 different retirement account variations (401k, IRA, Roth IRA, SEP IRA) but ontology only has generic 'Account' class",
  "sector": "Finance",
  "confidence": 0.85,
  "observations": [/* 15 observation IDs */]
}
```

**LLM Prompt** (Abbreviated):
```
SECTOR: Finance
PATTERN: 15 retirement account types observed, ontology has 1 generic class

CONSTRAINTS:
- Q3: Hot path must stay ≤8 ticks (currently 6 ticks)
- FIN-001: All account modifications require approval
- FIN-003: All transactions need reporting_category
- Guard: Cannot remove Account, account_id, approval_required

PROPOSE ΔΣ:
```

**LLM Response**:
```json
{
  "reasoning": "Retirement accounts are a common financial product category with specific tax treatment and regulatory requirements. Adding RetirementAccount as a subclass of Account allows proper categorization without breaking approval chains or hot path performance. Each new class adds ~1 tick, so adding 1 class keeps us at 7 ticks (within budget).",
  "confidence": 0.82,
  "estimated_ticks": 7,
  "delta_sigma": {
    "added_classes": [
      {
        "uri": "finance:RetirementAccount",
        "label": "Retirement Account",
        "subclass_of": "finance:Account",
        "properties_required": ["account_id", "tax_status", "contribution_limit"]
      }
    ],
    "added_properties": [
      {
        "uri": "finance:tax_status",
        "label": "Tax Treatment Status",
        "domain": "finance:RetirementAccount",
        "range": "xsd:string",
        "required": true
      },
      {
        "uri": "finance:contribution_limit",
        "label": "Annual Contribution Limit",
        "domain": "finance:RetirementAccount",
        "range": "xsd:decimal",
        "required": true
      }
    ]
  },
  "doctrines_satisfied": ["FIN-001", "FIN-003"],
  "invariants_checked": ["Q1", "Q2", "Q3", "Q4"]
}
```

**Validation Pipeline**:
1. **Static Check**: ✅ PASS (RetirementAccount is proper subclass, properties have valid types)
2. **Invariant Q1**: ✅ PASS (no temporal cycles)
3. **Invariant Q2**: ✅ PASS (tax_status:string and contribution_limit:decimal are valid types)
4. **Invariant Q3**: ✅ PASS (estimated 7 ticks ≤ 8)
5. **Invariant Q4**: ✅ PASS (7 ticks within SLO)
6. **Invariant Q5**: ✅ PASS (estimated <5% regression)
7. **Doctrine Check**: ✅ PASS (approval_required inherited from Account, satisfies FIN-001)
8. **Guard Check**: ✅ PASS (no protected classes/properties removed)
9. **Performance Check**: ✅ PASS (7 ticks ≤ 8)
10. **Rollback Check**: ✅ PASS (can remove RetirementAccount without breaking existing data)
11. **Compatibility Check**: ✅ PASS (existing Account instances unaffected)

**Outcome**: ✅ **APPROVED** → Promote to snapshot → Generate C header → Deploy

**Learning**: Record as successful finance proposal in corpus for future few-shot examples

---

### 10.2 Example 2: Healthcare Protocol Change (Requires Review)

**Pattern**:
```json
{
  "description": "New clinical evidence published for diabetes treatment: SGLT2 inhibitors now recommended as first-line therapy",
  "sector": "Healthcare",
  "confidence": 0.78,
  "observations": [/* clinical study references */]
}
```

**LLM Prompt**:
```
SECTOR: Healthcare
PATTERN: New diabetes treatment evidence (SGLT2 inhibitors)

CONSTRAINTS:
- Q1: No retroactive changes to patient records
- HEALTH-002: Protocol changes require clinical review
- Guard: Cannot remove DiabetesTreatmentProtocol class

PROPOSE ΔΣ:
```

**LLM Response**:
```json
{
  "reasoning": "Clinical evidence supports updating diabetes treatment protocol to include SGLT2 inhibitors as first-line therapy. This adds a new medication class and updates the treatment protocol without modifying existing patient records.",
  "confidence": 0.75,
  "delta_sigma": {
    "added_classes": [
      {
        "uri": "health:SGLT2Inhibitor",
        "label": "SGLT2 Inhibitor Medication",
        "subclass_of": "health:Medication",
        "properties_required": ["drug_name", "dosage", "contraindications"]
      }
    ],
    "modified_shapes": [
      {
        "uri": "health:DiabetesTreatmentProtocol",
        "added_properties": ["first_line_therapy"],
        "property_values": {
          "first_line_therapy": "health:SGLT2Inhibitor"
        }
      }
    ]
  },
  "doctrines_satisfied": ["HEALTH-001"],
  "invariants_checked": ["Q1", "Q2"]
}
```

**Validation Pipeline**:
1. **Static Check**: ✅ PASS
2. **Invariant Q1**: ✅ PASS (no retroactive changes)
3. **Invariant Q2**: ✅ PASS (type soundness maintained)
4. **Doctrine Check**: ❌ **FAIL** (HEALTH-002 not satisfied: no clinical_review_status)

**Validation Report**:
```json
{
  "passed": false,
  "stages": [
    {"name": "static_check", "passed": true},
    {"name": "invariant_Q1", "passed": true},
    {"name": "invariant_Q2", "passed": true},
    {
      "name": "doctrine_check",
      "passed": false,
      "message": "HEALTH-002 violation: DiabetesTreatmentProtocol modification requires clinical_review_status=approved"
    }
  ]
}
```

**Outcome**: ❌ **REJECTED** with message: "Proposal requires clinical review. Recommend: (1) Submit protocol update to clinical review board, (2) Once approved, resubmit proposal with clinical_review_status=approved."

**Learning**:
- Record as rejected proposal due to HEALTH-002
- Increase prompt emphasis on clinical review requirement
- Add few-shot example showing proper clinical review workflow

---

### 10.3 Example 3: Manufacturing Safety Violation (Rejected)

**Pattern**:
```json
{
  "description": "Assembly line robot downtime is high due to excessive safety interlocks. Operators request faster emergency override.",
  "sector": "Manufacturing",
  "confidence": 0.65,
  "observations": [/* maintenance logs */]
}
```

**LLM Response**:
```json
{
  "reasoning": "To reduce downtime, add emergency override capability that bypasses safety interlocks for experienced operators.",
  "confidence": 0.60,
  "delta_sigma": {
    "added_properties": [
      {
        "uri": "mfg:emergency_override_enabled",
        "label": "Emergency Override",
        "domain": "mfg:RoboticAssembly",
        "range": "xsd:boolean"
      }
    ],
    "removed_properties": ["mfg:safety_interlock_required"]  // ❌ DANGER
  }
}
```

**Validation Pipeline**:
1. **Static Check**: ✅ PASS
2. **Guard Check**: ❌ **FAIL** (safety_interlock_required is a protected property)

**Outcome**: ❌ **REJECTED** with message: "CRITICAL SAFETY VIOLATION: Cannot remove safety_interlock_required (protected property). This would create unsafe operating conditions. Alternative: Add supervised override mode that logs all uses and requires manager approval."

**Learning**:
- Record as critical safety violation
- Add to training corpus as example of "what NOT to do"
- Increase prompt warnings about safety-critical properties

---

## 11. Safety Considerations

### 11.1 What LLM Must NOT Be Able To Do

1. **Remove Essential Classes/Properties**
   - Guard constraints prevent removal of core ontology elements
   - Validation pipeline checks against protected lists

2. **Create Circular Dependencies**
   - Q1 invariant (no retrocausation) enforced via cycle detection
   - Graph analysis before promotion

3. **Increase Ticks Significantly**
   - Q3/Q4 invariants enforce ≤8 tick budget
   - Performance estimator validates before promotion

4. **Bypass Approval Requirements**
   - Doctrine validation ensures approval chains preserved
   - Cannot remove approval-related properties

5. **Violate Segregation of Duties**
   - Cannot propose changes that merge conflicting roles
   - Doctrine enforcement at validation stage

### 11.2 Defense Mechanisms

**Layer 1: Prompt Engineering**
- Explicit warnings in prompts about forbidden operations
- Few-shot examples showing rejected dangerous proposals

**Layer 2: Constrained Generation**
- Use LMQL/Guidance to prevent generating forbidden tokens (e.g., "removed_classes": ["Account"])
- Token masking for protected property names

**Layer 3: Validation Pipeline**
- Multi-stage validation (static, invariant, doctrine, guard)
- Fail-fast on critical violations
- Human-in-the-loop for high-risk sectors (healthcare, manufacturing)

**Layer 4: Rollback Capability**
- All proposals must be reversible
- Snapshot versioning enables instant rollback
- Automated rollback if post-deployment metrics degrade

**Layer 5: Audit Trail**
- All proposals logged with cryptographic receipts
- Cannot delete or modify proposal history
- Regular security audits of proposal patterns

### 11.3 Human-in-the-Loop Requirements

For **critical sectors** (healthcare, manufacturing, finance), proposals must undergo human review before promotion:

```rust
pub enum PromotionPolicy {
    Automatic,           // Low-risk sectors (logistics, generic)
    HumanReview,         // Medium-risk (finance)
    ClinicalReview,      // High-risk (healthcare, manufacturing)
    MultiPartyApproval,  // Critical infrastructure
}

impl PromotionWorkflow {
    pub async fn promote_with_policy(
        &self,
        proposal: Proposal,
        policy: PromotionPolicy,
    ) -> Result<SnapshotId> {
        // Validate proposal
        let report = self.validation_pipeline.validate(&proposal).await?;
        ensure!(report.passed, "Proposal failed validation");

        // Apply promotion policy
        match policy {
            PromotionPolicy::Automatic => {
                self.promote_immediately(proposal).await
            }
            PromotionPolicy::HumanReview => {
                self.request_human_review(proposal).await?;
                self.await_approval().await?;
                self.promote_after_approval(proposal).await
            }
            PromotionPolicy::ClinicalReview => {
                self.request_clinical_review(proposal).await?;
                self.await_clinical_approval().await?;
                self.promote_after_approval(proposal).await
            }
            PromotionPolicy::MultiPartyApproval => {
                self.request_multi_party_approval(proposal).await?;
                self.await_quorum_approval(2, 3).await?;  // 2 of 3 approvers
                self.promote_after_approval(proposal).await
            }
        }
    }
}
```

---

## 12. API Design

### 12.1 Core Trait

```rust
#[async_trait]
pub trait LLMProposer: Send + Sync {
    /// Generate a proposal from an observed pattern
    async fn generate_proposal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &[GuardProfile],
    ) -> Result<Proposal>;

    /// Validate a proposal against constraints
    async fn validate_proposal(
        &self,
        proposal: &Proposal,
        doctrines: &[DoctrineRule],
    ) -> Result<ValidationReport>;

    /// Learn from proposal outcome (accepted or rejected)
    async fn record_outcome(
        &self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()>;

    /// Get few-shot examples for a sector
    fn get_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample>;
}
```

### 12.2 Implementation Example

```rust
pub struct OpenAILLMProposer {
    client: OpenAIClient,
    prompt_adapter: Arc<PromptAdapter>,
    learning_system: Arc<Mutex<ProposalLearningSystem>>,
    validation_pipeline: Arc<ValidationPipeline>,
}

#[async_trait]
impl LLMProposer for OpenAILLMProposer {
    async fn generate_proposal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &[GuardProfile],
    ) -> Result<Proposal> {
        // Build request
        let request = ProposalRequest {
            pattern: pattern.clone(),
            current_snapshot_id: self.get_current_snapshot_id()?,
            doctrines: doctrines.to_vec(),
            invariants: invariants.clone(),
            guard_constraints: guards[0].clone(),  // TODO: merge multiple guards
            performance_budget: self.calculate_performance_budget(guards)?,
        };

        // Build constrained prompt
        let prompt = self.prompt_adapter.build_prompt(&request);

        // Call LLM
        let response = self.client
            .chat_completion(&[
                ChatMessage::system("You are an ontology evolution assistant."),
                ChatMessage::user(&prompt),
            ])
            .await?;

        // Parse response
        let proposal = self.parse_proposal_response(&response, pattern)?;

        // Post-hoc validation (Strategy C)
        self.validate_all_constraints(&proposal, &request)?;

        Ok(proposal)
    }

    async fn validate_proposal(
        &self,
        proposal: &Proposal,
        doctrines: &[DoctrineRule],
    ) -> Result<ValidationReport> {
        self.validation_pipeline.validate(proposal).await
    }

    async fn record_outcome(
        &self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()> {
        let mut learning = self.learning_system.lock().await;
        learning.record_outcome(proposal, report)
    }

    fn get_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample> {
        let learning = self.learning_system.blocking_lock();
        learning.get_few_shot_examples(sector, count)
    }
}
```

### 12.3 CLI Integration

```bash
# Generate proposal from pattern
knhk ontology propose --pattern-id <pattern-id>

# Validate proposal
knhk ontology validate --proposal-id <proposal-id>

# Promote proposal (with human review for critical sectors)
knhk ontology promote --proposal-id <proposal-id> --require-review

# List proposals (pending, accepted, rejected)
knhk ontology proposals --status pending

# View proposal details
knhk ontology proposal <proposal-id>

# Rollback to previous snapshot
knhk ontology rollback
```

### 12.4 Rust API Example

```rust
// Initialize proposer
let proposer = OpenAILLMProposer::new(
    OpenAIClient::new(api_key),
    prompt_adapter,
    learning_system,
    validation_pipeline,
)?;

// Detect pattern (from observation plane)
let pattern = observation_plane.detect_pattern().await?;

// Load constraints
let doctrines = doctrine_store.get_for_sector(&pattern.sector)?;
let invariants = HardInvariants::default();  // Q1-Q5
let guards = guard_store.get_for_sector(&pattern.sector)?;

// Generate proposal
let proposal = proposer
    .generate_proposal(&pattern, &doctrines, &invariants, &guards)
    .await?;

// Validate
let report = proposer.validate_proposal(&proposal, &doctrines).await?;

if report.passed {
    // Promote to snapshot
    let new_snapshot_id = change_plane.promote_overlay(&proposal.delta_sigma)?;
    ontology_state.promote_snapshot(new_snapshot_id)?;

    // Record success
    proposer.record_outcome(proposal, report).await?;

    println!("✅ Proposal promoted to snapshot {}", new_snapshot_id);
} else {
    // Record failure
    proposer.record_outcome(proposal, report).await?;

    println!("❌ Proposal rejected: {:?}", report.stages);
}
```

---

## 13. Open Questions & Future Work

### 13.1 LLM Fine-Tuning (Phase 2)

**Question**: Should we fine-tune a domain-specific LLM on our proposal corpus?

**Options**:
1. Use general-purpose LLM (GPT-4, Claude) with prompt engineering (Phase 1)
2. Fine-tune open-source LLM (Llama 3, Mistral) on our corpus (Phase 2)
3. Hybrid: general LLM for exploration, fine-tuned for production (Phase 3)

**Tradeoffs**:
- Fine-tuning: Better constraint adherence, lower token costs, slower iteration
- General LLM: Faster iteration, more flexible, higher token costs

**Recommendation**: Start with general LLM (Phase 1), collect corpus, fine-tune in Phase 2 after 1000+ proposals.

### 13.2 Multi-Agent Proposal Generation (Phase 3)

**Question**: Should multiple LLM agents collaborate on proposals?

**Architecture**:
```
Proposer Agent → Critic Agent → Refinement Agent → Validator
     ↓              ↓                  ↓               ↓
  Generate      Identify          Fix issues      Final check
  ΔΣ draft      violations        and improve     before promotion
```

**Benefits**: Higher quality proposals, fewer rejections, collaborative reasoning

**Costs**: More LLM calls, increased latency, coordination complexity

**Recommendation**: Defer to Phase 3 after measuring single-agent performance.

### 13.3 Real-Time Streaming Proposals (Phase 4)

**Question**: Should proposals be generated in real-time as patterns are detected, or in batch?

**Current Design**: Batch mode (generate proposals periodically, e.g., daily)

**Alternative**: Streaming mode (generate proposal immediately upon pattern detection)

**Tradeoffs**:
- Batch: Lower cost, more comprehensive analysis, easier to review
- Streaming: Faster adaptation, real-time evolution, higher cost

**Recommendation**: Start with batch mode, add streaming for critical patterns in Phase 4.

---

## 14. Success Metrics

### 14.1 Proposal Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Proposal Acceptance Rate | >70% | % of proposals passing validation |
| First-Try Success Rate | >50% | % of proposals accepted without regeneration |
| Average Confidence (Accepted) | >0.75 | Mean LLM confidence for accepted proposals |
| Average Confidence (Rejected) | <0.60 | Mean LLM confidence for rejected proposals |

### 14.2 Constraint Adherence Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Q3 Violation Rate | <10% | % of proposals exceeding 8-tick budget |
| Doctrine Violation Rate | <15% | % of proposals violating sector doctrines |
| Guard Violation Rate | <5% | % of proposals removing protected elements |
| Schema Violation Rate | <10% | % of proposals with invalid RDF structure |

### 14.3 Learning Effectiveness Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Acceptance Rate Improvement | +10% per month | Change in acceptance rate over time |
| Repeat Violations | <20% | % of same constraint violated repeatedly |
| Corpus Size | >1000 proposals | Total accepted + rejected proposals |
| Few-Shot Example Count | >50 per sector | Number of sector-specific examples |

### 14.4 Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Proposal Generation Time | <30s | Time from pattern → proposal |
| Validation Time | <5s | Time to validate proposal |
| End-to-End Latency | <60s | Pattern → validated proposal |
| LLM Token Usage | <10K tokens/proposal | Prompt + response tokens |

---

## 15. Related Work & References

### 15.1 Academic Research

- **Constrained Language Models**: [LMQL (2023)](https://lmql.ai) - Language Model Query Language for constrained generation
- **Ontology Evolution**: [Stojanovic et al. (2002)](https://link.springer.com/chapter/10.1007/3-540-45810-7_40) - Ontology evolution strategies
- **LLM Planning**: [ReAct (2023)](https://arxiv.org/abs/2210.03629) - Reasoning + Acting with LLMs

### 15.2 Industry Standards

- [OWL 2 Web Ontology Language](https://www.w3.org/TR/owl2-overview/) - W3C standard for ontologies
- [SHACL Shapes Constraint Language](https://www.w3.org/TR/shacl/) - W3C standard for RDF validation
- [OpenTelemetry](https://opentelemetry.io/) - Observability framework

### 15.3 KNHK Documentation

- [Autonomous Ontology System Design](../autonomous-ontology-system-design.md) - Core ontology architecture
- [Autonomous Ontology ADRs](../autonomous-ontology-adr.md) - Architecture decisions
- [CLAUDE.md](../../CLAUDE.md) - Project development guidelines

---

## 16. Conclusion

The LLM-Based Overlay Proposer is a **constraint-aware autonomous system** that generates ontology evolution proposals (ΔΣ) from observed patterns while respecting hard invariants (Q1-Q5), organizational doctrines, performance budgets, and guard constraints.

**Key Design Principles**:
1. **Defense-in-Depth**: Hybrid constraint enforcement (prompt + guided + validation)
2. **Fail-Safe**: Multi-stage validation with fail-fast on critical violations
3. **Learning-Enabled**: Continuous improvement from accepted/rejected proposals
4. **Sector-Specific**: Tailored prompts and constraints per domain
5. **Human-in-the-Loop**: Required review for critical sectors (healthcare, manufacturing)
6. **Auditable**: Complete proposal history with cryptographic receipts

**Implementation Phases**:
- **Phase 1** (Current): Design specification + basic prototype with OpenAI API
- **Phase 2**: Validation pipeline integration + learning system
- **Phase 3**: Fine-tuned LLM on proposal corpus
- **Phase 4**: Multi-agent collaboration + real-time streaming
- **Phase 5**: Production hardening + security audit

**Next Steps**:
1. Review this design with system architects and domain experts
2. Prototype basic LLM proposer (Strategy A: prompt engineering)
3. Implement validation pipeline integration
4. Collect initial corpus of proposals for learning
5. Iterate based on acceptance rate metrics

---

**Document Status**: ✅ Complete Design Specification
**Last Updated**: 2025-11-16
**Next Milestone**: Prototype implementation (Phase 1)
**Estimated Effort**: 8-10 weeks for Phase 1 + Phase 2
