#!/bin/bash

# macOS Package Build Script for Codex
# This script creates a macOS installer package (.pkg) for Codex

set -e

# Default configuration
VERSION="0.4.4"
IDENTIFIER="com.lyxamour.codex"
PKG_NAME="Codex-$VERSION.pkg"
OUTPUT_DIR="/tmp/codex-pkg"
SOURCE_DIR="$(pwd)"
BUILD_DIR="$SOURCE_DIR/target/release"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Function to check dependencies
check_dependencies() {
    print_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for cargo
    if ! command_exists cargo; then
        missing_deps+=("cargo (Rust build system)")
    fi
    
    # Check for pkgbuild
    if ! command_exists pkgbuild; then
        missing_deps+=("pkgbuild (macOS package builder)")
    fi
    
    # Check for productbuild
    if ! command_exists productbuild; then
        missing_deps+=("productbuild (macOS product builder)")
    fi
    
    # Check for pkgutil
    if ! command_exists pkgutil; then
        missing_deps+=("pkgutil (macOS package utility)")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        print_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    print_success "All dependencies are installed."
}

# Function to build the binary
build_binary() {
    print_info "Building Codex binary..."
    
    cd "$SOURCE_DIR"
    cargo build --release
    
    if [ ! -f "$BUILD_DIR/codex" ]; then
        print_error "Failed to build Codex binary."
        exit 1
    fi
    
    print_success "Codex binary built successfully."
}

# Function to create temporary build structure
create_build_structure() {
    print_info "Creating temporary build structure..."
    
    # Clean up previous build
    rm -rf "$OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"
    
    # Create component structure
    local component_dir="$OUTPUT_DIR/component"
    mkdir -p "$component_dir/usr/local/bin"
    mkdir -p "$component_dir/usr/local/share/codex"
    mkdir -p "$component_dir/Library/LaunchDaemons"
    mkdir -p "$component_dir/Library/Application Support/Codex"
    
    # Copy binary
    cp "$BUILD_DIR/codex" "$component_dir/usr/local/bin/"
    chmod +x "$component_dir/usr/local/bin/codex"
    
    # Create empty directories for data and logs
    mkdir -p "$component_dir/Users/$USER/codex/config"
    mkdir -p "$component_dir/Users/$USER/codex/data"
    mkdir -p "$component_dir/Users/$USER/codex/logs"
    mkdir -p "$component_dir/Users/$USER/codex/cache"
    
    print_success "Temporary build structure created."
}

# Function to create post-install script
create_post_install_script() {
    print_info "Creating post-install script..."
    
    local script_path="$OUTPUT_DIR/postinstall"
    cat > "$script_path" << 'EOF'
#!/bin/bash

# Codex Post-Install Script

# Create user directories if they don't exist
for user in $(dscl . list /Users | grep -v "^_"); do
    local home_dir=$(dscl . read /Users/$user NFSHomeDirectory | cut -d " " -f 2)
    if [ -d "$home_dir" ] && [ "$user" != "root" ]; then
        mkdir -p "$home_dir/codex/config"
        mkdir -p "$home_dir/codex/data"
        mkdir -p "$home_dir/codex/logs"
        mkdir -p "$home_dir/codex/cache"
        chown -R "$user:$user" "$home_dir/codex"
    fi
done

# Create symlink if needed
if [ ! -L "/usr/bin/codex" ]; then
    ln -sf "/usr/local/bin/codex" "/usr/bin/codex" || true
fi

# Add /usr/local/bin to PATH for common shells
for user in $(dscl . list /Users | grep -v "^_"); do
    local home_dir=$(dscl . read /Users/$user NFSHomeDirectory | cut -d " " -f 2)
    if [ -d "$home_dir" ] && [ "$user" != "root" ]; then
        # Bash
        if [ -f "$home_dir/.bash_profile" ] && ! grep -q "/usr/local/bin" "$home_dir/.bash_profile"; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$home_dir/.bash_profile"
        fi
        
        if [ -f "$home_dir/.bashrc" ] && ! grep -q "/usr/local/bin" "$home_dir/.bashrc"; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$home_dir/.bashrc"
        fi
        
        # Zsh
        if [ -f "$home_dir/.zshrc" ] && ! grep -q "/usr/local/bin" "$home_dir/.zshrc"; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$home_dir/.zshrc"
        fi
        
        # Fish
        if [ -d "$home_dir/.config/fish" ]; then
            if [ -f "$home_dir/.config/fish/config.fish" ] && ! grep -q "/usr/local/bin" "$home_dir/.config/fish/config.fish"; then
                echo 'set -x PATH "$HOME/.local/bin" "$PATH"' >> "$home_dir/.config/fish/config.fish"
            fi
        fi
    fi
done

# Set proper permissions
chmod +x /usr/local/bin/codex
chown root:wheel /usr/local/bin/codex
EOF
    
    chmod +x "$script_path"
    print_success "Post-install script created."
}

