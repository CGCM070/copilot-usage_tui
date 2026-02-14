#!/bin/bash

# Verify which version of copilot-usage you're running

echo "════════════════════════════════════════════════════════════"
echo "  Copilot-Usage Installation Check"
echo "════════════════════════════════════════════════════════════"
echo ""

if ! command -v copilot-usage &> /dev/null; then
    echo "ERROR: copilot-usage not found in PATH"
    echo ""
    echo "Install it first:"
    echo "  ./install.sh"
    exit 1
fi

BINARY_PATH=$(which copilot-usage)
VERSION=$(copilot-usage --version 2>/dev/null || echo "unknown")
HASH=$(sha256sum "$BINARY_PATH" | cut -d' ' -f1 | cut -c1-16)
BUILD_DATE=$(stat -c %y "$BINARY_PATH" 2>/dev/null | cut -d' ' -f1-2)
SIZE=$(du -h "$BINARY_PATH" | cut -f1)

echo "Installation details:"
echo "  Binary: $BINARY_PATH"
echo "  Version: $VERSION"
echo "  Hash (16): $HASH"
echo "  Built: $BUILD_DATE"
echo "  Size: $SIZE"
echo ""

# Check if it's in the expected location
if [[ "$BINARY_PATH" == *".cargo/bin/copilot-usage" ]]; then
    echo "✓ Correctly installed in ~/.cargo/bin/"
elif [[ "$BINARY_PATH" == *".local/bin/copilot-usage" ]]; then
    echo "⚠ Installed in ~/.local/bin/ (old method)"
    echo "  Consider reinstalling with ./install.sh"
else
    echo "⚠ Unexpected location: $BINARY_PATH"
fi

echo ""

# Compare with source directory build
if [ -f "target/release/copilot-usage" ]; then
    SOURCE_HASH=$(sha256sum target/release/copilot-usage | cut -d' ' -f1 | cut -c1-16)
    SOURCE_DATE=$(stat -c %y target/release/copilot-usage 2>/dev/null | cut -d' ' -f1-2)
    
    echo "Local build (target/release/):"
    echo "  Hash (16): $SOURCE_HASH"
    echo "  Built: $SOURCE_DATE"
    echo ""
    
    if [ "$HASH" = "$SOURCE_HASH" ]; then
        echo "✓ Installed binary matches local build"
    else
        echo "⚠ WARNING: Installed binary differs from local build!"
        echo "  Run ./install.sh to update"
    fi
else
    echo "No local build found in target/release/"
    echo "Run ./install.sh to build and install"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
