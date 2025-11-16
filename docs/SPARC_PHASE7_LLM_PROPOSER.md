# SPARC Phase 7: LLM-Based Proposer Implementation Design

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Implementation Design (Phase 1)
**Authors**: Backend API Developer Agent
**Related Documents**:
- [LLM Overlay Proposer Design](designs/llm-overlay-proposer-design.md)
- [Autonomous Ontology System Design](autonomous-ontology-system-design.md)
- [CLAUDE.md](../CLAUDE.md)

---

## Executive Summary

This document specifies the **production-grade implementation** of the LLM-Based Proposer for KNHK's autonomous ontology evolution system. The proposer generates constraint-aware ontology change proposals (ΔΣ) using large language models while guaranteeing compliance with hard invariants (Q1-Q5), organizational doctrines, performance budgets, and guard constraints.

**Key Innovation**: Defense-in-depth constraint enforcement combining:
1. Constraint-aware prompt engineering (Strategy A)
2. Guided decoding for critical constraints (Strategy B)
3. Post-hoc validation pipeline (Strategy C)
4. Learning loop for continuous improvement

**Production Requirements**:
- ✅ All proposals must pass Weaver schema validation (source of truth)
- ✅ Q1-Q5 invariants must be preserved (validated at token level where possible)
- ✅ Performance budget ≤8 ticks for hot path operations
- ✅ Doctrine compliance for sector-specific rules
- ✅ Guard constraint preservation (no removal of protected elements)
- ✅ Learning from accepted/rejected proposals
- ✅ Observability via OpenTelemetry spans/metrics

---

## 1. Architecture Overview

### 1.1 System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                    MAPE-K COORDINATOR                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Monitor  │→ │ Analyze  │→ │   Plan   │→ │ Execute  │       │
│  └──────────┘  └──────────┘  └────┬─────┘  └──────────┘       │
│                                    │                             │
│                          Pattern Detected                        │
└────────────────────────────────────┼─────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     LLM PROPOSER SYSTEM                          │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 1. CONSTRAINT LOADING                                      │ │
│  │    - Load Q1-Q5 invariants from current state             │ │
│  │    - Load sector doctrines (finance, healthcare, etc.)    │ │
│  │    - Load guard profiles (protected elements)             │ │
│  │    - Calculate performance budget (remaining ticks)       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 2. PROMPT ENGINE                                           │ │
│  │    - Extract relevant ontology context                     │ │
│  │    - Generate constraint-aware system prompt              │ │
│  │    - Build sector-specific task prompt                    │ │
│  │    - Include few-shot examples from learning corpus       │ │
│  │    - Specify JSON output schema                           │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 3. LLM GENERATION                                          │ │
│  │    [STRATEGY A: Prompt-based constraints]                 │ │
│  │    - Send prompt to LLM API (OpenAI, Ollama, etc.)        │ │
│  │                                                            │ │
│  │    [STRATEGY B: Guided decoding] (optional, for Q3/Q4)    │ │
│  │    - Use constrained generation library (Outlines)        │ │
│  │    - Enforce token-level constraints on critical fields   │ │
│  │                                                            │ │
│  │    [STRATEGY C: Post-hoc validation] (always)             │ │
│  │    - Parse JSON response                                  │ │
│  │    - Validate against all constraints                     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 4. PROPOSAL EXTRACTION                                     │ │
│  │    - Parse LLM response to SigmaDiff format               │ │
│  │    - Extract reasoning and confidence score               │ │
│  │    - Estimate performance impact (ticks)                  │ │
│  │    - Attach constraint metadata                           │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                     Proposal (ΔΣ)                               │
└────────────────────────────┼────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  VALIDATION PIPELINE                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Schema   │→ │ Q1-Q5    │→ │ Doctrine │→ │ Guard    │       │
│  │ Check    │  │ Check    │  │ Check    │  │ Check    │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│                                    │                             │
│                              Valid/Invalid                       │
└────────────────────────────────────┼─────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     LEARNING SYSTEM                              │
│  - Record proposal outcome (accepted/rejected)                  │
│  - Extract failure patterns (which constraints violated)        │
│  - Update few-shot example corpus                               │
│  - Adapt prompt templates based on success rate                 │
│  - Measure improvement metrics (acceptance rate trend)          │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Core Components

| Component | Responsibility | Input | Output |
|-----------|---------------|-------|--------|
| **Proposer** | Orchestrate proposal generation | `DetectedPattern`, `Constraints` | `Proposal` |
| **PromptEngine** | Build constraint-aware prompts | `Pattern`, `Doctrines`, `Invariants` | `String` (prompt) |
| **LLMClient** | Call LLM API | `Prompt` | `LLMResponse` (JSON) |
| **ValidatorLLM** | Post-hoc validation | `Proposal`, `Constraints` | `ValidationReport` |
| **LearningSystem** | Track outcomes and adapt | `Proposal`, `ValidationReport` | Updated corpus |

