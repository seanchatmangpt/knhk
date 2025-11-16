//! Security Validation Testing
//!
//! Verifies signature verification, tamper detection, forgery prevention,
//! and authorization enforcement.

use std::sync::Arc;
use parking_lot::RwLock;
use ring::{digest, hmac, rand, signature};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use serde::{Serialize, Deserialize};
use base64;

/// Security test harness
pub struct SecurityTestHarness {
    key_manager: Arc<KeyManager>,
    signature_verifier: Arc<SignatureVerifier>,
    tamper_detector: Arc<TamperDetector>,
    authorization_enforcer: Arc<AuthorizationEnforcer>,
}

/// Key management system
pub struct KeyManager {
    signing_keys: Arc<RwLock<Vec<Ed25519KeyPair>>>,
    verification_keys: Arc<RwLock<Vec<UnparsedPublicKey<Vec<u8>>>>>,
    key_rotation_history: Arc<RwLock<Vec<KeyRotationEvent>>>,
}

#[derive(Debug, Clone)]
pub struct KeyRotationEvent {
    pub timestamp: u64,
    pub old_key_id: String,
    pub new_key_id: String,
    pub reason: String,
}

impl KeyManager {
    pub fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let mut signing_keys = Vec::new();
        let mut verification_keys = Vec::new();

        // Generate initial key pairs
        for _ in 0..3 {
            let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
            let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
            let public_key = key_pair.public_key();

            verification_keys.push(UnparsedPublicKey::new(
                &signature::ED25519,
                public_key.as_ref().to_vec()
            ));
            signing_keys.push(key_pair);
        }

        Self {
            signing_keys: Arc::new(RwLock::new(signing_keys)),
            verification_keys: Arc::new(RwLock::new(verification_keys)),
            key_rotation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn rotate_keys(&self) -> Result<(), String> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| format!("Key generation failed: {:?}", e))?;

        let new_key = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|e| format!("Key parsing failed: {:?}", e))?;

        let public_key = new_key.public_key();

        // Add new key
        self.signing_keys.write().push(new_key);
        self.verification_keys.write().push(UnparsedPublicKey::new(
            &signature::ED25519,
            public_key.as_ref().to_vec()
        ));

        // Record rotation event
        self.key_rotation_history.write().push(KeyRotationEvent {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            old_key_id: format!("key_{}", self.signing_keys.read().len() - 1),
            new_key_id: format!("key_{}", self.signing_keys.read().len()),
            reason: "Scheduled rotation".to_string(),
        });

        Ok(())
    }

    pub fn get_current_signing_key(&self) -> Option<Vec<u8>> {
        self.signing_keys.read()
            .last()
            .map(|k| k.public_key().as_ref().to_vec())
    }
}

/// Signature verification system
pub struct SignatureVerifier {
    key_manager: Arc<KeyManager>,
    verification_cache: Arc<RwLock<Vec<VerificationResult>>>,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub descriptor_id: u64,
    pub signature: Vec<u8>,
    pub valid: bool,
    pub timestamp: u64,
    pub key_id: String,
}

