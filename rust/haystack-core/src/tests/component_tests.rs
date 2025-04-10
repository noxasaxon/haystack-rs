use std::collections::HashMap;
use serde_json::Value;

use crate::component_system::{Component, TextSplitter, TextJoiner};

#[test]
fn test_text_splitter() {
    let text_splitter = TextSplitter::new(5);
    
    let mut inputs = HashMap::new();
    inputs.insert("text".to_string(), Value::String("Hello, world!".to_string()));
    
    let outputs = text_splitter.run(inputs).unwrap();
    
    let chunks = outputs.get("chunks").unwrap().as_array().unwrap();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].as_str().unwrap(), "Hello");
    assert_eq!(chunks[1].as_str().unwrap(), ", wor");
    assert_eq!(chunks[2].as_str().unwrap(), "ld!");
    
    assert_eq!(outputs.get("count").unwrap().as_u64().unwrap(), 3);
}

#[test]
fn test_text_joiner() {
    let text_joiner = TextJoiner::new(" | ".to_string());
    
    let chunks = vec!["Hello", "world", "test"];
    
    let mut inputs = HashMap::new();
    inputs.insert("chunks".to_string(), serde_json::to_value(chunks).unwrap());
    
    let outputs = text_joiner.run(inputs).unwrap();
    
    let joined_text = outputs.get("text").unwrap().as_str().unwrap();
    assert_eq!(joined_text, "Hello | world | test");
}