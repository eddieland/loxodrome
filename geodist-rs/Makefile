.DEFAULT_GOAL := help

### Makefile

.PHONY: help
help: ## Display this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {section="General"} /^### /{section=substr($$0,5); printf "\n\033[1m%s\033[0m\n", section} /^[a-zA-Z0-9_-]+:.*?## / {match($$0, /## (.*)$$/, a); printf "  \033[36m%-18s\033[0m %s\n", substr($$1,1,length($$1)-1), a[1]}' $(MAKEFILE_LIST)

### Development

.PHONY: fmt
fmt: ## Format code using rustfmt
	cargo clippy --fix --allow-dirty --workspace
	cargo fmt --all

.PHONY: lint
lint: ## Run clippy for linting
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: bench
bench: ## Run benchmarks (requires bench feature)
	cargo bench --features bench

.PHONY: test
test: build ## Run tests
	cargo nextest run --workspace

.PHONY: test-all
test-all: ## Run tests with all features
	cargo nextest run --all-features

.PHONY: check
check: ## Run cargo check
	cargo check --workspace

.PHONY: doc
doc: ## Generate documentation
	cargo doc --workspace --no-deps

.PHONY: watch-test
watch-test: ## Run tests in watch mode (requires cargo-watch)
	cargo watch -x "nextest run --workspace"

.PHONY: all
all: fmt lint test ## Run fmt, lint, and test

### Coverage

.PHONY: coverage
coverage: ## Run code coverage
	cargo llvm-cov nextest --workspace

.PHONY: coverage-html
coverage-html: ## Generate HTML coverage report
	cargo llvm-cov nextest --workspace --html

.PHONY: coverage-open
coverage-open: ## Generate HTML coverage report and open it in browser
	cargo llvm-cov nextest --workspace --html --open

.PHONY: coverage-report
coverage-report: ## Generate LCOV report
	cargo llvm-cov nextest --workspace --lcov --output-path lcov.info

### Build

.PHONY: build
build: ## Build the project
	cargo build --workspace

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

### Installation

.PHONY: install-dev-tools
install-dev-tools: ## Install development tools
	rustup show # Ensures rust-toolchain.toml is applied
	cargo install cargo-nextest
	cargo install cargo-llvm-cov
