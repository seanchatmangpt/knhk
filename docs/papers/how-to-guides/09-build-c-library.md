# How-to Guide 9: Build the C Library

## Goal

Successfully compile KNHK's C library component using appropriate build tools and methods, optimize for your target platform, and troubleshoot common build issues.

**Time Estimate**: 1.5-2 hours
**Prerequisites**: [Setup Development Environment](01-setup-development-environment.md), [Run Tests Efficiently](02-run-tests-efficiently.md)
**Difficulty**: Intermediate
**Outcomes**: Compiled C library (.so/.dll/.dylib) ready for integration

---

## The C Library in KNHK

KNHK includes a C library component that provides:
- **Performance-critical operations** (optimized assembly)
- **System integration** (direct OS calls)
- **Interop boundary** (Rust ↔ C communication)
- **Hardware-specific features** (CPU intrinsics)

```
KNHK Architecture:
┌──────────────────────────────┐
│    Rust Application Layer    │  (Your code)
│  (High-level business logic) │
└──────────────────────────────┘
         ↓ FFI (Foreign Function Interface)
┌──────────────────────────────┐
│  C Library (this guide)      │  (Performance critical)
│  (System-level operations)   │
└──────────────────────────────┘
         ↓
┌──────────────────────────────┐
│    Operating System          │
│    (Linux, macOS, Windows)   │
└──────────────────────────────┘
```

---

## Part 1: Understanding the Build System

### The Makefile

KNHK uses a `Makefile` to orchestrate C library compilation:

```bash
# Typical KNHK Makefile structure
make build              # Build C library (all platforms)
make build-debug       # Build with debug symbols
make build-release     # Build optimized for production
make build-static      # Static library (.a) instead of shared
make clean             # Remove build artifacts
```

### Build Outputs

```
After `make build`:
├─ target/
│  ├─ debug/
│  │  └─ libknhk.so          (Linux, debug)
│  │  └─ libknhk.dylib       (macOS, debug)
│  │  └─ knhk.dll            (Windows, debug)
│  └─ release/
│     └─ libknhk.so          (Linux, optimized)
│     └─ libknhk.dylib       (macOS, optimized)
│     └─ knhk.dll            (Windows, optimized)
```

### Build Configuration

The C build can be controlled via:

```bash
# Environment variables
export CFLAGS="-O3 -march=native"      # Custom compiler flags
export CC=clang                         # Use Clang instead of GCC
export AR=llvm-ar                       # Use LLVM archiver

# Or command-line options
make build CFLAGS="-O3 -march=native"
make build CC=clang
make build RELEASE=1                   # Force release build
```

---

## Part 2: Basic C Library Build

### Step 1: Verify Build Prerequisites

```bash
# Check you have a C compiler
gcc --version
clang --version
```

**If missing:**
```bash
# Linux (Ubuntu/Debian)
sudo apt-get install build-essential gcc g++ make

# Linux (RHEL/CentOS)
sudo yum install gcc g++ make

# macOS
xcode-select --install

# Windows (via Visual Studio or mingw-w64)
# Download from: https://visualstudio.microsoft.com/
# OR: https://www.mingw-w64.org/
```

### Step 2: Build the Library

```bash
# Standard build (automatically detects platform)
make build

# Output (on Linux):
# Compiling C library...
# Created: target/debug/libknhk.so
# Build successful ✓

# Verify the library was created
ls -lh target/debug/libknhk.so*
# -rw-r--r-- 1 user group 2.3M Nov 15 12:34 target/debug/libknhk.so
```

### Step 3: Verify the Build

```bash
# Check the library is valid
file target/debug/libknhk.so
# target/debug/libknhk.so: ELF 64-bit LSB shared object, x86-64...

# Check for exported symbols (C functions available)
nm target/debug/libknhk.so | grep knhk_
# 0000000000005c20 T knhk_initialize
# 0000000000005d40 T knhk_process
# 0000000000005e60 T knhk_cleanup
```

---

## Part 3: Build Variations and Options

### Build Type: Debug vs. Release

