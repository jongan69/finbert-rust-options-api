#!/bin/bash
set -e

# FinBERT Sentiment Analysis API - Raspberry Pi Setup Script
# This script sets up the entire FinBERT API on a Raspberry Pi

echo "🚀 Setting up FinBERT Sentiment Analysis API on Raspberry Pi..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if running on Raspberry Pi
print_step "Checking system compatibility..."
if ! grep -q "Raspberry Pi" /proc/cpuinfo 2>/dev/null && ! grep -q "BCM" /proc/cpuinfo 2>/dev/null; then
    print_warning "This script is designed for Raspberry Pi, but will continue anyway..."
fi

# Check architecture
ARCH=$(uname -m)
print_status "Detected architecture: $ARCH"

# Update system
print_step "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install required system dependencies
print_step "Installing system dependencies..."
sudo apt install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    cmake

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    print_step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    print_status "Rust installed successfully"
else
    print_status "Rust is already installed"
    rustc --version
fi

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Clone the ONNX model from Hugging Face
print_step "Downloading FinBERT ONNX model..."
if [ -d "finbert-onnx" ]; then
    print_status "ONNX model directory already exists, updating..."
    cd finbert-onnx
    git pull
    cd ..
else
    print_status "Cloning FinBERT ONNX model from Hugging Face..."
    git clone https://huggingface.co/jonngan/finbert-onnx
fi

# Verify model files exist
if [ ! -f "finbert-onnx/model.onnx" ]; then
    print_error "ONNX model file not found! Download may have failed."
    exit 1
fi

if [ ! -f "finbert-onnx/tokenizer.json" ]; then
    print_error "Tokenizer file not found! Download may have failed."
    exit 1
fi

print_status "✅ ONNX model files verified"

# Set up environment variables
print_step "Setting up environment configuration..."
if [ ! -f ".env" ]; then
    cp .env.example .env
    print_status "Created .env file from template"
    print_warning "⚠️  IMPORTANT: You need to edit .env and set your Alpaca API credentials!"
    print_warning "   Edit the file: nano .env"
    print_warning "   Set APCA_API_KEY_ID and APCA_API_SECRET_KEY"
else
    print_status ".env file already exists"
fi

# Build the application
print_step "Building FinBERT API..."
print_status "This may take 10-30 minutes on Raspberry Pi depending on the model..."

# Set optimization for Raspberry Pi
export CARGO_TARGET_DIR="./target"

# Build with optimizations for ARM
if [[ "$ARCH" == "aarch64" ]] || [[ "$ARCH" == "arm64" ]]; then
    print_status "Building for ARM64/AArch64..."
    cargo build --release
elif [[ "$ARCH" == "armv7l" ]] || [[ "$ARCH" == "armhf" ]]; then
    print_status "Building for ARMv7..."
    cargo build --release
else
    print_status "Building for architecture: $ARCH..."
    cargo build --release
fi

# Verify build succeeded
if [ ! -f "target/release/finbert-rs" ]; then
    print_error "Build failed! Binary not found."
    exit 1
fi

print_status "✅ Build completed successfully!"

# Test the application
print_step "Testing the application..."
if [ -f ".env" ]; then
    # Check if environment variables are set
    source .env
    if [ -z "$APCA_API_KEY_ID" ] || [ -z "$APCA_API_SECRET_KEY" ] || [ "$APCA_API_KEY_ID" = "your_alpaca_api_key_here" ]; then
        print_warning "Alpaca API credentials not set in .env file"
        print_warning "Setting dummy credentials for testing..."
        export APCA_API_KEY_ID="test_key"
        export APCA_API_SECRET_KEY="test_secret"
    fi
fi

# Quick test run (3 seconds)
print_status "Running quick startup test..."
timeout 3s ./target/release/finbert-rs || true

# Create systemd service file
print_step "Creating systemd service..."
sudo tee /etc/systemd/system/finbert-api.service > /dev/null << EOF
[Unit]
Description=FinBERT Sentiment Analysis API
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$(pwd)
EnvironmentFile=$(pwd)/.env
ExecStart=$(pwd)/target/release/finbert-rs
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# Resource limits for Raspberry Pi
LimitNOFILE=65536
MemoryMax=1G

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable finbert-api.service

print_status "✅ Systemd service created and enabled"

# Create convenience scripts
print_step "Creating convenience scripts..."

# Start script
cat > start-api.sh << 'EOF'
#!/bin/bash
echo "🚀 Starting FinBERT API..."
sudo systemctl start finbert-api.service
sleep 2
sudo systemctl status finbert-api.service --no-pager
echo ""
echo "📊 API should be available at: http://$(hostname -I | awk '{print $1}'):3000"
echo "❤️  Health check: curl http://$(hostname -I | awk '{print $1}'):3000/health"
EOF

# Stop script
cat > stop-api.sh << 'EOF'
#!/bin/bash
echo "🛑 Stopping FinBERT API..."
sudo systemctl stop finbert-api.service
sudo systemctl status finbert-api.service --no-pager
EOF

# Status script
cat > status-api.sh << 'EOF'
#!/bin/bash
echo "📊 FinBERT API Status:"
sudo systemctl status finbert-api.service --no-pager
echo ""
echo "📝 Recent logs:"
sudo journalctl -u finbert-api.service --no-pager -n 10
EOF

# Logs script
cat > logs-api.sh << 'EOF'
#!/bin/bash
echo "📝 Following FinBERT API logs (Ctrl+C to exit):"
sudo journalctl -u finbert-api.service -f
EOF

chmod +x start-api.sh stop-api.sh status-api.sh logs-api.sh

print_status "✅ Created convenience scripts: start-api.sh, stop-api.sh, status-api.sh, logs-api.sh"

# Show final instructions
print_step "Setup completed! 🎉"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
print_status "✅ FinBERT Sentiment Analysis API is ready!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
print_warning "📋 NEXT STEPS:"
echo "   1. Edit your API credentials: nano .env"
echo "   2. Set APCA_API_KEY_ID and APCA_API_SECRET_KEY"
echo "   3. Start the API: ./start-api.sh"
echo ""
print_status "🔧 MANAGEMENT COMMANDS:"
echo "   • Start API:    ./start-api.sh"
echo "   • Stop API:     ./stop-api.sh" 
echo "   • Check status: ./status-api.sh"
echo "   • View logs:    ./logs-api.sh"
echo ""
print_status "🌐 ENDPOINTS (when running):"
LOCAL_IP=$(hostname -I | awk '{print $1}')
echo "   • API:     http://$LOCAL_IP:3000/analyze"
echo "   • Health:  http://$LOCAL_IP:3000/health"
echo "   • Metrics: http://$LOCAL_IP:3000/metrics"
echo ""
print_status "📁 FILES CREATED:"
echo "   • Binary:     ./target/release/finbert-rs"
echo "   • Service:    /etc/systemd/system/finbert-api.service"
echo "   • Config:     ./.env"
echo "   • Model:      ./finbert-onnx/"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
print_status "🚀 Run './start-api.sh' to start your FinBERT API!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"