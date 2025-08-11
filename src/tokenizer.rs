use anyhow::Result;
use burn::tensor::{Tensor, backend::Backend};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Tokenizer {
    vocab: HashMap<String, u32>,
    special_tokens: HashMap<String, u32>,
    max_length: usize,
}

impl Tokenizer {
    pub fn new(model_path: &str) -> Result<Self> {
        let model_path = Path::new(model_path);
        
        // Load vocabulary
        let vocab_path = model_path.join("vocab.txt");
        let vocab_content = fs::read_to_string(vocab_path)?;
        let mut vocab = HashMap::new();
        
        for (index, line) in vocab_content.lines().enumerate() {
            vocab.insert(line.to_string(), index as u32);
        }
        
        // Load special tokens
        let special_tokens_path = model_path.join("special_tokens_map.json");
        let special_tokens_content = fs::read_to_string(special_tokens_path)?;
        let special_tokens_value: Value = serde_json::from_str(&special_tokens_content)?;
        
        let mut special_tokens = HashMap::new();
        if let Some(special_tokens_obj) = special_tokens_value.as_object() {
            for (key, value) in special_tokens_obj {
                if let Some(token) = value.as_str() {
                    if let Some(&token_id) = vocab.get(token) {
                        special_tokens.insert(key.clone(), token_id);
                    }
                }
            }
        }
        
        Ok(Tokenizer {
            vocab,
            special_tokens,
            max_length: 512,
        })
    }
    
    pub fn tokenize<B: Backend>(&self, text: &str) -> Result<Tensor<B, 2>> {
        // Simple tokenization - split on whitespace and punctuation
        let words: Vec<&str> = text
            .split_whitespace()
            .flat_map(|word| {
                // Simple word splitting on punctuation
                word.split_inclusive(|c: char| c.is_ascii_punctuation())
            })
            .collect();
        
        let mut token_ids = Vec::new();
        
        // Add [CLS] token
        if let Some(&cls_id) = self.special_tokens.get("cls_token") {
            token_ids.push(cls_id);
        }
        
        // Tokenize words
        for word in words.iter().take(self.max_length - 2) {
            if let Some(&token_id) = self.vocab.get(word) {
                token_ids.push(token_id);
            } else {
                // Use UNK token for unknown words
                if let Some(&unk_id) = self.special_tokens.get("unk_token") {
                    token_ids.push(unk_id);
                }
            }
        }
        
        // Add [SEP] token
        if let Some(&sep_id) = self.special_tokens.get("sep_token") {
            token_ids.push(sep_id);
        }
        
        // Pad to max_length
        while token_ids.len() < self.max_length {
            if let Some(&pad_id) = self.special_tokens.get("pad_token") {
                token_ids.push(pad_id);
            } else {
                token_ids.push(0); // Default padding
            }
        }
        
        // Create attention mask (1 for real tokens, 0 for padding)
        let mut attention_mask = vec![1; token_ids.len()];
        for i in token_ids.len()..self.max_length {
            attention_mask[i] = 0;
        }
        
        // Convert to tensors
        let input_ids = Tensor::from_vec(token_ids, (1, self.max_length));
        let attention_mask = Tensor::from_vec(attention_mask, (1, self.max_length));
        
        // For now, return input_ids as the main tensor
        // In a full implementation, you'd return both input_ids and attention_mask
        Ok(input_ids)
    }
    
    pub fn get_special_token(&self, token_type: &str) -> Option<u32> {
        self.special_tokens.get(token_type).copied()
    }
}
