#!/bin/bash

# Setup systemd service for current user's PyTorch virtual environment
# This script adapts the service file to work with your existing setup

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

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

print_status "Setting up FinBERT Rust Options API service for current user setup..."

# Get current user info
CURRENT_USER=$(logname || echo $SUDO_USER)
if [[ -z "$CURRENT_USER" ]]; then
    print_error "Could not determine current user"
    exit 1
fi

print_status "Detected user: $CURRENT_USER"

# Configuration
SERVICE_NAME="finbert-api"
SERVICE_USER="$CURRENT_USER"
SERVICE_GROUP="$CURRENT_USER"
INSTALL_DIR="/opt/finbert-rs"
BINARY_PATH="/usr/local/bin/finbert-rs"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

# Check if PyTorch virtual environment exists
PYTORCH_VENV="/home/$CURRENT_USER/pytorch-venv"
if [[ ! -d "$PYTORCH_VENV" ]]; then
    print_error "PyTorch virtual environment not found at $PYTORCH_VENV"
    print_error "Please run ./fix-pytorch-pi.sh first"
    exit 1
fi

# Check if the binary exists
if [[ ! -f "target/release/finbert-rs" ]]; then
    print_warning "Binary not found. Building release version..."
    sudo -u "$CURRENT_USER" cargo build --release
fi

# Create installation directory
print_status "Creating installation directory..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/logs"

# Copy binary
print_status "Installing binary..."
cp target/release/finbert-rs "$BINARY_PATH"
chmod +x "$BINARY_PATH"
print_success "Binary installed to $BINARY_PATH"

# Set ownership
print_status "Setting file permissions..."
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"
chown "$SERVICE_USER:$SERVICE_GROUP" "$BINARY_PATH"

# Copy and customize service file
print_status "Installing systemd service..."
cp finbert-api.service "$SERVICE_FILE"

# Update service file with correct paths for current user
sed -i "s|User=.*|User=$SERVICE_USER|g" "$SERVICE_FILE"
sed -i "s|Group=.*|Group=$SERVICE_GROUP|g" "$SERVICE_FILE"
sed -i "s|WorkingDirectory=.*|WorkingDirectory=$INSTALL_DIR|g" "$SERVICE_FILE"
sed -i "s|ExecStart=.*|ExecStart=$BINARY_PATH|g" "$SERVICE_FILE"
sed -i "s|PYTORCH_VENV=.*|PYTORCH_VENV=$PYTORCH_VENV|g" "$SERVICE_FILE"
sed -i "s|LIBTORCH=.*|LIBTORCH=$PYTORCH_VENV/lib/python3.11/site-packages/torch/lib|g" "$SERVICE_FILE"
sed -i "s|LD_LIBRARY_PATH=.*|LD_LIBRARY_PATH=$PYTORCH_VENV/lib/python3.11/site-packages/torch/lib|g" "$SERVICE_FILE"
sed -i "s|/home/finbert/pytorch-venv|$PYTORCH_VENV|g" "$SERVICE_FILE"
sed -i "s|ReadWritePaths=.*|ReadWritePaths=$INSTALL_DIR/logs $PYTORCH_VENV|g" "$SERVICE_FILE"

# Set proper permissions for service file
chmod 644 "$SERVICE_FILE"

# Reload systemd
print_status "Reloading systemd daemon..."
systemctl daemon-reload

# Enable service
print_status "Enabling service to start on boot..."
systemctl enable "$SERVICE_NAME"

print_success "Service installation completed!"
echo ""
print_warning "IMPORTANT: You need to configure your Alpaca API credentials:"
echo ""
echo "1. Edit the service file:"
echo "   sudo nano $SERVICE_FILE"
echo ""
echo "2. Update these lines with your actual credentials:"
echo "   Environment=APCA_API_KEY_ID=your_actual_api_key"
echo "   Environment=APCA_API_SECRET_KEY=your_actual_secret_key"
echo ""
echo "3. Reload the service:"
echo "   sudo systemctl daemon-reload"
echo ""
echo "4. Start the service:"
echo "   sudo systemctl start $SERVICE_NAME"
echo ""
echo "5. Check service status:"
echo "   sudo systemctl status $SERVICE_NAME"
echo ""
echo "6. View logs:"
echo "   sudo journalctl -u $SERVICE_NAME -f"
echo ""

# Check if credentials are configured
if grep -q "your_alpaca_api_key_here" "$SERVICE_FILE"; then
    print_warning "Service installed but credentials need to be configured!"
    print_warning "Please update the service file with your Alpaca API credentials before starting."
else
    print_success "Service is ready to start!"
    echo ""
    echo "To start the service now:"
    echo "sudo systemctl start $SERVICE_NAME"
    echo ""
    echo "To check the status:"
    echo "sudo systemctl status $SERVICE_NAME"
fi

print_success "Installation completed successfully!"
