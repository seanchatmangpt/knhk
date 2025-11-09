// knhk-sidecar: Build script for gRPC proto compilation

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/kgc-sidecar.proto")?;
    Ok(())
}
