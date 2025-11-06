fn main() {
    // Link to KNHK C library
    // Path is relative to rust/knhk-hot/build.rs
    // Library is located at c/libknhk.a from repository root
    println!("cargo:rustc-link-search=native=../../c");
    println!("cargo:rustc-link-lib=static=knhk");
    println!("cargo:rustc-link-lib=c");
    
    // Rerun if library changes
    println!("cargo:rerun-if-changed=../../c/libknhk.a");
}

