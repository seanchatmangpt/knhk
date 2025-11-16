# Mock Implementation Guide for KNHK London School TDD

## Overview

This guide provides comprehensive patterns for implementing mocks in KNHK following London School TDD principles.

## Core Principles

### 1. Mock External Dependencies Only

**Rule**: Only mock dependencies you don't own.

**✅ DO Mock**:
- `git2::Repository` (external library)
- `sled::Db` (external library)
- `std::fs::File` (I/O)
- `reqwest::Client` (HTTP)
- `tokio::time::Instant` (time)

**❌ DON'T Mock**:
- Your domain logic
- Pure functions
- Simple data structures
- Value objects

### 2. Track All Interactions

Every mock must track:
- **What** methods were called
- **How many times** they were called
- **What arguments** they received
- **In what order** they were called

```rust
struct MockDatabase {
    // State
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,

    // Interaction tracking
    insert_calls: Arc<Mutex<Vec<(Vec<u8>, Vec<u8>)>>>,  // (key, value)
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,                // (key)
    flush_calls: Arc<Mutex<usize>>,                     // count

    // Call ordering
    call_sequence: Arc<Mutex<Vec<String>>>,
}
```

### 3. Provide Verification Methods

```rust
impl MockDatabase {
    // Verify specific calls
    fn verify_insert_called(&self, key: &[u8]) -> bool {
        self.insert_calls.lock().unwrap()
            .iter()
            .any(|(k, _)| k == key)
    }

    fn verify_insert_called_with(&self, key: &[u8], value: &[u8]) -> bool {
        self.insert_calls.lock().unwrap()
            .iter()
            .any(|(k, v)| k == key && v == value)
    }

    // Verify call counts
    fn insert_call_count(&self) -> usize {
        self.insert_calls.lock().unwrap().len()
    }

    fn get_call_count(&self) -> usize {
        self.get_calls.lock().unwrap().len()
    }

    // Verify call order
    fn verify_call_order(&self, expected: &[&str]) -> bool {
        let sequence = self.call_sequence.lock().unwrap();
        sequence.iter()
            .map(|s| s.as_str())
            .eq(expected.iter().copied())
    }
}
```

## Mock Patterns

### Pattern 1: Trait-Based Mocking

For external libraries, define a trait that abstracts the dependency:

```rust
// Trait abstraction
trait Database: Send + Sync {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String>;
    fn flush(&self) -> Result<(), String>;
}

// Production implementation
struct SledDatabase {
    db: sled::Db,
}

impl Database for SledDatabase {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String> {
        self.db.insert(key, value)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.db.get(key)
            .map(|opt| opt.map(|iv| iv.to_vec()))
            .map_err(|e| e.to_string())
    }

    fn flush(&self) -> Result<(), String> {
        self.db.flush()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

// Mock implementation
struct MockDatabase {
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
    insert_calls: Arc<Mutex<Vec<(Vec<u8>, Vec<u8>)>>>,
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    flush_calls: Arc<Mutex<usize>>,
}

impl Database for MockDatabase {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String> {
        self.insert_calls.lock().unwrap().push((key.to_vec(), value.clone()));
        self.data.lock().unwrap().insert(key.to_vec(), value);
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.get_calls.lock().unwrap().push(key.to_vec());
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    fn flush(&self) -> Result<(), String> {
        *self.flush_calls.lock().unwrap() += 1;
        Ok(())
    }
}
```

### Pattern 2: Configurable Failure Mocking

Mocks should support failure injection for error path testing:

