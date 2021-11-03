
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
lint: ## Run lints (eg, flake8), does not fail if there are problems
	pipenv run pylint -E fatcat*.py fatcat_tools fatcat_web tests/*.py || true
	pipenv run flake8 *.py tests/ fatcat_web/ fatcat_tools/ --exit-zero
	pipenv run isort -q -c . || true
	pipenv run mypy *.py fatcat_web/ fatcat_tools/ --ignore-missing-imports

.PHONY: lint-verbose
lint-verbose: ## Run lints with extra warnings/notes shown
	pipenv run pylint fatcat*.py fatcat_tools fatcat_web tests/*.py

.PHONY: fmt
fmt: ## Run code formating on all source code
	pipenv run isort --atomic .
	pipenv run black --line-length 96 .

.PHONY: test
test: ## Run all tests and lints
	@curl --silent localhost:9411/v0/changelog > /dev/null || (echo "API not running locally, bailing early from tests" && exit 1)
	pipenv run pytest

.PHONY: coverage
coverage: ## Run all tests with coverage
	pipenv run pytest --cov --cov-report=term --cov-report=html

.PHONY: test-cli
test-cli: ## Run CLI commands. WARNING: may mutate local database
	pipenv run ./tests/cli.sh > /dev/null

.PHONY: serve
serve: ## Run web service locally, with reloading
	pipenv run ./fatcat_webface.py --debug
