ROOT := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
include $(ROOT).toolkit/targets/type.mk

##@ Development

setup: ## Install dependencies
	cargo fetch

setup-dev: git-config ## Setup development environment
	cargo fetch
	cargo install cargo-watch

dev: ## Run tests on file change
	cargo watch -x test

##@ Build

build: ## Build project
	cargo build --release

##@ Test

test: ## Run tests
	cargo test

##@ Lint

clippy: ## Run clippy
	cargo clippy -- -D warnings

fmt: ## Run rustfmt
	cargo fmt --check

lint: clippy fmt ## Run all linting checks

check: test clippy fmt ## Run all checks (test + clippy + fmt)

##@ CI

git-config: ## Configure git to push into current organization
	git config --global --replace-all \
		url."https://github.com/$$ORGANIZATION/".insteadOf \
		"https://github.com/sigil-enterprises/"

help: ## Show help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[:a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
