use serde_json::Value;
use reqwest::Client;
use crate::types::OptionsQuery;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::timeout;

// Get News from Alpaca with timeout and retry logic
pub async fn get_alpaca_news() -> Result<Value, String> {
    let key = std::env::var("APCA_API_KEY_ID")
        .map_err(|_| "APCA_API_KEY_ID missing".to_string())?;
    let secret = std::env::var("APCA_API_SECRET_KEY")
        .map_err(|_| "APCA_API_SECRET_KEY missing".to_string())?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
    
    // Retry logic with exponential backoff
    let mut attempt = 0;
    let max_attempts = 3;
    
    while attempt < max_attempts {
        let resp = timeout(
            Duration::from_secs(60),
            client.get("https://data.alpaca.markets/v1beta1/news?sort=desc&limit=50")
                .header("APCA-API-KEY-ID", key.clone())
                .header("APCA-API-SECRET-KEY", secret.clone())
                .header("accept", "application/json")
                .send()
        ).await
            .map_err(|_| "Request timeout".to_string())?
            .map_err(|e| format!("alpaca news req error: {e}"))?;
        
        if resp.status().is_success() {
            let v = resp.json::<Value>().await
                .map_err(|e| format!("alpaca news json error: {e}"))?;
            return Ok(v);
        }
        
        // If not successful, retry with exponential backoff
        attempt += 1;
        if attempt < max_attempts {
            let delay = Duration::from_secs(2_u64.pow(attempt as u32));
            tokio::time::sleep(delay).await;
        }
    }
    
    Err("Failed to fetch news after all retry attempts".to_string())
}

// Get Options from Alpaca
pub async fn fetch_alpaca_options(symbol: &str, q: &OptionsQuery) -> Result<Value, String> {
    let key = std::env::var("APCA_API_KEY_ID")
        .map_err(|_| "APCA_API_KEY_ID missing".to_string())?;
    let secret = std::env::var("APCA_API_SECRET_KEY")
        .map_err(|_| "APCA_API_SECRET_KEY missing".to_string())?;
    // helper to perform a single request with an optional feed override and retry logic
    async fn do_request(symbol: &str, headers: (&str, &str), q: &OptionsQuery, feed_override: Option<&str>) -> Result<Value, String> {
        let (key, secret) = headers;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
        
        // Retry logic with exponential backoff
        let mut attempt = 0;
        let max_attempts = 3;
        
        while attempt < max_attempts {
            let mut req = client
                .get(format!("https://data.alpaca.markets/v1beta1/options/snapshots/{symbol}"))
                .header("APCA-API-KEY-ID", key)
                .header("APCA-API-SECRET-KEY", secret)
                .header("accept", "application/json");
            
            let mut qp: Vec<(String, String)> = Vec::new();
            if let Some(f) = feed_override.or(q.feed.as_deref()) { qp.push(("feed".into(), f.to_string())); }
            if let Some(v) = &q.r#type { qp.push(("type".into(), v.clone())); }
            qp.push(("limit".into(), q.alpaca_limit.unwrap_or(100).to_string()));
            if let Some(v) = q.strike_price_gte { qp.push(("strike_price_gte".into(), v.to_string())); }
            if let Some(v) = q.strike_price_lte { qp.push(("strike_price_lte".into(), v.to_string())); }
            if let Some(v) = &q.expiration_date { qp.push(("expiration_date".into(), v.clone())); }
            if let Some(v) = &q.expiration_date_gte { qp.push(("expiration_date_gte".into(), v.clone())); }
            if let Some(v) = &q.expiration_date_lte { qp.push(("expiration_date_lte".into(), v.clone())); }
            if let Some(v) = &q.root_symbol { qp.push(("root_symbol".into(), v.clone())); }
            if let Some(v) = &q.page_token { qp.push(("page_token".into(), v.clone())); }
            req = req.query(&qp);
            
            let resp = timeout(Duration::from_secs(60), req.send()).await
                .map_err(|_| "Request timeout".to_string())?
                .map_err(|e| format!("alpaca req error: {e}"))?;
            
            if resp.status().is_success() {
                return resp.json::<Value>().await.map_err(|e| format!("alpaca json error: {e}"));
            }
            
            // If not successful, retry with exponential backoff
            attempt += 1;
            if attempt < max_attempts {
                let delay = Duration::from_secs(2_u64.pow(attempt as u32));
                tokio::time::sleep(delay).await;
            }
        }
        
        Err("Failed to fetch options after all retry attempts".to_string())
    }

    let headers = (key.as_str(), secret.as_str());
    // Always include feed in the URL: use provided feed or default to indicative
    let feed = q.feed.as_deref().unwrap_or("indicative");
    do_request(symbol, headers, q, Some(feed)).await
}

