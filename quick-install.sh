#!/bin/bash

# Quick install script for rapid development
# Skips pre-commit checks for faster iteration

set -e

echo "Quick install (skipping checks)..."

# Eliminar config para que siempre muestre wizard
rm -f ~/.config/copilot-usage/config.toml

# Install using cargo install
cargo install --path . --force --quiet

# Verify
if command -v copilot-usage &> /dev/null; then
    NEW_PATH=$(which copilot-usage)
    NEW_HASH=$(sha256sum "$NEW_PATH" | cut -d' ' -f1 | cut -c1-16)
    NEW_DATE=$(date +"%Y-%m-%d %H:%M:%S")
    echo "✓ Installed: $NEW_PATH"
    echo "  Hash: $NEW_HASH"
    echo "  Time: $NEW_DATE"
    echo ""
    echo "→ Run 'copilot-usage' to configure (wizard)"
else
    echo "ERROR: Installation failed!"
    exit 1
fi
