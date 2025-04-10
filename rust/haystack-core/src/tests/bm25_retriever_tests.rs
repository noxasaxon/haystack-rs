use std::collections::HashMap;

use serde_json::Value;

use crate::component_system::Component;
use crate::component_systems::retrievers::in_memory::bm25_retriever::InMemoryBM25Retriever;
use crate::document_stores::{DocumentStore, FilterPolicy};
use crate::document_stores::in_memory::document_store::InMemoryDocumentStore;
use crate::pipeline::Pipeline;
use haystack_dataclasses::Document;

#[test]
fn test_bm25_retriever_in_pipeline() {
    // Create a document store
    let mut doc_store = InMemoryDocumentStore::new();
    
    // Create some test documents
    let documents = vec![
        Document::with_params(
            Some("1".to_string()),
            Some("The quick brown fox jumps over the lazy dog".to_string()),
            None,
            Some(HashMap::from([
                ("type".to_string(), Value::String("animal".to_string())),
            ])),
            None,
            None,
            None,
        ),
        Document::with_params(
            Some("2".to_string()),
            Some("The five boxing wizards jump quickly".to_string()),
            None,
            Some(HashMap::from([
                ("type".to_string(), Value::String("sports".to_string())),
            ])),
            None,
            None,
            None,
        ),
    ];
    
    // Write documents to the store
    doc_store.write_documents(&documents).unwrap();
    
    // Create a BM25 retriever
    let retriever = InMemoryBM25Retriever::new(
        doc_store,
        10,
        FilterPolicy::Replace,
        None,
    ).unwrap();
    
    // Create a pipeline with the retriever
    let mut pipeline = Pipeline::new();
    pipeline.add_component("retriever", retriever).unwrap();
    
    // Connect the components
    pipeline.connect("retriever", "query", "output", "documents").unwrap();
    
    // Run the pipeline
    let result = pipeline.run(HashMap::from([
        ("query".to_string(), Value::String("fox jumps".to_string())),
    ])).unwrap();
    
    // Check the result
    let retrieved_docs = result.get("output").unwrap();
    let documents = serde_json::from_value::<Vec<Document>>(retrieved_docs.clone()).unwrap();
    
    // Make sure we got the right documents
    assert_eq!(documents.len(), 2);
    
    // The first document should be more relevant (contain "fox jumps")
    assert_eq!(documents[0].id, "1");
    assert!(documents[0].score.unwrap() > documents[1].score.unwrap());
}

#[test]
fn test_bm25_retriever_with_filters() {
    // Create a document store
    let mut doc_store = InMemoryDocumentStore::new();
    
    // Create some test documents
    let documents = vec![
        Document::with_params(
            Some("1".to_string()),
            Some("The quick brown fox jumps over the lazy dog".to_string()),
            None,
            Some(HashMap::from([
                ("type".to_string(), Value::String("animal".to_string())),
            ])),
            None,
            None,
            None,
        ),
        Document::with_params(
            Some("2".to_string()),
            Some("The five boxing wizards jump quickly".to_string()),
            None,
            Some(HashMap::from([
                ("type".to_string(), Value::String("sports".to_string())),
            ])),
            None,
            None,
            None,
        ),
    ];
    
    // Write documents to the store
    doc_store.write_documents(&documents).unwrap();
    
    // Create a BM25 retriever with filters
    let retriever = InMemoryBM25Retriever::new(
        doc_store,
        10,
        FilterPolicy::Replace,
        None,
    ).unwrap();
    
    // Run the retriever directly with filters
    let result = retriever.run(HashMap::from([
        ("query".to_string(), Value::String("jump".to_string())),
        ("filters".to_string(), serde_json::to_value(HashMap::from([
            ("type".to_string(), Value::String("sports".to_string())),
        ])).unwrap()),
    ])).unwrap();
    
    // Check the result
    let retrieved_docs = result.get("documents").unwrap();
    let documents = serde_json::from_value::<Vec<Document>>(retrieved_docs.clone()).unwrap();
    
    // We should only get the sports document
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].id, "2");
}