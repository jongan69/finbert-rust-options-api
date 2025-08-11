#!/bin/bash

# Simple install script that works with current environment
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo "🤖 FinBERT Rust Simple Install Script"
echo "======================================"

# Check if we're in the pytorch-venv
if [[ "$VIRTUAL_ENV" != *"pytorch-venv"* ]]; then
    print_error "Please activate the pytorch-venv first:"
    echo "source ~/pytorch-venv/bin/activate"
    exit 1
fi

print_success "Using current virtual environment: $VIRTUAL_ENV"

# Install compatible PyTorch version
print_status "Installing compatible PyTorch for Raspberry Pi..."
pip install torch==2.1.0 torchvision==0.16.0 torchaudio==2.1.0 --index-url https://download.pytorch.org/whl/cpu

# Source Rust environment
print_status "Setting up Rust environment..."
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
    export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v cargo >/dev/null 2>&1; then
    print_error "Cargo not found. Please install Rust first."
    exit 1
fi

# Set up PyTorch environment variables
print_status "Setting up PyTorch environment variables..."
TORCH_PATH=$(python -c "import torch; print(torch.__file__)" 2>/dev/null)
if [[ -z "$TORCH_PATH" ]]; then
    print_error "PyTorch not found in Python environment"
    exit 1
fi

export LIBTORCH="$(dirname "$TORCH_PATH")/lib"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
export LIBTORCH_INCLUDE="$(dirname "$TORCH_PATH")"
export LIBTORCH_USE_PYTORCH=1
export LIBTORCH_BYPASS_VERSION_CHECK=1
export LIBTORCH_STATIC=0
export LIBTORCH_CXX11_ABI=1
export TORCH_CUDA_VERSION=none
export CC=gcc
export CXX=g++

print_status "Environment variables set:"
print_status "  LIBTORCH=$LIBTORCH"
print_status "  LIBTORCH_INCLUDE=$LIBTORCH_INCLUDE"

# Build the project
print_status "Building project..."
cargo build --release

if [[ $? -eq 0 ]]; then
    print_success "✅ Build completed successfully!"
else
    print_error "❌ Build failed"
    exit 1
fi