impl SignatureVerifier {
    pub fn new(key_manager: Arc<KeyManager>) -> Self {
        Self {
            key_manager,
            verification_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn sign_descriptor(&self, descriptor: &[u8]) -> Result<Vec<u8>, String> {
        let keys = self.key_manager.signing_keys.read();
        let key = keys.last().ok_or("No signing key available")?;

        let signature = key.sign(descriptor);
        Ok(signature.as_ref().to_vec())
    }

    pub fn verify_signature(&self, descriptor: &[u8], signature: &[u8]) -> bool {
        let keys = self.key_manager.verification_keys.read();

        // Try all keys (for key rotation support)
        for key in keys.iter() {
            if key.verify(descriptor, signature).is_ok() {
                self.verification_cache.write().push(VerificationResult {
                    descriptor_id: Self::hash_descriptor(descriptor),
                    signature: signature.to_vec(),
                    valid: true,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    key_id: format!("key_{}", 0), // Simplified
                });
                return true;
            }
        }

        false
    }

    fn hash_descriptor(descriptor: &[u8]) -> u64 {
        let hash = digest::digest(&digest::SHA256, descriptor);
        let bytes = hash.as_ref();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    pub fn test_forgery_prevention(&self) -> bool {
        let descriptor = b"test_descriptor";
        let signature = self.sign_descriptor(descriptor).unwrap();

        // Attempt to forge signature
        let mut forged_signature = signature.clone();
        forged_signature[0] ^= 0xFF; // Flip bits

        // Verify original works
        if !self.verify_signature(descriptor, &signature) {
            return false;
        }

        // Verify forgery fails
        !self.verify_signature(descriptor, &forged_signature)
    }
}

/// Tamper detection system
pub struct TamperDetector {
    checksums: Arc<RwLock<HashMap<u64, Checksum>>>,
    tamper_events: Arc<RwLock<Vec<TamperEvent>>>,
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub descriptor_id: u64,
    pub hash: Vec<u8>,
    pub hmac: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct TamperEvent {
    pub descriptor_id: u64,
    pub detection_time: u64,
    pub tamper_type: TamperType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum TamperType {
    ChecksumMismatch,
    HmacFailure,
    UnauthorizedModification,
    ReplayAttack,
    InjectionAttempt,
}

impl TamperDetector {
    pub fn new() -> Self {
        Self {
            checksums: Arc::new(RwLock::new(HashMap::new())),
            tamper_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn protect_descriptor(&self, descriptor_id: u64, data: &[u8]) -> Checksum {
        let hash = digest::digest(&digest::SHA256, data);

        let key = hmac::Key::new(hmac::HMAC_SHA256, b"test_hmac_key");
        let tag = hmac::sign(&key, data);

        let checksum = Checksum {
            descriptor_id,
            hash: hash.as_ref().to_vec(),
            hmac: tag.as_ref().to_vec(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        self.checksums.write().insert(descriptor_id, checksum.clone());
        checksum
    }

    pub fn verify_integrity(&self, descriptor_id: u64, data: &[u8]) -> Result<(), TamperType> {
        let checksums = self.checksums.read();
        let stored = checksums.get(&descriptor_id)
            .ok_or(TamperType::UnauthorizedModification)?;

        // Verify hash
        let current_hash = digest::digest(&digest::SHA256, data);
        if current_hash.as_ref() != stored.hash.as_slice() {
            self.record_tamper_event(descriptor_id, TamperType::ChecksumMismatch);
            return Err(TamperType::ChecksumMismatch);
        }

        // Verify HMAC
        let key = hmac::Key::new(hmac::HMAC_SHA256, b"test_hmac_key");
        match hmac::verify(&key, data, stored.hmac.as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.record_tamper_event(descriptor_id, TamperType::HmacFailure);
                Err(TamperType::HmacFailure)
            }
        }
    }

    fn record_tamper_event(&self, descriptor_id: u64, tamper_type: TamperType) {
        self.tamper_events.write().push(TamperEvent {
            descriptor_id,
            detection_time: chrono::Utc::now().timestamp_millis() as u64,
            tamper_type: tamper_type.clone(),
            details: format!("Tamper detected: {:?}", tamper_type),
        });
    }

    pub fn test_tamper_detection(&self) -> bool {
        let descriptor_id = 12345;
        let original_data = b"original_descriptor_data";

        // Protect original
        self.protect_descriptor(descriptor_id, original_data);

        // Verify original passes
        if self.verify_integrity(descriptor_id, original_data).is_err() {
            return false;
        }

        // Tamper with data
        let mut tampered_data = original_data.to_vec();
        tampered_data[0] ^= 0xFF;

        // Verify tampered data fails
        self.verify_integrity(descriptor_id, &tampered_data).is_err()
    }

    pub fn get_tamper_events(&self) -> Vec<TamperEvent> {
        self.tamper_events.read().clone()
    }
}

/// Authorization enforcement system
pub struct AuthorizationEnforcer {
    policies: Arc<RwLock<Vec<AuthorizationPolicy>>>,
    access_log: Arc<RwLock<Vec<AccessAttempt>>>,
}

#[derive(Debug, Clone)]
pub struct AuthorizationPolicy {
    pub id: String,
    pub resource: String,
    pub required_permissions: Vec<Permission>,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
}

#[derive(Debug, Clone)]
pub enum Condition {
    TimeWindow { start: u64, end: u64 },
    RateLimit { max_requests: u64, window_seconds: u64 },
    SourceRestriction { allowed_sources: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct AccessAttempt {
    pub timestamp: u64,
    pub principal: String,
    pub resource: String,
    pub action: Permission,
    pub granted: bool,
    pub reason: String,
}

impl AuthorizationEnforcer {
    pub fn new() -> Self {
        let policies = vec![
            AuthorizationPolicy {
                id: "default_read".to_string(),
                resource: "descriptors/*".to_string(),
                required_permissions: vec![Permission::Read],
                conditions: vec![],
            },
            AuthorizationPolicy {
                id: "admin_write".to_string(),
                resource: "descriptors/admin/*".to_string(),
                required_permissions: vec![Permission::Admin, Permission::Write],
                conditions: vec![
                    Condition::TimeWindow {
                        start: 0,
                        end: u64::MAX,
                    },
                ],
            },
        ];

        Self {
            policies: Arc::new(RwLock::new(policies)),
            access_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn check_authorization(
        &self,
        principal: &str,
        resource: &str,
        action: Permission,
        context: &AuthorizationContext,
    ) -> bool {
        let policies = self.policies.read();

        for policy in policies.iter() {
            if self.resource_matches(&policy.resource, resource) {
                let has_permission = policy.required_permissions.contains(&action);
                let conditions_met = self.evaluate_conditions(&policy.conditions, context);

                let granted = has_permission && conditions_met;

                self.access_log.write().push(AccessAttempt {
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    principal: principal.to_string(),
                    resource: resource.to_string(),
                    action,
                    granted,
                    reason: if granted {
                        "Policy matched".to_string()
                    } else {
                        "Insufficient permissions or conditions not met".to_string()
                    },
                });

                if granted {
                    return true;
                }
            }
        }

        false
    }

    fn resource_matches(&self, pattern: &str, resource: &str) -> bool {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            resource.starts_with(prefix)
        } else {
            pattern == resource
        }
    }

    fn evaluate_conditions(&self, conditions: &[Condition], context: &AuthorizationContext) -> bool {
        for condition in conditions {
            match condition {
                Condition::TimeWindow { start, end } => {
                    let now = chrono::Utc::now().timestamp_millis() as u64;
                    if now < *start || now > *end {
                        return false;
                    }
                }
                Condition::RateLimit { max_requests, window_seconds: _ } => {
                    if context.request_count > *max_requests {
                        return false;
                    }
                }
                Condition::SourceRestriction { allowed_sources } => {
                    if !allowed_sources.contains(&context.source) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn test_bypass_prevention(&self) -> bool {
        let context = AuthorizationContext {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            source: "untrusted".to_string(),
            request_count: 100,
        };

        // Normal user shouldn't have admin access
        let normal_access = self.check_authorization(
            "normal_user",
            "descriptors/admin/sensitive",
            Permission::Write,
            &context,
        );

        // Admin should have access (simulated by different principal)
        let admin_context = AuthorizationContext {
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            source: "trusted".to_string(),
            request_count: 1,
        };

        let admin_access = self.check_authorization(
            "admin_user",
            "descriptors/admin/sensitive",
            Permission::Admin,
            &admin_context,
        );

        !normal_access // Normal user should be denied
    }
}

#[derive(Debug)]
pub struct AuthorizationContext {
    pub timestamp: u64,
    pub source: String,
    pub request_count: u64,
}

/// Pattern injection prevention
pub struct InjectionPrevention {
    pattern_validator: Arc<PatternValidator>,
    injection_attempts: Arc<RwLock<Vec<InjectionAttempt>>>,
}

#[derive(Debug, Clone)]
pub struct InjectionAttempt {
    pub timestamp: u64,
    pub pattern: String,
    pub injection_type: InjectionType,
    pub blocked: bool,
}

#[derive(Debug, Clone)]
pub enum InjectionType {
    SqlInjection,
    CommandInjection,
    PathTraversal,
    ScriptInjection,
    PatternInjection,
}

pub struct PatternValidator {
    allowed_patterns: Vec<String>,
    forbidden_sequences: Vec<String>,
}

impl PatternValidator {
    pub fn new() -> Self {
        Self {
            allowed_patterns: vec![
                r"^[a-zA-Z0-9_]+$".to_string(),
                r"^\d{1,10}$".to_string(),
            ],
            forbidden_sequences: vec![
                "..".to_string(),
                "//".to_string(),
                "\\".to_string(),
                ";".to_string(),
                "'".to_string(),
                "\"".to_string(),
                "--".to_string(),
                "/*".to_string(),
                "*/".to_string(),
                "<script>".to_string(),
                "</script>".to_string(),
                "eval(".to_string(),
                "exec(".to_string(),
            ],
        }
    }

    pub fn validate_pattern(&self, pattern: &str) -> Result<(), InjectionType> {
        // Check for forbidden sequences
        for forbidden in &self.forbidden_sequences {
            if pattern.contains(forbidden) {
                return Err(match forbidden.as_str() {
                    ".." | "//" | "\\" => InjectionType::PathTraversal,
                    ";" | "--" | "/*" | "*/" => InjectionType::SqlInjection,
                    "'" | "\"" => InjectionType::SqlInjection,
                    "<script>" | "</script>" => InjectionType::ScriptInjection,
                    "eval(" | "exec(" => InjectionType::CommandInjection,
                    _ => InjectionType::PatternInjection,
                });
            }
        }

        Ok(())
    }
}

impl InjectionPrevention {
    pub fn new() -> Self {
        Self {
            pattern_validator: Arc::new(PatternValidator::new()),
            injection_attempts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn check_pattern(&self, pattern: &str) -> bool {
        match self.pattern_validator.validate_pattern(pattern) {
            Ok(_) => true,
            Err(injection_type) => {
                self.injection_attempts.write().push(InjectionAttempt {
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    pattern: pattern.to_string(),
                    injection_type,
                    blocked: true,
                });
                false
            }
        }
    }

    pub fn test_injection_prevention(&self) -> bool {
        let test_cases = vec![
            ("valid_pattern", true),
            ("../../etc/passwd", false),
            ("'; DROP TABLE users; --", false),
            ("<script>alert('xss')</script>", false),
            ("eval(malicious_code)", false),
            ("normal_123", true),
        ];

        for (pattern, expected) in test_cases {
            if self.check_pattern(pattern) != expected {
                println!("Injection test failed for pattern: {}", pattern);
                return false;
            }
        }

        true
    }
}

use std::collections::HashMap;

impl SecurityTestHarness {
    pub fn new() -> Self {
        let key_manager = Arc::new(KeyManager::new());

        Self {
            key_manager: key_manager.clone(),
            signature_verifier: Arc::new(SignatureVerifier::new(key_manager.clone())),
            tamper_detector: Arc::new(TamperDetector::new()),
            authorization_enforcer: Arc::new(AuthorizationEnforcer::new()),
        }
    }

    pub fn run_all_security_tests(&self) -> SecurityTestResult {
        let mut passed = 0;
        let mut failed = 0;

        // Test signature verification
        if self.signature_verifier.test_forgery_prevention() {
            println!("✓ Signature forgery prevention: PASS");
            passed += 1;
        } else {
            println!("✗ Signature forgery prevention: FAIL");
            failed += 1;
        }

        // Test tamper detection
        if self.tamper_detector.test_tamper_detection() {
            println!("✓ Tamper detection: PASS");
            passed += 1;
        } else {
            println!("✗ Tamper detection: FAIL");
            failed += 1;
        }

        // Test authorization bypass prevention
        if self.authorization_enforcer.test_bypass_prevention() {
            println!("✓ Authorization bypass prevention: PASS");
            passed += 1;
        } else {
            println!("✗ Authorization bypass prevention: FAIL");
            failed += 1;
        }

        // Test injection prevention
        let injection_prevention = InjectionPrevention::new();
        if injection_prevention.test_injection_prevention() {
            println!("✓ Injection prevention: PASS");
            passed += 1;
        } else {
            println!("✗ Injection prevention: FAIL");
            failed += 1;
        }

        SecurityTestResult {
            tests_passed: passed,
            tests_failed: failed,
            security_score: (passed as f64 / (passed + failed) as f64) * 100.0,
        }
    }
}

#[derive(Debug)]
pub struct SecurityTestResult {
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub security_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_security_validation() {
        let harness = SecurityTestHarness::new();
        let result = harness.run_all_security_tests();

        assert_eq!(result.tests_failed, 0, "Security tests failed");
        assert_eq!(result.security_score, 100.0, "Security score below 100%");

        println!("\nSecurity Test Summary:");
        println!("  Tests passed: {}", result.tests_passed);
        println!("  Tests failed: {}", result.tests_failed);
        println!("  Security score: {:.2}%", result.security_score);
    }

    #[test]
    fn test_key_rotation() {
        let key_manager = KeyManager::new();

        let initial_key = key_manager.get_current_signing_key();
        assert!(initial_key.is_some());

        key_manager.rotate_keys().unwrap();

        let new_key = key_manager.get_current_signing_key();
        assert!(new_key.is_some());
        assert_ne!(initial_key, new_key);
    }

    #[test]
    fn test_signature_verification() {
        let key_manager = Arc::new(KeyManager::new());
        let verifier = SignatureVerifier::new(key_manager);

        let data = b"test data";
        let signature = verifier.sign_descriptor(data).unwrap();

        assert!(verifier.verify_signature(data, &signature));

        // Test with modified data
        let modified_data = b"modified data";
        assert!(!verifier.verify_signature(modified_data, &signature));
    }
}