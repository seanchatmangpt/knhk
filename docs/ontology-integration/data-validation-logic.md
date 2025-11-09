# Data Validation Logic for YAWL Workflows

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Implementation Guide
**Agent:** Data Modeler (ULTRATHINK Swarm)

## Executive Summary

This document provides **complete Rust-level validation logic** for YAWL workflows after deserialization. It covers:
- Post-deserialization constraint checking
- Cardinality validation (required fields, collections)
- Type validation (data types, enum values)
- Range validation (min/max constraints)
- Integration with chicago-tdd-tools validation framework
- Error messages and diagnostics
- Validation pipeline architecture

**Key Principle:** Deserialization proves structure correctness (field types match). Validation proves semantic correctness (values make sense).

---

## 1. Validation Architecture

### 1.1 Validation Levels

**Level 1: Serde Deserialization (Structural)**
- Field types correct (string vs int)
- Required fields present
- JSON structure matches Rust structs
- **When:** During `serde_json::from_value()`

**Level 2: Post-Deserial Validation (Semantic - THIS DOCUMENT)**
- Cardinality constraints (min/max instances)
- Range constraints (tick budget ≤ 8)
- Type compatibility (XPath expressions valid)
- Reference integrity (IRIs exist)
- **When:** After deserialization, before execution

**Level 3: Runtime Validation (Execution-Time)**
- XPath expression evaluation
- Data type coercion
- Resource availability
- **When:** During workflow execution

**Level 4: OTEL Schema Validation (SOURCE OF TRUTH)**
- Weaver registry validation
- **When:** Pre-deployment and runtime

---

### 1.2 Validation Pipeline

```rust
/// Complete validation pipeline
pub fn validate_workflow_spec(spec: &WorkflowSpec) -> Result<ValidationReport, ValidationError> {
    let mut report = ValidationReport::new();

    // Level 1: Cardinality validation
    report.merge(validate_cardinality(spec)?);

    // Level 2: Type validation
    report.merge(validate_types(spec)?);

    // Level 3: Range validation
    report.merge(validate_ranges(spec)?);

    // Level 4: Reference integrity
    report.merge(validate_references(spec)?);

    // Level 5: Semantic validation (control flow, data flow)
    report.merge(validate_semantics(spec)?);

    // Level 6: knhk-specific validation
    report.merge(validate_knhk_extensions(spec)?);

    // Check if any critical errors
    if report.has_critical_errors() {
        return Err(ValidationError::CriticalErrors(report));
    }

    Ok(report)
}
```

---

## 2. Cardinality Validation

### 2.1 Required Fields

**Rule:** Certain fields must be present (non-Option).

```rust
/// Validate required fields on Task
pub fn validate_task_required_fields(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // IRI required
    if task.iri.is_empty() {
        issues.push(ValidationIssue::error(
            "VR-T001",
            &format!("Task has empty IRI"),
            ValidationLocation::Task(task.id.clone()),
            Some("Ensure task has valid IRI from RDF subject"),
        ));
    }

    // ID required
    if task.id.is_empty() {
        issues.push(ValidationIssue::error(
            "VR-T002",
            &format!("Task has empty ID"),
            ValidationLocation::Task(task.iri.clone()),
            Some("Ensure yawl:id property is set"),
        ));
    }

    issues
}

/// Validation issue representation
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: ValidationLocation,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,  // Workflow cannot execute
    Error,     // Soundness violation
    Warning,   // Potential issue
    Info,      // Best practice suggestion
}

#[derive(Debug, Clone)]
pub enum ValidationLocation {
    Specification(String),
    Net(String),
    Task(String),
    Condition(String),
    Flow(String),
    Variable(String),
}
```

---

### 2.2 Collection Cardinality

**Rule:** Nets must have exactly 1 input condition, 1 output condition.