---

## 2. Constraint-Aware Prompt Generation

### 2.1 Prompt Structure

The prompt must encode ALL constraints so the LLM understands the bounded solution space:

```rust
pub struct PromptTemplate {
    system_prompt: String,      // Role definition
    constraint_section: String,  // Q1-Q5, doctrines, guards
    pattern_section: String,     // Observed pattern description
    context_section: String,     // Relevant ontology excerpt
    output_schema: String,       // Expected JSON structure
    few_shot_examples: Vec<String>, // Success/failure examples
}
```

### 2.2 System Prompt Template

```
You are an ontology evolution assistant for the KNHK knowledge graph system.
Your task is to propose minimal, safe ontology changes (ΔΣ) based on observed patterns.

CRITICAL CONSTRAINTS (MUST NEVER VIOLATE):

1. Hard Invariants (Q1-Q5):
   - Q1: No Retrocausation - Time flows forward only, no temporal cycles
   - Q2: Type Soundness - All properties must have valid domain/range types
   - Q3: Guard Preservation - Hot path operations must have ≤8 execution steps
   - Q4: SLO Compliance - Hot path execution time must be ≤8 CPU ticks
   - Q5: Performance Bounds - No performance regression >10% on existing benchmarks

2. Sector Doctrines ({sector}):
{doctrine_list}

3. Guard Constraints (IMMUTABLE):
   - Protected classes (CANNOT REMOVE): {protected_classes}
   - Protected properties (CANNOT REMOVE): {protected_properties}
   - Maximum run length: {max_run_len} steps
   - Performance tier: {performance_tier}

4. Performance Budget:
   - Current tick consumption: {consumed_ticks}/{max_ticks} ticks
   - Remaining budget: {remaining_ticks} ticks
   - Cost estimates:
     * New class: ~1 tick overhead
     * New property: ~0.5 ticks overhead
     * New validation: ~3 ticks overhead
   - YOUR BUDGET FOR THIS PROPOSAL: {remaining_ticks} ticks

5. Immutability Rules:
   - Cannot modify historical data (Q1 enforcement)
   - Cannot remove protected classes/properties (guard enforcement)
   - All proposals must be reversible (rollback capability)

REASONING GUIDANCE:
- Propose MINIMAL changes (prefer evolution over revolution)
- Explain WHY each change is needed
- Explain HOW constraints are satisfied
- Provide confidence score (0.0-1.0) based on:
  * Pattern clarity (how well understood is the need?)
  * Constraint satisfaction confidence
  * Domain knowledge certainty
  * Estimated performance impact accuracy
```

### 2.3 Pattern Section Template

```
OBSERVED PATTERN:
{pattern_description}

Pattern Details:
- Sector: {sector}
- Confidence: {pattern_confidence}
- Observation Count: {observation_count}
- First Observed: {first_observed}
- Last Observed: {last_observed}

Recommended Action: {recommended_action}
```

### 2.4 Context Section Template

```
CURRENT ONTOLOGY SNAPSHOT (Relevant Excerpt):

Classes:
{relevant_classes}

Properties:
{relevant_properties}

SHACL Shapes:
{relevant_shapes}

Current Performance Profile:
- Hot path tick count: {current_ticks}/8
- Warm path operations: {warm_ops}
- Cold path operations: {cold_ops}
```

### 2.5 Output Schema

```json
{
  "reasoning": "string (required) - Explain WHY this change is needed and HOW constraints are satisfied",
  "confidence": "number (required) - 0.0 to 1.0",
  "estimated_ticks": "integer (required) - Predicted execution time after change",
  "delta_sigma": {
    "added_classes": [
      {
        "uri": "string (required) - e.g., knhk:NewClass",
        "label": "string (required)",
        "subclass_of": "string (required) - Parent class URI",
        "properties_required": ["string"] (optional),
        "properties_optional": ["string"] (optional)
      }
    ],
    "added_properties": [
      {
        "uri": "string (required) - e.g., knhk:newProperty",
        "label": "string (required)",
        "domain": "string (required) - Class URI",
        "range": "string (required) - Datatype or Class URI",
        "required": "boolean (required)",
        "cardinality": "enum (required) - One|ZeroOrOne|ZeroOrMore|OneOrMore"
      }
    ],
    "removed_classes": ["string"] (optional - URIs to remove),
    "removed_properties": ["string"] (optional - URIs to remove),
    "modified_shapes": [
      {
        "uri": "string (required) - SHACL shape URI",
        "added_constraints": ["string"] (optional),
        "removed_constraints": ["string"] (optional)
      }
    ]
  },
  "doctrines_satisfied": ["string"] (required - Doctrine IDs claimed),
  "invariants_satisfied": ["string"] (required - Q1, Q2, Q3, Q4, Q5),
  "rollback_plan": "string (required) - How to undo this change if needed"
}
```

### 2.6 Few-Shot Examples

