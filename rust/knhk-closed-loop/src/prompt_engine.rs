// Prompt Engine: Constraint-aware prompt generation for LLM proposer
// Encodes Q1-Q5 invariants, doctrines, guards, and performance budgets into prompts

use crate::invariants::HardInvariants;
use crate::proposer::{GuardProfile, PerformanceBudget, ProposalRequest, Sector};
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, PromptEngineError>;

#[derive(Debug, thiserror::Error)]
pub enum PromptEngineError {
    #[error("Template rendering failed: {0}")]
    TemplateRenderFailed(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Prompt builder for constraint-aware LLM prompting
pub struct PromptEngine {
    sector_templates: HashMap<Sector, SectorTemplate>,
    emphasis_weights: HashMap<String, f64>,
    example_count: usize,
}

impl PromptEngine {
    pub fn new() -> Self {
        let mut sector_templates = HashMap::new();

        // Initialize sector-specific templates
        sector_templates.insert(Sector::Finance, SectorTemplate::finance());
        sector_templates.insert(Sector::Healthcare, SectorTemplate::healthcare());
        sector_templates.insert(Sector::Manufacturing, SectorTemplate::manufacturing());
        sector_templates.insert(Sector::Logistics, SectorTemplate::logistics());
        sector_templates.insert(Sector::Generic, SectorTemplate::generic());

        PromptEngine {
            sector_templates,
            emphasis_weights: HashMap::new(),
            example_count: 3, // Default: 3 few-shot examples
        }
    }

    /// Build the complete prompt for a proposal request
    pub fn build_full_prompt(&self, request: &ProposalRequest) -> Result<String> {
        let mut sections = Vec::new();

        // System prompt (role definition)
        sections.push(self.build_system_prompt(&request.sector)?);

        // Constraint section (Q1-Q5, doctrines, guards)
        sections.push(self.build_constraint_section(request)?);

        // Pattern section (observed pattern description)
        sections.push(self.build_pattern_section(request)?);

        // Context section (relevant ontology excerpt)
        sections.push(self.build_context_section(request)?);

        // Output schema
        sections.push(self.build_output_schema()?);

        // Few-shot examples
        sections.push(self.build_few_shot_section(&request.sector)?);

        Ok(sections.join("\n\n"))
    }

    pub fn build_system_prompt(&self, sector: &Sector) -> Result<String> {
        let template = self.sector_templates.get(sector).ok_or_else(|| {
            PromptEngineError::MissingField(format!("sector template: {}", sector))
        })?;

        Ok(format!(
            r#"You are an ontology evolution assistant for the KNHK knowledge graph system.
Your task is to propose minimal, safe ontology changes (ΔΣ) based on observed patterns.

SECTOR: {}
SECTOR CONTEXT: {}

CRITICAL RULES:
- Propose MINIMAL changes (prefer evolution over revolution)
- Explain WHY each change is needed
- Explain HOW constraints are satisfied
- Provide confidence score (0.0-1.0) based on pattern clarity and constraint satisfaction
- NEVER violate hard invariants (Q1-Q5)
- NEVER remove protected classes or properties
- ALWAYS stay within performance budget"#,
            sector, template.context
        ))
    }

    pub fn build_constraint_section(&self, request: &ProposalRequest) -> Result<String> {
        let mut section = String::from("CRITICAL CONSTRAINTS (MUST NEVER VIOLATE):\n\n");

        // 1. Hard Invariants (Q1-Q5)
        section.push_str("1. Hard Invariants (Q1-Q5):\n");
        section.push_str(&self.format_invariants(&request.invariants));
        section.push_str("\n");

        // 2. Sector Doctrines
        section.push_str(&format!("2. Sector Doctrines ({}):\n", request.sector));
        if request.doctrines.is_empty() {
            section.push_str("   - No sector-specific doctrines\n");
        } else {
            for doctrine in &request.doctrines {
                section.push_str(&format!("   - {}: {}\n", doctrine.id, doctrine.description));
            }
        }
        section.push_str("\n");

        // 3. Guard Constraints
        section.push_str("3. Guard Constraints (IMMUTABLE BOUNDARIES):\n");
        section.push_str(&self.format_guards(&request.guard_profile));
        section.push_str("\n");

        // 4. Performance Budget
        let emphasis = self
            .emphasis_weights
            .get("performance_budget")
            .unwrap_or(&1.0);
        let header = if *emphasis > 1.5 {
            "4. ⚠️ CRITICAL PERFORMANCE BUDGET (FREQUENTLY VIOLATED):"
        } else {
            "4. Performance Budget:"
        };

        section.push_str(header);
        section.push_str("\n");
        section.push_str(&self.format_performance_budget(&request.performance_budget));
        section.push_str("\n");

        // 5. Immutability Rules
        section.push_str("5. Immutability Rules:\n");
        section.push_str("   - Cannot modify historical data (Q1 enforcement)\n");
        section.push_str("   - Cannot remove protected classes/properties (guard enforcement)\n");
        section.push_str("   - All proposals must be reversible (rollback capability)\n");

        Ok(section)
    }

    fn format_invariants(&self, invariants: &HardInvariants) -> String {
        format!(
            r#"   - Q1: No Retrocausation - Time flows forward only, no temporal cycles
   - Q2: Type Soundness - All properties must have valid domain/range types
   - Q3: Guard Preservation - Hot path operations must have ≤8 execution steps
   - Q4: SLO Compliance - Hot path execution time must be ≤8 CPU ticks
   - Q5: Performance Bounds - No performance regression >10% on existing benchmarks
   Current status: {}"#,
            {
                if invariants.all_preserved() {
                    "✅ All invariants preserved".to_string()
                } else {
                    format!("⚠️ Violations: {:?}", invariants.which_violated())
                }
            }
        )
    }

    fn format_guards(&self, guards: &GuardProfile) -> String {
        format!(
            r#"   - Protected classes (CANNOT REMOVE): {}
   - Protected properties (CANNOT REMOVE): {}
   - Maximum run length: {} steps
   - Performance tier: {:?}"#,
            if guards.protected_classes.is_empty() {
                "none".to_string()
            } else {
                guards.protected_classes.join(", ")
            },
            if guards.protected_properties.is_empty() {
                "none".to_string()
            } else {
                guards.protected_properties.join(", ")
            },
            guards.max_run_len,
            guards.performance_tier
        )
    }

    fn format_performance_budget(&self, budget: &PerformanceBudget) -> String {
        format!(
            r#"   - Current tick consumption: {}/{} ticks
   - Remaining budget: {} ticks
   - Cost estimates:
     * New class: ~{} ticks overhead
     * New property: ~{} ticks overhead
     * New validation: ~{} ticks overhead
   - YOUR BUDGET FOR THIS PROPOSAL: {} ticks"#,
            budget.consumed_ticks,
            budget.max_ticks,
            budget.remaining_ticks,
            budget.cost_per_class,
            budget.cost_per_property,
            budget.cost_per_validation,
            budget.remaining_ticks
        )
    }

    pub fn build_pattern_section(&self, request: &ProposalRequest) -> Result<String> {
        let pattern = &request.pattern;

        Ok(format!(
            r#"OBSERVED PATTERN:
{}

Pattern Details:
- Sector: {}
- Confidence: {:.2}
- Detected At: {}

Recommended Action: {:?}"#,
            pattern.name,
            request.sector,
            pattern.confidence,
            pattern.detected_at,
            pattern.recommended_action
        ))
    }

    pub fn build_context_section(&self, request: &ProposalRequest) -> Result<String> {
        // Extract relevant ontology context from snapshot
        let (relevant_classes, relevant_properties) = Self::extract_relevant_context(request);

        let mut section = format!(
            r#"CURRENT ONTOLOGY SNAPSHOT:
Snapshot ID: {}

Relevant Classes:"#,
            request.current_snapshot_id
        );

        if relevant_classes.is_empty() {
            section.push_str("\n(No directly related classes found in current snapshot)");
        } else {
            for class in &relevant_classes {
                section.push_str(&format!("\n- {}", class));
            }
        }

        section.push_str("\n\nRelevant Properties:");

        if relevant_properties.is_empty() {
            section.push_str("\n(No directly related properties found in current snapshot)");
        } else {
            for prop in &relevant_properties {
                section.push_str(&format!("\n- {}", prop));
            }
        }

        section.push_str(&format!(
            r#"

Current Performance Profile:
- Hot path tick count: {}/{}
- Remaining budget: {} ticks
- Performance tier: {:?}"#,
            request.performance_budget.consumed_ticks,
            request.performance_budget.max_ticks,
            request.performance_budget.remaining_ticks,
            request.guard_profile.performance_tier
        ));

        Ok(section)
    }

    /// Extract relevant ontology context based on the pattern
    fn extract_relevant_context(request: &ProposalRequest) -> (Vec<String>, Vec<String>) {
        let mut classes = Vec::new();
        let mut properties = Vec::new();

        // Extract from pattern name using keyword analysis
        let description = request.pattern.name.to_lowercase();

        // Heuristic: Look for class-like terms (capitalized words, "account", "order", etc.)
        // In production, this would query the actual ontology snapshot
        if description.contains("account") {
            classes.push("knhk:Account (base class for all accounts)".to_string());
            properties.push("knhk:accountId → xsd:string (required)".to_string());
            properties.push("knhk:balance → xsd:decimal (optional)".to_string());
        }

        if description.contains("order") || description.contains("transaction") {
            classes.push("knhk:Order (represents customer orders)".to_string());
            properties.push("knhk:orderId → xsd:string (required)".to_string());
            properties.push("knhk:orderDate → xsd:dateTime (required)".to_string());
        }

        if description.contains("patient") || description.contains("medical") {
            classes.push("health:Patient (medical patient record)".to_string());
            properties.push("health:patientId → xsd:string (required)".to_string());
            properties.push("health:medicalHistory → xsd:string (optional)".to_string());
        }

        if description.contains("equipment") || description.contains("machine") {
            classes.push("mfg:Equipment (manufacturing equipment)".to_string());
            properties.push("mfg:equipmentId → xsd:string (required)".to_string());
            properties.push("mfg:manufacturer → xsd:string (optional)".to_string());
        }

        // Add sector-specific context
        match request.sector {
            Sector::Finance => {
                if classes.is_empty() {
                    classes.push("finance:Account (financial account base)".to_string());
                }
            }
            Sector::Healthcare => {
                if classes.is_empty() {
                    classes.push("health:MedicalRecord (patient medical data)".to_string());
                }
            }
            Sector::Manufacturing => {
                if classes.is_empty() {
                    classes.push("mfg:ProductionLine (manufacturing line)".to_string());
                }
            }
            Sector::Logistics => {
                if classes.is_empty() {
                    classes.push("logistics:Shipment (delivery tracking)".to_string());
                }
            }
            Sector::Generic => {}
        }

        (classes, properties)
    }

    pub fn build_output_schema(&self) -> Result<String> {
        Ok(r#"OUTPUT FORMAT (JSON):
You must respond with ONLY valid JSON in this exact format:

{
  "reasoning": "string (required) - Explain WHY this change is needed and HOW constraints are satisfied",
  "confidence": "number (required) - 0.0 to 1.0",
  "estimated_ticks": "integer (required) - Predicted execution time after change (MUST be ≤ remaining budget)",
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
        "cardinality": "string (required) - One|ZeroOrOne|ZeroOrMore|OneOrMore"
      }
    ],
    "removed_classes": ["string"] (optional - URIs to remove, BUT AVOID IF PROTECTED),
    "removed_properties": ["string"] (optional - URIs to remove, BUT AVOID IF PROTECTED),
    "modified_shapes": [
      {
        "uri": "string (required) - SHACL shape URI",
        "added_constraints": ["string"] (optional),
        "removed_constraints": ["string"] (optional)
      }
    ]
  },
  "doctrines_satisfied": ["string"] (required - Doctrine IDs that this proposal satisfies),
  "invariants_satisfied": ["string"] (required - List which Q1-Q5 invariants are preserved),
  "rollback_plan": "string (required) - How to undo this change if needed)"
}

IMPORTANT:
- Do NOT include any text before or after the JSON
- Ensure all strings are properly escaped
- Ensure all required fields are present
- Ensure estimated_ticks ≤ remaining budget
- Ensure no protected classes/properties are removed"#.to_string())
    }

