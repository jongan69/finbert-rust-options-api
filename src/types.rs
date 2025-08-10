use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
