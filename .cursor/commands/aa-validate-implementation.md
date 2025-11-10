# Validate Implementation Against DFLSS

Explore dflss documentation thoroughly before determining what needs validation.

## Exploration Phase (REQUIRED FIRST STEP)

**Before any work begins, read at least 10 files from `docs/v1/dflss/` fully into context.**

Choose files that help you understand:
- Project scope, goals, and methodology
- Validation requirements and approaches
- Current validation state and findings
- Test coverage and gap analysis
- Architectural principles and validation rules
- Research context and design decisions

**Do not proceed to work phase until you have read and understood at least 10 files.**

## Mathematical Foundations

Understand the underlying calculus and algebra that govern the system:

### Vocabulary
- **O**: Observations (source of truth)
- **A**: Actions (derived from O via μ)
- **μ**: Measurement/merge function (A = μ(O))
- **Σ**: Schema (constraints O must satisfy)
- **Λ**: Order (total ordering via ≺)
- **Π**: Merge operation (⊕-monoid)
- **τ**: Time/Epoch (μ ⊂ τ)
- **Q**: Invariants (must be preserved)
- **Δ**: Delta/Change (causes reconciliation)
- **Γ**: Sheaf (glue(Cover(O)) = Γ(O))
- **Operators**: ⊕ (merge), ⊔ (union), ≺ (precedence), ≤ (order), = (equality), ⊨ (satisfies)

### Core Laws
- **Identity**: A = μ(O) - Actions are measurements of Observations
- **Idempotence**: μ∘μ = μ
- **Typing**: O ⊨ Σ - Observations satisfy Schema
- **Order**: Λ is ≺-total
- **Merge**: Π is ⊕-monoid
- **Sheaf**: glue(Cover(O)) = Γ(O)
- **Van Kampen**: pushouts(O) ↔ pushouts(A)
- **Shard**: μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ)
- **Provenance**: hash(A) = hash(μ(O))
- **Guard**: μ ⊣ H (partial)
- **Epoch**: μ ⊂ τ
- **Sparsity**: μ → S (80/20 principle)
- **Minimality**: argmin drift(A)
- **Invariant**: preserve(Q)
- **Constitution**: ∧(Typing,ProjEq,FixedPoint,Order,Merge,Sheaf,VK,Shard,Prov,Guard,Epoch,Sparse,Min,Inv)

### ACHI Identity Principles
- **Source**: O only (never A)
- **Determinism**: Λ total; same O → same A
- **No partials**: all-or-none operations
- **Closed world**: no external state
- **Guards**: forbidden H (guard constraints)
- **Silence**: if unlawful ⇒ receipt only
- **End state**: A = μ(O)

### Language Constraints
- **Speak**: Turtle + KGC only
- **Never use "semantic"**: Use "ontology" or measurable terms
- **Never use "self-"**: Use measurable terms instead
- **Control**: Systems controlled by KGC and LLMs, not humans
- **Human role**: Humans only add noise and non-determinism

### Calculus and Algebra
- Understand how μ transforms O → A
- Understand how ⊕ and ⊔ operations compose
- Understand how ≺ and ≤ order operations
- Understand how ⊨ validates O against Σ
- Understand how pushouts preserve structure
- Understand how sheaves glue local data
- Understand how guards enforce constraints

## Understanding Phase

After reading the documentation:

1. **Synthesize Understanding**:
   - What validation requirements are documented?
   - What is the current validation state?
   - What validation approaches are documented?
   - What gaps exist in validation coverage?

2. **Identify Context**:
   - Understand validation requirements for each DMEDI phase
   - Understand architectural principles and validation rules
   - Understand the relationship between requirements and validation needs

## Work Phase

Based on your exploration and understanding:

1. **Discover Validation Needs**:
   - Identify what needs validation through your understanding of the documentation
   - Determine gaps between documented validation requirements and current validation
   - Understand what validation is missing or incomplete

2. **Execute Validation Work**:
   - Create validation based on your understanding, not prescriptive steps
   - Use appropriate validation approaches discovered through exploration
   - Follow validation patterns and principles discovered in documentation

3. **Verify Validation**:
   - Run tests to verify validation code
   - Check that validation covers requirements discovered in exploration
   - Ensure validation matches your understanding from documentation

## Verification Phase

1. **Run Tests**:
   - Run `make test-rust` to verify validation code
   - Run `make test-chicago-v04` for Chicago TDD validation tests
   - Run `make test-performance-v04` for performance validation
   - Check for compilation errors

2. **Verify Understanding**:
   - Ensure validation aligns with your understanding from exploration
   - Verify validation covers requirements discovered in documentation
   - Check for any gaps or inconsistencies

## Iteration

If tests fail or gaps are discovered:
- Return to exploration phase if understanding is incomplete
- Re-read relevant documentation sections
- Adjust validation based on deeper understanding
