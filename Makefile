# Makefile for znote

BINARY_NAME  = znote
CARGO        = cargo
INSTALL_DIR  = $(HOME)/.local/bin
COMPLETIONS_DIR_BASH = $(HOME)/.bash_completion.d
COMPLETIONS_DIR_ZSH  = $(HOME)/.zfunc
COMPLETIONS_DIR_FISH = $(HOME)/.config/fish/completions

.PHONY: all build test run clean release lint install uninstall help ui-install ui-build ui-lint ui-clean ui-watch dev dev-server

all: build

## 🏗  Build (debug)
build: ui-build
	$(CARGO) build

## 🧪 Test
test:
	$(CARGO) test

## 🚀 Run
run:
	$(CARGO) run

## 🧹 Clean
clean: ui-clean
	$(CARGO) clean

## 📦 Release build
release: ui-build
	$(CARGO) build --release

## 🔍 Lint
lint: ui-lint
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt -- --check

## 📦 Dependencies
install-rg:
	@if command -v rg >/dev/null 2>&1; then \
		echo "✓ ripgrep already installed"; \
	elif command -v apt-get >/dev/null 2>&1; then \
		sudo apt-get update && sudo apt-get install -y ripgrep; \
	else \
		$(CARGO) install ripgrep; \
	fi

deps-install: install-rg ui-install

## 📥 Install
# Builds a release binary, copies it to ~/.local/bin, and installs shell
# completions for any shells that are detected on this system.
install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✓ Installed $(BINARY_NAME) → $(INSTALL_DIR)/$(BINARY_NAME)"
	@# --- bash ---
	@if command -v bash >/dev/null 2>&1; then \
		mkdir -p $(COMPLETIONS_DIR_BASH); \
		$(INSTALL_DIR)/$(BINARY_NAME) completions bash > $(COMPLETIONS_DIR_BASH)/$(BINARY_NAME); \
		echo "✓ Bash completions → $(COMPLETIONS_DIR_BASH)/$(BINARY_NAME)"; \
		echo "  Activate now:  source $(COMPLETIONS_DIR_BASH)/$(BINARY_NAME)"; \
		echo "  Add to ~/.bashrc:  echo 'source $(COMPLETIONS_DIR_BASH)/$(BINARY_NAME)' >> ~/.bashrc"; \
	fi
	@# --- zsh ---
	@if command -v zsh >/dev/null 2>&1; then \
		mkdir -p $(COMPLETIONS_DIR_ZSH); \
		$(INSTALL_DIR)/$(BINARY_NAME) completions zsh > $(COMPLETIONS_DIR_ZSH)/_$(BINARY_NAME); \
		echo "✓ Zsh completions  → $(COMPLETIONS_DIR_ZSH)/_$(BINARY_NAME)"; \
		echo "  Ensure ~/.zfunc is in fpath and compinit is called (see docs/completions.md)"; \
	fi
	@# --- fish ---
	@if command -v fish >/dev/null 2>&1; then \
		mkdir -p $(COMPLETIONS_DIR_FISH); \
		$(INSTALL_DIR)/$(BINARY_NAME) completions fish > $(COMPLETIONS_DIR_FISH)/$(BINARY_NAME).fish; \
		echo "✓ Fish completions → $(COMPLETIONS_DIR_FISH)/$(BINARY_NAME).fish  (auto-loaded)"; \
	fi

## 🌐 UI Build
ui-install:
	cd ui && npm install

ui-build:
	cd ui && npm run build

ui-watch:
	cd ui && npx vite build --watch

ui-lint:
	cd ui && npm run lint

ui-clean:
	rm -rf ui/dist

## 🛠  Development (requires cargo-watch)
dev:
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "❌ Error: 'cargo-watch' is not installed."; \
		echo "👉 Install it with: cargo install cargo-watch"; \
		exit 1; \
	fi
	@echo "🚀 Starting full-stack dev mode (UI + Server)..."
	@$(MAKE) -j2 ui-watch dev-server

dev-server:
	ZNOTE_DIR=examples cargo watch -i "ui/src/*" -i "ui/node_modules/*" -i "target/*" -x "run -- serve"

## 🗑  Uninstall
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✓ Removed $(INSTALL_DIR)/$(BINARY_NAME)"
	@rm -f $(COMPLETIONS_DIR_BASH)/$(BINARY_NAME)
	@rm -f $(COMPLETIONS_DIR_ZSH)/_$(BINARY_NAME)
	@rm -f $(COMPLETIONS_DIR_FISH)/$(BINARY_NAME).fish
	@echo "✓ Removed shell completions"

## 📖 Help
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build      Build the project (debug) [Rust + UI]"
	@echo "  test       Run tests"
	@echo "  run        Run the project"
	@echo "  clean      Clean build artifacts [Rust + UI]"
	@echo "  release    Build release binary [Rust + UI]"
	@echo "  lint       Run clippy/fmt and UI lint"
	@echo "  install    Build release + install to ~/.local/bin (+ shell completions)"
	@echo "  uninstall  Remove installed binary and completions"
	@echo "  ui-install Install UI dependencies"
	@echo "  ui-build   Build UI static assets"
	@echo "  ui-watch   Build UI assets in watch mode"
	@echo "  ui-lint    Run UI linting"
	@echo "  dev        Run UI and Server watchers in parallel"
	@echo "  help       Show this help message"
