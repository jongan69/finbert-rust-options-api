# 🚀 Quick Start Guide - FinBERT Rust Options API

## One-Command Installation & Run

The easiest way to get started is with our unified install and run script:

```bash
./install-and-run.sh
```

This single command will:
- ✅ Check system requirements
- ✅ Set up Python virtual environment
- ✅ Install PyTorch with compatibility fixes
- ✅ Download FinBERT model
- ✅ Build the Rust application
- ✅ Start the API server

## 🎯 Different Usage Modes

### Full Setup & Run (Default)
```bash
./install-and-run.sh
```

### Setup Only (First time setup)
```bash
./install-and-run.sh --setup-only
```

### Build Only (After setup)
```bash
./install-and-run.sh --build-only
```

### Run Only (After build)
```bash
./install-and-run.sh --run-only
```

### Clean Build (Force rebuild)
```bash
./install-and-run.sh --clean
```

## 📋 Prerequisites

Before running the script, ensure you have:

- **Rust** (install with: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Python 3.8+**
- **Git**
- **4GB+ RAM** (recommended)

## 🔧 Configuration

After setup, edit the `.env` file with your Alpaca API credentials:

```bash
nano .env
```

Add your credentials:
```env
APCA_API_KEY_ID=your_api_key_here
APCA_API_SECRET_KEY=your_secret_key_here
APCA_BASE_URL=https://paper-api.alpaca.markets
```

## 🌐 API Endpoints

Once running, the API will be available at:

- **Health Check:** http://127.0.0.1:3000/health
- **Analysis:** http://127.0.0.1:3000/analyze
- **Metrics:** http://127.0.0.1:3000/metrics

## 🧪 Test the API

```bash
# Health check
curl http://127.0.0.1:3000/health

# Run analysis
curl http://127.0.0.1:3000/analyze
```

## 🔍 Troubleshooting

### Common Issues

**"Permission denied"**
```bash
chmod +x install-and-run.sh
```

**"Rust not found"**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**"Python not found"**
```bash
sudo apt update
sudo apt install python3 python3-pip python3-venv
```

**"Memory issues"**
```bash
# The script automatically detects low memory and uses single-core builds
# For manual control:
export CARGO_BUILD_JOBS=1
./install-and-run.sh --build-only
```

### Detailed Troubleshooting

For detailed troubleshooting, see [RASPBERRY_PI_TROUBLESHOOTING.md](RASPBERRY_PI_TROUBLESHOOTING.md)

## 📊 Performance Tips

### Raspberry Pi 4 (4GB+ RAM)
- Uses all cores for compilation
- Optimal performance

### Raspberry Pi 3 or lower RAM
- Automatically uses single-core builds
- May take longer but uses less memory

### Custom Build Settings
```bash
# Force single core
export CARGO_BUILD_JOBS=1
./install-and-run.sh --build-only

# Use specific number of cores
export CARGO_BUILD_JOBS=2
./install-and-run.sh --build-only
```

## 🔄 Updating

To update the application:

```bash
# Pull latest changes
git pull

# Rebuild with latest changes
./install-and-run.sh --build-only

# Run updated version
./install-and-run.sh --run-only
```

## 📝 Logs

The application logs to stdout. For debugging:

```bash
# Run with debug logging
RUST_LOG=debug ./install-and-run.sh --run-only
```

## 🛑 Stopping the API

Press `Ctrl+C` to stop the API server.

## 📞 Support

If you encounter issues:

1. Check the troubleshooting guide
2. Look at the build output for specific errors
3. Ensure all prerequisites are met
4. Try the `--clean` option for a fresh build

---

**Happy trading! 🤖📈**
