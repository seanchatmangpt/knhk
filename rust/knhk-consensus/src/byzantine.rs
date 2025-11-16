// Byzantine Fault Detection and Analysis
// Detects and reports Byzantine replica behavior

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByzantineError {
    #[error("Detection failed: {0}")]
    DetectionFailed(String),

    #[error("Invalid replica behavior")]
    InvalidBehavior,
}

/// Types of Byzantine behavior
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FaultType {
    /// Replica sends conflicting messages
    EquivocationFault,
    /// Replica fails to respond
    SilentFault,
    /// Replica sends messages out of order
    OrderingFault,
    /// Replica forges signatures
    AuthenticationFault,
    /// Replica sends invalid state transitions
    LogicalFault,
}

/// Byzantine fault report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FaultReport {
    pub faulty_replica: usize,
    pub fault_type: FaultType,
    pub evidence: Vec<u8>,
    pub timestamp: u64,
    pub severity: u8, // 1-10
}

/// Byzantine fault detector
#[derive(Debug)]
pub struct ByzantineFaultDetector {
    pub total_replicas: usize,
    pub known_faults: Vec<FaultReport>,
    pub suspected_replicas: Vec<usize>,
}

impl ByzantineFaultDetector {
    /// Create a new Byzantine fault detector
    pub fn new(total_replicas: usize) -> Self {
        Self {
            total_replicas,
            known_faults: Vec::new(),
            suspected_replicas: Vec::new(),
        }
    }

    /// Detect equivocation (conflicting messages)
    pub fn detect_equivocation(&mut self, replica_id: usize, msg1: &[u8], msg2: &[u8]) -> Result<(), ByzantineError> {
        // Phase 8 implementation stub
        // TODO: Implement equivocation detection
        // Step 1: Verify messages are from same view/sequence
        // Step 2: Verify messages have same hash (should be identical)
        // Step 3: If different, replica is equivocating
        // Step 4: Create fault report

        if msg1 != msg2 {
            let mut evidence = Vec::new();
            evidence.extend_from_slice(msg1);
            evidence.extend_from_slice(msg2);

            let report = FaultReport {
                faulty_replica: replica_id,
                fault_type: FaultType::EquivocationFault,
                evidence,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                severity: 9,
            };

            self.known_faults.push(report);
            self.suspected_replicas.push(replica_id);

            tracing::warn!(
                "Byzantine detector: equivocation detected from replica {}",
                replica_id
            );
        }

        Ok(())
    }

