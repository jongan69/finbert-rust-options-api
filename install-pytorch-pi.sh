#!/bin/bash

# Install pre-built ARM64 PyTorch for Raspberry Pi
# This is faster than building from source

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

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "aarch64" && "$ARCH" != "armv7l" ]]; then
    print_error "Unsupported architecture: $ARCH"
    print_error "This script is for ARM64 (aarch64) or ARM32 (armv7l) Raspberry Pi"
    exit 1
fi

print_status "Installing PyTorch for $ARCH architecture..."

# Install system dependencies
print_status "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
    python3 \
    python3-pip \
    python3-dev \
    python3-venv \
    python3-full \
    libopenblas-dev \
    liblapack-dev \
    libgomp1 \
    libnuma-dev \
    pkg-config

# Create virtual environment
VENV_DIR="$HOME/pytorch-venv"
print_status "Creating Python virtual environment..."
python3 -m venv "$VENV_DIR"

# Activate virtual environment
print_status "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

# Upgrade pip in virtual environment
print_status "Upgrading pip..."
pip install --upgrade pip

# Install PyTorch using pip in virtual environment
print_status "Installing PyTorch via pip in virtual environment..."
if [[ "$ARCH" == "aarch64" ]]; then
    # For ARM64, try to find pre-built wheels
    print_status "Searching for ARM64 PyTorch wheels..."
    
    # Try installing from PyPI (may not work for all versions)
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
    
    # If that fails, try alternative sources
    if [[ $? -ne 0 ]]; then
        print_warning "PyPI installation failed, trying alternative sources..."
        
        # Try installing from community builds
        pip install --pre torch torchvision torchaudio --extra-index-url https://download.pytorch.org/whl/nightly/cpu
    fi
else
    # For ARM32, use different approach
    print_status "Installing PyTorch for ARM32..."
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
fi

# Verify installation
print_status "Verifying PyTorch installation..."
python3 -c "
import torch
print(f'PyTorch version: {torch.__version__}')
print(f'CPU available: {torch.backends.cpu.is_built()}')
print(f'Device: {torch.device(\"cpu\")}')
print('PyTorch installation successful!')
"

# Set up environment for Rust
print_status "Setting up environment for Rust..."

# Find PyTorch installation path in virtual environment
PYTORCH_PATH=$(python3 -c "import torch; print(torch.__file__)" | head -1)
PYTORCH_LIB_PATH=$(dirname "$PYTORCH_PATH")/lib

export LIBTORCH="$PYTORCH_LIB_PATH"
export LD_LIBRARY_PATH="$PYTORCH_LIB_PATH:$LD_LIBRARY_PATH"

# Add to .bashrc for persistence
echo "" >> ~/.bashrc
echo "# PyTorch ARM64 installation (Virtual Environment)" >> ~/.bashrc
echo "export PYTORCH_VENV=$VENV_DIR" >> ~/.bashrc
echo "export LIBTORCH=$LIBTORCH" >> ~/.bashrc
echo "export LD_LIBRARY_PATH=$PYTORCH_LIB_PATH:\$LD_LIBRARY_PATH" >> ~/.bashrc
echo "alias activate-pytorch='source \$PYTORCH_VENV/bin/activate'" >> ~/.bashrc

print_success "Environment variables added to ~/.bashrc"
print_warning "Please restart your terminal or run: source ~/.bashrc"

print_success "PyTorch installation completed!"
print_status "Virtual environment created at: $VENV_DIR"
print_status "To activate: source $VENV_DIR/bin/activate"
print_status "You can now build your Rust project with FinBERT support."
