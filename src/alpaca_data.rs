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
    
    // Analyze both contract types and select the best one
    let mut short_term_score = 0.0;
    let mut leap_score = 0.0;
    let mut short_term_contract = None;
    let mut leap_contract = None;
    
    // Calculate scores for both contract types
    if let Some(contract) = hoi_result.short_term {
        short_term_score = calculate_option_score(&contract, spot_price, composite_score);
        short_term_contract = Some(contract);
    }
    
    if let Some(contract) = hoi_result.leap {
        leap_score = calculate_option_score(&contract, spot_price, composite_score);
        leap_contract = Some(contract);
    }
    
    // Select the best contract based on score
    if short_term_score > leap_score {
        // Short-term is better
        if let Some(contract) = short_term_contract {
            options_analysis.push(serde_json::json!({
                "contract_type": "short_term",
                "contract": contract,
                "option_score": short_term_score,
                "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
            }));
        }
    } else if leap_score > 0.0 {
        // LEAP is better (or only option available)
        if let Some(contract) = leap_contract {
            options_analysis.push(serde_json::json!({
                "contract_type": "leap",
                "contract": contract,
                "option_score": leap_score,
                "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
            }));
        }
    } else if short_term_score > 0.0 {
        // Fallback to short-term if LEAP score is 0
        if let Some(contract) = short_term_contract {
            options_analysis.push(serde_json::json!({
                "contract_type": "short_term",
                "contract": contract,
                "option_score": short_term_score,
                "undervalued_indicators": calculate_undervalued_indicators(&contract, spot_price, composite_score)
            }));
        }
    }
    
    Ok(serde_json::json!({
        "symbol": symbol,
        "underlying_metrics": underlying_metrics,
        "options_analysis": options_analysis,
        "error": hoi_result.error
    }))
}

// Debug function to log contract data structure
fn debug_contract_data(contract: &Value, symbol: &str) {
    eprintln!("DEBUG: Contract data for {}: {}", symbol, serde_json::to_string_pretty(contract).unwrap_or_else(|_| "Failed to serialize".to_string()));
    
    if let Some(contract_key) = contract.get("contract_key").and_then(|k| k.as_str()) {
        eprintln!("DEBUG: Contract key: {}", contract_key);
        eprintln!("DEBUG: Parsed strike price: {}", parse_strike_price_from_contract_key(contract_key));
        eprintln!("DEBUG: Parsed expiration date: {}", parse_expiration_date_from_contract_key(contract_key));
    } else {
        eprintln!("DEBUG: No contract_key found in contract data");
    }
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
                        // Debug the contract data
                        debug_contract_data(&contract_data, symbol);
                        result.short_term = Some(contract_data);
                    }
                    if contracts.len() > 1 {
                        let mut contract_data = contracts[1].1.clone();
                        // Add contract key information to the contract data
                        contract_data["contract_key"] = serde_json::Value::String(contracts[1].0.clone());
                        // Debug the contract data
                        debug_contract_data(&contract_data, symbol);
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
    
    // Time to expiry factor (prefer contracts with reasonable time decay)
    if let Some(expiry_str) = contract.get("contract_key").and_then(|k| k.as_str()) {
        if let Some(days_to_expiry) = parse_days_to_expiry(expiry_str) {
            if days_to_expiry < 30 {
                // Very short-term options get penalty (high theta decay)
                score -= 2.0;
            } else if days_to_expiry > 365 {
                // Very long-term options get slight penalty (less leverage)
                score -= 1.0;
            } else {
                // Sweet spot: 30-365 days get bonus
                score += 1.0;
            }
        }
    }
    
    // Liquidity factor (prefer higher open interest)
    if let Some(oi) = contract.get("open_interest").and_then(|oi| oi.as_u64()) {
        if oi > 1000 {
            score += 2.0; // High liquidity bonus
        } else if oi > 100 {
            score += 1.0; // Medium liquidity bonus
        } else if oi < 50 {
            score -= 1.0; // Low liquidity penalty
        }
    }
    
    score
}

