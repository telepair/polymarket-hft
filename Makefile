# Polymarket HFT Makefile
# High-frequency trading system for Polymarket

.DEFAULT_GOAL := all

# =============================================================================
# Configuration
# =============================================================================
CARGO        := cargo
CARGO_FLAGS  := --locked
BINARY_NAME  := polymarket

# =============================================================================
# PHONY Targets
# =============================================================================
.PHONY: all fmt lint lint-rust lint-md check test test-unit test-integration \
        test-data test-gamma doc doc-open build release install run clean help

# =============================================================================
# Development Workflow
# =============================================================================

## Primary Targets
all: fmt lint check test build          ## Run full CI pipeline (fmt, lint, check, test, build)

## Code Quality
fmt:                                     ## Format code with rustfmt
	@echo "Formatting code..."
	@$(CARGO) fmt

lint: lint-rust lint-md                  ## Run all linters

lint-rust:                               ## Lint Rust code with clippy
	@echo "Linting Rust code..."
	@$(CARGO) clippy $(CARGO_FLAGS) --all-targets --all-features -- -D warnings

lint-md:                                 ## Lint Markdown files
	@echo "Linting Markdown files..."
	@markdownlint .

check:                                   ## Type-check without building
	@echo "Type-checking code..."
	@$(CARGO) check $(CARGO_FLAGS) --all-targets --all-features

# =============================================================================
# Testing
# =============================================================================

test: test-unit                          ## Run all unit tests

test-unit:                               ## Run unit tests only
	@echo "Running unit tests..."
	@$(CARGO) test $(CARGO_FLAGS) --all-targets --all-features

test-integration: test-data test-gamma   ## Run all integration tests (requires network)

test-data:                               ## Run Data API integration tests
	@echo "Running Data API integration tests..."
	@$(CARGO) test --test data_api_tests -- --ignored --nocapture

test-gamma:                              ## Run Gamma API integration tests
	@echo "Running Gamma API integration tests..."
	@$(CARGO) test --test gamma_api_tests -- --ignored --nocapture

# =============================================================================
# Documentation
# =============================================================================

doc:                                     ## Build documentation
	@echo "Building documentation..."
	@$(CARGO) doc $(CARGO_FLAGS) --no-deps

doc-open:                                ## Build and open documentation in browser
	@echo "Building and opening documentation..."
	@$(CARGO) doc $(CARGO_FLAGS) --no-deps --open

# =============================================================================
# Build & Run
# =============================================================================

build:                                   ## Build debug binary
	@echo "Building debug binary..."
	@$(CARGO) build $(CARGO_FLAGS)

release:                                 ## Build optimized release binary
	@echo "Building release binary..."
	@$(CARGO) build $(CARGO_FLAGS) --release

install:                                 ## Install binary to ~/.cargo/bin
	@echo "Installing $(BINARY_NAME)..."
	@$(CARGO) install $(CARGO_FLAGS) --path .

run:                                     ## Run CLI (use ARGS="..." for arguments)
	@$(CARGO) run $(CARGO_FLAGS) -- $(ARGS)

# =============================================================================
# Maintenance
# =============================================================================

clean:                                   ## Remove build artifacts
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean

update:                                  ## Update dependencies
	@echo "Updating dependencies..."
	@$(CARGO) update

# =============================================================================
# Help
# =============================================================================

help:                                    ## Show available targets
	@echo "Polymarket HFT - Available targets:"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ { \
		printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 \
	}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Examples:"
	@echo "  make                    # Run full CI pipeline"
	@echo "  make test-integration   # Run network tests"
	@echo "  make run ARGS='data health'"
