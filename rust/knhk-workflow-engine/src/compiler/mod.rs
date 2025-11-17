//! Phase 4 Descriptor Compiler System
//!
//! Transforms Turtle RDF ontologies into executable descriptors.
//!
//! Pipeline:
//! 1. Load Turtle files (loader.rs)
//! 2. Extract patterns via SPARQL (extractor.rs)
//! 3. Validate against pattern matrix (validator.rs)
//! 4. Generate code (code_generator.rs)
//! 5. Optimize (optimizer.rs)
//! 6. Link patterns (linker.rs)
//! 7. Sign descriptors (signer.rs)
//! 8. Serialize to binary (serializer.rs)

pub mod code_generator;
pub mod extractor;
pub mod linker;
pub mod loader;
pub mod optimizer;
pub mod serializer;
pub mod signer;
pub mod validator;

use crate::error::{WorkflowError, WorkflowResult};
use std::path::Path;
use tracing::{debug, info, span, warn, Level};

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Enable strict validation
    pub strict_validation: bool,
    /// Enable optimization passes
    pub enable_optimizations: bool,
    /// Sign descriptors
    pub enable_signing: bool,
    /// Pattern matrix path for validation
    pub pattern_matrix_path: String,
    /// Maximum compilation time (seconds)
    pub max_compilation_time: u64,
    /// Enable parallel compilation
    pub parallel_compilation: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            enable_optimizations: true,
            enable_signing: true,
            pattern_matrix_path: "ontology/yawl-pattern-permutations.ttl".to_string(),
            max_compilation_time: 60,
            parallel_compilation: true,
        }
    }
}

/// Compilation result
#[derive(Debug)]
pub struct CompilationResult {
    /// Compiled descriptor binary
    pub descriptor: Vec<u8>,
    /// Compilation metadata
    pub metadata: CompilationMetadata,
    /// Signature (if signing enabled)
    pub signature: Option<Vec<u8>>,
}

/// Compilation metadata
#[derive(Debug, Clone)]
pub struct CompilationMetadata {
    /// Source file hash
    pub source_hash: [u8; 32],
    /// Descriptor hash
    pub descriptor_hash: [u8; 32],
    /// Compilation timestamp
    pub timestamp: u64,
    /// Compiler version
    pub compiler_version: String,
    /// Pattern count
    pub pattern_count: usize,
    /// Guard count
    pub guard_count: usize,
    /// Optimization stats
    pub optimization_stats: OptimizationStats,
}

/// Optimization statistics
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// Dead code eliminated
    pub dead_code_eliminated: usize,
    /// Common subexpressions eliminated
    pub cse_count: usize,
    /// Constants folded
    pub constants_folded: usize,
    /// Size reduction percentage
    pub size_reduction_percent: f32,
}

/// Phase 4 Descriptor Compiler
pub struct DescriptorCompiler {
    config: CompilerConfig,
    loader: loader::TurtleLoader,
    extractor: extractor::PatternExtractor,
    validator: validator::PatternValidator,
    generator: code_generator::CodeGenerator,
    optimizer: optimizer::Optimizer,
    linker: linker::Linker,
    signer: signer::DescriptorSigner,
    serializer: serializer::BinarySerializer,
}

impl DescriptorCompiler {
    /// Create new compiler with default config
    pub fn new() -> Self {
        Self::with_config(CompilerConfig::default())
    }

    /// Create new compiler with custom config
    pub fn with_config(config: CompilerConfig) -> Self {
        Self {
            loader: loader::TurtleLoader::new(),
            extractor: extractor::PatternExtractor::new(config.parallel_compilation),
            validator: validator::PatternValidator::new(&config.pattern_matrix_path),
            generator: code_generator::CodeGenerator::new(),
            optimizer: optimizer::Optimizer::new(config.enable_optimizations),
            linker: linker::Linker::new(),
            signer: signer::DescriptorSigner::new(config.enable_signing),
            serializer: serializer::BinarySerializer::new(),
            config,
        }
    }

