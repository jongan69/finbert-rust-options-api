# 📁 FinBERT Rust API - Project Structure

## 🎯 Overview

This is a clean, production-ready Rust API for financial sentiment analysis using FinBERT. The project has been streamlined to focus on essential components only.

## 📂 Directory Structure

```
finbert-rs/
├── src/                    # Source code
│   ├── main.rs            # Main application entry point
│   ├── alpaca_data.rs     # Alpaca Markets API integration
│   └── types.rs           # Data structures and types
├── finbert/               # FinBERT model files (auto-downloaded by install script)
├── install.sh             # ⭐ Main installation script (recommended)
├── install-and-run.sh     # Original installation script (fallback)
├── Cargo.toml             # Rust project configuration
├── env.example            # Environment variables template
├── README.md              # Project documentation
└── .gitignore             # Git ignore rules
```

## 🚀 Quick Start

### Recommended Installation
```bash
git clone <repository-url>
cd finbert-rs
./install.sh
```

### Alternative Installation
```bash
git clone <repository-url>
cd finbert-rs
./install-and-run.sh
```

## 🔧 Key Features

- **Automatic PyTorch Installation**: Installs correct PyTorch version for your platform
- **Automatic FinBERT Download**: Downloads model from Hugging Face automatically
- **Clean Installation**: No manual setup required
- **Cross-Platform Support**: Works on Raspberry Pi, macOS, Linux
- **Production Ready**: Includes error handling, monitoring, and API endpoints

## 📊 API Endpoints

- **Health Check**: `GET /health`
- **Sentiment Analysis**: `POST /analyze`
- **Metrics**: `GET /metrics`

## 🛠️ Development

```bash
# Build the project
cargo build --release

# Run the application
cargo run --release

# Run tests
cargo test
```

## 📝 Environment Variables

Copy `env.example` to `.env` and configure:
- `APCA_API_KEY_ID`: Your Alpaca API key
- `APCA_API_SECRET_KEY`: Your Alpaca secret key
- `APCA_BASE_URL`: Alpaca API base URL

---

**Note**: This project automatically installs PyTorch for your platform and downloads the FinBERT model, eliminating version conflicts and manual setup requirements.
