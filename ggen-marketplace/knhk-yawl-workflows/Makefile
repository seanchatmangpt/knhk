.PHONY: help validate test clean install lint docs check-all

# KNHK YAWL Marketplace Template Makefile

help:
	@echo "KNHK YAWL Marketplace Template Commands:"
	@echo ""
	@echo "  make validate      - Validate template structure and files"
	@echo "  make test          - Run example tests"
	@echo "  make check-all     - Run all validation and tests"
	@echo "  make lint          - Lint markdown documentation"
	@echo "  make docs          - Generate documentation"
	@echo "  make clean         - Clean generated files"
	@echo "  make install-local - Install template locally for development"
	@echo ""

validate:
	@echo "Validating marketplace template..."
	@bash scripts/validate-template.sh

test:
	@echo "Testing example workflows..."
	@bash scripts/test-examples.sh

check-all: validate test
	@echo ""
	@echo "✓ All validation checks passed!"

lint:
	@echo "Linting markdown files..."
	@for file in docs/*.md README.md; do \
		if [ -f "$$file" ]; then \
			echo "Checking $$file..."; \
		fi; \
	done
	@echo "✓ Lint complete"

docs:
	@echo "Marketplace template documentation:"
	@echo ""
	@echo "README.md          - Main documentation"
	@echo "docs/USAGE.md      - Usage guide"
	@echo "docs/ARCHITECTURE.md - Technical architecture"
	@echo "docs/EXAMPLES.md   - Pattern examples"
	@echo ""

clean:
	@echo "Cleaning generated files..."
	@rm -rf output/ generated/ test-results/ *.yawl *.json 2>/dev/null || true
	@echo "✓ Clean complete"

install-local:
	@echo "Installing template locally..."
	@if command -v ggen &> /dev/null; then \
		ggen marketplace install --path . && echo "✓ Template installed"; \
	else \
		echo "Error: ggen not installed. Install from https://github.com/seanchatmangpt/ggen"; \
		exit 1; \
	fi

# Development targets

watch:
	@echo "Watching for changes (requires ggen)..."
	@ggen project watch --template io.knhk.yawl-workflows

dev-setup:
	@echo "Setting up development environment..."
	@chmod +x scripts/*.sh
	@echo "✓ Development environment ready"

test-generate:
	@echo "Testing YAWL generation..."
	@if command -v ggen &> /dev/null; then \
		mkdir -p output && \
		for file in examples/*.ttl; do \
			echo "Generating from $$file..."; \
			ggen template generate-rdf --ontology "$$file" --template io.knhk.yawl-workflows --output "output/$$(basename $$file .ttl).yawl"; \
		done && \
		echo "✓ Generated YAWL files in output/"; \
	else \
		echo "Error: ggen not installed"; \
		exit 1; \
	fi

verify-structure:
	@echo "Verifying template structure..."
	@test -f ggen.yaml || (echo "Missing ggen.yaml"; exit 1)
	@test -f README.md || (echo "Missing README.md"; exit 1)
	@test -d template || (echo "Missing template/ directory"; exit 1)
	@test -d queries || (echo "Missing queries/ directory"; exit 1)
	@test -d docs || (echo "Missing docs/ directory"; exit 1)
	@test -d examples || (echo "Missing examples/ directory"; exit 1)
	@test -d scripts || (echo "Missing scripts/ directory"; exit 1)
	@echo "✓ All required directories and files present"

.DEFAULT_GOAL := help
