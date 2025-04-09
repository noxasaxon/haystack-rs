use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::byte_stream::ByteStream;
use crate::sparse_embedding::SparseEmbedding;

/// Legacy fields that were present in Haystack 1.x Document class
const LEGACY_FIELDS: [&str; 3] = ["content_type", "id_hash_keys", "dataframe"];

/// Base data class containing some data to be queried
///
/// Can contain text snippets and binary data. Documents can be sorted by score and
/// serialized to/from JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for the document
    pub id: String,
    /// Text of the document, if the document contains text
    pub content: Option<String>,
    /// Binary data associated with the document
    pub blob: Option<ByteStream>,
    /// Additional custom metadata for the document
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,
    /// Score of the document. Used for ranking, usually assigned by retrievers
    pub score: Option<f32>,
    /// Dense vector representation of the document
    pub embedding: Option<Vec<f32>>,
    /// Sparse vector representation of the document
    pub sparse_embedding: Option<SparseEmbedding>,
}

impl Document {
    /// Create a new Document with default values
    pub fn new() -> Self {
        let mut doc = Self {
            id: String::new(),
            content: None,
            blob: None,
            meta: HashMap::new(),
            score: None,
            embedding: None,
            sparse_embedding: None,
        };
        
        // Generate ID if not set
        if doc.id.is_empty() {
            doc.id = doc.create_id();
        }
        
        doc
    }
    
    /// Create a new Document with specified values
    pub fn with_params(
        id: Option<String>,
        content: Option<String>,
        blob: Option<ByteStream>,
        meta: Option<HashMap<String, serde_json::Value>>,
        score: Option<f32>,
        embedding: Option<Vec<f32>>,
        sparse_embedding: Option<SparseEmbedding>,
    ) -> Self {
        let mut doc = Self {
            id: id.unwrap_or_default(),
            content,
            blob,
            meta: meta.unwrap_or_default(),
            score,
            embedding,
            sparse_embedding,
        };
        
        // Generate ID if not set
        if doc.id.is_empty() {
            doc.id = doc.create_id();
        }
        
        doc
    }

