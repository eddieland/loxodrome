.DEFAULT_GOAL := help

PY_DIR := loxodrome
RS_DIR := loxodrome-rs

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
lint: ## Run linting for Python and Rust projects
	$(MAKE) -C $(PY_DIR) lint
	$(MAKE) -C $(RS_DIR) lint

.PHONY: fmt
fmt: ## Format Python and Rust code
	$(MAKE) -C $(PY_DIR) fmt
	$(MAKE) -C $(RS_DIR) fmt

.PHONY: test
test: ## Run tests for Python and Rust projects
	$(MAKE) -C $(PY_DIR) test
	$(MAKE) -C $(RS_DIR) test

.PHONY: build
build: ## Build Python and Rust projects
	$(MAKE) -C $(PY_DIR) build
	$(MAKE) -C $(RS_DIR) build

### Rust

.PHONY: bench-rust
bench-rust: ## Run Rust benchmarks
	$(MAKE) -C $(RS_DIR) bench

### Analysis

.PHONY: cloc
cloc: ## Count lines of code using Docker
	docker run --rm -v "$(PWD):/tmp" aldanial/cloc /tmp \
		--exclude-dir=.git,.github,.twig,example,docs,ref,target \
		--fullpath

.PHONY: bench
bench: bench-rust ## Run Rust benchmarks