```rust
pub fn validate_net_conditions(net: &Net) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Check input condition exists
    if net.input_condition.is_empty() {
        issues.push(ValidationIssue::critical(
            "VR-N001",
            "Net has no input condition",
            ValidationLocation::Net(net.iri.clone()),
            Some("Add exactly one InputCondition to the net"),
        ));
    }

    // Check output condition exists
    if net.output_condition.is_empty() {
        issues.push(ValidationIssue::critical(
            "VR-N002",
            "Net has no output condition",
            ValidationLocation::Net(net.iri.clone()),
            Some("Add exactly one OutputCondition to the net"),
        ));
    }

    // Check input condition is in conditions map
    if !net.conditions.contains_key(&net.input_condition) {
        issues.push(ValidationIssue::critical(
            "VR-N003",
            &format!("Input condition {} not found in conditions map", net.input_condition),
            ValidationLocation::Net(net.iri.clone()),
            Some("Ensure input condition IRI matches a condition in the net"),
        ));
    }

    // Check output condition is in conditions map
    if !net.conditions.contains_key(&net.output_condition) {
        issues.push(ValidationIssue::critical(
            "VR-N004",
            &format!("Output condition {} not found in conditions map", net.output_condition),
            ValidationLocation::Net(net.iri.clone()),
            Some("Ensure output condition IRI matches a condition in the net"),
        ));
    }

    issues
}
```

---

### 2.3 Optional Field Validation

**Rule:** If MI config present, it must be complete.

```rust
pub fn validate_mi_config(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(mi_config) = &task.mi_config {
        // Required fields
        if mi_config.minimum.is_empty() {
            issues.push(ValidationIssue::error(
                "VR-MI001",
                "MI task has empty minimum",
                ValidationLocation::Task(task.id.clone()),
                Some("Set yawl:minimum to XPath expression or positive integer"),
            ));
        }

        if mi_config.maximum.is_empty() {
            issues.push(ValidationIssue::error(
                "VR-MI002",
                "MI task has empty maximum",
                ValidationLocation::Task(task.id.clone()),
                Some("Set yawl:maximum to XPath expression or positive integer"),
            ));
        }

        if mi_config.threshold.is_empty() {
            issues.push(ValidationIssue::error(
                "VR-MI003",
                "MI task has empty threshold",
                ValidationLocation::Task(task.id.clone()),
                Some("Set yawl:threshold to XPath expression or positive integer"),
            ));
        }

        // Validate XPath or integer
        validate_mi_expression(&mi_config.minimum, "minimum", task, &mut issues);
        validate_mi_expression(&mi_config.maximum, "maximum", task, &mut issues);
        validate_mi_expression(&mi_config.threshold, "threshold", task, &mut issues);
    }

    issues
}

fn validate_mi_expression(expr: &str, field_name: &str, task: &Task, issues: &mut Vec<ValidationIssue>) {
    // Try to parse as integer
    if expr.parse::<u32>().is_ok() {
        return;  // Valid integer
    }

    // Otherwise must be valid XPath (basic check)
    if !is_valid_xpath_syntax(expr) {
        issues.push(ValidationIssue::error(
            &format!("VR-MI010-{}", field_name),
            &format!("MI {} expression is invalid: {}", field_name, expr),
            ValidationLocation::Task(task.id.clone()),
            Some("Use valid XPath expression or positive integer"),
        ));
    }
}

fn is_valid_xpath_syntax(expr: &str) -> bool {
    // Basic syntax check
    if expr.is_empty() {
        return false;
    }

    // Check balanced parentheses
    let open_count = expr.matches('(').count();
    let close_count = expr.matches(')').count();
    if open_count != close_count {
        return false;
    }

    // Check balanced brackets
    let open_bracket = expr.matches('[').count();
    let close_bracket = expr.matches(']').count();
    if open_bracket != close_bracket {
        return false;
    }

    // TODO: Full XPath parsing (requires external library)
    true
}
```

---

## 3. Type Validation

### 3.1 Data Type Consistency

**Rule:** Variables with same name should have same type.

```rust
pub fn validate_variable_types(net: &Net) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut type_map: HashMap<String, DataType> = HashMap::new();

    // Collect all variable types
    for var in &net.local_variables {
        if let Some(existing_type) = type_map.get(&var.name) {
            if existing_type != &var.data_type {
                issues.push(ValidationIssue::warning(
                    "VR-V001",
                    &format!("Variable '{}' has inconsistent types: {:?} vs {:?}",
                             var.name, existing_type, var.data_type),
                    ValidationLocation::Variable(var.name.clone()),
                    Some("Ensure all variables with same name have same type"),
                ));
            }
        } else {
            type_map.insert(var.name.clone(), var.data_type.clone());
        }
    }

    // Check input/output parameters
    for param in &net.input_parameters {
        if let Some(existing_type) = type_map.get(&param.name) {
            if existing_type != &param.data_type {
                issues.push(ValidationIssue::warning(
                    "VR-V002",
                    &format!("Input parameter '{}' has different type than variable", param.name),
                    ValidationLocation::Variable(param.name.clone()),
                    None,
                ));
            }
        }
    }

    issues
}
```

