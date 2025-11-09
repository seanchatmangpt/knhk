// rust/knhk-etl/src/hook_registry.rs
// Hook registry maps predicates to validation kernels and guards
// LAW: Hooks implement μ ⊣ H at ingress. Guards enforce O ⊨ Σ and preserve Q.

extern crate alloc;

use crate::ingest::RawTriple;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use knhk_hot::KernelType;
use std::sync::Arc;

/// Shared reference to HookRegistry for Arc-based sharing
pub type SharedHookRegistry = Arc<HookRegistry>;

/// Guard function type: validates triple against invariants
/// Returns true if triple passes guard, false otherwise
pub type GuardFn = Box<dyn Fn(&RawTriple) -> bool + Send + Sync>;

/// Hook metadata: compiled hook information
#[derive(Debug, Clone)]
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

    /// Hook counter for generating unique IDs
    #[allow(dead_code)]
    hook_counter: AtomicU64,
}

// NOTE: HookRegistry is NOT Clone because guard_map contains function pointers.
// Use Arc<HookRegistry> (or SharedHookRegistry type alias) for shared ownership.

impl HookRegistry {
    /// Create new hook registry with default kernel
    pub fn new() -> Self {
        Self {
            kernel_map: BTreeMap::new(),
            guard_map: BTreeMap::new(),
            hooks: Vec::new(),
            default_kernel: KernelType::AskSp, // Conservative default
            hook_counter: AtomicU64::new(0),
        }
    }

    /// Create with custom default kernel
    pub fn with_default_kernel(default_kernel: KernelType) -> Self {
        Self {
            kernel_map: BTreeMap::new(),
            guard_map: BTreeMap::new(),
            hooks: Vec::new(),
            default_kernel,
            hook_counter: AtomicU64::new(0),
        }
    }

    /// Create a shared HookRegistry wrapped in Arc
    /// Use this when you need to share the registry across multiple contexts
    pub fn shared() -> SharedHookRegistry {
        Arc::new(Self::new())
    }

    /// Create a shared HookRegistry with custom default kernel
    pub fn shared_with_default(default_kernel: KernelType) -> SharedHookRegistry {
        Arc::new(Self::with_default_kernel(default_kernel))
    }

