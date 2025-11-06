// rust/knhk-etl/src/ingester_example.rs
// Example usage of the Ingester pattern

#[cfg(feature = "std")]
pub mod examples {
    use super::super::ingester::*;
    use super::super::error::PipelineError;
    
    /// Example: Ingest from file
    pub fn example_file_ingester() -> Result<(), PipelineError> {
        let mut ingester = FileIngester::new("data/example.ttl".to_string())
            .with_format("turtle".to_string());
        
        let data = ingester.ingest()?;
        println!("Ingested {} bytes from {}", data.data.len(), data.source);
        Ok(())
    }
    
    /// Example: Ingest from stdin
    pub fn example_stdin_ingester() -> Result<(), PipelineError> {
        let mut ingester = StdinIngester::new()
            .with_format("turtle".to_string());
        
        let data = ingester.ingest()?;
        println!("Ingested {} bytes from stdin", data.data.len());
        Ok(())
    }
    
    /// Example: Multi-ingester combining multiple sources
    pub fn example_multi_ingester() -> Result<(), PipelineError> {
        let mut multi = MultiIngester::new();
        
        multi.add_ingester(Box::new(FileIngester::new("data/file1.ttl".to_string())));
        multi.add_ingester(Box::new(FileIngester::new("data/file2.ttl".to_string())));
        
        let results = multi.ingest_all()?;
        println!("Ingested {} sources", results.len());
        Ok(())
    }
}