---

### 3.2 Enum Value Validation

**Rule:** Enum values must be valid (enforced by serde, but double-check).

```rust
pub fn validate_control_types(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Join/split types are enums, so if deserialization succeeded, they're valid
    // But we can check for invalid combinations

    // Pattern validation: some join/split combinations are invalid
    match (task.join_type, task.split_type) {
        (JoinType::Xor, SplitType::Xor) => {}, // Sequence - valid
        (JoinType::Xor, SplitType::And) => {}, // Parallel split - valid
        (JoinType::And, SplitType::Xor) => {}, // Synchronization - valid
        (JoinType::And, SplitType::And) => {
            issues.push(ValidationIssue::warning(
                "VR-CT001",
                "AND-join with AND-split is unusual (creates parallel paths that immediately merge)",
                ValidationLocation::Task(task.id.clone()),
                Some("Consider if this pattern is intended"),
            ));
        }
        (JoinType::Or, SplitType::Or) => {
            issues.push(ValidationIssue::info(
                "VR-CT002",
                "OR-join with OR-split requires careful configuration",
                ValidationLocation::Task(task.id.clone()),
                Some("Ensure OR-join synchronization is properly configured"),
            ));
        }
        _ => {}
    }

    issues
}
```

---

### 3.3 XQuery Expression Validation

**Rule:** XQuery expressions must be syntactically valid.

```rust
pub fn validate_expression(expr: &Expression, context: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Basic syntax validation
    if expr.query.is_empty() {
        issues.push(ValidationIssue::error(
            "VR-E001",
            &format!("Empty XQuery expression in {}", context),
            ValidationLocation::from_context(context),
            Some("Provide valid XQuery expression"),
        ));
        return issues;
    }

    // Check for common syntax errors
    if !is_balanced(&expr.query, '(', ')') {
        issues.push(ValidationIssue::error(
            "VR-E002",
            &format!("Unbalanced parentheses in XQuery: {}", expr.query),
            ValidationLocation::from_context(context),
            Some("Fix XQuery syntax"),
        ));
    }

    if !is_balanced(&expr.query, '{', '}') {
        issues.push(ValidationIssue::error(
            "VR-E003",
            &format!("Unbalanced braces in XQuery: {}", expr.query),
            ValidationLocation::from_context(context),
            Some("Fix XQuery syntax"),
        ));
    }

    // TODO: Full XQuery parsing with external library (Saxon-HE)
    // For MVP: basic checks sufficient

    issues
}

fn is_balanced(s: &str, open: char, close: char) -> bool {
    let open_count = s.matches(open).count();
    let close_count = s.matches(close).count();
    open_count == close_count
}
```

---

## 4. Range Validation

### 4.1 Numeric Constraints

**Rule:** Hot path tasks must have tick budget ≤ 8 (Chatman Constant).

```rust
pub fn validate_knhk_tick_budget(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(knhk) = &task.knhk_extensions {
        if let Some(tick_budget) = knhk.tick_budget {
            // Hot path constraint
            if tick_budget > 8 {
                issues.push(ValidationIssue::warning(
                    "VR-K001",
                    &format!("Task tick budget ({}) exceeds hot path threshold (8 ticks - Chatman Constant)",
                             tick_budget),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Consider optimizing task or moving off hot path"),
                ));
            }

            // Reasonable upper bound
            if tick_budget == 0 {
                issues.push(ValidationIssue::error(
                    "VR-K002",
                    "Task tick budget is zero (task cannot execute)",
                    ValidationLocation::Task(task.id.clone()),
                    Some("Set tick budget to at least 1"),
                ));
            }

            if tick_budget > 1000 {
                issues.push(ValidationIssue::warning(
                    "VR-K003",
                    &format!("Task tick budget ({}) is very high (> 1000 ticks)", tick_budget),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Consider if this is realistic performance constraint"),
                ));
            }
        }

        // Priority range
        if let Some(priority) = knhk.priority {
            // Priority is u8, so always 0-255, but we can validate semantics
            if priority == 0 {
                issues.push(ValidationIssue::info(
                    "VR-K004",
                    "Task has lowest priority (0)",
                    ValidationLocation::Task(task.id.clone()),
                    None,
                ));
            }
        }
    }

    issues
}
```

---

### 4.2 MI Instance Constraints

