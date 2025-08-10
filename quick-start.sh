#!/bin/bash

# FinBERT Rust Options API - Quick Start Script
# For users who want to get the API running quickly

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

echo -e "${GREEN}🚀 FinBERT Rust Options API - Quick Start${NC}"
echo ""

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if virtual environment exists
if [[ ! -d "$HOME/pytorch-venv" ]]; then
    print_warning "PyTorch virtual environment not found"
    print_status "Running full installation script..."
    chmod +x install-and-run.sh
    ./install-and-run.sh
    exit 0
fi

# Quick setup for existing environment
print_status "Setting up environment..."

# Activate virtual environment
source ~/pytorch-venv/bin/activate

# Set environment variables
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"

# Check if FinBERT model exists
if [[ ! -d "finbert" ]]; then
    print_status "Downloading FinBERT model..."
    git clone https://huggingface.co/ProsusAI/finbert
fi

# Build the project
print_status "Building project..."
cargo build --release

# Check if .env exists
if [[ ! -f ".env" ]]; then
    print_warning "Creating .env file..."
    cat > .env << EOF
# Alpaca API Configuration
APCA_API_KEY_ID=your_alpaca_api_key_here
APCA_API_SECRET_KEY=your_alpaca_secret_key_here
APCA_BASE_URL=https://api.alpaca.markets

# Logging
RUST_LOG=info
EOF
    print_warning "Please update .env with your Alpaca API credentials"
fi

# Run the API
print_success "Starting FinBERT API..."
print_status "API will be available at: http://localhost:3000"
print_status "Press Ctrl+C to stop"
echo ""

cargo run --release
