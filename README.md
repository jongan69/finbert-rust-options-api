# 🤖 FinBERT Sentiment Analysis Trading API

A production-ready Rust API that performs real-time sentiment analysis on financial news headlines and generates actionable trading signals for options trading. This API combines the power of FinBERT (Financial BERT) with Alpaca Markets data to provide institutional-grade trading intelligence.

## 🎯 Overview

This API analyzes financial news sentiment, filters out crypto assets, and generates comprehensive trading signals with risk metrics, making it perfect for automated trading bots and algorithmic trading systems.

### Key Features
- **Real-time sentiment analysis** using FinBERT model
- **Options chain analysis** with high open interest filtering
- **Professional risk metrics** (Sharpe, Sortino, Calmar ratios)
- **Trading signal generation** with confidence scores
- **Crypto filtering** to avoid unnecessary API calls
- **Production-ready** with retry logic, timeouts, and monitoring
- **Parallel processing** for optimal performance

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ 
- Alpaca Markets API credentials
- 4GB+ RAM (for FinBERT model loading)

### Installation

1. **Clone the repository**
```bash
git clone <repository-url>
cd finbert-rs
```

2. **Set up environment variables**
```bash
# Create .env file
cp .env.example .env

# Add your Alpaca credentials
echo "APCA_API_KEY_ID=your_api_key_here" >> .env
echo "APCA_API_SECRET_KEY=your_secret_key_here" >> .env
echo "APCA_BASE_URL=https://paper-api.alpaca.markets" >> .env
```

3. **Build and run**
```bash
cargo build --release
cargo run
```

The API will be available at `http://127.0.0.1:3000`

## 📊 API Endpoints

### 1. Main Analysis Endpoint
**`GET /analyze`**

Performs complete sentiment analysis and generates trading signals.

**Response Time:** ~2-5 seconds (depending on market conditions)

**Example Response:**
```json
{
  "market_summary": {
    "timestamp": "2024-01-15T18:12:02.123Z",
    "total_signals": 15,
    "bullish_signals": 12,
    "bearish_signals": 3,
    "high_confidence_signals": 8,
    "market_sentiment": "BULLISH",
    "overall_confidence": 0.78,
    "risk_level": "MEDIUM",
    "recommended_position_size": 15.6
  },
  "trading_signals": [
    {
      "symbol": "NVTS",
      "signal_type": "BUY_CALL",
      "confidence": 0.85,
      "sentiment_score": 0.94,
      "risk_score": 0.35,
      "expected_return": 0.75,
      "max_loss": 1.25,
      "time_horizon": "SHORT_TERM",
      "entry_price": 1.25,
      "strike_price": 15.0,
      "expiration_date": "2024-01-19",
      "volume": 1500,
      "open_interest": 2500,
      "implied_volatility": 0.45,
      "delta": 0.6,
      "gamma": 0.05,
      "theta": -0.02,
      "vega": 0.1,
      "financial_metrics": {
        "sharpe_ratio": 1.25,
        "sortino_ratio": 1.45,
        "calmar_ratio": 2.1,
        "max_drawdown": 0.15,
        "volatility": 0.28,
        "composite_score": 1.6,
        "kelly_fraction": 0.35,
        "var_95": 0.46,
        "expected_shortfall": 0.56
      },
      "reasoning": [
        "Sentiment: call (confidence: 0.94)",
        "High volume",
        "Low cost entry",
        "Strong risk-adjusted returns"
      ]
    }
  ],
  "sentiment_analysis": [
    {
      "headline": "Apple's New AI Feature Boosts Stock",
      "symbols": ["AAPL"],
      "sentiment": "Positive",
      "confidence": 0.94
    }
  ],
  "risk_metrics": {
    "portfolio_var": 0.12,
    "max_portfolio_drawdown": 0.25,
    "correlation_matrix": [[1.0, 0.3], [0.3, 1.0]],
    "diversification_score": 0.85,
    "sector_exposure": {
      "TECH": 0.3,
      "FINANCE": 0.2,
      "HEALTHCARE": 0.2,
      "OTHER": 0.3
    },
    "volatility_regime": "NORMAL"
  },
  "execution_metadata": {
    "processing_time_ms": 2450,
    "symbols_analyzed": 42,
    "options_analyzed": 15,
    "crypto_symbols_filtered": 5,
    "api_calls_made": 43,
    "cache_hit_rate": 0.0
  }
}
```

