# KNHK WASM Makefile
# Simplified commands for building and testing WASM modules

.PHONY: all build build-web build-nodejs build-bundler optimize test clean install-deps help

# Default target
all: install-deps build optimize

# Install required dependencies
install-deps:
	@echo "📦 Installing dependencies..."
	@command -v wasm-pack > /dev/null || cargo install wasm-pack
	@command -v wasm-opt > /dev/null || cargo install wasm-opt

# Build all targets
build: build-web build-nodejs build-bundler

# Build for web (ES modules)
build-web:
	@echo "🌐 Building for web target..."
	wasm-pack build --target web --out-dir ../../wasm-dist/web --release

# Build for Node.js
build-nodejs:
	@echo "🟢 Building for Node.js target..."
	wasm-pack build --target nodejs --out-dir ../../wasm-dist/nodejs --release

# Build for bundlers (webpack/vite/rollup)
build-bundler:
	@echo "📦 Building for bundler target..."
	wasm-pack build --target bundler --out-dir ../../wasm-dist/bundler --release

# Optimize WASM binaries
optimize:
	@echo "⚡ Optimizing WASM binaries..."
	@for target in web nodejs bundler; do \
		if [ -f "../../wasm-dist/$$target/knhk_wasm_bg.wasm" ]; then \
			echo "  Optimizing $$target..."; \
			wasm-opt -Oz --enable-simd \
				-o "../../wasm-dist/$$target/knhk_wasm_bg_opt.wasm" \
				"../../wasm-dist/$$target/knhk_wasm_bg.wasm"; \
			mv "../../wasm-dist/$$target/knhk_wasm_bg_opt.wasm" \
				"../../wasm-dist/$$target/knhk_wasm_bg.wasm"; \
		fi \
	done

# Run tests
test:
	@echo "🧪 Running WASM tests..."
	wasm-pack test --headless --firefox
	wasm-pack test --headless --chrome

# Run benchmarks
bench:
	@echo "📊 Running benchmarks..."
	cargo bench --bench wasm_vs_native

# Check code quality
check:
	@echo "🔍 Running clippy..."
	cargo clippy --target wasm32-unknown-unknown -- -D warnings
	@echo "📝 Checking formatting..."
	cargo fmt -- --check

# Format code
fmt:
	@echo "✨ Formatting code..."
	cargo fmt

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf ../../wasm-dist

# Show file sizes
size:
	@echo "📊 WASM binary sizes:"
	@for target in web nodejs bundler; do \
		if [ -f "../../wasm-dist/$$target/knhk_wasm_bg.wasm" ]; then \
			size=$$(du -h "../../wasm-dist/$$target/knhk_wasm_bg.wasm" | cut -f1); \
			echo "  $$target: $$size"; \
		fi \
	done

# Create distribution package
dist: build optimize
	@echo "📦 Creating distribution package..."
	cd ../../wasm-dist && tar -czf knhk-wasm.tar.gz web/ nodejs/ bundler/
	@echo "✅ Created: wasm-dist/knhk-wasm.tar.gz"

# Development build (faster, with debug symbols)
dev:
	@echo "🔧 Building dev version..."
	wasm-pack build --target web --out-dir ../../wasm-dist/web --dev

# Watch mode (requires cargo-watch)
watch:
	@echo "👀 Watching for changes..."
	cargo watch -s 'make dev'

# Publish to npm (requires authentication)
publish:
	@echo "🚀 Publishing to npm..."
	cd ../../wasm-dist/web && npm publish

# Help
help:
	@echo "KNHK WASM Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all          - Install deps, build, and optimize (default)"
	@echo "  build        - Build all targets (web, nodejs, bundler)"
	@echo "  build-web    - Build for web only"
	@echo "  build-nodejs - Build for Node.js only"
	@echo "  optimize     - Optimize WASM binaries with wasm-opt"
	@echo "  test         - Run WASM tests in browsers"
	@echo "  bench        - Run performance benchmarks"
	@echo "  check        - Run clippy and format checks"
	@echo "  fmt          - Format code"
	@echo "  clean        - Remove build artifacts"
	@echo "  size         - Show WASM binary sizes"
	@echo "  dist         - Create distribution package"
	@echo "  dev          - Fast development build"
	@echo "  watch        - Watch and rebuild on changes"
	@echo "  publish      - Publish to npm"
	@echo "  help         - Show this help"
