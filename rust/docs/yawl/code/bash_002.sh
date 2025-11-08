# Run all tests
cargo test --features native

# Run hooks engine tests
cargo test --features native hooks_native::tests

# Run error validation tests
cargo test --features native hooks_native::tests::test_error

# Run benchmarks
cargo bench --features native