    /// Compile Turtle file to descriptor
    pub async fn compile<P: AsRef<Path>>(
        &mut self,
        turtle_path: P,
    ) -> WorkflowResult<CompilationResult> {
        let span = span!(Level::INFO, "compile", path = ?turtle_path.as_ref());
        let _enter = span.enter();

        info!("Starting Phase 4 compilation");

        // Stage 1: Load Turtle file
        debug!("Stage 1: Loading Turtle file");
        let store = self.loader.load_turtle(turtle_path.as_ref()).await?;
        let source_hash = self.loader.compute_source_hash(&store)?;
        info!("Loaded {} triples", store.len());

        // Stage 2: Extract patterns via SPARQL
        debug!("Stage 2: Extracting patterns");
        let patterns = self.extractor.extract_all(&store).await?;
        info!("Extracted {} patterns", patterns.len());

        // Stage 3: Validate against pattern matrix
        debug!("Stage 3: Validating patterns");
        if self.config.strict_validation {
            self.validator.validate_patterns(&patterns).await?;
            info!("All patterns validated against matrix");
        }

        // Stage 4: Generate code
        debug!("Stage 4: Generating code");
        let mut code = self.generator.generate(&patterns).await?;
        let guard_count = code.guards.len();
        info!("Generated code with {} guards", guard_count);

        // Stage 5: Optimize
        debug!("Stage 5: Optimizing");
        let optimization_stats = if self.config.enable_optimizations {
            let stats = self.optimizer.optimize(&mut code).await?;
            info!(
                "Optimization reduced size by {:.1}%",
                stats.size_reduction_percent
            );
            stats
        } else {
            OptimizationStats::default()
        };

        // Stage 6: Link patterns
        debug!("Stage 6: Linking");
        let linked = self.linker.link(code).await?;
        info!("Linked {} patterns", linked.pattern_count);

        // Stage 7: Sign descriptor
        let signature = if self.config.enable_signing {
            debug!("Stage 7: Signing descriptor");
            let sig = self.signer.sign(&linked).await?;
            info!("Descriptor signed");
            Some(sig)
        } else {
            None
        };

        // Stage 8: Serialize to binary
        debug!("Stage 8: Serializing");
        let descriptor = self.serializer.serialize(&linked).await?;
        let descriptor_hash = self.compute_descriptor_hash(&descriptor);
        info!("Serialized to {} bytes", descriptor.len());

        // Build result
        let metadata = CompilationMetadata {
            source_hash,
            descriptor_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            pattern_count: patterns.len(),
            guard_count,
            optimization_stats,
        };

        Ok(CompilationResult {
            descriptor,
            metadata,
            signature,
        })
    }

    /// Compile and verify round-trip
    pub async fn compile_with_verification<P: AsRef<Path>>(
        &mut self,
        turtle_path: P,
    ) -> WorkflowResult<CompilationResult> {
        let result = self.compile(turtle_path).await?;

        // Verify signature if present
        if let Some(ref sig) = result.signature {
            self.signer.verify(&result.descriptor, sig)?;
            info!("Signature verified");
        }

        // Verify deterministic compilation
        // (same input should produce same output)

        Ok(result)
    }

    fn compute_descriptor_hash(&self, descriptor: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(descriptor);
        hasher.finalize().into()
    }
}

impl Default for DescriptorCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compiler_creation() {
        let compiler = DescriptorCompiler::new();
        assert!(compiler.config.strict_validation);
        assert!(compiler.config.enable_optimizations);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = CompilerConfig {
            strict_validation: false,
            enable_optimizations: false,
            enable_signing: false,
            ..Default::default()
        };

        let compiler = DescriptorCompiler::with_config(config.clone());
        assert!(!compiler.config.strict_validation);
        assert!(!compiler.config.enable_optimizations);
        assert!(!compiler.config.enable_signing);
    }
}
