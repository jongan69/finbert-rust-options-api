#!/bin/bash

# Quick fix for externally managed Python environment on Raspberry Pi

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

print_status "Fixing externally managed Python environment..."

# Install required packages
print_status "Installing Python virtual environment support..."
sudo apt-get update
sudo apt-get install -y python3-venv python3-full

# Create virtual environment
VENV_DIR="$HOME/pytorch-venv"
print_status "Creating virtual environment at $VENV_DIR..."
python3 -m venv "$VENV_DIR"

# Activate and install PyTorch
print_status "Activating virtual environment and installing PyTorch..."
source "$VENV_DIR/bin/activate"
pip install --upgrade pip

# Install PyTorch
print_status "Installing PyTorch..."
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu

# Verify installation
print_status "Verifying installation..."
python3 -c "
import torch
print(f'PyTorch version: {torch.__version__}')
print(f'Device: {torch.device(\"cpu\")}')
print(f'CUDA available: {torch.cuda.is_available()}')
print('PyTorch installation successful!')
"

# Set up environment variables
print_status "Setting up environment variables..."
PYTORCH_PATH=$(python3 -c "import torch; print(torch.__file__)" | head -1)
PYTORCH_LIB_PATH=$(dirname "$PYTORCH_PATH")/lib

# Add to .bashrc
echo "" >> ~/.bashrc
echo "# PyTorch Virtual Environment" >> ~/.bashrc
echo "export PYTORCH_VENV=$VENV_DIR" >> ~/.bashrc
echo "export LIBTORCH=$PYTORCH_LIB_PATH" >> ~/.bashrc
echo "export LD_LIBRARY_PATH=$PYTORCH_LIB_PATH:\$LD_LIBRARY_PATH" >> ~/.bashrc
echo "alias activate-pytorch='source \$PYTORCH_VENV/bin/activate'" >> ~/.bashrc

print_success "Fix completed!"
print_status "Virtual environment: $VENV_DIR"
print_status "To activate: source $VENV_DIR/bin/activate"
print_status "To build Rust project: cargo build --release"
