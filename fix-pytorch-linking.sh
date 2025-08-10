#!/bin/bash

# Fix PyTorch linking for ARM64/Raspberry Pi
# This script resolves the "skipping incompatible" linking errors

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "🔧 Fixing PyTorch linking for ARM64..."

# Check if virtual environment exists
if [[ ! -d "$HOME/pytorch-venv" ]]; then
    print_error "PyTorch virtual environment not found at $HOME/pytorch-venv"
    print_error "Please run ./fix-pytorch-pi.sh first"
    exit 1
fi

# Clean build cache completely
print_status "🧹 Cleaning build cache..."
cargo clean
rm -rf target/release/build/torch-sys-* 2>/dev/null || true
rm -rf ~/.cargo/registry/cache/*/torch-sys* 2>/dev/null || true

# Activate virtual environment
print_status "🔌 Activating virtual environment..."
source ~/pytorch-venv/bin/activate

# Set environment variables correctly
print_status "⚙️ Setting environment variables..."
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"

print_status "📁 LIBTORCH: $LIBTORCH"
print_status "🔗 LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Verify ARM64 libraries
print_status "🔍 Checking library architecture..."
if [[ -d "$LIBTORCH" ]]; then
    for lib in "$LIBTORCH"/libtorch*.so; do
        if [[ -f "$lib" ]]; then
            arch=$(file "$lib" | grep -o "ARM aarch64\|x86-64\|ELF 64-bit")
            if [[ "$arch" == "ARM aarch64" ]]; then
                print_success "$(basename "$lib"): $arch"
            else
                print_warning "$(basename "$lib"): $arch (may cause issues)"
            fi
        fi
    done
else
    print_error "PyTorch lib directory not found: $LIBTORCH"
    exit 1
fi

# Check for conflicting x86_64 libraries
print_status "🔍 Checking for conflicting x86_64 libraries..."
conflicting_libs=$(find /usr -name "libtorch*.so" 2>/dev/null | head -5)
if [[ -n "$conflicting_libs" ]]; then
    print_warning "Found potentially conflicting system libraries:"
    echo "$conflicting_libs"
    print_warning "Make sure your virtual environment libraries are used"
fi

# Build with correct environment
print_status "⚡️ Building with ARM64 PyTorch..."
cargo build --release

print_success "Build completed successfully!"
print_status "You can now run: cargo run"
