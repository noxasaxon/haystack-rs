use std::collections::HashMap;
use serde_json::Value;

use crate::component_system::{TextSplitter, TextJoiner};
use crate::pipeline::Pipeline;

#[test]
fn test_pipeline_with_splitter_and_joiner() {
    let mut pipeline = Pipeline::new();
    
    // Add components
    pipeline.add_component("splitter", TextSplitter::new(5)).unwrap();
    pipeline.add_component("joiner", TextJoiner::new(" | ".to_string())).unwrap();
    
    // Connect components
    pipeline.connect("splitter", "chunks", "joiner", "chunks").unwrap();
    
    // Prepare input
    let mut inputs = HashMap::new();
    inputs.insert("splitter.text".to_string(), Value::String("Hello, world!".to_string()));
    
    // Run pipeline
    let outputs = pipeline.run(inputs).unwrap();
    
    // Check outputs
    let joined_text = outputs.get("joiner.text").unwrap().as_str().unwrap();
    assert_eq!(joined_text, "Hello | , wor | ld!");
    
    let count = outputs.get("splitter.count").unwrap().as_u64().unwrap();
    assert_eq!(count, 3);
}