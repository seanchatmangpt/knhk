# Makefile for KNHK 8-Tick POC
# Library build system

CC = clang
CFLAGS = -O3 -std=c11 -Wall -Wextra
RAPTOR_CFLAGS = $(shell pkg-config --cflags raptor2 2>/dev/null || echo "-I/opt/homebrew/Cellar/raptor/2.0.16/include")
RAPTOR_LIBS = $(shell pkg-config --libs raptor2 2>/dev/null || echo "-L/opt/homebrew/Cellar/raptor/2.0.16/lib -lraptor2")

# Platform-specific flags
ifeq ($(shell uname -m),arm64)
    CFLAGS += -march=armv8.5-a+fp16
    ARCH = arm64
else ifeq ($(shell uname -m),x86_64)
    CFLAGS += -mavx2
    ARCH = x86_64
endif

# Include directories
INCLUDES = -Iinclude -Isrc

# Library sources
LIB_SRCS = src/knhk.c src/simd.c src/rdf.c src/core.c src/clock.c src/aot/aot_guard.c
LIB_OBJS = $(LIB_SRCS:.c=.o)
LIB = libknhk.a

# Targets
BENCH = tools/knhk_bench
TEST_ENTERPRISE = tests/chicago_enterprise_use_cases
TEST_V1 = tests/chicago_v1_test
TEST_RECEIPTS = tests/chicago_receipts
TEST_CONSTRUCT8 = tests/chicago_construct8
TEST_CONSTRUCT8_PIPELINE = tests/chicago_construct8_pipeline
TEST_BATCH = tests/chicago_batch
TEST_GUARDS = tests/chicago_guards
TEST_INTEGRATION = tests/chicago_integration
TEST_INTEGRATION_V2 = tests/chicago_integration_v2
TEST_INTEGRATION_DOCKER = tests/integration/docker_test.sh
TEST_KAFKA_INTEGRATION = tests/integration/test_kafka_integration
TEST_LOCKCHAIN_INTEGRATION = tests/integration/test_lockchain_integration
TEST_ETL_INTEGRATION = tests/integration/test_etl_integration
TEST_GAPS_V1 = tests/chicago_gaps_v1
TEST_CLI_BOOT = tests/chicago_cli_boot
TEST_CLI_CONNECT = tests/chicago_cli_connect
TEST_CLI_COVER = tests/chicago_cli_cover
TEST_CLI_ADMIT = tests/chicago_cli_admit
TEST_CLI_REFLEX = tests/chicago_cli_reflex
TEST_CLI_EPOCH = tests/chicago_cli_epoch
TEST_CLI_ROUTE = tests/chicago_cli_route
TEST_CLI_RECEIPT = tests/chicago_cli_receipt
TEST_CLI_METRICS = tests/chicago_cli_metrics
TEST_CLI_COVERAGE = tests/chicago_cli_coverage
TEST_V04 = tests/chicago_v04_test
TEST_E2E = tests/chicago_integration_e2e
TEST_NETWORK = tests/chicago_network_integration
TEST_CLI = tests/chicago_cli_integration
TEST_CONFIG = tests/chicago_configuration
TEST_LOCKCHAIN_INT = tests/chicago_lockchain_integration
TEST_PERF_V04 = tests/chicago_performance_v04
TEST_WARM_PATH = tests/chicago_warm_path
TEST_CONFIG_V05 = tests/chicago_config
TEST_COLD_PATH_UNRDF = chicago_cold_path_unrdf_integration

# Rust unrdf integration library
RUST_UNRDF_LIB = ../rust/knhk-unrdf/target/release/libknhk_unrdf.a

.PHONY: all clean test test-enterprise test-v1 test-receipts test-construct8 test-construct8-pipeline test-batch test-guards test-integration test-integration-v2 test-performance test-connectors test-etl test-lockchain test-otel test-gaps-v1 test-chicago-v04 test-e2e test-network test-cli-integration test-config test-lockchain-integration test-performance-v04 test-cli-boot test-cli-connect test-cli-cover test-cli-admit test-cli-reflex test-cli-epoch test-cli-route test-cli-receipt test-cli-metrics test-cli-coverage test-integration-docker test-warm-path test-config-v05 test-cold-path-unrdf validate-v0.4.0 lib bench