// Helper function to parse days to expiry from contract key
fn parse_days_to_expiry(contract_key: &str) -> Option<u32> {
    // Contract key format: "AAPL240920C00150000" (AAPL + YYMMDD + C/P + Strike)
    if contract_key.len() >= 10 {
        let date_part = &contract_key[4..10]; // Extract YYMMDD
        if let Ok(date_str) = date_part.parse::<u32>() {
            let year = 2000 + (date_str / 10000);
            let month = (date_str % 10000) / 100;
            let day = date_str % 100;
            
            // Simple calculation: estimate days to expiry
            // This is a rough approximation - in production you'd want proper date handling
            let current_year = 2025; // Assuming current year
            let current_month = 9;   // Assuming current month (September)
            let current_day = 16;    // Assuming current day
            
            if year == current_year {
                let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                let mut days_to_expiry = 0;
                
                // Add days from current month to expiry month
                for m in current_month..month {
                    days_to_expiry += days_in_month[m as usize - 1];
                }
                
                // Add remaining days
                days_to_expiry += day as i32 - current_day as i32;
                
                if days_to_expiry > 0 {
                    return Some(days_to_expiry as u32);
                }
            }
        }
    }
    None
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
    
    // Get spot price for proper moneyness calculation
    let spot_price = contract.get("underlying_price")
        .and_then(|p| p.as_f64())
        .or_else(|| contract.get("spot_price").and_then(|p| p.as_f64()))
        .or_else(|| contract.get("last_price").and_then(|p| p.as_f64()))
        .unwrap_or_else(|| {
            // Estimate spot price from strike and entry price
            if strike_price > 0.0 {
                strike_price * 0.95 // Assume slightly out-of-the-money
            } else {
                entry_price * 100.0 // Rough estimate
            }
        });
    
    // Calculate proper moneyness (spot/strike, not entry/strike)
    let moneyness = if strike_price > 0.0 { spot_price / strike_price } else { 1.0 };
    
    // Estimate expected return based on moneyness, volatility, and time to expiry
    let base_return = if moneyness > 0.9 && moneyness < 1.1 {
        // Near-the-money options have higher expected returns
        implied_volatility * 0.8
    } else if moneyness > 0.8 && moneyness < 1.2 {
        // Close to money
        implied_volatility * 0.6
    } else {
        // Out-of-the-money options have lower expected returns
        implied_volatility * 0.3
    };
    
    // Adjust for time to expiry (longer time = higher potential return)
    let time_factor = if time_to_expiry > 365.0 {
        1.5 // LEAPs have higher potential
    } else if time_to_expiry > 90.0 {
        1.2 // Medium-term options
    } else {
        1.0 // Short-term options
    };
    
    let expected_return = base_return * time_factor;
    
    // Calculate volatility (use implied volatility as base)
    let volatility = implied_volatility * (1.0 + (volume / 10000.0).min(1.0));
    
    // Calculate Sharpe ratio (more realistic)
    let risk_free_rate = get_dynamic_risk_free_rate();
    let daily_risk_free = risk_free_rate / 252.0;
    let sharpe = if volatility > 0.0 {
        let excess_return = expected_return - daily_risk_free;
        excess_return / volatility
    } else {
        0.0
    };
    
    // Calculate Sortino ratio (using dynamic downside deviation)
    let downside_deviation = calculate_dynamic_downside_deviation(volatility, expected_return, time_to_expiry);
    let sortino = if downside_deviation > 0.0 {
        let excess_return = expected_return - daily_risk_free;
        excess_return / downside_deviation
    } else {
        0.0
    };
    
    // Calculate max drawdown (more realistic for options)
    let max_drawdown = if time_to_expiry > 0.0 {
        let time_factor = (time_to_expiry / 30.0).min(1.0);
        let base_drawdown = 0.2 + (implied_volatility * 0.5); // 20-50% based on IV
        base_drawdown * (1.0 - time_factor * 0.3) // Slightly lower for longer-term options
    } else {
        0.3 // Default 30% for unknown expiry
    };
    
    // Calculate CAGR (annualized expected return)
    let cagr = expected_return * 252.0; // Annualize daily return
    
    // Calculate Calmar ratio
    let calmar = if max_drawdown > 0.0 { cagr / max_drawdown } else { 0.0 };
    
    // Calculate Kelly fraction (options-specific approach)
    let kelly = if volatility > 0.0 && entry_price > 0.0 {
        // For options, use a more sophisticated approach
        let win_prob = if moneyness > 0.95 && moneyness < 1.05 {
            0.60 // Near-the-money options
        } else if moneyness > 0.85 && moneyness < 1.15 {
            0.50 // Close to money
        } else if moneyness > 0.7 && moneyness < 1.3 {
            0.40 // Reasonable moneyness
        } else {
            0.25 // Far out-of-the-money
        };
        
        // Calculate potential win/loss based on option characteristics
        let potential_win = if moneyness > 0.9 {
            // Near-the-money: potential for 50-200% gains
            expected_return * 3.0 + 0.5
        } else {
            // Out-of-the-money: potential for 100-500% gains
            expected_return * 5.0 + 0.2
        };
        
        let potential_loss = 1.0; // Maximum loss is premium paid (normalized)
        
        // Kelly formula: f = (bp - q) / b
        // where b = odds received (potential_win), p = win probability, q = loss probability
        let kelly_raw = (win_prob * potential_win - (1.0 - win_prob) * potential_loss) / potential_win;
        
        // Apply additional factors
        let liquidity_factor = if volume > 1000.0 { 1.0 } else if volume > 500.0 { 0.8 } else { 0.6 };
        let time_factor = if time_to_expiry > 30.0 { 1.0 } else { 0.7 }; // Penalty for very short-term
        
        let adjusted_kelly = kelly_raw * liquidity_factor * time_factor;
        
        // Ensure reasonable bounds
        if adjusted_kelly > 0.0 {
            adjusted_kelly.max(0.02).min(0.25) // 2-25% position size
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    // Calculate composite score with dynamic weights
    let composite_score = calculate_dynamic_composite_score(sharpe, sortino, calmar, volatility, time_to_expiry);
    
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
    
    // Handle different possible formats
    if contract_key.len() >= 15 {
        // Try the standard format first (last 8 characters)
        let strike_part = &contract_key[contract_key.len()-8..];
        if let Ok(strike_int) = strike_part.parse::<u32>() {
            // Convert from integer representation to decimal (divide by 1000)
            return strike_int as f64 / 1000.0;
        }
        
        // Try alternative format (last 7 characters)
        if contract_key.len() >= 14 {
            let strike_part = &contract_key[contract_key.len()-7..];
            if let Ok(strike_int) = strike_part.parse::<u32>() {
                return strike_int as f64 / 1000.0;
            }
        }
        
        // Try alternative format (last 6 characters)
        if contract_key.len() >= 13 {
            let strike_part = &contract_key[contract_key.len()-6..];
            if let Ok(strike_int) = strike_part.parse::<u32>() {
                return strike_int as f64 / 1000.0;
            }
        }
    }
    
    // If all parsing attempts fail, try to extract any numeric part at the end
    let mut numeric_end = String::new();
    for c in contract_key.chars().rev() {
        if c.is_ascii_digit() {
            numeric_end.push(c);
        } else {
            break;
        }
    }
    
    if !numeric_end.is_empty() {
        numeric_end = numeric_end.chars().rev().collect();
        if let Ok(strike_int) = numeric_end.parse::<u32>() {
            // Try different scaling factors
            if strike_int > 1000000 {
                return strike_int as f64 / 1000.0; // 6+ digits, likely in thousandths
            } else if strike_int > 10000 {
                return strike_int as f64 / 100.0;  // 5 digits, likely in hundredths
            } else {
                return strike_int as f64; // 4 or fewer digits, likely whole dollars
            }
        }
    }
    
    0.0
}

// Parse expiration date from contract key (format: SYMBOLYYMMDDC/PSSTRIKEPRICE)
fn parse_expiration_date_from_contract_key(contract_key: &str) -> String {
    // Contract key format: SYMBOLYYMMDDC/PSSTRIKEPRICE
    // Example: AAPL240119C00150000 (AAPL, 2024-01-19, Call, $150.00)
    
    // Try different positions for the date part
    let possible_positions = vec![
        (15, 9),  // Standard format: last 15 chars, skip last 9
        (14, 8),  // Alternative format: last 14 chars, skip last 8
        (13, 7),  // Alternative format: last 13 chars, skip last 7
        (12, 6),  // Alternative format: last 12 chars, skip last 6
    ];
    
    for (total_len, skip_end) in possible_positions {
        if contract_key.len() >= total_len {
            let start_pos = contract_key.len() - total_len;
            let end_pos = contract_key.len() - skip_end;
            
            if end_pos > start_pos && end_pos <= contract_key.len() {
                let date_part = &contract_key[start_pos..end_pos];
                if date_part.len() == 6 {
                    // Parse YYMMDD format
                    if let (Ok(year), Ok(month), Ok(day)) = (
                        date_part[0..2].parse::<u32>(),
                        date_part[2..4].parse::<u32>(),
                        date_part[4..6].parse::<u32>(),
                    ) {
                        // Validate date components
                        if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                            // Convert 2-digit year to 4-digit (assuming 20xx)
                            let full_year = 2000 + year;
                            return format!("{:04}-{:02}-{:02}", full_year, month, day);
                        }
                    }
                }
            }
        }
    }
    
    // If standard parsing fails, try to find any 6-digit sequence that looks like a date
    for i in 0..=contract_key.len().saturating_sub(6) {
        let date_part = &contract_key[i..i+6];
        if date_part.chars().all(|c| c.is_ascii_digit()) {
            if let (Ok(year), Ok(month), Ok(day)) = (
                date_part[0..2].parse::<u32>(),
                date_part[2..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
            ) {
                // Validate date components
                if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                    // Convert 2-digit year to 4-digit (assuming 20xx)
                    let full_year = 2000 + year;
                    return format!("{:04}-{:02}-{:02}", full_year, month, day);
                }
            }
        }
    }
    
    String::new()
}

// Fundamental risk assessment for a symbol
pub fn assess_fundamental_risk(symbol: &str, contract: &serde_json::Value) -> (f64, Vec<String>) {
    let mut risk_factors = Vec::new();
    let mut risk_score = 0.0;
    
    // Extract basic price data
    let entry_price = contract.get("latestQuote")
        .and_then(|q| q.get("ap"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);
    
    let volume = contract.get("latestQuote")
        .and_then(|q| q.get("as"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let open_interest = contract.get("open_interest")
        .and_then(|oi| oi.as_u64())
        .unwrap_or(0);
    
    // 1. Price-based risk filters
    if entry_price < 0.05 {
        risk_score += 0.3;
        risk_factors.push("Extremely low price (<$0.05) - high risk of delisting".to_string());
    } else if entry_price < 0.10 {
        risk_score += 0.2;
        risk_factors.push("Very low price (<$0.10) - penny stock risk".to_string());
    }
    
    // 2. Liquidity risk filters
    if volume < 100 {
        risk_score += 0.25;
        risk_factors.push("Very low volume (<100) - execution risk".to_string());
    } else if volume < 500 {
        risk_score += 0.15;
        risk_factors.push("Low volume (<500) - liquidity concerns".to_string());
    }
    
    if open_interest < 50 {
        risk_score += 0.2;
        risk_factors.push("Very low open interest (<50) - limited liquidity".to_string());
    }
    
    // 3. Sector-specific risk filters
    let sector_risk = classify_sector_risk(symbol);
    risk_score += sector_risk.0;
    risk_factors.extend(sector_risk.1);
    
    // 4. Volatility risk filter
    let implied_volatility = contract.get("implied_volatility")
        .and_then(|iv| iv.as_f64())
        .unwrap_or(0.3);
    
    if implied_volatility > 1.0 {
        risk_score += 0.3;
        risk_factors.push("Extreme volatility (>100%) - high risk".to_string());
    } else if implied_volatility > 0.8 {
        risk_score += 0.2;
        risk_factors.push("Very high volatility (>80%) - elevated risk".to_string());
    }
    
    // 5. Market cap estimation (rough)
    let estimated_market_cap = estimate_market_cap(symbol, entry_price);
    if estimated_market_cap < 50_000_000.0 { // < $50M
        risk_score += 0.25;
        risk_factors.push("Small cap stock (<$50M) - high volatility risk".to_string());
    }
    
    // Cap risk score at 1.0
    (risk_score.min(1.0), risk_factors)
}

// Classify sector-specific risks
fn classify_sector_risk(symbol: &str) -> (f64, Vec<String>) {
    let mut risk_score = 0.0;
    let mut risk_factors = Vec::new();
    
    // Biotech/Pharma risk
    if is_biotech_symbol(symbol) {
        risk_score += 0.3;
        risk_factors.push("Biotech sector - high regulatory and clinical trial risk".to_string());
    }
    
    // Small cap biotech risk
    if is_small_biotech(symbol) {
        risk_score += 0.2;
        risk_factors.push("Small biotech - extreme volatility and binary outcomes".to_string());
    }
    
    // Energy sector risk
    if is_energy_symbol(symbol) {
        risk_score += 0.15;
        risk_factors.push("Energy sector - commodity price volatility".to_string());
    }
    
    // Mining/Materials risk
    if is_materials_symbol(symbol) {
        risk_score += 0.2;
        risk_factors.push("Materials sector - commodity and economic cycle risk".to_string());
    }
    
    (risk_score, risk_factors)
}

// Helper functions for sector classification
fn is_biotech_symbol(symbol: &str) -> bool {
    let biotech_indicators = [
        "BIO", "PHARMA", "THERA", "GEN", "CELL", "MED", "CURE", "LIFE", "HEALTH",
        "ATYR", "OSCR", "RCAT", "AREC", "HYLN", "UUUU" // Known biotech tickers
    ];
    biotech_indicators.iter().any(|&indicator| symbol.contains(indicator))
}

fn is_small_biotech(symbol: &str) -> bool {
    let small_biotech = ["ATYR", "OSCR", "RCAT", "AREC", "HYLN", "UUUU"];
    small_biotech.contains(&symbol)
}

fn is_energy_symbol(symbol: &str) -> bool {
    let energy_indicators = ["OIL", "GAS", "ENERGY", "POWER", "FUEL", "DRILL"];
    energy_indicators.iter().any(|&indicator| symbol.contains(indicator))
}

fn is_materials_symbol(symbol: &str) -> bool {
    let materials_indicators = ["MINING", "METAL", "GOLD", "SILVER", "COPPER", "STEEL"];
    materials_indicators.iter().any(|&indicator| symbol.contains(indicator))
}

// Rough market cap estimation
fn estimate_market_cap(symbol: &str, price: f64) -> f64 {
    // This is a very rough estimation - in production, you'd want real market cap data
    let estimated_shares = match symbol {
        // Large caps
        "AAPL" | "MSFT" | "GOOGL" | "AMZN" | "TSLA" => 15_000_000_000.0,
        // Mid caps
        "NIO" | "BAC" => 2_000_000_000.0,
        // Small caps (most biotech)
        _ => 50_000_000.0,
    };
    price * estimated_shares
}

// Convert option analysis to trading signal with fundamental risk filtering
pub fn convert_to_trading_signal(
    symbol: &str,
    option_analysis: &crate::types::OptionAnalysis,
    sentiment_score: f64,
    overall_sentiment: &str,
) -> crate::types::TradingSignal {
    let contract = &option_analysis.contract;
    
    // Perform fundamental risk assessment
    let (fundamental_risk_score, risk_factors) = assess_fundamental_risk(symbol, contract);
    
    // Extract option data
    let entry_price = contract.get("latestQuote")
        .and_then(|q| q.get("ap"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);
    
    // Extract strike price from contract key with debugging
    let strike_price = if let Some(contract_key) = contract.get("contract_key").and_then(|k| k.as_str()) {
        let parsed_strike = parse_strike_price_from_contract_key(contract_key);
        if parsed_strike > 0.0 {
            parsed_strike
        } else {
            // Try to extract from other possible fields
            contract.get("strike_price")
                .and_then(|s| s.as_f64())
                .or_else(|| contract.get("strike").and_then(|s| s.as_f64()))
                .unwrap_or(0.0)
        }
    } else {
        // Fallback: try to extract from other possible fields
        contract.get("strike_price")
            .and_then(|s| s.as_f64())
            .or_else(|| contract.get("strike").and_then(|s| s.as_f64()))
            .unwrap_or(0.0)
    };
    
    // Extract expiration date from contract key with debugging
    let expiration_date = if let Some(contract_key) = contract.get("contract_key").and_then(|k| k.as_str()) {
        let parsed_date = parse_expiration_date_from_contract_key(contract_key);
        if !parsed_date.is_empty() {
            parsed_date
        } else {
            // Fallback to contract field if available
            contract.get("expiration_date")
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string()
        }
    } else {
        // Fallback to contract field if available
        contract.get("expiration_date")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string()
    };
    
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
    
    // Calculate Greeks dynamically based on option characteristics
    let implied_volatility = contract.get("implied_volatility")
        .and_then(|iv| iv.as_f64())
        .or_else(|| contract.get("iv").and_then(|iv| iv.as_f64()))
        .or_else(|| contract.get("impliedVolatility").and_then(|iv| iv.as_f64()))
        .unwrap_or_else(|| {
            // Estimate IV based on volume and time to expiry
            let time_to_expiry = calculate_time_to_expiry(contract);
            let volume_factor = (volume as f64 / 10000.0).min(1.0);
            let base_iv = 0.2 + (time_to_expiry / 365.0) * 0.1; // 20-30% base IV
            base_iv + volume_factor * 0.1 // Add up to 10% based on volume
        });
    
    // Get underlying asset price (spot price) - this should be different from entry_price
    let spot_price = contract.get("underlying_price")
        .and_then(|p| p.as_f64())
        .or_else(|| contract.get("spot_price").and_then(|p| p.as_f64()))
        .or_else(|| contract.get("last_price").and_then(|p| p.as_f64()))
        .or_else(|| {
            // Estimate spot price from strike and moneyness
            if strike_price > 0.0 {
                // Assume at-the-money or slightly out-of-the-money
                Some(strike_price * 0.95) // 5% below strike as estimate
            } else {
                Some(entry_price * 100.0) // Rough estimate if no other data
            }
        });
    
    // Calculate Greeks using Black-Scholes approximations
    let (delta, gamma, theta, vega) = if let Some(spot) = spot_price {
        calculate_option_greeks(
            spot, strike_price, implied_volatility, 
            calculate_time_to_expiry(contract), overall_sentiment == "call"
        )
    } else {
        (0.0, 0.0, 0.0, 0.0) // Fallback if no spot price available
    };
    
    // Determine signal type based on sentiment score and overall sentiment
    // Use more aggressive thresholds to create variety in signals
    let signal_type = if sentiment_score > 0.9 {
        // Very strong positive sentiment -> calls
        match option_analysis.contract_type.as_str() {
            "short_term" => "BUY_CALL",
            "leap" => "BUY_CALL",
            _ => "BUY_CALL",
        }
    } else if sentiment_score < 0.2 {
        // Strong negative sentiment -> puts
        match option_analysis.contract_type.as_str() {
            "short_term" => "BUY_PUT",
            "leap" => "BUY_PUT",
            _ => "BUY_PUT",
        }
    } else if sentiment_score > 0.7 {
        // Moderate positive sentiment -> calls
        match option_analysis.contract_type.as_str() {
            "short_term" => "BUY_CALL",
            "leap" => "BUY_CALL",
            _ => "BUY_CALL",
        }
    } else if sentiment_score < 0.4 {
        // Moderate negative sentiment -> puts
        match option_analysis.contract_type.as_str() {
            "short_term" => "BUY_PUT",
            "leap" => "BUY_PUT",
            _ => "BUY_PUT",
        }
    } else {
        // Neutral sentiment (0.4-0.7) - use overall market sentiment
        match (overall_sentiment, option_analysis.contract_type.as_str()) {
            ("call", "short_term") => "BUY_CALL",
            ("call", "leap") => "BUY_CALL",
            ("put", "short_term") => "BUY_PUT", 
            ("put", "leap") => "BUY_PUT",
            _ => "BUY_CALL",
        }
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
            var_95: calculate_dynamic_var_95(metrics.volatility, metrics.mean_return, calculate_time_to_expiry(contract)),
            expected_shortfall: calculate_dynamic_expected_shortfall(metrics.volatility, metrics.mean_return, calculate_time_to_expiry(contract)),
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
    
    // Calculate expected return dynamically based on option characteristics
    let expected_return = if let Some(spot) = spot_price {
        calculate_expected_option_return(
            entry_price, strike_price, spot, implied_volatility, 
            calculate_time_to_expiry(contract), overall_sentiment == "call",
            volume, open_interest
        )
    } else {
        0.0 // Fallback if no spot price available
    };
    let max_loss = entry_price; // For long options, max loss is premium paid
    
    // Determine time horizon
    let time_horizon = if option_analysis.contract_type == "leap" { "LEAP" } else { "SHORT_TERM" };
    
    // Calculate combined risk score (technical + fundamental)
    let technical_risk_score = calculate_dynamic_risk_score(
        implied_volatility, financial_metrics.max_drawdown, 
        volume, open_interest, calculate_time_to_expiry(contract)
    );
    
    // Combine technical and fundamental risk (weighted average)
    let risk_score = (technical_risk_score * 0.6 + fundamental_risk_score * 0.4).min(1.0);
    
    // Generate reasoning
    let mut reasoning = Vec::new();
    reasoning.push(format!("Sentiment: {overall_sentiment} (confidence: {sentiment_score:.2})"));
    reasoning.extend(option_analysis.undervalued_indicators.clone());
    
    // Add fundamental risk warnings if present
    if !risk_factors.is_empty() {
        reasoning.extend(risk_factors.iter().take(2).cloned()); // Show top 2 risk factors
    }
    
    // Add positive indicators
    if financial_metrics.sharpe_ratio > 1.0 {
        reasoning.push("Strong risk-adjusted returns".to_string());
    }
    if volume > 1000 {
        reasoning.push("High volume".to_string());
    }
    
    // Calculate confidence score dynamically based on multiple factors
    let base_confidence = calculate_dynamic_confidence(
        sentiment_score, option_analysis.option_score, 
        financial_metrics.composite_score, volume, open_interest
    );
    
    // Apply fundamental risk penalty to confidence
    let confidence = if fundamental_risk_score > 0.7 {
        // High fundamental risk - severely reduce confidence
        base_confidence * 0.3
    } else if fundamental_risk_score > 0.5 {
        // Medium fundamental risk - moderate reduction
        base_confidence * 0.6
    } else if fundamental_risk_score > 0.3 {
        // Low-medium fundamental risk - slight reduction
        base_confidence * 0.8
    } else {
        // Low fundamental risk - no penalty
        base_confidence
    };
    
    crate::types::TradingSignal {
        symbol: symbol.to_string(),
        signal_type: signal_type.to_string(),
        confidence,
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
    let recommended_position_size = calculate_dynamic_position_size(overall_confidence, avg_risk, total_signals);
    
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
    
    // Calculate dynamic sector exposure based on actual symbols
    let sector_exposure = calculate_dynamic_sector_exposure(&symbols);
    
    // Determine volatility regime
    let avg_volatility = trading_signals.iter()
        .map(|s| s.financial_metrics.volatility)
        .sum::<f64>() / trading_signals.len() as f64;
    
    let volatility_regime = if avg_volatility < 0.2 { "LOW" } else if avg_volatility < 0.4 { "NORMAL" } else { "HIGH" };
    
    crate::types::RiskMetrics {
        portfolio_var,
        max_portfolio_drawdown,
        diversification_score,
        sector_exposure,
        volatility_regime: volatility_regime.to_string(),
    }
}

// Calculate option Greeks using Black-Scholes approximations
fn calculate_option_greeks(
    spot_price: f64,
    strike_price: f64,
    implied_volatility: f64,
    time_to_expiry: f64,
    is_call: bool,
) -> (f64, f64, f64, f64) {
    if time_to_expiry <= 0.0 || implied_volatility <= 0.0 || strike_price <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    
    let sqrt_t = (time_to_expiry / 365.0).sqrt();
    let d1 = ((spot_price / strike_price).ln() + 0.5 * implied_volatility * implied_volatility * time_to_expiry / 365.0) 
             / (implied_volatility * sqrt_t);
    let d2 = d1 - implied_volatility * sqrt_t;
    
    // Normal CDF approximation
    let n_d1 = 0.5 * (1.0 + erf_approximation(d1 / 1.4142135623730951));
    let n_d2 = 0.5 * (1.0 + erf_approximation(d2 / 1.4142135623730951));
    
    // Normal PDF
    let phi_d1 = (-0.5 * d1 * d1).exp() / (2.0 * std::f64::consts::PI).sqrt();
    
    // Calculate Greeks
    let delta = if is_call { n_d1 } else { n_d1 - 1.0 };
    let gamma = phi_d1 / (spot_price * implied_volatility * sqrt_t);
    let theta = -(spot_price * phi_d1 * implied_volatility) / (2.0 * sqrt_t) 
                - 0.01 * strike_price * (-0.05 * time_to_expiry / 365.0).exp() * n_d2;
    let vega = spot_price * phi_d1 * sqrt_t / 100.0; // Per 1% change in IV
    
    (delta, gamma, theta, vega)
}

// Error function approximation for normal CDF
fn erf_approximation(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    
    sign * y
}

// Calculate expected option return dynamically
fn calculate_expected_option_return(
    entry_price: f64,
    strike_price: f64,
    spot_price: f64,
    implied_volatility: f64,
    time_to_expiry: f64,
    is_call: bool,
    volume: u64,
    open_interest: u64,
) -> f64 {
    if entry_price <= 0.0 || time_to_expiry <= 0.0 || spot_price <= 0.0 {
        return 0.0;
    }
    
    // Base expected return from moneyness (spot/strike ratio)
    let moneyness = if strike_price > 0.0 { spot_price / strike_price } else { 1.0 };
    let base_return = if is_call {
        // For calls, higher moneyness = higher expected return
        (moneyness - 0.8).max(0.0) * 0.5
    } else {
        // For puts, lower moneyness = higher expected return
        (0.8 - moneyness).max(0.0) * 0.5
    };
    
    // Adjust for time decay
    let time_factor = (time_to_expiry / 30.0).min(1.0); // Favor longer-term options
    
    // Adjust for liquidity (volume and open interest)
    let liquidity_factor = ((volume as f64 / 1000.0).min(1.0) + (open_interest as f64 / 1000.0).min(1.0)) / 2.0;
    
    // Adjust for volatility (higher IV = higher potential return but also higher risk)
    let volatility_factor = (implied_volatility / 0.3).min(2.0); // Cap at 2x normal volatility
    
    // Combine factors
    let expected_return = base_return * time_factor * (0.5 + 0.5 * liquidity_factor) * volatility_factor;
    
    // Cap at reasonable levels (0-100% return)
    expected_return.clamp(0.0, 1.0)
}

// Calculate dynamic risk score
fn calculate_dynamic_risk_score(
    implied_volatility: f64,
    max_drawdown: f64,
    volume: u64,
    open_interest: u64,
    time_to_expiry: f64,
) -> f64 {
    // Volatility risk (0-0.4)
    let vol_risk = (implied_volatility / 0.5).min(1.0) * 0.4;
    
    // Drawdown risk (0-0.3)
    let drawdown_risk = (max_drawdown / 0.5).min(1.0) * 0.3;
    
    // Liquidity risk (0-0.2) - lower volume/OI = higher risk
    let liquidity_risk = (1.0 - ((volume as f64 / 10000.0).min(1.0) + (open_interest as f64 / 10000.0).min(1.0)) / 2.0) * 0.2;
    
    // Time decay risk (0-0.1) - shorter expiry = higher risk
    let time_risk = (1.0 - (time_to_expiry / 30.0).min(1.0)) * 0.1;
    
    (vol_risk + drawdown_risk + liquidity_risk + time_risk).clamp(0.0, 1.0)
}

// Calculate dynamic confidence score
fn calculate_dynamic_confidence(
    sentiment_score: f64,
    option_score: f64,
    composite_score: f64,
    volume: u64,
    open_interest: u64,
) -> f64 {
    // Sentiment confidence (0-0.4)
    let sentiment_confidence = sentiment_score * 0.4;
    
    // Option quality confidence (0-0.3)
    let option_confidence = (option_score / 10.0).min(1.0) * 0.3;
    
    // Financial metrics confidence (0-0.2)
    let financial_confidence = (composite_score / 5.0).min(1.0) * 0.2;
    
    // Liquidity confidence (0-0.1)
    let liquidity_confidence = ((volume as f64 / 10000.0).min(1.0) + (open_interest as f64 / 10000.0).min(1.0)) / 2.0 * 0.1;
    
    (sentiment_confidence + option_confidence + financial_confidence + liquidity_confidence).clamp(0.0, 1.0)
}

// Calculate dynamic VaR (95% confidence)
fn calculate_dynamic_var_95(volatility: f64, mean_return: f64, time_to_expiry: f64) -> f64 {
    if volatility <= 0.0 || time_to_expiry <= 0.0 {
        return 0.0;
    }
    
    // Adjust volatility for time horizon
    let time_adjusted_vol = volatility * (time_to_expiry / 365.0).sqrt();
    
    // Calculate VaR using normal distribution approximation
    // VaR = mean_return - 1.645 * volatility (for 95% confidence)
    let var_95 = mean_return - 1.645 * time_adjusted_vol;
    
    // For options, VaR should represent potential loss as percentage of premium paid
    // If mean_return is 0 or negative, VaR should be based on volatility alone
    if mean_return <= 0.0 {
        // Conservative estimate: potential loss up to 2 standard deviations
        2.0 * time_adjusted_vol
    } else {
        // Normal case: ensure VaR represents potential loss
        var_95.min(0.0).abs()
    }
}

// Calculate dynamic Expected Shortfall (Conditional VaR)
fn calculate_dynamic_expected_shortfall(volatility: f64, mean_return: f64, time_to_expiry: f64) -> f64 {
    if volatility <= 0.0 || time_to_expiry <= 0.0 {
        return 0.0;
    }
    
    // Expected Shortfall is typically 1.2-1.5x VaR for normal distributions
    let var_95 = calculate_dynamic_var_95(volatility, mean_return, time_to_expiry);
    let es_multiplier = 1.0 + (volatility * 0.5).min(0.5); // Higher volatility = higher ES multiplier
    
    var_95 * es_multiplier
}

// Calculate dynamic downside deviation
fn calculate_dynamic_downside_deviation(volatility: f64, expected_return: f64, time_to_expiry: f64) -> f64 {
    if volatility <= 0.0 {
        return 0.0;
    }
    
    // Base downside deviation as percentage of total volatility
    let base_downside_ratio = 0.6 + (expected_return * 0.3).min(0.3); // 60-90% based on expected return
    
    // Adjust for time to expiry (longer expiry = more downside risk)
    let time_factor = 1.0 + (time_to_expiry / 365.0).min(0.5) * 0.2;
    
    volatility * base_downside_ratio * time_factor
}

// Get dynamic risk-free rate (simplified - in production, fetch from API)
fn get_dynamic_risk_free_rate() -> f64 {
    // In a real implementation, this would fetch current Treasury rates
    // For now, use a reasonable estimate based on current market conditions
    let base_rate = 0.045; // 4.5% base rate
    
    // Add some variation based on time (simulate market conditions)
    let time_variation = (chrono::Utc::now().timestamp() % 86400) as f64 / 86400.0 * 0.01; // ±0.5%
    
    (base_rate + time_variation).clamp(0.01, 0.08) // Clamp between 1-8%
}

// Calculate dynamic composite score with adaptive weights
fn calculate_dynamic_composite_score(sharpe: f64, sortino: f64, calmar: f64, volatility: f64, time_to_expiry: f64) -> f64 {
    // Cap extreme values to prevent unrealistic scores
    let capped_sharpe = sharpe.min(3.0); // Cap Sharpe at 3.0
    let capped_sortino = sortino.min(4.0); // Cap Sortino at 4.0
    let capped_calmar = calmar.min(10.0); // Cap Calmar at 10.0
    
    // Base weights
    let mut sharpe_weight = 0.4;
    let mut sortino_weight = 0.4;
    let mut calmar_weight = 0.2;
    
    // Adjust weights based on volatility regime
    if volatility > 0.4 {
        // High volatility: emphasize Sortino ratio (downside protection)
        sortino_weight = 0.5;
        sharpe_weight = 0.3;
        calmar_weight = 0.2;
    } else if volatility < 0.2 {
        // Low volatility: emphasize Sharpe ratio (return efficiency)
        sharpe_weight = 0.5;
        sortino_weight = 0.3;
        calmar_weight = 0.2;
    }
    
    // Adjust for time horizon
    if time_to_expiry > 90.0 {
        // Long-term: emphasize Calmar ratio (drawdown protection)
        calmar_weight = 0.3;
        sharpe_weight = 0.35;
        sortino_weight = 0.35;
    }
    
    let raw_score = sharpe_weight * capped_sharpe + sortino_weight * capped_sortino + calmar_weight * capped_calmar;
    
    // Normalize to reasonable range (0-5)
    raw_score.min(5.0)
}

// Calculate dynamic position size based on multiple factors
fn calculate_dynamic_position_size(confidence: f64, risk: f64, total_signals: usize) -> f64 {
    // Base position size calculation
    let base_size = confidence * (1.0 - risk) * 100.0;
    
    // Adjust for number of signals (more signals = smaller individual positions)
    let diversification_factor = if total_signals > 0 {
        (1.0 / (total_signals as f64).sqrt()).min(1.0)
    } else {
        1.0
    };
    
    // Risk-adjusted cap
    let risk_cap = if risk < 0.3 { 25.0 } else if risk < 0.7 { 15.0 } else { 10.0 };
    
    (base_size * diversification_factor).min(risk_cap)
}

// Calculate dynamic sector exposure based on actual symbols
fn calculate_dynamic_sector_exposure(symbols: &[String]) -> std::collections::HashMap<String, f64> {
    let mut sector_counts = std::collections::HashMap::new();
    let total_symbols = symbols.len() as f64;
    
    // Simple sector classification based on symbol patterns
    for symbol in symbols {
        let sector = classify_symbol_sector(symbol);
        *sector_counts.entry(sector).or_insert(0.0) += 1.0;
    }
    
    // Convert counts to percentages
    let mut sector_exposure = std::collections::HashMap::new();
    for (sector, count) in sector_counts {
        sector_exposure.insert(sector, count / total_symbols);
    }
    
    sector_exposure
}

// Classify symbol into sector (simplified classification)
fn classify_symbol_sector(symbol: &str) -> String {
    let symbol_upper = symbol.to_uppercase();
    
    // Technology sector
    if symbol_upper.starts_with("AAPL") || symbol_upper.starts_with("MSFT") || 
       symbol_upper.starts_with("GOOGL") || symbol_upper.starts_with("AMZN") ||
       symbol_upper.starts_with("META") || symbol_upper.starts_with("NVDA") ||
       symbol_upper.starts_with("TSLA") || symbol_upper.starts_with("NFLX") {
        "TECH".to_string()
    }
    // Financial sector
    else if symbol_upper.starts_with("JPM") || symbol_upper.starts_with("BAC") ||
            symbol_upper.starts_with("WFC") || symbol_upper.starts_with("GS") ||
            symbol_upper.starts_with("MS") || symbol_upper.starts_with("C") {
        "FINANCE".to_string()
    }
    // Healthcare sector
    else if symbol_upper.starts_with("JNJ") || symbol_upper.starts_with("PFE") ||
            symbol_upper.starts_with("UNH") || symbol_upper.starts_with("ABBV") ||
            symbol_upper.starts_with("MRK") || symbol_upper.starts_with("TMO") {
        "HEALTHCARE".to_string()
    }
    // Energy sector
    else if symbol_upper.starts_with("XOM") || symbol_upper.starts_with("CVX") ||
            symbol_upper.starts_with("COP") || symbol_upper.starts_with("EOG") {
        "ENERGY".to_string()
    }
    // Consumer sector
    else if symbol_upper.starts_with("WMT") || symbol_upper.starts_with("PG") ||
            symbol_upper.starts_with("KO") || symbol_upper.starts_with("PEP") {
        "CONSUMER".to_string()
    }
    // Default to other
    else {
        "OTHER".to_string()
    }
}
