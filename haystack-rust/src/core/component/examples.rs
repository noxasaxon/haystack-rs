/*!
 * Example components for Haystack
 * 
 * This module contains simple example components to demonstrate the component interface.
 */

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::Value;

use super::{Component, ComponentBase, InputSocket, OutputSocket, Variadic, GreedyVariadic};
use crate::extract_variadic;

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
        
        println!("TextSplitter output chunks: {:?}", chunks);
        
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

/// A component that joins multiple text inputs into a single text with a separator
/// using variadic inputs
pub struct VariadicTextJoiner {
    /// Base component implementation
    base: ComponentBase,
    
    /// The separator to use between texts
    separator: String,
}

impl VariadicTextJoiner {
    /// Create a new VariadicTextJoiner component
    pub fn new(separator: String) -> Self {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        input_sockets.insert(
            "texts".to_string(),
            InputSocket::new_variadic::<String>(
                "texts".to_string(),
                "String".to_string(),
                None,
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
        init_parameters.insert("separator".to_string(), Value::from(separator.clone()));
        
        Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            separator,
        }
    }
}

impl Component for VariadicTextJoiner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Extract the variadic input, with special handling for pipeline connections
        let texts = if let Some(v) = inputs.get("texts") {
            println!("VariadicTextJoiner input: {:?}", v);
            
            if v.is_array() {
                let array = v.as_array().unwrap();
                println!("Array inputs: {:?}", array);
                
                // If array elements are strings, just extract them directly
                let values: Vec<String> = array.iter()
                    .filter_map(|v| {
                        let result = if v.is_string() {
                            Some(v.as_str().unwrap().to_string())
                        } else {
                            println!("Non-string value: {:?}", v);
                            v.as_str().map(|s| s.to_string())
                        };
                        result
                    })
                    .collect();
                
                println!("Extracted string values: {:?}", values);
                Variadic::from(values)
            } else {
                println!("Non-array input: {:?}", v);
                extract_variadic!(inputs, "texts", String)
            }
        } else {
            println!("No 'texts' input found in: {:?}", inputs.keys());
            Variadic::from(Vec::<String>::new())
        };
        
        // Join the texts
        let joined_text = texts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(&self.separator);
        let count = texts.len();
        
        // Create the outputs
        let mut outputs = HashMap::new();
        outputs.insert("text".to_string(), serde_json::to_value(joined_text)?);
        outputs.insert("count".to_string(), serde_json::to_value(count)?);
        
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

/// A component that processes text as soon as it receives it
/// using greedy variadic inputs
pub struct GreedyTextProcessor {
    /// Base component implementation
    base: ComponentBase,
    
    /// The prefix to add to each text
    prefix: String,
}

impl GreedyTextProcessor {
    /// Create a new GreedyTextProcessor component
    pub fn new(prefix: String) -> Self {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        input_sockets.insert(
            "texts".to_string(),
            InputSocket::new_variadic::<String>(
                "texts".to_string(),
                "String".to_string(),
                None,
                true, // greedy
            ),
        );
        
        let mut output_sockets = HashMap::new();
        output_sockets.insert(
            "processed_text".to_string(),
            OutputSocket::new(
                "processed_text".to_string(),
                std::any::TypeId::of::<String>(),
                "String".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("prefix".to_string(), Value::from(prefix.clone()));
        
        Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            prefix,
        }
    }
}

impl Component for GreedyTextProcessor {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        println!("GreedyTextProcessor inputs: {:?}", inputs);
        
        // Extract the greedy variadic input, with better handling for array format
        let texts = if let Some(v) = inputs.get("texts") {
            if v.is_array() {
                let array = v.as_array().unwrap();
                
                // Extract string values from the array
                let values: Vec<String> = array.iter()
                    .filter_map(|v| {
                        if v.is_string() {
                            Some(v.as_str().unwrap().to_string())
                        } else {
                            v.as_str().map(|s| s.to_string())
                        }
                    })
                    .collect();
                    
                println!("Extracted string values: {:?}", values);
                if values.is_empty() {
                    return Err(anyhow!("No text values found in array input"));
                }
                values
            } else if v.is_string() {
                vec![v.as_str().unwrap().to_string()]
            } else {
                return Err(anyhow!("Expected array or string for texts input, got: {:?}", v));
            }
        } else {
            println!("No 'texts' input found in: {:?}", inputs.keys());
            return Err(anyhow!("No texts input provided"));
        };
        
        if texts.is_empty() {
            return Err(anyhow!("No texts to process"));
        }
        
        // Process the text (in a real component this would be more complex)
        let processed_text = format!("{}{}", self.prefix, texts[0]);
        println!("Processed text: {}", processed_text);
        
        // Create the outputs
        let mut outputs = HashMap::new();
        outputs.insert("processed_text".to_string(), serde_json::to_value(processed_text)?);
        
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