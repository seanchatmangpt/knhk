//! Phase 5 Platform Integration
//!
//! Integrates quantum-safe cryptography with KNHK Phase 5 platform telemetry,
//! descriptor signing, and migration pathways from classical to quantum systems.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::sig::{QuantumSig, DilithiumSig};
use crate::kem::{QuantumKEM, KyberKEM};
use crate::nist::NISTLevel;
use tracing::{info, debug, warn};

/// Integration errors
#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Descriptor signing failed: {0}")]
    DescriptorSigningFailed(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Telemetry error: {0}")]
    TelemetryError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Phase 5 Descriptor (workflow definition) with quantum-safe signing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumSignedDescriptor {
    /// Descriptor ID
    pub descriptor_id: String,
    /// Descriptor version
    pub version: String,
    /// Descriptor content (JSON serialized)
    pub content: String,
    /// Classical signature (backward compatibility)
    pub classical_sig: Option<String>,
    /// Quantum-safe signature (Dilithium)
    pub quantum_sig: String,
    /// Signature timestamp
    pub signed_at: String,
    /// Signer public key (Dilithium)
    pub signer_pk: String,
}

impl QuantumSignedDescriptor {
    /// Create a new quantum-signed descriptor
    pub fn new(
        descriptor_id: String,
        version: String,
        content: String,
    ) -> Self {
        Self {
            descriptor_id,
            version,
            content,
            classical_sig: None,
            quantum_sig: String::new(),
            signed_at: chrono::Utc::now().to_rfc3339(),
            signer_pk: String::new(),
        }
    }

    /// Sign descriptor with quantum-safe signature
    pub fn sign(&mut self, sk: &[u8], pk: &[u8]) -> Result<()> {
        let sig = DilithiumSig::new();

        // Prepare message to sign (descriptor content + metadata)
        let msg = format!(
            "{}_{}_{}_{}",
            self.descriptor_id, self.version, self.content, self.signed_at
        );

        let signature = sig
            .sign(sk, msg.as_bytes())
            .map_err(|e| IntegrationError::DescriptorSigningFailed(e.to_string()))?;

        self.quantum_sig = hex::encode(&signature);
        self.signer_pk = hex::encode(pk);

        info!(
            descriptor_id = %self.descriptor_id,
            "Descriptor signed with quantum-safe signature"
        );

        Ok(())
    }

    /// Verify descriptor signature
    pub fn verify(&self) -> Result<bool> {
        let sig = DilithiumSig::new();

        let msg = format!(
            "{}_{}_{}_{}",
            self.descriptor_id, self.version, self.content, self.signed_at
        );

        let pk_bytes = hex::decode(&self.signer_pk)
            .map_err(|e| IntegrationError::DescriptorSigningFailed(e.to_string()))?;

        let sig_bytes = hex::decode(&self.quantum_sig)
            .map_err(|e| IntegrationError::DescriptorSigningFailed(e.to_string()))?;

        let valid = sig
            .verify(&pk_bytes, msg.as_bytes(), &sig_bytes)
            .map_err(|e| IntegrationError::DescriptorSigningFailed(e.to_string()))?;

        debug!(
            descriptor_id = %self.descriptor_id,
            valid = valid,
            "Descriptor signature verification completed"
        );

        Ok(valid)
    }
}

/// Migration tracker for classical → quantum cryptography transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationPhase {
    /// Phase name
    pub phase_name: String,
    /// Current NIST security level
    pub current_level: String,
    /// Target NIST security level
    pub target_level: String,
    /// Percentage of components migrated (0-100)
    pub migration_percentage: u8,
    /// Is this phase complete?
    pub complete: bool,
    /// Timestamp of this phase
    pub timestamp: String,
}

