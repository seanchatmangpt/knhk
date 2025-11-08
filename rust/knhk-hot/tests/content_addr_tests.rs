// Integration tests for content addressing

use knhk_hot::{content_hash, content_hash_128, ContentId};

#[test]
fn test_basic_content_addressing() {
    let data = b"integration test data";
    let cid = ContentId::from_bytes(data);

    assert!(cid.is_valid(), "ContentId should be valid");
    assert!(cid.is_computed(), "Hash should be computed");
}

#[test]
fn test_hash_consistency_across_calls() {
    let data = b"consistency test";

    let hash1 = content_hash(data);
    let hash2 = content_hash(data);

    assert_eq!(
        hash1, hash2,
        "Multiple calls with same data should produce same hash"
    );
}

#[test]
fn test_different_data_produces_different_hashes() {
    let hash1 = content_hash(b"data1");
    let hash2 = content_hash(b"data2");

    assert_ne!(
        hash1, hash2,
        "Different data should produce different hashes"
    );
}

#[test]
fn test_128bit_truncation() {
    let data = b"truncation test";
    let full_hash = content_hash(data);
    let truncated = content_hash_128(data);

    assert_eq!(
        &full_hash[..16],
        &truncated[..],
        "Truncated hash should match first 16 bytes"
    );
}

#[test]
fn test_empty_data() {
    let cid = ContentId::from_bytes(&[]);
    assert!(cid.is_valid());
    assert!(cid.is_computed());
}

#[test]
fn test_large_data() {
    let large = vec![0xAAu8; 100_000];
    let cid = ContentId::from_bytes(&large);

    assert!(cid.is_valid());
    assert!(cid.is_computed());
}

#[test]
fn test_content_id_equality() {
    let data = b"equality";
    let cid1 = ContentId::from_bytes(data);
    let cid2 = ContentId::from_bytes(data);

    assert_eq!(cid1, cid2);
    assert!(cid1.constant_time_eq(&cid2));
}

#[test]
fn test_hex_representation() {
    let cid = ContentId::from_bytes(b"hex test");
    let hex = cid.to_hex();

    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_known_vector_blake3() {
    // BLAKE3 of empty string is well-known
    let empty_hash = content_hash(&[]);

    // BLAKE3("") = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
    let expected = [
        0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6, 0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc, 0xc9,
        0x49, 0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7, 0xcc, 0x9a, 0x93, 0xca, 0xe4, 0x1f,
        0x32, 0x62,
    ];

    assert_eq!(
        empty_hash, expected,
        "Empty BLAKE3 hash should match known vector"
    );
}

#[test]
fn test_collision_resistance() {
    // Test that similar inputs produce very different hashes (avalanche effect)
    let hash1 = content_hash(b"test");
    let hash2 = content_hash(b"Test"); // Only case difference

    // Count differing bits
    let differing_bits = hash1
        .iter()
        .zip(hash2.iter())
        .map(|(a, b)| (a ^ b).count_ones())
        .sum::<u32>();

    // With good avalanche, expect ~50% of bits to differ (128 out of 256)
    assert!(
        differing_bits > 50,
        "Avalanche effect should change many bits (got {})",
        differing_bits
    );
}

#[test]
fn test_thread_safety() {
    use std::thread;

    let data = b"thread safety test";
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let data = data.to_vec();
            thread::spawn(move || ContentId::from_bytes(&data))
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All threads should produce the same hash
    for cid in &results[1..] {
        assert_eq!(cid.bytes, results[0].bytes);
    }
}
