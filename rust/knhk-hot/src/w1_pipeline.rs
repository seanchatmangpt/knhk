// W1 JSON Pipeline - Two-Stage Architecture (simdjson-style)
// Stage 1: Structural index with SIMD classifiers
// Stage 2: Tape building with field dictionary
//
// Target: ≤200 ns AVX2, ≤120 ns AVX-512 for 256B ATM JSON

use std::arch::aarch64::*;

// ============================================================================
// ShapeCard: Field Dictionary + Present Mask
// ============================================================================

/// Field dictionary for ATM-shape JSON (perfect hash, 6-16 keys)
/// Fits in one 64-byte cache line
#[repr(C, align(64))]
pub struct ShapeCard {
    /// Field names (interned, 8-byte max)
    pub field_names: [u64; 16],
    /// Field present mask (1 bit per field)
    pub present_mask: u16,
    /// Value spans: (offset, len) for each field
    pub value_spans: [(u32, u16); 16],
    /// UTF-8 validation result
    pub utf8_ok: bool,
    /// Shape hash (for fast equality check)
    pub shape_hash: u64,
}

impl Default for ShapeCard {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeCard {
    pub fn new() -> Self {
        Self {
            field_names: [0; 16],
            present_mask: 0,
            value_spans: [(0, 0); 16],
            utf8_ok: true,
            shape_hash: 0,
        }
    }

    /// Check if field is present
    #[inline(always)]
    pub fn has_field(&self, field_idx: u8) -> bool {
        debug_assert!(field_idx < 16);
        (self.present_mask & (1 << field_idx)) != 0
    }

