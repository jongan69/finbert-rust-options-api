# FinBERT Options API - Systemd Service Setup

This guide explains how to set up the FinBERT Options API as a persistent systemd service on Linux.

## Quick Setup

1. **Run the setup script** (requires root privileges):
   ```bash
   sudo ./setup-service.sh
   ```

2. **Configure your environment variables** by editing the service file:
   ```bash
   sudo nano /etc/systemd/system/finbert-options-api.service
   ```

3. **Add your Alpaca API credentials** to the service file:
   ```ini
   Environment=APCA_API_KEY_ID=your_api_key_here
   Environment=APCA_API_SECRET_KEY=your_secret_key_here
   Environment=APCA_BASE_URL=https://paper-api.alpaca.markets
   ```

4. **Start the service**:
   ```bash
   sudo systemctl start finbert-options-api
   ```

5. **Check service status**:
   ```bash
   sudo systemctl status finbert-options-api
   ```

## Manual Setup

If you prefer to set up the service manually:

### 1. Create Service User
```bash
sudo useradd --system --no-create-home --shell /bin/false finbert-api
```

### 2. Create Installation Directory
```bash
sudo mkdir -p /opt/finbert-options-api
sudo chown finbert-api:finbert-api /opt/finbert-options-api
```

### 3. Build and Install Application
```bash
# Copy your application to the install directory
sudo cp -r . /opt/finbert-options-api/
sudo chown -R finbert-api:finbert-api /opt/finbert-options-api

# Build the application
cd /opt/finbert-options-api
sudo -u finbert-api cargo build --release
```

### 4. Install Service File
```bash
sudo cp finbert-options-api.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable finbert-options-api
```

## Service Management

### Start/Stop/Restart Service
```bash
sudo systemctl start finbert-options-api    # Start the service
sudo systemctl stop finbert-options-api     # Stop the service
sudo systemctl restart finbert-options-api  # Restart the service
```

### Check Service Status
```bash
sudo systemctl status finbert-options-api
```

### View Logs
```bash
# View recent logs
sudo journalctl -u finbert-options-api

# Follow logs in real-time
sudo journalctl -u finbert-options-api -f

# View logs from today
sudo journalctl -u finbert-options-api --since today
```

### Enable/Disable Auto-start
```bash
sudo systemctl enable finbert-options-api   # Enable auto-start on boot
sudo systemctl disable finbert-options-api  # Disable auto-start on boot
```

## Configuration

### Environment Variables

The service file includes these default environment variables:

```ini
Environment=RUST_LOG=info
Environment=SERVER_HOST=0.0.0.0
Environment=SERVER_PORT=3000
Environment=MAX_CONCURRENT_REQUESTS=10
Environment=REQUEST_TIMEOUT_SECS=60
Environment=MAX_TEXT_LENGTH=10000
```

### Required Alpaca API Variables

You must add these to the service file:

```ini
Environment=APCA_API_KEY_ID=your_api_key_here
Environment=APCA_API_SECRET_KEY=your_secret_key_here
Environment=APCA_BASE_URL=https://paper-api.alpaca.markets  # or https://api.alpaca.markets for live trading
```

### Optional Configuration

```ini
Environment=SENTIMENT_MODEL_PATH=finbert-onnx
Environment=MAX_CONCURRENT_REQUESTS=20
Environment=REQUEST_TIMEOUT_SECS=120
```

## Security Features

The service file includes several security hardening features:

- **NoNewPrivileges**: Prevents privilege escalation
- **PrivateTmp**: Uses private /tmp directory
- **ProtectSystem**: Read-only system directories
- **ProtectHome**: No access to user home directories
- **Resource Limits**: Limits on file descriptors and processes

## Troubleshooting

### Service Won't Start
1. Check the service status: `sudo systemctl status finbert-options-api`
2. View logs: `sudo journalctl -u finbert-options-api -n 50`
3. Verify environment variables are set correctly
4. Check file permissions: `ls -la /opt/finbert-options-api/`

### Permission Issues
```bash
# Fix ownership
sudo chown -R finbert-api:finbert-api /opt/finbert-options-api

# Fix permissions
sudo chmod +x /opt/finbert-options-api/target/release/finbert-rust-options-api
```

### Port Already in Use
If port 3000 is already in use, change the port in the service file:
```ini
Environment=SERVER_PORT=3001
```

Then restart the service:
```bash
sudo systemctl restart finbert-options-api
```

## API Endpoints

Once the service is running, you can access:

- **Health Check**: `http://your-server:3000/health`
- **Analysis**: `http://your-server:3000/analyze`
- **Metrics**: `http://your-server:3000/metrics`

## Uninstalling

To remove the service:

```bash
sudo systemctl stop finbert-options-api
sudo systemctl disable finbert-options-api
sudo rm /etc/systemd/system/finbert-options-api.service
sudo systemctl daemon-reload
sudo userdel finbert-api
sudo rm -rf /opt/finbert-options-api
```

