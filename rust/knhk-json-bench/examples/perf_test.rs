//! Simple one-time performance test for knhk JSON parsing

use knhk_json_bench::parse_json;
use std::time::Instant;

fn main() {
    let test_cases = vec![
        ("simple", r#"{"key": "value", "number": 42}"#),
        (
            "nested",
            r#"{"array": [1, 2, 3, 4, 5], "object": {"nested": true}}"#,
        ),
        (
            "twitter",
            r#"{
    "statuses": [
        {"text": "First tweet", "user": {"screen_name": "user1"}, "retweet_count": 10},
        {"text": "Second tweet", "user": {"screen_name": "user2"}, "retweet_count": 20}
    ]
}"#,
        ),
    ];

    println!("Performance Test - Single Run\n");
    println!(
        "{:<15} {:<12} {:<12} {:<12}",
        "Test Case", "Size (bytes)", "Time (ns)", "Throughput"
    );
    println!("{}", "-".repeat(60));

    for (name, json) in test_cases {
        let bytes = json.as_bytes();
        let size = bytes.len();

        let start = Instant::now();
        let result = parse_json(bytes.to_vec());
        let elapsed = start.elapsed();

        let nanos = elapsed.as_nanos();
        let throughput_mbps = if nanos > 0 {
            (size as f64 * 1000.0) / nanos as f64
        } else {
            0.0
        };

        match result {
            Ok(_) => {
                println!(
                    "{:<15} {:<12} {:<12} {:<12.2} MB/s",
                    name, size, nanos, throughput_mbps
                );
            }
            Err(e) => {
                println!("{:<15} {:<12} ERROR: {}", name, size, e);
            }
        }
    }
}
