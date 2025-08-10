#!/bin/bash

# FinBERT Rust Options API - Complete Installation and Run Script
# This script handles everything from PyTorch installation to running the API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

print_header() {
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}================================${NC}"
}

# Configuration
VENV_DIR="$HOME/pytorch-venv"
PROJECT_DIR="$(pwd)"
API_PORT=3000

print_header "FinBERT Rust Options API - Complete Setup"
echo ""
print_status "This script will install and run the FinBERT API on your system"
print_status "Project directory: $PROJECT_DIR"
print_status "Virtual environment: $VENV_DIR"
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_error "This script should not be run as root"
    print_error "Please run as a regular user"
    exit 1
fi

# Check architecture
ARCH=$(uname -m)
print_status "Detected architecture: $ARCH"

if [[ "$ARCH" != "aarch64" && "$ARCH" != "armv7l" && "$ARCH" != "x86_64" ]]; then
    print_error "Unsupported architecture: $ARCH"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "source ~/.cargo/env"
    exit 1
fi

print_success "Rust is installed: $(cargo --version)"

# Step 1: Install system dependencies
print_header "Step 1: Installing System Dependencies"

print_step "Updating package lists..."
sudo apt-get update

print_step "Installing system dependencies..."
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
    pkg-config \
    build-essential \
    curl \
    git

print_success "System dependencies installed"

# Step 2: Set up Python virtual environment
print_header "Step 2: Setting Up Python Virtual Environment"

if [[ -d "$VENV_DIR" ]]; then
    print_warning "Virtual environment already exists at $VENV_DIR"
    read -p "Do you want to recreate it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_step "Removing existing virtual environment..."
        rm -rf "$VENV_DIR"
    else
        print_status "Using existing virtual environment"
    fi
fi

if [[ ! -d "$VENV_DIR" ]]; then
    print_step "Creating Python virtual environment..."
    python3 -m venv "$VENV_DIR"
    print_success "Virtual environment created"
fi

# Step 3: Install PyTorch
print_header "Step 3: Installing PyTorch"

print_step "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

print_step "Upgrading pip..."
pip install --upgrade pip

print_step "Installing PyTorch..."
if [[ "$ARCH" == "aarch64" || "$ARCH" == "armv7l" ]]; then
    print_status "Installing ARM64 PyTorch..."
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
    
    # Check if headers are available
    print_step "Checking PyTorch headers..."
    if [[ ! -f "$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib\/include\/torch\/torch.h')" ]]; then
        print_warning "PyTorch headers not found. ARM64 wheels may not include development headers."
        print_status "Attempting to install PyTorch with development components..."
        pip uninstall torch torchvision torchaudio -y
        pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu --force-reinstall
        
        # Check again
        if [[ ! -f "$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib\/include\/torch\/torch.h')" ]]; then
            print_warning "PyTorch headers still missing. This may cause build issues."
            print_status "You may need to build PyTorch from source or use a different approach."
        else
            print_success "PyTorch headers found after reinstall"
        fi
    else
        print_success "PyTorch headers found"
    fi
else
    print_status "Installing x86_64 PyTorch..."
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
fi

print_step "Verifying PyTorch installation..."
python3 -c "
import torch
print(f'PyTorch version: {torch.__version__}')
print(f'Device: {torch.device(\"cpu\")}')
print('PyTorch installation successful!')
"

# Step 4: Set up environment variables
print_header "Step 4: Setting Up Environment Variables"

print_step "Finding PyTorch library path..."
PYTORCH_PATH=$(python3 -c "import torch; print(torch.__file__)" | head -1)
PYTORCH_LIB_PATH=$(dirname "$PYTORCH_PATH")/lib

export LIBTORCH="$PYTORCH_LIB_PATH"
export LD_LIBRARY_PATH="$PYTORCH_LIB_PATH:$LD_LIBRARY_PATH"

