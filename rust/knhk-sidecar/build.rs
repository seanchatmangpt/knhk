// knhk-sidecar: Build script for gRPC proto compilation

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["proto/kgc-sidecar.proto"], &["proto"])?;
    Ok(())
}