    pub fn build_few_shot_section(&self, sector: &Sector) -> Result<String> {
        let template = self.sector_templates.get(sector).ok_or_else(|| {
            PromptEngineError::MissingField(format!("sector template: {}", sector))
        })?;

        let mut section = String::from("EXAMPLES:\n\n");

        for (i, example) in template.examples.iter().enumerate() {
            section.push_str(&format!("EXAMPLE {}:\n", i + 1));
            section.push_str(example);
            section.push_str("\n\n");
        }

        Ok(section)
    }

    pub fn increase_emphasis(&mut self, section: &str, weight: f64) {
        self.emphasis_weights.insert(section.to_string(), weight);
    }

    pub fn set_example_count(&mut self, count: usize) {
        self.example_count = count;
    }
}

impl Default for PromptEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Sector-specific prompt templates
#[derive(Clone, Debug)]
pub struct SectorTemplate {
    pub context: String,
    pub examples: Vec<String>,
}

impl SectorTemplate {
    pub fn finance() -> Self {
        SectorTemplate {
            context: r#"Finance sector focuses on:
- Regulatory compliance (SOX, Basel III, GDPR)
- Audit trails (immutable, timestamped)
- Approval chains (two-step verification)
- Transaction integrity
- Real-time fraud detection (<100ms SLA)"#.to_string(),
            examples: vec![
                r#"Pattern: "Observed 15 retirement account variations (401k, IRA, Roth IRA)"
Proposal: {
  "reasoning": "Finance sector requires specialized account types for regulatory compliance. Adding RetirementAccount as subclass of Account satisfies approval chain requirements (FIN-001) and maintains audit trails (FIN-002). Adds only 1 tick overhead.",
  "confidence": 0.85,
  "estimated_ticks": 7,
  "delta_sigma": {
    "added_classes": [{
      "uri": "finance:RetirementAccount",
      "label": "Retirement Account",
      "subclass_of": "finance:Account",
      "properties_required": ["account_id", "tax_status"]
    }]
  }
}
Result: ✅ ACCEPTED"#.to_string(),
            ],
        }
    }