// Crypto filter - symbols that don't have traditional options
pub fn is_crypto_symbol(symbol: &str) -> bool {
    let crypto_symbols: HashSet<&str> = [
        "BTC", "ETH", "BTCUSD", "ETHUSD", "SHIBUSD", "LTCUSD", "ADA", "DOT", "LINK", "UNI",
        "BCH", "LTC", "XRP", "XLM", "EOS", "TRX", "VET", "MATIC", "AVAX", "SOL", "ATOM", "FTM",
        "NEAR", "ALGO", "ICP", "FIL", "THETA", "XTZ", "AAVE", "COMP", "MKR", "SNX", "CRV", "YFI",
        "SUSHI", "1INCH", "BAL", "REN", "ZRX", "BAND", "KNC", "STORJ", "MANA", "SAND", "ENJ", "CHZ",
        "HOT", "DOGE", "SHIB", "BABYDOGE", "SAFEMOON", "ELON", "FLOKI", "PEPE", "BONK", "WIF"
    ].iter().cloned().collect();
    
    crypto_symbols.contains(symbol)
}

// Get Stocks from Alpaca

// High Open Interest Result structure
#[derive(Debug)]
struct HighOpenInterestResult {
    short_term: Option<Value>,
    leap: Option<Value>,
    error: Option<String>,
}

// Analyze ticker options with high open interest
pub async fn analyze_ticker_options(
    symbol: &str,
    underlying_metrics: &Value,
    option_type: Option<&str>,
) -> Result<Value, String> {
    // Get high open interest contracts
    let hoi_result = get_high_open_interest_contracts(symbol, option_type).await;
    
    let spot_price = underlying_metrics.get("spot_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let composite_score = underlying_metrics.get("metrics")
        .and_then(|m| m.get("composite_score"))
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);
    
    let mut options_analysis = Vec::new();
    
    // Analyze short-term contracts
    if let Some(contract) = hoi_result.short_term {
        let option_score = calculate_option_score(&contract, spot_price, composite_score);
        options_analysis.push(serde_json::json!({
            "contract_type": "short_term",
            "contract": contract,
            "option_score": option_score,
            "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
        }));
    }
    
    // Analyze LEAP contracts
    if let Some(contract) = hoi_result.leap {
        let option_score = calculate_option_score(&contract, spot_price, composite_score);
        options_analysis.push(serde_json::json!({
            "contract_type": "leap",
            "contract": contract,
            "option_score": option_score,
            "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
        }));
    }
    
    Ok(serde_json::json!({
        "symbol": symbol,
        "underlying_metrics": underlying_metrics,
        "options_analysis": options_analysis,
        "error": hoi_result.error
    }))
}

// Get high open interest contracts
async fn get_high_open_interest_contracts(symbol: &str, option_type: Option<&str>) -> HighOpenInterestResult {
    let mut result = HighOpenInterestResult {
        short_term: None,
        leap: None,
        error: None,
    };
    
    // Fetch options data
    let options_query = crate::types::OptionsQuery {
        feed: Some("indicative".to_string()),
        r#type: option_type.map(|t| t.to_string()),
        alpaca_limit: Some(50), // Get more contracts to find high OI
        ..Default::default()
    };
    
    match fetch_alpaca_options(symbol, &options_query).await {
        Ok(options_data) => {
            if let Some(snapshots) = options_data.get("snapshots") {
                if let Some(snapshots_obj) = snapshots.as_object() {
                    // Find contracts with highest open interest
                    let mut contracts: Vec<(&String, &Value)> = snapshots_obj.iter().collect();
                    
                    // Sort by open interest (if available) or use volume as proxy
                    contracts.sort_by(|a, b| {
                        let a_oi = a.1.get("latestQuote").and_then(|q| q.get("as")).and_then(|v| v.as_u64()).unwrap_or(0);
                        let b_oi = b.1.get("latestQuote").and_then(|q| q.get("as")).and_then(|v| v.as_u64()).unwrap_or(0);
                        b_oi.cmp(&a_oi) // Sort descending
                    });
                    
                    // Take top contracts and add contract key information
                    if !contracts.is_empty() {
                        let mut contract_data = contracts[0].1.clone();
                        // Add contract key information to the contract data
                        contract_data["contract_key"] = serde_json::Value::String(contracts[0].0.clone());
                        result.short_term = Some(contract_data);
                    }
                    if contracts.len() > 1 {
                        let mut contract_data = contracts[1].1.clone();
                        // Add contract key information to the contract data
                        contract_data["contract_key"] = serde_json::Value::String(contracts[1].0.clone());
                        result.leap = Some(contract_data);
                    }
                }
            }
        }
        Err(e) => {
            result.error = Some(e);
        }
    }
    
    result
}

