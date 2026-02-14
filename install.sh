#!/bin/bash

# Installation script for copilot-usage
# Uses cargo install for safer, atomic installation

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Installing copilot-usage"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Show pre-installation info
if command -v copilot-usage &> /dev/null; then
    echo "Previous installation found:"
    PREV_VERSION=$(copilot-usage --version 2>/dev/null || echo "unknown")
    PREV_PATH=$(which copilot-usage)
    echo "  Version: $PREV_VERSION"
    echo "  Location: $PREV_PATH"
    if [ -f "$PREV_PATH" ]; then
        PREV_HASH=$(sha256sum "$PREV_PATH" | cut -d' ' -f1 | cut -c1-16)
        PREV_DATE=$(stat -c %y "$PREV_PATH" 2>/dev/null | cut -d' ' -f1-2 || echo "unknown")
        echo "  Hash (first 16): $PREV_HASH"
        echo "  Build date: $PREV_DATE"
    fi
    echo ""
fi

# Run pre-commit checks
echo "Running pre-commit checks..."
echo "  → cargo clippy..."
if ! cargo clippy -- -D warnings 2>&1 | grep -q "0 warnings emitted"; then
    echo ""
    echo "WARNING: Clippy found issues. Fix them before installing:"
    echo "  cargo clippy -- -D warnings"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "  → cargo fmt check..."
if ! cargo fmt -- --check &> /dev/null; then
    echo ""
    echo "WARNING: Code is not formatted. Run: cargo fmt"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "  → cargo test..."
if ! cargo test --quiet 2>&1 | tail -n 5; then
    echo ""
    echo "WARNING: Tests failed. Fix them before installing."
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo "All checks passed!"
echo ""

# Install using cargo install (atomic, safer than cp)
echo "Building and installing release binary..."
echo "  → Using: cargo install --path . --force"
echo ""

cargo install --path . --force

# Check if ~/.cargo/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    echo ""
    echo "WARNING: ~/.cargo/bin is not in your PATH."
    echo "Add this to your shell configuration file (.bashrc, .zshrc, etc.):"
    echo '  export PATH="$HOME/.cargo/bin:$PATH"'
    echo ""
fi

# Verify installation
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Installation successful!"
echo "════════════════════════════════════════════════════════════"
echo ""

if command -v copilot-usage &> /dev/null; then
    NEW_VERSION=$(copilot-usage --version 2>/dev/null || echo "unknown")
    NEW_PATH=$(which copilot-usage)
    echo "Installed binary info:"
    echo "  Version: $NEW_VERSION"
    echo "  Location: $NEW_PATH"
    
    if [ -f "$NEW_PATH" ]; then
        NEW_HASH=$(sha256sum "$NEW_PATH" | cut -d' ' -f1 | cut -c1-16)
        NEW_DATE=$(stat -c %y "$NEW_PATH" 2>/dev/null | cut -d' ' -f1-2 || echo "unknown")
        NEW_SIZE=$(du -h "$NEW_PATH" | cut -f1)
        echo "  Hash (first 16): $NEW_HASH"
        echo "  Build date: $NEW_DATE"
        echo "  Binary size: $NEW_SIZE"
    fi
else
    echo "ERROR: Installation completed but binary not found in PATH!"
    echo "Check that ~/.cargo/bin is in your PATH."
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Run 'copilot-usage' to start the setup wizard"
echo "  2. For Waybar integration, see: waybar-config-example.json"
echo ""
echo "════════════════════════════════════════════════════════════"