mod alpaca_data;
mod types;

use rust_bert::pipelines::sentiment::SentimentModel;
use rust_bert::pipelines::sentiment::SentimentConfig;
use rust_bert::pipelines::sentiment::SentimentPolarity;
use crate::alpaca_data::get_alpaca_news;

fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Load FinBERT model
    let sentiment_model = SentimentModel::new(SentimentConfig::default())?;
    
    // Run the async part
    tokio::runtime::Runtime::new()?.block_on(async {
        // Predict sentiment
        let input = get_alpaca_news().await.map_err(|e| anyhow::anyhow!(e))?;
        
        let news_array = input["news"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected 'news' array from Alpaca API"))?;
        
        let headlines: Vec<&str> = news_array
            .iter()
            .filter_map(|item| item["headline"].as_str())
            .collect();
        let output = sentiment_model.predict(&headlines);

        for sentiment in output {
            match sentiment.polarity {
                SentimentPolarity::Positive => println!("Positive ({:.2}%)", sentiment.score * 100.0),
                SentimentPolarity::Negative => println!("Negative ({:.2}%)", sentiment.score * 100.0),
            }
        }

        Ok::<(), anyhow::Error>(())
    })
}