```rust
struct MockDatabase {
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,

    // Failure injection
    insert_should_fail: Arc<Mutex<bool>>,
    get_should_fail: Arc<Mutex<bool>>,
    flush_should_fail: Arc<Mutex<bool>>,

    // Interaction tracking
    insert_calls: Arc<Mutex<Vec<(Vec<u8>, Vec<u8>)>>>,
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    flush_calls: Arc<Mutex<usize>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(BTreeMap::new())),
            insert_should_fail: Arc::new(Mutex::new(false)),
            get_should_fail: Arc::new(Mutex::new(false)),
            flush_should_fail: Arc::new(Mutex::new(false)),
            insert_calls: Arc::new(Mutex::new(Vec::new())),
            get_calls: Arc::new(Mutex::new(Vec::new())),
            flush_calls: Arc::new(Mutex::new(0)),
        }
    }

    // Configure failure
    fn set_insert_failure(&self, should_fail: bool) {
        *self.insert_should_fail.lock().unwrap() = should_fail;
    }

    fn set_get_failure(&self, should_fail: bool) {
        *self.get_should_fail.lock().unwrap() = should_fail;
    }
}

impl Database for MockDatabase {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String> {
        self.insert_calls.lock().unwrap().push((key.to_vec(), value.clone()));

        if *self.insert_should_fail.lock().unwrap() {
            return Err("Injected insert failure".to_string());
        }

        self.data.lock().unwrap().insert(key.to_vec(), value);
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.get_calls.lock().unwrap().push(key.to_vec());

        if *self.get_should_fail.lock().unwrap() {
            return Err("Injected get failure".to_string());
        }

        Ok(self.data.lock().unwrap().get(key).cloned())
    }
}
```

**Usage**:
```rust
#[test]
fn test_handles_insert_failure() {
    let mock_db = Arc::new(MockDatabase::new());
    mock_db.set_insert_failure(true);

    let storage = Storage::new(mock_db.clone());
    let result = storage.persist_root(cycle, root, proof);

    assert!(result.is_err(), "Should propagate insert error");
    assert_eq!(mock_db.insert_call_count(), 1, "Should have attempted insert");
}
```

### Pattern 3: Spy Pattern (Record and Replay)

For complex interaction verification:

```rust
#[derive(Debug, Clone, PartialEq)]
enum DatabaseCall {
    Insert { key: Vec<u8>, value: Vec<u8> },
    Get { key: Vec<u8> },
    Flush,
    Range { start: Vec<u8>, end: Vec<u8> },
}

struct SpyDatabase {
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,

    // Record all calls in order
    calls: Arc<Mutex<Vec<DatabaseCall>>>,
}

impl SpyDatabase {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(BTreeMap::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Verify exact call sequence
    fn verify_calls(&self, expected: &[DatabaseCall]) -> bool {
        let calls = self.calls.lock().unwrap();
        calls.len() == expected.len() &&
            calls.iter().zip(expected).all(|(a, b)| a == b)
    }

    // Verify call pattern
    fn verify_call_pattern(&self, pattern: &[DatabaseCall]) -> bool {
        let calls = self.calls.lock().unwrap();
        calls.windows(pattern.len())
            .any(|window| window == pattern)
    }
}

impl Database for SpyDatabase {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String> {
        self.calls.lock().unwrap().push(DatabaseCall::Insert {
            key: key.to_vec(),
            value: value.clone(),
        });
        self.data.lock().unwrap().insert(key.to_vec(), value);
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.calls.lock().unwrap().push(DatabaseCall::Get {
            key: key.to_vec(),
        });
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    fn flush(&self) -> Result<(), String> {
        self.calls.lock().unwrap().push(DatabaseCall::Flush);
        Ok(())
    }
}
```

**Usage**:
```rust
#[test]
fn test_persist_root_call_sequence() {
    let spy_db = Arc::new(SpyDatabase::new());
    let storage = Storage::new(spy_db.clone());

    storage.persist_root(cycle, root, proof).unwrap();

    // Verify exact call sequence
    let expected = vec![
        DatabaseCall::Insert { key: format!("root:{:020}", cycle).into_bytes(), value: serialized },
        DatabaseCall::Flush,
    ];

    assert!(spy_db.verify_calls(&expected), "Call sequence incorrect");
}
```

### Pattern 4: Stub Pattern (Pre-configured Responses)

For testing with pre-determined return values:

