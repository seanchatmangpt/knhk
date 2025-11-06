// rust/knhk-etl/tests/chicago_tdd_ingester.rs
// Chicago TDD tests for Ingester Pattern
// Tests behaviors, not implementation details

use knhk_etl::ingester::*;
use knhk_etl::error::PipelineError;
use alloc::collections::BTreeMap;

#[test]
fn test_file_ingester_ingests_data_from_file() {
    // Arrange: Create a temporary test file
    #[cfg(feature = "std")]
    {
        use std::fs;
        use std::io::Write;
        
        let test_file = "/tmp/knhk_test_ingester.txt";
        let test_content = b"test data content";
        
        // Create test file
        let mut file = fs::File::create(test_file).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test data");
        drop(file);
        
        // Act: Ingest from file
        let mut ingester = FileIngester::new(test_file.to_string());
        let result = ingester.ingest();
        
        // Assert: Data ingested successfully
        assert!(result.is_ok(), "File ingester should succeed");
        let ingested = result.unwrap();
        assert_eq!(ingested.data, test_content, "Ingested data should match file content");
        assert_eq!(ingested.source, test_file, "Source should be file path");
        assert_eq!(ingested.metadata.get("source_type"), Some(&"file".to_string()));
        
        // Cleanup
        let _ = fs::remove_file(test_file);
    }
}

#[test]
fn test_file_ingester_with_format_hint() {
    // Arrange: Create ingester with format hint
    #[cfg(feature = "std")]
    {
        use std::fs;
        use std::io::Write;
        
        let test_file = "/tmp/knhk_test_format.ttl";
        let test_content = b"@prefix ex: <http://example.org/> .";
        
        let mut file = fs::File::create(test_file).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test data");
        drop(file);
        
        // Act: Ingest with format hint
        let mut ingester = FileIngester::new(test_file.to_string())
            .with_format("turtle".to_string());
        let result = ingester.ingest();
        
        // Assert: Format hint preserved
        assert!(result.is_ok());
        let ingested = result.unwrap();
        assert_eq!(ingested.format_hint, Some("turtle".to_string()));
        
        // Cleanup
        let _ = fs::remove_file(test_file);
    }
}

#[test]
fn test_file_ingester_returns_error_for_nonexistent_file() {
    // Arrange: Create ingester for non-existent file
    let mut ingester = FileIngester::new("/nonexistent/file/path.txt".to_string());
    
    // Act: Attempt to ingest
    let result = ingester.ingest();
    
    // Assert: Should return error
    assert!(result.is_err(), "Should return error for non-existent file");
    match result.unwrap_err() {
        PipelineError::IngestError(_) => {},
        _ => panic!("Should return IngestError"),
    }
}

#[test]
fn test_file_ingester_source_returns_path() {
    // Arrange: Create file ingester
    let ingester = FileIngester::new("test/path.txt".to_string());
    
    // Act: Get source
    let source = ingester.source();
    
    // Assert: Source matches path
    assert_eq!(source, "test/path.txt");
}

#[test]
fn test_file_ingester_does_not_support_streaming() {
    // Arrange: Create file ingester
    let ingester = FileIngester::new("test.txt".to_string());
    
    // Act: Check streaming support
    let supports = ingester.supports_streaming();
    
    // Assert: File ingester doesn't support streaming
    assert!(!supports, "File ingester should not support streaming");
}

#[test]
fn test_memory_ingester_ingests_provided_data() {
    // Arrange: Create memory ingester with test data
    let test_data = b"memory test data".to_vec();
    let mut ingester = MemoryIngester::new(test_data.clone(), "memory_source".to_string());
    
    // Act: Ingest data
    let result = ingester.ingest();
    
    // Assert: Data ingested correctly
    assert!(result.is_ok());
    let ingested = result.unwrap();
    assert_eq!(ingested.data, test_data);
    assert_eq!(ingested.source, "memory_source");
    assert_eq!(ingested.metadata.get("source_type"), Some(&"memory".to_string()));
}

