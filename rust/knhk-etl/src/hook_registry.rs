// rust/knhk-etl/src/hook_registry.rs
// Hook registry maps predicates to validation kernels and guards
// LAW: Hooks implement μ ⊣ H at ingress. Guards enforce O ⊨ Σ and preserve Q.

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use knhk_hot::KernelType;
use crate::ingest::RawTriple;

/// Guard function type: validates triple against invariants
/// Returns true if triple passes guard, false otherwise
pub type GuardFn = fn(&RawTriple) -> bool;

/// Hook metadata: compiled hook information
#[derive(Clone, Debug)]
pub struct HookMetadata {
    /// Unique hook ID
    pub id: u64,
    /// Predicate this hook applies to
    pub predicate: u64,
    /// Kernel type for validation/computation
    pub kernel_type: KernelType,
    /// Invariants this hook must preserve (Q)
    pub invariants: Vec<String>,
    /// Timestamp when hook was compiled
    pub compiled_at: u64,
    /// Hash of hook template (for verification)
    pub hash: [u8; 32],
}

/// Hook registry for predicate-to-kernel mapping
/// Implements μ ⊣ H: hooks at ingress point
pub struct HookRegistry {
    /// Predicate ID → Kernel type mapping
    kernel_map: BTreeMap<u64, KernelType>,

    /// Predicate ID → Guard function mapping
    guard_map: BTreeMap<u64, GuardFn>,

    /// Compiled hook metadata
    hooks: Vec<HookMetadata>,

    /// Default kernel type for unregistered predicates
    default_kernel: KernelType,
}

impl HookRegistry {
    /// Create new hook registry with default kernel
    pub fn new() -> Self {
        Self {
            kernel_map: BTreeMap::new(),
            guard_map: BTreeMap::new(),
            hooks: Vec::new(),
            default_kernel: KernelType::AskSp, // Conservative default
        }
    }

    /// Create with custom default kernel
    pub fn with_default_kernel(default_kernel: KernelType) -> Self {
        Self {
            kernel_map: BTreeMap::new(),
            guard_map: BTreeMap::new(),
            hooks: Vec::new(),
            default_kernel,
        }
    }

    /// Register a hook: predicate → kernel + guard
    ///
    /// # Arguments
    /// * `predicate` - Predicate ID this hook applies to
    /// * `kernel_type` - Kernel to execute for this predicate
    /// * `guard` - Guard function to validate triples
    /// * `invariants` - List of invariants this hook must preserve (Q)
    ///
    /// # Returns
    /// - Ok(hook_id): Successfully registered hook
    /// - Err(error): Registration failed (duplicate predicate)
    pub fn register_hook(
        &mut self,
        predicate: u64,
        kernel_type: KernelType,
        guard: GuardFn,
        invariants: Vec<String>,
    ) -> Result<u64, HookRegistryError> {
        // Check for conflicts (one hook per predicate)
        if self.kernel_map.contains_key(&predicate) {
            return Err(HookRegistryError::DuplicatePredicate(predicate));
        }

        // Generate hook ID (sequential)
        let hook_id = self.hooks.len() as u64;

        // Store mappings
        self.kernel_map.insert(predicate, kernel_type);
        self.guard_map.insert(predicate, guard);

        // Compute hook hash (for verification)
        let hash = self.compute_hook_hash(predicate, kernel_type);

        // Store metadata
        let metadata = HookMetadata {
            id: hook_id,
            predicate,
            kernel_type,
            invariants,
            compiled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
            hash,
        };

        self.hooks.push(metadata);

        Ok(hook_id)
    }

    /// Get kernel type for predicate (returns default if not registered)
    pub fn get_kernel(&self, predicate: u64) -> KernelType {
        self.kernel_map.get(&predicate)
            .copied()
            .unwrap_or(self.default_kernel)
    }

    /// Execute guard for predicate
    ///
    /// LAW: O ⊨ Σ (observations must conform to schema)
    ///
    /// # Returns
    /// - true: Triple passes guard validation
    /// - false: Triple fails guard (reject or escalate)
    pub fn check_guard(&self, predicate: u64, triple: &RawTriple) -> bool {
        if let Some(guard_fn) = self.guard_map.get(&predicate) {
            guard_fn(triple)
        } else {
            // No guard registered: use default behavior
            // Conservative: reject unknown predicates
            false
        }
    }

    /// Get hook metadata by hook ID
    pub fn get_hook(&self, hook_id: u64) -> Option<&HookMetadata> {
        self.hooks.get(hook_id as usize)
    }