print_status "LIBTORCH: $LIBTORCH"
print_status "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Add to .bashrc for persistence
if ! grep -q "PYTORCH_VENV" ~/.bashrc; then
    print_step "Adding environment variables to ~/.bashrc..."
    echo "" >> ~/.bashrc
    echo "# PyTorch Virtual Environment" >> ~/.bashrc
    echo "export PYTORCH_VENV=$VENV_DIR" >> ~/.bashrc
    echo "export LIBTORCH=$LIBTORCH" >> ~/.bashrc
    echo "export LD_LIBRARY_PATH=$PYTORCH_LIB_PATH:\$LD_LIBRARY_PATH" >> ~/.bashrc
    echo "alias activate-pytorch='source \$PYTORCH_VENV/bin/activate'" >> ~/.bashrc
    print_success "Environment variables added to ~/.bashrc"
fi

# Step 5: Check for FinBERT model
print_header "Step 5: Setting Up FinBERT Model"

if [[ ! -d "finbert" ]]; then
    print_step "Cloning FinBERT model..."
    git clone https://huggingface.co/ProsusAI/finbert
    print_success "FinBERT model downloaded"
else
    print_status "FinBERT model already exists"
fi

# Step 6: Build Rust project
print_header "Step 6: Building Rust Project"

print_step "Cleaning previous builds..."
cargo clean

print_step "Building with release optimizations..."

# Try to build
if cargo build --release; then
    print_success "Rust project built successfully"
else
    print_warning "Build failed. This may be due to missing PyTorch headers."
    print_status "Attempting to fix PyTorch linking issues..."
    
    # Run the fix script
    if [[ -f "fix-pytorch-linking.sh" ]]; then
        print_step "Running PyTorch linking fix..."
        ./fix-pytorch-linking.sh
    else
        print_error "fix-pytorch-linking.sh not found. Manual intervention required."
        print_status "Please run the following commands manually:"
        echo "cargo clean"
        echo "rm -rf target/release/build/torch-sys-*"
        echo "source ~/pytorch-venv/bin/activate"
        echo "export LIBTORCH=\"\$(python3 -c \"import torch; print(torch.__file__)\" | head -1 | sed 's/__init__.py/lib/')\""
        echo "export LD_LIBRARY_PATH=\"\$LIBTORCH:\$LD_LIBRARY_PATH\""
        echo "cargo build --release"
        exit 1
    fi
fi

# Step 7: Set up environment file
print_header "Step 7: Setting Up Environment Configuration"

if [[ ! -f ".env" ]]; then
    print_step "Creating .env file..."
    cp .env.example .env 2>/dev/null || {
        print_step "Creating .env file from template..."
        cat > .env << EOF
# Alpaca API Configuration
APCA_API_KEY_ID=your_alpaca_api_key_here
APCA_API_SECRET_KEY=your_alpaca_secret_key_here
APCA_BASE_URL=https://api.alpaca.markets

# Logging
RUST_LOG=info
EOF
    }
    print_warning "Please update .env file with your Alpaca API credentials"
    print_warning "Edit .env and add your actual API keys before running the API"
else
    print_status ".env file already exists"
fi

# Step 8: Test the build
print_header "Step 8: Testing the Build"

print_step "Checking if binary was created..."
if [[ -f "target/release/finbert-rs" ]]; then
    print_success "Binary created successfully"
    ls -lh target/release/finbert-rs
else
    print_error "Binary not found. Build may have failed."
    exit 1
fi

# Step 9: Run the API
print_header "Step 9: Running the FinBERT API"

print_status "API will be available at: http://localhost:$API_PORT"
print_status "Health check: http://localhost:$API_PORT/health"
print_status "Analysis endpoint: http://localhost:$API_PORT/analyze"
echo ""

# Check if API keys are configured
if grep -q "your_alpaca_api_key_here" .env; then
    print_warning "API keys not configured in .env file"
    print_warning "The API will start but may not be able to fetch market data"
    echo ""
    read -p "Do you want to configure API keys now? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_step "Please edit the .env file with your API keys:"
        echo "nano .env"
        echo ""
        print_warning "After editing, restart the API"
        exit 0
    fi
fi

print_step "Starting the FinBERT API..."
print_status "Press Ctrl+C to stop the API"
echo ""

# Run the API
cargo run --release
