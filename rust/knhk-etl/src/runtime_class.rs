// rust/knhk-etl/src/runtime_class.rs
// Runtime class tracking and classification
// Classifies operations into R1 (Hot), W1 (Warm), C1 (Cold) per Reflex Enterprise Blueprint

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::format;

/// Runtime class for operation classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeClass {
    /// R1 Hot: ASK/COUNT/COMPARE/VALIDATE, ≤8 items, 8 ticks budget, ≤2 ns/op SLO
    R1,
    /// W1 Warm: CONSTRUCT8, prebind, AOT transforms, ≤500 µs budget, ≤1 ms SLO
    W1,
    /// C1 Cold: Full SPARQL/SHACL, joins, analytics, ≤200 ms budget, ≤500 ms SLO
    C1,
}

/// Runtime class metadata
#[derive(Debug, Clone)]
pub struct RuntimeClassMetadata {
    /// Runtime class (R1/W1/C1)
    pub class: RuntimeClass,
    /// Budget in nanoseconds
    pub budget_ns: u64,
    /// SLO p99 in nanoseconds
    pub slo_p99_ns: u64,
    /// Operation type description
    pub operation_type: String,
}

impl RuntimeClass {
    /// Get metadata for a runtime class
    pub fn metadata(&self) -> RuntimeClassMetadata {
        match self {
            RuntimeClass::R1 => RuntimeClassMetadata {
                class: RuntimeClass::R1,
                budget_ns: 8, // 8 ticks ≈ 2ns at 4GHz
                slo_p99_ns: 2, // ≤2 ns/op on-core
                operation_type: "R1 Hot".to_string(),
            },
            RuntimeClass::W1 => RuntimeClassMetadata {
                class: RuntimeClass::W1,
                budget_ns: 500_000, // ≤500 µs
                slo_p99_ns: 1_000_000, // ≤1 ms
                operation_type: "W1 Warm".to_string(),
            },
            RuntimeClass::C1 => RuntimeClassMetadata {
                class: RuntimeClass::C1,
                budget_ns: 200_000_000, // ≤200 ms
                slo_p99_ns: 500_000_000, // ≤500 ms
                operation_type: "C1 Cold".to_string(),
            },
        }
    }

    /// Classify operation based on operation type and data size
    /// 
    /// # Arguments
    /// * `operation_type` - Operation type (e.g., "ASK_SP", "CONSTRUCT8", "SPARQL_SELECT")
    /// * `data_size` - Number of items/triples (≤8 for R1)
    /// 
    /// # Returns
    /// `RuntimeClass` classification
    pub fn classify_operation(operation_type: &str, data_size: usize) -> Result<RuntimeClass, String> {
        // R1 Hot: ASK/COUNT/COMPARE/VALIDATE operations with ≤8 items
        if Self::is_r1_operation(operation_type) && data_size <= 8 {
            return Ok(RuntimeClass::R1);
        }

        // W1 Warm: CONSTRUCT8, prebind, AOT transforms
        if Self::is_w1_operation(operation_type) {
            return Ok(RuntimeClass::W1);
        }

        // C1 Cold: Complex operations (SPARQL/SHACL, joins, analytics)
        if Self::is_c1_operation(operation_type) || data_size > 10000 {
            return Ok(RuntimeClass::C1);
        }

        // Default: route to W1 for SPARQL queries that don't fit R1
        if Self::is_sparql_query(operation_type) {
            return Ok(RuntimeClass::W1);
        }

        Err(format!("Unable to classify operation: {} with data_size: {}", operation_type, data_size))
    }

    /// Check if operation is R1 (Hot path)
    fn is_r1_operation(operation_type: &str) -> bool {
        let op_upper = operation_type.to_uppercase();
        op_upper.starts_with("ASK_") ||
        op_upper.starts_with("COUNT_") ||
        op_upper.starts_with("COMPARE_") ||
        op_upper.starts_with("VALIDATE_") ||
        op_upper == "UNIQUE_SP"
    }