    pub fn healthcare() -> Self {
        SectorTemplate {
            context: r#"Healthcare sector focuses on:
- Patient safety (HIPAA privacy)
- Clinical validation (protocols require review)
- Medical record immutability
- Encryption requirements
- Diagnosis coding (ICD-10)"#.to_string(),
            examples: vec![
                r#"Pattern: "New diabetes treatment evidence (SGLT2 inhibitors)"
Proposal: {
  "reasoning": "Clinical evidence supports adding SGLT2Inhibitor medication class. Requires clinical review (HEALTH-002) before deployment. No retroactive changes to patient records (Q1).",
  "confidence": 0.75,
  "estimated_ticks": 6,
  "delta_sigma": {
    "added_classes": [{
      "uri": "health:SGLT2Inhibitor",
      "label": "SGLT2 Inhibitor Medication",
      "subclass_of": "health:Medication",
      "properties_required": ["drug_name", "dosage", "contraindications"]
    }]
  }
}
Result: ⚠️ REQUIRES CLINICAL REVIEW"#.to_string(),
            ],
        }
    }

    pub fn manufacturing() -> Self {
        SectorTemplate {
            context: r#"Manufacturing sector focuses on:
- Equipment safety interlocks
- Quality control standards
- Regulatory certification
- Maintenance schedules
- Production line simulation"#.to_string(),
            examples: vec![
                r#"Pattern: "New robotic assembly equipment type"
Proposal: {
  "reasoning": "Add RoboticAssembly class for new equipment. Preserves safety interlocks (protected property). Requires certification before deployment.",
  "confidence": 0.80,
  "estimated_ticks": 7,
  "delta_sigma": {
    "added_classes": [{
      "uri": "mfg:RoboticAssembly",
      "label": "Robotic Assembly Equipment",
      "subclass_of": "mfg:Equipment",
      "properties_required": ["equipment_id", "safety_certified"]
    }]
  }
}
Result: ✅ ACCEPTED"#.to_string(),
            ],
        }
    }

