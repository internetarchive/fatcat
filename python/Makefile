
SHELL = /bin/bash
.SHELLFLAGS = -o pipefail -c

.PHONY: help
help: ## Print info about all commands
	@echo "Commands:"
	@echo
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "    \033[01;32m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: test
test: ## Run all tests and lints
	curl --silent localhost:9411/v0/changelog > /dev/null || (echo "API not running locally, bailing early from tests" && exit 1)
	pipenv run pytest
	pipenv run pylint -j 0 -E fatcat*.py fatcat_tools fatcat_web tests/*.py
	pipenv run flake8 tests/ fatcat_web/ fatcat_tools/ *.py --count --select=E9,F63,F7,F82 --show-source --statistics

.PHONY: test-cli
test-cli: ## Run CLI commands. WARNING: may mutate local database
	pipenv run ./tests/cli.sh > /dev/null

.PHONY: dev
dev: ## Run web service locally, with reloading
	pipenv run ./fatcat_webface.py --debug
