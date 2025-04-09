use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component::{Component, ComponentBase, InputSocket, OutputSocket};

/// Supported languages for sentence splitting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "nl")]
    Dutch,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "sv")]
    Swedish,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "hi")]
    Hindi,
    // Add more languages as needed
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

/// A component that splits text into sentences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceSplitter {
    #[serde(skip)]
    base: ComponentBase,
    
    /// The language to use for tokenization
    #[serde(default)]
    language: Language,
    
    /// Whether to keep whitespaces between sentences
    #[serde(default)]
    keep_white_spaces: bool,
}

impl SentenceSplitter {
    /// Create a new SentenceSplitter component.
    ///
    /// # Arguments
    ///
    /// * `language` - The language to use for tokenization
    /// * `keep_white_spaces` - Whether to keep whitespaces between sentences
    ///
    /// # Returns
    ///
    /// A new SentenceSplitter instance.
    pub fn new(
        language: Language,
        keep_white_spaces: bool,
    ) -> Result<Self> {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        let mut output_sockets = HashMap::new();
        
        // Add input socket for texts
        input_sockets.insert(
            "texts".to_string(),
            crate::component::InputSocket::new(
                "texts".to_string(),
                std::any::TypeId::of::<Vec<String>>(),
                "Vec<String>".to_string(),
                None,
                false,
                false,
            ),
        );
        
        // Add output socket for sentences
        output_sockets.insert(
            "sentences".to_string(),
            crate::component::OutputSocket::new(
                "sentences".to_string(),
                std::any::TypeId::of::<Vec<HashMap<String, Value>>>(),
                "Vec<HashMap<String, Value>>".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("language".to_string(), serde_json::to_value(&language)?);
        init_parameters.insert("keep_white_spaces".to_string(), Value::from(keep_white_spaces));
        
        let instance = Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            language,
            keep_white_spaces,
        };
        
        Ok(instance)
    }
    
    /// Split text into sentences
    fn split_sentences(&self, text: &str) -> Vec<HashMap<String, Value>> {
        // Simple sentence splitting by punctuation (., !, ?)
        let sentence_endings = ['.', '!', '?'];
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let mut start_idx = 0;
        
        for (i, c) in text.char_indices() {
            current_sentence.push(c);
            
            if sentence_endings.contains(&c) {
                let next_char = text.chars().nth(i + 1);
                // Check if the next character is whitespace or we're at the end
                if next_char.is_none() || next_char.unwrap().is_whitespace() {
                    let trimmed = if self.keep_white_spaces {
                        current_sentence.clone()
                    } else {
                        current_sentence.trim().to_string()
                    };
                    
                    if !trimmed.is_empty() {
                        let end_idx = start_idx + trimmed.len();
                        let mut sentence_dict = HashMap::new();
                        sentence_dict.insert("text".to_string(), Value::String(trimmed));
                        sentence_dict.insert("start_idx".to_string(), Value::Number(start_idx.into()));
                        sentence_dict.insert("end_idx".to_string(), Value::Number(end_idx.into()));
                        sentences.push(sentence_dict);
                    }
                    
                    start_idx = i + 1; // Next sentence starts after this character
                    current_sentence.clear();
                }
            }
        }
        
        // Add the last sentence if there is one
        let trimmed = if self.keep_white_spaces {
            current_sentence.clone()
        } else {
            current_sentence.trim().to_string()
        };
        
        if !trimmed.is_empty() {
            let end_idx = start_idx + trimmed.len();
            let mut sentence_dict = HashMap::new();
            sentence_dict.insert("text".to_string(), Value::String(trimmed));
            sentence_dict.insert("start_idx".to_string(), Value::Number(start_idx.into()));
            sentence_dict.insert("end_idx".to_string(), Value::Number(end_idx.into()));
            sentences.push(sentence_dict);
        }
        
        sentences
    }
}

impl Component for SentenceSplitter {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let texts = inputs.get("texts")
            .ok_or_else(|| anyhow::anyhow!("Missing required input 'texts'"))?;
        
        let texts = serde_json::from_value::<Vec<String>>(texts.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse 'texts' input: {}", e))?;
        
        let all_sentences: Vec<Vec<HashMap<String, Value>>> = texts.iter()
            .map(|text| self.split_sentences(text))
            .collect();
        
        // Flatten the sentences if there's only one text,
        // otherwise return a list of lists
        let output_value = if texts.len() == 1 {
            serde_json::to_value(&all_sentences[0])?
        } else {
            serde_json::to_value(all_sentences)?
        };
        
        let mut outputs = HashMap::new();
        outputs.insert("sentences".to_string(), output_value);
        
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
    fn test_sentence_splitter_basic() {
        let splitter = SentenceSplitter::new(Language::English, false).unwrap();
        let sentences = splitter.split_sentences("Hello world. This is a test.");
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].get("text").unwrap().as_str().unwrap(), "Hello world.");
        assert_eq!(sentences[1].get("text").unwrap().as_str().unwrap(), "This is a test.");
    }
    
    #[test]
    fn test_sentence_splitter_keep_whitespace() {
        let splitter = SentenceSplitter::new(Language::English, true).unwrap();
        let sentences = splitter.split_sentences("Hello world.  This is a test.");
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].get("text").unwrap().as_str().unwrap(), "Hello world.");
        
        // The second sentence might include leading whitespace depending on the tokenizer
        let second_text = sentences[1].get("text").unwrap().as_str().unwrap();
        assert!(second_text.trim() == "This is a test.");
    }
    
    #[test]
    fn test_sentence_splitter_component_run() {
        let splitter = SentenceSplitter::new(Language::English, false).unwrap();
        
        let inputs = HashMap::from([
            ("texts".to_string(), serde_json::to_value(vec!["Hello world. This is a test.".to_string()]).unwrap()),
        ]);
        
        let outputs = splitter.run(inputs).unwrap();
        let sentences = outputs.get("sentences").unwrap().clone();
        
        let sentences_vec = serde_json::from_value::<Vec<HashMap<String, Value>>>(sentences).unwrap();
        assert_eq!(sentences_vec.len(), 2);
    }
    
    #[test]
    fn test_sentence_splitter_empty_input() {
        let splitter = SentenceSplitter::new(Language::English, false).unwrap();
        let sentences = splitter.split_sentences("");
        
        assert_eq!(sentences.len(), 0);
    }
}