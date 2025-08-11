#!/bin/bash

# Build-only script that uses current environment
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo "🤖 FinBERT Build Script for Raspberry Pi"
echo "========================================"

# Check if we're in a virtual environment
if [[ -z "$VIRTUAL_ENV" ]]; then
    print_error "Please activate your virtual environment first:"
    echo "source ~/pytorch-venv/bin/activate"
    exit 1
fi

print_success "Using virtual environment: $VIRTUAL_ENV"

# Check PyTorch
TORCH_VERSION=$(python -c "import torch; print(torch.__version__)" 2>/dev/null || echo "none")
if [[ "$TORCH_VERSION" == "none" ]]; then
    print_error "PyTorch not found in current environment"
    exit 1
fi

print_status "Found PyTorch version: $TORCH_VERSION"

# Source Rust environment
print_status "Setting up Rust environment..."
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v cargo >/dev/null 2>&1; then
    print_error "Cargo not found. Please install Rust first."
    exit 1
fi

print_status "Found Cargo: $(cargo --version)"

# Set up PyTorch environment variables
print_status "Setting up PyTorch environment variables..."
TORCH_PATH=$(python -c "import torch; print(torch.__file__)" 2>/dev/null)
if [[ -z "$TORCH_PATH" ]]; then
    print_error "Could not locate PyTorch installation"
    exit 1
fi

# Set environment variables for torch-sys
export LIBTORCH="$(dirname "$TORCH_PATH")/lib"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
export LIBTORCH_INCLUDE="$(dirname "$TORCH_PATH")"
export LIBTORCH_USE_PYTORCH=1
export LIBTORCH_BYPASS_VERSION_CHECK=1
export LIBTORCH_STATIC=0

# ARM64 specific settings
export LIBTORCH_CXX11_ABI=1
export TORCH_CUDA_VERSION=none
export TORCH_CUDA_ARCH_LIST=""
export CMAKE_PREFIX_PATH="$LIBTORCH"
export CC=gcc
export CXX=g++

print_status "Environment configured:"
print_status "  LIBTORCH: $LIBTORCH"
print_status "  LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"
print_status "  LIBTORCH_CXX11_ABI: $LIBTORCH_CXX11_ABI"

# Check if libtorch directory exists
if [[ ! -d "$LIBTORCH" ]]; then
    print_error "LibTorch directory not found: $LIBTORCH"
    print_status "Checking alternative locations..."
    
    # Try alternative paths
    ALT_LIBTORCH="$(dirname "$TORCH_PATH")/../torch/lib"
    if [[ -d "$ALT_LIBTORCH" ]]; then
        export LIBTORCH="$ALT_LIBTORCH"
        export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
        export CMAKE_PREFIX_PATH="$LIBTORCH"
        print_status "Using alternative LibTorch path: $LIBTORCH"
    else
        print_error "Could not find LibTorch libraries"
        print_status "Available directories:"
        find "$(dirname "$TORCH_PATH")" -name "lib" -type d 2>/dev/null || true
        find "$(dirname "$TORCH_PATH")/.." -name "lib" -type d 2>/dev/null || true
        exit 1
    fi
fi

# Clean previous builds
print_status "Cleaning previous build..."
cargo clean 2>/dev/null || true

# Build with single job to avoid memory issues on Pi
export CARGO_BUILD_JOBS=1

print_status "Building project (this may take 10-30 minutes)..."
print_status "Using single-threaded build to avoid memory issues..."

# Build the project
if cargo build --release; then
    print_success "✅ Build completed successfully!"
    print_status "Binary location: target/release/finbert-rs"
    
    if [[ -f "target/release/finbert-rs" ]]; then
        print_success "✅ Binary verified and ready to run!"
    else
        print_error "❌ Binary not found after build"
    fi
else
    print_error "❌ Build failed"
    print_status "Common issues on Raspberry Pi:"
    print_status "1. Insufficient memory - try increasing swap space"
    print_status "2. Missing system dependencies - ensure build-essential is installed"
    print_status "3. PyTorch/LibTorch version mismatch - may need PyTorch 2.1.0"
    exit 1
fi