    /// Get value span for field
    #[inline(always)]
    pub fn get_value_span(&self, field_idx: u8) -> Option<(u32, u16)> {
        if self.has_field(field_idx) {
            Some(self.value_spans[field_idx as usize])
        } else {
            None
        }
    }
}

// ============================================================================
// Stage 1: Structural Index (SIMD Classifiers)
// ============================================================================

/// Stage 1 result: structural character positions
pub struct StructuralIndex {
    /// Positions of structural characters: { } [ ] , :
    pub structural_chars: Vec<u32>,
    /// In-string mask (1 bit per byte, 1 = inside string)
    pub in_string_mask: Vec<u64>,
    /// Quote positions
    pub quote_positions: Vec<u32>,
    /// Escape positions (for proper quote pairing)
    pub escape_positions: Vec<u32>,
}

impl Default for StructuralIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuralIndex {
    pub fn new() -> Self {
        Self {
            structural_chars: Vec::with_capacity(256),
            in_string_mask: Vec::with_capacity(16),
            quote_positions: Vec::with_capacity(32),
            escape_positions: Vec::with_capacity(16),
        }
    }
}

/// Stage 1: Find structural characters with SIMD
///
/// Target: ≤1.2 cycles/byte AVX2, ≤0.9 AVX-512
///
/// # Safety
/// This function uses unsafe SIMD operations. The caller must ensure:
/// - `json` is a valid UTF-8 byte slice
/// - `index` is a valid mutable reference
#[cfg(target_arch = "aarch64")]
pub unsafe fn stage1_structural_index(json: &[u8], index: &mut StructuralIndex) {
    index.structural_chars.clear();
    index.in_string_mask.clear();
    index.quote_positions.clear();
    index.escape_positions.clear();

    let len = json.len();
    let chunks = len / 16;

    // NEON 128-bit processing (16 bytes per iteration)
    let _structural_chars_v = vdupq_n_u8(b'{');
    let open_brace = vdupq_n_u8(b'{');
    let close_brace = vdupq_n_u8(b'}');
    let open_bracket = vdupq_n_u8(b'[');
    let close_bracket = vdupq_n_u8(b']');
    let comma = vdupq_n_u8(b',');
    let colon = vdupq_n_u8(b':');
    let quote = vdupq_n_u8(b'"');
    let backslash = vdupq_n_u8(b'\\');

    let mut in_string = false;
    let mut prev_was_escape = false;

    for chunk_idx in 0..chunks {
        let offset = chunk_idx * 16;
        let chunk_ptr = json.as_ptr().add(offset);
        let chunk = vld1q_u8(chunk_ptr);

        // Find structural characters
        let is_open_brace = vceqq_u8(chunk, open_brace);
        let is_close_brace = vceqq_u8(chunk, close_brace);
        let is_open_bracket = vceqq_u8(chunk, open_bracket);
        let is_close_bracket = vceqq_u8(chunk, close_bracket);
        let is_comma = vceqq_u8(chunk, comma);
        let is_colon = vceqq_u8(chunk, colon);
        let is_quote = vceqq_u8(chunk, quote);
        let _is_escape = vceqq_u8(chunk, backslash);

        // Combine all structural characters
        let structural = vorrq_u8(
            vorrq_u8(
                vorrq_u8(is_open_brace, is_close_brace),
                vorrq_u8(is_open_bracket, is_close_bracket),
            ),
            vorrq_u8(vorrq_u8(is_comma, is_colon), is_quote),
        );

        // Extract positions using ARM bit reversal + leading zeros
        // (simdjson ARM-specific optimization)
        let _structural_mask = vget_lane_u64(vreinterpret_u64_u8(vget_low_u8(structural)), 0);
        let _ = vget_lane_u64(vreinterpret_u64_u8(vget_high_u8(structural)), 0) << 8;

        // TODO: Use bit reversal + leading zeros for ARM (instead of trailing zeros)
        // This is the simdjson ARM-specific optimization

        // For now, scalar fallback for quote tracking
        for i in 0..16 {
            let byte = json[offset + i];
            let is_quote_byte = byte == b'"';
            let is_escape_byte = byte == b'\\';

            if is_quote_byte && !prev_was_escape {
                in_string = !in_string;
                index.quote_positions.push((offset + i) as u32);
            }

            if is_escape_byte {
                index.escape_positions.push((offset + i) as u32);
            }

            prev_was_escape = is_escape_byte && !prev_was_escape;

            // Record structural chars outside strings
            if !in_string
                && (byte == b'{'
                    || byte == b'}'
                    || byte == b'['
                    || byte == b']'
                    || byte == b','
                    || byte == b':')
            {
                index.structural_chars.push((offset + i) as u32);
            }
        }
    }

    // Handle remaining bytes (scalar)
    #[allow(clippy::needless_range_loop)]
    for i in (chunks * 16)..len {
        let byte = json[i];
        if byte == b'"' && !prev_was_escape {
            in_string = !in_string;
            index.quote_positions.push(i as u32);
        }

        if byte == b'\\' {
            index.escape_positions.push(i as u32);
        }

        prev_was_escape = byte == b'\\' && !prev_was_escape;

        if !in_string
            && (byte == b'{'
                || byte == b'}'
                || byte == b'['
                || byte == b']'
                || byte == b','
                || byte == b':')
        {
            index.structural_chars.push(i as u32);
        }
    }
}

/// Generic fallback for non-ARM platforms
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn stage1_structural_index(json: &[u8], index: &mut StructuralIndex) {
    index.structural_chars.clear();
    index.quote_positions.clear();
    index.escape_positions.clear();

    let mut in_string = false;
    let mut prev_was_escape = false;

    for (i, &byte) in json.iter().enumerate() {
        if byte == b'"' && !prev_was_escape {
            in_string = !in_string;
            index.quote_positions.push(i as u32);
        }

        if byte == b'\\' {
            index.escape_positions.push(i as u32);
        }

        prev_was_escape = byte == b'\\' && !prev_was_escape;

        if !in_string && matches!(byte, b'{' | b'}' | b'[' | b']' | b',' | b':') {
            index.structural_chars.push(i as u32);
        }
    }
}

// ============================================================================
// Stage 2: Tape Building + Field Dictionary
// ============================================================================

/// Tape token (64-bit word)
///
/// Format: [kind:8 | aux:8 | payload/offset:48]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TapeToken {
    /// Token kind (0=object, 1=array, 2=string, 3=number, 4=bool, 5=null)
    pub kind: u8,
    /// Auxiliary data (string length, array size, etc.)
    pub aux: u8,
    /// Payload or offset into arena
    pub payload: u64, // Bottom 48 bits used
}

/// Stage 2 result: tape + field dictionary
pub struct TapeBuilder {
    /// Tape of 64-bit tokens
    pub tape: Vec<TapeToken>,
    /// String arena (for strings longer than 6 bytes)
    pub arena: Vec<u8>,
    /// Field dictionary for this document
    pub shape: ShapeCard,
}

