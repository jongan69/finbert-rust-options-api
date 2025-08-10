use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use rust_bert::pipelines::sentiment::{SentimentConfig, SentimentModel, SentimentPolarity};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use once_cell::sync::Lazy;

mod alpaca_data;
mod types;

use types::*;

// Global configuration
#[derive(Clone)]
pub struct AppConfig {
    pub max_concurrent_requests: usize,
    pub sentiment_model_path: String,
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
    pub alpaca_base_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            sentiment_model_path: "finbert-sentiment".to_string(),
            alpaca_api_key: std::env::var("APCA_API_KEY_ID").unwrap_or_default(),
            alpaca_secret_key: std::env::var("APCA_API_SECRET_KEY").unwrap_or_default(),
            alpaca_base_url: std::env::var("APCA_BASE_URL").unwrap_or_else(|_| "https://paper-api.alpaca.markets".to_string()),
        }
    }
}

// Thread-safe sentiment model with lazy initialization
static SENTIMENT_MODEL: Lazy<Mutex<Option<SentimentModel>>> = Lazy::new(|| {
    Mutex::new(None)
});

// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
}

// Custom error type for better error handling
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Sentiment analysis failed: {0}")]
    SentimentAnalysis(String),
    #[error("Alpaca API error: {0}")]
    AlpacaApi(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::SentimentAnalysis(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::AlpacaApi(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize configuration
    let config = AppConfig::default();
    
    // Validate required environment variables
    if config.alpaca_api_key.is_empty() || config.alpaca_secret_key.is_empty() {
        eprintln!("❌ Missing required environment variables: ALPACA_API_KEY and ALPACA_SECRET_KEY");
        std::process::exit(1);
    }
    
    // Initialize application state
    let state = Arc::new(AppState { config });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);
    
    // Build our application with routes
    let app = Router::new()
        .route("/analyze", get(analyze_endpoint))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))
        .layer(cors)
        .with_state(state);
    
    // Run the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📊 Analysis endpoint: http://127.0.0.1:3000/analyze");
    println!("❤️  Health check: http://127.0.0.1:3000/health");
    println!("📈 Metrics: http://127.0.0.1:3000/metrics");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// Metrics endpoint for monitoring
