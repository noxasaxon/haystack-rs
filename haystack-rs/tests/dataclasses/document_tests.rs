use std::collections::HashMap;
use serde_json::Value;
use haystack_dataclasses::{Document, ByteStream, SparseEmbedding};

#[test]
fn test_document_creation() {
    // Test default constructor
    let doc = Document::new();
    assert!(!doc.id.is_empty());
    assert!(doc.content.is_none());
    assert!(doc.blob.is_none());
    assert!(doc.meta.is_empty());
    assert!(doc.score.is_none());
    assert!(doc.embedding.is_none());
    assert!(doc.sparse_embedding.is_none());
    
    // Test with_params constructor
    let meta = HashMap::from([
        ("key1".to_string(), Value::String("value1".to_string())),
        ("key2".to_string(), Value::Number(42.into())),
    ]);
    
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("test content".to_string()),
        None,
        Some(meta.clone()),
        Some(0.9),
        Some(vec![0.1, 0.2, 0.3]),
        None,
    );
    
    assert_eq!(doc.id, "test-id");
    assert_eq!(doc.content, Some("test content".to_string()));
    assert!(doc.blob.is_none());
    assert_eq!(doc.meta.len(), 2);
    assert_eq!(doc.meta["key1"], Value::String("value1".to_string()));
    assert_eq!(doc.score, Some(0.9));
    assert_eq!(doc.embedding, Some(vec![0.1, 0.2, 0.3]));
    assert!(doc.sparse_embedding.is_none());
}

#[test]
fn test_document_auto_id_generation() {
    // Test ID generation when none provided
    let doc = Document::with_params(
        None,
        Some("test content".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    // ID should be generated automatically
    assert!(!doc.id.is_empty());
}

#[test]
fn test_document_to_dict() {
    let meta = HashMap::from([
        ("key1".to_string(), Value::String("value1".to_string())),
        ("key2".to_string(), Value::Number(42.into())),
    ]);
    
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("test content".to_string()),
        None,
        Some(meta.clone()),
        Some(0.9),
        Some(vec![0.1, 0.2, 0.3]),
        None,
    );
    
    // Test normal serialization
    let dict = doc.to_dict(false);
    assert!(dict.is_object());
    let obj = dict.as_object().unwrap();
    
    assert_eq!(obj["id"].as_str().unwrap(), "test-id");
    assert_eq!(obj["content"].as_str().unwrap(), "test content");
    // Due to floating point representation, compare approximately
    let score = obj["score"].as_f64().unwrap();
    assert!((score - 0.9).abs() < 1e-6);
    
    // Check meta field exists and is not flattened
    assert!(obj.contains_key("meta"));
    let meta_obj = obj["meta"].as_object().unwrap();
    assert_eq!(meta_obj["key1"].as_str().unwrap(), "value1");
    assert_eq!(meta_obj["key2"].as_i64().unwrap(), 42);
    
    // Test flattened serialization
    let flat_dict = doc.to_dict(true);
    assert!(flat_dict.is_object());
    let flat_obj = flat_dict.as_object().unwrap();
    
    assert_eq!(flat_obj["id"].as_str().unwrap(), "test-id");
    assert_eq!(flat_obj["content"].as_str().unwrap(), "test content");
    // Due to floating point representation, compare approximately
    let flat_score = flat_obj["score"].as_f64().unwrap();
    assert!((flat_score - 0.9).abs() < 1e-6);
    
    // Check that meta fields are flattened (directly in the main object)
    assert!(!flat_obj.contains_key("meta"));
    assert_eq!(flat_obj["key1"].as_str().unwrap(), "value1");
    assert_eq!(flat_obj["key2"].as_i64().unwrap(), 42);
}

#[test]
fn test_document_from_dict() {
    // Create a document dictionary
    let mut doc_obj = serde_json::Map::new();
    doc_obj.insert("id".to_string(), Value::String("test-id".to_string()));
    doc_obj.insert("content".to_string(), Value::String("test content".to_string()));
    doc_obj.insert("score".to_string(), Value::Number(serde_json::Number::from_f64(0.9).unwrap()));
    
    let mut meta_obj = serde_json::Map::new();
    meta_obj.insert("key1".to_string(), Value::String("value1".to_string()));
    meta_obj.insert("key2".to_string(), Value::Number(42.into()));
    doc_obj.insert("meta".to_string(), Value::Object(meta_obj));
    
    let doc_dict = Value::Object(doc_obj);
    
    // Deserialize
    let doc = Document::from_dict(&doc_dict).unwrap();
    
    assert_eq!(doc.id, "test-id");
    assert_eq!(doc.content, Some("test content".to_string()));
    assert_eq!(doc.score, Some(0.9));
    assert_eq!(doc.meta.len(), 2);
    assert_eq!(doc.meta["key1"], Value::String("value1".to_string()));
    assert_eq!(doc.meta["key2"], Value::Number(42.into()));
}

#[test]
fn test_document_from_dict_flattened() {
    // Create a document dictionary with flattened meta
    let mut doc_obj = serde_json::Map::new();
    doc_obj.insert("id".to_string(), Value::String("test-id".to_string()));
    doc_obj.insert("content".to_string(), Value::String("test content".to_string()));
    doc_obj.insert("score".to_string(), Value::Number(serde_json::Number::from_f64(0.9).unwrap()));
    // Add flattened metadata
    doc_obj.insert("key1".to_string(), Value::String("value1".to_string()));
    doc_obj.insert("key2".to_string(), Value::Number(42.into()));
    
    let doc_dict = Value::Object(doc_obj);
    
    // Deserialize
    let doc = Document::from_dict(&doc_dict).unwrap();
    
    assert_eq!(doc.id, "test-id");
    assert_eq!(doc.content, Some("test content".to_string()));
    assert_eq!(doc.score, Some(0.9));
    assert_eq!(doc.meta.len(), 2);
    assert_eq!(doc.meta["key1"], Value::String("value1".to_string()));
    assert_eq!(doc.meta["key2"], Value::Number(42.into()));
}

#[test]
fn test_document_with_blob() {
    // Create a document with blob data
    let data = vec![1, 2, 3, 4, 5];
    let blob = ByteStream::new(
        data.clone(),
        Some("application/octet-stream".to_string()),
        None,
    );
    
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("test content".to_string()),
        Some(blob),
        None,
        None,
        None,
        None,
    );
    
    assert!(doc.blob.is_some());
    let blob = doc.blob.as_ref().unwrap();
    assert_eq!(blob.data, data);
    assert_eq!(blob.mime_type, Some("application/octet-stream".to_string()));
    
    // Test serialization and deserialization with blob
    let dict = doc.to_dict(false);
    let restored_doc = Document::from_dict(&dict).unwrap();
    
    assert!(restored_doc.blob.is_some());
    let restored_blob = restored_doc.blob.as_ref().unwrap();
    assert_eq!(restored_blob.data, data);
    assert_eq!(restored_blob.mime_type, Some("application/octet-stream".to_string()));
}

