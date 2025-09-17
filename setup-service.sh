#!/bin/bash

# FinBERT Options API Service Setup Script
# This script sets up the FinBERT Options API as a systemd service

set -e

echo "🚀 Setting up FinBERT Options API as a systemd service..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Configuration
SERVICE_NAME="finbert-options-api"
SERVICE_USER="finbert-api"
INSTALL_DIR="/opt/finbert-options-api"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

echo "📋 Configuration:"
echo "  Service Name: $SERVICE_NAME"
echo "  Service User: $SERVICE_USER"
echo "  Install Directory: $INSTALL_DIR"
echo "  Service File: $SERVICE_FILE"
echo ""

# Create service user if it doesn't exist
if ! id "$SERVICE_USER" &>/dev/null; then
    echo "👤 Creating service user: $SERVICE_USER"
    useradd --system --no-create-home --shell /bin/false "$SERVICE_USER"
else
    echo "✅ Service user $SERVICE_USER already exists"
fi

# Create installation directory
echo "📁 Creating installation directory: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

# Copy application files
echo "📦 Copying application files..."
cp -r . "$INSTALL_DIR/"

# Set ownership
echo "🔐 Setting ownership to $SERVICE_USER"
chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"

# Build the application
echo "🔨 Building the application..."
cd "$INSTALL_DIR"
sudo -u "$SERVICE_USER" cargo build --release

# Copy service file
echo "📄 Installing service file..."
cp "$INSTALL_DIR/finbert-options-api.service" "$SERVICE_FILE"

# Set proper permissions on service file
chmod 644 "$SERVICE_FILE"

# Reload systemd
echo "🔄 Reloading systemd daemon..."
systemctl daemon-reload

# Enable the service
echo "✅ Enabling service..."
systemctl enable "$SERVICE_NAME"

echo ""
echo "🎉 Service setup complete!"
echo ""
echo "📋 Next steps:"
echo "  1. Configure environment variables in $SERVICE_FILE"
echo "  2. Add your Alpaca API credentials:"
echo "     - APCA_API_KEY_ID"
echo "     - APCA_API_SECRET_KEY"
echo "     - APCA_BASE_URL (optional, defaults to paper trading)"
echo "  3. Start the service:"
echo "     sudo systemctl start $SERVICE_NAME"
echo "  4. Check service status:"
echo "     sudo systemctl status $SERVICE_NAME"
echo "  5. View logs:"
echo "     sudo journalctl -u $SERVICE_NAME -f"
echo ""
echo "🔧 Service management commands:"
echo "  Start:   sudo systemctl start $SERVICE_NAME"
echo "  Stop:    sudo systemctl stop $SERVICE_NAME"
echo "  Restart: sudo systemctl restart $SERVICE_NAME"
echo "  Status:  sudo systemctl status $SERVICE_NAME"
echo "  Logs:    sudo journalctl -u $SERVICE_NAME -f"
echo "  Disable: sudo systemctl disable $SERVICE_NAME"