async fn metrics_endpoint(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "config": {
            "max_concurrent_requests": state.config.max_concurrent_requests,
            "alpaca_base_url": state.config.alpaca_base_url,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn analyze_endpoint(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let start_time = std::time::Instant::now();
    
    match perform_analysis(&state.config).await {
        Ok(mut response) => {
            // Update execution metadata with actual timing
            response.execution_metadata.processing_time_ms = start_time.elapsed().as_millis() as u64;
            
            println!("✅ Analysis completed successfully in {}ms", response.execution_metadata.processing_time_ms);
            Ok((StatusCode::OK, Json(response)).into_response())
        }
        Err(e) => {
            println!("❌ Analysis failed: {e}");
            Err(AppError::Internal(e.to_string()))
        }
    }
}

async fn perform_analysis(config: &AppConfig) -> anyhow::Result<TradingBotResponse> {
    // Get news and filter headlines with symbols
    let input = alpaca_data::get_alpaca_news().await
        .map_err(|e| anyhow::anyhow!("Alpaca API error: {}", e))?;
    
    let news_array = input["news"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected 'news' array from Alpaca API"))?;
    
    // Filter news with symbols and collect headlines
    let mut news_with_symbols = Vec::new();
    let mut headlines = Vec::new();
    
    for item in news_array {
        if let Some(symbols) = item["symbols"].as_array() {
            if !symbols.is_empty() {
                let headline = item["headline"].as_str().unwrap_or("");
                let symbols_vec: Vec<String> = symbols.iter()
                    .filter_map(|s| s.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                news_with_symbols.push((headline.to_string(), symbols_vec));
                headlines.push(headline);
            }
        }
    }
    
    // Run sentiment analysis with better error handling
    let sentiments = {
        let mut model_guard = SENTIMENT_MODEL.lock().await;
        if model_guard.is_none() {
            // Initialize model in a blocking task to avoid runtime conflicts
            let model = tokio::task::spawn_blocking(|| {
                SentimentModel::new(SentimentConfig::default())
            }).await
                .map_err(|e| anyhow::anyhow!("Failed to spawn blocking task: {}", e))?
                .map_err(|e| anyhow::anyhow!("Failed to initialize sentiment model: {}", e))?;
            *model_guard = Some(model);
        }
        
        model_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Failed to get sentiment model reference"))?
            .predict(&headlines)
    };
    
    // Create sentiment analysis results in parallel
    let mut sentiment_results: Vec<_> = news_with_symbols.iter().zip(sentiments.iter()).map(|((headline, symbols), sentiment)| {
        let sentiment_str = match sentiment.polarity {
            SentimentPolarity::Positive => "Positive".to_string(),
            SentimentPolarity::Negative => "Negative".to_string(),
        };
        
        SentimentAnalysis {
            headline: headline.clone(),
            symbols: symbols.clone(),
            sentiment: sentiment_str,
            confidence: sentiment.score,
        }
    }).collect();
    
    // Sort news analysis by confidence (highest to lowest)
    sentiment_results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    // Count sentiments (for potential future use)
    let _positive_count = sentiments.iter().filter(|s| s.polarity == SentimentPolarity::Positive).count();
    let _negative_count = sentiments.iter().filter(|s| s.polarity == SentimentPolarity::Negative).count();
    
    let news_analysis = sentiment_results;
    
    // Deduplicate symbols efficiently and filter out crypto
    let all_symbols: Vec<String> = news_with_symbols.iter()
        .flat_map(|(_, symbols)| symbols.iter())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    
    let crypto_symbols: Vec<String> = all_symbols.iter()
        .filter(|symbol| alpaca_data::is_crypto_symbol(symbol))
        .cloned()
        .collect();
    
    let unique_symbols_vec: Vec<String> = all_symbols.iter()
        .filter(|symbol| !alpaca_data::is_crypto_symbol(symbol))
        .cloned()
        .collect();
    
    println!("Filtered out {} crypto symbols: {:?}", crypto_symbols.len(), crypto_symbols);
    
    // Analyze options for unique symbols in parallel
    let overall_sentiment = if sentiments.iter().any(|s| s.polarity == SentimentPolarity::Positive) {
        "call"
    } else {
        "put"
    };
    
    // Create futures for parallel options analysis
    let options_futures: Vec<_> = unique_symbols_vec.iter().map(|symbol| {
        let symbol = symbol.clone();
        let sentiment = overall_sentiment.to_string();
        async move {
            match alpaca_data::analyze_ticker_options(&symbol, &serde_json::json!({}), Some(&sentiment)).await {
                Ok(analysis) => {
                    let mut top_options = Vec::new();
                    
                    // Convert the analysis to our structured format
                    let options_analysis_vec = if let Some(analysis_array) = analysis["options_analysis"].as_array() {
                        analysis_array.iter().map(|item| {
                            let contract = &item["contract"];
                            
                            // Calculate financial metrics for the contract
                            let financial_metrics = alpaca_data::calculate_option_financial_metrics(contract);
                            
                            // Create enhanced contract with financial metrics
                            let mut enhanced_contract = contract.clone();
                            if let Some(metrics) = financial_metrics {
                                enhanced_contract["financial_metrics"] = serde_json::json!({
                                    "sharpe_ratio": metrics.sharpe,
                                    "sortino_ratio": metrics.sortino,
                                    "calmar_ratio": metrics.calmar,
                                    "max_drawdown": metrics.max_drawdown,
                                    "volatility": metrics.volatility,
                                    "composite_score": metrics.composite_score,
                                    "kelly_fraction": metrics.kelly_fraction,
                                });
                            }
                            
                            OptionAnalysis {
                                contract_type: item["contract_type"].as_str().unwrap_or("").to_string(),
                                contract: enhanced_contract,
                                option_score: item["option_score"].as_f64().unwrap_or(0.0),
                                undervalued_indicators: item["undervalued_indicators"].as_array()
                                    .map(|arr| arr.iter().filter_map(|i| i.as_str()).map(|s| s.to_string()).collect())
                                    .unwrap_or_default(),
                            }
                        }).collect()
                    } else {
                        Vec::new()
                    };
                    
                    let symbol_analysis = SymbolOptionsAnalysis {
                        symbol: symbol.clone(),
                        underlying_metrics: analysis["underlying_metrics"].clone(),
                        options_analysis: options_analysis_vec,
                        error: analysis["error"].as_str().map(|s| s.to_string()),
                    };
                    
                    // Collect top options for summary
                    for option in &symbol_analysis.options_analysis {
                        if option.option_score > 1.0 {
                            top_options.push(TopOption {
                                symbol: symbol.clone(),
                                score: option.option_score,
                                indicators: option.undervalued_indicators.clone(),
                            });
                        }
                    }
                    
                    Ok::<(SymbolOptionsAnalysis, Vec<TopOption>), String>((symbol_analysis, top_options))
                }
                Err(e) => {
                    let symbol_analysis = SymbolOptionsAnalysis {
                        symbol: symbol.clone(),
                        underlying_metrics: serde_json::json!({}),
                        options_analysis: Vec::new(),
                        error: Some(e),
                    };
                    Ok::<(SymbolOptionsAnalysis, Vec<TopOption>), String>((symbol_analysis, Vec::new()))
                }
            }
        }
    }).collect();
    
    // Execute all futures in parallel with concurrency limit
    let mut options_analysis = Vec::new();
    let mut top_options = Vec::new();
    
    // Use futures::stream::iter with buffer_unordered for controlled concurrency
    use futures::stream::{self, StreamExt};
    let mut stream = stream::iter(options_futures).buffer_unordered(config.max_concurrent_requests);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok((symbol_analysis, symbol_top_options)) => {
                options_analysis.push(symbol_analysis);
                top_options.extend(symbol_top_options);
            }
            Err(_) => {
                // Handle any errors that might occur during parallel execution
            }
        }
    }
    
    // Sort top options by score (highest to lowest)
    top_options.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    top_options.truncate(10); // Keep top 10
    
    // Sort options analysis by highest option score for each symbol
    options_analysis.sort_by(|a, b| {
        let a_max_score = a.options_analysis.iter().map(|opt| opt.option_score).fold(0.0, f64::max);
        let b_max_score = b.options_analysis.iter().map(|opt| opt.option_score).fold(0.0, f64::max);
        b_max_score.partial_cmp(&a_max_score).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Convert options analysis to trading signals
    let mut trading_signals = Vec::new();
    for symbol_analysis in &options_analysis {
        for option in &symbol_analysis.options_analysis {
            let sentiment_score = news_analysis.iter()
                .find(|news| news.symbols.contains(&symbol_analysis.symbol))
                .map(|news| news.confidence)
                .unwrap_or(0.5);
            
            let signal = alpaca_data::convert_to_trading_signal(
                &symbol_analysis.symbol,
                option,
                sentiment_score,
                overall_sentiment,
            );
            trading_signals.push(signal);
        }
    }
    
    // Sort trading signals by confidence (highest to lowest)
    trading_signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    // Calculate market summary and risk metrics
    let market_summary = alpaca_data::calculate_market_summary(&trading_signals, &news_analysis);
    let risk_metrics = alpaca_data::calculate_risk_metrics(&trading_signals);
    
    // Create execution metadata
    let execution_metadata = ExecutionMetadata {
        processing_time_ms: 0, // Will be set by the endpoint
        symbols_analyzed: unique_symbols_vec.len(),
        options_analyzed: trading_signals.len(),
        crypto_symbols_filtered: crypto_symbols.len(),
        api_calls_made: unique_symbols_vec.len() + 1, // +1 for news API
        cache_hit_rate: 0.0,
    };

    Ok(TradingBotResponse {
        market_summary,
        trading_signals,
        sentiment_analysis: news_analysis,
        risk_metrics,
        execution_metadata,
    })
}