#[test]
fn test_memory_ingester_with_format_hint() {
    // Arrange: Create memory ingester with format
    let test_data = b"{}".to_vec();
    let mut ingester = MemoryIngester::new(test_data, "json_source".to_string())
        .with_format("json".to_string());
    
    // Act: Ingest
    let result = ingester.ingest();
    
    // Assert: Format hint preserved
    assert!(result.is_ok());
    let ingested = result.unwrap();
    assert_eq!(ingested.format_hint, Some("json".to_string()));
}

#[test]
fn test_memory_ingester_source_returns_provided_source() {
    // Arrange: Create memory ingester
    let ingester = MemoryIngester::new(vec![1, 2, 3], "custom_source".to_string());
    
    // Act: Get source
    let source = ingester.source();
    
    // Assert: Source matches
    assert_eq!(source, "custom_source");
}

#[test]
fn test_multi_ingester_combines_multiple_sources() {
    // Arrange: Create multiple ingesters
    let mut multi = MultiIngester::new();
    
    let data1 = b"data1".to_vec();
    let data2 = b"data2".to_vec();
    
    multi.add_ingester(Box::new(MemoryIngester::new(data1.clone(), "source1".to_string())));
    multi.add_ingester(Box::new(MemoryIngester::new(data2.clone(), "source2".to_string())));
    
    // Act: Ingest all
    let result = multi.ingest_all();
    
    // Assert: All sources ingested
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].data, data1);
    assert_eq!(results[1].data, data2);
}

#[test]
fn test_multi_ingester_handles_empty_list() {
    // Arrange: Create empty multi-ingester
    let mut multi = MultiIngester::new();
    
    // Act: Ingest all
    let result = multi.ingest_all();
    
    // Assert: Returns empty vector
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_multi_ingester_propagates_errors() {
    // Arrange: Create multi-ingester with failing ingester
    #[cfg(feature = "std")]
    {
        let mut multi = MultiIngester::new();
        
        // Add ingester that will fail
        multi.add_ingester(Box::new(FileIngester::new("/nonexistent/file.txt".to_string())));
        
        // Act: Ingest all
        let result = multi.ingest_all();
        
        // Assert: Error propagated
        assert!(result.is_err());
    }
}

#[test]
fn test_stdin_ingester_supports_streaming() {
    // Arrange: Create stdin ingester
    let ingester = StdinIngester::new();
    
    // Act: Check streaming support
    let supports = ingester.supports_streaming();
    
    // Assert: Stdin ingester supports streaming
    assert!(supports, "Stdin ingester should support streaming");
}

#[test]
fn test_stdin_ingester_source_returns_stdin() {
    // Arrange: Create stdin ingester
    let ingester = StdinIngester::new();
    
    // Act: Get source
    let source = ingester.source();
    
    // Assert: Source is stdin
    assert_eq!(source, "stdin");
}

#[test]
fn test_ingester_trait_consistency() {
    // Arrange: Create different ingester types
    let file_ingester: Box<dyn Ingester> = Box::new(FileIngester::new("test.txt".to_string()));
    let memory_ingester: Box<dyn Ingester> = Box::new(MemoryIngester::new(vec![1, 2, 3], "mem".to_string()));
    
    // Act: Use trait methods
    let file_source = file_ingester.source();
    let memory_source = memory_ingester.source();
    
    // Assert: Both implement trait correctly
    assert_eq!(file_source, "test.txt");
    assert_eq!(memory_source, "mem");
}

#[test]
fn test_ingested_data_contains_metadata() {
    // Arrange: Create memory ingester
    let mut ingester = MemoryIngester::new(vec![1, 2, 3], "test_source".to_string());
    
    // Act: Ingest
    let result = ingester.ingest().unwrap();
    
    // Assert: Metadata present
    assert!(result.metadata.contains_key("source_type"));
    assert_eq!(result.metadata.get("source_type"), Some(&"memory".to_string()));
}

