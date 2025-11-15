// Chicago TDD Example: User Registration
//
// Demonstrates Chicago TDD principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators (no mocks for user repository)
// 3. AAA pattern (Arrange, Act, Assert)
// 4. Test behavior, not implementation
//
// Three test cases:
// - Successful registration
// - Duplicate email (error case)
// - Invalid email format (validation error)

use std::collections::HashMap;

/// User entity
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: String,
    email: String,
    name: String,
}

impl User {
    fn new(id: String, email: String, name: String) -> Self {
        Self { id, email, name }
    }
}

/// User repository (real implementation, not mock)
struct UserRepository {
    users: HashMap<String, User>,
}

impl UserRepository {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Save user to repository
    fn save(&mut self, user: User) -> Result<(), String> {
        if self.users.contains_key(&user.email) {
            return Err(format!("User with email {} already exists", user.email));
        }

        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    /// Find user by email
    fn find_by_email(&self, email: &str) -> Option<&User> {
        self.users.get(email)
    }

    /// Count total users
    fn count(&self) -> usize {
        self.users.len()
    }
}

/// Email validation
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// User registration service
struct UserRegistrationService {
    repository: UserRepository,
}

impl UserRegistrationService {
    fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    /// Register new user
    /// Returns user ID on success, error message on failure
    fn register_user(&mut self, email: String, name: String) -> Result<String, String> {
        // Validate email format
        if !is_valid_email(&email) {
            return Err(format!("Invalid email format: {}", email));
        }

        // Generate user ID (simplified)
        let user_id = format!("user_{}", self.repository.count() + 1);

        // Create user
        let user = User::new(user_id.clone(), email, name);

        // Save to repository
        self.repository.save(user)?;

        Ok(user_id)
    }

    /// Get user by email
    fn get_user(&self, email: &str) -> Option<&User> {
        self.repository.find_by_email(email)
    }
}

// ============================================================================
// Tests (Chicago TDD Style)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Successful registration
    // Chicago TDD: Test observable outcome (user saved in repository)
    #[test]
    fn test_user_registration_success() {
        // Arrange: Create service with empty repository
        let repository = UserRepository::new();
        let mut service = UserRegistrationService::new(repository);

        // Act: Register new user
        let result = service.register_user("alice@example.com".to_string(), "Alice".to_string());

        // Assert: Registration succeeded
        assert!(result.is_ok(), "Registration should succeed");
        let user_id = result.unwrap();

        // Assert: User saved in repository (state-based verification)
        let saved_user = service
            .get_user("alice@example.com")
            .expect("User should be saved");
        assert_eq!(saved_user.id, user_id);
        assert_eq!(saved_user.email, "alice@example.com");
        assert_eq!(saved_user.name, "Alice");

        // Assert: Repository state changed
        assert_eq!(
            service.repository.count(),
            1,
            "Repository should have 1 user"
        );
    }

    // Test 2: Duplicate email (error case)
    // Chicago TDD: Test error behavior with real repository
    #[test]
    fn test_user_registration_duplicate_email() {
        // Arrange: Create service and register first user
        let repository = UserRepository::new();
        let mut service = UserRegistrationService::new(repository);

        service
            .register_user("bob@example.com".to_string(), "Bob".to_string())
            .expect("First registration should succeed");

        // Act: Attempt to register user with same email
        let result = service.register_user("bob@example.com".to_string(), "Robert".to_string());

        // Assert: Registration failed with appropriate error
        assert!(result.is_err(), "Registration should fail for duplicate email");
        let error = result.unwrap_err();
        assert!(
            error.contains("already exists"),
            "Error should mention duplicate, got: {}",
            error
        );

        // Assert: Repository state unchanged (still 1 user, not 2)
        assert_eq!(
            service.repository.count(),
            1,
            "Repository should still have 1 user"
        );

        // Assert: Original user unchanged
        let saved_user = service
            .get_user("bob@example.com")
            .expect("Original user should exist");
        assert_eq!(saved_user.name, "Bob", "Original user name unchanged");
    }

    // Test 3: Invalid email format
    // Chicago TDD: Test validation with real service
    #[test]
    fn test_user_registration_invalid_email() {
        // Arrange: Create service with empty repository
        let repository = UserRepository::new();
        let mut service = UserRegistrationService::new(repository);

        // Act: Attempt to register user with invalid email
        let result = service.register_user("invalid-email".to_string(), "Charlie".to_string());

        // Assert: Registration failed with appropriate error
        assert!(
            result.is_err(),
            "Registration should fail for invalid email"
        );
        let error = result.unwrap_err();
        assert!(
            error.contains("Invalid email format"),
            "Error should mention invalid email, got: {}",
            error
        );

        // Assert: No user saved in repository (state unchanged)
        assert_eq!(
            service.repository.count(),
            0,
            "Repository should be empty"
        );
        assert!(
            service.get_user("invalid-email").is_none(),
            "Invalid user should not be saved"
        );
    }

