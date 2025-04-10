use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Class representing a sparse embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseEmbedding {
    /// List of indices of non-zero elements in the embedding
    pub indices: Vec<i32>,
    /// List of values of non-zero elements in the embedding
    pub values: Vec<f32>,
}

impl SparseEmbedding {
    /// Create a new SparseEmbedding
    ///
    /// # Arguments
    /// * `indices` - List of indices of non-zero elements in the embedding
    /// * `values` - List of values of non-zero elements in the embedding
    ///
    /// # Returns
    /// * `Result<SparseEmbedding>` - A new SparseEmbedding if the indices and values lists are the same length
    pub fn new(indices: Vec<i32>, values: Vec<f32>) -> Result<Self> {
        if indices.len() != values.len() {
            return Err(anyhow!("Length of indices and values must be the same."));
        }
        
        Ok(Self {
            indices,
            values,
        })
    }

    /// Convert the SparseEmbedding to a dictionary-like structure
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut dict = HashMap::new();
        dict.insert("indices".to_string(), serde_json::to_value(&self.indices).unwrap());
        dict.insert("values".to_string(), serde_json::to_value(&self.values).unwrap());
        dict
    }

    /// Create a SparseEmbedding from a dictionary-like structure
    pub fn from_dict(dict: &HashMap<String, serde_json::Value>) -> Result<Self> {
        let indices = match dict.get("indices") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| anyhow!("Failed to parse indices: {}", e))?,
            None => return Err(anyhow!("Missing 'indices' field")),
        };
        
        let values = match dict.get("values") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| anyhow!("Failed to parse values: {}", e))?,
            None => return Err(anyhow!("Missing 'values' field")),
        };
        
        Self::new(indices, values)
    }
}

impl std::fmt::Display for SparseEmbedding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SparseEmbedding with {} non-zero elements", self.indices.len())
    }
}