Include 2-3 examples for each sector:

**Good Example (Finance)**:
```json
{
  "pattern": "Observed 15 retirement account variations (401k, IRA, Roth IRA)",
  "reasoning": "Finance sector requires specialized account types for regulatory compliance...",
  "confidence": 0.85,
  "estimated_ticks": 7,
  "delta_sigma": {
    "added_classes": [
      {
        "uri": "finance:RetirementAccount",
        "label": "Retirement Account",
        "subclass_of": "finance:Account",
        "properties_required": ["account_id", "tax_status"]
      }
    ]
  },
  "validation_result": "✅ ACCEPTED"
}
```

**Bad Example (Violates Q3)**:
```json
{
  "pattern": "Need detailed transaction validation",
  "reasoning": "Add 10-step validation process",
  "estimated_ticks": 12,
  "validation_result": "❌ REJECTED - Q3 violation: exceeds 8-tick budget"
}
```

---

## 3. Guided Decoding Strategy

For **critical constraints** (Q3: max_run_len ≤ 8), use guided decoding to enforce token-level constraints:

### 3.1 Constraint Grammar (LMQL-style)

```python
@lmql.query
async def generate_constrained_proposal(
    pattern: str,
    max_ticks: int,
    protected_classes: List[str]
):
    '''lmql
    """Propose ontology change for: {pattern}

    Constraints:
    - Max ticks: {max_ticks}
    - Protected classes: {protected_classes}

    Output:
    {
        "estimated_ticks": [TICKS where TICKS <= max_ticks],
        "removed_classes": [REMOVED where all(c not in protected_classes for c in REMOVED)]
    }
    """
    '''
```

### 3.2 Token Masking

For critical fields, mask invalid tokens:

```rust
pub struct ConstrainedDecoding {
    max_ticks: u32,
    protected_classes: Vec<String>,
    protected_properties: Vec<String>,
}

impl ConstrainedDecoding {
    pub fn mask_invalid_tokens(&self, field: &str, token: &str) -> bool {
        match field {
            "estimated_ticks" => {
                // Only allow numeric tokens ≤ max_ticks
                token.parse::<u32>().ok()
                    .map(|val| val <= self.max_ticks)
                    .unwrap_or(false)
            }
            "removed_classes" => {
                // Block tokens matching protected class names
                !self.protected_classes.iter().any(|c| token.contains(c))
            }
            "removed_properties" => {
                // Block tokens matching protected property names
                !self.protected_properties.iter().any(|p| token.contains(p))
            }
            _ => true // Allow all other tokens
        }
    }
}
```

### 3.3 Rust Integration (Ollama with Constraints)

```rust
use ollama_rs::generation::completion::request::GenerationRequest;

pub async fn generate_with_constraints(
    prompt: &str,
    constraints: &ConstrainedDecoding,
) -> Result<String> {
    let request = GenerationRequest::new("llama3".to_string(), prompt.to_string())
        .temperature(0.3)  // Lower temperature for more constraint adherence
        .top_p(0.9)
        .top_k(40);

    // Note: Actual constraint enforcement would require custom sampling
    // or integration with libraries like Outlines/LMQL via Python bridge

    let response = ollama_client.generate(request).await?;
    Ok(response.response)
}
```

---

## 4. Confidence Scoring Algorithm

Confidence score (0.0-1.0) combines multiple factors:

### 4.1 Scoring Components

```rust
pub struct ConfidenceScore {
    pattern_clarity: f64,        // How well is the pattern understood?
    constraint_adherence: f64,   // How confident in constraint satisfaction?
    domain_knowledge: f64,        // How well does LLM know the domain?
    performance_estimate: f64,    // How accurate is tick estimate?
}

impl ConfidenceScore {
    pub fn compute(&self) -> f64 {
        // Weighted average
        let weights = [0.3, 0.4, 0.2, 0.1]; // Constraint adherence most important
        let scores = [
            self.pattern_clarity,
            self.constraint_adherence,
            self.domain_knowledge,
            self.performance_estimate,
        ];

        weights.iter()
            .zip(scores.iter())
            .map(|(w, s)| w * s)
            .sum()
    }

    pub fn from_proposal(proposal: &Proposal, validation: &ValidationReport) -> Self {
        ConfidenceScore {
            pattern_clarity: proposal.pattern.confidence,
            constraint_adherence: Self::calculate_constraint_adherence(validation),
            domain_knowledge: 0.8, // TODO: Calibrate per sector
            performance_estimate: Self::estimate_performance_confidence(proposal),
        }
    }

    fn calculate_constraint_adherence(validation: &ValidationReport) -> f64 {
        let total_checks = validation.stages.len() as f64;
        let passed_checks = validation.stages.iter()
            .filter(|s| s.passed)
            .count() as f64;

        passed_checks / total_checks
    }

    fn estimate_performance_confidence(proposal: &Proposal) -> f64 {
        // Higher confidence if tick estimate is well within budget
        let budget_utilization = proposal.estimated_ticks as f64 / 8.0;

        if budget_utilization <= 0.75 {
            0.9 // Well within budget
        } else if budget_utilization <= 0.9 {
            0.7 // Close to budget
        } else {
            0.5 // Very close to limit
        }
    }
}
```