// Calculate option score based on various factors
fn calculate_option_score(contract: &Value, _spot_price: f64, composite_score: f64) -> f64 {
    let mut score = 0.0;
    
    // Base score from composite sentiment
    score += composite_score * 0.3;
    
    // Volume/Open Interest factor
    if let Some(volume) = contract.get("latestQuote").and_then(|q| q.get("as")).and_then(|v| v.as_u64()) {
        score += (volume as f64 / 1000.0).min(10.0); // Cap at 10 points
    }
    
    // Price factor (lower price = higher score for affordability)
    if let Some(price) = contract.get("latestQuote").and_then(|q| q.get("ap")).and_then(|p| p.as_f64()) {
        if price > 0.0 {
            score += (1.0 / price).min(5.0); // Cap at 5 points
        }
    }
    
    score
}

// Calculate undervalued indicators
fn calculate_undervalued_indicators(contract: &Value, _spot_price: f64, composite_score: f64) -> Vec<String> {
    let mut indicators = Vec::new();
    
    // High volume indicator
    if let Some(volume) = contract.get("latestQuote").and_then(|q| q.get("as")).and_then(|v| v.as_u64()) {
        if volume > 1000 {
            indicators.push("High volume".to_string());
        }
    }
    
    // Low price indicator
    if let Some(price) = contract.get("latestQuote").and_then(|q| q.get("ap")).and_then(|p| p.as_f64()) {
        if price < 1.0 {
            indicators.push("Low cost entry".to_string());
        }
    }
    
    // Strong sentiment indicator
    if composite_score > 0.7 {
        indicators.push("Strong sentiment".to_string());
    }
    
    indicators
}

