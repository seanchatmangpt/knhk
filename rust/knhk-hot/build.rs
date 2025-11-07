fn main() {
    // Link to KNHK C library
    // Use absolute path from CARGO_MANIFEST_DIR for reliability
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_lib_dir = format!("{}/../../c", manifest_dir);

    println!("cargo:rustc-link-search=native={}", c_lib_dir);
    println!("cargo:rustc-link-lib=static=knhk");
    println!("cargo:rustc-link-lib=c");

    // Rerun if library changes
    println!("cargo:rerun-if-changed={}/libknhk.a", c_lib_dir);
}