```bash
# Debug Build (default)
make build
# or explicitly:
make build-debug

# Characteristics:
# - Larger binary size
# - Debug symbols included
# - No optimizations
# - Slower execution
# - Better for development

# Release Build
make build-release

# Characteristics:
# - Smaller binary
# - No debug symbols (can strip)
# - Full optimizations (-O3)
# - Faster execution
# - Better for production
```

### Optimization Levels

```bash
# Manual optimization control
make build CFLAGS="-O0"     # No optimization (fastest build)
make build CFLAGS="-O1"     # Basic optimization
make build CFLAGS="-O2"     # Moderate optimization
make build CFLAGS="-O3"     # Aggressive optimization (default for release)
make build CFLAGS="-Os"     # Optimize for size
make build CFLAGS="-Ofast"  # Ultra-aggressive (may break compatibility)

# Platform-specific optimizations
make build CFLAGS="-O3 -march=native"         # Optimize for THIS machine
make build CFLAGS="-O3 -march=x86-64"         # Portable 64-bit
make build CFLAGS="-O3 -march=armv8-a"        # ARM architecture
make build CFLAGS="-O3 -march=znver3"         # AMD Ryzen
```

### Static vs. Shared Libraries

```bash
# Shared library (default, smaller download, depends on .so/.dll)
make build                  # Creates libknhk.so

# Static library (larger, self-contained, no runtime dependency)
make build-static          # Creates libknhk.a

# When to use:
# Shared: Multiple applications using same library, frequent updates
# Static: Single application, deployment to restricted environments
```

### Compiler Selection

```bash
# Use GCC (default)
make build                      # Uses gcc
make build CC=gcc

# Use Clang (faster, better diagnostics)
make build CC=clang

# Use Intel ICC (best optimization on Intel CPUs)
make build CC=icc

# Cross-compile to different architecture
make build CC=aarch64-linux-gnu-gcc     # Compile ARM on x86
make build CC=x86_64-w64-mingw32-gcc    # Compile Windows on Linux
```

---

## Part 4: Performance-Optimized Builds

### Build for Maximum Performance

```bash
# Aggressive optimization for current CPU
make build-release CFLAGS="-O3 -march=native -flto"

# Breakdown of flags:
# -O3              : Highest optimization level
# -march=native    : Optimize for current CPU (uses all available features)
# -flto            : Link-time optimization (14% smaller, 8% faster)

# Additional micro-optimizations:
make build CFLAGS="-O3 -march=native -flto -funroll-loops -finline-functions"
```

### Build for Portable Distribution

```bash
# Optimized but compatible with older systems
make build-release CFLAGS="-O3 -march=x86-64 -mtune=generic"

# This binary will:
# ✓ Run on systems from ~2010 onwards
# ✓ Get reasonable performance boost
# ✓ Avoid CPU-specific instructions that might not exist
```

### Measure Build Impact

```bash
# Test performance improvement
# Before optimization:
make build-release CFLAGS="-O0"
make test-performance-v04
# Result: operation_x: 12 ticks

# After optimization:
make build-release CFLAGS="-O3 -march=native"
make test-performance-v04
# Result: operation_x: 8 ticks  ✓

# Measured: 33% faster!
```

---

## Part 5: Build Troubleshooting

### Issue 1: Compiler Not Found

**Error:**
```
make: gcc: command not found
```

**Solution:**
```bash
# Install build essentials
sudo apt-get install build-essential    # Linux
brew install gcc                         # macOS
choco install mingw                      # Windows (via Chocolatey)

# Or specify alternative compiler
make build CC=clang
```

### Issue 2: Missing Build Tool

**Error:**
```
make: *** No rule to make target 'build'. Stop.
```

**Solution:**
```bash
# Verify Makefile exists and has build target
ls -la Makefile

# If missing, verify you're in the right directory
pwd
# Should be: /home/user/knhk (or your KNHK root)

# If Makefile is missing, regenerate build system
# (This is project-specific, check CLAUDE.md)
```

### Issue 3: Linking Errors

**Error:**
```
/usr/bin/ld: cannot find -lm
collect2: error: ld returned 1 exit status
```

**Explanation:** Missing math library

**Solution:**
```bash
# Install development libraries
sudo apt-get install libm6 libc6-dev    # Linux
make build LDFLAGS="-lm"                 # Explicitly link math
```