// Calculate financial metrics for an option contract using options-specific data
pub fn calculate_option_financial_metrics(contract: &Value) -> Option<crate::types::MetricsResult> {
    // Extract option-specific data
    let entry_price = contract.get("latestQuote")
        .and_then(|q| q.get("ap"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);
    
    let strike_price = contract.get("contract_key")
        .and_then(|k| k.as_str())
        .map(parse_strike_price_from_contract_key)
        .unwrap_or(0.0);
    
    let volume = contract.get("latestQuote")
        .and_then(|q| q.get("as"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as f64;
    
    let implied_volatility = contract.get("implied_volatility")
        .and_then(|iv| iv.as_f64())
        .unwrap_or(0.3);
    
    // Skip if we don't have essential data
    if entry_price <= 0.0 {
        return None;
    }
    
    // Use a default strike price if not available
    let strike_price = if strike_price > 0.0 { strike_price } else { entry_price * 1.1 };
    
    // Calculate options-specific metrics
    let time_to_expiry = calculate_time_to_expiry(contract);
    let moneyness = if strike_price > 0.0 { entry_price / strike_price } else { 1.0 };
    
    // Estimate expected return based on moneyness and volatility
    let expected_return = if moneyness > 0.8 && moneyness < 1.2 {
        // Near-the-money options have higher expected returns
        implied_volatility * 0.5
    } else {
        // Out-of-the-money options have lower expected returns
        implied_volatility * 0.2
    };
    
    // Calculate volatility (use implied volatility as base)
    let volatility = implied_volatility * (1.0 + (volume / 10000.0).min(1.0));
    
    // Calculate Sharpe ratio (simplified)
    let risk_free_rate = 0.05; // 5% annual
    let sharpe = if volatility > 0.0 {
        (expected_return - risk_free_rate / 252.0) / volatility
    } else {
        0.0
    };
    
    // Calculate Sortino ratio (using volatility as downside deviation proxy)
    let sortino = if volatility > 0.0 {
        (expected_return - risk_free_rate / 252.0) / (volatility * 0.8) // Assume 80% of volatility is downside
    } else {
        0.0
    };
    
    // Calculate max drawdown (estimated based on option characteristics)
    let max_drawdown = if time_to_expiry > 0.0 {
        let time_factor = (time_to_expiry / 30.0).min(1.0);
        (1.0 - time_factor) * 0.5 + 0.1 // Range from 10% to 60% drawdown
    } else {
        0.3 // Default 30% for unknown expiry
    };
    
    // Calculate CAGR (annualized expected return)
    let cagr = expected_return * 252.0; // Annualize daily return
    
    // Calculate Calmar ratio
    let calmar = if max_drawdown > 0.0 { cagr / max_drawdown } else { 0.0 };
    
    // Calculate Kelly fraction (simplified for options)
    let kelly = if volatility > 0.0 {
        let win_prob = if moneyness > 0.9 { 0.6 } else { 0.4 }; // Estimate win probability
        let avg_win = expected_return * 2.0; // Estimate average win
        let avg_loss = entry_price; // Average loss is premium paid
        ((win_prob * avg_win - (1.0 - win_prob) * avg_loss) / avg_win).clamp(0.0, 0.25)
    } else {
        0.0
    };
    
    // Calculate composite score
    let composite_score = 0.4 * sharpe + 0.4 * sortino + 0.2 * calmar;
    
    Some(crate::types::MetricsResult {
        n_periods: 1,
        mean_return: expected_return,
        volatility,
        downside_deviation: volatility * 0.8,
        cagr,
        max_drawdown,
        sharpe,
        sortino,
        calmar,
        kelly_fraction: kelly,
        composite_score,
    })
}

// Calculate time to expiry in days
fn calculate_time_to_expiry(contract: &Value) -> f64 {
    // Try to get expiration date from contract key first
    if let Some(expiration_str) = contract.get("contract_key")
        .and_then(|k| k.as_str())
        .map(parse_expiration_date_from_contract_key)
        .filter(|s| !s.is_empty())
    {
        if let Ok(expiration_date) = chrono::NaiveDate::parse_from_str(&expiration_str, "%Y-%m-%d") {
            let today = chrono::Utc::now().date_naive();
            let duration = expiration_date.signed_duration_since(today);
            let days = duration.num_days() as f64;
            return if days > 0.0 { days } else { 1.0 }; // Minimum 1 day
        }
    }
    
    // Fallback: try to get from expiration_date field directly
    if let Some(expiration_str) = contract.get("expiration_date")
        .and_then(|e| e.as_str())
        .filter(|s| !s.is_empty())
    {
        if let Ok(expiration_date) = chrono::NaiveDate::parse_from_str(expiration_str, "%Y-%m-%d") {
            let today = chrono::Utc::now().date_naive();
            let duration = expiration_date.signed_duration_since(today);
            let days = duration.num_days() as f64;
            return if days > 0.0 { days } else { 1.0 }; // Minimum 1 day
        }
    }
    
    30.0 // Default to 30 days if we can't parse
}

// Parse strike price from contract key (format: SYMBOLYYMMDDC/PSSTRIKEPRICE)
fn parse_strike_price_from_contract_key(contract_key: &str) -> f64 {
    // Contract key format: SYMBOLYYMMDDC/PSSTRIKEPRICE
    // Example: AAPL240119C00150000 (AAPL, 2024-01-19, Call, $150.00)
    if contract_key.len() >= 15 {
        // Extract the strike price part (last 8 characters, but we need to handle decimal)
        let strike_part = &contract_key[contract_key.len()-8..];
        if let Ok(strike_int) = strike_part.parse::<u32>() {
            // Convert from integer representation to decimal (divide by 1000)
            return strike_int as f64 / 1000.0;
        }
    }
    0.0
}

// Parse expiration date from contract key (format: SYMBOLYYMMDDC/PSSTRIKEPRICE)
fn parse_expiration_date_from_contract_key(contract_key: &str) -> String {
    // Contract key format: SYMBOLYYMMDDC/PSSTRIKEPRICE
    // Example: AAPL240119C00150000 (AAPL, 2024-01-19, Call, $150.00)
    if contract_key.len() >= 15 {
        // Extract the date part (YYMMDD) - positions 4-9 from the end
        let date_part = &contract_key[contract_key.len()-15..contract_key.len()-9];
        if date_part.len() == 6 {
            // Parse YYMMDD format
            if let (Ok(year), Ok(month), Ok(day)) = (
                date_part[0..2].parse::<u32>(),
                date_part[2..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
            ) {
                // Convert 2-digit year to 4-digit (assuming 20xx)
                let full_year = 2000 + year;
                return format!("{:04}-{:02}-{:02}", full_year, month, day);
            }
        }
    }
    String::new()
}

// Convert option analysis to trading signal
pub fn convert_to_trading_signal(
    symbol: &str,
    option_analysis: &crate::types::OptionAnalysis,
    sentiment_score: f64,
    overall_sentiment: &str,
) -> crate::types::TradingSignal {
    let contract = &option_analysis.contract;
    
    // Extract option data
    let entry_price = contract.get("latestQuote")
        .and_then(|q| q.get("ap"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);
    
    // Extract strike price from contract key
    let strike_price = contract.get("contract_key")
        .and_then(|k| k.as_str())
        .map(parse_strike_price_from_contract_key)
        .unwrap_or(0.0);
    
    // Extract expiration date from contract key
    let expiration_date = contract.get("contract_key")
        .and_then(|k| k.as_str())
        .map(parse_expiration_date_from_contract_key)
        .unwrap_or_else(|| {
            // Fallback to contract field if available
            contract.get("expiration_date")
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string()
        });
    
    let volume = contract.get("latestQuote")
        .and_then(|q| q.get("as"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Extract open interest - try multiple possible field names
    let open_interest = contract.get("open_interest")
        .and_then(|oi| oi.as_u64())
        .or_else(|| contract.get("oi").and_then(|oi| oi.as_u64()))
        .or_else(|| contract.get("openInterest").and_then(|oi| oi.as_u64()))
        .or_else(|| contract.get("outstanding_contracts").and_then(|oi| oi.as_u64()))
        .or_else(|| {
            // Fallback to volume as proxy for open interest when OI is not available
            // This is common in market data APIs where OI is not provided
            contract.get("latestQuote")
                .and_then(|q| q.get("as"))
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(0);
    
    // Calculate Greeks (simplified approximations)
    let implied_volatility = contract.get("implied_volatility")
        .and_then(|iv| iv.as_f64())
        .unwrap_or(0.3);
    
    let delta = if overall_sentiment == "call" { 0.6 } else { -0.4 };
    let gamma = 0.05;
    let theta = -0.02;
    let vega = 0.1;
    
    // Determine signal type
    let signal_type = match (overall_sentiment, option_analysis.contract_type.as_str()) {
        ("call", "short_term") => "BUY_CALL",
        ("call", "leap") => "BUY_CALL",
        ("put", "short_term") => "BUY_PUT", 
        ("put", "leap") => "BUY_PUT",
        _ => "BUY_CALL",
    };
    
    // Calculate risk metrics
    let financial_metrics = if let Some(metrics) = calculate_option_financial_metrics(contract) {
        crate::types::FinancialMetrics {
            sharpe_ratio: metrics.sharpe,
            sortino_ratio: metrics.sortino,
            calmar_ratio: metrics.calmar,
            max_drawdown: metrics.max_drawdown,
            volatility: metrics.volatility,
            composite_score: metrics.composite_score,
            kelly_fraction: metrics.kelly_fraction,
            var_95: metrics.volatility * 1.645, // 95% VaR approximation
            expected_shortfall: metrics.volatility * 2.0, // Simplified ES
        }
    } else {
        crate::types::FinancialMetrics {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_drawdown: 0.0,
            volatility: 0.0,
            composite_score: 0.0,
            kelly_fraction: 0.0,
            var_95: 0.0,
            expected_shortfall: 0.0,
        }
    };
    
    // Calculate expected return and max loss
    let expected_return = option_analysis.option_score * 0.1; // Convert score to return
    let max_loss = entry_price; // For long options, max loss is premium paid
    
    // Determine time horizon
    let time_horizon = if option_analysis.contract_type == "leap" { "LEAP" } else { "SHORT_TERM" };
    
    // Calculate risk score (0-1, higher = more risky)
    let risk_score = (1.0 - financial_metrics.sharpe_ratio / 3.0).clamp(0.0, 1.0);
    
    // Generate reasoning
    let mut reasoning = Vec::new();
    reasoning.push(format!("Sentiment: {overall_sentiment} (confidence: {sentiment_score:.2})"));
    reasoning.extend(option_analysis.undervalued_indicators.clone());
    if financial_metrics.sharpe_ratio > 1.0 {
        reasoning.push("Strong risk-adjusted returns".to_string());
    }
    if volume > 1000 {
        reasoning.push("High liquidity".to_string());
    }
    
    crate::types::TradingSignal {
        symbol: symbol.to_string(),
        signal_type: signal_type.to_string(),
        confidence: option_analysis.option_score / 10.0, // Normalize to 0-1
        sentiment_score,
        risk_score,
        expected_return,
        max_loss,
        time_horizon: time_horizon.to_string(),
        entry_price,
        strike_price,
        expiration_date,
        volume,
        open_interest,
        implied_volatility,
        delta,
        gamma,
        theta,
        vega,
        financial_metrics,
        reasoning,
    }
}

// Calculate market summary from trading signals
pub fn calculate_market_summary(
    trading_signals: &[crate::types::TradingSignal],
    _sentiment_analysis: &[crate::types::SentimentAnalysis],
) -> crate::types::MarketSummary {
    let total_signals = trading_signals.len();
    let bullish_signals = trading_signals.iter()
        .filter(|s| s.signal_type.contains("CALL"))
        .count();
    let bearish_signals = trading_signals.iter()
        .filter(|s| s.signal_type.contains("PUT"))
        .count();
    
    let high_confidence_signals = trading_signals.iter()
        .filter(|s| s.confidence > 0.7)
        .count();
    
    let overall_confidence = if total_signals > 0 {
        trading_signals.iter().map(|s| s.confidence).sum::<f64>() / total_signals as f64
    } else {
        0.0
    };
    
    // Determine market sentiment
    let market_sentiment = if bullish_signals > (bearish_signals as f64 * 1.5) as usize {
        "BULLISH"
    } else if bearish_signals > (bullish_signals as f64 * 1.5) as usize {
        "BEARISH"
    } else {
        "NEUTRAL"
    };
    
    // Determine risk level
    let avg_risk = if total_signals > 0 {
        trading_signals.iter().map(|s| s.risk_score).sum::<f64>() / total_signals as f64
    } else {
        0.5
    };
    
    let risk_level = if avg_risk < 0.3 { "LOW" } else if avg_risk < 0.7 { "MEDIUM" } else { "HIGH" };
    
    // Calculate recommended position size based on confidence and risk
    let recommended_position_size = (overall_confidence * (1.0 - avg_risk) * 100.0).min(20.0);
    
    crate::types::MarketSummary {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_signals,
        bullish_signals,
        bearish_signals,
        high_confidence_signals,
        market_sentiment: market_sentiment.to_string(),
        overall_confidence,
        risk_level: risk_level.to_string(),
        recommended_position_size,
    }
}

// Calculate portfolio risk metrics
pub fn calculate_risk_metrics(trading_signals: &[crate::types::TradingSignal]) -> crate::types::RiskMetrics {
    let symbols: Vec<String> = trading_signals.iter().map(|s| s.symbol.clone()).collect();
    
    // Simplified correlation matrix (identity matrix for now)
    let correlation_matrix = vec![vec![1.0; symbols.len()]; symbols.len()];
    
    // Calculate portfolio VaR (simplified)
    let portfolio_var = trading_signals.iter()
        .map(|s| s.financial_metrics.var_95 * s.expected_return)
        .sum::<f64>() / trading_signals.len() as f64;
    
    // Calculate max portfolio drawdown
    let max_portfolio_drawdown = trading_signals.iter()
        .map(|s| s.financial_metrics.max_drawdown)
        .fold(0.0, f64::max);
    
    // Calculate diversification score
    let diversification_score = if symbols.len() > 1 {
        1.0 - (1.0 / symbols.len() as f64)
    } else {
        0.0
    };
    
    // Simplified sector exposure (all equal for now)
    let mut sector_exposure = std::collections::HashMap::new();
    sector_exposure.insert("TECH".to_string(), 0.3);
    sector_exposure.insert("FINANCE".to_string(), 0.2);
    sector_exposure.insert("HEALTHCARE".to_string(), 0.2);
    sector_exposure.insert("OTHER".to_string(), 0.3);
    
    // Determine volatility regime
    let avg_volatility = trading_signals.iter()
        .map(|s| s.financial_metrics.volatility)
        .sum::<f64>() / trading_signals.len() as f64;
    
    let volatility_regime = if avg_volatility < 0.2 { "LOW" } else if avg_volatility < 0.4 { "NORMAL" } else { "HIGH" };
    
    crate::types::RiskMetrics {
        portfolio_var,
        max_portfolio_drawdown,
        correlation_matrix,
        diversification_score,
        sector_exposure,
        volatility_regime: volatility_regime.to_string(),
    }
}
