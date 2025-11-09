fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic-build 0.14 uses configure() -> Builder pattern
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/workflow_engine.proto"], &["proto"])?;
    Ok(())
}
