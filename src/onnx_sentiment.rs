use anyhow::Result;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value,
};
use tokenizers::Tokenizer;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::env;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SentimentResult {
    pub sentiment: String,
    pub confidence: f64,
    #[allow(dead_code)]
    pub scores: Vec<f64>,
}

pub struct OnnxSentimentModel {
    session: Session,
    tokenizer: Tokenizer,
}

impl OnnxSentimentModel {
    pub fn new(model_path: &str) -> Result<Self> {
        let model_dir = Self::resolve_model_path(model_path)?;
        
        let model_file = model_dir.join("model.onnx");
        let tokenizer_file = model_dir.join("tokenizer.json");
        
        // Verify files exist
        if !model_file.exists() {
            return Err(anyhow::anyhow!(
                "ONNX model file not found: {}. Current working directory: {}",
                model_file.display(),
                env::current_dir().unwrap_or_default().display()
            ));
        }
        
        if !tokenizer_file.exists() {
            return Err(anyhow::anyhow!(
                "Tokenizer file not found: {}. Current working directory: {}",
                tokenizer_file.display(),
                env::current_dir().unwrap_or_default().display()
            ));
        }

        // Validate model file integrity
        Self::validate_model_file(&model_file)?;

        // Create optimized ONNX Runtime session with error handling
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)? // Reduce optimization for compatibility
            .with_intra_threads(num_cpus::get().min(4))? // Reduce threads for Pi
            .commit_from_file(&model_file)
            .map_err(|e| anyhow::anyhow!("Failed to load ONNX model: {}. The model may be corrupted or incompatible with this ONNX Runtime version. Try re-downloading the model.", e))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(OnnxSentimentModel {
            session,
            tokenizer,
        })
    }
    
    fn validate_model_file(model_file: &Path) -> Result<()> {
        use std::fs::File;
        use std::io::Read;
        
        let mut file = File::open(model_file)
            .map_err(|e| anyhow::anyhow!("Cannot open model file: {}", e))?;
        
        let file_size = file.metadata()
            .map_err(|e| anyhow::anyhow!("Cannot get model file metadata: {}", e))?
            .len();
        
        if file_size < 1000 {
            return Err(anyhow::anyhow!("Model file is too small ({} bytes), likely corrupted", file_size));
        }
        
        // Check ONNX magic header
        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .map_err(|e| anyhow::anyhow!("Cannot read model file header: {}", e))?;
        
        // ONNX files should start with protobuf encoding
        if header[0] != 0x08 {
            return Err(anyhow::anyhow!(
                "Invalid ONNX model file format. Expected protobuf header, got: {:02x?}", 
                &header[..4]
            ));
        }
        
        tracing::info!("Model file validation passed: {} bytes", file_size);
        Ok(())
    }
    
    fn resolve_model_path(model_path: &str) -> Result<PathBuf> {
        let path = Path::new(model_path);
        
        // If it's already absolute and exists, use it
        if path.is_absolute() && path.exists() {
            return Ok(path.to_path_buf());
        }
        
        // Try relative to current directory
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        
        // Try relative to the project root (where Cargo.toml is)
        let current_exe = env::current_exe()
            .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;
        
        let mut search_path = current_exe.parent().unwrap().to_path_buf();
        
        // Go up directories looking for Cargo.toml
        for _ in 0..5 {
            let cargo_toml = search_path.join("Cargo.toml");
            if cargo_toml.exists() {
                let model_dir = search_path.join(model_path);
                if model_dir.exists() {
                    return Ok(model_dir);
                }
            }
            
            if let Some(parent) = search_path.parent() {
                search_path = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        // Finally, try relative to where we think the project is
        let cwd = env::current_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
        
        let model_dir = cwd.join(model_path);
        if model_dir.exists() {
            return Ok(model_dir);
        }
        
        // Try going up from current directory
        let mut parent_search = cwd.clone();
        for _ in 0..3 {
            if let Some(parent) = parent_search.parent() {
                parent_search = parent.to_path_buf();
                let candidate = parent_search.join(model_path);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        
        Err(anyhow::anyhow!(
            "Could not find model directory '{}' in any of the searched locations. Current directory: {}",
            model_path,
            cwd.display()
        ))
    }

    pub fn predict(&mut self, text: &str) -> Result<SentimentResult> {
        // Input validation
        if text.trim().is_empty() {
            return Err(anyhow::anyhow!("Input text cannot be empty"));
        }
        
        let max_length = std::env::var("MAX_TEXT_LENGTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);
            
        if text.len() > max_length {
            return Err(anyhow::anyhow!("Input text too long (max {} characters)", max_length));
        }
        
        // Tokenize the input text
        let encoding = self.tokenizer.encode(text.trim(), true)
            .map_err(|e| anyhow::anyhow!("Failed to encode text: {}", e))?;
        
        // Prepare input tensors
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Convert to ONNX tensors
        let input_ids_tensor = Value::from_array(
            ndarray::Array2::from_shape_vec(
                (1, input_ids.len()),
                input_ids.iter().map(|&x| i64::from(x)).collect(),
            )?,
        )?;

        let attention_mask_tensor = Value::from_array(
            ndarray::Array2::from_shape_vec(
                (1, attention_mask.len()),
                attention_mask.iter().map(|&x| i64::from(x)).collect(),
            )?,
        )?;

        // Run inference
        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor
        ])?;

        // Extract logits from output
        let logits_tensor = &outputs["logits"];
        let logits = logits_tensor.try_extract_tensor::<f32>()?;
        let (_, logits_data) = logits;

        // Apply softmax to get probabilities 
        let num_classes = 3; // positive, negative, neutral
        let mut max_val = f32::NEG_INFINITY;
        
        // Find max for numerical stability
        for &logit in logits_data.iter().take(num_classes) {
            max_val = max_val.max(logit);
        }
        
        // Compute softmax
        let mut sum = 0.0f32;
        let mut scores = Vec::with_capacity(num_classes);
        
        for &logit in logits_data.iter().take(num_classes) {
            let exp_val = (logit - max_val).exp();
            scores.push(exp_val);
            sum += exp_val;
        }
        
        // Normalize probabilities
        for score in &mut scores {
            *score /= sum;
        }

        // Get the predicted class and confidence
        let predicted_class = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map_or(0, |(i, _)| i);

        let confidence = f64::from(scores[predicted_class]);

        // Map class indices to sentiment labels (based on config.json id2label)
        let sentiment_labels = ["positive", "negative", "neutral"];
        let sentiment = sentiment_labels[predicted_class].to_string();

        Ok(SentimentResult {
            sentiment,
            confidence,
            scores: scores.iter().map(|&x| f64::from(x)).collect(),
        })
    }

    pub fn predict_batch(&mut self, texts: &[String]) -> Result<Vec<SentimentResult>> {
        let mut results = Vec::new();

        for text in texts {
            let result = self.predict(text)?;
            results.push(result);
        }

        Ok(results)
    }
}

// Thread-safe wrapper for the sentiment model
pub type OnnxSentimentModelArc = Arc<Mutex<Option<OnnxSentimentModel>>>;

pub async fn initialize_onnx_sentiment_model() -> Result<OnnxSentimentModelArc> {
    let model_path = std::env::var("SENTIMENT_MODEL_PATH").unwrap_or_else(|_| "finbert-onnx".to_string());
    let model = OnnxSentimentModel::new(&model_path)?;
    Ok(Arc::new(Mutex::new(Some(model))))
}

#[allow(dead_code)]
pub async fn predict_sentiment(
    model_arc: &OnnxSentimentModelArc,
    text: &str,
) -> Result<SentimentResult> {
    let mut model_guard = model_arc.lock().await;
    let model = model_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Sentiment model not initialized"))?;

    model.predict(text)
}

pub async fn predict_sentiment_batch(
    model_arc: &OnnxSentimentModelArc,
    texts: &[String],
) -> Result<Vec<SentimentResult>> {
    let mut model_guard = model_arc.lock().await;
    let model = model_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Sentiment model not initialized"))?;

    model.predict_batch(texts)
}