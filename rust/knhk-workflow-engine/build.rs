fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TRIZ Principle 2 (Taking Out): Make protobuf compilation OPTIONAL
    // Only attempt protobuf compilation if grpc feature is enabled

    #[cfg(feature = "grpc")]
    {
        println!("cargo:warning=Building with gRPC support (requires protoc)");
        match tonic_prost_build::compile_protos("proto/workflow_engine.proto") {
            Ok(_) => {
                println!("cargo:warning=Protobuf compilation successful");
            }
            Err(e) => {
                println!("cargo:warning=Protobuf compilation failed: {}", e);
                println!("cargo:warning=gRPC features may not be available");
                println!("cargo:warning=Workflow execution will still work via HTTP/REST");
                // Non-fatal - grpc is optional
            }
        }
    }

    #[cfg(not(feature = "grpc"))]
    {
        println!("cargo:warning=gRPC support disabled (feature flag not set)");
        println!("cargo:warning=Using HTTP/REST (JTBD accomplishable)");
    }

    Ok(())
}
