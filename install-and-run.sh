#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="finbert-rs"
VENV_PATH="$HOME/pytorch-venv"
PYTORCH_VERSION="2.1.2"
RUST_BERT_VERSION="0.23.0"

# Function to print colored output
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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check system requirements
check_requirements() {
    print_step "Checking system requirements..."
    
    # Check if running on ARM64
    if [[ "$(uname -m)" != "aarch64" ]]; then
        print_warning "This script is optimized for ARM64 (Raspberry Pi). You're running on $(uname -m)"
    fi
    
    # Check Rust
    if ! command_exists cargo; then
        print_error "Rust/Cargo not found. Please install Rust first:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Check Python
    if ! command_exists python3; then
        print_error "Python3 not found. Please install Python 3.8+ first."
        exit 1
    fi
    
    # Check Python version
    local python_version=$(python3 --version 2>&1 | grep -oP '\d+\.\d+' | head -1)
    print_status "Detected Python version: $python_version"
    
    # Convert version to comparable numbers (e.g., 3.11 -> 311, 3.8 -> 308)
    local major_minor=$(echo "$python_version" | sed 's/\.//')
    local required_version="38"
    
    if [[ "$major_minor" -lt "$required_version" ]]; then
        print_error "Python 3.8+ required, found $python_version"
        exit 1
    fi
    
    # Check memory
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    if [[ "$mem_gb" -lt 2 ]]; then
        print_warning "Low memory detected (${mem_gb}GB). 4GB+ recommended for optimal performance."
    fi
    
    print_success "System requirements check passed"
}

# Function to setup Python virtual environment
setup_python_env() {
    print_step "Setting up Python virtual environment..."
    
    if [[ ! -d "$VENV_PATH" ]]; then
        print_status "Creating virtual environment at $VENV_PATH"
        python3 -m venv "$VENV_PATH"
    fi
    
    print_status "Activating virtual environment..."
    source "$VENV_PATH/bin/activate"
    
    # Upgrade pip
    pip install --upgrade pip
    
    print_success "Python environment ready"
}

# Function to install PyTorch with compatibility fixes
install_pytorch() {
    print_step "Installing PyTorch with compatibility fixes..."
    
    # Check current PyTorch version
    local current_version=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
    print_status "Current PyTorch version: $current_version"
    
    # Fix NumPy version first
    local numpy_version=$(python3 -c "import numpy; print(numpy.__version__)" 2>/dev/null || echo "not installed")
    if [[ "$numpy_version" == "2."* ]]; then
        print_warning "NumPy 2.x detected, downgrading to 1.x for compatibility"
        pip uninstall numpy -y
        pip install "numpy<2.0"
    fi
    
    # Install compatible PyTorch version
    if [[ "$current_version" != "$PYTORCH_VERSION" ]]; then
        print_status "Installing PyTorch $PYTORCH_VERSION for torch-sys compatibility..."
        pip uninstall torch torchvision torchaudio -y 2>/dev/null || true
        pip install "torch==$PYTORCH_VERSION" "torchvision==0.16.2" "torchaudio==$PYTORCH_VERSION" --index-url https://download.pytorch.org/whl/cpu
        
        # Verify installation
        local new_version=$(python3 -c "import torch; print(torch.__version__)")
        if [[ "$new_version" == "$PYTORCH_VERSION" ]]; then
            print_success "PyTorch $PYTORCH_VERSION installed successfully"
        else
            print_error "Failed to install PyTorch $PYTORCH_VERSION"
            exit 1
        fi
    else
        print_success "PyTorch $PYTORCH_VERSION already installed"
    fi
}

# Function to setup environment variables
setup_environment() {
    print_step "Setting up environment variables..."
    
    # Get PyTorch path
    local torch_path=$(python3 -c "import torch; print(torch.__file__)" 2>/dev/null)
    if [[ -z "$torch_path" ]]; then
        print_error "PyTorch not found. Please ensure PyTorch is installed."
        exit 1
    fi
    
    # Set environment variables
    export LIBTORCH="$(echo "$torch_path" | sed 's/__init__.py/lib/')"
    export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
    export LIBTORCH_INCLUDE="$(echo "$torch_path" | sed 's/__init__.py//')"
    export LIBTORCH_USE_PYTORCH=1
    export LIBTORCH_CXX11_ABI=0
    export LIBTORCH_STATIC=0
    
    print_status "Environment variables set:"
    print_status "  LIBTORCH: $LIBTORCH"
    print_status "  LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"
    print_status "  LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
    
    print_success "Environment setup complete"
}