    /// Get hook metadata by predicate ID
    pub fn get_hook_by_predicate(&self, predicate: u64) -> Option<&HookMetadata> {
        self.hooks.iter().find(|h| h.predicate == predicate)
    }

    /// List all registered hooks
    pub fn list_hooks(&self) -> &[HookMetadata] {
        &self.hooks
    }

    /// Count registered hooks
    pub fn count(&self) -> usize {
        self.hooks.len()
    }

    /// Check if predicate has registered hook
    pub fn has_hook(&self, predicate: u64) -> bool {
        self.kernel_map.contains_key(&predicate)
    }

    /// Unregister hook (careful: breaks active pipelines)
    pub fn unregister_hook(&mut self, predicate: u64) -> Result<(), HookRegistryError> {
        if !self.kernel_map.contains_key(&predicate) {
            return Err(HookRegistryError::NoHookFound(predicate));
        }

        self.kernel_map.remove(&predicate);
        self.guard_map.remove(&predicate);

        // Remove from metadata list
        self.hooks.retain(|h| h.predicate != predicate);

        Ok(())
    }

    /// Compute hash of hook template (for verification)
    fn compute_hook_hash(&self, predicate: u64, kernel_type: KernelType) -> [u8; 32] {
        // Use std DefaultHasher for now (blake3 requires external crate)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        predicate.hash(&mut hasher);
        (kernel_type as u8).hash(&mut hasher);
        let hash = hasher.finish();

        // Convert u64 hash to [u8; 32] by repeating
        let mut result = [0u8; 32];
        let hash_bytes = hash.to_le_bytes();
        for (i, byte) in result.iter_mut().enumerate() {
            *byte = hash_bytes[i % 8];
        }
        result
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook registry errors
#[derive(Debug, Clone, PartialEq)]
pub enum HookRegistryError {
    /// Attempted to register hook for predicate that already has one
    DuplicatePredicate(u64),
    /// Attempted to access hook that doesn't exist
    NoHookFound(u64),
}

impl core::fmt::Display for HookRegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HookRegistryError::DuplicatePredicate(p) => {
                write!(f, "Hook already registered for predicate {}", p)
            }
            HookRegistryError::NoHookFound(p) => {
                write!(f, "No hook found for predicate {}", p)
            }
        }
    }
}

/// Standard guard functions
/// These implement common validation patterns
pub mod guards {
    use super::*;

    /// Always valid guard (pass-through)
    /// Use for predicates with no constraints
    pub fn always_valid(_triple: &RawTriple) -> bool {
        true
    }

    /// Never valid guard (always reject)
    /// Use for disabled predicates
    pub fn always_reject(_triple: &RawTriple) -> bool {
        false
    }

    /// Check that subject is non-empty
    pub fn check_subject_nonempty(triple: &RawTriple) -> bool {
        !triple.subject.is_empty()
    }

    /// Check that object is non-empty
    pub fn check_object_nonempty(triple: &RawTriple) -> bool {
        !triple.object.is_empty()
    }

