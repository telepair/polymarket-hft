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
.PHONY: all fmt lint lint-rust lint-md check test test-integration test-all doc doc-open build release install run clean update help

# =============================================================================
# Development Workflow
# =============================================================================

## Primary Targets
all: fmt lint check test-all doc build          ## Run full CI pipeline (fmt, lint, check, test, build)

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

test:                                 ## Run unit tests
	@echo "Running unit tests..."
	@$(CARGO) test $(CARGO_FLAGS) --lib --bins

test-integration:                     ## Run integration tests
	@echo "Running integration tests..."
	@$(CARGO) test $(CARGO_FLAGS) --test '*'

test-all:                             ## Run all tests (unit + integration + doc)
	@echo "Running all tests..."
	@$(CARGO) test $(CARGO_FLAGS) --all-targets --all-features

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