#[test]
fn test_document_with_sparse_embedding() {
    // Create sparse embedding
    let sparse_embedding = SparseEmbedding::new(
        vec![1, 4, 10],
        vec![0.5, 0.8, 0.2],
    ).unwrap();
    
    // Create document with sparse embedding
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("test content".to_string()),
        None,
        None,
        None,
        None,
        Some(sparse_embedding),
    );
    
    assert!(doc.sparse_embedding.is_some());
    let se = doc.sparse_embedding.as_ref().unwrap();
    assert_eq!(se.indices, vec![1, 4, 10]);
    assert_eq!(se.values, vec![0.5, 0.8, 0.2]);
    
    // Test serialization and deserialization with sparse embedding
    let dict = doc.to_dict(false);
    let restored_doc = Document::from_dict(&dict).unwrap();
    
    assert!(restored_doc.sparse_embedding.is_some());
    let restored_se = restored_doc.sparse_embedding.as_ref().unwrap();
    assert_eq!(restored_se.indices, vec![1, 4, 10]);
    assert_eq!(restored_se.values, vec![0.5, 0.8, 0.2]);
}

#[test]
fn test_document_content_type() {
    // Document with content
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("test content".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    assert_eq!(doc.content_type().unwrap(), "text");
    
    // Document without content
    let doc = Document::with_params(
        Some("test-id".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    
    assert!(doc.content_type().is_err());
}

#[test]
fn test_document_display() {
    // Test display formatting with short content
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some("short content".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    let display = format!("{}", doc);
    assert!(display.contains("test-id"));
    assert!(display.contains("short content"));
    
    // Test display formatting with long content (should be truncated)
    let long_content = "a".repeat(200);
    let doc = Document::with_params(
        Some("test-id".to_string()),
        Some(long_content.clone()),
        None,
        None,
        None,
        None,
        None,
    );
    
    let display = format!("{}", doc);
    assert!(display.contains("test-id"));
    assert!(display.contains("..."));
    assert!(display.len() < long_content.len() + 50); // Ensure truncation happened
}