# Function to create component package
create_component_package() {
    print_info "Creating component package..."
    
    local component_pkg="$OUTPUT_DIR/codex-component.pkg"
    local component_dir="$OUTPUT_DIR/component"
    
    pkgbuild \
        --root "$component_dir" \
        --identifier "$IDENTIFIER" \
        --version "$VERSION" \
        --install-location "/" \
        --scripts "$OUTPUT_DIR" \
        "$component_pkg"
    
    if [ ! -f "$component_pkg" ]; then
        print_error "Failed to create component package."
        exit 1
    fi
    
    print_success "Component package created."
}

# Function to create distribution file
create_distribution_file() {
    print_info "Creating distribution file..."
    
    local dist_file="$OUTPUT_DIR/Distribution"
    
    cat > "$dist_file" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1.0">
    <title>Codex AI</title>
    <background file="background.png" alignment="bottomleft" scale="none" />
    <welcome file="welcome.html" />
    <readme file="readme.html" />
    <license file="license.html" />
    <conclusion file="conclusion.html" />
    
    <pkg-ref id="com.lyxamour.codex">
        <bundle-version>
            <bundle CFBundleShortVersionString="__VERSION__" CFBundleIdentifier="__IDENTIFIER__" />
        </bundle-version>
    </pkg-ref>
    
    <options customize="never" require-scripts="false" />
    <choices-outline>
        <line choice="default" />
    </choices-outline>
    <choice id="default" title="Codex AI">
        <pkg-ref id="com.lyxamour.codex" />
    </choice>
    
    <pkg-ref id="com.lyxamour.codex" version="__VERSION__" onConclusion="none">codex-component.pkg</pkg-ref>
</installer-gui-script>
EOF
    
    # Replace placeholders
    sed -i '' "s/__VERSION__/$VERSION/g" "$dist_file"
    sed -i '' "s/__IDENTIFIER__/$IDENTIFIER/g" "$dist_file"
    
    print_success "Distribution file created."
}