impl MigrationPhase {
    pub fn new(
        phase_name: String,
        current_level: String,
        target_level: String,
    ) -> Self {
        Self {
            phase_name,
            current_level,
            target_level,
            migration_percentage: 0,
            complete: false,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn update_progress(&mut self, percentage: u8) -> Result<()> {
        if percentage > 100 {
            return Err(IntegrationError::MigrationFailed(
                "Migration percentage cannot exceed 100".to_string(),
            ));
        }
        self.migration_percentage = percentage;
        if percentage == 100 {
            self.complete = true;
        }
        Ok(())
    }
}

/// Quantum integration telemetry for Phase 5 platform
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumTelemetry {
    /// KEM operations count
    pub kem_operations: u64,
    /// Signature operations count
    pub sig_operations: u64,
    /// Average KEM latency (ms)
    pub avg_kem_latency_ms: f64,
    /// Average signature latency (ms)
    pub avg_sig_latency_ms: f64,
    /// Hybrid mode enabled?
    pub hybrid_enabled: bool,
    /// NIST compliance level
    pub nist_level: String,
    /// Last telemetry update
    pub last_update: String,
}

impl Default for QuantumTelemetry {
    fn default() -> Self {
        Self {
            kem_operations: 0,
            sig_operations: 0,
            avg_kem_latency_ms: 0.0,
            avg_sig_latency_ms: 0.0,
            hybrid_enabled: false,
            nist_level: "Level 1".to_string(),
            last_update: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl QuantumTelemetry {
    pub fn record_kem_operation(&mut self, latency_ms: f64) {
        self.kem_operations += 1;
        // Update moving average
        let prev_avg = self.avg_kem_latency_ms;
        self.avg_kem_latency_ms =
            (prev_avg * (self.kem_operations - 1) as f64 + latency_ms) / self.kem_operations as f64;
    }

    pub fn record_sig_operation(&mut self, latency_ms: f64) {
        self.sig_operations += 1;
        // Update moving average
        let prev_avg = self.avg_sig_latency_ms;
        self.avg_sig_latency_ms =
            (prev_avg * (self.sig_operations - 1) as f64 + latency_ms) / self.sig_operations as f64;
    }

    pub fn update_timestamp(&mut self) {
        self.last_update = chrono::Utc::now().to_rfc3339();
    }

    pub fn as_otel_attributes(&self) -> Vec<(&'static str, String)> {
        vec![
            ("quantum.kem.operations", self.kem_operations.to_string()),
            ("quantum.sig.operations", self.sig_operations.to_string()),
            ("quantum.kem.latency_ms", format!("{:.2}", self.avg_kem_latency_ms)),
            ("quantum.sig.latency_ms", format!("{:.2}", self.avg_sig_latency_ms)),
            ("quantum.hybrid.enabled", self.hybrid_enabled.to_string()),
            ("quantum.nist.level", self.nist_level.clone()),
        ]
    }
}

/// Quantum integration service for Phase 5 platform
pub struct QuantumIntegration {
    telemetry: QuantumTelemetry,
    migration_phases: Vec<MigrationPhase>,
}

impl QuantumIntegration {
    pub fn new() -> Self {
        Self {
            telemetry: QuantumTelemetry::default(),
            migration_phases: Vec::new(),
        }
    }

    /// Add a migration phase
    pub fn add_migration_phase(&mut self, phase: MigrationPhase) {
        self.migration_phases.push(phase);
    }

    /// Get current telemetry
    pub fn get_telemetry(&self) -> QuantumTelemetry {
        self.telemetry.clone()
    }

    /// Record operation metrics
    pub fn record_operation(&mut self, operation_type: &str, latency_ms: f64) {
        match operation_type {
            "kem" => self.telemetry.record_kem_operation(latency_ms),
            "sig" => self.telemetry.record_sig_operation(latency_ms),
            _ => warn!("Unknown operation type: {}", operation_type),
        }
        self.telemetry.update_timestamp();
    }

    /// Initialize quantum-safe migration from classical
    pub fn initialize_migration(&mut self) -> Result<()> {
        info!("Initializing quantum-safe migration from classical cryptography");

        let phase = MigrationPhase::new(
            "Phase 1: Hybrid Deployment".to_string(),
            "Classical Only".to_string(),
            "Hybrid (Classical + Quantum)".to_string(),
        );

        self.add_migration_phase(phase);

        Ok(())
    }

    /// Advance migration phase
    pub fn advance_migration(&mut self, phase_index: usize, percentage: u8) -> Result<()> {
        if phase_index >= self.migration_phases.len() {
            return Err(IntegrationError::MigrationFailed(
                "Invalid phase index".to_string(),
            ));
        }

        self.migration_phases[phase_index].update_progress(percentage)?;
        info!(
            phase = %self.migration_phases[phase_index].phase_name,
            percentage = percentage,
            "Migration phase advanced"
        );

        Ok(())
    }

    /// Generate migration report
    pub fn migration_report(&self) -> String {
        let mut report = String::from("=== Quantum Migration Report ===\n");
        for (idx, phase) in self.migration_phases.iter().enumerate() {
            report.push_str(&format!(
                "Phase {}: {} ({}% complete, {})\n",
                idx + 1,
                phase.phase_name,
                phase.migration_percentage,
                if phase.complete { "✓" } else { "in progress" }
            ));
        }
        report.push_str("\n=== Telemetry Summary ===\n");
        report.push_str(&format!(
            "KEM Operations: {} (avg latency: {:.2}ms)\n",
            self.telemetry.kem_operations, self.telemetry.avg_kem_latency_ms
        ));
        report.push_str(&format!(
            "Signature Operations: {} (avg latency: {:.2}ms)\n",
            self.telemetry.sig_operations, self.telemetry.avg_sig_latency_ms
        ));
        report
    }
}

impl Default for QuantumIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_signed_descriptor() {
        let mut descriptor = QuantumSignedDescriptor::new(
            "test-descriptor".to_string(),
            "1.0.0".to_string(),
            "{}".to_string(),
        );

        let sig = DilithiumSig::new();
        let (pk, sk) = sig.keygen().expect("keygen failed");

        descriptor.sign(&sk, &pk).expect("signing failed");
        assert!(!descriptor.quantum_sig.is_empty());
        assert!(!descriptor.signer_pk.is_empty());
    }

    #[test]
    fn test_migration_phase() {
        let mut phase = MigrationPhase::new(
            "Test Phase".to_string(),
            "Level 1".to_string(),
            "Level 3".to_string(),
        );

        phase.update_progress(50).expect("update failed");
        assert_eq!(phase.migration_percentage, 50);
        assert!(!phase.complete);

        phase.update_progress(100).expect("update failed");
        assert!(phase.complete);
    }

    #[test]
    fn test_quantum_telemetry() {
        let mut telemetry = QuantumTelemetry::default();
        telemetry.record_kem_operation(10.5);
        telemetry.record_kem_operation(12.3);
        assert_eq!(telemetry.kem_operations, 2);

        let attrs = telemetry.as_otel_attributes();
        assert!(!attrs.is_empty());
    }

    #[test]
    fn test_quantum_integration() {
        let mut integration = QuantumIntegration::new();
        integration.initialize_migration().expect("initialization failed");
        assert_eq!(integration.migration_phases.len(), 1);

        integration.record_operation("kem", 11.5);
        assert_eq!(integration.telemetry.kem_operations, 1);

        let report = integration.migration_report();
        assert!(report.contains("Quantum Migration Report"));
    }
}
