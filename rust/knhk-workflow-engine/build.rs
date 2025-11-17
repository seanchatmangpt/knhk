fn main() {
    // Compile proto files if tonic-build is available
    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .out_dir("src/api")
            .compile(&["proto/workflow_engine.proto"], &["proto"])
            .unwrap_or_else(|e| {
                println!("cargo:warning=Failed to compile proto files: {}", e);
                println!("cargo:warning=gRPC support may not work correctly");
            });
    }

    #[cfg(not(feature = "grpc"))]
    {
        println!("cargo:warning=knhk-workflow-engine: Using HTTP/REST (no protoc needed)");
        println!("cargo:warning=gRPC support skipped - enable 'grpc' feature to build proto files");
    }
}