# Function to create HTML resources
create_html_resources() {
    print_info "Creating HTML resources..."
    
    # Create welcome.html
    cat > "$OUTPUT_DIR/welcome.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Welcome to Codex AI</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        h1 { color: #333; }
        p { line-height: 1.6; color: #666; }
    </style>
</head>
<body>
    <h1>Welcome to Codex AI</h1>
    <p>Codex AI is a powerful CLI-based AI programming tool with local knowledge base, remote scraping, and multi-AI platform support.</p>
    <p>This installer will guide you through the installation process.</p>
</body>
</html>
EOF
    
    # Create readme.html
    cat > "$OUTPUT_DIR/readme.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Codex AI Readme</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        h1 { color: #333; }
        h2 { color: #555; }
        p, ul { line-height: 1.6; color: #666; }
        code { font-family: Menlo, Monaco, Consolas, monospace; background-color: #f0f0f0; padding: 2px 4px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>Codex AI Readme</h1>
    
    <h2>What is Codex AI?</h2>
    <p>Codex AI is a CLI-based AI programming tool that helps developers with:</p>
    <ul>
        <li>Local code knowledge base</li>
        <li>Remote web scraping</li>
        <li>Multi-AI platform support</li>
        <li>Code generation and explanation</li>
        <li>Interactive chat interface</li>
    </ul>
    
    <h2>Usage</h2>
    <p>After installation, you can use Codex by running:</p>
    <pre><code>codex [command] [options]</code></pre>
    
    <h3>Basic Commands</h3>
    <ul>
        <li><code>codex chat</code> - Start interactive chat interface</li>
        <li><code>codex ask &lt;question&gt;</code> - Ask a question</li>
        <li><code>codex index &lt;path&gt;</code> - Index code in a directory</li>
        <li><code>codex scrape &lt;url&gt;</code> - Scrape web content</li>
    </ul>
    
    <h2>Configuration</h2>
    <p>Codex configuration files are located in <code>~/codex/config/</code>.</p>
    <p>Main configuration file: <code>~/codex/config/app.yaml</code></p>
</body>
</html>
EOF
    
    # Create license.html
    cat > "$OUTPUT_DIR/license.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>MIT License</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        h1 { color: #333; }
        pre { background-color: #f5f5f5; padding: 20px; border-radius: 5px; overflow-x: auto; font-family: Menlo, Monaco, Consolas, monospace; }
    </style>
</head>
<body>
    <h1>MIT License</h1>
    <pre>
MIT License

Copyright (c) 2024 Lyxamour

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
    </pre>
</body>
</html>
EOF
    
    # Create conclusion.html
    cat > "$OUTPUT_DIR/conclusion.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Installation Complete</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        h1 { color: #333; }
        p, ul { line-height: 1.6; color: #666; }
        code { font-family: Menlo, Monaco, Consolas, monospace; background-color: #f0f0f0; padding: 2px 4px; border-radius: 3px; }
        .success { color: #4CAF50; font-weight: bold; }
    </style>
</head>
<body>
    <h1>Installation Complete</h1>
    <p class="success">✓ Codex AI has been successfully installed!</p>
    
    <h2>What's Next?</h2>
    <ul>
        <li>Open Terminal and type <code>codex --help</code> to see available commands</li>
        <li>Run <code>codex chat</code> to start the interactive chat interface</li>
        <li>Index your codebase with <code>codex index /path/to/your/code</code></li>
        <li>Configure AI providers in <code>~/codex/config/ai.yaml</code></li>
    </ul>
    
    <h2>Documentation</h2>
    <p>For more information, visit:</p>
    <ul>
        <li><a href="https://github.com/lyxamour/codex">GitHub Repository</a></li>
        <li><a href="https://github.com/lyxamour/codex/blob/main/README.md">README</a></li>
        <li><a href="https://github.com/lyxamour/codex/blob/main/docs/user_guide.md">User Guide</a></li>
    </ul>
    
    <p>Enjoy using Codex AI!</p>
</body>
</html>
EOF
    
    print_success "HTML resources created."
}

# Function to build the final package
build_final_package() {
    print_info "Building final package..."
    
    cd "$OUTPUT_DIR"
    
    productbuild \
        --distribution "Distribution" \
        --package-path "codex-component.pkg" \
        --resources "$OUTPUT_DIR" \
        "$SOURCE_DIR/$PKG_NAME"
    
    if [ ! -f "$SOURCE_DIR/$PKG_NAME" ]; then
        print_error "Failed to build final package."
        exit 1
    fi
    
    print_success "Final package created: $SOURCE_DIR/$PKG_NAME"
}

# Function to clean up
clean_up() {
    print_info "Cleaning up..."
    rm -rf "$OUTPUT_DIR"
    print_success "Cleanup completed."
}

# Function to show help
show_help() {
    echo "Codex macOS Package Build Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --version <version>    Specify package version (default: $VERSION)"
    echo "  --identifier <id>      Specify package identifier (default: $IDENTIFIER)"
    echo "  --output-dir <dir>     Specify output directory (default: $SOURCE_DIR)"
    echo "  --no-clean             Don't clean up temporary files"
    echo "  --help                 Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 --version 1.0.0 --identifier com.example.codex"
}

# Main function
main() {
    local no_clean=false
    local output_dir="$SOURCE_DIR"
    
    # Parse command line arguments
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --identifier)
                IDENTIFIER="$2"
                shift 2
                ;;
            --output-dir)
                output_dir="$2"
                shift 2
                ;;
            --no-clean)
                no_clean=true
                shift
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
    
    # Set final package name
    PKG_NAME="Codex-$VERSION.pkg"
    
    print_info "Starting Codex macOS package build..."
    
    # Run build steps
    check_dependencies
    build_binary
    create_build_structure
    create_post_install_script
    create_component_package
    create_distribution_file
    create_html_resources
    build_final_package
    
    # Clean up if requested
    if [ "$no_clean" = false ]; then
        clean_up
    else
        print_warning "Temporary files not cleaned up: $OUTPUT_DIR"
    fi
    
    print_info "Codex macOS package build completed successfully!"
    print_info "Package location: $output_dir/$PKG_NAME"
}

# Run the main function
main "$@"