# Function to download FinBERT model
download_finbert_model() {
    print_step "Setting up FinBERT model..."
    
    if [[ ! -d "finbert" ]]; then
        print_status "Downloading FinBERT model from Hugging Face..."
        git clone https://huggingface.co/ProsusAI/finbert finbert
        if [[ $? -eq 0 ]]; then
            print_success "FinBERT model downloaded successfully"
        else
            print_error "Failed to download FinBERT model"
            exit 1
        fi
    else
        print_success "FinBERT model already exists"
    fi
}

# Function to setup environment file
setup_env_file() {
    print_step "Setting up environment configuration..."
    
    if [[ ! -f ".env" ]]; then
        print_status "Creating .env file from template..."
        cp env.example .env
        print_warning "Please edit .env file with your Alpaca API credentials:"
        echo "  APCA_API_KEY_ID=your_api_key_here"
        echo "  APCA_API_SECRET_KEY=your_secret_key_here"
        echo "  APCA_BASE_URL=https://paper-api.alpaca.markets"
    else
        print_success ".env file already exists"
    fi
}

# Function to clean build cache
clean_build_cache() {
    print_step "Cleaning build cache..."
    
    cargo clean
    rm -rf target/release/build/torch-sys-*
    rm -rf ~/.cargo/registry/cache/*/torch-sys*
    
    print_success "Build cache cleaned"
}

# Function to build the project
build_project() {
    print_step "Building FinBERT Rust application..."
    
    # Set build jobs based on available memory
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    if [[ "$mem_gb" -lt 4 ]]; then
        export CARGO_BUILD_JOBS=1
        print_warning "Using single core build due to limited memory"
    else
        export CARGO_BUILD_JOBS=$(nproc)
        print_status "Using $(nproc) cores for build"
    fi
    
    print_status "Building with release profile..."
    if cargo build --release; then
        print_success "Build completed successfully!"
    else
        print_error "Build failed. Check the error messages above."
        exit 1
    fi
}

# Function to run the application
run_application() {
    print_step "Starting FinBERT Rust application..."
    
    # Check if binary exists
    if [[ ! -f "target/release/$PROJECT_NAME" ]]; then
        print_error "Binary not found. Please build the project first."
        exit 1
    fi
    
    print_success "🚀 Starting FinBERT Rust API..."
    print_status "📊 Analysis endpoint: http://127.0.0.1:3000/analyze"
    print_status "❤️  Health check: http://127.0.0.1:3000/health"
    print_status "📈 Metrics: http://127.0.0.1:3000/metrics"
    echo ""
    
    # Run the application
    exec cargo run --release
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --setup-only     Only setup environment, don't run"
    echo "  --build-only     Only build the project, don't run"
    echo "  --run-only       Only run the application (assumes setup is complete)"
    echo "  --clean          Clean build cache before building"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0               # Full setup and run"
    echo "  $0 --setup-only  # Setup environment only"
    echo "  $0 --run-only    # Run existing build"
    echo ""
}

# Function to handle signals
cleanup() {
    print_status "Shutting down..."
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    echo -e "${CYAN}🤖 FinBERT Rust Options API - Install & Run Script${NC}"
    echo -e "${CYAN}================================================${NC}"
    echo ""
    
    # Parse command line arguments
    SETUP_ONLY=false
    BUILD_ONLY=false
    RUN_ONLY=false
    CLEAN_BUILD=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --setup-only)
                SETUP_ONLY=true
                shift
                ;;
            --build-only)
                BUILD_ONLY=true
                shift
                ;;
            --run-only)
                RUN_ONLY=true
                shift
                ;;
            --clean)
                CLEAN_BUILD=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Change to script directory
    cd "$SCRIPT_DIR"
    
    if [[ "$RUN_ONLY" == true ]]; then
        print_step "Run-only mode selected"
        run_application
    elif [[ "$BUILD_ONLY" == true ]]; then
        print_step "Build-only mode selected"
        check_requirements
        setup_python_env
        install_pytorch
        setup_environment
        download_finbert_model
        setup_env_file
        if [[ "$CLEAN_BUILD" == true ]]; then
            clean_build_cache
        fi
        build_project
    elif [[ "$SETUP_ONLY" == true ]]; then
        print_step "Setup-only mode selected"
        check_requirements
        setup_python_env
        install_pytorch
        setup_environment
        download_finbert_model
        setup_env_file
        print_success "Setup completed successfully!"
        echo ""
        print_status "Next steps:"
        print_status "1. Edit .env file with your Alpaca API credentials"
        print_status "2. Run: $0 --build-only"
        print_status "3. Run: $0 --run-only"
    else
        # Full setup and run
        print_step "Full setup and run mode"
        check_requirements
        setup_python_env
        install_pytorch
        setup_environment
        download_finbert_model
        setup_env_file
        if [[ "$CLEAN_BUILD" == true ]]; then
            clean_build_cache
        fi
        build_project
        run_application
    fi
}

# Run main function
main "$@"
