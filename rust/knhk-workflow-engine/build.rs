fn main() {
    // 80/20 Principle: Skip gRPC/protobuf compilation entirely
    println!("cargo:warning=knhk-workflow-engine: Using HTTP/REST (no protoc needed)");
    println!("cargo:warning=gRPC support skipped for pragmatic build simplification");
}
