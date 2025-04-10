use std::collections::HashMap;
use serde_json::Value;
use haystack_dataclasses::{Answer, Document, ExtractedAnswer, GeneratedAnswer, Span};

#[test]
fn test_extracted_answer_creation() {
    let doc = Document::with_params(
        Some("test-doc-id".to_string()),
        Some("This is a test document with some content.".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    let meta = HashMap::from([
        ("source".to_string(), Value::String("test".to_string())),
        ("confidence".to_string(), Value::Number(serde_json::Number::from_f64(0.95).unwrap())),
    ]);
    
    let answer = ExtractedAnswer::new(
        "What is this?".to_string(),
        0.8,
        Some("a test document".to_string()),
        Some(doc.clone()),
        Some("This is a test document".to_string()),
        Some(Span { start: 8, end: 23 }),
        Some(Span { start: 8, end: 23 }),
        Some(meta.clone()),
    );
    
    assert_eq!(answer.query, "What is this?");
    assert_eq!(answer.score, 0.8);
    assert_eq!(answer.data, Some("a test document".to_string()));
    assert!(answer.document.is_some());
    assert_eq!(answer.document.as_ref().unwrap().id, "test-doc-id");
    assert_eq!(answer.context, Some("This is a test document".to_string()));
    assert!(answer.document_offset.is_some());
    assert_eq!(answer.document_offset.as_ref().unwrap().start, 8);
    assert_eq!(answer.document_offset.as_ref().unwrap().end, 23);
    assert!(answer.context_offset.is_some());
    assert_eq!(answer.context_offset.as_ref().unwrap().start, 8);
    assert_eq!(answer.context_offset.as_ref().unwrap().end, 23);
    assert_eq!(answer.meta.len(), 2);
    assert_eq!(answer.meta["source"], Value::String("test".to_string()));
    assert_eq!(answer.meta["confidence"], Value::Number(serde_json::Number::from_f64(0.95).unwrap()));
}

#[test]
fn test_extracted_answer_to_dict() {
    let doc = Document::with_params(
        Some("test-doc-id".to_string()),
        Some("This is a test document with some content.".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    let meta = HashMap::from([
        ("source".to_string(), Value::String("test".to_string())),
    ]);
    
    let answer = ExtractedAnswer::new(
        "What is this?".to_string(),
        0.8,
        Some("a test document".to_string()),
        Some(doc.clone()),
        Some("This is a test document".to_string()),
        Some(Span { start: 8, end: 23 }),
        Some(Span { start: 8, end: 23 }),
        Some(meta.clone()),
    );
    
    let dict = answer.to_dict().unwrap();
    
    // Check type and init_parameters structure
    assert!(dict.is_object());
    let obj = dict.as_object().unwrap();
    assert_eq!(obj["type"].as_str().unwrap(), "ExtractedAnswer");
    assert!(obj.contains_key("init_parameters"));
    
    // Check init parameters
    let params = obj["init_parameters"].as_object().unwrap();
    assert_eq!(params["query"].as_str().unwrap(), "What is this?");
    // Due to floating point representation, compare approximately
    let score = params["score"].as_f64().unwrap();
    assert!((score - 0.8).abs() < 1e-6);
    assert_eq!(params["data"].as_str().unwrap(), "a test document");
    assert!(params.contains_key("document"));
    assert_eq!(params["context"].as_str().unwrap(), "This is a test document");
    assert!(params.contains_key("document_offset"));
    assert!(params.contains_key("context_offset"));
    assert!(params.contains_key("meta"));
}

#[test]
fn test_extracted_answer_from_dict() {
    // Create a dictionary representation
    let mut params = serde_json::Map::new();
    params.insert("query".to_string(), Value::String("What is this?".to_string()));
    params.insert("score".to_string(), Value::Number(serde_json::Number::from_f64(0.8).unwrap()));
    params.insert("data".to_string(), Value::String("a test document".to_string()));
    params.insert("context".to_string(), Value::String("This is a test document".to_string()));
    
    // Add document
    let doc_dict = Document::with_params(
        Some("test-doc-id".to_string()),
        Some("This is a test document with some content.".to_string()),
        None,
        None,
        None,
        None,
        None,
    ).to_dict(false);
    params.insert("document".to_string(), doc_dict);
    
    // Add spans
    let mut doc_offset = serde_json::Map::new();
    doc_offset.insert("start".to_string(), Value::Number(8.into()));
    doc_offset.insert("end".to_string(), Value::Number(23.into()));
    params.insert("document_offset".to_string(), Value::Object(doc_offset));
    
    let mut ctx_offset = serde_json::Map::new();
    ctx_offset.insert("start".to_string(), Value::Number(8.into()));
    ctx_offset.insert("end".to_string(), Value::Number(23.into()));
    params.insert("context_offset".to_string(), Value::Object(ctx_offset));
    
    // Add metadata
    let mut meta = serde_json::Map::new();
    meta.insert("source".to_string(), Value::String("test".to_string()));
    params.insert("meta".to_string(), Value::Object(meta));
    
    // Create final dict
    let mut dict = serde_json::Map::new();
    dict.insert("type".to_string(), Value::String("ExtractedAnswer".to_string()));
    dict.insert("init_parameters".to_string(), Value::Object(params));
    
    // Deserialize
    let answer = ExtractedAnswer::from_dict(&Value::Object(dict)).unwrap();
    
    assert_eq!(answer.query, "What is this?");
    assert_eq!(answer.score, 0.8);
    assert_eq!(answer.data, Some("a test document".to_string()));
    assert!(answer.document.is_some());
    assert_eq!(answer.document.as_ref().unwrap().id, "test-doc-id");
    assert_eq!(answer.context, Some("This is a test document".to_string()));
    assert!(answer.document_offset.is_some());
    assert_eq!(answer.document_offset.as_ref().unwrap().start, 8);
    assert_eq!(answer.document_offset.as_ref().unwrap().end, 23);
    assert!(answer.context_offset.is_some());
    assert_eq!(answer.context_offset.as_ref().unwrap().start, 8);
    assert_eq!(answer.context_offset.as_ref().unwrap().end, 23);
    assert_eq!(answer.meta.len(), 1);
    assert_eq!(answer.meta["source"], Value::String("test".to_string()));
}

#[test]
fn test_answer_trait_implementation_for_extracted_answer() {
    let answer = ExtractedAnswer::new(
        "What is this?".to_string(),
        0.8,
        Some("a test document".to_string()),
        None,
        None,
        None,
        None,
        None,
    );
    
    let answer_trait: &dyn Answer = &answer;
    
    assert_eq!(answer_trait.query(), "What is this?");
    assert!(answer_trait.meta().is_empty());
    assert!(answer_trait.to_dict().is_ok());
}

#[test]
fn test_generated_answer_creation() {
    let docs = vec![
        Document::with_params(
            Some("doc1".to_string()),
            Some("Document 1 content".to_string()),
            None,
            None,
            None,
            None,
            None,
        ),
        Document::with_params(
            Some("doc2".to_string()),
            Some("Document 2 content".to_string()),
            None,
            None,
            None,
            None,
            None,
        ),
    ];
    
    let meta = HashMap::from([
        ("source".to_string(), Value::String("llm".to_string())),
        ("model".to_string(), Value::String("gpt-4".to_string())),
    ]);
    
    let answer = GeneratedAnswer::new(
        "This is a generated answer".to_string(),
        "What is the question?".to_string(),
        docs.clone(),
        Some(meta.clone()),
    );
    
    assert_eq!(answer.data, "This is a generated answer");
    assert_eq!(answer.query, "What is the question?");
    assert_eq!(answer.documents.len(), 2);
    assert_eq!(answer.documents[0].id, "doc1");
    assert_eq!(answer.documents[1].id, "doc2");
    assert_eq!(answer.meta.len(), 2);
    assert_eq!(answer.meta["source"], Value::String("llm".to_string()));
    assert_eq!(answer.meta["model"], Value::String("gpt-4".to_string()));
}

#[test]
fn test_generated_answer_to_dict() {
    let docs = vec![
        Document::with_params(
            Some("doc1".to_string()),
            Some("Document 1 content".to_string()),
            None,
            None,
            None,
            None,
            None,
        ),
    ];
    
    let meta = HashMap::from([
        ("source".to_string(), Value::String("llm".to_string())),
    ]);
    
    let answer = GeneratedAnswer::new(
        "This is a generated answer".to_string(),
        "What is the question?".to_string(),
        docs.clone(),
        Some(meta.clone()),
    );
    
    let dict = answer.to_dict().unwrap();
    
    // Check type and init_parameters structure
    assert!(dict.is_object());
    let obj = dict.as_object().unwrap();
    assert_eq!(obj["type"].as_str().unwrap(), "GeneratedAnswer");
    assert!(obj.contains_key("init_parameters"));
    
    // Check init parameters
    let params = obj["init_parameters"].as_object().unwrap();
    assert_eq!(params["data"].as_str().unwrap(), "This is a generated answer");
    assert_eq!(params["query"].as_str().unwrap(), "What is the question?");
    assert!(params.contains_key("documents"));
    assert!(params["documents"].is_array());
    assert_eq!(params["documents"].as_array().unwrap().len(), 1);
    assert!(params.contains_key("meta"));
}

#[test]
fn test_generated_answer_from_dict() {
    // Create a dictionary representation
    let mut params = serde_json::Map::new();
    params.insert("data".to_string(), Value::String("This is a generated answer".to_string()));
    params.insert("query".to_string(), Value::String("What is the question?".to_string()));
    
    // Add documents
    let doc_dict = Document::with_params(
        Some("doc1".to_string()),
        Some("Document 1 content".to_string()),
        None,
        None,
        None,
        None,
        None,
    ).to_dict(false);
    params.insert("documents".to_string(), Value::Array(vec![doc_dict]));
    
    // Add metadata
    let mut meta = serde_json::Map::new();
    meta.insert("source".to_string(), Value::String("llm".to_string()));
    params.insert("meta".to_string(), Value::Object(meta));
    
    // Create final dict
    let mut dict = serde_json::Map::new();
    dict.insert("type".to_string(), Value::String("GeneratedAnswer".to_string()));
    dict.insert("init_parameters".to_string(), Value::Object(params));
    
    // Deserialize
    let answer = GeneratedAnswer::from_dict(&Value::Object(dict)).unwrap();
    
    assert_eq!(answer.data, "This is a generated answer");
    assert_eq!(answer.query, "What is the question?");
    assert_eq!(answer.documents.len(), 1);
    assert_eq!(answer.documents[0].id, "doc1");
    assert_eq!(answer.meta.len(), 1);
    assert_eq!(answer.meta["source"], Value::String("llm".to_string()));
}

#[test]
fn test_answer_trait_implementation_for_generated_answer() {
    let answer = GeneratedAnswer::new(
        "This is a generated answer".to_string(),
        "What is the question?".to_string(),
        Vec::new(),
        None,
    );
    
    let answer_trait: &dyn Answer = &answer;
    
    assert_eq!(answer_trait.query(), "What is the question?");
    assert!(answer_trait.meta().is_empty());
    assert!(answer_trait.to_dict().is_ok());
}