### 4.2 Confidence Thresholds

| Confidence Range | Interpretation | Action |
|------------------|----------------|--------|
| 0.9 - 1.0 | Very High | Auto-approve for low-risk sectors |
| 0.75 - 0.9 | High | Approve with human review |
| 0.6 - 0.75 | Medium | Require multiple approvals |
| 0.4 - 0.6 | Low | Reject or request refinement |
| 0.0 - 0.4 | Very Low | Auto-reject |

---

## 5. Learning Loop Design

### 5.1 Proposal Corpus Structure

```rust
pub struct ProposalCorpus {
    accepted_proposals: Vec<ProposalOutcome>,
    rejected_proposals: Vec<ProposalOutcome>,
    constraint_violations: HashMap<String, Vec<ProposalId>>,
    sector_examples: HashMap<Sector, Vec<FewShotExample>>,
    metrics: LearningMetrics,
}

pub struct ProposalOutcome {
    proposal: Proposal,
    validation_report: ValidationReport,
    timestamp: DateTime<Utc>,
    feedback: Option<String>,
}

pub struct FewShotExample {
    pattern: String,
    proposal: SigmaDiff,
    reasoning: String,
    confidence: f64,
    validation_result: ValidationReport,
}

pub struct LearningMetrics {
    acceptance_rate: f64,
    acceptance_rate_trend: Vec<(DateTime<Utc>, f64)>,
    q3_violation_rate: f64,
    doctrine_violation_rate: f64,
    avg_confidence_accepted: f64,
    avg_confidence_rejected: f64,
}
```

### 5.2 Learning Workflow

```rust
impl LearningSystem {
    pub async fn record_outcome(
        &mut self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()> {
        let outcome = ProposalOutcome {
            proposal: proposal.clone(),
            validation_report: report.clone(),
            timestamp: Utc::now(),
            feedback: None,
        };

        if report.passed {
            // Add to accepted proposals
            self.corpus.accepted_proposals.push(outcome.clone());

            // Extract as few-shot example
            let example = FewShotExample::from_outcome(&outcome);
            self.corpus.sector_examples
                .entry(proposal.pattern.sector)
                .or_default()
                .push(example);

            tracing::info!(
                proposal_id = %proposal.id,
                confidence = proposal.confidence,
                "Accepted proposal recorded for learning"
            );
        } else {
            // Add to rejected proposals
            self.corpus.rejected_proposals.push(outcome);

            // Track constraint violations
            for stage in &report.stages {
                if !stage.passed {
                    self.corpus.constraint_violations
                        .entry(stage.name.clone())
                        .or_default()
                        .push(proposal.id.clone());
                }
            }

            tracing::warn!(
                proposal_id = %proposal.id,
                violations = ?self.get_violations(&report),
                "Rejected proposal recorded for learning"
            );
        }

        // Update metrics
        self.update_metrics();

        // Adapt prompts if needed
        self.adapt_prompts_from_patterns()?;

        Ok(())
    }

    fn update_metrics(&mut self) {
        let total = self.corpus.accepted_proposals.len()
            + self.corpus.rejected_proposals.len();

        if total == 0 {
            return;
        }

        self.corpus.metrics.acceptance_rate =
            self.corpus.accepted_proposals.len() as f64 / total as f64;

        self.corpus.metrics.acceptance_rate_trend.push((
            Utc::now(),
            self.corpus.metrics.acceptance_rate,
        ));

        // Calculate Q3 violation rate
        let q3_violations = self.corpus.constraint_violations
            .get("Q3")
            .map(|v| v.len())
            .unwrap_or(0);
        self.corpus.metrics.q3_violation_rate =
            q3_violations as f64 / total as f64;

        // Calculate average confidence
        self.corpus.metrics.avg_confidence_accepted =
            self.corpus.accepted_proposals.iter()
                .map(|o| o.proposal.confidence)
                .sum::<f64>() / self.corpus.accepted_proposals.len().max(1) as f64;

        self.corpus.metrics.avg_confidence_rejected =
            self.corpus.rejected_proposals.iter()
                .map(|o| o.proposal.confidence)
                .sum::<f64>() / self.corpus.rejected_proposals.len().max(1) as f64;
    }

    fn adapt_prompts_from_patterns(&mut self) -> Result<()> {
        // If Q3 is frequently violated, increase emphasis in prompts
        if self.corpus.metrics.q3_violation_rate > 0.2 {
            self.prompt_adapter.increase_emphasis("performance_budget", 1.5);
            tracing::info!("Increasing prompt emphasis on performance budget");
        }

        // If acceptance rate is low, add more examples
        if self.corpus.metrics.acceptance_rate < 0.5
            && self.corpus.accepted_proposals.len() > 10 {
            // Increase few-shot example count
            self.prompt_adapter.set_example_count(5);
            tracing::info!("Increasing few-shot example count to 5");
        }

        Ok(())
    }

    pub fn get_few_shot_examples(
        &self,
        sector: &Sector,
        count: usize,
    ) -> Vec<FewShotExample> {
        self.corpus.sector_examples
            .get(sector)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .take(count)
            .collect()
    }
}
```

