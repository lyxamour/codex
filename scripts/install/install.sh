#!/bin/bash

# Codex AI Installation Script
# Supports Linux and macOS systems
# Supports multiple architectures (x86_64, aarch64)

set -e

# Default configuration
DEFAULT_INSTALL_DIR="$HOME/codex/bin"
DEFAULT_CONFIG_DIR="$HOME/codex/config"
DEFAULT_DATA_DIR="$HOME/codex/data"
DEFAULT_LOGS_DIR="$HOME/codex/logs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub repository information
GITHUB_OWNER="lyxamour"
GITHUB_REPO="codex"
GITHUB_API_URL="https://api.github.com"

# Function to print messages with color
print_message() {
    local color="$1"
    local message="$2"
    echo -e "${color}${message}${NC}"
}

# Function to print error messages
print_error() {
    local message="$1"
    print_message "$RED" "ERROR: $message"
}

# Function to print success messages
print_success() {
    local message="$1"
    print_message "$GREEN" "✓ $message"
}

# Function to print info messages
print_info() {
    local message="$1"
    print_message "$BLUE" "ℹ️  $message"
}

# Function to print warning messages
print_warning() {
    local message="$1"
    print_message "$YELLOW" "⚠️  $message"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect the operating system
detect_os() {
    if [ "$(uname -s)" = "Linux" ]; then
        echo "linux"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "macos"
    else
        print_error "Unsupported operating system: $(uname -s)"
        exit 1
    fi
}

# Function to detect the architecture
detect_arch() {
    local arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) print_error "Unsupported architecture: $arch" ; exit 1 ;;
    esac
}

# Function to get the latest release version
get_latest_version() {
    print_info "Checking for latest Codex version..."
    if command_exists curl; then
        LATEST_VERSION=$(curl -s "$GITHUB_API_URL/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/latest" | grep -o '"tag_name": "[^\"]*"' | cut -d '"' -f 4 | sed 's/^v//')
    elif command_exists wget; then
        LATEST_VERSION=$(wget -qO- "$GITHUB_API_URL/repos/$GITHUB_OWNER/$GITHUB_REPO/releases/latest" | grep -o '"tag_name": "[^\"]*"' | cut -d '"' -f 4 | sed 's/^v//')
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    if [ -z "$LATEST_VERSION" ]; then
        print_error "Failed to get the latest version. Please check your internet connection."
        exit 1
    fi
    
    print_success "Latest version: v$LATEST_VERSION"
    echo "$LATEST_VERSION"
}

# Function to download the binary
download_binary() {
    local version="$1"
    local os="$2"
    local arch="$3"
    local output_dir="$4"
    
    local download_url="https://github.com/$GITHUB_OWNER/$GITHUB_REPO/releases/download/v$version/codex-$os-$arch.tar.gz"
    local temp_file="/tmp/codex-$version-$os-$arch.tar.gz"
    
    print_info "Downloading Codex v$version for $os-$arch..."
    
    if command_exists curl; then
        curl -L -o "$temp_file" "$download_url"
    elif command_exists wget; then
        wget -O "$temp_file" "$download_url"
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    # Extract the binary
    print_info "Extracting binary..."
    mkdir -p "$output_dir"
    tar -xzf "$temp_file" -C "$output_dir"
    
    # Make the binary executable
    chmod +x "$output_dir/codex"
    
    # Clean up
    rm "$temp_file"
    
    print_success "Binary downloaded and extracted to $output_dir"
}

# Function to check if Codex is already installed
is_installed() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    [ -f "$install_dir/codex" ]
}

# Function to get the current installed version
get_installed_version() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    if is_installed "$install_dir"; then
        "$install_dir/codex" --version 2>&1 | grep -o 'version [^ ]*' | cut -d ' ' -f 2
    else
        echo ""
    fi
}

# Function to add directory to PATH
do_add_to_path() {
    local dir="$1"
    local shell="$(basename "$SHELL")"
    
    print_info "Adding $dir to PATH..."
    
    case "$shell" in
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                echo "export PATH=\"$dir:\$PATH\"" >> "$HOME/.bashrc"
                print_success "Added to $HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                echo "export PATH=\"$dir:\$PATH\"" >> "$HOME/.bash_profile"
                print_success "Added to $HOME/.bash_profile"
            fi
            ;;
        zsh)
            if [ -f "$HOME/.zshrc" ]; then
                echo "export PATH=\"$dir:\$PATH\"" >> "$HOME/.zshrc"
                print_success "Added to $HOME/.zshrc"
            fi
            ;;
        fish)
            if [ -f "$HOME/.config/fish/config.fish" ]; then
                echo "set -x PATH \"$dir\" \$PATH" >> "$HOME/.config/fish/config.fish"
                print_success "Added to $HOME/.config/fish/config.fish"
            fi
            ;;
        *)
            print_warning "Unknown shell: $shell. Please add $dir to your PATH manually."
            ;;
    esac
    
    # Also add to current session
    export PATH="$dir:$PATH"
}

