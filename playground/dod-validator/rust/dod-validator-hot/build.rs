fn main() {
    // Link to KNHK C library
    // Path is relative to playground/dod-validator/rust/dod-validator-hot
    println!("cargo:rustc-link-search=native=../../../c");
    println!("cargo:rustc-link-lib=static=knhk");
    
    // Include KNHK headers
    println!("cargo:rerun-if-changed=../../../c/include/knhk.h");
    println!("cargo:rerun-if-changed=../../c/hot_validators.h");
}