    // Additional Test: Multiple registrations
    // Chicago TDD: Test state changes with multiple operations
    #[test]
    fn test_multiple_user_registrations() {
        // Arrange: Create service
        let repository = UserRepository::new();
        let mut service = UserRegistrationService::new(repository);

        // Act: Register multiple users
        let user1 = service
            .register_user("alice@example.com".to_string(), "Alice".to_string())
            .expect("Register Alice");
        let user2 = service
            .register_user("bob@example.com".to_string(), "Bob".to_string())
            .expect("Register Bob");
        let user3 = service
            .register_user("charlie@example.com".to_string(), "Charlie".to_string())
            .expect("Register Charlie");

        // Assert: All users saved
        assert_eq!(service.repository.count(), 3, "Should have 3 users");

        // Assert: Each user retrievable
        assert!(service.get_user("alice@example.com").is_some());
        assert!(service.get_user("bob@example.com").is_some());
        assert!(service.get_user("charlie@example.com").is_some());

        // Assert: User IDs unique
        assert_ne!(user1, user2);
        assert_ne!(user2, user3);
        assert_ne!(user1, user3);
    }
}

fn main() {
    println!("=== Chicago TDD Example: User Registration ===\n");

    // Run all tests
    println!("Running tests...\n");

    // Simulate test execution (in real code, use `cargo test`)
    let repository = UserRepository::new();
    let mut service = UserRegistrationService::new(repository);

    // Example 1: Successful registration
    println!("--- Test 1: Successful Registration ---");
    match service.register_user("alice@example.com".to_string(), "Alice".to_string()) {
        Ok(user_id) => {
            println!("✅ User registered: {}", user_id);
            println!("   Email: alice@example.com");
            println!("   Name: Alice");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // Example 2: Duplicate email
    println!("--- Test 2: Duplicate Email ---");
    match service.register_user("alice@example.com".to_string(), "Alice2".to_string()) {
        Ok(user_id) => println!("❌ Should have failed, got: {}", user_id),
        Err(e) => println!("✅ Correctly rejected: {}", e),
    }
    println!();

    // Example 3: Invalid email
    println!("--- Test 3: Invalid Email ---");
    match service.register_user("invalid-email".to_string(), "Bob".to_string()) {
        Ok(user_id) => println!("❌ Should have failed, got: {}", user_id),
        Err(e) => println!("✅ Correctly rejected: {}", e),
    }
    println!();

    println!("=== Chicago TDD Principles ===");
    println!("1. ✅ State-based verification (check repository state, not method calls)");
    println!("2. ✅ Real collaborators (UserRepository is real, not mocked)");
    println!("3. ✅ AAA pattern (Arrange, Act, Assert)");
    println!("4. ✅ Test behavior (user saved/not saved, not internal implementation)");
    println!("5. ✅ Test outcomes (verify final state, not interactions)");
    println!();

    println!("=== Comparison: Chicago vs London TDD ===");
    println!("\nChicago TDD (Used Here):");
    println!("  - Use real UserRepository");
    println!("  - Verify user saved in repository (state)");
    println!("  - Tests remain valid after refactoring");
    println!();

    println!("London TDD (Not Used):");
    println!("  - Mock UserRepository");
    println!("  - Verify repository.save() called (interaction)");
    println!("  - Tests break if implementation changes");
    println!();

    println!("=== Run Unit Tests ===");
    println!("cargo test --test chicago_tdd_user_registration");
}

// Key Takeaways:
//
// 1. **State-Based Verification**: Test observable outcomes
//    - User saved in repository? ✅
//    - Repository count increased? ✅
//    - User retrievable by email? ✅
//
// 2. **Real Collaborators**: Use actual UserRepository
//    - No mocks for core business logic
//    - Tests prove real components work together
//    - More confidence in production behavior
//
// 3. **AAA Pattern**: Clear test structure
//    - Arrange: Set up test data and services
//    - Act: Execute operation being tested
//    - Assert: Verify outcomes and state changes
//
// 4. **Test Behavior, Not Implementation**:
//    - Don't verify internal method calls
//    - Verify observable outcomes (saved user, error messages)
//    - Tests survive refactoring
//
// 5. **Error Cases Matter**:
//    - Test both success and failure paths
//    - Verify error messages are descriptive
//    - Ensure state unchanged on errors
//
// When to use mocks vs real collaborators:
// - ✅ Real: Core business logic (repositories, services)
// - ✅ Real: In-memory databases (fast, isolated)
// - ❌ Mock: External APIs (network calls to 3rd party)
// - ❌ Mock: Slow operations (only if can't be made fast)
// - ❌ Mock: Non-deterministic operations (time, randomness)
//
// Run tests:
// $ cargo test --test chicago_tdd_user_registration -- --nocapture
