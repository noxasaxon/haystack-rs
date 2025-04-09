use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use bm25;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;

use crate::document_stores::{DocumentStore, WriteMode};
use haystack_dataclasses::document::Document;

/// Wrapper for the BM25 model from the bm25 crate
#[derive(Clone, Debug)]
struct BM25Model {
    model: bm25::BM25,
}

// BM25Model is Sized by default because all its fields are Sized
// This is to explicitly address the compiler error about Option<BM25Model> and as_ref()

impl BM25Model {
    /// Create a new BM25 model
    fn new(docs: &[(&str, &str)]) -> Result<Self> {
        let model = bm25::BM25::new(docs, Default::default())
            .map_err(|e| anyhow!("Failed to create BM25 model: {}", e))?;
        
        Ok(Self { model })
    }
    
    /// Search for documents matching the query
    fn search(&self, query: &str, filter_ids: Option<Vec<&str>>) -> Result<Vec<(String, f64)>> {
        let results = self.model.search(query, filter_ids)
            .map_err(|e| anyhow!("BM25 search failed: {}", e))?;
        
        // Convert results to (String, f64) pairs
        let results = results.into_iter()
            .map(|(id, score)| (id.to_string(), score))
            .collect();
        
        Ok(results)
    }
}

/// BM25 algorithm variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BM25Algorithm {
    /// Original BM25 algorithm (Okapi)
    BM25Okapi,
    /// BM25 with document length normalization
    BM25L,
    /// BM25 with additive smoothing
    BM25Plus,
}

impl Default for BM25Algorithm {
    fn default() -> Self {
        BM25Algorithm::BM25Okapi
    }
}

/// In-memory document store that stores documents in memory
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct InMemoryDocumentStore {
    /// Map of document ID to document
    #[serde(skip)]
    documents: HashMap<String, Document>,
    
    /// BM25 search model
    #[serde(skip)]
    bm25_model: Option<BM25Model>,
    
    /// Map of document IDs to their content strings (for BM25 indexing)
    #[serde(skip)]
    document_content_map: HashMap<String, String>,
    
    /// Algorithm to use for BM25 retrieval
    bm25_algorithm: BM25Algorithm,
    
    /// Whether to scale BM25 scores to 0-1 range
    scale_score: bool,
    
    /// Term saturation parameter for BM25
    k1: f64,
    
    /// Length normalization parameter for BM25
    b: f64,
    
    /// Smoothing parameter for BM25+
    delta: f64,
    
    /// Document name for serialization/deserialization
    name: String,
    
    /// Default write mode
    write_mode: WriteMode,
}

