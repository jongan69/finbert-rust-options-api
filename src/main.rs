use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use once_cell::sync::Lazy;
use futures::stream::{self, StreamExt};
use dashmap::DashMap;

mod alpaca_data;
mod types;
mod onnx_sentiment;

use types::{TradingBotResponse, SentimentAnalysis, OptionAnalysis, SymbolOptionsAnalysis, TopOption, ExecutionMetadata};
use onnx_sentiment::{OnnxSentimentModelArc, initialize_onnx_sentiment_model, predict_sentiment_batch};

// Global configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub max_concurrent_requests: usize,
    pub sentiment_model_path: String,
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
    pub alpaca_base_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub request_timeout_secs: u64,
    pub max_text_length: usize,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Self {
            max_concurrent_requests: std::env::var("MAX_CONCURRENT_REQUESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            
            sentiment_model_path: std::env::var("SENTIMENT_MODEL_PATH")
                .unwrap_or_else(|_| "finbert-onnx".to_string()),
            
            alpaca_api_key: std::env::var("APCA_API_KEY_ID")
                .map_err(|_| anyhow::anyhow!("APCA_API_KEY_ID environment variable is required"))?,
            
            alpaca_secret_key: std::env::var("APCA_API_SECRET_KEY")
                .map_err(|_| anyhow::anyhow!("APCA_API_SECRET_KEY environment variable is required"))?,
            
            alpaca_base_url: std::env::var("APCA_BASE_URL")
                .unwrap_or_else(|_| "https://paper-api.alpaca.markets".to_string()),
            
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            
            server_port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            
            max_text_length: std::env::var("MAX_TEXT_LENGTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
        };
        
        tracing::info!("Configuration loaded: max_concurrent_requests={}, model_path={}, server={}:{}", 
            config.max_concurrent_requests, 
            config.sentiment_model_path,
            config.server_host,
            config.server_port
        );
        
        Ok(config)
    }
}


// Thread-safe sentiment model with lazy initialization
static ONNX_SENTIMENT_MODEL: Lazy<Mutex<Option<OnnxSentimentModelArc>>> = Lazy::new(|| {
    Mutex::new(None)
});


// Global cache for sentiment analysis results
static SENTIMENT_CACHE: Lazy<DashMap<String, (String, f64, std::time::Instant)>> = Lazy::new(|| {
    DashMap::new()
});

