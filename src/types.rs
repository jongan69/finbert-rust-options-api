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
    pub correlation_matrix: Vec<Vec<f64>>,
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

#[derive(Debug, Clone)]
pub struct CompositeWeights {
    pub sharpe: f64,
    pub sortino: f64,
    pub calmar: f64,
}

impl Default for CompositeWeights {
    fn default() -> Self {
        Self {
            sharpe: 0.4,
            sortino: 0.4,
            calmar: 0.2,
        }
    }
}

// Financial Metrics Calculations
pub fn compute_returns_from_prices(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }
    prices
        .windows(2)
        .map(|w| if w[0] != 0.0 { w[1] / w[0] - 1.0 } else { 0.0 })
        .collect()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn stddev_sample(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let m = mean(values);
    let var = values
        .iter()
        .map(|v| {
            let d = v - m;
            d * d
        })
        .sum::<f64>()
        / ((n - 1) as f64);
    var.sqrt()
}

fn downside_deviation(values: &[f64], target_per_period: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_sq = values
        .iter()
        .map(|r| (r - target_per_period).min(0.0))
        .map(|d| d * d)
        .sum::<f64>();
    (sum_sq / (values.len() as f64)).sqrt()
}

fn equity_curve_from_returns(returns: &[f64]) -> Vec<f64> {
    let mut equity = Vec::with_capacity(returns.len() + 1);
    let mut value = 1.0_f64;
    equity.push(value);
    for r in returns {
        value *= 1.0 + *r;
        equity.push(value);
    }
    equity
}

fn max_drawdown_from_equity(equity: &[f64]) -> f64 {
    let mut peak = f64::MIN;
    let mut max_dd = 0.0_f64;
    for &v in equity {
        if v > peak {
            peak = v;
        }
        if peak > 0.0 {
            let dd = (peak - v) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

fn cagr_from_equity(equity: &[f64], periods_per_year: f64) -> f64 {
    if equity.len() < 2 {
        return 0.0;
    }
    let total_periods = (equity.len() - 1) as f64;
    let start = equity.first().copied().unwrap_or(1.0);
    let end = equity.last().copied().unwrap_or(1.0);
    if start <= 0.0 || end <= 0.0 || total_periods <= 0.0 {
        return 0.0;
    }
    (end / start).powf(periods_per_year / total_periods) - 1.0
}

pub fn risk_free_per_period(rf_annual: f64, periods_per_year: f64) -> f64 {
    if rf_annual <= -1.0 {
        return 0.0;
    }
    (1.0 + rf_annual).powf(1.0 / periods_per_year) - 1.0
}

pub fn compute_metrics_from_returns(
    returns: &[f64],
    rf_annual: f64,
    target_return_annual: f64,
    periods_per_year: usize,
    weights: Option<CompositeWeights>,
) -> MetricsResult {
    let ppy = periods_per_year as f64;
    let rf_p = risk_free_per_period(rf_annual, ppy);
    let target_p = risk_free_per_period(target_return_annual, ppy);

    let n = returns.len();
    let mean_r = mean(returns);
    let vol = stddev_sample(returns);
    let dd = downside_deviation(returns, target_p);

    let excess: Vec<f64> = returns.iter().map(|r| r - rf_p).collect();
    let mean_excess = mean(&excess);

    let sharpe = if vol > 1e-12 { (mean_excess / vol) * ppy.sqrt() } else { 0.0 };
    let sortino = if dd > 1e-12 { (mean_excess / dd) * ppy.sqrt() } else { 0.0 };

    let equity = equity_curve_from_returns(returns);
    let max_dd = max_drawdown_from_equity(&equity);
    let cagr = cagr_from_equity(&equity, ppy);
    let calmar = if max_dd > 1e-12 { cagr / max_dd } else { cagr / 1e-12 };

    let variance = vol * vol;
    let mut kelly = if variance > 1e-12 { mean_excess / variance } else { 0.0 };
    if !kelly.is_finite() {
        kelly = 0.0;
    }
    // Clamp to [0, 1] for practical sizing
    kelly = kelly.clamp(0.0, 1.0);

    let w = weights.unwrap_or_default();
    let mut composite = w.sharpe * sharpe + w.sortino * sortino + w.calmar * calmar;
    if !composite.is_finite() {
        composite = 0.0;
    }

    MetricsResult {
        n_periods: n,
        mean_return: mean_r,
        volatility: vol,
        downside_deviation: dd,
        cagr,
        max_drawdown: max_dd,
        sharpe,
        sortino,
        calmar,
        kelly_fraction: kelly,
        composite_score: composite,
    }
}
