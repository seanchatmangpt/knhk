# Build with native features (Rust-native RDF)
cargo build --features native --release

# Build with unrdf integration (JavaScript)
cargo build --features unrdf --release

# Build everything
cargo build --features native,unrdf --release