### 5.3 Prompt Adaptation

```rust
pub struct PromptAdapter {
    base_template: String,
    emphasis_weights: HashMap<String, f64>,
    example_count: usize,
}

impl PromptAdapter {
    pub fn build_prompt(&self, request: &ProposalRequest) -> String {
        let mut prompt = self.base_template.clone();

        // Adjust emphasis
        for (section, weight) in &self.emphasis_weights {
            if *weight > 1.5 {
                prompt = prompt.replace(
                    &format!("{}:", section.to_uppercase()),
                    &format!("⚠️ CRITICAL {} (FREQUENTLY VIOLATED):", section.to_uppercase())
                );
            }
        }

        // Add few-shot examples
        let examples = self.learning_system.get_few_shot_examples(
            &request.pattern.sector,
            self.example_count,
        );

        let example_text = examples.iter()
            .map(|ex| format!(
                "EXAMPLE:\nPattern: {}\nProposal: {:?}\nResult: {}\nReasoning: {}\n",
                ex.pattern,
                ex.proposal,
                if ex.validation_result.passed { "✅ ACCEPTED" } else { "❌ REJECTED" },
                ex.reasoning
            ))
            .collect::<Vec<_>>()
            .join("\n");

        prompt.replace("{few_shot_examples}", &example_text)
    }

    pub fn increase_emphasis(&mut self, section: &str, weight: f64) {
        *self.emphasis_weights.entry(section.to_string()).or_insert(1.0) = weight;
    }
}
```

---

## 6. Production Hardening Strategies

### 6.1 Rate Limiting

```rust
pub struct RateLimiter {
    max_proposals_per_hour: usize,
    recent_proposals: Arc<RwLock<VecDeque<DateTime<Utc>>>>,
}

impl RateLimiter {
    pub fn check_rate_limit(&self) -> Result<()> {
        let mut recent = self.recent_proposals.write();

        // Remove proposals older than 1 hour
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        recent.retain(|ts| *ts > cutoff);

        if recent.len() >= self.max_proposals_per_hour {
            return Err(anyhow!(
                "Rate limit exceeded: {} proposals in last hour (max: {})",
                recent.len(),
                self.max_proposals_per_hour
            ));
        }

        recent.push_back(Utc::now());
        Ok(())
    }
}
```

### 6.2 Cost Control

```rust
pub struct CostController {
    max_tokens_per_proposal: usize,
    total_tokens_used: Arc<AtomicUsize>,
    budget_limit_tokens: usize,
}

impl CostController {
    pub fn check_token_budget(&self, prompt_tokens: usize) -> Result<()> {
        if prompt_tokens > self.max_tokens_per_proposal {
            return Err(anyhow!(
                "Prompt too large: {} tokens (max: {})",
                prompt_tokens,
                self.max_tokens_per_proposal
            ));
        }

        let current_usage = self.total_tokens_used.load(Ordering::Relaxed);
        if current_usage + prompt_tokens > self.budget_limit_tokens {
            return Err(anyhow!(
                "Token budget exceeded: {} + {} > {}",
                current_usage,
                prompt_tokens,
                self.budget_limit_tokens
            ));
        }

        Ok(())
    }

    pub fn record_usage(&self, tokens: usize) {
        self.total_tokens_used.fetch_add(tokens, Ordering::Relaxed);
    }
}
```

### 6.3 Timeout Handling

```rust
pub async fn generate_with_timeout(
    llm_client: &LLMClient,
    prompt: &str,
    timeout_ms: u64,
) -> Result<String> {
    match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        llm_client.generate(prompt)
    ).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(anyhow!("LLM generation failed: {}", e)),
        Err(_) => Err(anyhow!("LLM generation timed out after {}ms", timeout_ms)),
    }
}
```

### 6.4 Caching

```rust
pub struct PromptCache {
    cache: Arc<DashMap<String, CachedResponse>>,
    max_age_seconds: u64,
}

struct CachedResponse {
    response: String,
    timestamp: DateTime<Utc>,
}

impl PromptCache {
    pub fn get(&self, prompt_hash: &str) -> Option<String> {
        self.cache.get(prompt_hash).and_then(|entry| {
            let age = Utc::now().signed_duration_since(entry.timestamp);
            if age.num_seconds() < self.max_age_seconds as i64 {
                Some(entry.response.clone())
            } else {
                None
            }
        })
    }

    pub fn put(&self, prompt_hash: String, response: String) {
        self.cache.insert(prompt_hash, CachedResponse {
            response,
            timestamp: Utc::now(),
        });
    }
}
```