# Function to install Codex
install_codex() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    local add_to_path="${2:-true}"
    
    # Create necessary directories
    mkdir -p "$install_dir"
    mkdir -p "$DEFAULT_CONFIG_DIR"
    mkdir -p "$DEFAULT_DATA_DIR"
    mkdir -p "$DEFAULT_LOGS_DIR"
    
    # Get system info
    local os="$(detect_os)"
    local arch="$(detect_arch)"
    local version="$(get_latest_version)"
    
    # Check if already installed
    if is_installed "$install_dir"; then
        local installed_version="$(get_installed_version "$install_dir")"
        if [ "$installed_version" = "$version" ]; then
            print_success "Codex v$version is already installed in $install_dir"
            return 0
        else
            print_warning "Codex v$installed_version is already installed. Updating to v$version..."
        fi
    fi
    
    # Download binary
    download_binary "$version" "$os" "$arch" "$install_dir"
    
    # Add to PATH if requested
    if [ "$add_to_path" = "true" ]; then
        do_add_to_path "$install_dir"
    fi
    
    # Create symlink if install dir is not in PATH
    if ! echo "$PATH" | grep -q "$install_dir"; then
        print_info "Creating symlink in /usr/local/bin..."
        if [ -w "/usr/local/bin" ]; then
            ln -sf "$install_dir/codex" /usr/local/bin/codex
            print_success "Created symlink /usr/local/bin/codex -> $install_dir/codex"
        else
            print_warning "Cannot create symlink in /usr/local/bin (permission denied). Please run as root or add $install_dir to your PATH."
        fi
    fi
    
    print_success "Codex v$version has been successfully installed!"
    print_info "You can now use Codex by running: codex"
    
    if [ "$add_to_path" = "true" ]; then
        print_warning "To use Codex in the current session, run: source $HOME/.bashrc (or equivalent for your shell)"
    fi
}

# Function to uninstall Codex
uninstall_codex() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    if ! is_installed "$install_dir"; then
        print_error "Codex is not installed in $install_dir"
        return 1
    fi
    
    print_info "Uninstalling Codex..."
    
    # Remove the binary
    rm -f "$install_dir/codex"
    
    # Remove symlink if it exists
    if [ -L "/usr/local/bin/codex" ]; then
        rm -f /usr/local/bin/codex
    fi
    
    # Remove empty directories
    rmdir "$install_dir" 2>/dev/null || true
    rmdir "$DEFAULT_CONFIG_DIR" 2>/dev/null || true
    rmdir "$DEFAULT_DATA_DIR" 2>/dev/null || true
    rmdir "$DEFAULT_LOGS_DIR" 2>/dev/null || true
    rmdir "$HOME/codex" 2>/dev/null || true
    
    print_success "Codex has been successfully uninstalled!"
    print_warning "You may need to remove the PATH entry from your shell configuration file manually."
}

# Function to update Codex
update_codex() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    if ! is_installed "$install_dir"; then
        print_error "Codex is not installed in $install_dir"
        return 1
    fi
    
    local installed_version="$(get_installed_version "$install_dir")"
    local latest_version="$(get_latest_version)"
    
    if [ "$installed_version" = "$latest_version" ]; then
        print_success "Codex is already up to date (v$installed_version)"
        return 0
    fi
    
    print_info "Updating Codex from v$installed_version to v$latest_version..."
    
    # Get system info
    local os="$(detect_os)"
    local arch="$(detect_arch)"
    
    # Download and replace the binary
    download_binary "$latest_version" "$os" "$arch" "$install_dir"
    
    print_success "Codex has been successfully updated to v$latest_version!"
}

# Function to show help
show_help() {
    echo "Codex AI Installation Script"
    echo ""
    echo "Usage: $0 [options] [command]"
    echo ""
    echo "Commands:"
    echo "  install    Install Codex (default)"
    echo "  uninstall  Uninstall Codex"
    echo "  update     Update Codex to the latest version"
    echo "  check      Check if Codex is installed and its version"
    echo "  help       Show this help message"
    echo ""
    echo "Options:"
    echo "  --install-dir <dir>  Specify installation directory (default: $DEFAULT_INSTALL_DIR)"
    echo "  --no-add-to-path     Don't add installation directory to PATH"
    echo "  --version            Show script version"
    echo "  --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 install                      Install Codex to default directory and add to PATH"
    echo "  $0 install --no-add-to-path     Install Codex without adding to PATH"
    echo "  $0 install --install-dir /opt/codex  Install Codex to /opt/codex"
    echo "  $0 update                       Update Codex to the latest version"
    echo "  $0 uninstall                    Uninstall Codex"
    echo "  $0 check                        Check installed version"
}

# Main function
main() {
    local command="install"
    local install_dir="$DEFAULT_INSTALL_DIR"
    local add_to_path="true"
    
    # Parse command line arguments
    while [ "$#" -gt 0 ]; do
        case "$1" in
            install|uninstall|update|check|help)
                command="$1"
                shift
                ;;
            --install-dir)
                install_dir="$2"
                shift 2
                ;;
            --no-add-to-path)
                add_to_path="false"
                shift
                ;;
            --version)
                echo "Codex Installation Script v1.0.0"
                exit 0
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown argument: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Execute the command
    case "$command" in
        install)
            install_codex "$install_dir" "$add_to_path"
            ;;
        uninstall)
            uninstall_codex "$install_dir"
            ;;
        update)
            update_codex "$install_dir"
            ;;
        check)
            local installed_version="$(get_installed_version "$install_dir")"
            if [ -n "$installed_version" ]; then
                print_success "Codex is installed (v$installed_version) in $install_dir"
            else
                print_warning "Codex is not installed in $install_dir"
            fi
            ;;
        help)
            show_help
            ;;
        *)
            print_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Run the main function
main "$@"
