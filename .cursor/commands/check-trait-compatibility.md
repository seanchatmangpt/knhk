Check that all traits are dyn compatible (no async trait methods).

This command verifies:
1. No async trait methods in public traits
2. Traits use sync methods that can be made async in implementations
3. Proper use of block_in_place for async operations in trait implementations

Run checks:
1. Search for async trait methods: `grep -r "async fn" --include="*.rs" | grep "trait"`
2. Review trait definitions for async methods
3. Check trait implementations for proper async handling
4. Verify dyn compatibility: `cargo check --workspace`

Common issues:
- Async trait methods break dyn compatibility
- Missing block_in_place for async operations in sync trait methods
- Incorrect async/await usage in trait implementations

Fix pattern:
```rust
// ❌ Bad: Async trait method
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ Good: Sync trait method, async implementation
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;
}

impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async operations here
                Ok(ServiceHandle::new())
            })
        })
    }
}
```

