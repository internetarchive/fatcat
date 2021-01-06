
SHELL = /bin/bash
.SHELLFLAGS = -o pipefail -c

.PHONY: help
help: ## Print info about all commands
	@echo "Commands:"
	@echo
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "    \033[01;32m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: dep
dep: ## Create local virtualenv using pipenv
	pipenv install --dev

.PHONY: lint
lint: ## Run lints (eg, flake8)
	pipenv run flake8 *.py tests/ fatcat_web/ fatcat_tools/ --select=E9,F63,F7,F82
	pipenv run flake8 *.py tests/ fatcat_web/ fatcat_tools/ --exit-zero

.PHONY: lint-mypy
lint-mypy: ## Run mypy type checks (not part of regular lint yet)
	pipenv run mypy *.py fatcat_web/ fatcat_tools/ --ignore-missing-imports

# Not ready for 'black' yet
#.PHONY: fmt
#fmt: ## Run code formating on all source code
#	pipenv run black *.py fatcat_web/ fatcat_tools/ tests/

.PHONY: test
test: ## Run all tests and lints
	@curl --silent localhost:9411/v0/changelog > /dev/null || (echo "API not running locally, bailing early from tests" && exit 1)
	pipenv run pytest

.PHONY: coverage
coverage: ## Run all tests with coverage
	pipenv run pytest --cov

.PHONY: test-cli
test-cli: ## Run CLI commands. WARNING: may mutate local database
	pipenv run ./tests/cli.sh > /dev/null

.PHONY: serve
serve: ## Run web service locally, with reloading
	pipenv run ./fatcat_webface.py --debug
