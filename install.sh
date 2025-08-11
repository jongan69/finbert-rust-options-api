#!/bin/bash

# Preserve existing PATH and add standard locations
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:$HOME/.cargo/bin:${PATH}"

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

# Set proper home directory - use actual user's home if running with sudo
if [[ -n "$SUDO_USER" ]]; then
    ACTUAL_HOME=$(eval echo "~$SUDO_USER")
    VENV_PATH="$ACTUAL_HOME/pytorch-venv"
else
    VENV_PATH="$HOME/pytorch-venv"
fi

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
    local arch_info="$(uname -m 2>/dev/null || echo "unknown")"
    if [[ "$arch_info" != "aarch64" ]]; then
        print_warning "This script is optimized for ARM64 (Raspberry Pi). You're running on $arch_info"
    fi
    
    # Check for required system packages on Raspberry Pi
    if [[ "$arch_info" == "aarch64" ]]; then
        print_status "Checking required system packages for Raspberry Pi..."
        missing_packages=()
        
        if ! dpkg -l | grep -q "build-essential"; then
            missing_packages+=("build-essential")
        fi
        if ! dpkg -l | grep -q "pkg-config"; then
            missing_packages+=("pkg-config")
        fi
        if ! dpkg -l | grep -q "libssl-dev"; then
            missing_packages+=("libssl-dev")
        fi
        if ! dpkg -l | grep -q "cmake"; then
            missing_packages+=("cmake")
        fi
        
        if [[ ${#missing_packages[@]} -gt 0 ]]; then
            print_warning "Missing required packages: ${missing_packages[*]}"
            print_status "Please install them with: sudo apt update && sudo apt install ${missing_packages[*]}"
            read -p "Install missing packages now? (y/N): " install_deps
            if [[ "$install_deps" =~ ^[Yy]$ ]]; then
                sudo apt update && sudo apt install "${missing_packages[@]}"
            else
                print_error "Required packages not installed. Exiting..."
                exit 1
            fi
        fi
    fi
    
    # Check if running as root and warn about Rust installation
    if [[ "$EUID" -eq 0 ]]; then
        print_warning "Running as root. Checking for Rust installation..."
        # When running as sudo, check the actual user's Rust installation
        local actual_user="${SUDO_USER:-$USER}"
        local actual_home
        if [[ -n "$SUDO_USER" ]]; then
            actual_home=$(eval echo "~$SUDO_USER")
        else
            actual_home="$HOME"
        fi
        
        if [[ -f "$actual_home/.cargo/env" ]]; then
            print_status "Found Rust installation for user $actual_user"
            source "$actual_home/.cargo/env"
            export PATH="$actual_home/.cargo/bin:$PATH"
        fi
    fi
    
    # Check Rust
    if ! command_exists cargo; then
        print_status "Attempting to source Rust environment..."
        if [[ -f "$HOME/.cargo/env" ]]; then
            source "$HOME/.cargo/env"
            export PATH="$HOME/.cargo/bin:$PATH"
        fi
        
        if ! command_exists cargo; then
            if [[ "$EUID" -eq 0 ]]; then
                print_error "Rust not found when running as root."
                print_status "Please run this script without sudo:"
                print_status "  ./install.sh"
                print_status "The script will prompt for sudo when needed for system packages."
            else
                print_error "Rust/Cargo not found. Please install Rust first:"
                echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                echo "Then run: source ~/.cargo/env"
            fi
            exit 1
        else
            print_success "Rust environment loaded successfully"
        fi
    fi
    
    # Check Python
    if ! command_exists python3; then
        print_error "Python3 not found. Please install Python 3.8+ first."
        exit 1
    fi
    
    # Check Python version
    local python_version=$(python3 --version 2>&1 | sed 's/Python //' | cut -d. -f1,2)
    print_status "Detected Python version: $python_version"
    
    # Convert version to comparable numbers (e.g., 3.11 -> 311, 3.8 -> 308)
    local major_minor=$(echo "$python_version" | sed 's/\.//')
    local required_version="38"
    
    if [[ "$major_minor" -lt "$required_version" ]]; then
        print_error "Python 3.8+ required, found $python_version"
        exit 1
    fi
    
    # Check memory
    if command_exists free; then
        local mem_gb=$(free -g | awk '/^Mem:/{print $2}' 2>/dev/null || echo "0")
        if [[ "$mem_gb" -lt 2 ]]; then
            print_warning "Low memory detected (${mem_gb}GB). 4GB+ recommended for optimal performance."
        fi
    else
        print_warning "Could not detect memory size. 4GB+ recommended for optimal performance."
    fi
    
    print_success "System requirements check passed"
}

# Function to setup Python virtual environment
setup_python_env() {
    print_step "Setting up Python virtual environment..."
    
    # Check if we're already in the target virtual environment
    if [[ -n "$VIRTUAL_ENV" && "$VIRTUAL_ENV" == "$VENV_PATH" ]]; then
        print_status "Already in target virtual environment: $VENV_PATH"
    else
        if [[ ! -d "$VENV_PATH" ]]; then
            print_status "Creating virtual environment at $VENV_PATH"
            # Find python3 command
            local python_cmd
            if command_exists python3; then
                python_cmd="python3"
            elif command_exists python; then
                python_cmd="python"
            else
                print_error "Python not found. Please ensure Python 3.8+ is installed."
                exit 1
            fi
            
            "$python_cmd" -m venv "$VENV_PATH"
        fi
        
        print_status "Activating virtual environment..."
        source "$VENV_PATH/bin/activate"
    fi
    
    # Upgrade pip
    pip install --upgrade pip
    
    # Install PyTorch based on architecture
    local arch_info="$(uname -m 2>/dev/null || echo "unknown")"
    if [[ "$arch_info" == "aarch64" ]]; then
        print_status "Installing compatible PyTorch for ARM64 (Raspberry Pi)..."
        # Use PyTorch 2.1.0 which is compatible with torch-sys 0.17.0
        pip install torch==2.1.0 torchvision==0.16.0 torchaudio==2.1.0 --index-url https://download.pytorch.org/whl/cpu
    else
        print_status "Installing compatible PyTorch for $arch_info..."
        pip install torch==2.1.0 torchvision==0.16.0 torchaudio==2.1.0 --index-url https://download.pytorch.org/whl/cpu
    fi
    
    print_success "Python environment ready"
}

# Function to clean existing PyTorch installation
clean_pytorch() {
    print_step "Cleaning existing PyTorch installation..."
    
    print_warning "Removing existing PyTorch installation to ensure clean build"
    
    # Check if we're currently in the target virtual environment
    local should_clean_venv=true
    if [[ -n "$VIRTUAL_ENV" && "$VIRTUAL_ENV" == "$VENV_PATH" ]]; then
        print_status "Currently in target virtual environment, will clean PyTorch packages only"
        should_clean_venv=false
        # Clean PyTorch packages from current environment
        pip uninstall -y torch torchvision torchaudio 2>/dev/null || true
    else
        # Deactivate virtual environment if active and different from target
        if [[ -n "$VIRTUAL_ENV" ]]; then
            print_status "Deactivating virtual environment"
            if command_exists deactivate; then
                deactivate
            else
                unset VIRTUAL_ENV
                export PATH="${PATH//$VIRTUAL_ENV\/bin:/}"
            fi
        fi
        
        # Remove existing virtual environment
        if [[ -d "$VENV_PATH" ]]; then
            print_status "Removing existing virtual environment"
            /bin/rm -rf "$VENV_PATH" 2>/dev/null || true
        fi
    fi
    
    # Clean cargo cache
    print_status "Cleaning cargo cache"
    if command_exists cargo; then
        cargo clean 2>/dev/null || true
        /bin/rm -rf target/ 2>/dev/null || true
        /bin/rm -rf ~/.cargo/registry/cache/*/torch-sys* 2>/dev/null || true
    else
        print_warning "Cargo not found in PATH, skipping cargo clean"
    fi
    
    print_success "Cleanup completed"
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

# Function to build the project
build_project() {
    print_step "Building FinBERT Rust application..."
    
    # Set build jobs based on available memory
    if command_exists free; then
        local mem_gb=$(free -g | awk '/^Mem:/{print $2}' 2>/dev/null || echo "0")
        if [[ "$mem_gb" -lt 4 ]]; then
            export CARGO_BUILD_JOBS=1
            print_warning "Using single core build due to limited memory (${mem_gb}GB)"
            # Add swap usage for low memory systems
            if [[ "$mem_gb" -lt 2 ]]; then
                print_warning "Very low memory detected. Consider increasing swap space."
                print_status "You can increase swap with: sudo dphys-swapfile swapoff && sudo sed -i 's/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=2048/' /etc/dphys-swapfile && sudo dphys-swapfile setup && sudo dphys-swapfile swapon"
            fi
        else
            # Limit concurrent jobs on ARM64 to prevent memory issues
            local arch_info="$(uname -m 2>/dev/null || echo "unknown")"
            if [[ "$arch_info" == "aarch64" ]]; then
                export CARGO_BUILD_JOBS=2
                print_status "Using 2 cores for ARM64 build (${mem_gb}GB available)"
            else
                local cpu_count="$(nproc 2>/dev/null || echo "4")"
                export CARGO_BUILD_JOBS="$cpu_count"
                print_status "Using $cpu_count cores for build (${mem_gb}GB available)"
            fi
        fi
    else
        export CARGO_BUILD_JOBS=1
        print_warning "Using single core build (could not detect memory)"
    fi
    
    print_status "Building with release profile..."
    
    # Setup PyTorch environment
    setup_pytorch_environment
    
    # Check if cargo is available
    if ! command_exists cargo; then
        print_error "Cargo not found. Please ensure Rust is properly installed and in PATH."
        print_status "Try running: source ~/.cargo/env"
        exit 1
    fi
    
    if cargo build --release; then
        print_success "Build completed successfully!"
        print_success "✅ Build completed successfully!"
    else
        print_error "Build failed. Check the error messages above."
        exit 1
    fi
}

# Function to setup PyTorch environment
setup_pytorch_environment() {
    print_status "Setting up PyTorch environment..."
    
    # Get PyTorch path
    local torch_path=$(python3 -c "import torch; print(torch.__file__)" 2>/dev/null)
    if [[ -z "$torch_path" ]]; then
        print_error "PyTorch not found. Please ensure PyTorch is installed."
        exit 1
    fi
    
    # Set environment variables
    export LIBTORCH="$(echo "$torch_path" | /bin/sed 's/__init__.py/lib/' 2>/dev/null || echo "$torch_path/../lib")"
    export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
    export LIBTORCH_INCLUDE="$(echo "$torch_path" | /bin/sed 's/__init__.py//' 2>/dev/null || echo "$torch_path/..")"
    export LIBTORCH_USE_PYTORCH=1
    export LIBTORCH_BYPASS_VERSION_CHECK=1
    export LIBTORCH_STATIC=0
    
    # Set C++ ABI compatibility for ARM64/Raspberry Pi
    local arch_info="$(uname -m 2>/dev/null || echo "unknown")"
    if [[ "$arch_info" == "aarch64" ]]; then
        export LIBTORCH_CXX11_ABI=1  # Use new C++11 ABI for ARM64
        export TORCH_CUDA_VERSION=none  # Disable CUDA for CPU-only build
        export CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu  # Explicit target for ARM64
        export CC=gcc
        export CXX=g++
    else
        export LIBTORCH_CXX11_ABI=0  # Use old ABI for other architectures
    fi
    
    # Additional environment variables for torch-sys compatibility
    export TORCH_CUDA_ARCH_LIST=""  # Empty for CPU-only builds
    export CMAKE_PREFIX_PATH="$LIBTORCH"
    
    print_status "PyTorch environment variables set"
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
    echo "  --help           Show this help message"
    echo ""
    echo "Features:"
    echo "  ✅ Clean installation - removes existing PyTorch"
    echo "  ✅ Automatic PyTorch installation for your platform"
    echo "  ✅ No manual setup required"
    echo "  ✅ Eliminates all version compatibility issues"
    echo ""
    echo "Examples:"
    echo "  $0               # Full clean setup and run"
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
    echo -e "${CYAN}🤖 FinBERT Rust Options API - Clean Install & Run Script${NC}"
    echo -e "${CYAN}=====================================================${NC}"
    echo -e "${CYAN}Clean installation with automatic PyTorch setup${NC}"
    echo ""
    
    # Warn if running as root
    if [[ "$EUID" -eq 0 ]]; then
        print_warning "Running as root/sudo detected!"
        print_status "For best results, run without sudo: ./install.sh"
        print_status "The script will prompt for sudo when needed for system packages."
        echo ""
    fi
    
    # Parse command line arguments
    SETUP_ONLY=false
    BUILD_ONLY=false
    RUN_ONLY=false
    
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
        clean_pytorch
        setup_python_env
        setup_env_file
        download_finbert_model
        build_project
    elif [[ "$SETUP_ONLY" == true ]]; then
        print_step "Setup-only mode selected"
        check_requirements
        clean_pytorch
        setup_python_env
        setup_env_file
        download_finbert_model
        print_success "Setup completed successfully!"
        echo ""
        print_status "Next steps:"
        print_status "1. Edit .env file with your Alpaca API credentials"
        print_status "2. Run: $0 --build-only"
        print_status "3. Run: $0 --run-only"
    else
        # Full setup and run
        print_step "Full clean setup and run mode"
        check_requirements
        clean_pytorch
        setup_python_env
        setup_env_file
        download_finbert_model
        build_project
        run_application
    fi
}

# Run main function
main "$@"