### 2. Health Check
**`GET /health`**

Returns API health status and version information.

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T18:12:02.123Z",
  "version": "0.1.0"
}
```

### 3. Metrics
**`GET /metrics`**

Returns configuration and system metrics.

```json
{
  "config": {
    "max_concurrent_requests": 10,
    "alpaca_base_url": "https://paper-api.alpaca.markets"
  },
  "timestamp": "2024-01-15T18:12:02.123Z"
}
```

## 🤖 Trading Bot Integration Guide

### Python Integration Example

```python
import requests
import json
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime
import time

@dataclass
class TradingSignal:
    symbol: str
    signal_type: str  # "BUY_CALL", "BUY_PUT", "SELL_CALL", "SELL_PUT"
    confidence: float
    sentiment_score: float
    risk_score: float
    expected_return: float
    max_loss: float
    entry_price: float
    strike_price: float
    expiration_date: str
    volume: int
    open_interest: int
    implied_volatility: float
    delta: float
    gamma: float
    theta: float
    vega: float
    sharpe_ratio: float
    sortino_ratio: float
    calmar_ratio: float
    kelly_fraction: float
    reasoning: List[str]

class FinBERTTradingBot:
    def __init__(self, api_url: str = "http://127.0.0.1:3000"):
        self.api_url = api_url
        self.session = requests.Session()
        
    def get_trading_signals(self) -> Dict:
        """Fetch trading signals from the API"""
        try:
            response = self.session.get(f"{self.api_url}/analyze", timeout=60)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            print(f"API request failed: {e}")
            return None
    
    def filter_high_confidence_signals(self, signals: List[TradingSignal], 
                                     min_confidence: float = 0.8,
                                     max_risk: float = 0.4) -> List[TradingSignal]:
        """Filter signals based on confidence and risk criteria"""
        return [
            signal for signal in signals
            if signal.confidence >= min_confidence and signal.risk_score <= max_risk
        ]
    
    def calculate_position_size(self, signal: TradingSignal, 
                              portfolio_value: float,
                              max_risk_per_trade: float = 0.02) -> float:
        """Calculate position size using Kelly Criterion and risk management"""
        # Use Kelly fraction for optimal sizing
        kelly_size = signal.kelly_fraction * portfolio_value
        
        # Apply risk management constraints
        max_loss_amount = portfolio_value * max_risk_per_trade
        max_position = max_loss_amount / signal.max_loss if signal.max_loss > 0 else 0
        
        return min(kelly_size, max_position)
    
    def execute_trade(self, signal: TradingSignal, position_size: float):
        """Execute trade through your broker API"""
        # Implement your broker-specific trade execution here
        print(f"Executing {signal.signal_type} for {signal.symbol}")
        print(f"Position size: ${position_size:,.2f}")
        print(f"Entry price: ${signal.entry_price}")
        print(f"Expected return: {signal.expected_return:.2%}")
        print(f"Max loss: ${signal.max_loss}")
        print(f"Confidence: {signal.confidence:.2%}")
        print(f"Risk score: {signal.risk_score:.2%}")
        print("---")
    
    def run_trading_cycle(self, portfolio_value: float = 100000):
        """Main trading cycle"""
        print("🔄 Starting trading cycle...")
        
        # Get market analysis
        analysis = self.get_trading_signals()
        if not analysis:
            print("❌ Failed to get trading signals")
            return
        
        # Extract market summary
        market_summary = analysis["market_summary"]
        print(f"📊 Market Sentiment: {market_summary['market_sentiment']}")
        print(f"📈 Overall Confidence: {market_summary['overall_confidence']:.2%}")
        print(f"⚠️  Risk Level: {market_summary['risk_level']}")
        print(f"💰 Recommended Position Size: {market_summary['recommended_position_size']:.1f}%")
        
        # Process trading signals
        signals = []
        for signal_data in analysis["trading_signals"]:
            signal = TradingSignal(
                symbol=signal_data["symbol"],
                signal_type=signal_data["signal_type"],
                confidence=signal_data["confidence"],
                sentiment_score=signal_data["sentiment_score"],
                risk_score=signal_data["risk_score"],
                expected_return=signal_data["expected_return"],
                max_loss=signal_data["max_loss"],
                entry_price=signal_data["entry_price"],
                strike_price=signal_data["strike_price"],
                expiration_date=signal_data["expiration_date"],
                volume=signal_data["volume"],
                open_interest=signal_data["open_interest"],
                implied_volatility=signal_data["implied_volatility"],
                delta=signal_data["delta"],
                gamma=signal_data["gamma"],
                theta=signal_data["theta"],
                vega=signal_data["vega"],
                sharpe_ratio=signal_data["financial_metrics"]["sharpe_ratio"],
                sortino_ratio=signal_data["financial_metrics"]["sortino_ratio"],
                calmar_ratio=signal_data["financial_metrics"]["calmar_ratio"],
                kelly_fraction=signal_data["financial_metrics"]["kelly_fraction"],
                reasoning=signal_data["reasoning"]
            )
            signals.append(signal)
        
        # Filter high-confidence signals
        high_confidence_signals = self.filter_high_confidence_signals(signals)
        print(f"🎯 Found {len(high_confidence_signals)} high-confidence signals")
        
        # Execute trades
        total_invested = 0
        for signal in high_confidence_signals:
            position_size = self.calculate_position_size(signal, portfolio_value)
            if position_size > 0:
                self.execute_trade(signal, position_size)
                total_invested += position_size
        
        print(f"💼 Total invested: ${total_invested:,.2f}")
        print(f"📊 Processing time: {analysis['execution_metadata']['processing_time_ms']}ms")
        print("✅ Trading cycle completed")

