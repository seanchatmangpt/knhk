fn main() {
    // Link to KNHK C library for CLI binary
    println!("cargo:rustc-link-search=native=../../../c");
    println!("cargo:rustc-link-lib=static=knhk");
}

