/*!
 * Document Stores
 * 
 * This module contains document store implementations for storing and retrieving documents.
 */

pub mod in_memory;

use std::collections::HashMap;
use anyhow::Result;
use haystack_dataclasses::document::Document;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Filter policy for document retrieval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterPolicy {
    /// Replace any existing filters with the new filters
    Replace,
    
    /// Merge the new filters with any existing filters
    Merge,
}

/// Default write mode for document stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteMode {
    /// Raise an error if a document with the same ID already exists
    Fail,
    
    /// Skip writing documents that have the same ID as existing ones
    Skip, 
    
    /// Overwrite existing documents with new ones that have the same ID
    Overwrite,
}

impl Default for WriteMode {
    fn default() -> Self {
        WriteMode::Overwrite
    }
}

/// Base trait for all document stores
#[async_trait::async_trait]
pub trait DocumentStore: Send + Sync {
    /// Write documents to the store
    fn write_documents(&mut self, documents: &[Document]) -> Result<Vec<Document>>;
    
    /// Asynchronously write documents to the store
    async fn write_documents_async(&mut self, documents: &[Document]) -> Result<Vec<Document>> {
        // Default implementation calls the synchronous version
        self.write_documents(documents)
    }
    
    /// Get documents by ID
    fn get_documents(&self, ids: &[String]) -> Result<Vec<Document>>;
    
    /// Asynchronously get documents by ID
    async fn get_documents_async(&self, ids: &[String]) -> Result<Vec<Document>> {
        // Default implementation calls the synchronous version
        self.get_documents(ids)
    }
    
    /// Delete documents by ID
    fn delete_documents(&mut self, ids: &[String]) -> Result<()>;
    
    /// Asynchronously delete documents by ID
    async fn delete_documents_async(&mut self, ids: &[String]) -> Result<()> {
        // Default implementation calls the synchronous version
        self.delete_documents(ids)
    }
    
    /// Get all documents in the store
    fn get_all_documents(&self) -> Result<Vec<Document>>;
    
    /// Asynchronously get all documents in the store
    async fn get_all_documents_async(&self) -> Result<Vec<Document>> {
        // Default implementation calls the synchronous version
        self.get_all_documents()
    }
    
    /// Count documents in the store
    fn count_documents(&self) -> Result<usize>;
    
    /// Asynchronously count documents in the store
    async fn count_documents_async(&self) -> Result<usize> {
        // Default implementation calls the synchronous version
        self.count_documents()
    }
    
    /// Search for documents using filters
    fn filter_documents(&self, filters: &HashMap<String, Value>) -> Result<Vec<Document>>;
    
    /// Asynchronously search for documents using filters
    async fn filter_documents_async(&self, filters: &HashMap<String, Value>) -> Result<Vec<Document>> {
        // Default implementation calls the synchronous version
        self.filter_documents(filters)
    }
}

// Re-export document store implementations
pub use in_memory::document_store::InMemoryDocumentStore;