all: $(BENCH)

# Build static library
lib: $(LIB)

$(LIB): $(LIB_OBJS)
	ar rcs $@ $^

# Compile library objects
src/%.o: src/%.c
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) -c $< -o $@

# Benchmark tool
$(BENCH): tools/knhk_bench.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

# Enterprise Chicago TDD test suite
$(TEST_ENTERPRISE): tests/chicago_enterprise_use_cases.c tests/chicago_basic_operations.c tests/chicago_cardinality.c tests/chicago_object_operations.c tests/chicago_advanced.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_enterprise_use_cases.c tests/chicago_basic_operations.c tests/chicago_cardinality.c tests/chicago_object_operations.c tests/chicago_advanced.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

# v1.0 Chicago TDD test suites
$(TEST_V1): tests/chicago_v1_test.c tests/chicago_v1_receipts.c tests/chicago_v1_operations.c tests/chicago_v1_validation.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_v1_test.c tests/chicago_v1_receipts.c tests/chicago_v1_operations.c tests/chicago_v1_validation.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_RECEIPTS): tests/chicago_receipts.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_CONSTRUCT8): tests/chicago_construct8.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_CONSTRUCT8_PIPELINE): tests/chicago_construct8_pipeline.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_BATCH): tests/chicago_batch.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_GUARDS): tests/chicago_guards.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_INTEGRATION): tests/chicago_integration.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_INTEGRATION_V2): tests/chicago_integration_v2.c tests/chicago_integration_core.c tests/chicago_integration_systems.c tests/chicago_integration_advanced.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_integration_v2.c tests/chicago_integration_core.c tests/chicago_integration_systems.c tests/chicago_integration_advanced.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_PERFORMANCE): tests/chicago_performance.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_CONNECTORS): tests/chicago_connectors.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_ETL): tests/chicago_etl.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_LOCKCHAIN): tests/chicago_lockchain.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_OTEL): tests/chicago_otel.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_GAPS_V1): tests/chicago_gaps_v1.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

# v0.4.0 Chicago TDD test suites
$(TEST_V04): tests/chicago_v04_test.c tests/chicago_integration_e2e.c tests/chicago_network_integration.c tests/chicago_cli_integration.c tests/chicago_configuration.c tests/chicago_lockchain_integration.c tests/chicago_performance_v04.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_v04_test.c tests/chicago_integration_e2e.c tests/chicago_network_integration.c tests/chicago_cli_integration.c tests/chicago_configuration.c tests/chicago_lockchain_integration.c tests/chicago_performance_v04.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_E2E): tests/chicago_integration_e2e.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_integration_e2e.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_NETWORK): tests/chicago_network_integration.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_network_integration.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_CLI): tests/chicago_cli_integration.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_cli_integration.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_CONFIG): tests/chicago_configuration.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_configuration.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_LOCKCHAIN_INT): tests/chicago_lockchain_integration.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_lockchain_integration.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

$(TEST_PERF_V04): tests/chicago_performance_v04.c tests/chicago_test_helpers.c $(LIB)
	$(CC) $(CFLAGS) -DSTANDALONE_PERFORMANCE_TEST $(INCLUDES) $(RAPTOR_CFLAGS) tests/chicago_performance_v04.c tests/chicago_test_helpers.c -o $@ -L. -lknhk $(RAPTOR_LIBS)

# v0.5.0 Warm path tests
$(TEST_WARM_PATH): tests/chicago_warm_path.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $(RAPTOR_CFLAGS) $< -o $@ -L. -lknhk $(RAPTOR_LIBS)

# v0.5.0 Configuration tests
$(TEST_CONFIG_V05): tests/chicago_config.c
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@

# CLI noun Chicago TDD tests
$(TEST_CLI_BOOT): tests/chicago_cli_boot.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_CONNECT): tests/chicago_cli_connect.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_COVER): tests/chicago_cli_cover.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_ADMIT): tests/chicago_cli_admit.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_REFLEX): tests/chicago_cli_reflex.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_EPOCH): tests/chicago_cli_epoch.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_ROUTE): tests/chicago_cli_route.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_RECEIPT): tests/chicago_cli_receipt.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_METRICS): tests/chicago_cli_metrics.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

