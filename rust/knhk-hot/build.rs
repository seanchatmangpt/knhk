fn main() {
    // Compile workflow_patterns.c and ring_buffer.c
    cc::Build::new()
        .file("src/workflow_patterns.c")
        .file("src/ring_buffer.c")
        .opt_level(3)
        .flag("-march=native")
        .flag("-fno-strict-aliasing")
        .warnings(false) // Suppress unused parameter warnings
        .compile("workflow_patterns");

    // Try to link to KNHK C library if it exists (optional for workflow patterns)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_lib_dir = format!("{}/../../c", manifest_dir);
    let lib_path = format!("{}/libknhk.a", c_lib_dir);

    if std::path::Path::new(&lib_path).exists() {
        println!("cargo:rustc-link-search=native={}", c_lib_dir);
        println!("cargo:rustc-link-lib=static=knhk");
    } else {
        // libknhk.a not found - workflow_patterns will work standalone
        eprintln!("Note: libknhk.a not found at {}", lib_path);
        eprintln!("Workflow patterns will work, but other FFI functions may not link");
    }

    // Rerun if files change
    println!("cargo:rerun-if-changed=src/workflow_patterns.c");
    println!("cargo:rerun-if-changed=src/workflow_patterns.h");
    println!("cargo:rerun-if-changed=src/ring_buffer.c");
    println!("cargo:rerun-if-changed={}", lib_path);
}
