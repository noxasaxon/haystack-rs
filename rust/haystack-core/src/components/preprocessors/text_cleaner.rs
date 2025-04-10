use std::collections::HashMap;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component_system::{Component, ComponentBase, InputSocket, OutputSocket};

/// A component that cleans text strings by removing regex matches, converting 
/// to lowercase, removing punctuation and removing numbers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCleaner {
    #[serde(skip)]
    base: ComponentBase,
    
    /// Optional list of regex patterns to remove from the text
    #[serde(skip_serializing_if = "Option::is_none")]
    remove_regexps: Option<Vec<String>>,
    
    /// Whether to convert the text to lowercase
    #[serde(default)]
    convert_to_lowercase: bool,
    
    /// Whether to remove punctuation from the text
    #[serde(default)]
    remove_punctuation: bool,
    
    /// Whether to remove numbers from the text
    #[serde(default)]
    remove_numbers: bool,
    
    /// Compiled regex pattern
    #[serde(skip)]
    compiled_regex: Option<Regex>,
}

impl TextCleaner {
    /// Create a new TextCleaner component.
    ///
    /// # Arguments
    ///
    /// * `remove_regexps` - Optional list of regex patterns to remove from the text
    /// * `convert_to_lowercase` - Whether to convert the text to lowercase
    /// * `remove_punctuation` - Whether to remove punctuation from the text
    /// * `remove_numbers` - Whether to remove numbers from the text
    ///
    /// # Returns
    ///
    /// A new TextCleaner instance.
    pub fn new(
        remove_regexps: Option<Vec<String>>,
        convert_to_lowercase: bool,
        remove_punctuation: bool,
        remove_numbers: bool,
    ) -> Result<Self> {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        let mut output_sockets = HashMap::new();
        
        // Add input socket for documents
        input_sockets.insert(
            "texts".to_string(),
            crate::component_system::InputSocket::new(
                "texts".to_string(),
                std::any::TypeId::of::<Vec<String>>(),
                "Vec<String>".to_string(),
                None,
                false,
                false,
            ),
        );
        
        // Add output socket for documents
        output_sockets.insert(
            "texts".to_string(),
            crate::component_system::OutputSocket::new(
                "texts".to_string(),
                std::any::TypeId::of::<Vec<String>>(),
                "Vec<String>".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("convert_to_lowercase".to_string(), Value::from(convert_to_lowercase));
        init_parameters.insert("remove_punctuation".to_string(), Value::from(remove_punctuation));
        init_parameters.insert("remove_numbers".to_string(), Value::from(remove_numbers));
        
        if let Some(regexps) = &remove_regexps {
            init_parameters.insert("remove_regexps".to_string(), serde_json::to_value(regexps)?);
        }
        
        let mut instance = Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            remove_regexps,
            convert_to_lowercase,
            remove_punctuation,
            remove_numbers,
            compiled_regex: None,
        };
        
        // Compile regex pattern if remove_regexps is provided
        if let Some(patterns) = &instance.remove_regexps {
            if !patterns.is_empty() {
                // Create case-insensitive pattern by adding (?i) at the start
                let combined_pattern = format!("(?i){}", patterns.join("|"));
                instance.compiled_regex = Some(Regex::new(&combined_pattern)?);
            }
        }
        
        Ok(instance)
    }
    
    /// Clean a text string according to the specified options.
    fn clean_text(&self, text: &str) -> String {
        let mut cleaned_text = text.to_string();
        
        // Replace regex matches with empty string
        if let Some(regex) = &self.compiled_regex {
            cleaned_text = regex.replace_all(&cleaned_text, "").to_string();
        }
        
        // Convert to lowercase if required
        if self.convert_to_lowercase {
            cleaned_text = cleaned_text.to_lowercase();
        }
        
        // Remove punctuation if required
        if self.remove_punctuation {
            cleaned_text = cleaned_text.chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect();
        }
        
        // Remove numbers if required
        if self.remove_numbers {
            cleaned_text = cleaned_text.chars()
                .filter(|c| !c.is_ascii_digit())
                .collect();
        }
        
        // Clean up extra whitespace that might be introduced by the above operations
        let re = regex::Regex::new(r"\s+").unwrap();
        cleaned_text = re.replace_all(&cleaned_text, " ").to_string();
        cleaned_text = cleaned_text.trim().to_string();
        
        cleaned_text
    }
}

impl Component for TextCleaner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let texts = inputs.get("texts")
            .ok_or_else(|| anyhow::anyhow!("Missing required input 'texts'"))?;
        
        let texts = serde_json::from_value::<Vec<String>>(texts.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse 'texts' input: {}", e))?;
        
        let cleaned_texts: Vec<String> = texts.iter()
            .map(|text| self.clean_text(text))
            .collect();
        
        let mut outputs = HashMap::new();
        outputs.insert("texts".to_string(), serde_json::to_value(cleaned_texts)?);
        
        Ok(outputs)
    }
    
    fn warm_up(&self) -> Result<()> {
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_cleaner_empty_input() {
        let text_cleaner = TextCleaner::new(None, false, false, false).unwrap();
        let clean_text = text_cleaner.clean_text("");
        assert_eq!(clean_text, "");
    }
    
    #[test]
    fn test_text_cleaner_lowercase() {
        let text_cleaner = TextCleaner::new(None, true, false, false).unwrap();
        let clean_text = text_cleaner.clean_text("Hello World");
        assert_eq!(clean_text, "hello world");
    }
    
    #[test]
    fn test_text_cleaner_remove_punctuation() {
        let text_cleaner = TextCleaner::new(None, false, true, false).unwrap();
        let clean_text = text_cleaner.clean_text("Hello, World!");
        assert_eq!(clean_text, "Hello World");
    }
    
    #[test]
    fn test_text_cleaner_remove_numbers() {
        let text_cleaner = TextCleaner::new(None, false, false, true).unwrap();
        let clean_text = text_cleaner.clean_text("Hello 123 World");
        assert_eq!(clean_text, "Hello World");
    }
    
    #[test]
    fn test_text_cleaner_remove_regex() {
        let text_cleaner = TextCleaner::new(Some(vec!["[Hh]ello".to_string()]), false, false, false).unwrap();
        let clean_text = text_cleaner.clean_text("Hello World");
        assert_eq!(clean_text, "World");
    }
    
    #[test]
    fn test_text_cleaner_all_options() {
        let text_cleaner = TextCleaner::new(
            Some(vec!["test".to_string()]), 
            true, 
            true, 
            true
        ).unwrap();
        let clean_text = text_cleaner.clean_text("Hello, Test 123 World!");
        // Expected result has "test" removed (case insensitive), lowercase, no punctuation, no numbers
        assert_eq!(clean_text, "hello world");
    }
    
    #[test]
    fn test_text_cleaner_component_run() {
        let text_cleaner = TextCleaner::new(None, true, true, true).unwrap();
        
        let inputs = HashMap::from([
            ("texts".to_string(), serde_json::to_value(vec!["Hello, 123 World!".to_string()]).unwrap()),
        ]);
        
        let outputs = text_cleaner.run(inputs).unwrap();
        let cleaned_texts = serde_json::from_value::<Vec<String>>(outputs.get("texts").unwrap().clone()).unwrap();
        
        assert_eq!(cleaned_texts, vec!["hello world"]);
    }
}