$(TEST_CLI_COVERAGE): tests/chicago_cli_coverage.c $(LIB)
	$(CC) $(CFLAGS) $(INCLUDES) $< -o $@ -L. -lknhk

# Integration tests with Docker
$(TEST_KAFKA_INTEGRATION): tests/integration/test_kafka_integration.c
	$(CC) $(CFLAGS) -Iinclude $< -o $@

$(TEST_LOCKCHAIN_INTEGRATION): tests/integration/test_lockchain_integration.c
	$(CC) $(CFLAGS) -Iinclude $< -o $@

$(TEST_ETL_INTEGRATION): tests/integration/test_etl_integration.c
	$(CC) $(CFLAGS) -Iinclude $< -o $@

# Build benchmark
bench: $(BENCH)

# Run enterprise tests
test-enterprise: $(TEST_ENTERPRISE)
	@echo "Running Enterprise Chicago TDD Tests..."
	@./$(TEST_ENTERPRISE)

# Run v1.0 tests
test-v1: $(TEST_V1)
	@echo "Running v1.0 Chicago TDD Tests..."
	@./$(TEST_V1)

test-receipts: $(TEST_RECEIPTS)
	@echo "Running Receipts Tests..."
	@./$(TEST_RECEIPTS)

test-construct8: $(TEST_CONSTRUCT8)
	@echo "Running CONSTRUCT8 Tests..."
	@./$(TEST_CONSTRUCT8)

test-construct8-pipeline: $(TEST_CONSTRUCT8_PIPELINE)
	@echo "Running CONSTRUCT8 Pipeline Tests (Rust → C → Rust)..."
	@./$(TEST_CONSTRUCT8_PIPELINE)

test-batch: $(TEST_BATCH)
	@echo "Running Batch Tests..."
	@./$(TEST_BATCH)

test-guards: $(TEST_GUARDS)
	@echo "Running Guard Tests..."
	@./$(TEST_GUARDS)

test-integration: $(TEST_INTEGRATION)
	@echo "Running Integration Tests..."
	@./$(TEST_INTEGRATION)

test-integration-v2: $(TEST_INTEGRATION_V2)
	@echo "Running Integration v2 Tests (80/20 Critical Path)..."
	@./$(TEST_INTEGRATION_V2)

test-performance: $(TEST_PERFORMANCE)
	@echo "Running Performance Tests..."
	@if [ -f $(TEST_PERFORMANCE) ]; then \
		chmod +x $(TEST_PERFORMANCE) && ./$(TEST_PERFORMANCE); \
	else \
		echo "Performance test executable not found. Building..."; \
		$(MAKE) $(TEST_PERFORMANCE) && chmod +x $(TEST_PERFORMANCE) && ./$(TEST_PERFORMANCE); \
	fi

test-connectors: $(TEST_CONNECTORS)
	@echo "Running Connector Tests..."
	@./$(TEST_CONNECTORS)

test-etl: $(TEST_ETL)
	@echo "Running ETL Pipeline Tests..."
	@./$(TEST_ETL)

test-lockchain: $(TEST_LOCKCHAIN)
	@echo "Running Lockchain Tests..."
	@./$(TEST_LOCKCHAIN)

test-otel: $(TEST_OTEL)
	@echo "Running OpenTelemetry Tests..."
	@./$(TEST_OTEL)

test-gaps-v1: $(TEST_GAPS_V1)
	@echo "Running v1.0 Gaps Integration Tests..."
	@./$(TEST_GAPS_V1)

# v0.4.0 Chicago TDD test targets
test-chicago-v04: $(TEST_V04)
	@echo "Running v0.4.0 Chicago TDD Tests..."
	@./$(TEST_V04)

test-e2e: $(TEST_E2E)
	@echo "Running E2E Integration Tests..."
	@./$(TEST_E2E)

test-network: $(TEST_NETWORK)
	@echo "Running Network Integration Tests..."
	@./$(TEST_NETWORK)