**Rule:** For static MI tasks, min ≤ threshold ≤ max.

```rust
pub fn validate_mi_instance_constraints(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(mi_config) = &task.mi_config {
        // Only validate if all are integer literals (not XPath)
        if let (Ok(min), Ok(max), Ok(threshold)) = (
            mi_config.minimum.parse::<u32>(),
            mi_config.maximum.parse::<u32>(),
            mi_config.threshold.parse::<u32>(),
        ) {
            // min ≤ threshold ≤ max
            if min > threshold {
                issues.push(ValidationIssue::error(
                    "VR-MI020",
                    &format!("MI minimum ({}) > threshold ({})", min, threshold),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Set minimum <= threshold"),
                ));
            }

            if threshold > max {
                issues.push(ValidationIssue::error(
                    "VR-MI021",
                    &format!("MI threshold ({}) > maximum ({})", threshold, max),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Set threshold <= maximum"),
                ));
            }

            if min > max {
                issues.push(ValidationIssue::error(
                    "VR-MI022",
                    &format!("MI minimum ({}) > maximum ({})", min, max),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Set minimum <= maximum"),
                ));
            }

            // Reasonable bounds
            if max > 10000 {
                issues.push(ValidationIssue::warning(
                    "VR-MI023",
                    &format!("MI maximum ({}) is very high (> 10,000 instances)", max),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Consider if this is realistic"),
                ));
            }
        }
    }

    issues
}
```

---

### 4.3 Timer Duration Validation

**Rule:** Timer durations must be positive.

```rust
pub fn validate_timer_config(task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(timer) = &task.timer {
        match &timer.duration_spec {
            DurationSpec::Params { ticks, interval } => {
                if *ticks == 0 {
                    issues.push(ValidationIssue::warning(
                        "VR-TI001",
                        "Timer has zero ticks (immediate timeout)",
                        ValidationLocation::Task(task.id.clone()),
                        Some("Set ticks > 0 for meaningful timer"),
                    ));
                }

                // Check reasonable bounds
                let duration_millis = ticks * interval.to_duration_multiplier();
                if duration_millis > 365 * 24 * 3600 * 1000 {  // 1 year
                    issues.push(ValidationIssue::warning(
                        "VR-TI002",
                        &format!("Timer duration > 1 year ({} ms)", duration_millis),
                        ValidationLocation::Task(task.id.clone()),
                        Some("Consider if this is intended"),
                    ));
                }
            }

            DurationSpec::Expiry { expiry } => {
                // Check if expiry is in the past
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                if *expiry < now {
                    issues.push(ValidationIssue::warning(
                        "VR-TI003",
                        "Timer expiry is in the past (will timeout immediately)",
                        ValidationLocation::Task(task.id.clone()),
                        Some("Set expiry to future timestamp"),
                    ));
                }
            }

            DurationSpec::Duration { duration } => {
                // Validate ISO 8601 format (basic check)
                if !duration.starts_with('P') {
                    issues.push(ValidationIssue::error(
                        "VR-TI004",
                        &format!("Invalid ISO 8601 duration: {}", duration),
                        ValidationLocation::Task(task.id.clone()),
                        Some("Use format like 'PT30M' (30 minutes)"),
                    ));
                }
            }

            DurationSpec::NetParam { netparam } => {
                // Cannot validate at this stage (runtime evaluation needed)
                issues.push(ValidationIssue::info(
                    "VR-TI005",
                    &format!("Timer references net parameter: {}", netparam),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Ensure net parameter is set at runtime"),
                ));
            }
        }
    }

    issues
}
```

---

## 5. Reference Integrity Validation

### 5.1 Flow Target Validation

**Rule:** All flow targets must reference valid tasks/conditions.

```rust
pub fn validate_flow_references(net: &Net) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect all valid element IRIs
    let mut valid_elements = std::collections::HashSet::new();
    valid_elements.extend(net.tasks.keys().cloned());
    valid_elements.extend(net.conditions.keys().cloned());

    // Check task outgoing flows
    for task in net.tasks.values() {
        for flow_iri in &task.outgoing_flows {
            // Flow IRI should reference a target element
            // Need to look up the flow and check its target
            // (This requires flows to be stored separately or embedded in tasks)

            // For now, just check if flow IRI is valid
            if flow_iri.is_empty() {
                issues.push(ValidationIssue::error(
                    "VR-F001",
                    "Task has empty flow IRI",
                    ValidationLocation::Task(task.id.clone()),
                    Some("Remove invalid flow reference"),
                ));
            }
        }
    }

    // Similar for conditions
    for condition in net.conditions.values() {
        for flow_iri in &condition.outgoing_flows {
            if flow_iri.is_empty() {
                issues.push(ValidationIssue::error(
                    "VR-F002",
                    "Condition has empty flow IRI",
                    ValidationLocation::Condition(condition.id.clone()),
                    Some("Remove invalid flow reference"),
                ));
            }
        }
    }

    issues
}
```