```rust
struct StubDatabase {
    // Pre-configured responses
    get_responses: Arc<Mutex<HashMap<Vec<u8>, Option<Vec<u8>>>>>,
    insert_results: Arc<Mutex<Vec<Result<(), String>>>>,

    // Interaction tracking
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    insert_calls: Arc<Mutex<Vec<(Vec<u8>, Vec<u8>)>>>,
}

impl StubDatabase {
    fn new() -> Self {
        Self {
            get_responses: Arc::new(Mutex::new(HashMap::new())),
            insert_results: Arc::new(Mutex::new(Vec::new())),
            get_calls: Arc::new(Mutex::new(Vec::new())),
            insert_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Configure stub responses
    fn stub_get(&self, key: Vec<u8>, response: Option<Vec<u8>>) {
        self.get_responses.lock().unwrap().insert(key, response);
    }

    fn stub_insert_success(&self) {
        self.insert_results.lock().unwrap().push(Ok(()));
    }

    fn stub_insert_failure(&self, error: String) {
        self.insert_results.lock().unwrap().push(Err(error));
    }
}

impl Database for StubDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        self.get_calls.lock().unwrap().push(key.to_vec());

        Ok(self.get_responses.lock().unwrap()
            .get(key)
            .cloned()
            .unwrap_or(None))
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String> {
        self.insert_calls.lock().unwrap().push((key.to_vec(), value));

        self.insert_results.lock().unwrap()
            .pop()
            .unwrap_or(Ok(()))
    }
}
```

**Usage**:
```rust
#[test]
fn test_handles_missing_root() {
    let stub_db = Arc::new(StubDatabase::new());

    // Stub get to return None
    let key = format!("root:{:020}", 999).into_bytes();
    stub_db.stub_get(key, None);

    let storage = Storage::new(stub_db.clone());
    let result = storage.get_root(999).unwrap();

    assert!(result.is_none(), "Should return None for missing root");
    assert_eq!(stub_db.get_call_count(), 1, "Should have called get once");
}
```

## Git Repository Mocking

### Pattern: Mock git2::Repository

```rust
trait GitRepository: Send + Sync {
    fn commit(
        &self,
        update_ref: Option<&str>,
        author: &Signature,
        committer: &Signature,
        message: &str,
        tree: &Tree,
        parents: &[&Commit],
    ) -> Result<Oid, String>;

    fn create_blob(&self, content: &[u8]) -> Result<Oid, String>;
    fn get_index(&self) -> Result<Index, String>;
}

struct MockGitRepository {
    commits: Arc<Mutex<Vec<MockCommit>>>,
    blobs: Arc<Mutex<Vec<(Oid, Vec<u8>)>>>,
    commit_calls: Arc<Mutex<Vec<MockCommitCall>>>,
    blob_calls: Arc<Mutex<Vec<Vec<u8>>>>,
}

#[derive(Debug, Clone)]
struct MockCommit {
    id: Oid,
    message: String,
    tree_id: Oid,
    parent_ids: Vec<Oid>,
}

#[derive(Debug, Clone)]
struct MockCommitCall {
    message: String,
    tree_id: Oid,
}

impl MockGitRepository {
    fn new() -> Self {
        Self {
            commits: Arc::new(Mutex::new(Vec::new())),
            blobs: Arc::new(Mutex::new(Vec::new())),
            commit_calls: Arc::new(Mutex::new(Vec::new())),
            blob_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn verify_commit_created(&self, message_contains: &str) -> bool {
        self.commits.lock().unwrap()
            .iter()
            .any(|c| c.message.contains(message_contains))
    }

    fn verify_blob_created(&self, content: &[u8]) -> bool {
        self.blobs.lock().unwrap()
            .iter()
            .any(|(_, blob_content)| blob_content == content)
    }

    fn commit_count(&self) -> usize {
        self.commits.lock().unwrap().len()
    }
}

impl GitRepository for MockGitRepository {
    fn commit(
        &self,
        _update_ref: Option<&str>,
        _author: &Signature,
        _committer: &Signature,
        message: &str,
        tree: &Tree,
        _parents: &[&Commit],
    ) -> Result<Oid, String> {
        let id = Oid::from_bytes(&[0u8; 20]).unwrap();

        self.commit_calls.lock().unwrap().push(MockCommitCall {
            message: message.to_string(),
            tree_id: tree.id(),
        });

        self.commits.lock().unwrap().push(MockCommit {
            id,
            message: message.to_string(),
            tree_id: tree.id(),
            parent_ids: Vec::new(),
        });

        Ok(id)
    }

    fn create_blob(&self, content: &[u8]) -> Result<Oid, String> {
        self.blob_calls.lock().unwrap().push(content.to_vec());

        let id = Oid::from_bytes(&[0u8; 20]).unwrap();
        self.blobs.lock().unwrap().push((id, content.to_vec()));

        Ok(id)
    }
}
```

