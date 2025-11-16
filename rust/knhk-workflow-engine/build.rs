fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only compile protos if grpc feature is enabled
    #[cfg(feature = "grpc")]
    {
        tonic_prost_build::compile_protos("proto/workflow_engine.proto")?;
    }
    Ok(())
}