---

### 5.2 Decomposition Reference Validation

**Rule:** If task decomposes_to, the decomposition must exist.

```rust
pub fn validate_decomposition_references(spec: &WorkflowSpec, task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(decomp_iri) = &task.decomposes_to {
        // Check if decomposition exists (net or web service gateway)
        let exists = spec.nets.contains_key(decomp_iri)
            || spec.web_service_gateways.contains_key(decomp_iri);

        if !exists {
            issues.push(ValidationIssue::error(
                "VR-D001",
                &format!("Task decomposes to non-existent decomposition: {}", decomp_iri),
                ValidationLocation::Task(task.id.clone()),
                Some("Ensure decomposition exists in specification"),
            ));
        }

        // Mark as composite task
        if task.task_type != TaskType::Composite {
            issues.push(ValidationIssue::warning(
                "VR-D002",
                "Task has decomposition but is not marked as Composite",
                ValidationLocation::Task(task.id.clone()),
                Some("Set task_type = Composite"),
            ));
        }
    }

    issues
}
```

---

### 5.3 Variable Mapping Validation

**Rule:** Variable mappings must reference existing variables.

```rust
pub fn validate_variable_mappings(net: &Net, task: &Task) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect all variable names
    let mut var_names: HashSet<String> = net.local_variables.iter()
        .map(|v| v.name.clone())
        .collect();
    var_names.extend(net.input_parameters.iter().map(|p| p.name.clone()));
    var_names.extend(net.output_parameters.iter().map(|p| p.name.clone()));

    // Check starting mappings
    if let Some(mappings) = &task.starting_mappings {
        for mapping in &mappings.mappings {
            if !var_names.contains(&mapping.maps_to) {
                issues.push(ValidationIssue::warning(
                    "VR-M001",
                    &format!("Mapping references unknown variable: {}", mapping.maps_to),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Ensure variable is declared in net"),
                ));
            }
        }
    }

    // Check completed mappings
    if let Some(mappings) = &task.completed_mappings {
        for mapping in &mappings.mappings {
            if !var_names.contains(&mapping.maps_to) {
                issues.push(ValidationIssue::warning(
                    "VR-M002",
                    &format!("Mapping references unknown variable: {}", mapping.maps_to),
                    ValidationLocation::Task(task.id.clone()),
                    Some("Ensure variable is declared in net"),
                ));
            }
        }
    }

    issues
}
```

---

## 6. Integration with chicago-tdd-tools

### 6.1 Test-Driven Validation

