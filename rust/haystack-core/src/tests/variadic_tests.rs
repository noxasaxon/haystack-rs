/*!
 * Tests for variadic sockets
 */

use std::collections::HashMap;
use serde_json::json;

use crate::component::{Component, TextSplitter};
use crate::component::examples::{VariadicTextJoiner, GreedyTextProcessor};
use crate::pipeline::{Pipeline, ConnectionValidation};

#[test]
fn test_variadic_text_joiner() {
    let joiner = VariadicTextJoiner::new(", ".to_string());
    
    // Prepare inputs
    let mut inputs = HashMap::new();
    inputs.insert("texts".to_string(), json!(["Hello", "World", "Test"]));
    
    // Run the component
    let outputs = joiner.run(inputs).unwrap();
    
    // Verify outputs
    assert_eq!(outputs.get("text").unwrap().as_str().unwrap(), "Hello, World, Test");
    assert_eq!(outputs.get("count").unwrap().as_u64().unwrap(), 3);
}

#[test]
fn test_greedy_text_processor() {
    let processor = GreedyTextProcessor::new("Processed: ".to_string());
    
    // Prepare inputs
    let mut inputs = HashMap::new();
    inputs.insert("texts".to_string(), json!(["Hello"]));
    
    // Run the component
    let outputs = processor.run(inputs).unwrap();
    
    // Verify outputs
    assert_eq!(outputs.get("processed_text").unwrap().as_str().unwrap(), "Processed: Hello");
}

#[test]
fn test_variadic_component_in_pipeline() {
    // Create pipeline components
    let splitter = TextSplitter::new(5);
    let joiner = VariadicTextJoiner::new(" | ".to_string());
    
    // Create pipeline
    let mut pipeline = Pipeline::with_validation(ConnectionValidation::Relaxed);
    pipeline.add_component("splitter", splitter).unwrap();
    pipeline.add_component("joiner", joiner).unwrap();
    
    // Connect components
    pipeline.connect("splitter", "chunks", "joiner", "texts").unwrap();
    
    // Prepare inputs
    let mut inputs = HashMap::new();
    inputs.insert("splitter.text".to_string(), json!("HelloWorldThisIsATest"));
    
    // Remove any placeholder - we want to get our values from the splitter
    // inputs.insert("joiner.texts".to_string(), json!([])); // This causes issues
    
    // Run the pipeline
    let outputs = pipeline.run(inputs).unwrap();
    
    // Print all outputs for debugging
    println!("Pipeline outputs: {:?}", outputs);
    
    // Verify outputs
    let expected_text = "Hello | World | ThisI | sATes | t";
    assert_eq!(outputs.get("joiner.text").unwrap().as_str().unwrap(), expected_text);
    assert_eq!(outputs.get("joiner.count").unwrap().as_u64().unwrap(), 5);
}

#[test]
fn test_multiple_inputs_to_variadic() {
    // Create pipeline components
    let splitter1 = TextSplitter::new(5);
    let splitter2 = TextSplitter::new(3);
    let joiner = VariadicTextJoiner::new(" + ".to_string());
    
    // Create pipeline
    let mut pipeline = Pipeline::with_validation(ConnectionValidation::Relaxed);
    pipeline.add_component("splitter1", splitter1).unwrap();
    pipeline.add_component("splitter2", splitter2).unwrap();
    pipeline.add_component("joiner", joiner).unwrap();
    
    // Connect components
    pipeline.connect("splitter1", "chunks", "joiner", "texts").unwrap();
    pipeline.connect("splitter2", "chunks", "joiner", "texts").unwrap();
    
    // Prepare inputs
    let mut inputs = HashMap::new();
    inputs.insert("splitter1.text".to_string(), json!("HelloWorld"));
    inputs.insert("splitter2.text".to_string(), json!("Testing"));
    
    // Run the pipeline
    let outputs = pipeline.run(inputs).unwrap();
    
    // Verify outputs - should have all chunks from both splitters
    let joiner_text = outputs.get("joiner.text").unwrap().as_str().unwrap();
    let count = outputs.get("joiner.count").unwrap().as_u64().unwrap();
    
    // Should have 2 chunks from splitter1 and 3 chunks from splitter2
    assert_eq!(count, 5);
    
    // The text should contain all chunks joined with ' + '
    assert!(joiner_text.contains("Hello"));
    assert!(joiner_text.contains("World"));
    assert!(joiner_text.contains("Tes"));
    assert!(joiner_text.contains("tin"));
    assert!(joiner_text.contains("g"));
}

#[test]
fn test_greedy_variadic_in_pipeline() {
    // Create a simpler test with just the processor
    let processor = GreedyTextProcessor::new("Processed: ".to_string());
    
    // Create pipeline with just the processor
    let mut pipeline = Pipeline::new();
    pipeline.add_component("processor", processor).unwrap();
    
    // Directly provide the texts input
    let mut direct_inputs = HashMap::new();
    direct_inputs.insert("processor.texts".to_string(), json!(["Hello", "World"]));
    
    // Run the pipeline with direct inputs
    let outputs = pipeline.run(direct_inputs).unwrap();
    
    // Print all outputs for debugging
    println!("Pipeline outputs: {:?}", outputs);
    
    // Verify outputs - processor should have output
    assert!(outputs.contains_key("processor.processed_text"), "Missing processor.processed_text in outputs");
    
    let processed_text = outputs.get("processor.processed_text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
        
    println!("Processed text: {}", processed_text);
    assert!(processed_text.starts_with("Processed:"), "Expected processed text to start with 'Processed:', got: {}", processed_text);
}