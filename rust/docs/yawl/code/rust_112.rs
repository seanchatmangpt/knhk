match pattern.execute(input) {
    Ok(results) => {
        // Process results
    }
    Err(PatternError::ValidationFailed(msg)) => {
        // Ingress validation failed
    }
    Err(PatternError::ExecutionFailed(msg)) => {
        // Runtime execution error
    }
    Err(PatternError::TooManyBranches) => {
        // Branch limit exceeded
    }
    Err(PatternError::InvalidConfiguration(msg)) => {
        // Invalid configuration
    }
}