test-cli-integration: $(TEST_CLI)
	@echo "Running CLI Integration Tests..."
	@./$(TEST_CLI)

test-config: $(TEST_CONFIG)
	@echo "Running Configuration Tests..."
	@./$(TEST_CONFIG)

test-config-v05: $(TEST_CONFIG_V05)
	@echo "Running v0.5.0 Configuration Tests..."
	@./$(TEST_CONFIG_V05)

test-warm-path: $(TEST_WARM_PATH)
	@echo "Running Warm Path Tests..."
	@./$(TEST_WARM_PATH)

test-lockchain-integration: $(TEST_LOCKCHAIN_INT)
	@echo "Running Lockchain Integration Tests..."
	@./$(TEST_LOCKCHAIN_INT)

test-performance-v04: $(TEST_PERF_V04)
	@echo "Running Performance Tests v0.4.0..."
	@./$(TEST_PERF_V04)

test-cli-boot: $(TEST_CLI_BOOT)
	@echo "Running CLI Boot Noun Tests..."
	@./$(TEST_CLI_BOOT)

test-cli-connect: $(TEST_CLI_CONNECT)
	@echo "Running CLI Connect Noun Tests..."
	@./$(TEST_CLI_CONNECT)

test-cli-cover: $(TEST_CLI_COVER)
	@echo "Running CLI Cover Noun Tests..."
	@./$(TEST_CLI_COVER)

test-cli-admit: $(TEST_CLI_ADMIT)
	@echo "Running CLI Admit Noun Tests..."
	@./$(TEST_CLI_ADMIT)

test-cli-reflex: $(TEST_CLI_REFLEX)
	@echo "Running CLI Reflex Noun Tests..."
	@./$(TEST_CLI_REFLEX)

test-cli-epoch: $(TEST_CLI_EPOCH)
	@echo "Running CLI Epoch Noun Tests..."
	@./$(TEST_CLI_EPOCH)

test-cli-route: $(TEST_CLI_ROUTE)
	@echo "Running CLI Route Noun Tests..."
	@./$(TEST_CLI_ROUTE)

test-cli-receipt: $(TEST_CLI_RECEIPT)
	@echo "Running CLI Receipt Noun Tests..."
	@./$(TEST_CLI_RECEIPT)

test-cli-metrics: $(TEST_CLI_METRICS)
	@echo "Running CLI Metrics Noun Tests..."
	@./$(TEST_CLI_METRICS)

test-cli-coverage: $(TEST_CLI_COVERAGE)
	@echo "Running CLI Coverage Noun Tests..."
	@./$(TEST_CLI_COVERAGE)

test-cli-all: test-cli-boot test-cli-connect test-cli-cover test-cli-admit test-cli-reflex test-cli-epoch test-cli-route test-cli-receipt test-cli-metrics test-cli-coverage
	@echo "All CLI noun tests completed!"

# Build Rust unrdf integration library
$(RUST_UNRDF_LIB):
	@echo "Building Rust unrdf integration library..."
	@cd ../rust/knhk-unrdf && cargo build --release

# Cold path unrdf integration test
$(TEST_COLD_PATH_UNRDF): ../tests/chicago_cold_path_unrdf_integration.c $(RUST_UNRDF_LIB) $(LIB)
	@echo "Building cold path unrdf integration test..."
	$(CC) $(CFLAGS) $(INCLUDES) ../tests/chicago_cold_path_unrdf_integration.c -o $@ -L. -L../rust/knhk-unrdf/target/release -lknhk -lknhk_unrdf -lpthread -ldl $(RAPTOR_LIBS)

test-cold-path-unrdf: $(TEST_COLD_PATH_UNRDF)
	@echo "Running Cold Path unrdf Integration Tests (C → Rust → unrdf)..."
	@./$(TEST_COLD_PATH_UNRDF)

test-integration-docker: $(TEST_KAFKA_INTEGRATION) $(TEST_LOCKCHAIN_INTEGRATION) $(TEST_ETL_INTEGRATION)
	@echo "Running Docker Integration Tests..."
	@chmod +x $(TEST_INTEGRATION_DOCKER)
	@$(TEST_INTEGRATION_DOCKER)

