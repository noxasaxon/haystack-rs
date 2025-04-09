use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::component::{Component, ComponentBase};
use crate::document_stores::{FilterPolicy, DocumentStore};
use crate::document_stores::in_memory::document_store::InMemoryDocumentStore;
use haystack_dataclasses::document::Document;

/// In-memory BM25 retriever component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryBM25Retriever {
    /// Base component implementation
    #[serde(skip)]
    base: ComponentBase,
    
    /// The document store to retrieve from
    #[serde(skip)]
    document_store: InMemoryDocumentStore,
    
    /// Number of documents to retrieve
    top_k: usize,
    
    /// Filter policy for retrieval
    filter_policy: FilterPolicy,
    
    /// Document filters to use during retrieval
    #[serde(default)]
    filters: Option<HashMap<String, Value>>,
}

impl InMemoryBM25Retriever {
    /// Create a new BM25 retriever
    pub fn new(
        document_store: InMemoryDocumentStore,
        top_k: usize,
        filter_policy: FilterPolicy,
        filters: Option<HashMap<String, Value>>,
    ) -> Result<Self> {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        let mut output_sockets = HashMap::new();
        
        // Add input socket for query
        input_sockets.insert(
            "query".to_string(),
            crate::component::InputSocket::new(
                "query".to_string(),
                std::any::TypeId::of::<String>(),
                "String".to_string(),
                None,
                false,
                false,
            ),
        );
        
        // Add optional input socket for filters
        input_sockets.insert(
            "filters".to_string(),
            crate::component::InputSocket::new(
                "filters".to_string(),
                std::any::TypeId::of::<HashMap<String, Value>>(),
                "HashMap<String, Value>".to_string(),
                Some(Value::Null),
                false,
                false,
            ),
        );
        
        // Add output socket for documents
        output_sockets.insert(
            "documents".to_string(),
            crate::component::OutputSocket::new(
                "documents".to_string(),
                std::any::TypeId::of::<Vec<Document>>(),
                "Vec<Document>".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("top_k".to_string(), Value::from(top_k));
        init_parameters.insert("filter_policy".to_string(), Value::String(match filter_policy {
            FilterPolicy::Replace => "replace",
            FilterPolicy::Merge => "merge",
        }.to_string()));
        
        if let Some(filters) = &filters {
            init_parameters.insert("filters".to_string(), serde_json::to_value(filters)?);
        }
        
        Ok(Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            document_store,
            top_k,
            filter_policy,
            filters,
        })
    }
    
    /// Get effective filters based on the filter policy
    fn get_effective_filters(
        &self,
        runtime_filters: Option<&HashMap<String, Value>>,
    ) -> Option<HashMap<String, Value>> {
        match (runtime_filters, &self.filters) {
            // No filters
            (None, None) => None,
            
            // Only runtime filters
            (Some(runtime), None) => Some(runtime.clone()),
            
            // Only init filters
            (None, Some(init)) => Some(init.clone()),
            
            // Both runtime and init filters
            (Some(runtime), Some(init)) => {
                match self.filter_policy {
                    FilterPolicy::Replace => Some(runtime.clone()),
                    FilterPolicy::Merge => {
                        let mut merged = init.clone();
                        merged.extend(runtime.clone());
                        Some(merged)
                    }
                }
            }
        }
    }
}

impl Component for InMemoryBM25Retriever {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Get query
        let query = inputs.get("query")
            .ok_or_else(|| anyhow!("Missing required input 'query'"))?;
        
        let query = query.as_str()
            .ok_or_else(|| anyhow!("Query must be a string"))?;
        
        // Get runtime filters (if any)
        let runtime_filters = if let Some(filters) = inputs.get("filters") {
            if filters.is_null() {
                None
            } else {
                Some(serde_json::from_value::<HashMap<String, Value>>(filters.clone())
                    .map_err(|e| anyhow!("Failed to parse filters: {}", e))?)
            }
        } else {
            None
        };
        
        // Get effective filters
        let effective_filters = self.get_effective_filters(runtime_filters.as_ref());
        
        // Perform retrieval
        let documents = self.document_store.bm25_retrieval(query, self.top_k, effective_filters.as_ref())?;
        
        // Return results
        let mut outputs = HashMap::new();
        outputs.insert("documents".to_string(), serde_json::to_value(documents)?);
        
