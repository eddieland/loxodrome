.DEFAULT_GOAL := help

PY_DIR := pygeodist
RS_DIR := geodist-rs

### Makefile

.PHONY: help
help: ## Display this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {section="General"} /^### /{section=substr($$0,5); printf "\n\033[1m%s\033[0m\n", section} /^[a-zA-Z0-9_-]+:.*?## / {match($$0, /## (.*)$$/, a); printf "  \033[36m%-18s\033[0m %s\n", substr($$1,1,length($$1)-1), a[1]}' $(MAKEFILE_LIST)

### Aggregate

.PHONY: all
all: fmt lint test build ## Run formatting, linting, tests, and builds for all projects

.PHONY: lint
lint: lint-python lint-rust ## Run linting for Python and Rust projects

.PHONY: fmt
fmt: fmt-python fmt-rust ## Format Python and Rust code

.PHONY: test
test: test-python test-rust ## Run tests for Python and Rust projects

.PHONY: build
build: build-python build-rust ## Build Python and Rust projects

### Python

.PHONY: lint-python
lint-python: ## Run Python linters and type checks
	$(MAKE) -C $(PY_DIR) lint

.PHONY: fmt-python
fmt-python: ## Format Python code
	$(MAKE) -C $(PY_DIR) fmt

.PHONY: test-python
test-python: ## Run Python tests
	$(MAKE) -C $(PY_DIR) test

.PHONY: build-python
build-python: ## Build Python artifacts
	$(MAKE) -C $(PY_DIR) build

### Rust

.PHONY: lint-rust
lint-rust: ## Run Rust lints
	$(MAKE) -C $(RS_DIR) lint

.PHONY: fmt-rust
fmt-rust: ## Format Rust code
	$(MAKE) -C $(RS_DIR) fmt

.PHONY: bench-rust
bench-rust: ## Run Rust benchmarks
	$(MAKE) -C $(RS_DIR) bench

.PHONY: test-rust
test-rust: ## Run Rust tests
	$(MAKE) -C $(RS_DIR) test

.PHONY: build-rust
build-rust: ## Build Rust artifacts
	$(MAKE) -C $(RS_DIR) build

.PHONY: bench
bench: bench-rust ## Run Rust benchmarks

### Analysis

.PHONY: cloc
cloc: ## Count lines of code using Docker
	docker run --rm -v "$(PWD):/tmp" aldanial/cloc /tmp \
		--exclude-dir=.git,.github,.twig,example,docs,ref,target \
		--fullpath