### 6.5 Observability

```rust
#[tracing::instrument(
    skip(self, pattern),
    fields(
        pattern_id = %pattern.id,
        sector = %pattern.sector,
        confidence = pattern.confidence
    )
)]
pub async fn generate_proposal(
    &self,
    pattern: &DetectedPattern,
) -> Result<Proposal> {
    let start = Instant::now();

    // Emit OTEL span
    let span = tracing::info_span!(
        "llm_proposal_generation",
        pattern_id = %pattern.id,
        sector = %pattern.sector
    );

    let _enter = span.enter();

    // Check rate limit
    self.rate_limiter.check_rate_limit()?;

    // Generate proposal
    let result = self.generate_proposal_internal(pattern).await;

    let duration = start.elapsed();

    // Emit metrics
    match &result {
        Ok(proposal) => {
            tracing::info!(
                duration_ms = duration.as_millis(),
                confidence = proposal.confidence,
                estimated_ticks = proposal.estimated_ticks,
                "Proposal generated successfully"
            );
        }
        Err(e) => {
            tracing::error!(
                duration_ms = duration.as_millis(),
                error = %e,
                "Proposal generation failed"
            );
        }
    }

    result
}
```

---

## 7. Integration with MAPE-K Loop

### 7.1 Integration Points

The LLM Proposer integrates into the **Analyze → Plan** transition:

```rust
// coordinator.rs - Phase 2: Analyze
async fn phase_analyze(&self, cycle: &mut LoopCycle, cycle_id: &str)
    -> Result<(Vec<DetectedPattern>, String), CoordinationError>
{
    let patterns = self.pattern_detector.detect_patterns().await;

    // ... create receipt ...

    Ok((patterns, receipt_id))
}

// coordinator.rs - Phase 3: Plan (NEW - LLM Integration)
async fn phase_plan(&self, patterns: &[DetectedPattern], cycle: &mut LoopCycle, cycle_id: &str)
    -> Result<(Vec<Proposal>, Vec<String>), CoordinationError>
{
    let mut proposals = Vec::new();
    let mut receipt_ids = Vec::new();

    for pattern in patterns {
        // Load constraints for this pattern's sector
        let doctrines = self.doctrine_store.get_for_sector(&pattern.sector)?;
        let invariants = HardInvariants::default();
        let guards = self.guard_store.get_for_sector(&pattern.sector)?;

        // Generate proposal using LLM
        let proposal = self.llm_proposer
            .generate_proposal(pattern, &doctrines, &invariants, &guards)
            .await?;

        // Create receipt for proposal
        let receipt = Receipt::create(
            ReceiptOperation::ProposalGenerated {
                delta_description: proposal.delta_sigma.to_string(),
            },
            ReceiptOutcome::Pending { next_stage: "validation".to_string() },
            vec![format!("Confidence: {}", proposal.confidence)],
            pattern.sector.clone(),
            &self.signing_key,
            None,
        )?;

        let receipt_id = self.receipt_store.append(receipt).await?;

        proposals.push(proposal);
        receipt_ids.push(receipt_id);
        cycle.proposals_generated += 1;
    }

    Ok((proposals, receipt_ids))
}
```

### 7.2 Data Flow

```
Pattern Detection (Analyze)
    ↓
    DetectedPattern
    ↓
Constraint Loading (Plan)
    ├→ Doctrines (sector-specific)
    ├→ Invariants (Q1-Q5)
    ├→ Guards (protected elements)
    └→ Performance Budget
    ↓
LLM Proposal Generation (Plan)
    ↓
    Proposal (ΔΣ)
    ↓
Validation Pipeline (Execute)
    ├→ Schema Check
    ├→ Invariant Check (Q1-Q5)
    ├→ Doctrine Check
    ├→ Guard Check
    └→ Performance Check
    ↓
Validation Report
    ↓
Learning System (Knowledge)
    └→ Update corpus, adapt prompts
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_includes_all_constraints() {
        let engine = PromptEngine::new();
        let request = create_test_request();

        let prompt = engine.build_prompt(&request);

        // Assert all constraint sections present
        assert!(prompt.contains("Q1: No Retrocausation"));
        assert!(prompt.contains("Q2: Type Soundness"));
        assert!(prompt.contains("Q3: Guard Preservation"));
        assert!(prompt.contains("Q4: SLO Compliance"));
        assert!(prompt.contains("Q5: Performance Bounds"));
        assert!(prompt.contains(&request.pattern.sector.to_string()));
    }

    #[test]
    fn test_confidence_score_calculation() {
        let score = ConfidenceScore {
            pattern_clarity: 0.9,
            constraint_adherence: 0.8,
            domain_knowledge: 0.7,
            performance_estimate: 0.9,
        };

        let confidence = score.compute();

        // Weighted average: 0.3*0.9 + 0.4*0.8 + 0.2*0.7 + 0.1*0.9
        assert!((confidence - 0.82).abs() < 0.01);
    }

    #[test]
    fn test_performance_budget_estimation() {
        let budget = PerformanceBudget {
            max_ticks: 8,
            consumed_ticks: 5,
            remaining_ticks: 3,
            cost_per_class: 1.0,
            cost_per_property: 0.5,
            cost_per_validation: 3.0,
        };

        let diff = SigmaDiff {
            added_classes: vec![create_test_class()],
            added_properties: vec![create_test_property()],
            ..Default::default()
        };

        let cost = budget.estimate_cost(&diff);
        assert_eq!(cost, 2); // 1 class + 1 property = 1 + 0.5 = 1.5 → 2 ticks
    }
}
```