    /// Check that predicate matches expected
    pub fn check_predicate_matches(expected: &str) -> impl Fn(&RawTriple) -> bool + '_ {
        move |triple: &RawTriple| triple.predicate == expected
    }

    /// Check that object is a valid integer literal
    pub fn check_object_integer(triple: &RawTriple) -> bool {
        // Remove quotes and try to parse
        let obj = triple.object.trim_matches('"');
        obj.parse::<i64>().is_ok()
    }

    /// Check that object is a valid URI (starts with http:// or https://)
    pub fn check_object_uri(triple: &RawTriple) -> bool {
        triple.object.starts_with("http://") || triple.object.starts_with("https://")
    }

    /// Check that subject is a URI
    pub fn check_subject_uri(triple: &RawTriple) -> bool {
        triple.subject.starts_with("http://") || triple.subject.starts_with("https://")
    }

    /// Check cardinality: at most one (would need context for full implementation)
    /// This is a placeholder - full implementation needs access to existing assertions
    pub fn check_cardinality_one(_triple: &RawTriple) -> bool {
        // TODO: Check against existing assertions in store
        // For now, always pass (conservative)
        true
    }

    /// Check that triple has no graph (default graph only)
    pub fn check_default_graph_only(triple: &RawTriple) -> bool {
        triple.graph.is_none()
    }

    /// Compose two guards with AND logic
    pub fn and_guard(
        guard1: GuardFn,
        guard2: GuardFn,
    ) -> impl Fn(&RawTriple) -> bool {
        move |triple: &RawTriple| guard1(triple) && guard2(triple)
    }

    /// Compose two guards with OR logic
    pub fn or_guard(
        guard1: GuardFn,
        guard2: GuardFn,
    ) -> impl Fn(&RawTriple) -> bool {
        move |triple: &RawTriple| guard1(triple) || guard2(triple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_registration() {
        let mut registry = HookRegistry::new();

        let hook_id = registry.register_hook(
            200, // predicate
            KernelType::AskSp,
            guards::always_valid,
            vec!["cardinality >= 1".to_string()],
        ).unwrap();

        assert_eq!(hook_id, 0);
        assert_eq!(registry.get_kernel(200), KernelType::AskSp);
        assert!(registry.has_hook(200));
    }

    #[test]
    fn test_duplicate_predicate_error() {
        let mut registry = HookRegistry::new();

        registry.register_hook(
            200,
            KernelType::AskSp,
            guards::always_valid,
            vec![],
        ).unwrap();

        // Try to register again
        let result = registry.register_hook(
            200,
            KernelType::CountSpGe,
            guards::always_valid,
            vec![],
        );

        assert!(matches!(result, Err(HookRegistryError::DuplicatePredicate(200))));
    }

    #[test]
    fn test_guard_execution() {
        let mut registry = HookRegistry::new();

        registry.register_hook(
            200,
            KernelType::ValidateSp,
            guards::check_subject_nonempty,
            vec![],
        ).unwrap();

        let valid_triple = RawTriple {
            subject: "http://example.org/subject".to_string(),
            predicate: "http://example.org/predicate".to_string(),
            object: "http://example.org/object".to_string(),
            graph: None,
        };

        let invalid_triple = RawTriple {
            subject: "".to_string(),
            predicate: "http://example.org/predicate".to_string(),
            object: "http://example.org/object".to_string(),
            graph: None,
        };

        assert!(registry.check_guard(200, &valid_triple));
        assert!(!registry.check_guard(200, &invalid_triple));
    }

    #[test]
    fn test_default_kernel() {
        let registry = HookRegistry::with_default_kernel(KernelType::CountSpGe);

        // Unregistered predicate should return default
        assert_eq!(registry.get_kernel(999), KernelType::CountSpGe);
    }

    #[test]
    fn test_unregister_hook() {
        let mut registry = HookRegistry::new();

        registry.register_hook(200, KernelType::AskSp, guards::always_valid, vec![]).unwrap();
        assert!(registry.has_hook(200));

        registry.unregister_hook(200).unwrap();
        assert!(!registry.has_hook(200));
    }

    #[test]
    fn test_get_hook_by_predicate() {
        let mut registry = HookRegistry::new();

        registry.register_hook(
            200,
            KernelType::AskSp,
            guards::always_valid,
            vec!["test invariant".to_string()],
        ).unwrap();

        let hook = registry.get_hook_by_predicate(200).unwrap();
        assert_eq!(hook.predicate, 200);
        assert_eq!(hook.kernel_type, KernelType::AskSp);
        assert_eq!(hook.invariants.len(), 1);
    }

    #[test]
    fn test_guard_functions() {
        let triple = RawTriple {
            subject: "http://example.org/subject".to_string(),
            predicate: "http://example.org/predicate".to_string(),
            object: "\"42\"".to_string(),
            graph: None,
        };

        assert!(guards::always_valid(&triple));
        assert!(!guards::always_reject(&triple));
        assert!(guards::check_subject_nonempty(&triple));
        assert!(guards::check_object_nonempty(&triple));
        assert!(guards::check_subject_uri(&triple));
        assert!(guards::check_object_integer(&triple));
        assert!(guards::check_default_graph_only(&triple));
    }

    #[test]
    fn test_guard_object_uri() {
        let uri_triple = RawTriple {
            subject: "http://example.org/subject".to_string(),
            predicate: "http://example.org/predicate".to_string(),
            object: "http://example.org/object".to_string(),
            graph: None,
        };

        let literal_triple = RawTriple {
            subject: "http://example.org/subject".to_string(),
            predicate: "http://example.org/predicate".to_string(),
            object: "\"not a uri\"".to_string(),
            graph: None,
        };

        assert!(guards::check_object_uri(&uri_triple));
        assert!(!guards::check_object_uri(&literal_triple));
    }

    #[test]
    fn test_list_hooks() {
        let mut registry = HookRegistry::new();

        registry.register_hook(200, KernelType::AskSp, guards::always_valid, vec![]).unwrap();
        registry.register_hook(201, KernelType::CountSpGe, guards::always_valid, vec![]).unwrap();

        let hooks = registry.list_hooks();
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].predicate, 200);
        assert_eq!(hooks[1].predicate, 201);
    }
}
