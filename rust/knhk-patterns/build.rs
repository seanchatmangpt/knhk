fn main() {
    // Compile workflow_patterns.c directly in knhk-patterns
    // This avoids dependency on knhk-hot which requires libknhk.a
    cc::Build::new()
        .file("../knhk-hot/src/workflow_patterns.c")
        .include("../knhk-hot/src")
        .opt_level(3)
        .flag("-march=native")
        .flag("-fno-strict-aliasing")
        .warnings(false) // Suppress unused parameter warnings
        .compile("workflow_patterns");

    // Rerun if C files change
    println!("cargo:rerun-if-changed=../knhk-hot/src/workflow_patterns.c");
    println!("cargo:rerun-if-changed=../knhk-hot/src/workflow_patterns.h");
}