impl Default for TapeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeBuilder {
    pub fn new() -> Self {
        Self {
            tape: Vec::with_capacity(256),
            arena: Vec::with_capacity(1024),
            shape: ShapeCard::new(),
        }
    }

    /// Stage 2: Build tape from structural index
    ///
    /// Target: ≤0.6 cycles/byte AVX2, ≤0.4 AVX-512
    pub fn build_tape(&mut self, _json: &[u8], _index: &StructuralIndex) {
        self.tape.clear();
        self.arena.clear();
        self.shape.present_mask = 0;

        // TODO: Implement actual tape building
        // For now, placeholder
    }
}

// ============================================================================
// SoA Packer: Convert Tape to μ Runs
// ============================================================================

/// SoA run (subject, predicate, object) with len ≤ 8
#[repr(C)]
pub struct SoARun {
    pub subjects: [u64; 8],
    pub predicates: [u64; 8],
    pub objects: [u64; 8],
    pub len: u8,
}

impl Default for SoARun {
    fn default() -> Self {
        Self::new()
    }
}

impl SoARun {
    pub fn new() -> Self {
        Self {
            subjects: [0; 8],
            predicates: [0; 8],
            objects: [0; 8],
            len: 0,
        }
    }
}

/// SoA Packer: Convert tape to predicate-grouped runs
pub struct SoAPacker {
    pub runs: Vec<SoARun>,
}

impl Default for SoAPacker {
    fn default() -> Self {
        Self::new()
    }
}

impl SoAPacker {
    pub fn new() -> Self {
        Self {
            runs: Vec::with_capacity(16),
        }
    }

    /// Pack tape tokens into SoA runs grouped by predicate
    ///
    /// Target: ≤40 ns/object
    pub fn pack_from_tape(&mut self, _tape: &[TapeToken], _shape: &ShapeCard) {
        self.runs.clear();

        // TODO: Implement SoA packing
        // Map dictionary indices to predicate IDs
        // Emit (s,p,o) with inline small ints, offsets for strings
        // Group into runs of ≤8
    }
}

// ============================================================================
// Shape-Locked Fast Path (ATM JSON ≤128-256B)
// ============================================================================

/// Shape-locked kernel for fixed ATM JSON format
///
/// Target: ≤64–90 ns AVX-512, ≤120 ns AVX2 for small payloads
pub struct ATMShapeKernel {
    /// Hardcoded key literals for memcmp
    pub expected_keys: [&'static [u8]; 6],
    /// Expected field order
    pub field_order: [u8; 6],
}

impl Default for ATMShapeKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl ATMShapeKernel {
    pub fn new() -> Self {
        Self {
            expected_keys: [
                b"account_id",
                b"transaction_id",
                b"amount",
                b"currency",
                b"timestamp",
                b"status",
            ],
            field_order: [0, 1, 2, 3, 4, 5],
        }
    }

    /// Fast path for shape-locked ATM JSON
    ///
    /// Uses hardcoded key checks with memcmp64 or AVX2 pcmpeqb
    ///
    /// # Safety
    /// This function uses unsafe SIMD operations. The caller must ensure:
    /// - `json` is a valid UTF-8 byte slice
    /// - `runs` is a valid mutable vector
    pub unsafe fn parse_shape_locked(&self, _json: &[u8], _runs: &mut Vec<SoARun>) -> bool {
        // TODO: Implement shape-locked fast path
        // - Hardcode key literals
        // - Compare with two memcmp64 or AVX2 pcmpeqb + pmovmskb
        // - Number fast path with table checks; no FP parse
        // - Fuse S1+S2 for this shape
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_card() {
        let mut card = ShapeCard::new();
        card.present_mask = 0b000111; // Fields 0, 1, 2 present

        assert!(card.has_field(0));
        assert!(card.has_field(1));
        assert!(card.has_field(2));
        assert!(!card.has_field(3));
    }

    #[test]
    fn test_stage1_simple_json() {
        let json = br#"{"key":"value"}"#;
        let mut index = StructuralIndex::new();

        unsafe {
            stage1_structural_index(json, &mut index);
        }

        // Should find: { } , :
        assert!(index.structural_chars.len() >= 3);
        assert!(index.quote_positions.len() >= 2);
    }
}
