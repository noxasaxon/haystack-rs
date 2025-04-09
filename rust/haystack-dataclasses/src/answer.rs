use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::document::Document;

/// Trait defining the Answer protocol
pub trait Answer {
    /// The data content of the answer
    fn data(&self) -> &dyn std::any::Any;
    
    /// The query that produced this answer
    fn query(&self) -> &str;
    
    /// Metadata associated with the answer
    fn meta(&self) -> &HashMap<String, Value>;
    
    /// Convert the answer to a dictionary representation
    fn to_dict(&self) -> Result<Value>;
    
    /// Create an answer from a dictionary representation
    fn from_dict(data: &Value) -> Result<Self> where Self: Sized;
}

/// A span of text representing the position of an answer in a document or context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Start index of the span
    pub start: usize,
    
    /// End index of the span
    pub end: usize,
}

/// An answer extracted from a document with its context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedAnswer {
    /// The query that produced this answer
    pub query: String,
    
    /// Confidence score for the answer
    pub score: f32,
    
    /// The actual answer text
    pub data: Option<String>,
    
    /// The document from which the answer was extracted
    pub document: Option<Document>,
    
    /// Context surrounding the answer
    pub context: Option<String>,
    
    /// Position of the answer in the document
    pub document_offset: Option<Span>,
    
    /// Position of the answer in the context
    pub context_offset: Option<Span>,
    
    /// Additional metadata
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}

impl ExtractedAnswer {
    /// Create a new ExtractedAnswer
    pub fn new(
        query: String,
        score: f32,
        data: Option<String>,
        document: Option<Document>,
        context: Option<String>,
        document_offset: Option<Span>,
        context_offset: Option<Span>,
        meta: Option<HashMap<String, Value>>,
    ) -> Self {
        Self {
            query,
            score,
            data,
            document,
            context,
            document_offset,
            context_offset,
            meta: meta.unwrap_or_default(),
        }
    }
    
    /// Serialize the object to a dictionary
    pub fn to_dict(&self) -> Result<Value> {
        let mut dict = serde_json::Map::new();
        
        dict.insert("data".to_string(), serde_json::to_value(&self.data)?);
        dict.insert("query".to_string(), serde_json::to_value(&self.query)?);
        
        let document = if let Some(doc) = &self.document {
            let doc_dict = doc.to_dict(false);
            serde_json::to_value(doc_dict)?
        } else {
            Value::Null
        };
        dict.insert("document".to_string(), document);
        
        dict.insert("context".to_string(), serde_json::to_value(&self.context)?);
        dict.insert("score".to_string(), serde_json::to_value(&self.score)?);
        
        let document_offset = if let Some(offset) = &self.document_offset {
            serde_json::to_value(offset)?
        } else {
            Value::Null
        };
        dict.insert("document_offset".to_string(), document_offset);
        
        let context_offset = if let Some(offset) = &self.context_offset {
            serde_json::to_value(offset)?
        } else {
            Value::Null
        };
        dict.insert("context_offset".to_string(), context_offset);
        
        dict.insert("meta".to_string(), serde_json::to_value(&self.meta)?);
        
        // Create init_parameters structure
        let mut init_parameters = serde_json::Map::new();
        for (key, value) in &dict {
            init_parameters.insert(key.clone(), value.clone());
        }
        
        // Create the final dictionary with type and init_parameters
        let mut result = serde_json::Map::new();
        result.insert("type".to_string(), Value::String("ExtractedAnswer".to_string()));
        result.insert("init_parameters".to_string(), Value::Object(init_parameters));
        
        Ok(Value::Object(result))
    }
    
    /// Deserialize the object from a dictionary
    pub fn from_dict(data: &Value) -> Result<Self> {
        let data_obj = data.as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected an object for ExtractedAnswer deserialization"))?;
        
        let init_params = data_obj.get("init_parameters")
            .ok_or_else(|| anyhow::anyhow!("Missing 'init_parameters' in serialized ExtractedAnswer"))?
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected 'init_parameters' to be an object"))?;
        
        let query = init_params.get("query")
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' in serialized ExtractedAnswer"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected 'query' to be a string"))?
            .to_string();
        
        let score = init_params.get("score")
            .ok_or_else(|| anyhow::anyhow!("Missing 'score' in serialized ExtractedAnswer"))?
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Expected 'score' to be a number"))? as f32;
        
        let data = init_params.get("data")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let document = if let Some(doc_value) = init_params.get("document") {
            if doc_value.is_null() {
                None
            } else {
                Some(Document::from_dict(doc_value)
                    .context("Failed to deserialize document in ExtractedAnswer")?)
            }
        } else {
            None
        };
        
        let context = init_params.get("context")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let document_offset = if let Some(offset_value) = init_params.get("document_offset") {
            if offset_value.is_null() {
                None
            } else {
                Some(serde_json::from_value::<Span>(offset_value.clone())
                    .context("Failed to deserialize document_offset in ExtractedAnswer")?)
            }
        } else {
            None
        };
        
        let context_offset = if let Some(offset_value) = init_params.get("context_offset") {
            if offset_value.is_null() {
                None
            } else {
                Some(serde_json::from_value::<Span>(offset_value.clone())
                    .context("Failed to deserialize context_offset in ExtractedAnswer")?)
            }
        } else {
            None
        };
        
        let meta = init_params.get("meta")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), v.clone());
                }
                map
            })
            .unwrap_or_default();
        
        Ok(Self {
            query,
            score,
            data,
            document,
            context,
            document_offset,
            context_offset,
            meta,
        })
    }
}