    /// Registers a validation hook for a predicate with kernel and guard.
    ///
    /// # Purpose
    /// Implements the KNHK LAW: μ ⊣ H (hooks at ingress). Associates a predicate
    /// with a validation kernel and guard function to enforce schema invariants.
    /// Each predicate can have at most one hook (no duplicates allowed).
    ///
    /// # Arguments
    /// * `predicate` - Predicate ID (u64 hash) this hook validates:
    ///   - Example: `hash("http://example.org/name")` → predicate ID
    ///   - Obtained from Transform stage hashing
    /// * `kernel_type` - Validation kernel to execute:
    ///   - `KernelType::AskSp` - Check if (subject, predicate) exists
    ///   - `KernelType::CountSpGe` - Count assertions for cardinality checks
    ///   - `KernelType::ValidateSp` - Full schema validation
    /// * `guard` - Guard function `fn(&RawTriple) -> bool` that validates triples:
    ///   - Returns `true` if triple passes validation
    ///   - Returns `false` if triple violates invariants (reject or escalate)
    /// * `invariants` - List of invariants this hook preserves (Q):
    ///   - Example: `["cardinality >= 1", "object is URI"]`
    ///   - Used for documentation and verification
    ///
    /// # Returns
    /// * `Ok(hook_id)` - Successfully registered hook with unique ID
    /// * `Err(HookRegistryError)` - Registration failed (see Errors)
    ///
    /// # Errors
    /// * `HookRegistryError::DuplicatePredicate` - Hook already exists for this predicate
    ///
    /// # Performance
    /// * Registration: O(log n) BTreeMap insert
    /// * Hook lookup: O(log n) during pipeline execution
    /// * Guard execution: User-defined, should be ≤8 ticks
    ///
    /// # Example
    /// ```rust
    /// use knhk_etl::hook_registry::{HookRegistry, guards};
    /// use knhk_hot::KernelType;
    ///
    /// let mut registry = HookRegistry::new();
    ///
    /// // Register hook for name predicate (cardinality: exactly 1)
    /// let name_predicate = 200; // Hash of "http://example.org/name"
    /// let hook_id = registry.register_hook(
    ///     name_predicate,
    ///     KernelType::ValidateSp,
    ///     guards::check_object_nonempty, // Guard: object must be non-empty
    ///     vec!["cardinality == 1".to_string(), "object is literal".to_string()],
    /// ).expect("Failed to register hook");
    ///
    /// println!("Registered hook ID: {}", hook_id);
    ///
    /// // Register hook for age predicate (must be integer)
    /// let age_predicate = 201;
    /// registry.register_hook(
    ///     age_predicate,
    ///     KernelType::ValidateSp,
    ///     guards::check_object_integer, // Guard: object must be integer
    ///     vec!["object is xsd:integer".to_string()],
    /// ).expect("Failed to register hook");
    /// ```
    ///
    /// # Custom Guards
    /// ```rust
    /// use knhk_etl::hook_registry::{HookRegistry, GuardFn};
    /// use knhk_etl::RawTriple;
    /// use knhk_hot::KernelType;
    ///
    /// // Custom guard: check email format
    /// fn check_email_format(triple: &RawTriple) -> bool {
    ///     triple.object.contains("@") && triple.object.contains(".")
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let email_predicate = 300;
    /// registry.register_hook(
    ///     email_predicate,
    ///     KernelType::ValidateSp,
    ///     check_email_format,
    ///     vec!["object is valid email".to_string()],
    /// ).expect("Failed to register hook");
    /// ```
    ///
    /// # See Also
    /// * [`HookRegistry::get_kernel`] - Lookup kernel for predicate
    /// * [`HookRegistry::check_guard`] - Execute guard validation
    /// * [`guards`] - Standard guard functions module
    /// * [`KernelType`] - Available validation kernels
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
        self.kernel_map
            .get(&predicate)
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
        // Note: Assertion conflict checking will be implemented in v1.1
        // For v1.0, hook registration does not check against existing assertions
        // For now, always pass (conservative)
        true
    }

    /// Check that triple has no graph (default graph only)
    pub fn check_default_graph_only(triple: &RawTriple) -> bool {
        triple.graph.is_none()
    }

    /// Compose two guards with AND logic
    pub fn and_guard(guard1: GuardFn, guard2: GuardFn) -> GuardFn {
        Box::new(move |triple: &RawTriple| guard1(triple) && guard2(triple))
    }

    /// Compose two guards with OR logic
    pub fn or_guard(guard1: GuardFn, guard2: GuardFn) -> impl Fn(&RawTriple) -> bool {
        move |triple: &RawTriple| guard1(triple) || guard2(triple)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_hook_registration() {
        let mut registry = HookRegistry::new();

        let hook_id = registry
            .register_hook(
                200, // predicate
                KernelType::AskSp,
                Box::new(guards::always_valid),
                vec!["cardinality >= 1".to_string()],
            )
            .expect("Hook registration should succeed");

        assert_eq!(hook_id, 0);
        assert_eq!(registry.get_kernel(200), KernelType::AskSp);
        assert!(registry.has_hook(200));
    }

    #[test]
    fn test_duplicate_predicate_error() {
        let mut registry = HookRegistry::new();

        registry
            .register_hook(
                200,
                KernelType::AskSp,
                Box::new(guards::always_valid),
                vec![],
            )
            .expect("Hook registration should succeed");

        // Try to register again
        let result = registry.register_hook(
            200,
            KernelType::CountSpGe,
            Box::new(guards::always_valid),
            vec![],
        );

        assert!(matches!(
            result,
            Err(HookRegistryError::DuplicatePredicate(200))
        ));
    }

    #[test]
    fn test_guard_execution() {
        let mut registry = HookRegistry::new();

        registry
            .register_hook(
                200,
                KernelType::ValidateSp,
                Box::new(guards::check_subject_nonempty),
                vec![],
            )
            .expect("Hook registration should succeed");

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

        registry
            .register_hook(
                200,
                KernelType::AskSp,
                Box::new(guards::always_valid),
                vec![],
            )
            .expect("Hook registration should succeed");
        assert!(registry.has_hook(200));

        registry
            .unregister_hook(200)
            .expect("Hook unregistration should succeed");
        assert!(!registry.has_hook(200));
    }

    #[test]
    fn test_get_hook_by_predicate() {
        let mut registry = HookRegistry::new();

        registry
            .register_hook(
                200,
                KernelType::AskSp,
                Box::new(guards::always_valid),
                vec!["test invariant".to_string()],
            )
            .expect("Hook registration should succeed");

        let hook = registry
            .get_hook_by_predicate(200)
            .expect("Hook lookup should succeed");
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

        registry
            .register_hook(
                200,
                KernelType::AskSp,
                Box::new(guards::always_valid),
                vec![],
            )
            .expect("Hook registration should succeed");
        registry
            .register_hook(
                201,
                KernelType::CountSpGe,
                Box::new(guards::always_valid),
                vec![],
            )
            .expect("Hook registration should succeed");

        let hooks = registry.list_hooks();
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].predicate, 200);
        assert_eq!(hooks[1].predicate, 201);
    }
}