# Usage example
if __name__ == "__main__":
    bot = FinBERTTradingBot()
    
    # Run single cycle
    bot.run_trading_cycle(portfolio_value=100000)
    
    # Or run continuous monitoring
    # while True:
    #     bot.run_trading_cycle(portfolio_value=100000)
    #     time.sleep(300)  # Wait 5 minutes between cycles
```

### JavaScript/Node.js Integration

```javascript
const axios = require('axios');

class FinBERTTradingBot {
    constructor(apiUrl = 'http://127.0.0.1:3000') {
        this.apiUrl = apiUrl;
        this.client = axios.create({
            timeout: 60000,
            headers: {
                'Content-Type': 'application/json'
            }
        });
    }

    async getTradingSignals() {
        try {
            const response = await this.client.get(`${this.apiUrl}/analyze`);
            return response.data;
        } catch (error) {
            console.error('API request failed:', error.message);
            return null;
        }
    }

    filterSignals(signals, minConfidence = 0.8, maxRisk = 0.4) {
        return signals.filter(signal => 
            signal.confidence >= minConfidence && signal.risk_score <= maxRisk
        );
    }

    calculatePositionSize(signal, portfolioValue, maxRiskPerTrade = 0.02) {
        const kellySize = signal.financial_metrics.kelly_fraction * portfolioValue;
        const maxLossAmount = portfolioValue * maxRiskPerTrade;
        const maxPosition = signal.max_loss > 0 ? maxLossAmount / signal.max_loss : 0;
        
        return Math.min(kellySize, maxPosition);
    }

    async executeTrade(signal, positionSize) {
        // Implement your broker-specific trade execution here
        console.log(`Executing ${signal.signal_type} for ${signal.symbol}`);
        console.log(`Position size: $${positionSize.toLocaleString()}`);
        console.log(`Entry price: $${signal.entry_price}`);
        console.log(`Expected return: ${(signal.expected_return * 100).toFixed(2)}%`);
        console.log(`Confidence: ${(signal.confidence * 100).toFixed(2)}%`);
        console.log('---');
    }