## Testing Patterns with Mocks

### Pattern 1: Verify Collaboration

```rust
#[test]
fn test_storage_collaborates_with_database_correctly() {
    // Arrange
    let mock_db = Arc::new(MockDatabase::new());
    let storage = Storage::new(mock_db.clone());

    let cycle = 42u64;
    let root = [0x42u8; 32];
    let proof = create_test_proof(cycle, root);

    // Act
    storage.persist_root(cycle, root, proof).unwrap();

    // Assert: Verify collaboration
    let expected_key = format!("root:{:020}", cycle).into_bytes();
    assert!(mock_db.verify_insert_called(&expected_key), "Should call insert with correct key");
    assert!(mock_db.verify_flush_called(), "Should call flush after insert");
    assert_eq!(mock_db.insert_call_count(), 1, "Should call insert exactly once");
}
```

### Pattern 2: Verify Error Handling

```rust
#[test]
fn test_storage_handles_database_error() {
    // Arrange
    let mock_db = Arc::new(MockDatabase::new());
    mock_db.set_insert_failure(true);

    let storage = Storage::new(mock_db.clone());
    let cycle = 42u64;
    let root = [0x42u8; 32];
    let proof = create_test_proof(cycle, root);

    // Act
    let result = storage.persist_root(cycle, root, proof);

    // Assert: Error is propagated
    assert!(result.is_err(), "Should return error when database fails");
    assert!(mock_db.verify_insert_called(&format!("root:{:020}", cycle).into_bytes()),
            "Should have attempted insert");
    assert_eq!(mock_db.flush_call_count(), 0, "Should not call flush after insert failure");
}
```

### Pattern 3: Verify Call Ordering

```rust
#[test]
fn test_storage_calls_database_in_correct_order() {
    // Arrange
    let spy_db = Arc::new(SpyDatabase::new());
    let storage = Storage::new(spy_db.clone());

    // Act
    storage.persist_root(cycle, root, proof).unwrap();

    // Assert: Verify call order
    let expected_order = vec![
        DatabaseCall::Insert { ... },
        DatabaseCall::Flush,
    ];

    assert!(spy_db.verify_calls(&expected_order), "Calls should be in correct order");
}
```

## Summary

**Key Takeaways**:

1. **Mock External Dependencies Only**: Only mock what you don't own
2. **Track All Interactions**: Record what, how many, with what args, in what order
3. **Provide Verification Methods**: Make it easy to assert on interactions
4. **Support Failure Injection**: Test error paths with configurable failures
5. **Use Appropriate Pattern**: Mock/Stub/Spy based on test needs

**Mock Types**:
- **Mock**: Records interactions for verification
- **Stub**: Pre-configured responses
- **Spy**: Records and replays calls
- **Fake**: Simple in-memory implementation

**When to Use Each**:
- **Mock**: Verify HOW component collaborates
- **Stub**: Control WHAT component receives
- **Spy**: Verify EXACT call sequence
- **Fake**: Simple alternative to real implementation (for integration tests)

**Result**: Comprehensive behavior verification without false positives.