impl InMemoryDocumentStore {
    /// Create a new in-memory document store
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            bm25_model: None,
            document_content_map: HashMap::new(),
            bm25_algorithm: BM25Algorithm::BM25Okapi,
            scale_score: false,
            k1: 1.5,
            b: 0.75,
            delta: 1.0,
            name: format!("InMemoryDocumentStore-{}", Uuid::new_v4()),
            write_mode: WriteMode::Overwrite,
        }
    }
    
    /// Create a new in-memory document store with custom parameters
    pub fn with_params(
        bm25_algorithm: BM25Algorithm,
        scale_score: bool,
        k1: f64,
        b: f64,
        delta: f64,
        name: Option<String>,
        write_mode: WriteMode,
    ) -> Self {
        Self {
            documents: HashMap::new(),
            bm25_model: None,
            document_content_map: HashMap::new(),
            bm25_algorithm,
            scale_score,
            k1,
            b,
            delta,
            name: name.unwrap_or_else(|| format!("InMemoryDocumentStore-{}", Uuid::new_v4())),
            write_mode,
        }
    }
    
    /// Update the BM25 model with the current documents
    fn update_bm25_model(&mut self) -> Result<()> {
        // Extract content strings from all documents
        let mut docs: Vec<(&str, &str)> = Vec::new();
        
        for (id, doc) in &self.documents {
            if let Some(content) = &doc.content {
                docs.push((id, content));
                self.document_content_map.insert(id.clone(), content.clone());
            }
        }
        
        // Create a new BM25 model
        // Note: The Rust bm25 crate doesn't expose parameters like k1 and b directly
        // We keep them in our struct for API compatibility with Python, but they're not used
        let model = BM25Model::new(&docs)?;
        
        self.bm25_model = Some(model);
        
        Ok(())
    }
    
    /// Perform BM25 retrieval with the given query
    pub fn bm25_retrieval(
        &self,
        query: &str,
        top_k: usize,
        filters: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<Document>> {
        let model = self.bm25_model.as_ref().ok_or_else(|| {
            anyhow!("BM25 model has not been initialized. No documents have been added to the store.")
        })?;
        
        // Get the document IDs that match the filters (if any)
        let filtered_ids = if let Some(filters) = filters {
            let filtered_docs = self.filter_documents(filters)?;
            filtered_docs.into_iter().map(|doc| doc.id).collect::<HashSet<_>>()
        } else {
            // If no filters, consider all documents
            self.documents.keys().cloned().collect::<HashSet<_>>()
        };
        
        // Perform BM25 search
        let mut results = model.search(query, Some(filtered_ids.iter().map(|id| id.as_str()).collect::<Vec<_>>()))?;
        
        // Scale scores if requested
        if self.scale_score {
            // Use a simple min-max scaling for now
            if !results.is_empty() {
                let max_score = results.iter().map(|(_, score)| *score).fold(f64::NEG_INFINITY, f64::max);
                let min_score = results.iter().map(|(_, score)| *score).fold(f64::INFINITY, f64::min);
                
                if max_score > min_score {
                    for (_, score) in &mut results {
                        *score = (*score - min_score) / (max_score - min_score);
                    }
                }
            }
        }
        
        // Limit to top_k results
        results.truncate(top_k);
        
        // Convert results to Documents
        let mut documents = Vec::new();
        for (id, score) in results {
            if let Some(mut doc) = self.documents.get(&id).cloned() {
                doc.score = Some(score as f32);
                documents.push(doc);
            }
        }
        
        Ok(documents)
    }
}

#[async_trait::async_trait]
impl DocumentStore for InMemoryDocumentStore {
    fn write_documents(&mut self, documents: &[Document]) -> Result<Vec<Document>> {
        let mut written_docs = Vec::new();
        
        for doc in documents {
            // Clone the document
            let document = doc.clone();
            
            // Check if document already exists
            if self.documents.contains_key(&document.id) {
                match self.write_mode {
                    WriteMode::Fail => {
                        return Err(anyhow!("Document with ID {} already exists", document.id));
                    }
                    WriteMode::Skip => {
                        continue;
                    }
                    WriteMode::Overwrite => {
                        // Continue with overwrite
                    }
                }
            }
            
            // Store the document
            self.documents.insert(document.id.clone(), document.clone());
            written_docs.push(document);
        }
        
        // Update the BM25 model
        self.update_bm25_model()?;
        
        Ok(written_docs)
    }
    
    fn get_documents(&self, ids: &[String]) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        
        for id in ids {
            if let Some(doc) = self.documents.get(id) {
                documents.push(doc.clone());
            }
        }
        