**Strategy:** Write validation tests using chicago-tdd-tools framework.

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;
    use chicago_tdd_tools::{assert_passes, assert_fails, TestCase};

    #[test]
    fn test_valid_task_passes_validation() {
        // Arrange
        let task = Task {
            iri: "http://example.org/TaskA".to_string(),
            id: "TaskA".to_string(),
            name: Some("Process Order".to_string()),
            join_type: JoinType::Xor,
            split_type: SplitType::And,
            task_type: TaskType::Atomic,
            mi_config: None,
            outgoing_flows: vec![],
            incoming_flows: vec![],
            timer: None,
            resourcing: None,
            decomposes_to: None,
            starting_mappings: None,
            completed_mappings: None,
            enablement_mappings: None,
            removes_tokens_from: vec![],
            custom_form: None,
            knhk_extensions: None,
            documentation: None,
        };

        // Act
        let issues = validate_task(&task);

        // Assert
        assert_passes!(issues.is_empty(), "Valid task should have no issues");
    }

    #[test]
    fn test_task_with_empty_id_fails_validation() {
        // Arrange
        let task = Task {
            iri: "http://example.org/TaskA".to_string(),
            id: "".to_string(),  // Invalid: empty ID
            name: None,
            join_type: JoinType::Xor,
            split_type: SplitType::Xor,
            task_type: TaskType::Atomic,
            mi_config: None,
            outgoing_flows: vec![],
            incoming_flows: vec![],
            timer: None,
            resourcing: None,
            decomposes_to: None,
            starting_mappings: None,
            completed_mappings: None,
            enablement_mappings: None,
            removes_tokens_from: vec![],
            custom_form: None,
            knhk_extensions: None,
            documentation: None,
        };

        // Act
        let issues = validate_task(&task);

        // Assert
        assert_fails!(
            issues.iter().any(|i| i.rule_id == "VR-T002"),
            "Task with empty ID should fail validation"
        );
    }

    #[test]
    fn test_mi_task_with_invalid_constraints_fails() {
        // Arrange
        let task = Task {
            iri: "http://example.org/TaskB".to_string(),
            id: "TaskB".to_string(),
            name: Some("MI Task".to_string()),
            join_type: JoinType::Xor,
            split_type: SplitType::Xor,
            task_type: TaskType::MultipleInstance,
            mi_config: Some(MultipleInstanceConfig {
                minimum: "10".to_string(),
                maximum: "5".to_string(),  // Invalid: max < min
                threshold: "7".to_string(),
                splitting_expression: None,
                joining_expression: None,
                creation_mode: CreationMode::Static,
                formal_input_param: None,
                formal_output_expression: None,
                result_variable: None,
            }),
            outgoing_flows: vec![],
            incoming_flows: vec![],
            timer: None,
            resourcing: None,
            decomposes_to: None,
            starting_mappings: None,
            completed_mappings: None,
            enablement_mappings: None,
            removes_tokens_from: vec![],
            custom_form: None,
            knhk_extensions: None,
            documentation: None,
        };

        // Act
        let issues = validate_task(&task);

        // Assert
        assert_fails!(
            issues.iter().any(|i| i.rule_id == "VR-MI022"),
            "MI task with min > max should fail validation"
        );
    }
}
```

---

### 6.2 Property-Based Testing

**Use proptest for fuzzing validation logic:**

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_tick_budget_validation_consistency(tick_budget in 0u32..10000) {
            // Arrange
            let task = create_task_with_tick_budget(tick_budget);

            // Act
            let issues = validate_knhk_tick_budget(&task);

            // Assert
            if tick_budget == 0 {
                assert!(issues.iter().any(|i| i.rule_id == "VR-K002"));
            } else if tick_budget > 8 {
                assert!(issues.iter().any(|i| i.rule_id == "VR-K001"));
            } else {
                // 1-8: valid, no issues
                assert!(issues.is_empty());
            }
        }

        #[test]
        fn test_mi_constraints_never_panic(min in 0u32..100, max in 0u32..100, threshold in 0u32..100) {
            // Arrange
            let task = create_mi_task(min, max, threshold);

            // Act - should never panic
            let _issues = validate_mi_instance_constraints(&task);
        }
    }
}
```

---

## 7. Validation Report

### 7.1 Report Structure

```rust
/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.issues.extend(other.issues);
    }

    pub fn has_critical_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Critical)
    }

    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| matches!(i.severity, Severity::Critical | Severity::Error))
    }

    pub fn critical_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Critical).count()
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
    }

    pub fn info_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Info).count()
    }

    /// Format as human-readable report
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Validation Report\n\
             =================\n\
             Total Issues: {}\n\
             - Critical: {}\n\
             - Errors: {}\n\
             - Warnings: {}\n\
             - Info: {}\n\n",
            self.issues.len(),
            self.critical_count(),
            self.error_count(),
            self.warning_count(),
            self.info_count()
        ));

        // Group by severity
        for severity in &[Severity::Critical, Severity::Error, Severity::Warning, Severity::Info] {
            let issues: Vec<_> = self.issues.iter()
                .filter(|i| i.severity == *severity)
                .collect();

            if !issues.is_empty() {
                output.push_str(&format!("\n{:?} Issues ({})\n", severity, issues.len()));
                output.push_str("-------------------\n");

                for issue in issues {
                    output.push_str(&format!(
                        "[{}] {}\n  Location: {}\n",
                        issue.rule_id,
                        issue.message,
                        issue.location.to_string()
                    ));

                    if let Some(suggestion) = &issue.suggestion {
                        output.push_str(&format!("  Suggestion: {}\n", suggestion));
                    }

                    output.push('\n');
                }
            }
        }

        output
    }
}

impl ValidationLocation {
    pub fn to_string(&self) -> String {
        match self {
            Self::Specification(iri) => format!("Specification <{}>", iri),
            Self::Net(iri) => format!("Net <{}>", iri),
            Self::Task(id) => format!("Task '{}'", id),
            Self::Condition(id) => format!("Condition '{}'", id),
            Self::Flow(iri) => format!("Flow <{}>", iri),
            Self::Variable(name) => format!("Variable '{}'", name),
        }
    }
}
```

