
SHELL = /bin/bash
.SHELLFLAGS = -o pipefail -c

.PHONY: help
help: ## Print info about all commands
	@echo "Commands:"
	@echo
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "    \033[01;32m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: test
test: ## Run all tests and lints
	cargo test -- --test-threads 1

.PHONY: serve
serve: ## Run web service locally, with reloading
	#./target/debug/fatcatd
	cargo run --bin fatcatd

.PHONY: dev-db-reset
dev-db-reset: ## delete any existing local (dev) postgresql database and recreate from scratch
	diesel database reset
	DATABASE_URL=$(TEST_DATABASE_URL) diesel database reset