        Ok(documents)
    }
    
    fn delete_documents(&mut self, ids: &[String]) -> Result<()> {
        for id in ids {
            self.documents.remove(id);
            self.document_content_map.remove(id);
        }
        
        // Update the BM25 model
        self.update_bm25_model()?;
        
        Ok(())
    }
    
    fn get_all_documents(&self) -> Result<Vec<Document>> {
        Ok(self.documents.values().cloned().collect())
    }
    
    fn count_documents(&self) -> Result<usize> {
        Ok(self.documents.len())
    }
    
    fn filter_documents(&self, filters: &HashMap<String, Value>) -> Result<Vec<Document>> {
        if filters.is_empty() {
            return Ok(self.documents.values().cloned().collect());
        }
        
        let filtered_docs = self.documents.values().filter(|doc| {
            // Check if the document matches all filters
            for (key, value) in filters {
                // Handle special metadata fields
                if key == "id" {
                    if let Some(filter_id) = value.as_str() {
                        if doc.id != filter_id {
                            return false;
                        }
                    }
                    continue;
                }
                
                // Check metadata fields
                if !doc.meta.contains_key(key) {
                    return false;
                }
                
                let doc_value = &doc.meta[key];
                
                // Simple equality check for now
                if doc_value != value {
                    return false;
                }
            }
            
            true
        }).cloned().collect();
        
        Ok(filtered_docs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_documents() -> Vec<Document> {
        vec![
            Document::with_params(
                Some("1".to_string()),
                Some("The quick brown fox jumps over the lazy dog".to_string()),
                None,
                Some(HashMap::from([
                    ("type".to_string(), Value::String("article".to_string())),
                    ("category".to_string(), Value::String("animals".to_string())),
                ])),
                None,
                None,
                None,
            ),
            Document::with_params(
                Some("2".to_string()),
                Some("A fast yellow fox leaps across a sleeping hound".to_string()),
                None,
                Some(HashMap::from([
                    ("type".to_string(), Value::String("article".to_string())),
                    ("category".to_string(), Value::String("animals".to_string())),
                ])),
                None,
                None,
                None,
            ),
            Document::with_params(
                Some("3".to_string()),
                Some("The five boxing wizards jump quickly".to_string()),
                None,
                Some(HashMap::from([
                    ("type".to_string(), Value::String("article".to_string())),
                    ("category".to_string(), Value::String("sports".to_string())),
                ])),
                None,
                None,
                None,
            ),
        ]
    }
    
    #[test]
    fn test_write_documents() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        
        let written = doc_store.write_documents(&documents).unwrap();
        assert_eq!(written.len(), 3);
        assert_eq!(doc_store.count_documents().unwrap(), 3);
    }
    
    #[test]
    fn test_get_documents() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        doc_store.write_documents(&documents).unwrap();
        
        let retrieved = doc_store.get_documents(&["1".to_string(), "3".to_string()]).unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].id, "1");
        assert_eq!(retrieved[1].id, "3");
    }
    
    #[test]
    fn test_delete_documents() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        doc_store.write_documents(&documents).unwrap();
        
        doc_store.delete_documents(&["1".to_string()]).unwrap();
        assert_eq!(doc_store.count_documents().unwrap(), 2);
        assert!(doc_store.get_documents(&["1".to_string()]).unwrap().is_empty());
    }
    
    #[test]
    fn test_filter_documents() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        doc_store.write_documents(&documents).unwrap();
        
        let filters = HashMap::from([
            ("category".to_string(), Value::String("sports".to_string())),
        ]);
        
        let filtered = doc_store.filter_documents(&filters).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "3");
    }
    
    #[test]
    fn test_bm25_retrieval() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        doc_store.write_documents(&documents).unwrap();
        
        let results = doc_store.bm25_retrieval("fox jumps", 10, None).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|doc| doc.id == "1"));
        
        // Document 1 should be more relevant than Document 2 for this query
        let doc1_pos = results.iter().position(|doc| doc.id == "1").unwrap();
        let doc2_pos = results.iter().position(|doc| doc.id == "2").unwrap();
        assert!(doc1_pos < doc2_pos);
    }
    
    #[test]
    fn test_bm25_retrieval_with_filters() {
        let mut doc_store = InMemoryDocumentStore::new();
        let documents = create_test_documents();
        doc_store.write_documents(&documents).unwrap();
        
        let filters = HashMap::from([
            ("category".to_string(), Value::String("sports".to_string())),
        ]);
        
        let results = doc_store.bm25_retrieval("jump", 10, Some(&filters)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "3");
    }
}