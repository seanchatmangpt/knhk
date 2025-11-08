// knhk-hot v1.0 — Content Addressing via BLAKE3
// Provides content-addressed hashing (≤1 tick performance target)
//
// Uses the official blake3 Rust crate (reference implementation)
// - SIMD-optimized (AVX2, AVX-512, NEON)
// - Zero-cost abstraction (no FFI overhead)
// - Constant-time operations for security

use std::fmt;

/// Content ID (256-bit BLAKE3 hash with metadata)
/// Compatible with ByteCore bs_cid_t structure (40 bytes, 8-byte aligned)
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct ContentId {
    /// BLAKE3 hash bytes (256-bit)
    pub bytes: [u8; 32],
    /// Magic number for validation (0x43494442 = "CIDB")
    magic: u32,
    /// Flags (BS_CID_FLAG_VALID | BS_CID_FLAG_COMPUTED)
    flags: u16,
    /// Reserved for alignment
    reserved: u16,
}

// Constants matching ByteCore ABI
const BS_CID_MAGIC: u32 = 0x43494442; // "CIDB"
const BS_CID_FLAG_VALID: u16 = 0x01;
const BS_CID_FLAG_COMPUTED: u16 = 0x02;

impl ContentId {
    /// Create a new, uninitialized ContentId
    pub fn new() -> Self {
        Self {
            bytes: [0u8; 32],
            magic: BS_CID_MAGIC,
            flags: BS_CID_FLAG_VALID,
            reserved: 0,
        }
    }

    /// Compute content ID from arbitrary bytes using BLAKE3
    ///
    /// Performance: ≤1 tick for typical payloads (<64 bytes)
    ///             ≤1000 cycles for larger payloads
    ///
    /// Uses SIMD acceleration when available (AVX2/AVX-512/NEON)
    ///
    /// # Examples
    ///
    /// ```
    /// use knhk_hot::ContentId;
    ///
    /// let data = b"hello world";
    /// let cid = ContentId::from_bytes(data);
    /// assert!(cid.is_valid());
    /// assert!(cid.is_computed());
    /// ```
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut cid = Self::new();

        // Compute BLAKE3 hash using optimized implementation
        let hash = blake3::hash(data);
        cid.bytes.copy_from_slice(hash.as_bytes());
        cid.flags |= BS_CID_FLAG_COMPUTED;

