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

# Install the binary
install: 
	$(CARGO) install --path .

# Uninstall the binary
uninstall: 
	$(CARGO) uninstall codex

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