    pub fn logistics() -> Self {
        SectorTemplate {
            context: r#"Logistics sector focuses on:
- Delivery SLAs
- Route optimization (<1s for real-time)
- Inventory accuracy
- Real-time tracking
- FIFO/LIFO constraints"#.to_string(),
            examples: vec![
                r#"Pattern: "Drone delivery service observed"
Proposal: {
  "reasoning": "Add DroneDelivery shipping method. Maintains delivery SLA constraints and tracking requirements. Adds 1 tick overhead.",
  "confidence": 0.78,
  "estimated_ticks": 6,
  "delta_sigma": {
    "added_classes": [{
      "uri": "logistics:DroneDelivery",
      "label": "Drone Delivery Service",
      "subclass_of": "logistics:ShippingMethod",
      "properties_required": ["tracking_id", "delivery_zone"]
    }]
  }
}
Result: ✅ ACCEPTED"#.to_string(),
            ],
        }
    }

    pub fn generic() -> Self {
        SectorTemplate {
            context: "Generic sector with minimal constraints".to_string(),
            examples: vec![r#"Pattern: "New entity type observed"
Proposal: {
  "reasoning": "Add new class to represent observed entity pattern. Minimal overhead.",
  "confidence": 0.70,
  "estimated_ticks": 5,
  "delta_sigma": {
    "added_classes": [{
      "uri": "knhk:NewEntity",
      "label": "New Entity",
      "subclass_of": "owl:Thing",
      "properties_required": ["id"]
    }]
  }
}
Result: ✅ ACCEPTED"#
                .to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{DetectedPattern, PatternAction};
    use chrono::Utc;

    fn create_test_request() -> ProposalRequest {
        ProposalRequest {
            pattern: DetectedPattern {
                name: "Test Pattern".to_string(),
                confidence: 0.85,
                detected_at: Utc::now().timestamp_millis() as u64,
                evidence_count: 5,
                evidence_ids: vec!["obs-1".to_string(), "obs-2".to_string()],
                recommended_action: PatternAction::ProposeChange {
                    description: "Add new class".to_string(),
                },
            },
            sector: Sector::Finance,
            current_snapshot_id: "snapshot-123".to_string(),
            doctrines: vec![],
            invariants: HardInvariants::default(),
            guard_profile: GuardProfile {
                id: "guard-1".to_string(),
                name: "Test Guard".to_string(),
                protected_classes: vec!["Account".to_string()],
                protected_properties: vec!["account_id".to_string()],
                max_run_len: 8,
                performance_tier: PerformanceTier::HotPath,
            },
            performance_budget: PerformanceBudget::new(8, 5),
        }
    }

    #[test]
    fn test_build_system_prompt() {
        let engine = PromptEngine::new();
        let prompt = engine.build_system_prompt(&Sector::Finance).unwrap();

        assert!(prompt.contains("ontology evolution assistant"));
        assert!(prompt.contains("Finance"));
        assert!(prompt.contains("NEVER violate hard invariants"));
    }

    #[test]
    fn test_build_constraint_section() {
        let engine = PromptEngine::new();
        let request = create_test_request();

        let section = engine.build_constraint_section(&request).unwrap();

        assert!(section.contains("Q1: No Retrocausation"));
        assert!(section.contains("Q2: Type Soundness"));
        assert!(section.contains("Q3: Guard Preservation"));
        assert!(section.contains("Q4: SLO Compliance"));
        assert!(section.contains("Q5: Performance Bounds"));
        assert!(section.contains("Protected classes"));
        assert!(section.contains("Performance Budget"));
    }

    #[test]
    fn test_performance_budget_formatting() {
        let engine = PromptEngine::new();
        let budget = PerformanceBudget::new(8, 5);

        let formatted = engine.format_performance_budget(&budget);

        assert!(formatted.contains("5/8 ticks"));
        assert!(formatted.contains("Remaining budget: 3 ticks"));
    }

    #[test]
    fn test_emphasis_weight_application() {
        let mut engine = PromptEngine::new();
        engine.increase_emphasis("performance_budget", 2.0);

        let request = create_test_request();
        let section = engine.build_constraint_section(&request).unwrap();

        assert!(section.contains("⚠️ CRITICAL PERFORMANCE BUDGET"));
    }

    #[test]
    fn test_full_prompt_structure() {
        let engine = PromptEngine::new();
        let request = create_test_request();

        let prompt = engine.build_full_prompt(&request).unwrap();

        // Verify all major sections present
        assert!(prompt.contains("ontology evolution assistant"));
        assert!(prompt.contains("CRITICAL CONSTRAINTS"));
        assert!(prompt.contains("OBSERVED PATTERN"));
        assert!(prompt.contains("CURRENT ONTOLOGY SNAPSHOT"));
        assert!(prompt.contains("OUTPUT FORMAT"));
        assert!(prompt.contains("EXAMPLES"));
    }

    #[test]
    fn test_sector_templates() {
        let finance = SectorTemplate::finance();
        assert!(finance.context.contains("Regulatory compliance"));
        assert!(!finance.examples.is_empty());

        let healthcare = SectorTemplate::healthcare();
        assert!(healthcare.context.contains("HIPAA"));
        assert!(!healthcare.examples.is_empty());
    }
}
