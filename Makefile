# KNHK Root Makefile
# Provides convenient targets for project-wide operations

.PHONY: validate-v1.0 validate-production-ready help

help:
	@echo "KNHK Makefile Targets:"
	@echo "  validate-v1.0          - Run v1.0 Definition of Done validation"
	@echo "  validate-production-ready - Run production readiness validation"
	@echo ""
	@echo "See c/Makefile for C-specific targets"

validate-v1.0:
	@bash scripts/validate_v1.0.sh

validate-production-ready:
	@bash scripts/validate-production-ready.sh




validate-dod-v1:
	@echo "Validating Definition of Done v1.0..."
	@./scripts/validate-dod-v1.sh || true
	@bash scripts/generate-dod-report-from-json.sh
	@echo ""
	@echo "Reports generated:"
	@echo "  - docs/V1-DOD-VALIDATION-REPORT.md"
	@echo "  - docs/V1-DOD-PROGRESS.md"
