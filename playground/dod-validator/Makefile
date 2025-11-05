# DOD Validator Makefile

# Paths
KNHK_ROOT = ../..
C_DIR = c
RUST_DIR = rust

# Build targets
.PHONY: all clean test build

all: build

build:
	@echo "Building DOD Validator..."
	cd $(RUST_DIR) && cargo build --release

clean:
	@echo "Cleaning DOD Validator..."
	cd $(RUST_DIR) && cargo clean

test:
	@echo "Running tests..."
	cd $(RUST_DIR) && cargo test

install: build
	@echo "Installing DOD Validator..."
	cp $(RUST_DIR)/target/release/dod-validator /usr/local/bin/

help:
	@echo "KNHK DoD Validator Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make build    - Build the validator"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install  - Install binary to /usr/local/bin"
	@echo "  make help     - Show this help"

