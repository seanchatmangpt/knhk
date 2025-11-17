// knhk-sidecar: Build script for gRPC proto compilation
// TRIZ Principle 2: Make protoc compilation optional

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "grpc")]
    {
        println!("cargo:warning=Building with gRPC support (requires protoc)");
        match tonic_prost_build::compile_protos("proto/kgc-sidecar.proto") {
            Ok(_) => {
                println!("cargo:warning=Proto compilation successful");
            }
            Err(e) => {
                println!("cargo:warning=Proto compilation failed: {}", e);
                println!("cargo:warning=gRPC features may not be available");
                println!("cargo:warning=Sidecar will function over HTTP/REST instead");
            }
        }
    }

    #[cfg(not(feature = "grpc"))]
    {
        println!("cargo:warning=gRPC support disabled (feature flag not set)");
        println!("cargo:warning=Using HTTP/REST (no protoc needed)");
    }

    Ok(())
}