    async runTradingCycle(portfolioValue = 100000) {
        console.log('🔄 Starting trading cycle...');

        const analysis = await this.getTradingSignals();
        if (!analysis) {
            console.log('❌ Failed to get trading signals');
            return;
        }

        const { market_summary, trading_signals } = analysis;

        console.log(`📊 Market Sentiment: ${market_summary.market_sentiment}`);
        console.log(`📈 Overall Confidence: ${(market_summary.overall_confidence * 100).toFixed(2)}%`);
        console.log(`⚠️  Risk Level: ${market_summary.risk_level}`);

        const highConfidenceSignals = this.filterSignals(trading_signals);
        console.log(`🎯 Found ${highConfidenceSignals.length} high-confidence signals`);

        let totalInvested = 0;
        for (const signal of highConfidenceSignals) {
            const positionSize = this.calculatePositionSize(signal, portfolioValue);
            if (positionSize > 0) {
                await this.executeTrade(signal, positionSize);
                totalInvested += positionSize;
            }
        }

        console.log(`💼 Total invested: $${totalInvested.toLocaleString()}`);
        console.log(`📊 Processing time: ${analysis.execution_metadata.processing_time_ms}ms`);
        console.log('✅ Trading cycle completed');
    }
}

// Usage
const bot = new FinBERTTradingBot();
bot.runTradingCycle(100000);
```

## 📈 Signal Interpretation Guide

### Signal Types
- **`BUY_CALL`**: Bullish sentiment, buy call options
- **`BUY_PUT`**: Bearish sentiment, buy put options
- **`SELL_CALL`**: Bearish sentiment, sell call options (covered calls)
- **`SELL_PUT`**: Bullish sentiment, sell put options (cash-secured puts)

### Confidence Levels
- **0.9+**: Very high confidence - Strong signal
- **0.8-0.9**: High confidence - Good signal
- **0.7-0.8**: Medium confidence - Moderate signal
- **<0.7**: Low confidence - Weak signal

### Risk Scores
- **0.0-0.3**: Low risk
- **0.3-0.7**: Medium risk
- **0.7-1.0**: High risk

### Financial Metrics
- **Sharpe Ratio**: >1.0 = Good risk-adjusted returns
- **Sortino Ratio**: >1.0 = Good downside risk management
- **Calmar Ratio**: >1.0 = Good return vs drawdown
- **Kelly Fraction**: Optimal position sizing (0.0-1.0)

## ⚙️ Configuration

### Environment Variables
```bash
# Required
APCA_API_KEY_ID=your_alpaca_api_key
APCA_API_SECRET_KEY=your_alpaca_secret_key

# Optional
APCA_BASE_URL=https://paper-api.alpaca.markets
```

### Performance Tuning
```rust
// In src/main.rs - AppConfig
pub struct AppConfig {
    pub max_concurrent_requests: usize,  // Default: 10
    pub sentiment_model_path: String,    // Default: "finbert-sentiment"
    // ... other fields
}
```

## 🔧 Deployment

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/finbert-rs /usr/local/bin/
EXPOSE 3000
CMD ["finbert-rs"]
```

### Systemd Service Installation

#### Quick Installation
```bash
# Build the release version
cargo build --release

# Install as systemd service (requires sudo)
sudo ./install-service.sh
```

#### Manual Installation
1. **Create the service file** `/etc/systemd/system/finbert-api.service`:
```ini
[Unit]
Description=FinBERT Sentiment Analysis Trading API
Documentation=https://github.com/your-repo/finbert-rs
After=network.target
Wants=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
User=finbert
Group=finbert
WorkingDirectory=/opt/finbert-rs
Environment=RUST_LOG=info
Environment=APCA_API_KEY_ID=your_alpaca_api_key_here
Environment=APCA_API_SECRET_KEY=your_alpaca_secret_key_here
Environment=APCA_BASE_URL=https://paper-api.alpaca.markets
ExecStart=/usr/local/bin/finbert-rs
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=finbert-api

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/finbert-rs/logs
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictSUIDSGID=true

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
MemoryMax=4G
CPUQuota=200%

# Health check
ExecStartPre=/bin/systemctl is-active --quiet network-online.target || exit 1
TimeoutStartSec=60
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
```

2. **Install the binary**:
```bash
sudo cp target/release/finbert-rs /usr/local/bin/
sudo chmod +x /usr/local/bin/finbert-rs
```

