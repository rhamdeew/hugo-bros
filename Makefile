.PHONY: help dev dev-app tauri-dev build build-tauri build-release preview check check-watch lint lint-fix format format-check clean install install-dev test
.PHONY: cargo-check cargo-clippy cargo-test cargo-clean clean-node clean-all deps-update deps-check bump-patch-version

# Default target
.DEFAULT_GOAL := help

# Variables
NPM := npm
CARGO := cargo
TAURI := npm run tauri

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development targets
dev: ## Run Vite dev server only (frontend)
	$(NPM) run dev

dev-app: ## Run full app in development mode (frontend + Tauri)
	$(NPM) run dev:app

tauri-dev: ## Run Tauri dev mode
	$(TAURI) dev

# Build targets
build: ## Build frontend for production
	$(NPM) run build

build-tauri: ## Build full app for production (frontend + Tauri)
	$(NPM) run build:tauri

build-release: ## Build release version of Tauri app
	$(TAURI) build

# Preview targets
preview: ## Preview production build locally
	$(NPM) run preview

# Type checking targets
check: ## Run type checking
	$(NPM) run check

check-watch: ## Run type checking in watch mode
	$(NPM) run check:watch

# Linting targets
lint: ## Run linter
	$(NPM) run lint

lint-fix: ## Fix linting issues automatically
	$(NPM) run lint:fix

# Formatting targets
format: ## Format code with Prettier
	$(NPM) run format

format-check: ## Check code formatting
	$(NPM) run format:check

# Installation targets
install: ## Install all dependencies
	$(NPM) install

install-dev: ## Install dev dependencies
	$(NPM) install --include=dev

# Rust/Cargo targets
cargo-check: ## Check Rust code for errors
	cd src-tauri && $(CARGO) check

cargo-clippy: ## Run Clippy linter on Rust code
	cd src-tauri && $(CARGO) clippy

cargo-test: ## Run Rust tests
	cd src-tauri && $(CARGO) test

cargo-clean: ## Clean Rust build artifacts
	cd src-tauri && $(CARGO) clean

# Testing targets
test: ## Run all tests
	$(MAKE) cargo-test

# Cleaning targets
clean: ## Clean all build artifacts
	rm -rf .svelte-kit
	rm -rf build
	rm -rf dist
	cd src-tauri && $(CARGO) clean

clean-node: ## Clean node_modules
	rm -rf node_modules

clean-all: clean clean-node ## Clean everything including node_modules
	@echo "Cleaned all build artifacts and dependencies"

# Version bump targets
bump-patch-version: ## Bump patch version, tag commit, and push to origin
	@CURRENT=$$(node -p "require('./package.json').version"); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	PATCH=$$(echo $$CURRENT | cut -d. -f3); \
	NEW_PATCH=$$((PATCH + 1)); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	echo "Current version: $$CURRENT → New version: $$NEW_VERSION"; \
	printf "Update version in package.json, tauri.conf.json, Cargo.toml? [Y/n] "; read ANS; \
	[ "$${ANS:-Y}" = "Y" ] || [ "$${ANS:-Y}" = "y" ] || { echo "Aborted."; exit 1; }; \
	node -e "const fs=require('fs'); const p=JSON.parse(fs.readFileSync('package.json')); p.version='$$NEW_VERSION'; fs.writeFileSync('package.json', JSON.stringify(p, null, 2)+'\n')"; \
	node -e "const fs=require('fs'); const p=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json')); p.version='$$NEW_VERSION'; fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(p, null, 2)+'\n')"; \
	sed -i '' "s/^version = \".*\"/version = \"$$NEW_VERSION\"/" src-tauri/Cargo.toml; \
	echo "Files updated."; \
	printf "Commit message [Bump version to $$NEW_VERSION]: "; read MSG; \
	MSG="$${MSG:-Bump version to $$NEW_VERSION}"; \
	printf "Commit as \"$$MSG\" and tag as v$$NEW_VERSION? [Y/n] "; read ANS; \
	[ "$${ANS:-Y}" = "Y" ] || [ "$${ANS:-Y}" = "y" ] || { echo "Aborted."; exit 1; }; \
	git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml; \
	git commit -m "$$MSG"; \
	git tag "v$$NEW_VERSION"; \
	echo "Committed and tagged v$$NEW_VERSION."; \
	printf "Push branch and tag to origin? [Y/n] "; read ANS; \
	[ "$${ANS:-Y}" = "Y" ] || [ "$${ANS:-Y}" = "y" ] || { echo "Aborted."; exit 1; }; \
	git push origin HEAD; \
	git push origin "v$$NEW_VERSION"; \
	echo "Pushed v$$NEW_VERSION to origin."

# Utility targets
deps-update: ## Update all dependencies
	$(NPM) update
	cd src-tauri && $(CARGO) update

deps-check: ## Check for outdated dependencies
	$(NPM) outdated
	cd src-tauri && $(CARGO) outdated