### 8.2 Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_proposal_generation() {
    // Setup
    let proposer = setup_test_proposer().await;
    let pattern = DetectedPattern {
        name: "New account types observed".to_string(),
        sector: Sector::Finance,
        confidence: 0.85,
        recommended_action: PatternAction::ProposeChange {
            description: "Add RetirementAccount class".to_string(),
        },
        ..Default::default()
    };

    // Generate proposal
    let proposal = proposer.generate_proposal(&pattern).await
        .expect("Proposal generation failed");

    // Assertions
    assert!(proposal.confidence > 0.7);
    assert!(proposal.estimated_ticks <= 8);
    assert!(!proposal.delta_sigma.added_classes.is_empty());
    assert!(proposal.doctrines_satisfied.contains(&"FIN-001".to_string()));
}

#[tokio::test]
async fn test_proposal_rejects_q3_violation() {
    let proposer = setup_test_proposer().await;
    let pattern = create_pattern_requiring_12_ticks();

    let proposal = proposer.generate_proposal(&pattern).await
        .expect("Proposal generation failed");

    let validation = proposer.validate_proposal(&proposal).await
        .expect("Validation failed");

    // Should be rejected for Q3 violation
    assert!(!validation.passed);
    assert!(validation.stages.iter()
        .any(|s| s.name == "Q3" && !s.passed));
}
```

### 8.3 Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_all_proposals_respect_budget(
        class_count in 0usize..=5,
        property_count in 0usize..=10
    ) {
        let budget = PerformanceBudget {
            max_ticks: 8,
            consumed_ticks: 0,
            remaining_ticks: 8,
            cost_per_class: 1.0,
            cost_per_property: 0.5,
            cost_per_validation: 3.0,
        };

        let diff = SigmaDiff {
            added_classes: (0..class_count).map(|_| create_test_class()).collect(),
            added_properties: (0..property_count).map(|_| create_test_property()).collect(),
            ..Default::default()
        };

        let cost = budget.estimate_cost(&diff);

        // Property: Cost should never exceed max_ticks if budget allows
        if budget.can_afford(&diff) {
            prop_assert!(cost <= budget.max_ticks);
        }
    }

    #[test]
    fn prop_confidence_in_valid_range(
        pattern_conf in 0.0f64..=1.0,
        constraint_conf in 0.0f64..=1.0,
        domain_conf in 0.0f64..=1.0,
        perf_conf in 0.0f64..=1.0
    ) {
        let score = ConfidenceScore {
            pattern_clarity: pattern_conf,
            constraint_adherence: constraint_conf,
            domain_knowledge: domain_conf,
            performance_estimate: perf_conf,
        };

        let confidence = score.compute();

        // Property: Confidence always in [0, 1]
        prop_assert!(confidence >= 0.0);
        prop_assert!(confidence <= 1.0);
    }
}
```

### 8.4 Test Scenarios (Critical Paths)

| # | Scenario | Expected Outcome |
|---|----------|------------------|
| 1 | Generate proposal for new class within budget | ✅ Accepted, ticks ≤ 8 |
| 2 | Generate proposal that exceeds tick budget | ❌ Rejected (Q3 violation) |
| 3 | Generate proposal removing protected class | ❌ Rejected (Guard violation) |
| 4 | Generate proposal violating doctrine | ❌ Rejected (Doctrine violation) |
| 5 | Generate proposal with confidence < 0.6 | ⚠️ Flagged for review |
| 6 | Learning system adapts after 10 rejections | ✅ Prompt emphasis increased |
| 7 | Rate limiter blocks >N proposals/hour | ❌ Rate limit error |
| 8 | Cost controller blocks oversized prompts | ❌ Token budget error |
| 9 | LLM timeout triggers graceful fallback | ⚠️ Timeout error, no proposal |
| 10 | Cache hit returns cached response | ✅ Fast response, no LLM call |

---

## 9. Implementation Outline

### 9.1 Module Structure