    /// Detect silent faults (missing messages)
    pub fn detect_silent_fault(&mut self, replica_id: usize, timeout_ms: u64) -> Result<(), ByzantineError> {
        // Phase 8 implementation stub
        // TODO: Implement silent fault detection
        // Step 1: Check if replica missed expected message
        // Step 2: Check if timeout exceeded
        // Step 3: Create fault report

        let report = FaultReport {
            faulty_replica: replica_id,
            fault_type: FaultType::SilentFault,
            evidence: format!("Missing messages for {}ms", timeout_ms).into_bytes(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            severity: 5,
        };

        self.known_faults.push(report);

        tracing::warn!(
            "Byzantine detector: silent fault detected from replica {} (timeout: {}ms)",
            replica_id,
            timeout_ms
        );

        Ok(())
    }

    /// Detect ordering faults (out-of-order messages)
    pub fn detect_ordering_fault(&mut self, replica_id: usize, expected: u64, actual: u64) -> Result<(), ByzantineError> {
        // Phase 8 implementation stub
        // TODO: Implement ordering fault detection

        if actual != expected {
            let report = FaultReport {
                faulty_replica: replica_id,
                fault_type: FaultType::OrderingFault,
                evidence: format!("Expected sequence {}, got {}", expected, actual).into_bytes(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                severity: 6,
            };

            self.known_faults.push(report);

            tracing::warn!(
                "Byzantine detector: ordering fault detected from replica {}",
                replica_id
            );
        }

        Ok(())
    }

    /// Get list of detected faulty replicas
    pub fn get_faulty_replicas(&self) -> Vec<usize> {
        let mut faulty = Vec::new();
        for report in &self.known_faults {
            if !faulty.contains(&report.faulty_replica) {
                faulty.push(report.faulty_replica);
            }
        }
        faulty
    }

    /// Check if system is still safe (f < n/3)
    pub fn is_system_safe(&self) -> bool {
        let faulty_count = self.get_faulty_replicas().len();
        let max_tolerable = (self.total_replicas - 1) / 3;
        faulty_count <= max_tolerable
    }

    /// Get fault summary
    pub fn get_summary(&self) -> FaultSummary {
        let faulty_replicas = self.get_faulty_replicas();
        let total_faults = self.known_faults.len();
        let system_safe = self.is_system_safe();
        let max_tolerable = (self.total_replicas - 1) / 3;

        FaultSummary {
            total_replicas: self.total_replicas,
            faulty_replicas,
            total_faults,
            system_safe,
            max_tolerable_faults: max_tolerable,
        }
    }
}

/// Byzantine fault summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FaultSummary {
    pub total_replicas: usize,
    pub faulty_replicas: Vec<usize>,
    pub total_faults: usize,
    pub system_safe: bool,
    pub max_tolerable_faults: usize,
}

impl FaultSummary {
    /// Get human-readable summary
    pub fn to_string_detailed(&self) -> String {
        format!(
            "Byzantine Fault Summary:\n  Total Replicas: {}\n  Faulty Replicas: {:?}\n  Total Faults: {}\n  System Safe: {}\n  Max Tolerable: {}",
            self.total_replicas,
            self.faulty_replicas,
            self.total_faults,
            self.system_safe,
            self.max_tolerable_faults
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byzantine_detector_creation() {
        let detector = ByzantineFaultDetector::new(4);
        assert_eq!(detector.total_replicas, 4);
        assert!(detector.known_faults.is_empty());
    }

    #[test]
    fn test_detect_equivocation() {
        let mut detector = ByzantineFaultDetector::new(4);
        let msg1 = b"message1";
        let msg2 = b"message2";

        let result = detector.detect_equivocation(0, msg1, msg2);
        assert!(result.is_ok());
        assert_eq!(detector.known_faults.len(), 1);
        assert_eq!(detector.known_faults[0].fault_type, FaultType::EquivocationFault);
    }

    #[test]
    fn test_detect_silent_fault() {
        let mut detector = ByzantineFaultDetector::new(4);
        let result = detector.detect_silent_fault(1, 5000);
        assert!(result.is_ok());
        assert_eq!(detector.known_faults.len(), 1);
    }

    #[test]
    fn test_detect_ordering_fault() {
        let mut detector = ByzantineFaultDetector::new(4);
        let result = detector.detect_ordering_fault(2, 5, 3);
        assert!(result.is_ok());
        assert_eq!(detector.known_faults.len(), 1);
        assert_eq!(detector.known_faults[0].fault_type, FaultType::OrderingFault);
    }

    #[test]
    fn test_get_faulty_replicas() {
        let mut detector = ByzantineFaultDetector::new(4);
        detector.detect_equivocation(0, b"msg1", b"msg2").unwrap();
        detector.detect_equivocation(1, b"msg1", b"msg2").unwrap();
        detector.detect_silent_fault(0, 5000).unwrap();

        let faulty = detector.get_faulty_replicas();
        assert_eq!(faulty.len(), 2);
    }

    #[test]
    fn test_system_safety_check() {
        let mut detector = ByzantineFaultDetector::new(4);
        assert!(detector.is_system_safe()); // 0 faults

        detector.detect_equivocation(0, b"msg1", b"msg2").unwrap();
        assert!(detector.is_system_safe()); // 1 fault, max 1 tolerable

        detector.detect_equivocation(1, b"msg1", b"msg2").unwrap();
        assert!(!detector.is_system_safe()); // 2 faults, max 1 tolerable
    }

    #[test]
    fn test_fault_summary() {
        let mut detector = ByzantineFaultDetector::new(4);
        detector.detect_equivocation(0, b"msg1", b"msg2").unwrap();

        let summary = detector.get_summary();
        assert_eq!(summary.total_replicas, 4);
        assert_eq!(summary.faulty_replicas.len(), 1);
        assert!(summary.system_safe);
        assert_eq!(summary.max_tolerable_faults, 1);
    }
}
