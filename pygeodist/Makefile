# Makefile for easy development workflows.
# See development.md for docs.
# Note GitHub Actions call uv directly, not this Makefile.

.DEFAULT_GOAL := default

.PHONY: default install lint test upgrade build clean agent-rules

### Makefile

.PHONY: help
help: ## Display this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {section="General"} /^### /{section=substr($$0,5); printf "\n\033[1m%s\033[0m\n", section} /^[a-zA-Z0-9_-]+:.*?## / {match($$0, /## (.*)$$/, a); printf "  \033[36m%-18s\033[0m %s\n", substr($$1,1,length($$1)-1), a[1]}' $(MAKEFILE_LIST)


### Installation

install: ## Install the project's dependencies with uv (**including all extras and dev dependencies**)
	uv sync --all-extras --dev

### Development

lint: ## Run linters and formatting check
	uv run ruff format --check src tests typings
	uv run ruff check src tests typings
	uv run mypy src tests
	uv run codespell src tests README.md CONTRIBUTING.md

test: ## Run tests
	uv run pytest

fmt: ## Format code
	uv run ruff check --fix --exit-zero src tests typings >/dev/null
	uv run ruff format src tests typings

upgrade: ## Upgrade dependencies
	uv sync --upgrade --all-extras --dev

build: ## Build the project
	uv build

clean: ## Clean up build artifacts and caches
	-rm -rf dist/
	-rm -rf *.egg-info/
	-rm -rf .pytest_cache/
	-rm -rf .mypy_cache/
	-rm -rf .venv/
	-rm -rf CLAUDE.md AGENTS.md
	-find . -type d -name "__pycache__" -exec rm -rf {} +