```
rust/knhk-closed-loop/src/
├── proposer.rs            # Main LLM proposer orchestrator
├── prompt_engine.rs       # Constraint-aware prompt generation
├── validator_llm.rs       # Post-hoc LLM output validation
├── learning.rs            # Learning loop for proposal outcomes
└── llm/
    ├── client.rs          # LLM API client (Ollama, OpenAI, etc.)
    ├── constraints.rs     # Guided decoding constraints
    └── cache.rs           # Prompt/response caching
```

### 9.2 Core Traits

```rust
// proposer.rs
#[async_trait]
pub trait LLMProposer: Send + Sync {
    async fn generate_proposal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &[GuardProfile],
    ) -> Result<Proposal>;

    async fn validate_proposal(
        &self,
        proposal: &Proposal,
    ) -> Result<ValidationReport>;

    async fn record_outcome(
        &self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()>;
}

// prompt_engine.rs
pub trait PromptBuilder: Send + Sync {
    fn build_system_prompt(&self, sector: &Sector) -> String;
    fn build_constraint_section(&self, request: &ProposalRequest) -> String;
    fn build_context_section(&self, ontology: &OntologySnapshot) -> String;
    fn build_full_prompt(&self, request: &ProposalRequest) -> String;
}

// validator_llm.rs
#[async_trait]
pub trait ProposalValidator: Send + Sync {
    async fn validate_schema(&self, proposal: &Proposal) -> Result<()>;
    async fn validate_invariants(&self, proposal: &Proposal) -> Result<()>;
    async fn validate_doctrines(&self, proposal: &Proposal) -> Result<()>;
    async fn validate_guards(&self, proposal: &Proposal) -> Result<()>;
    async fn validate_all(&self, proposal: &Proposal) -> Result<ValidationReport>;
}

// learning.rs
pub trait LearningStrategy: Send + Sync {
    fn record_outcome(&mut self, outcome: ProposalOutcome) -> Result<()>;
    fn get_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample>;
    fn update_metrics(&mut self);
    fn adapt_prompts(&mut self) -> Result<()>;
}
```

---

## 10. Next Steps

### Phase 1: Basic Implementation (Weeks 1-2)
- [ ] Implement `PromptEngine` with constraint encoding
- [ ] Implement basic `LLMClient` for Ollama
- [ ] Implement `ProposalParser` to parse JSON responses
- [ ] Implement `ValidatorLLM` with schema/invariant checks
- [ ] Write unit tests for all modules

### Phase 2: Integration (Weeks 3-4)
- [ ] Integrate LLM proposer into MAPE-K coordinator
- [ ] Implement `LearningSystem` with corpus management
- [ ] Add rate limiting and cost control
- [ ] Add OpenTelemetry instrumentation
- [ ] Write integration tests

### Phase 3: Production Hardening (Weeks 5-6)
- [ ] Implement guided decoding for Q3/Q4 constraints
- [ ] Add prompt caching
- [ ] Add timeout handling and graceful degradation
- [ ] Implement sector-specific prompt templates
- [ ] Write property-based tests

### Phase 4: Validation & Deployment (Weeks 7-8)
- [ ] End-to-end testing with real patterns
- [ ] Weaver schema validation integration
- [ ] Performance benchmarking (proposal generation < 5s)
- [ ] Security audit (prompt injection prevention)
- [ ] Documentation and runbooks

---

## 11. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Proposal Acceptance Rate | >70% | % passing validation |
| First-Try Success Rate | >50% | % accepted without retry |
| Q3 Violation Rate | <10% | % rejected for performance |
| Doctrine Violation Rate | <15% | % rejected for doctrines |
| Avg Confidence (Accepted) | >0.75 | Mean confidence score |
| Avg Confidence (Rejected) | <0.60 | Mean confidence score |
| Generation Latency | <5s | Time to generate proposal |
| Token Efficiency | <10K tokens/proposal | Prompt + response size |

---

## 12. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| LLM hallucinates invalid proposals | High | Defense-in-depth validation |
| LLM violates Q3 performance budget | High | Guided decoding + post-hoc check |
| Prompt injection attacks | Critical | Input sanitization, constrained generation |
| High token costs | Medium | Caching, rate limiting, cost controls |
| Slow LLM response times | Medium | Timeout handling, async processing |
| Learning corpus bias | Low | Diverse sector examples, regular audits |

---

## Conclusion

This design provides a **production-grade LLM proposer** with:
- ✅ Defense-in-depth constraint enforcement
- ✅ Continuous learning from outcomes
- ✅ Production hardening (rate limits, costs, timeouts)
- ✅ Full observability (OTEL spans/metrics)
- ✅ Integration with existing MAPE-K architecture

**Next Action**: Begin Phase 1 implementation with `PromptEngine` and basic `LLMClient`.

---

**Document Status**: ✅ Complete Implementation Design
**Last Updated**: 2025-11-16
**Next Milestone**: Phase 1 Implementation (PromptEngine + LLMClient)
**Estimated Effort**: 8 weeks for Phases 1-4