### Issue 4: Permission Denied

**Error:**
```
make: gcc: Permission denied
```

**Explanation:** Your compiler is not executable

**Solution:**
```bash
# Fix executable permissions
chmod +x /usr/bin/gcc

# Or use a different compiler
make build CC=clang
```

### Issue 5: Out of Memory During Build

**Error:**
```
cc1: fatal error: virtual memory exhausted
```

**Solution:**
```bash
# Disable optimization to reduce memory usage
make build CFLAGS="-O0"

# Or use parallel builds more conservatively
make build -j2              # Use 2 cores instead of all

# Increase swap space (Linux)
# Create temporary swap file
sudo dd if=/dev/zero of=/swapfile bs=1G count=4
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### Issue 6: Platform-Specific Symbols

**Error:**
```
undefined reference to `__gettimeofday'
```

**Solution:**
```bash
# Add platform-specific linking flags
make build LDFLAGS="-lrt"                # Real-time library
make build LDFLAGS="-lsocket -lnsl"      # Network (Solaris)
```

---

## Part 6: Verification and Testing

### Verify Build Artifact Quality

```bash
# 1. Check binary is valid
file target/release/libknhk.so
# Should show: ELF 64-bit LSB shared object...

# 2. Check for exported symbols
nm target/release/libknhk.so | grep " T " | wc -l
# Should show: 45    (number of exported functions)

# 3. Check size is reasonable
ls -lh target/release/libknhk.so
# Should be: 2.5M (debug) or 1.2M (release with strip)

# 4. Strip debug symbols (reduces size 50%)
strip target/release/libknhk.so
ls -lh target/release/libknhk.so
# Now: 0.6M
```

### Run C Library Tests

```bash
# Test C library specifically
make test-c
# Output: Running C library tests...
#         test_initialize: PASS
#         test_process: PASS
#         test_cleanup: PASS
#         All tests passed ✓

# Or via Chicago TDD
make test-chicago-v04
# Includes C library validation tests
```

### Performance Verification

```bash
# Build and test performance
make build-release CFLAGS="-O3 -march=native"
make test-performance-v04

# Output should show all operations ≤8 ticks:
# √ operation_a: 5 ticks
# √ operation_b: 7 ticks
# √ operation_c: 3 ticks
# All operations within ≤8 tick limit ✓
```

### Integration Test

```bash
# Test C library integrated with Rust
cargo test --all

# Should include tests like:
# test c_lib_integration ... ok
# test c_lib_performance ... ok
# test c_lib_error_handling ... ok
```

---

## Part 7: Build Optimization Checklist

### Before Shipping to Production

- [ ] Build succeeds with zero warnings: `make build 2>&1 | grep -i warning`
- [ ] All symbols are exported: `nm libknhk.so | grep " T " | wc -l`
- [ ] Binary passes security checks: `checksec --file=libknhk.so`
- [ ] Performance tests pass: `make test-performance-v04`
- [ ] C unit tests pass: `make test-c`
- [ ] Integration tests pass: `cargo test --all`
- [ ] Binary size is optimal: `ls -lh libknhk.so` (should be <3M)
- [ ] Debug symbols are appropriate: `strip -s libknhk.so` for distribution

### Size Optimization

```bash
# Before optimization
ls -lh target/release/libknhk.so
# 2.5M

# Strip symbols (if not needed for debugging)
strip target/release/libknhk.so
ls -lh target/release/libknhk.so
# 1.2M    (52% reduction)

# Use UPX compression (optional, for distribution)
upx --best target/release/libknhk.so
ls -lh target/release/libknhk.so
# 0.4M    (84% reduction!)

# Verify stripped binary still works
make test-c
```

---

## Part 8: Advanced Build Topics

### Build Caching and Incremental Builds

```bash
# First build (full compilation)
time make build
# real: 45s

# Change one file and rebuild (incremental)
time make build
# real: 2s    (44x faster!)

# Force full rebuild
make clean && make build
# real: 45s
```

### Parallel Build

```bash
# Use all CPU cores for faster compilation
make build -j$(nproc)    # On Linux
make build -j$(sysctl -n hw.ncpu)    # On macOS

# Typically 4-8x faster on modern machines
```

