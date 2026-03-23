.DEFAULT_GOAL := help

help:
	@awk '{FS = ":.*##"} /^[A-z0-9_-]+:.*?##/ {printf "→ \033[36m%-21s\033[0m %s\n", $$1, $$2} /^##@/ {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)}' $(MAKEFILE_LIST) && echo


##@ DevOps

init: ## Initialise submodules and create .env
	git submodule update --init --recursive
	@test -f .env || (test -f .env.example && cp .env.example .env || touch .env)
	@echo "✓ Submodules initialised"

submodule-status: ## Show submodule status
	git submodule status

setup: ## Setup environment
	pip install -e .[test,ci,docs]

setup-dev: git-config ## Setup development environment
	pip install -e .[dev]

dev: ## Run tests on file change
	ptw

test: ## Run tests
	pytest .


build:
	@echo "No build, check errors"


##@ Internal

git-config:  ## Configure git to use current organization
	git config --global --replace-all \
		url."https://github.com/$$ORGANIZATION/".insteadOf \
		"https://github.com/slivern-corporate-services/"
