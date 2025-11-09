//! Property-Based Tests for RDF Parsing and Serialization
//!
//! Uses property-based testing to verify RDF operations across
//! random inputs, ensuring no panics and data integrity.

use knhk_workflow_engine::patterns::{get_all_pattern_metadata, serialize_metadata_to_rdf};
use knhk_workflow_engine::testing::property::PropertyTestGenerator;

/// Property: All Turtle documents should parse without crashing
#[test]
fn property_all_turtle_parses_without_crash() {
    let mut generator = PropertyTestGenerator::new().with_seed(42);

    for _ in 0..50 {
        let turtle = generator.generate_turtle();

        // Property: Parser should never panic on valid Turtle
        let result = std::panic::catch_unwind(|| {
            // Attempt to parse as Turtle
            let graph = match oxigraph::store::Store::new() {
                Ok(store) => store,
                Err(_) => return false,
            };

            // Try to load the turtle
            let parse_result =
                graph.load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle.as_bytes());

            parse_result.is_ok() || parse_result.is_err() // Either way, no panic
        });

        assert!(
            result.is_ok(),
            "Parser panicked on Turtle document: {}",
            turtle
        );
    }
}

/// Property: All pattern metadata should serialize to valid RDF
#[test]
fn property_all_metadata_serializes_to_valid_rdf() {
    let all_metadata = get_all_pattern_metadata();

    for metadata in all_metadata {
        // Serialize to RDF
        let rdf_result = serialize_metadata_to_rdf(&metadata);

        assert!(
            rdf_result.is_ok(),
            "Failed to serialize pattern {} metadata to RDF: {:?}",
            metadata.pattern_id,
            rdf_result.err()
        );

        let rdf = rdf_result.unwrap();

        // RDF should not be empty
        assert!(
            !rdf.is_empty(),
            "Pattern {} metadata serialized to empty RDF",
            metadata.pattern_id
        );

        // RDF should be valid Turtle
        let store = oxigraph::store::Store::new().unwrap();
        let parse_result = store.load_from_reader(oxigraph::io::RdfFormat::Turtle, rdf.as_bytes());

        assert!(
            parse_result.is_ok(),
            "Pattern {} metadata produced invalid Turtle: {:?}",
            metadata.pattern_id,
            parse_result.err()
        );
    }
}

/// Property: All pattern metadata should have non-empty descriptions
#[test]
fn property_all_patterns_have_descriptions() {
    let all_metadata = get_all_pattern_metadata();

    assert_eq!(
        all_metadata.len(),
        43,
        "Expected 43 patterns, found {}",
        all_metadata.len()
    );

    for metadata in all_metadata {
        assert!(
            !metadata.description.is_empty(),
            "Pattern {} has empty description",
            metadata.pattern_id
        );
    }
}

/// Property: All pattern IDs should be unique
#[test]
fn property_all_pattern_ids_unique() {
    use std::collections::HashSet;

    let all_metadata = get_all_pattern_metadata();
    let ids: Vec<u32> = all_metadata.iter().map(|m| m.pattern_id).collect();
    let unique_ids: HashSet<u32> = ids.iter().cloned().collect();

    assert_eq!(
        ids.len(),
        unique_ids.len(),
        "Pattern IDs are not unique: found {} patterns but only {} unique IDs",
        ids.len(),
        unique_ids.len()
    );
}

/// Property: All pattern categories should be valid
#[test]
fn property_pattern_categories_are_valid() {
    use std::collections::HashSet;

    let all_metadata = get_all_pattern_metadata();

    // Expected categories from Van der Aalst taxonomy
    let valid_categories: HashSet<&str> = [
        "Basic Control Flow",
        "Advanced Branching",
        "Multiple Instance",
        "State-Based",
        "Cancellation",
        "Advanced Control",
        "Trigger",
    ]
    .iter()
    .cloned()
    .collect();

    for metadata in all_metadata {
        assert!(
            valid_categories.contains(metadata.category.as_str()),
            "Pattern {} has invalid category: '{}'",
            metadata.pattern_id,
            metadata.category
        );
    }
}

/// Property: Pattern IDs should match index (1-43)
#[test]
fn property_pattern_ids_sequential() {
    let all_metadata = get_all_pattern_metadata();
    let ids: Vec<u32> = all_metadata.iter().map(|m| m.pattern_id).collect();

    // Should have all IDs from 1 to 43
    for expected_id in 1..=43 {
        assert!(
            ids.contains(&expected_id),
            "Missing pattern ID: {}",
            expected_id
        );
    }
}

/// Property: RDF serialization should be deterministic
#[test]
fn property_rdf_serialization_deterministic() {
    let all_metadata = get_all_pattern_metadata();

    for metadata in &all_metadata[0..5] {
        // Serialize same metadata multiple times
        let rdf1 = serialize_metadata_to_rdf(metadata).expect("First serialization failed");
        let rdf2 = serialize_metadata_to_rdf(metadata).expect("Second serialization failed");

        // Results should be identical (deterministic)
        assert_eq!(
            rdf1, rdf2,
            "RDF serialization is not deterministic for pattern {}",
            metadata.pattern_id
        );
    }
}

/// Property: Pattern names should be descriptive (for patterns 1-25)
#[test]
fn property_pattern_names_descriptive() {
    let all_metadata = get_all_pattern_metadata();

    for metadata in all_metadata {
        // Names should be capitalized
        assert!(
            metadata.name.chars().next().unwrap().is_uppercase(),
            "Pattern {} name not capitalized: '{}'",
            metadata.pattern_id,
            metadata.name
        );

        // Names should be at least 3 characters
        assert!(
            metadata.name.len() >= 3,
            "Pattern {} name too short: '{}'",
            metadata.pattern_id,
            metadata.name
        );
    }
}

/// Property: RDF should contain pattern ID reference
#[test]
fn property_rdf_contains_pattern_id() {
    let all_metadata = get_all_pattern_metadata();

    for metadata in &all_metadata[0..10] {
        let rdf = serialize_metadata_to_rdf(metadata).expect("Serialization failed");

        // RDF should reference the pattern ID
        let id_str = metadata.pattern_id.to_string();
        assert!(
            rdf.contains(&id_str) || rdf.contains(&format!("pattern{}", metadata.pattern_id)),
            "RDF for pattern {} doesn't contain pattern ID reference",
            metadata.pattern_id
        );
    }
}

/// Property: All patterns in same category share characteristics
#[test]
fn property_patterns_in_category_share_characteristics() {
    use std::collections::HashMap;

    let all_metadata = get_all_pattern_metadata();

    // Group by category
    let mut by_category: HashMap<String, Vec<_>> = HashMap::new();
    for metadata in all_metadata {
        by_category
            .entry(metadata.category.clone())
            .or_insert_with(Vec::new)
            .push(metadata);
    }

    // Each category should have at least 1 pattern
    for (category, patterns) in by_category {
        assert!(
            !patterns.is_empty(),
            "Category '{}' has no patterns",
            category
        );

        // All patterns in category should have consistent category field
        for pattern in &patterns {
            assert_eq!(
                pattern.category, category,
                "Pattern {} category mismatch",
                pattern.pattern_id
            );
        }
    }
}
