# Git Add Commit Push

Automatically add, commit, and push changes.

## Steps

1. **Fix issues first**: Run pre-commit validation and fix any issues
   - Run `make pre-commit` to check compilation, formatting, and linting
   - Fix any compilation errors (unclosed delimiters, syntax errors, etc.)
   - Run `make fmt` to format code if needed
   - Fix any linting issues reported by `make lint-rust`
   - **CRITICAL**: Never use `cargo fmt` directly - always use `make fmt`
2. **Run tests before committing**: Ensure all tests pass before committing
   - Run `make test-rust` for Rust code (tests run concurrently via scripts/run-all-rust-tests.sh)
   - Run `make test-c` for C code
   - Run `make test-integration-v2` for integration tests (C and Rust tests run concurrently)
   - Run `make test-chicago-v04` for Chicago TDD tests (concurrent execution)
   - Run `make test-performance-v04` for performance tests (concurrent execution)
   - Fix any failing tests before proceeding
3. **Stage changes**: `git add .`
4. **Commit changes**: `git commit -m "Description of changes"`
5. **Push changes**: `git push`

**Note**: 
- Always fix issues first using `make pre-commit` before running tests
- If tests fail, fix the issues before committing. Never commit code with failing tests.
- **NEVER use `cargo fmt` directly** - always use `make fmt` instead