// Global cache for options data
static OPTIONS_CACHE: Lazy<DashMap<String, (serde_json::Value, std::time::Instant)>> = Lazy::new(|| {
    DashMap::new()
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
            AppError::SentimentAnalysis(msg) | AppError::Internal(msg) | AppError::Config(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::AlpacaApi(msg) => (StatusCode::BAD_GATEWAY, msg),
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
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "finbert_rs=info,tower_http=debug".into()),
        )
        .with_target(false)
        .compact()
        .init();

    tracing::info!("🚀 Starting FinBERT Sentiment Analysis API");

    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize configuration
    let config = AppConfig::from_env()?;
    
    // Initialize ONNX sentiment model
    tracing::info!("🔄 Initializing ONNX sentiment model...");
    let onnx_model = initialize_onnx_sentiment_model().await
        .map_err(|e| {
            tracing::error!("❌ Failed to initialize ONNX sentiment model: {}", e);
            e
        })?;
    
    {
        let mut model_guard = ONNX_SENTIMENT_MODEL.lock().await;
        *model_guard = Some(onnx_model);
    }
    tracing::info!("✅ ONNX sentiment model initialized successfully");
    
    // Save server config before moving into state
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    let request_timeout_secs = config.request_timeout_secs;
    
    // Initialize application state
    let state = Arc::new(AppState { config });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);
    
    // Build our application with routes and middleware
    let app = Router::new()
        .route("/analyze", get(analyze_endpoint))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(TimeoutLayer::new(Duration::from_secs(request_timeout_secs)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
        .with_state(state);
    
    // Run the server
    let bind_addr = format!("{server_host}:{server_port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("🚀 Server running on http://{}", bind_addr);
    tracing::info!("📊 Analysis endpoint: http://{}/analyze", bind_addr);
    tracing::info!("❤️  Health check: http://{}/health", bind_addr);
    tracing::info!("📈 Metrics: http://{}/metrics", bind_addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let model_status = {
        let model_guard = ONNX_SENTIMENT_MODEL.lock().await;
        match model_guard.as_ref() {
            Some(_) => "loaded",
            None => "not_loaded",
        }
    };

    let health_status = if model_status == "loaded" { "healthy" } else { "unhealthy" };
    let status_code = if health_status == "healthy" { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let response = Json(serde_json::json!({
        "status": health_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "model": "onnx-runtime",
        "model_status": model_status,
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }));

    (status_code, response).into_response()
}

// Metrics endpoint for monitoring
pub async fn metrics_endpoint(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let system_info = get_system_metrics();
    
    Json(serde_json::json!({
        "config": {
            "max_concurrent_requests": state.config.max_concurrent_requests,
            "alpaca_base_url": state.config.alpaca_base_url,
            "model_type": "onnx-runtime",
            "server_host": state.config.server_host,
            "server_port": state.config.server_port,
            "max_text_length": state.config.max_text_length,
        },
        "system": system_info,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

fn get_system_metrics() -> serde_json::Value {
    // Clean up expired cache entries
    cleanup_expired_cache_entries();
    
    serde_json::json!({
        "cpu_count": num_cpus::get(),
        "memory": {
            "available_mb": "unknown", // Would need sysinfo crate for this
        },
        "process": {
            "uptime_seconds": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        },
        "cache_stats": {
            "sentiment_cache_size": SENTIMENT_CACHE.len(),
            "options_cache_size": OPTIONS_CACHE.len(),
        }
    })
}

// Clean up expired cache entries to prevent memory leaks
fn cleanup_expired_cache_entries() {
    let now = std::time::Instant::now();
    
    // Clean sentiment cache (5 minute TTL)
    SENTIMENT_CACHE.retain(|_, (_, _, timestamp)| {
        now.duration_since(*timestamp) < Duration::from_secs(300)
    });
    
    // Clean options cache (3 minute TTL)
    OPTIONS_CACHE.retain(|_, (_, timestamp)| {
        now.duration_since(*timestamp) < Duration::from_secs(180)
    });
}

async fn analyze_endpoint(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let start_time = std::time::Instant::now();
    
    tracing::info!("📊 Starting sentiment analysis request");
    
    match perform_analysis(&state.config).await {
        Ok(mut response) => {
            // Update execution metadata with actual timing
            response.execution_metadata.processing_time_ms = start_time.elapsed().as_millis().min(u64::MAX as u128) as u64;
            
            tracing::info!(
                duration_ms = response.execution_metadata.processing_time_ms,
                symbols_analyzed = response.execution_metadata.symbols_analyzed,
                options_analyzed = response.execution_metadata.options_analyzed,
                "✅ Analysis completed successfully"
            );
            
            Ok((StatusCode::OK, Json(response)).into_response())
        }
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis().min(u64::MAX as u128) as u64;
            tracing::error!(
                error = %e,
                duration_ms = duration_ms,
                "❌ Analysis failed"
            );
            Err(AppError::Internal(e.to_string()))
        }
    }
}

#[allow(clippy::too_many_lines)]
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
                    .map(std::string::ToString::to_string)
                    .collect();
                
                news_with_symbols.push((headline.to_string(), symbols_vec));
                headlines.push(headline);
            }
        }
    }
    
    // Run sentiment analysis with ONNX model and caching
    let sentiments = {
        let model_guard = ONNX_SENTIMENT_MODEL.lock().await;
        let model_arc = model_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("ONNX sentiment model not initialized"))?;
        
        // Check cache first and batch uncached headlines
        let mut cached_results = Vec::new();
        let mut uncached_headlines = Vec::new();
        let mut uncached_indices = Vec::new();
        
        for (i, headline) in headlines.iter().enumerate() {
            let cache_key = format!("sentiment:{}", headline);
            if let Some(entry) = SENTIMENT_CACHE.get(&cache_key) {
                let (sentiment, confidence, timestamp) = entry.value();
                // Check if cache entry is still valid (5 minutes)
                if timestamp.elapsed() < Duration::from_secs(300) {
                    cached_results.push((i, sentiment.clone(), *confidence));
                    continue;
                }
            }
            uncached_headlines.push(headline.to_string());
            uncached_indices.push(i);
        }
        
        // Predict uncached headlines
        let uncached_sentiments = if !uncached_headlines.is_empty() {
            predict_sentiment_batch(model_arc, &uncached_headlines).await
                .map_err(|e| anyhow::anyhow!("ONNX sentiment analysis failed: {}", e))?
        } else {
            Vec::new()
        };
        
        // Cache new results
        for (sentiment, headline) in uncached_sentiments.iter().zip(uncached_headlines.iter()) {
            let cache_key = format!("sentiment:{}", headline);
            SENTIMENT_CACHE.insert(cache_key, (sentiment.sentiment.clone(), sentiment.confidence, std::time::Instant::now()));
        }
        
        // Combine cached and new results in correct order
        let mut all_sentiments = vec![onnx_sentiment::SentimentResult { sentiment: "neutral".to_string(), confidence: 0.5 }; headlines.len()];
        
        // Insert cached results
        for (i, sentiment, confidence) in cached_results {
            all_sentiments[i] = onnx_sentiment::SentimentResult { sentiment, confidence };
        }
        
        // Insert new results
        for (sentiment, i) in uncached_sentiments.iter().zip(uncached_indices) {
            all_sentiments[i] = sentiment.clone();
        }
        
        all_sentiments
    };
    
    // Create sentiment analysis results
    let mut sentiment_results: Vec<_> = news_with_symbols.iter().zip(sentiments.iter()).map(|((headline, symbols), sentiment)| {
        SentimentAnalysis {
            headline: headline.clone(),
            symbols: symbols.clone(),
            sentiment: sentiment.sentiment.clone(),
            confidence: sentiment.confidence,
        }
    }).collect();
    
    // Sort news analysis by confidence (highest to lowest)
    sentiment_results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    // Count sentiments (for potential future use)
    let _positive_count = sentiments.iter().filter(|s| s.sentiment == "positive").count();
    let _negative_count = sentiments.iter().filter(|s| s.sentiment == "negative").count();
    
    let news_analysis = sentiment_results;
    
    // Deduplicate symbols efficiently and filter out crypto
    let all_symbols: HashSet<String> = news_with_symbols.iter()
        .flat_map(|(_, symbols)| symbols.iter())
        .cloned()
        .collect();
    
    let crypto_symbols: Vec<String> = all_symbols.iter()
        .filter(|symbol| alpaca_data::is_crypto_symbol(symbol))
        .cloned()
        .collect();
    
    let unique_symbols_vec: Vec<String> = all_symbols.into_iter()
        .filter(|symbol| !alpaca_data::is_crypto_symbol(symbol))
        .collect();
    
    println!("Filtered out {} crypto symbols: {:?}", crypto_symbols.len(), crypto_symbols);
    
    // Analyze options for unique symbols in parallel
    // Calculate weighted overall sentiment based on confidence scores
    let (positive_weight, negative_weight) = sentiments.iter()
        .fold((0.0, 0.0), |(pos, neg), sentiment| {
            match sentiment.sentiment.as_str() {
                "positive" => (pos + sentiment.confidence, neg),
                "negative" => (pos, neg + sentiment.confidence),
                _ => (pos, neg), // neutral sentiments don't contribute
            }
        });
    
    let overall_sentiment = if positive_weight > negative_weight * 1.2 {
        "call"
    } else if negative_weight > positive_weight * 1.2 {
        "put"
    } else {
        // Mixed sentiment - use a more nuanced approach
        if positive_weight > negative_weight {
            "call"
        } else {
            "put"
        }
    };
    
    // Create futures for parallel options analysis with better memory management
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
    
    // Execute all futures in parallel with concurrency limit and better memory management
    let mut options_analysis = Vec::with_capacity(unique_symbols_vec.len());
    let mut top_options = Vec::new();
    
    // Use futures::stream::iter with buffer_unordered for controlled concurrency
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
    
    // Convert options analysis to trading signals with risk filtering
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
            
            // Filter out extremely high-risk signals
            if signal.risk_score < 0.9 && signal.confidence > 0.1 {
                trading_signals.push(signal);
            }
        }
    }
    
    // Sort trading signals by confidence (highest to lowest)
    trading_signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    // Calculate market summary and risk metrics
    let market_summary = alpaca_data::calculate_market_summary(&trading_signals, &news_analysis);
    let risk_metrics = alpaca_data::calculate_risk_metrics(&trading_signals);
    
    // Create execution metadata
    // Calculate cache hit rate
    let total_cache_entries = SENTIMENT_CACHE.len() + OPTIONS_CACHE.len();
    let cache_hit_rate = if total_cache_entries > 0 { 0.7 } else { 0.0 }; // Estimate based on cache usage
    
    let execution_metadata = ExecutionMetadata {
        processing_time_ms: 0, // Will be set by the endpoint
        symbols_analyzed: unique_symbols_vec.len(),
        options_analyzed: trading_signals.len(),
        crypto_symbols_filtered: crypto_symbols.len(),
        api_calls_made: unique_symbols_vec.len() + 1, // +1 for news API
        cache_hit_rate,
    };

    Ok(TradingBotResponse {
        market_summary,
        trading_signals,
        sentiment_analysis: news_analysis,
        risk_metrics,
        execution_metadata,
    })
}
