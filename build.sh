#!/bin/bash
# CEOClaw Build Script (Cross-Platform)

set -e

echo "========================================"
echo "  CEOClaw Build Script"
echo "========================================"
echo ""

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGW;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

echo "🖥️  Detected platform: $MACHINE"
echo ""

# Check Rust
echo "[1/5] Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: Rust is not installed!"
    echo "   Please install Rust from: https://rustup.rs/"
    exit 1
fi
echo "✓ $(cargo --version)"
echo ""

# Check Node.js
echo "[2/5] Checking Node.js installation..."
if ! command -v node &> /dev/null; then
    echo "❌ ERROR: Node.js is not installed!"
    echo "   Please install Node.js from: https://nodejs.org/"
    exit 1
fi
echo "✓ Node.js $(node --version)"
echo ""

# Change to script directory
cd "$(dirname "$0")"
echo "📂 Working directory: $(pwd)"
echo ""

# Install dependencies
echo "[3/5] Installing dependencies..."
npm install
echo "✓ Dependencies installed"
echo ""

# Build
echo "[4/5] Building CEOClaw..."
echo "   This may take several minutes..."
npm run tauri:build
echo "✓ Build successful"
echo ""

# Show results
echo "[5/5] Build artifacts:"
BUNDLE_DIR="src-tauri/target/release/bundle"
if [ -d "$BUNDLE_DIR" ]; then
    find "$BUNDLE_DIR" -type f \( -name "*.exe" -o -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" \) | while read -r artifact; do
        echo "  📦 $(basename "$artifact")"
        echo "     $artifact"
    done
fi

if [ -f "src-tauri/target/release/ceo-claw" ]; then
    echo "  📦 ceo-claw (portable)"
    echo "     src-tauri/target/release/ceo-claw"
fi

echo ""
echo "========================================"
echo "  Build completed successfully!"
echo "========================================"