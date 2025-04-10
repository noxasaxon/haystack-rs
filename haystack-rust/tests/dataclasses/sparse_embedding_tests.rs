use std::collections::HashMap;
use serde_json::json;
use haystack_dataclasses::SparseEmbedding;

#[test]
fn test_sparse_embedding_creation() {
    let indices = vec![1, 4, 10];
    let values = vec![0.5, 0.8, 0.2];
    
    let sparse_embedding = SparseEmbedding::new(indices.clone(), values.clone()).unwrap();
    
    assert_eq!(sparse_embedding.indices, indices);
    assert_eq!(sparse_embedding.values, values);
}

#[test]
fn test_sparse_embedding_creation_error() {
    // Different lengths should cause an error
    let indices = vec![1, 4, 10];
    let values = vec![0.5, 0.8];
    
    let result = SparseEmbedding::new(indices, values);
    assert!(result.is_err());
}

#[test]
fn test_sparse_embedding_to_dict() {
    let indices = vec![1, 4, 10];
    let values = vec![0.5, 0.8, 0.2];
    
    let sparse_embedding = SparseEmbedding::new(indices.clone(), values.clone()).unwrap();
    let dict = sparse_embedding.to_dict();
    
    assert_eq!(dict.len(), 2);
    assert!(dict.contains_key("indices"));
    assert!(dict.contains_key("values"));
    
    let indices_json = dict.get("indices").unwrap();
    let values_json = dict.get("values").unwrap();
    
    // Convert the JSON values back to Vec
    let indices_vec: Vec<i32> = serde_json::from_value(indices_json.clone()).unwrap();
    let values_vec: Vec<f32> = serde_json::from_value(values_json.clone()).unwrap();
    
    assert_eq!(indices_vec, indices);
    assert_eq!(values_vec, values);
}

#[test]
fn test_sparse_embedding_from_dict() {
    let indices = vec![1, 4, 10];
    let values = vec![0.5, 0.8, 0.2];
    
    let mut dict = HashMap::new();
    dict.insert("indices".to_string(), json!(indices));
    dict.insert("values".to_string(), json!(values));
    
    let sparse_embedding = SparseEmbedding::from_dict(&dict).unwrap();
    
    assert_eq!(sparse_embedding.indices, indices);
    assert_eq!(sparse_embedding.values, values);
}

#[test]
fn test_sparse_embedding_from_dict_errors() {
    // Missing indices
    let mut dict = HashMap::new();
    dict.insert("values".to_string(), json!([0.5, 0.8, 0.2]));
    
    let result = SparseEmbedding::from_dict(&dict);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing 'indices'"));
    
    // Missing values
    let mut dict = HashMap::new();
    dict.insert("indices".to_string(), json!([1, 4, 10]));
    
    let result = SparseEmbedding::from_dict(&dict);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing 'values'"));
    
    // Different lengths
    let mut dict = HashMap::new();
    dict.insert("indices".to_string(), json!([1, 4, 10]));
    dict.insert("values".to_string(), json!([0.5, 0.8]));
    
    let result = SparseEmbedding::from_dict(&dict);
    // Just check that it returns an error - the exact message may vary
    assert!(result.is_err());
}

#[test]
fn test_sparse_embedding_display() {
    let indices = vec![1, 4, 10];
    let values = vec![0.5, 0.8, 0.2];
    
    let sparse_embedding = SparseEmbedding::new(indices, values).unwrap();
    let display = format!("{}", sparse_embedding);
    
    assert!(display.contains("SparseEmbedding"));
    assert!(display.contains("3"));
}