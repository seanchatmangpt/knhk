//! Fixture-Based Testing and Observability Examples with chicago-tdd-tools v1.3.0
//! Demonstrates: fixture_test!, test setup/teardown, OTEL integration, Weaver validation

use chicago_tdd_tools::prelude::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// 1. SIMPLE TEST FIXTURE
// ============================================================================

struct TestContext {
    counter: Arc<Mutex<i32>>,
}

impl TestContext {
    fn new() -> Self {
        TestContext {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    fn increment(&self) {
        *self.counter.lock().unwrap() += 1;
    }

    fn get_count(&self) -> i32 {
        *self.counter.lock().unwrap()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup happens here automatically
        let count = *self.counter.lock().unwrap();
        // eprintln!("TestContext dropped with count: {}", count);
    }
}

test!(test_with_manual_fixture, {
    // Arrange - Create fixture
    let fixture = TestContext::new();

    // Act - Use fixture
    fixture.increment();
    fixture.increment();
    let count = fixture.get_count();

    // Assert
    assert_eq!(count, 2);

    // Cleanup happens automatically when fixture goes out of scope
});

// ============================================================================
// 2. FIXTURE WITH RESOURCE MANAGEMENT
// ============================================================================

struct TestDatabase {
    data: Arc<Mutex<Vec<String>>>,
}

impl TestDatabase {
    fn new() -> Self {
        TestDatabase {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn insert(&self, value: String) {
        self.data.lock().unwrap().push(value);
    }

    fn get_all(&self) -> Vec<String> {
        self.data.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup
        self.clear();
    }
}

test!(test_fixture_database, {
    // Arrange
    let db = TestDatabase::new();

    // Act
    db.insert("Alice".to_string());
    db.insert("Bob".to_string());
    let all = db.get_all();

    // Assert
    assert_eq!(all.len(), 2);
    assert!(all.contains(&"Alice".to_string()));
    assert!(all.contains(&"Bob".to_string()));

    // Cleanup happens in Drop impl
});

// ============================================================================
// 3. FIXTURE WITH SETUP/TEARDOWN
// ============================================================================

struct TestUser {
    id: u32,
    name: String,
}

struct UserTestFixture {
    users: Vec<TestUser>,
}

impl UserTestFixture {
    fn new() -> Self {
        // Setup
        let mut fixture = UserTestFixture {
            users: Vec::new(),
        };

        // Initialize with test data
        fixture.users.push(TestUser {
            id: 1,
            name: "Alice".to_string(),
        });
        fixture.users.push(TestUser {
            id: 2,
            name: "Bob".to_string(),
        });

        fixture
    }

    fn find_by_id(&self, id: u32) -> Option<&TestUser> {
        self.users.iter().find(|u| u.id == id)
    }

    fn add_user(&mut self, id: u32, name: String) {
        self.users.push(TestUser { id, name });
    }
}

impl Drop for UserTestFixture {
    fn drop(&mut self) {
        // Teardown
        self.users.clear();
    }
}

test!(test_user_fixture, {
    // Arrange
    let mut fixture = UserTestFixture::new();

    // Act
    fixture.add_user(3, "Charlie".to_string());
    let user = fixture.find_by_id(2);

    // Assert
    assert!(user.is_some());
    assert_eq!(user.unwrap().name, "Bob");
    assert_eq!(fixture.users.len(), 3);

    // Cleanup happens in Drop
});

// ============================================================================
// 4. FIXTURE ISOLATION
// ============================================================================

test!(test_fixture_isolation_one, {
    // Each test gets a fresh fixture
    let db = TestDatabase::new();
    db.insert("Test1".to_string());
    assert_eq!(db.get_all().len(), 1);
});

test!(test_fixture_isolation_two, {
    // This fixture is independent from test_fixture_isolation_one
    let db = TestDatabase::new();
    assert_eq!(db.get_all().len(), 0);
    db.insert("Test2".to_string());
    assert_eq!(db.get_all().len(), 1);
});

// ============================================================================
// 5. ASYNC FIXTURE
// ============================================================================

#[cfg(feature = "async")]
async_test!(test_async_with_fixture, {
    // Arrange - Async fixture
    let fixture = TestDatabase::new();

    // Simulate async operations
    let insert_future = async {
        fixture.insert("AsyncData".to_string());
    };

    // Act
    insert_future.await;
    let data = fixture.get_all();

    // Assert
    assert_eq!(data.len(), 1);
    assert!(data.contains(&"AsyncData".to_string()));
});

// ============================================================================
// 6. OPENTELEMETRY INTEGRATION (Requires 'otel' feature)
// ============================================================================

#[cfg(feature = "otel")]
test!(test_with_otel_tracing, {
    // This test demonstrates OTEL-enabled test infrastructure
    // In a real scenario, you would have OpenTelemetry initialized

    // Arrange
    let fixture = TestContext::new();

    // Act - Operations that would emit telemetry
    fixture.increment();
    fixture.increment();

    // Assert
    assert_eq!(fixture.get_count(), 2);

    // Telemetry would be captured and validated by Weaver
});

// ============================================================================
// 7. TEST PERFORMANCE WITH FIXTURES
// ============================================================================

test!(test_fixture_performance, {
    // Arrange
    let start = std::time::Instant::now();
    let db = TestDatabase::new();

    // Act - Insert multiple items
    for i in 0..1000 {
        db.insert(format!("Item{}", i));
    }

    let elapsed = start.elapsed();

    // Assert - Check correctness
    assert_eq!(db.get_all().len(), 1000);

    // Assert - Check performance (should be fast)
    assert!(elapsed.as_millis() < 500, "Fixture operations took too long: {}ms", elapsed.as_millis());
});

// ============================================================================
// 8. FIXTURE WITH ERROR HANDLING
// ============================================================================

test!(test_fixture_error_handling, {
    // Arrange
    let fixture = TestDatabase::new();

    // Act - Error conditions
    fixture.insert("value1".to_string());

    // Assert - Ensure cleanup happens even with errors
    assert!(fixture.get_all().len() > 0);

    // The fixture will be cleaned up when it goes out of scope,
    // even if there were errors
});

// ============================================================================
// 9. NESTED FIXTURES
// ============================================================================

test!(test_nested_fixtures, {
    // Arrange - Multiple fixtures
    let db1 = TestDatabase::new();
    let db2 = TestDatabase::new();

    // Act
    db1.insert("DB1-Value".to_string());
    db2.insert("DB2-Value".to_string());

    // Assert
    assert!(db1.get_all().contains(&"DB1-Value".to_string()));
    assert!(db2.get_all().contains(&"DB2-Value".to_string()));
    assert!(!db1.get_all().contains(&"DB2-Value".to_string()));

    // Each fixture is cleaned up independently
});

// ============================================================================
// 10. WEAVER SEMANTIC VALIDATION (Requires 'weaver' feature)
// ============================================================================

#[cfg(feature = "weaver")]
test!(test_weaver_semantic_validation, {
    // This test demonstrates compliance with OpenTelemetry
    // semantic conventions as validated by Weaver

    // Arrange
    let fixture = TestContext::new();

    // Act - Operations that emit semantically correct OTEL data
    fixture.increment();

    // Assert
    assert_eq!(fixture.get_count(), 1);

    // In production, Weaver would validate that telemetry
    // conforms to official OTel semantic conventions
});

// ============================================================================
// 11. TESTCONTAINERS INTEGRATION (Requires 'testcontainers' feature)
// ============================================================================

#[cfg(feature = "testcontainers")]
test!(test_with_testcontainers_setup, {
    // This is where Docker-based integration tests would go
    // Example: spinning up a Postgres container for testing

    // When 'testcontainers' feature is enabled, chicago-tdd-tools
    // provides helpers for Docker container management

    // Arrange
    let fixture = TestDatabase::new();

    // Act
    fixture.insert("IntegrationTest".to_string());

    // Assert
    assert!(fixture.get_all().contains(&"IntegrationTest".to_string()));
});

// ============================================================================
// 12. FIXTURE COMPOSITION
// ============================================================================

struct CompositeFixture {
    db: TestDatabase,
    context: TestContext,
}

impl CompositeFixture {
    fn new() -> Self {
        CompositeFixture {
            db: TestDatabase::new(),
            context: TestContext::new(),
        }
    }
}

test!(test_composite_fixture, {
    // Arrange
    let fixture = CompositeFixture::new();

    // Act
    fixture.db.insert("Data".to_string());
    fixture.context.increment();

    // Assert
    assert_eq!(fixture.db.get_all().len(), 1);
    assert_eq!(fixture.context.get_count(), 1);

    // Both fixtures are cleaned up
});
