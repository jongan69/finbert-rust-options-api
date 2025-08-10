#!/bin/bash

# Build PyTorch from source for Raspberry Pi ARM64
# This script builds PyTorch libraries compatible with ARM64 architecture

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

# Check if running on Raspberry Pi
if [[ ! -f /proc/cpuinfo ]] || ! grep -q "Raspberry Pi" /proc/cpuinfo; then
    print_warning "This script is designed for Raspberry Pi. Are you sure you want to continue?"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "aarch64" && "$ARCH" != "armv7l" ]]; then
    print_error "Unsupported architecture: $ARCH"
    print_error "This script is for ARM64 (aarch64) or ARM32 (armv7l) Raspberry Pi"
    exit 1
fi

print_status "Building PyTorch for $ARCH architecture..."

# Install system dependencies
print_status "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    git \
    python3 \
    python3-pip \
    python3-dev \
    libopenblas-dev \
    liblapack-dev \
    libjpeg-dev \
    libpng-dev \
    libffi-dev \
    libssl-dev \
    libxml2-dev \
    libxslt-dev \
    zlib1g-dev \
    libgomp1 \
    libnuma-dev \
    pkg-config \
    ninja-build

# Set environment variables for ARM build
export USE_CUDA=0
export USE_DISTRIBUTED=0
export USE_MKLDNN=0
export USE_NNPACK=0
export USE_QNNPACK=0
export USE_PYTORCH_QNNI=0
export USE_XNNPACK=0
export USE_NUMPY=1
export USE_OPENMP=1
export BLAS=OpenBLAS
export USE_BLAS=1
export USE_LAPACK=1

# Create build directory
BUILD_DIR="$HOME/pytorch-build"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# Clone PyTorch repository
if [[ ! -d "pytorch" ]]; then
    print_status "Cloning PyTorch repository..."
    git clone --recursive https://github.com/pytorch/pytorch.git
    cd pytorch
else
    print_status "PyTorch repository already exists, updating..."
    cd pytorch
    git pull
    git submodule update --init --recursive
fi

# Checkout stable version (adjust as needed)
print_status "Checking out stable version..."
git checkout v2.1.0

# Configure build
print_status "Configuring PyTorch build..."
python3 setup.py clean

# Build PyTorch (CPU only)
print_status "Building PyTorch (this will take 2-4 hours)..."
print_warning "This is a CPU-intensive process. Make sure your Pi has adequate cooling!"
print_warning "Consider using a fan or heatsink to prevent thermal throttling."

# Build with reduced parallelism to avoid memory issues
export MAX_JOBS=2
python3 setup.py build

# Install PyTorch
print_status "Installing PyTorch..."
sudo python3 setup.py install

# Verify installation
print_status "Verifying PyTorch installation..."
python3 -c "import torch; print(f'PyTorch version: {torch.__version__}'); print(f'CPU available: {torch.backends.cpu.is_built()}')"

print_success "PyTorch built and installed successfully!"

# Set up environment for Rust
print_status "Setting up environment for Rust..."
export LIBTORCH="$BUILD_DIR/pytorch"
export LD_LIBRARY_PATH="$LIBTORCH/lib:$LD_LIBRARY_PATH"

# Add to .bashrc for persistence
echo "" >> ~/.bashrc
echo "# PyTorch ARM64 build" >> ~/.bashrc
echo "export LIBTORCH=$LIBTORCH" >> ~/.bashrc
echo "export LD_LIBRARY_PATH=$LIBTORCH/lib:\$LD_LIBRARY_PATH" >> ~/.bashrc

print_success "Environment variables added to ~/.bashrc"
print_warning "Please restart your terminal or run: source ~/.bashrc"

print_success "PyTorch build completed! You can now build your Rust project."