        cid
    }

    /// Check if ContentId structure is valid (magic number check)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.magic == BS_CID_MAGIC && (self.flags & BS_CID_FLAG_VALID) != 0
    }

    /// Check if hash has been computed
    #[inline]
    pub fn is_computed(&self) -> bool {
        (self.flags & BS_CID_FLAG_COMPUTED) != 0
    }

    /// Get the 256-bit hash as a byte slice
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Get a truncated 128-bit hash (first 16 bytes)
    /// Useful for space-constrained scenarios
    #[inline]
    pub fn as_bytes_128(&self) -> [u8; 16] {
        let mut short = [0u8; 16];
        short.copy_from_slice(&self.bytes[..16]);
        short
    }

    /// Constant-time equality check (prevents timing attacks)
    pub fn constant_time_eq(&self, other: &Self) -> bool {
        if !self.is_valid() || !other.is_valid() {
            return false;
        }

        // Use constant-time comparison to prevent timing side-channels
        use subtle::ConstantTimeEq;
        self.bytes.ct_eq(&other.bytes).into()
    }

    /// Convert to hexadecimal string representation
    pub fn to_hex(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl Default for ContentId {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ContentId {
    fn eq(&self, other: &Self) -> bool {
        self.constant_time_eq(other)
    }
}

impl Eq for ContentId {}

impl fmt::Debug for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentId")
            .field("hash", &format!("{}...", &self.to_hex()[..16]))
            .field("valid", &self.is_valid())
            .field("computed", &self.is_computed())
            .finish()
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Compute content hash for arbitrary data
///
/// Convenience function that returns just the 32-byte hash.
/// Use `ContentId::from_bytes()` if you need the full structure.
///
/// # Examples
///
/// ```
/// use knhk_hot::content_hash;
///
/// let hash = content_hash(b"hello world");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn content_hash(data: &[u8]) -> [u8; 32] {
    ContentId::from_bytes(data).bytes
}

/// Compute truncated 128-bit content hash
///
/// Useful when you need a shorter hash for space-constrained scenarios
/// while maintaining good collision resistance.
///
/// # Examples
///
/// ```
/// use knhk_hot::content_hash_128;
///
/// let hash = content_hash_128(b"hello world");
/// assert_eq!(hash.len(), 16);
/// ```
pub fn content_hash_128(data: &[u8]) -> [u8; 16] {
    ContentId::from_bytes(data).as_bytes_128()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_id_creation() {
        let cid = ContentId::new();
        assert!(cid.is_valid());
        assert!(!cid.is_computed());
        assert_eq!(cid.magic, BS_CID_MAGIC);
    }

    #[test]
    fn test_content_id_from_bytes() {
        let data = b"hello world";
        let cid = ContentId::from_bytes(data);

        assert!(cid.is_valid(), "ContentId should be valid");
        assert!(cid.is_computed(), "Hash should be computed");
        assert_ne!(cid.bytes, [0u8; 32], "Hash should not be all zeros");
    }

    #[test]
    fn test_content_id_deterministic() {
        let data = b"test data";
        let cid1 = ContentId::from_bytes(data);
        let cid2 = ContentId::from_bytes(data);

        assert_eq!(
            cid1.bytes, cid2.bytes,
            "Same input should produce same hash"
        );
        assert!(cid1.constant_time_eq(&cid2), "Constant-time eq should work");
    }

    #[test]
    fn test_content_id_different_inputs() {
        let cid1 = ContentId::from_bytes(b"input1");
        let cid2 = ContentId::from_bytes(b"input2");

        assert_ne!(
            cid1.bytes, cid2.bytes,
            "Different inputs should produce different hashes"
        );
        assert!(
            !cid1.constant_time_eq(&cid2),
            "Different hashes should not be equal"
        );
    }

    #[test]
    fn test_content_hash_convenience() {
        let data = b"test";
        let hash1 = content_hash(data);
        let cid = ContentId::from_bytes(data);

        assert_eq!(
            hash1, cid.bytes,
            "Convenience function should match ContentId"
        );
    }

    #[test]
    fn test_content_hash_128() {
        let data = b"truncated hash test";
        let hash_128 = content_hash_128(data);
        let cid = ContentId::from_bytes(data);

        assert_eq!(
            hash_128,
            cid.as_bytes_128(),
            "128-bit hash should be first 16 bytes"
        );
        assert_eq!(hash_128.len(), 16);
    }

    #[test]
    fn test_empty_input() {
        let cid = ContentId::from_bytes(&[]);
        assert!(cid.is_valid());
        assert!(cid.is_computed());
        // Empty input should still produce a valid hash (BLAKE3 of empty string)
        assert_ne!(cid.bytes, [0u8; 32]);
    }

    #[test]
    fn test_large_input() {
        let large_data = vec![0x42u8; 10_000];
        let cid = ContentId::from_bytes(&large_data);

        assert!(cid.is_valid());
        assert!(cid.is_computed());
        assert_ne!(cid.bytes, [0u8; 32]);
    }

    #[test]
    fn test_to_hex() {
        let cid = ContentId::from_bytes(b"hello");
        let hex = cid.to_hex();

        assert_eq!(
            hex.len(),
            64,
            "Hex string should be 64 chars (32 bytes * 2)"
        );
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "Should be valid hex"
        );
    }

    #[test]
    fn test_default() {
        let cid: ContentId = Default::default();
        assert!(cid.is_valid());
        assert!(!cid.is_computed());
    }

    #[test]
    fn test_equality_operators() {
        let data = b"equality test";
        let cid1 = ContentId::from_bytes(data);
        let cid2 = ContentId::from_bytes(data);
        let cid3 = ContentId::from_bytes(b"different");

        assert_eq!(cid1, cid2);
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_size_and_alignment() {
        use std::mem::{align_of, size_of};

        assert_eq!(size_of::<ContentId>(), 40, "ContentId must be 40 bytes");
        assert_eq!(
            align_of::<ContentId>(),
            8,
            "ContentId must be 8-byte aligned"
        );
    }

    #[test]
    fn test_debug_display() {
        let cid = ContentId::from_bytes(b"debug test");
        let debug_str = format!("{:?}", cid);
        let display_str = format!("{}", cid);

        assert!(debug_str.contains("ContentId"));
        assert!(debug_str.contains("valid"));
        assert_eq!(display_str.len(), 64, "Display should show full hex");
    }
}