    /// Check if operation is W1 (Warm path)
    fn is_w1_operation(operation_type: &str) -> bool {
        let op_upper = operation_type.to_uppercase();
        op_upper == "CONSTRUCT8" ||
        op_upper.starts_with("PREBIND_") ||
        op_upper.starts_with("AOT_") ||
        op_upper == "SELECT_SP" // Simple SELECT operations
    }

    /// Check if operation is C1 (Cold path)
    fn is_c1_operation(operation_type: &str) -> bool {
        let op_upper = operation_type.to_uppercase();
        op_upper.contains("SPARQL") ||
        op_upper.contains("SHACL") ||
        op_upper.contains("JOIN") ||
        op_upper.contains("ANALYTICS") ||
        op_upper.starts_with("UPDATE_") ||
        op_upper.starts_with("REASONING_")
    }

    /// Check if operation is a SPARQL query
    fn is_sparql_query(operation_type: &str) -> bool {
        let op_upper = operation_type.to_uppercase();
        op_upper.starts_with("SPARQL_") ||
        op_upper == "SELECT" ||
        op_upper == "CONSTRUCT" ||
        op_upper == "DESCRIBE" ||
        op_upper == "ASK"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r1_classification() {
        assert_eq!(RuntimeClass::classify_operation("ASK_SP", 5).expect("Classification should succeed"), RuntimeClass::R1);
        assert_eq!(RuntimeClass::classify_operation("COUNT_SP_GE", 8).expect("R1 classification should succeed"), RuntimeClass::R1);
        assert_eq!(RuntimeClass::classify_operation("COMPARE_O_EQ", 3).expect("R1 classification should succeed"), RuntimeClass::R1);
        assert_eq!(RuntimeClass::classify_operation("VALIDATE_DATATYPE_SP", 1).expect("R1 classification should succeed"), RuntimeClass::R1);
    }

    #[test]
    fn test_r1_data_size_limit() {
        assert_eq!(RuntimeClass::classify_operation("ASK_SP", 8).expect("R1 classification should succeed"), RuntimeClass::R1);
        assert_ne!(RuntimeClass::classify_operation("ASK_SP", 9).expect("R1 classification should succeed"), RuntimeClass::R1);
    }

    #[test]
    fn test_w1_classification() {
        assert_eq!(RuntimeClass::classify_operation("CONSTRUCT8", 5).expect("W1 classification should succeed"), RuntimeClass::W1);
        assert_eq!(RuntimeClass::classify_operation("PREBIND_TEMPLATE", 10).expect("W1 classification should succeed"), RuntimeClass::W1);
        assert_eq!(RuntimeClass::classify_operation("AOT_TRANSFORM", 100).expect("W1 classification should succeed"), RuntimeClass::W1);
    }

    #[test]
    fn test_c1_classification() {
        assert_eq!(RuntimeClass::classify_operation("SPARQL_SELECT", 100).expect("C1 classification should succeed"), RuntimeClass::C1);
        assert_eq!(RuntimeClass::classify_operation("SHACL_VALIDATE", 50).expect("C1 classification should succeed"), RuntimeClass::C1);
        assert_eq!(RuntimeClass::classify_operation("JOIN_QUERY", 200).expect("C1 classification should succeed"), RuntimeClass::C1);
    }

    #[test]
    fn test_metadata() {
        let r1_meta = RuntimeClass::R1.metadata();
        assert_eq!(r1_meta.budget_ns, 8);
        assert_eq!(r1_meta.slo_p99_ns, 2);

        let w1_meta = RuntimeClass::W1.metadata();
        assert_eq!(w1_meta.budget_ns, 500_000);
        assert_eq!(w1_meta.slo_p99_ns, 1_000_000);

        let c1_meta = RuntimeClass::C1.metadata();
        assert_eq!(c1_meta.budget_ns, 200_000_000);
        assert_eq!(c1_meta.slo_p99_ns, 500_000_000);
    }
}

