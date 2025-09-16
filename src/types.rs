use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptionsQuery {
    pub feed: Option<String>,
    pub r#type: Option<String>,
    pub alpaca_limit: Option<i32>,
    pub strike_price_gte: Option<f64>,
    pub strike_price_lte: Option<f64>,
    pub expiration_date: Option<String>,
    pub expiration_date_gte: Option<String>,
    pub expiration_date_lte: Option<String>,
    pub root_symbol: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub headline: String,
    pub symbols: Vec<String>,
    pub sentiment: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionAnalysis {
    pub contract_type: String,
    pub contract: serde_json::Value,
    pub option_score: f64,
    pub undervalued_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOptionsAnalysis {
    pub symbol: String,
    pub underlying_metrics: serde_json::Value,
    pub options_analysis: Vec<OptionAnalysis>,
    pub error: Option<String>,
}

// New Trading Bot Focused Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub symbol: String,
    pub signal_type: String, // "BUY_CALL", "BUY_PUT", "SELL_CALL", "SELL_PUT"
    pub confidence: f64,
    pub sentiment_score: f64,
    pub risk_score: f64,
    pub expected_return: f64,
    pub max_loss: f64,
    pub time_horizon: String, // "SHORT_TERM", "LEAP"
    pub entry_price: f64,
    pub strike_price: f64,
    pub expiration_date: String,
    pub volume: u64,
    pub open_interest: u64,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub financial_metrics: FinancialMetrics,
    pub reasoning: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub volatility: f64,
    pub composite_score: f64,
    pub kelly_fraction: f64,
    pub var_95: f64, // Value at Risk (95% confidence)
    pub expected_shortfall: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub timestamp: String,
    pub total_signals: usize,
    pub bullish_signals: usize,
    pub bearish_signals: usize,
    pub high_confidence_signals: usize,
    pub market_sentiment: String, // "BULLISH", "BEARISH", "NEUTRAL"
    pub overall_confidence: f64,
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH"
    pub recommended_position_size: f64, // Percentage of portfolio
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingBotResponse {
    pub market_summary: MarketSummary,
    pub trading_signals: Vec<TradingSignal>,
    pub sentiment_analysis: Vec<SentimentAnalysis>,
    pub risk_metrics: RiskMetrics,
    pub execution_metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub portfolio_var: f64,
    pub max_portfolio_drawdown: f64,
    pub diversification_score: f64,
    pub sector_exposure: std::collections::HashMap<String, f64>,
    pub volatility_regime: String, // "LOW", "NORMAL", "HIGH"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub processing_time_ms: u64,
    pub symbols_analyzed: usize,
    pub options_analyzed: usize,
    pub crypto_symbols_filtered: usize,
    pub api_calls_made: usize,
    pub cache_hit_rate: f64,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopOption {
    pub symbol: String,
    pub score: f64,
    pub indicators: Vec<String>,
}

// Financial Metrics Structures
#[derive(Debug, Clone, Serialize)]
pub struct MetricsResult {
    pub n_periods: usize,
    pub mean_return: f64,
    pub volatility: f64,
    pub downside_deviation: f64,
    pub cagr: f64,
    pub max_drawdown: f64,
    pub sharpe: f64,
    pub sortino: f64,
    pub calmar: f64,
    pub kelly_fraction: f64,
    pub composite_score: f64,
}
