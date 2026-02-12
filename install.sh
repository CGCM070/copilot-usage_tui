#!/bin/bash

# Installation script for copilot-usage

set -e

echo "Installing copilot-usage..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build release binary
echo "Building release binary..."
cargo build --release

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/copilot-usage ~/.local/bin/

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "WARNING: ~/.local/bin is not in your PATH."
    echo "Add this to your shell configuration file (.bashrc, .zshrc, etc.):"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    echo ""
fi

echo "Installation complete!"
echo ""
echo "Run 'copilot-usage' to start the setup wizard."
echo ""
echo "For Waybar integration, add this to your Waybar config:"
echo '  "custom/copilot": {'
echo '    "exec": "copilot-usage --waybar",'
echo '    "interval": 300,'
echo '    "return-type": "json",'
echo '    "format": " {}",'
echo '    "tooltip": true'
echo '  }'