3. **Create service user**:
```bash
sudo useradd --system --shell /bin/false --home-dir /opt/finbert-rs --create-home finbert
```

4. **Enable and start the service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable finbert-api
sudo systemctl start finbert-api
```

#### Service Management
Use the provided management script for easy service control:

```bash
# Check service status
./manage-service.sh status

# Start the service
sudo ./manage-service.sh start

# Stop the service
sudo ./manage-service.sh stop

# Restart the service
sudo ./manage-service.sh restart

# View logs
./manage-service.sh logs

# Check API health
./manage-service.sh health

# Show API metrics
./manage-service.sh metrics

# Show all available commands
./manage-service.sh help
```

## 📊 Monitoring & Alerting

### Health Checks
```bash
# Check API health
curl http://localhost:3000/health

# Monitor metrics
curl http://localhost:3000/metrics
```

### Logging
The API provides structured logging for:
- Request processing times
- Error rates and types
- API call counts
- Crypto filtering statistics

### Performance Metrics
- **Processing Time**: Target <5 seconds
- **Error Rate**: Target <1%
- **API Call Efficiency**: Crypto filtering reduces calls by 20-30%

## 🚨 Risk Management

### Position Sizing
```python
# Conservative approach
position_size = min(
    signal.kelly_fraction * portfolio_value,
    portfolio_value * 0.02 / signal.max_loss  # 2% max risk per trade
)
```

### Stop Losses
```python
# Set stop loss based on max_loss
stop_loss = signal.entry_price - signal.max_loss
```

### Portfolio Limits
```python
# Maximum portfolio exposure
max_portfolio_exposure = 0.20  # 20% of portfolio
max_single_position = 0.05     # 5% per position
```

## 🔍 Troubleshooting

### Common Issues

1. **Model Loading Slow**
   - First request takes 10-30 seconds
   - Subsequent requests are fast
   - Consider pre-warming the model

2. **API Timeouts**
   - Default timeout: 30 seconds
   - Implement retry logic in your bot
   - Check Alpaca API status

3. **High Memory Usage**
   - FinBERT model requires ~2GB RAM
   - Monitor memory usage
   - Restart service if needed

4. **No Trading Signals**
   - Check if news headlines contain symbols
   - Verify Alpaca API credentials
   - Check market hours (API works 24/7)

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug cargo run
```

## 📚 API Reference

### Response Schema

#### Market Summary
```typescript
interface MarketSummary {
  timestamp: string;
  total_signals: number;
  bullish_signals: number;
  bearish_signals: number;
  high_confidence_signals: number;
  market_sentiment: "BULLISH" | "BEARISH" | "NEUTRAL";
  overall_confidence: number;
  risk_level: "LOW" | "MEDIUM" | "HIGH";
  recommended_position_size: number;
}
```

#### Trading Signal
```typescript
interface TradingSignal {
  symbol: string;
  signal_type: "BUY_CALL" | "BUY_PUT" | "SELL_CALL" | "SELL_PUT";
  confidence: number;
  sentiment_score: number;
  risk_score: number;
  expected_return: number;
  max_loss: number;
  time_horizon: "SHORT_TERM" | "LEAP";
  entry_price: number;
  strike_price: number;
  expiration_date: string;
  volume: number;
  open_interest: number;
  implied_volatility: number;
  delta: number;
  gamma: number;
  theta: number;
  vega: number;
  financial_metrics: FinancialMetrics;
  reasoning: string[];
}
```

#### Financial Metrics
```typescript
interface FinancialMetrics {
  sharpe_ratio: number;
  sortino_ratio: number;
  calmar_ratio: number;
  max_drawdown: number;
  volatility: number;
  composite_score: number;
  kelly_fraction: number;
  var_95: number;
  expected_shortfall: number;
}
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo clippy` to ensure code quality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This software is for educational and research purposes only. Trading involves substantial risk of loss and is not suitable for all investors. Past performance does not guarantee future results. Always consult with a financial advisor before making investment decisions.

## 🆘 Support

- **Issues**: Create an issue on GitHub
- **Documentation**: Check this README and inline code comments
- **Community**: Join our Discord/Telegram for discussions

---

**Happy Trading! 🚀📈**