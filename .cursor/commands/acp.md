# Git Add Commit Push

Automatically add, commit, and push changes.

## Steps

1. **Run tests before committing**: Ensure all tests pass before committing
   - Run `make test-rust` for Rust code (tests run concurrently via scripts/run-all-rust-tests.sh)
   - Run `make test-c` for C code
   - Run `make test-integration-v2` for integration tests (C and Rust tests run concurrently)
   - Run `make test-chicago-v04` for Chicago TDD tests (concurrent execution)
   - Run `make test-performance-v04` for performance tests (concurrent execution)
   - Fix any failing tests before proceeding
2. **Stage changes**: `git add .`
3. **Commit changes**: `git commit -m "Description of changes"`
4. **Push changes**: `git push`

**Note**: If tests fail, fix the issues before committing. Never commit code with failing tests.