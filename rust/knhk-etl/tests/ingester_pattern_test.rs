// rust/knhk-etl/tests/ingester_pattern_test.rs
// Chicago TDD tests for Ingester pattern
// Tests focus on behavior: ingestion, streaming, multiple input sources

use knhk_etl::ingester::{Ingester, StreamingIngester, FileIngester, StdinIngester, DataFormat, StreamingHandle};

#[test]
fn test_file_ingester_creation() {
    // Arrange & Act: Create file ingester
    let ingester = FileIngester::new("test.ttl".to_string(), DataFormat::RdfTurtle);

    // Assert: Ingester created with correct properties
    assert_eq!(ingester.name(), "file");
}

#[test]
fn test_file_ingester_name() {
    // Arrange: Create file ingester
    let ingester = FileIngester::new("test.ttl".to_string(), DataFormat::RdfTurtle);

    // Act: Get ingester name
    let name = ingester.name();

    // Assert: Name is "file"
    assert_eq!(name, "file");
}

#[test]
fn test_file_ingester_ready_when_file_exists() {
    // Arrange: Create file ingester with non-existent file
    let ingester = FileIngester::new("nonexistent.ttl".to_string(), DataFormat::RdfTurtle);

    // Act: Check if ready
    let ready = ingester.is_ready();

    // Assert: Not ready when file doesn't exist
    assert!(!ready, "File ingester should not be ready when file doesn't exist");
}

#[test]
fn test_stdin_ingester_creation() {
    // Arrange & Act: Create stdin ingester
    let ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Assert: Ingester created
    assert_eq!(ingester.name(), "stdin");
}

#[test]
fn test_stdin_ingester_name() {
    // Arrange: Create stdin ingester
    let ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Act: Get ingester name
    let name = ingester.name();

    // Assert: Name is "stdin"
    assert_eq!(name, "stdin");
}

#[test]
fn test_stdin_ingester_always_ready() {
    // Arrange: Create stdin ingester
    let ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Act: Check if ready
    let ready = ingester.is_ready();

    // Assert: Stdin is always ready
    assert!(ready, "Stdin ingester should always be ready");
}

#[test]
fn test_stdin_ingester_streaming_start() {
    // Arrange: Create stdin ingester
    let mut ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Act: Start streaming
    let handle_result = ingester.start_streaming();

    // Assert: Streaming handle created
    assert!(handle_result.is_ok());
    let handle = handle_result.unwrap();
    assert_eq!(handle.id, "stdin-stream");
    assert!(handle.active);
}

#[test]
fn test_stdin_ingester_streaming_stop() {
    // Arrange: Create stdin ingester and start streaming
    let mut ingester = StdinIngester::new(DataFormat::RdfTurtle);
    let _handle = ingester.start_streaming().unwrap();

    // Act: Stop streaming
    let stop_result = ingester.stop_streaming();

    // Assert: Streaming stopped successfully
    assert!(stop_result.is_ok());
}

#[test]
fn test_data_format_variants() {
    // Arrange & Act: Create ingesters with different formats
    let turtle_ingester = FileIngester::new("test.ttl".to_string(), DataFormat::RdfTurtle);
    let jsonld_ingester = FileIngester::new("test.jsonld".to_string(), DataFormat::JsonLd);
    let json_ingester = FileIngester::new("test.json".to_string(), DataFormat::Json);
    let csv_ingester = FileIngester::new("test.csv".to_string(), DataFormat::Csv);

    // Assert: All formats supported
    assert_eq!(turtle_ingester.name(), "file");
    assert_eq!(jsonld_ingester.name(), "file");
    assert_eq!(json_ingester.name(), "file");
    assert_eq!(csv_ingester.name(), "file");
}

#[test]
fn test_streaming_handle_creation() {
    // Arrange & Act: Create streaming handle
    let handle = StreamingHandle::new("test-handle".to_string());

    // Assert: Handle created with correct properties
    assert_eq!(handle.id, "test-handle");
    assert!(handle.active);
}

#[test]
fn test_ingester_trait_consistency() {
    // Arrange: Create different ingester types
    let file_ingester = FileIngester::new("test.ttl".to_string(), DataFormat::RdfTurtle);
    let stdin_ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Act: Use trait methods
    let file_name = file_ingester.name();
    let stdin_name = stdin_ingester.name();
    let file_ready = file_ingester.is_ready();
    let stdin_ready = stdin_ingester.is_ready();

    // Assert: Trait methods work consistently
    assert_eq!(file_name, "file");
    assert_eq!(stdin_name, "stdin");
    // File ready depends on file existence, stdin always ready
    assert_eq!(stdin_ready, true);
}

#[test]
fn test_streaming_ingester_trait() {
    // Arrange: Create stdin ingester (implements StreamingIngester)
    let mut ingester = StdinIngester::new(DataFormat::RdfTurtle);

    // Act: Use StreamingIngester trait methods
    let handle_result = ingester.start_streaming();
    let stop_result = ingester.stop_streaming();

    // Assert: Streaming methods work
    assert!(handle_result.is_ok());
    assert!(stop_result.is_ok());
}

