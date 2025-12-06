# Makefile for Codex CLI AI Tool

# Rust commands
CARGO := cargo

# Default target
default: build

# Build the project
build: 
	$(CARGO) build

# Build in release mode
release: 
	$(CARGO) build --release

# Run the project
run: 
	$(CARGO) run

# Run in release mode
run-release: 
	$(CARGO) run --release

# Run tests
test: 
	$(CARGO) test

# Run tests with verbose output
test-v: 
	$(CARGO) test --verbose

# Run lints
lint: 
	$(CARGO) clippy

# Run type checks
typecheck: 
	$(CARGO) check

# Format code
format: 
	$(CARGO) fmt

# Clean build artifacts
clean: 
	$(CARGO) clean

# Show dependencies
 dependencies: 
	$(CARGO) tree

# Update dependencies
update: 
	$(CARGO) update

# Cross compile for common targets
cross-compile: 
	# Linux x86_64
	CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu $(CARGO) build --release
	# Windows x86_64
	CARGO_BUILD_TARGET=x86_64-pc-windows-gnu $(CARGO) build --release
	# macOS x86_64
	CARGO_BUILD_TARGET=x86_64-apple-darwin $(CARGO) build --release
	# macOS aarch64
	CARGO_BUILD_TARGET=aarch64-apple-darwin $(CARGO) build --release

# Install the binary to ~/codex/bin
install: release
	@mkdir -p ~/codex/bin
	@cp target/release/codex ~/codex/bin/
	@echo "Codex installed to ~/codex/bin"
	@echo ""
	@echo "============================================="
	@echo "To use Codex directly from the command line,"
	@echo "you need to add ~/codex/bin to your PATH."
	@echo "============================================="
	@echo ""
	@echo 'Check if ~/codex/bin is already in PATH:'
	@echo '  echo $$PATH | grep -q ~/codex/bin && echo "✓ Already in PATH" || echo "✗ Not in PATH"'
	@echo ""
	@echo 'To add it temporarily (current session only):'
	@echo '  export PATH=$$HOME/codex/bin:$$PATH'
	@echo ""
	@echo 'To add it permanently, add the following line to your shell configuration file:'
	@echo '  For Bash: ~/.bashrc or ~/.bash_profile'
	@echo '  For Zsh:  ~/.zshrc'
	@echo '  For Fish: ~/.config/fish/config.fish'
	@echo ""
	@echo '  export PATH=$$HOME/codex/bin:$$PATH'
	@echo ""
	@echo 'After adding, run "source <your_shell_config_file>" to apply changes immediately.'
	@echo ""

# Uninstall the binary from ~/codex/bin
uninstall: 
	@rm -f ~/codex/bin/codex
	@echo "Codex uninstalled from ~/codex/bin"

# Show help
help: 
	@echo "Available commands:"
	@echo "  make build          - Build the project"
	@echo "  make release        - Build in release mode"
	@echo "  make run            - Run the project"
	@echo "  make run-release    - Run in release mode"
	@echo "  make test           - Run tests"
	@echo "  make test-v         - Run tests with verbose output"
	@echo "  make lint           - Run lints"
	@echo "  make typecheck      - Run type checks"
	@echo "  make format         - Format code"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make dependencies   - Show dependencies"
	@echo "  make update         - Update dependencies"
	@echo "  make cross-compile  - Cross compile for common targets"
	@echo "  make install        - Install the binary"
	@echo "  make uninstall      - Uninstall the binary"
	@echo "  make help           - Show this help"

.PHONY: default build release run run-release test test-v lint typecheck format clean dependencies update cross-compile install uninstall help
