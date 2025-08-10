#!/bin/bash

# FinBERT API Systemd Service Installer
# This script installs the FinBERT API as a systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

print_status "Starting FinBERT API service installation..."

# Configuration
SERVICE_NAME="finbert-api"
SERVICE_USER="finbert"
SERVICE_GROUP="finbert"
INSTALL_DIR="/opt/finbert-rs"
BINARY_PATH="/usr/local/bin/finbert-rs"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if the binary exists
if [[ ! -f "target/release/finbert-rs" ]]; then
    print_warning "Binary not found. Building release version..."
    cargo build --release
fi

# Create service user and group
print_status "Creating service user and group..."
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd --system --shell /bin/false --home-dir "$INSTALL_DIR" --create-home "$SERVICE_USER"
    print_success "Created user: $SERVICE_USER"
else
    print_status "User $SERVICE_USER already exists"
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

# Copy service file
print_status "Installing systemd service..."
cp finbert-api.service "$SERVICE_FILE"

# Update service file with correct paths
sed -i "s|WorkingDirectory=.*|WorkingDirectory=$INSTALL_DIR|g" "$SERVICE_FILE"
sed -i "s|ExecStart=.*|ExecStart=$BINARY_PATH|g" "$SERVICE_FILE"

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