# Run all tests
test: test-enterprise test-v1 test-receipts test-construct8 test-batch test-guards test-integration test-integration-v2 test-performance test-connectors test-etl test-lockchain test-otel test-gaps-v1 test-cli-all test-chicago-v04
	@echo "All tests completed!"

# Validate v0.4.0 release readiness
validate-v0.4.0:
	@echo "Validating v0.4.0 release criteria..."
	@chmod +x scripts/validate_v0.4.0.sh
	@./scripts/validate_v0.4.0.sh

# Clean build artifacts
clean:
	rm -f $(BENCH) $(TEST_ENTERPRISE) $(TEST_V1) $(TEST_RECEIPTS) $(TEST_CONSTRUCT8) $(TEST_BATCH) $(TEST_GUARDS) $(TEST_INTEGRATION) $(TEST_INTEGRATION_V2) $(TEST_PERFORMANCE) $(TEST_CONNECTORS) $(TEST_ETL) $(TEST_LOCKCHAIN) $(TEST_OTEL) $(TEST_GAPS_V1) $(TEST_CLI_BOOT) $(TEST_CLI_CONNECT) $(TEST_CLI_COVER) $(TEST_CLI_ADMIT) $(TEST_CLI_REFLEX) $(TEST_CLI_EPOCH) $(TEST_CLI_ROUTE) $(TEST_CLI_RECEIPT) $(TEST_CLI_METRICS) $(TEST_CLI_COVERAGE) $(TEST_KAFKA_INTEGRATION) $(TEST_LOCKCHAIN_INTEGRATION) $(TEST_ETL_INTEGRATION) $(TEST_COLD_PATH_UNRDF)
	rm -f $(LIB) $(LIB_OBJS)
	rm -rf tests/*.dSYM tools/*.dSYM tests/integration/*.dSYM
	rm -rf book/book
	@cd rust/knhk-unrdf && cargo clean 2>/dev/null || true

.PHONY: docs docs-serve docs-clean

# mdbook documentation targets
docs:
	@which mdbook > /dev/null 2>&1 || (echo "Error: mdbook not found. Install with: cargo install mdbook" && exit 1)
	mdbook build book/

docs-serve:
	@which mdbook > /dev/null 2>&1 || (echo "Error: mdbook not found. Install with: cargo install mdbook" && exit 1)
	mdbook serve book/

docs-clean:
	rm -rf book/book

# Help target
help:
	@echo "Available targets:"
	@echo "  all              - Build benchmark tool"
	@echo "  lib              - Build static library"
	@echo "  bench            - Build benchmark tool"
	@echo "  test-enterprise  - Build and run enterprise tests"
	@echo "  test-v1          - Build and run v1.0 tests"
	@echo "  test-receipts    - Build and run receipt tests"
	@echo "  test-construct8  - Build and run CONSTRUCT8 tests"
	@echo "  test-batch       - Build and run batch tests"
	@echo "  test-guards      - Build and run guard tests"
	@echo "  test-integration - Build and run integration tests"
	@echo "  test-integration-v2 - Build and run integration v2 tests (80/20 critical path)"
	@echo "  test-performance - Build and run performance tests"
	@echo "  test-connectors  - Build and run connector tests"
	@echo "  test-etl         - Build and run ETL pipeline tests"
	@echo "  test-lockchain   - Build and run lockchain tests"
	@echo "  test-otel        - Build and run OpenTelemetry tests"
	@echo "  test-integration-docker - Run Docker integration tests with testcontainers"
	@echo "  test-chicago-v04 - Build and run v0.4.0 Chicago TDD tests"
	@echo "  test-e2e         - Build and run E2E integration tests"
	@echo "  test-network     - Build and run network integration tests"
	@echo "  test-cli-integration - Build and run CLI integration tests"
	@echo "  test-config      - Build and run configuration tests"
	@echo "  test-lockchain-integration - Build and run lockchain integration tests"
	@echo "  test-performance-v04 - Build and run performance tests v0.4.0"
	@echo "  validate-v0.4.0 - Validate v0.4.0 release readiness"
	@echo "  test             - Run all tests"
	@echo "  clean            - Remove build artifacts"
	@echo "  help             - Show this help message"