        Ok(outputs)
    }
    
    fn warm_up(&self) -> Result<()> {
        // Nothing to do
        Ok(())
    }
    
    fn input_sockets(&self) -> &HashMap<String, crate::component::InputSocket> {
        self.base.input_sockets()
    }
    
    fn output_sockets(&self) -> &HashMap<String, crate::component::OutputSocket> {
        self.base.output_sockets()
    }
    
    fn init_parameters(&self) -> &HashMap<String, Value> {
        self.base.init_parameters()
    }
}

#[cfg(test)]
mod tests {
    use crate::document_stores::DocumentStore;

    use super::*;
    
    fn create_test_document_store() -> InMemoryDocumentStore {
        let mut doc_store = InMemoryDocumentStore::new();
        
        let documents = vec![
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
        ];
        
        doc_store.write_documents(&documents).unwrap();
        doc_store
    }
    
    #[test]
    fn test_retriever_basic() {
        let doc_store = create_test_document_store();
        let retriever = InMemoryBM25Retriever::new(
            doc_store,
            10,
            FilterPolicy::Replace,
            None,
        ).unwrap();
        
        let inputs = HashMap::from([
            ("query".to_string(), Value::String("fox jumps".to_string())),
        ]);
        
        let outputs = retriever.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert!(!documents.is_empty());
        assert!(documents.iter().any(|doc| doc.id == "1"));
        
        // Document 1 should be more relevant than Document 2 for this query
        let doc1_pos = documents.iter().position(|doc| doc.id == "1").unwrap();
        let doc2_pos = documents.iter().position(|doc| doc.id == "2").unwrap();
        assert!(doc1_pos < doc2_pos);
    }
    
    #[test]
    fn test_retriever_with_filters() {
        let doc_store = create_test_document_store();
        let retriever = InMemoryBM25Retriever::new(
            doc_store,
            10,
            FilterPolicy::Replace,
            None,
        ).unwrap();
        
        let inputs = HashMap::from([
            ("query".to_string(), Value::String("jump".to_string())),
            ("filters".to_string(), serde_json::to_value(HashMap::from([
                ("category".to_string(), Value::String("sports".to_string())),
            ])).unwrap()),
        ]);
        
        let outputs = retriever.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "3");
    }
    
    #[test]
    fn test_retriever_with_init_filters() {
        let doc_store = create_test_document_store();
        let retriever = InMemoryBM25Retriever::new(
            doc_store,
            10,
            FilterPolicy::Replace,
            Some(HashMap::from([
                ("category".to_string(), Value::String("animals".to_string())),
            ])),
        ).unwrap();
        
        let inputs = HashMap::from([
            ("query".to_string(), Value::String("fox".to_string())),
        ]);
        
        let outputs = retriever.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents.len(), 2);
        assert!(documents.iter().any(|doc| doc.id == "1"));
        assert!(documents.iter().any(|doc| doc.id == "2"));
    }
    
    #[test]
    fn test_retriever_filter_policy_replace() {
        let doc_store = create_test_document_store();
        let retriever = InMemoryBM25Retriever::new(
            doc_store,
            10,
            FilterPolicy::Replace,
            Some(HashMap::from([
                ("category".to_string(), Value::String("animals".to_string())),
            ])),
        ).unwrap();
        
        let inputs = HashMap::from([
            ("query".to_string(), Value::String("jump".to_string())),
            ("filters".to_string(), serde_json::to_value(HashMap::from([
                ("category".to_string(), Value::String("sports".to_string())),
            ])).unwrap()),
        ]);
        
        let outputs = retriever.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        // Runtime filters should replace init filters
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "3");
    }
    
    #[test]
    fn test_retriever_filter_policy_merge() {
        let doc_store = create_test_document_store();
        let retriever = InMemoryBM25Retriever::new(
            doc_store,
            10,
            FilterPolicy::Merge,
            Some(HashMap::from([
                ("type".to_string(), Value::String("article".to_string())),
            ])),
        ).unwrap();
        
        let inputs = HashMap::from([
            ("query".to_string(), Value::String("jump".to_string())),
            ("filters".to_string(), serde_json::to_value(HashMap::from([
                ("category".to_string(), Value::String("sports".to_string())),
            ])).unwrap()),
        ]);
        
        let outputs = retriever.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        // Runtime filters should be merged with init filters
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "3");
    }
}