    /// Creates a hash of the given content that acts as the document's ID
    fn create_id(&self) -> String {
        let text = self.content.as_deref().unwrap_or("None");
        let dataframe = "None"; // this allows the ID creation to remain unchanged even if the dataframe field has been removed
        
        let blob_data = if let Some(blob) = &self.blob {
            format!("{:?}", blob.data)
        } else {
            "None".to_string()
        };
        
        let mime_type = if let Some(blob) = &self.blob {
            blob.mime_type.as_deref().unwrap_or("None")
        } else {
            "None"
        };
        
        let meta = format!("{:?}", self.meta);
        let embedding = format!("{:?}", self.embedding);
        
        let sparse_embedding = if let Some(se) = &self.sparse_embedding {
            format!("{:?}", se.to_dict())
        } else {
            "".to_string()
        };
        
        let data = format!("{}{}{}{}{}{}{}", text, dataframe, blob_data, mime_type, meta, embedding, sparse_embedding);
        
        // Create SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Converts Document into a dictionary
    ///
    /// `blob` field is converted to a JSON-serializable type
    pub fn to_dict(&self, flatten: bool) -> serde_json::Value {
        let mut data = serde_json::to_value(self).unwrap();
        
        // Handle blob serialization
        if let Some(blob_obj) = data.get_mut("blob").and_then(|b| b.as_object_mut()) {
            if let Some(blob_data) = blob_obj.get("data") {
                if let Some(_bytes) = blob_data.as_array() {
                    // Already properly serialized
                } else if let Some(blob) = &self.blob {
                    // Need to convert bytes to array of integers
                    let data_array: Vec<u8> = blob.data.clone();
                    blob_obj.insert("data".to_string(), serde_json::to_value(data_array).unwrap());
                }
            }
        }
        
        if flatten {
            let mut result = serde_json::Map::new();
            
            if let Some(obj) = data.as_object() {
                for (key, value) in obj {
                    if key != "meta" {
                        result.insert(key.clone(), value.clone());
                    }
                }
                
                // Add meta fields at top level
                if let Some(meta_obj) = obj.get("meta").and_then(|m| m.as_object()) {
                    for (key, value) in meta_obj {
                        result.insert(key.clone(), value.clone());
                    }
                }
            }
            
            serde_json::Value::Object(result)
        } else {
            data
        }
    }
    
    /// Creates a new Document object from a dictionary
    pub fn from_dict(data: &serde_json::Value) -> Result<Self> {
        let mut data_map = data.as_object()
            .context("Failed to parse Document data: not an object")?
            .clone();
        
        // Extract document fields
        let document_fields: Vec<&str> = LEGACY_FIELDS.iter()
            .chain(["id", "content", "blob", "meta", "score", "embedding", "sparse_embedding"].iter())
            .copied()
            .collect();
        
        // Unflatten metadata if it was flattened
        let mut meta = if let Some(meta_val) = data_map.remove("meta") {
            if let Some(meta_obj) = meta_val.as_object() {
                meta_obj.clone()
            } else {
                serde_json::Map::new()
            }
        } else {
            serde_json::Map::new()
        };
        
        // Extract flattened metadata
        let mut flatten_meta = serde_json::Map::new();
        let keys: Vec<String> = data_map.keys().cloned().collect();
        for key in keys {
            if !document_fields.contains(&key.as_str()) {
                if let Some(value) = data_map.remove(&key) {
                    flatten_meta.insert(key, value);
                }
            }
        }
        
        // We don't support passing both flatten keys and the `meta` keyword parameter
        if !meta.is_empty() && !flatten_meta.is_empty() {
            return Err(anyhow::anyhow!(
                "You can pass either the 'meta' parameter or flattened metadata keys as keyword arguments, \
                but currently you're passing both. Pass either the 'meta' parameter or flattened metadata keys."
            ));
        }
        
        // Merge metadata
        for (key, value) in flatten_meta {
            meta.insert(key, value);
        }
        
        // Put metadata back
        data_map.insert("meta".to_string(), serde_json::Value::Object(meta));
        
        // Handle blob conversion
        if let Some(blob_value) = data_map.get("blob").cloned() {
            if let Some(blob_obj) = blob_value.as_object() {
                let data_bytes = if let Some(data_array) = blob_obj.get("data").and_then(|d| d.as_array()) {
                    let mut bytes = Vec::new();
                    for val in data_array {
                        if let Some(num) = val.as_u64() {
                            bytes.push(num as u8);
                        }
                    }
                    bytes
                } else {
                    Vec::new()
                };
                
                let mime_type = blob_obj.get("mime_type")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());
                
                let meta = blob_obj.get("meta")
                    .and_then(|m| m.as_object())
                    .map(|o| {
                        let mut map = HashMap::new();
                        for (k, v) in o {
                            map.insert(k.clone(), v.clone());
                        }
                        map
                    })
                    .unwrap_or_default();
                
                data_map.insert("blob".to_string(), serde_json::to_value(ByteStream {
                    data: data_bytes,
                    mime_type,
                    meta,
                }).unwrap());
            }
        }
        
        // Handle sparse_embedding conversion
        if let Some(se_value) = data_map.get("sparse_embedding").cloned() {
            if let Some(se_obj) = se_value.as_object() {
                let mut se_map = HashMap::new();
                for (k, v) in se_obj {
                    se_map.insert(k.clone(), v.clone());
                }
                
                if let Ok(sparse_embedding) = SparseEmbedding::from_dict(&se_map) {
                    data_map.insert("sparse_embedding".to_string(), serde_json::to_value(sparse_embedding).unwrap());
                }
            }
        }
        
        // Deserialize the document
        let document: Document = serde_json::from_value(serde_json::Value::Object(data_map))
            .context("Failed to deserialize Document")?;
        
        Ok(document)
    }
    
    /// Returns the type of the content for the document (for backward compatibility with 1.x)
    pub fn content_type(&self) -> Result<&str> {
        if self.content.is_some() {
            Ok("text")
        } else {
            Err(anyhow::anyhow!("Content is not set."))
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fields = Vec::new();
        
        if let Some(content) = &self.content {
            if content.len() < 100 {
                fields.push(format!("content: '{}'", content));
            } else {
                fields.push(format!("content: '{}...'", &content[..100]));
            }
        }
        
        if let Some(blob) = &self.blob {
            fields.push(format!("blob: {} bytes", blob.data.len()));
        }
        
        if !self.meta.is_empty() {
            fields.push(format!("meta: {:?}", self.meta));
        }
        
        if let Some(score) = self.score {
            fields.push(format!("score: {}", score));
        }
        
        if let Some(embedding) = &self.embedding {
            fields.push(format!("embedding: vector of size {}", embedding.len()));
        }
        
        if let Some(sparse_embedding) = &self.sparse_embedding {
            fields.push(format!("sparse_embedding: vector with {} non-zero elements", sparse_embedding.indices.len()));
        }
        
        let fields_str = fields.join(", ");
        write!(f, "Document(id={}, {})", self.id, fields_str)
    }
}