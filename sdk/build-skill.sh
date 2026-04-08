#!/bin/bash
# Build script for CEOClaw WASM skills

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    print_error "wasm-pack is not installed. Please install it first:"
    echo "  cargo install wasm-pack"
    exit 1
fi

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    print_error "rustup is not installed. Please install Rust first."
    exit 1
fi

# Add WASM target
print_info "Adding WASM target..."
rustup target add wasm32-unknown-unknown

# Parse arguments
BUILD_TYPE="release"
TARGET_DIR="pkg"

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --target-dir)
            TARGET_DIR="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--debug] [--target-dir DIR]"
            echo "  --debug         Build in debug mode (default: release)"
            echo "  --target-dir    Output directory (default: pkg)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build the WASM module
print_info "Building WASM module in $BUILD_TYPE mode..."

BUILD_FLAG=""
if [ "$BUILD_TYPE" = "debug" ]; then
    BUILD_FLAG="--dev"
else
    BUILD_FLAG="--release"
fi

wasm-pack build \
    $BUILD_FLAG \
    --target web \
    --out-dir "$TARGET_DIR"

# Copy manifest to target directory
if [ -f "manifest.json" ]; then
    print_info "Copying manifest.json..."
    cp manifest.json "$TARGET_DIR/"
else
    print_warn "manifest.json not found. Skills require a manifest.json file."
fi

print_info "Build complete! Output in: $TARGET_DIR"
print_info ""
print_info "To install this skill in CEOClaw:"
print_info "  1. Open CEOClaw"
print_info "  2. Go to Settings > Skills"
print_info "  3. Click 'Install Skill' and select the manifest.json file from $TARGET_DIR"
print_info ""
print_info "Or use the CLI:"
print_info "  ceoclaw skill install --path $TARGET_DIR/manifest.json"