impl Answer for ExtractedAnswer {
    fn data(&self) -> &dyn std::any::Any {
        &self.data
    }
    
    fn query(&self) -> &str {
        &self.query
    }
    
    fn meta(&self) -> &HashMap<String, Value> {
        &self.meta
    }
    
    fn to_dict(&self) -> Result<Value> {
        self.to_dict()
    }
    
    fn from_dict(data: &Value) -> Result<Self> {
        Self::from_dict(data)
    }
}

/// An answer generated from a list of documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAnswer {
    /// The answer text
    pub data: String,
    
    /// The query that produced this answer
    pub query: String,
    
    /// Documents used to generate the answer
    pub documents: Vec<Document>,
    
    /// Additional metadata
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}

impl GeneratedAnswer {
    /// Create a new GeneratedAnswer
    pub fn new(
        data: String,
        query: String,
        documents: Vec<Document>,
        meta: Option<HashMap<String, Value>>,
    ) -> Self {
        Self {
            data,
            query,
            documents,
            meta: meta.unwrap_or_default(),
        }
    }
    
    /// Serialize the object to a dictionary
    pub fn to_dict(&self) -> Result<Value> {
        let mut dict = serde_json::Map::new();
        
        dict.insert("data".to_string(), Value::String(self.data.clone()));
        dict.insert("query".to_string(), Value::String(self.query.clone()));
        
        let documents: Vec<Value> = self.documents.iter()
            .map(|doc| doc.to_dict(false))
            .collect();
        dict.insert("documents".to_string(), serde_json::to_value(documents)?);
        
        dict.insert("meta".to_string(), serde_json::to_value(&self.meta)?);
        
        // Create init_parameters structure
        let mut init_parameters = serde_json::Map::new();
        for (key, value) in &dict {
            init_parameters.insert(key.clone(), value.clone());
        }
        
        // Create the final dictionary with type and init_parameters
        let mut result = serde_json::Map::new();
        result.insert("type".to_string(), Value::String("GeneratedAnswer".to_string()));
        result.insert("init_parameters".to_string(), Value::Object(init_parameters));
        
        Ok(Value::Object(result))
    }
    
    /// Deserialize the object from a dictionary
    pub fn from_dict(data: &Value) -> Result<Self> {
        let data_obj = data.as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected an object for GeneratedAnswer deserialization"))?;
        
        let init_params = data_obj.get("init_parameters")
            .ok_or_else(|| anyhow::anyhow!("Missing 'init_parameters' in serialized GeneratedAnswer"))?
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected 'init_parameters' to be an object"))?;
        
        let query = init_params.get("query")
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' in serialized GeneratedAnswer"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected 'query' to be a string"))?
            .to_string();
        
        let data = init_params.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' in serialized GeneratedAnswer"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected 'data' to be a string"))?
            .to_string();
        
        let documents_value = init_params.get("documents")
            .ok_or_else(|| anyhow::anyhow!("Missing 'documents' in serialized GeneratedAnswer"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected 'documents' to be an array"))?;
        
        let mut documents = Vec::new();
        for doc_value in documents_value {
            let doc = Document::from_dict(doc_value)
                .context("Failed to deserialize document in GeneratedAnswer")?;
            documents.push(doc);
        }
        
        let meta = init_params.get("meta")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), v.clone());
                }
                map
            })
            .unwrap_or_default();
        
        Ok(Self {
            data,
            query,
            documents,
            meta,
        })
    }
}

impl Answer for GeneratedAnswer {
    fn data(&self) -> &dyn std::any::Any {
        &self.data
    }
    
    fn query(&self) -> &str {
        &self.query
    }
    
    fn meta(&self) -> &HashMap<String, Value> {
        &self.meta
    }
    
    fn to_dict(&self) -> Result<Value> {
        self.to_dict()
    }
    
    fn from_dict(data: &Value) -> Result<Self> {
        Self::from_dict(data)
    }
}