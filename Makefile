# Polymarket SDK Makefile

.DEFAULT_GOAL := all

# =============================================================================
# Variables
# =============================================================================
CARGO := cargo

# =============================================================================
# PHONY Targets
# =============================================================================
.PHONY: all fmt lint lint-rust lint-md test test-integration doc build release clean help

# =============================================================================
# Main Targets
# =============================================================================

all: fmt lint test build ## Run all checks and build

fmt: ## Format code
	@echo "Formatting code..."
	@$(CARGO) fmt

lint: lint-rust lint-md ## Run all linters (clippy + markdownlint)

lint-rust:
	@echo "Linting Rust code..."
	@$(CARGO) clippy -- -D warnings

lint-md:
	@echo "Linting Markdown code..."
	@markdownlint .

test: ## Run tests
	@echo "Running tests..."
	@$(CARGO) test

test-integration: ## Run integration tests (requires network)
	@echo "Running integration tests..."
	@$(CARGO) test --test data_api_tests -- --ignored --nocapture

doc: ## Build documentation
	@echo "Building documentation..."
	@$(CARGO) doc --open

build: ## Build debug binary
	@echo "Building debug binary..."
	@$(CARGO) build

release: ## Build release binary
	@echo "Building release binary..."
	@$(CARGO) build --release

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean

# =============================================================================
# Help
# =============================================================================

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'
