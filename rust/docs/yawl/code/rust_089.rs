pub enum PatternError {
    ValidationFailed(String),      // Ingress validation failed
    ExecutionFailed(String),       // Runtime execution error
    TooManyBranches,               // Branch limit exceeded (max 1024)
    InvalidConfiguration(String),  // Invalid pattern configuration
}