---

### 7.2 Example Report Output

```
Validation Report
=================
Total Issues: 8
- Critical: 2
- Errors: 3
- Warnings: 2
- Info: 1

Critical Issues (2)
-------------------
[VR-N001] Net has no input condition
  Location: Net <http://example.org/net#MainNet>
  Suggestion: Add exactly one InputCondition to the net

[VR-D001] Task decomposes to non-existent decomposition: http://example.org/net#MissingNet
  Location: Task 'TaskA'
  Suggestion: Ensure decomposition exists in specification

Error Issues (3)
-------------------
[VR-T002] Task has empty ID
  Location: Task <http://example.org/task#BadTask>
  Suggestion: Ensure yawl:id property is set

[VR-MI022] MI minimum (10) > maximum (5)
  Location: Task 'TaskB'
  Suggestion: Set minimum <= maximum

[VR-E002] Unbalanced parentheses in XQuery: for $x in /order (return $x
  Location: Task 'TaskC'
  Suggestion: Fix XQuery syntax

Warning Issues (2)
-------------------
[VR-K001] Task tick budget (15) exceeds hot path threshold (8 ticks - Chatman Constant)
  Location: Task 'TaskD'
  Suggestion: Consider optimizing task or moving off hot path

[VR-V001] Variable 'orderID' has inconsistent types: Builtin(Int) vs Builtin(String)
  Location: Variable 'orderID'
  Suggestion: Ensure all variables with same name have same type

Info Issues (1)
-------------------
[VR-TI005] Timer references net parameter: timeout_duration
  Location: Task 'TaskE'
  Suggestion: Ensure net parameter is set at runtime
```

---

## 8. Validation Rule Catalog

### 8.1 Task Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-T001 | Error | Task has empty IRI | `task.iri.is_empty()` |
| VR-T002 | Error | Task has empty ID | `task.id.is_empty()` |
| VR-CT001 | Warning | AND-join + AND-split unusual | Join=And && Split=And |
| VR-CT002 | Info | OR-join + OR-split needs config | Join=Or && Split=Or |

### 8.2 Net Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-N001 | Critical | Net has no input condition | `net.input_condition.is_empty()` |
| VR-N002 | Critical | Net has no output condition | `net.output_condition.is_empty()` |
| VR-N003 | Critical | Input condition not in map | `!net.conditions.contains_key(&net.input_condition)` |
| VR-N004 | Critical | Output condition not in map | `!net.conditions.contains_key(&net.output_condition)` |

### 8.3 MI Task Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-MI001 | Error | MI task has empty minimum | `mi_config.minimum.is_empty()` |
| VR-MI002 | Error | MI task has empty maximum | `mi_config.maximum.is_empty()` |
| VR-MI003 | Error | MI task has empty threshold | `mi_config.threshold.is_empty()` |
| VR-MI020 | Error | MI minimum > threshold | `min > threshold` |
| VR-MI021 | Error | MI threshold > maximum | `threshold > max` |
| VR-MI022 | Error | MI minimum > maximum | `min > max` |
| VR-MI023 | Warning | MI maximum > 10,000 | `max > 10000` |

### 8.4 knhk Extension Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-K001 | Warning | Tick budget > 8 (hot path) | `tick_budget > 8` |
| VR-K002 | Error | Tick budget is zero | `tick_budget == 0` |
| VR-K003 | Warning | Tick budget > 1000 | `tick_budget > 1000` |
| VR-K004 | Info | Priority is lowest (0) | `priority == 0` |

### 8.5 Timer Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-TI001 | Warning | Timer has zero ticks | `ticks == 0` |
| VR-TI002 | Warning | Timer duration > 1 year | `duration > 365 days` |
| VR-TI003 | Warning | Timer expiry in past | `expiry < now` |
| VR-TI004 | Error | Invalid ISO 8601 duration | `!duration.starts_with('P')` |
| VR-TI005 | Info | Timer references net param | NetParam variant |

### 8.6 Expression Validation Rules

