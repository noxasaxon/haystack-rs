/*!
 * Example components for Haystack
 * 
 * This module contains simple example components to demonstrate the component interface.
 */

use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value;

use super::{Component, ComponentBase, InputSocket, OutputSocket};

/// A simple text splitter component that splits text into chunks
pub struct TextSplitter {
    /// The base component implementation
    base: ComponentBase,
    
    /// The maximum length of each chunk
    max_chunk_length: usize,
}

impl TextSplitter {
    /// Create a new TextSplitter component
    pub fn new(max_chunk_length: usize) -> Self {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        input_sockets.insert(
            "text".to_string(),
            InputSocket::new(
                "text".to_string(),
                std::any::TypeId::of::<String>(),
                "String".to_string(),
                None,
                false,
                false,
            ),
        );
        
        let mut output_sockets = HashMap::new();
        output_sockets.insert(
            "chunks".to_string(),
            OutputSocket::new(
                "chunks".to_string(),
                std::any::TypeId::of::<Vec<String>>(),
                "Vec<String>".to_string(),
            ),
        );
        output_sockets.insert(
            "count".to_string(),
            OutputSocket::new(
                "count".to_string(),
                std::any::TypeId::of::<usize>(),
                "usize".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("max_chunk_length".to_string(), Value::from(max_chunk_length));
        
        Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            max_chunk_length,
        }
    }
}

impl Component for TextSplitter {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Get the input text
        let text = inputs
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Split the text into chunks of max_chunk_length
        let chunks: Vec<String> = text
            .chars()
            .collect::<Vec<_>>()
            .chunks(self.max_chunk_length)
            .map(|c| c.iter().collect::<String>())
            .collect();
        
        // Create the outputs
        let mut outputs = HashMap::new();
        outputs.insert("chunks".to_string(), serde_json::to_value(chunks.clone())?);
        outputs.insert("count".to_string(), serde_json::to_value(chunks.len())?);
        
        Ok(outputs)
    }
    
    fn input_sockets(&self) -> &HashMap<String, InputSocket> {
        self.base.input_sockets()
    }
    
    fn output_sockets(&self) -> &HashMap<String, OutputSocket> {
        self.base.output_sockets()
    }
    
    fn init_parameters(&self) -> &HashMap<String, Value> {
        self.base.init_parameters()
    }
}

/// A simple text joiner component that joins text chunks
pub struct TextJoiner {
    /// The base component implementation
    base: ComponentBase,
    
    /// The separator to use between chunks
    separator: String,
}

impl TextJoiner {
    /// Create a new TextJoiner component
    pub fn new(separator: String) -> Self {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        input_sockets.insert(
            "chunks".to_string(),
            InputSocket::new(
                "chunks".to_string(),
                std::any::TypeId::of::<Vec<String>>(),
                "Vec<String>".to_string(),
                None,
                false,
                false,
            ),
        );
        
        let mut output_sockets = HashMap::new();
        output_sockets.insert(
            "text".to_string(),
            OutputSocket::new(
                "text".to_string(),
                std::any::TypeId::of::<String>(),
                "String".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("separator".to_string(), Value::from(separator.clone()));
        
        Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            separator,
        }
    }
}

impl Component for TextJoiner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Get the input chunks
        let chunks = inputs
            .get("chunks")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                   .filter_map(|v| v.as_str())
                   .map(|s| s.to_string())
                   .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        // Join the chunks
        let joined_text = chunks.join(&self.separator);
        
        // Create the outputs
        let mut outputs = HashMap::new();
        outputs.insert("text".to_string(), serde_json::to_value(joined_text)?);
        
        Ok(outputs)
    }
    
    fn input_sockets(&self) -> &HashMap<String, InputSocket> {
        self.base.input_sockets()
    }
    
    fn output_sockets(&self) -> &HashMap<String, OutputSocket> {
        self.base.output_sockets()
    }
    
    fn init_parameters(&self) -> &HashMap<String, Value> {
        self.base.init_parameters()
    }
}