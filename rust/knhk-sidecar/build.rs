// knhk-sidecar: Build script
// 80/20 Principle: Skip gRPC/protobuf - pure HTTP/REST only

fn main() {
    println!("cargo:warning=knhk-sidecar: Using HTTP/REST (no protoc needed)");
    println!("cargo:warning=gRPC support skipped for pragmatic build simplification");
}