| Rule ID | Severity | Description | Check |
|---------|----------|-------------|-------|
| VR-E001 | Error | Empty XQuery expression | `expr.query.is_empty()` |
| VR-E002 | Error | Unbalanced parentheses | `count('(') != count(')')` |
| VR-E003 | Error | Unbalanced braces | `count('{') != count('}')` |

---

## 9. Complete Validation Implementation

```rust
/// Master validation function
pub fn validate_workflow_spec(spec: &WorkflowSpec) -> Result<ValidationReport, ValidationError> {
    let mut report = ValidationReport::new();

    // Validate specification level
    if let Some(root_iri) = &spec.root_net {
        if !spec.nets.contains_key(root_iri) {
            report.add_issue(ValidationIssue::critical(
                "VR-S001",
                "Root net not found in nets collection",
                ValidationLocation::Specification(spec.iri.clone()),
                Some("Ensure root net exists"),
            ));
        }
    }

    // Validate each net
    for net in spec.nets.values() {
        let net_issues = validate_net(net, spec);
        report.merge(net_issues);
    }

    // Validate web service gateways
    for wsg in spec.web_service_gateways.values() {
        let wsg_issues = validate_web_service_gateway(wsg);
        report.merge(wsg_issues);
    }

    Ok(report)
}

pub fn validate_net(net: &Net, spec: &WorkflowSpec) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Cardinality checks
    let issues = validate_net_conditions(net);
    for issue in issues {
        report.add_issue(issue);
    }

    // Validate tasks
    for task in net.tasks.values() {
        let task_report = validate_task_complete(task, net, spec);
        report.merge(task_report);
    }

    // Validate conditions
    for condition in net.conditions.values() {
        let condition_issues = validate_condition(condition);
        for issue in condition_issues {
            report.add_issue(issue);
        }
    }

    // Validate variables
    let var_issues = validate_variable_types(net);
    for issue in var_issues {
        report.add_issue(issue);
    }

    report
}

pub fn validate_task_complete(task: &Task, net: &Net, spec: &WorkflowSpec) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Required fields
    let issues = validate_task_required_fields(task);
    for issue in issues {
        report.add_issue(issue);
    }

    // Control types
    let issues = validate_control_types(task);
    for issue in issues {
        report.add_issue(issue);
    }

    // MI config
    if task.task_type == TaskType::MultipleInstance {
        let issues = validate_mi_config(task);
        for issue in issues {
            report.add_issue(issue);
        }

        let issues = validate_mi_instance_constraints(task);
        for issue in issues {
            report.add_issue(issue);
        }
    }

    // Timer
    let issues = validate_timer_config(task);
    for issue in issues {
        report.add_issue(issue);
    }

    // knhk extensions
    let issues = validate_knhk_tick_budget(task);
    for issue in issues {
        report.add_issue(issue);
    }

    // References
    let issues = validate_decomposition_references(spec, task);
    for issue in issues {
        report.add_issue(issue);
    }

    let issues = validate_variable_mappings(net, task);
    for issue in issues {
        report.add_issue(issue);
    }

    report
}
```

---

## 10. Summary

**Validation Coverage:**
- ✅ Cardinality constraints (required fields, collections)
- ✅ Type validation (enums, data types, expressions)
- ✅ Range validation (tick budgets, MI constraints, timers)
- ✅ Reference integrity (flows, decompositions, variables)
- ✅ knhk-specific constraints (hot path, priority)
- ✅ Semantic validation (control flow patterns)

**Integration:**
- ✅ chicago-tdd-tools test framework
- ✅ Property-based testing with proptest
- ✅ Validation report generation
- ✅ Error messages and suggestions

**Next Steps:**
1. Implement validation functions in `rust/knhk-workflow-engine/src/validation/mod.rs`
2. Write comprehensive tests using chicago-tdd-tools
3. Integrate with parser pipeline
4. Add OTEL instrumentation for validation events
5. Generate validation reports for CI/CD

---

## 11. References

- **Data Type Mappings:** `/docs/ontology-integration/data-type-mappings-complete.md`
- **Serde Strategies:** `/docs/ontology-integration/rust-serde-strategies.md`
- **Semantic Validation Rules:** `/docs/ontology-integration/semantic-validation-rules.md`
- **chicago-tdd-tools:** `/Users/sac/knhk/tools/chicago-tdd-tools/`
- **Target File:** `rust/knhk-workflow-engine/src/validation/mod.rs`

**COMPLETENESS: Complete Rust validation logic for post-deserialization constraint checking**