### Cross-Compilation

```bash
# Build for different architecture
make build CC=aarch64-linux-gnu-gcc    # ARM64
make build CC=x86_64-w64-mingw32-gcc   # Windows

# Verify target architecture
file libknhk.so
# ELF 64-bit LSB shared object, ARM aarch64...
```

### Build with ASAN (Address Sanitizer)

```bash
# Detect memory errors at runtime
make build CFLAGS="-fsanitize=address -O1 -g"

# Run tests with sanitizer
./test_program
# May show: AddressSanitizer: heap-buffer-overflow
# This detects memory bugs!
```

---

## Part 9: Troubleshooting Advanced Issues

### Issue: Symbol Visibility

**Problem:** Rust can't find C functions

**Solution:**
```bash
# Check if symbols are exported
nm -D target/release/libknhk.so | grep knhk_process

# If missing, ensure functions use visibility attribute
// C code:
__attribute__((visibility("default")))
void knhk_process() { ... }

# Rebuild
make clean && make build
```

### Issue: ABI Compatibility

**Problem:** Rust FFI gets wrong function signature

**Solution:**
```bash
# Verify Rust FFI declaration matches C signature
// C code:
int knhk_initialize(const char* config);

// Rust code:
extern "C" {
    fn knhk_initialize(config: *const c_char) -> c_int;
}

# Must match exactly: return type, parameter types, calling convention
```

### Issue: Linking Against System Libraries

**Problem:** C code needs external libraries

**Solution:**
```bash
# Check what libraries the C code needs
ldd target/release/libknhk.so
# linux-vdso.so.1
# libc.so.6
# /lib64/ld-linux-x86-64.so.2

# If missing dependencies, link explicitly
make build LDFLAGS="-lcrypto -lssl"    # OpenSSL
make build LDFLAGS="-lz"               # Zlib
make build LDFLAGS="-lpthread"         # POSIX threads
```

---

## Part 10: Step-by-Step Complete Build

### Clean Build from Scratch

```bash
# Step 1: Clean any previous artifacts
make clean
rm -rf target/

# Step 2: Build debug version
make build
# Output: Compiling C library (debug)...
#         Created: target/debug/libknhk.so

# Step 3: Verify it works
make test-c
# Output: C library tests: 23/23 passed ✓

# Step 4: Build release version
make build-release CFLAGS="-O3 -march=native"
# Output: Compiling C library (release)...
#         Created: target/release/libknhk.so

# Step 5: Run performance tests
make test-performance-v04
# Output: All operations ≤8 ticks ✓

# Step 6: Verify binary quality
ls -lh target/release/libknhk.so        # Check size
nm target/release/libknhk.so | grep " T " | wc -l  # Check symbols
file target/release/libknhk.so          # Verify format
```

---

## Quick Reference: Build Commands

```bash
# Common builds
make build              # Debug build, current platform
make build-release      # Release build, optimized
make build-static       # Static library

# Build variations
make build CC=clang                          # Use Clang
make build CFLAGS="-O3 -march=native"       # Custom flags
make build-release CFLAGS="-Os"             # Optimize for size

# Verification
make test-c             # C unit tests
make test-chicago-v04   # Chicago TDD tests
make test-performance-v04   # Performance tests

# Cleanup
make clean              # Remove artifacts
rm -rf target/          # Full clean
```

---

## Summary

### The Build Process

1. **Prepare**: Ensure compiler is installed
2. **Build**: `make build` creates library
3. **Verify**: Check binary exists and has symbols
4. **Test**: Run C unit tests and integration tests
5. **Optimize**: Use -O3 -march=native for performance
6. **Validate**: Ensure ≤8 tick performance constraint
7. **Package**: Strip symbols and compress for distribution

### Key Principles

- **Start simple**: `make build` works for most cases
- **Profile before optimizing**: Measure impact of flags
- **Test after changing**: Every build change needs validation
- **Know your target**: Different CPUs need different flags
- **Keep debug symbols**: During development, strip for release

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Prerequisite for**: [How-to: Validate Production Readiness](10-validate-production-readiness.md)
