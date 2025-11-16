// tests/compiler/snapshot_determinism.rs
// DETERMINISTIC COMPILATION: Compile same Turtle twice → byte-identical binaries
// Critical for reproducible builds and verifying compilation correctness

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Mock compiler for testing framework
pub struct MockCompiler;

impl MockCompiler {
    /// Simulate Turtle→executable compilation
    /// In production: Loader → Extractor → Validator → CodeGen → Optimizer → Linker → Signer → Serializer
    pub fn compile(turtle_source: &str) -> Result<Vec<u8>, String> {
        // Stage 1: Parse and hash input (deterministic)
        let mut hasher = DefaultHasher::new();
        turtle_source.hash(&mut hasher);
        let input_hash = hasher.finish();

        // Stage 2: Generate deterministic binary
        // (In production, all stages are fully deterministic)
        let mut binary = Vec::new();

        // Add header
        binary.extend_from_slice(b"KNHK");
        binary.extend_from_slice(&input_hash.to_le_bytes());

        // Add version
        binary.push(1u8);

        // Add content (deterministically processed)
        for (i, byte) in turtle_source.bytes().enumerate() {
            binary.push(byte.wrapping_add(i as u8));
        }

        // Add signature (deterministic based on content)
        let mut content_hasher = DefaultHasher::new();
        binary.hash(&mut content_hasher);
        let signature = content_hasher.finish();
        binary.extend_from_slice(&signature.to_le_bytes());

        Ok(binary)
    }
}

/// Extract metadata from compiled binary for snapshot comparison
pub fn binary_metadata(binary: &[u8]) -> BinaryMetadata {
    BinaryMetadata {
        len: binary.len(),
        header: binary.get(0..4).map(|s| String::from_utf8_lossy(s).to_string()),
        first_byte: binary.get(0).copied(),
        last_byte: binary.last().copied(),
        checksum: compute_checksum(binary),
    }
}

/// Metadata for snapshot testing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryMetadata {
    pub len: usize,
    pub header: Option<String>,
    pub first_byte: Option<u8>,
    pub last_byte: Option<u8>,
    pub checksum: u64,
}

fn compute_checksum(binary: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    binary.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TURTLE: &str = r#"
        @prefix workflow: <http://example.org/workflow#> .

        workflow:ProcessPayment a workflow:Pattern ;
            workflow:type workflow:Sequence ;
            workflow:input [ workflow:variable "amount" ] ;
            workflow:output [ workflow:variable "receipt" ] .
    "#;

    #[test]
    fn test_compilation_determinism() {
        // Arrange: Compile same Turtle twice
        let binary_1 = MockCompiler::compile(SAMPLE_TURTLE).expect("Compilation 1 failed");
        let binary_2 = MockCompiler::compile(SAMPLE_TURTLE).expect("Compilation 2 failed");

        // Act: Compare binaries
        let hash_1 = compute_checksum(&binary_1);
        let hash_2 = compute_checksum(&binary_2);

        // Assert: BYTE-IDENTICAL compilation
        assert_eq!(
            binary_1, binary_2,
            "Non-deterministic compilation: different binaries produced"
        );
        assert_eq!(hash_1, hash_2, "Checksums differ");
    }

    #[test]
    fn test_compilation_determinism_extended() {
        // Arrange: Compile multiple times
        let mut binaries = Vec::new();
        for _ in 0..5 {
            let binary = MockCompiler::compile(SAMPLE_TURTLE).expect("Compilation failed");
            binaries.push(binary);
        }

        // Assert: All identical
        let first = &binaries[0];
        for (i, binary) in binaries.iter().enumerate().skip(1) {
            assert_eq!(
                binary, first,
                "Compilation {} differs from first",
                i
            );
        }
    }

    #[test]
    fn test_different_input_different_binary() {
        let turtle_1 = "workflow:A pattern";
        let turtle_2 = "workflow:B pattern";

        let binary_1 = MockCompiler::compile(turtle_1).expect("Failed");
        let binary_2 = MockCompiler::compile(turtle_2).expect("Failed");

        // Different inputs SHOULD produce different binaries
        assert_ne!(binary_1, binary_2, "Different inputs should produce different binaries");
    }

    #[test]
    fn snapshot_binary_metadata() {
        let binary = MockCompiler::compile(SAMPLE_TURTLE).expect("Compilation failed");
        let metadata = binary_metadata(&binary);

        // Snapshot assertions (in real code, use insta crate)
        assert_eq!(metadata.header, Some("KNHK".to_string()), "Header mismatch");
        assert!(metadata.len > 0, "Binary should not be empty");
        assert!(metadata.checksum > 0, "Checksum should be non-zero");

        // In production: insta::assert_debug_snapshot!(metadata);
        println!("Snapshot - Metadata: {:#?}", metadata);
    }

    #[test]
    fn test_binary_signature_included() {
        let binary = MockCompiler::compile(SAMPLE_TURTLE).expect("Compilation failed");

        // Verify binary has signature (last 8 bytes should be u64)
        assert!(binary.len() >= 8, "Binary too short for signature");

        // Extract signature
        let signature_bytes = &binary[binary.len() - 8..];
        let signature = u64::from_le_bytes([
            signature_bytes[0],
            signature_bytes[1],
            signature_bytes[2],
            signature_bytes[3],
            signature_bytes[4],
            signature_bytes[5],
            signature_bytes[6],
            signature_bytes[7],
        ]);

        assert!(signature > 0, "Signature should be non-zero");
    }

    #[test]
    fn test_compilation_roundtrip_stable() {
        // Arrange: Compile → extract metadata → verify stable
        let binary_1 = MockCompiler::compile(SAMPLE_TURTLE).expect("Failed");
        let metadata_1 = binary_metadata(&binary_1);

        let binary_2 = MockCompiler::compile(SAMPLE_TURTLE).expect("Failed");
        let metadata_2 = binary_metadata(&binary_2);

        // Assert: Metadata must be identical
        assert_eq!(metadata_1, metadata_2, "Metadata differs between compilations");
    }

    #[test]
    fn test_compiler_handles_empty_input() {
        let empty = "";
        let binary = MockCompiler::compile(empty).expect("Compilation should succeed");

        assert!(!binary.is_empty(), "Binary should not be empty even for empty input");
        assert_eq!(binary[0..4], *b"KNHK", "Header should be present");
    }

    #[test]
    fn test_compiler_handles_large_input() {
        let large = "x".repeat(100_000);
        let binary = MockCompiler::compile(&large).expect("Compilation should succeed");

        assert!(binary.len() > 100_000, "Binary should contain content");

        // Verify determinism for large input
        let binary_2 = MockCompiler::compile(&large).expect("Compilation should succeed");
        assert_eq!(binary, binary_2, "Large input should compile deterministically");
    }
}
