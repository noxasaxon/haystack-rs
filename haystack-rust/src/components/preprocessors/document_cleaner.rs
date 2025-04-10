/*!
 * DocumentCleaner component
 * 
 * This component cleans the text in documents.
 */

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use unicase::UniCase;
use unicode_normalization::UnicodeNormalization;

use crate::component_system::{Component, ComponentBase};
use haystack_dataclasses::Document;

/// The DocumentCleaner component cleans the text in documents
/// 
/// It removes extra whitespaces, empty lines, specified substrings, regexes,
/// page headers and footers (in this order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCleaner {
    /// Base component implementation
    #[serde(skip)]
    base: ComponentBase,
    
    /// If `true`, removes empty lines
    pub remove_empty_lines: bool,
    
    /// If `true`, removes extra whitespaces
    pub remove_extra_whitespaces: bool,
    
    /// If `true`, removes repeated substrings (headers and footers) from pages
    pub remove_repeated_substrings: bool,
    
    /// If `true`, keeps the IDs of the original documents
    pub keep_id: bool,
    
    /// List of substrings to remove from the text
    pub remove_substrings: Option<Vec<String>>,
    
    /// Regex to match and replace substrings by ""
    pub remove_regex: Option<String>,
    
    /// Unicode normalization form to apply to the text
    pub unicode_normalization: Option<String>,
    
    /// Whether to convert the text to ASCII only
    pub ascii_only: bool,
}

impl DocumentCleaner {
    /// Create a new DocumentCleaner component
    pub fn new(
        remove_empty_lines: bool,
        remove_extra_whitespaces: bool,
        remove_repeated_substrings: bool,
        keep_id: bool,
        remove_substrings: Option<Vec<String>>,
        remove_regex: Option<String>,
        unicode_normalization: Option<String>,
        ascii_only: bool,
    ) -> Result<Self> {
        // Validate parameters
        if let Some(unicode_form) = &unicode_normalization {
            if !["NFC", "NFKC", "NFD", "NFKD"].contains(&unicode_form.as_str()) {
                return Err(anyhow!("unicode_normalization must be one of 'NFC', 'NFKC', 'NFD', 'NFKD'."));
            }
        }
        
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        let mut output_sockets = HashMap::new();
        
        // Add input socket for documents
        input_sockets.insert(
            "documents".to_string(),
            crate::component_system::InputSocket::new(
                "documents".to_string(),
                std::any::TypeId::of::<Vec<Document>>(),
                "Vec<Document>".to_string(),
                None,
                false,
                false,
            ),
        );
        
        // Add output socket for documents
        output_sockets.insert(
            "documents".to_string(),
            crate::component_system::OutputSocket::new(
                "documents".to_string(),
                std::any::TypeId::of::<Vec<Document>>(),
                "Vec<Document>".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("remove_empty_lines".to_string(), Value::from(remove_empty_lines));
        init_parameters.insert("remove_extra_whitespaces".to_string(), Value::from(remove_extra_whitespaces));
        init_parameters.insert("remove_repeated_substrings".to_string(), Value::from(remove_repeated_substrings));
        init_parameters.insert("keep_id".to_string(), Value::from(keep_id));
        
        if let Some(substrings) = &remove_substrings {
            init_parameters.insert("remove_substrings".to_string(), serde_json::to_value(substrings)?);
        }
        
        if let Some(regex) = &remove_regex {
            init_parameters.insert("remove_regex".to_string(), Value::from(regex.clone()));
        }
        
        if let Some(unicode_form) = &unicode_normalization {
            init_parameters.insert("unicode_normalization".to_string(), Value::from(unicode_form.clone()));
        }
        
        init_parameters.insert("ascii_only".to_string(), Value::from(ascii_only));
        
        Ok(Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            remove_empty_lines,
            remove_extra_whitespaces,
            remove_repeated_substrings,
            keep_id,
            remove_substrings,
            remove_regex,
            unicode_normalization,
            ascii_only,
        })
    }
    
    /// Normalize the unicode of the text
    fn normalize_unicode(&self, text: &str, form: &str) -> String {
        match form {
            "NFC" => text.nfc().collect::<String>(),
            "NFKC" => text.nfkc().collect::<String>(),
            "NFD" => text.nfd().collect::<String>(),
            "NFKD" => text.nfkd().collect::<String>(),
            _ => text.to_string(), // This should never happen due to validation
        }
    }
    
    /// Convert the text to ASCII only
    fn ascii_only(&self, text: &str) -> String {
        // First normalize the text to NFKD to separate the characters and their diacritics
        let normalized = self.normalize_unicode(text, "NFKD");
        
        // Then filter out non-ASCII characters
        normalized.chars()
            .filter(|c| c.is_ascii())
            .collect::<String>()
    }
    
    /// Remove empty lines from text
    fn remove_empty_lines(&self, text: &str) -> String {
        let pages: Vec<&str> = text.split('\u{000C}').collect(); // Split by form feed
        let cleaned_pages: Vec<String> = pages.iter()
            .map(|&page| {
                page.split('\n')
                    .filter(|line| !line.trim().is_empty())
                    .collect::<Vec<&str>>()
                    .join("\n")
            })
            .collect();
        
        cleaned_pages.join("\u{000C}")
    }
    
    /// Remove extra whitespaces from text
    fn remove_extra_whitespaces(&self, text: &str) -> String {
        let re = Regex::new(r"\s\s+").unwrap();
        let pages: Vec<&str> = text.split('\u{000C}').collect(); // Split by form feed
        
        let cleaned_pages: Vec<String> = pages.iter()
            .map(|&page| {
                re.replace_all(page, " ").trim().to_string()
            })
            .collect();
        
        cleaned_pages.join("\u{000C}")
    }
    
    /// Remove substrings that match the specified regex from the text
    fn remove_regex(&self, text: &str, regex: &str) -> Result<String> {
        let re = Regex::new(regex).map_err(|e| anyhow!("Invalid regex: {}", e))?;
        let pages: Vec<&str> = text.split('\u{000C}').collect(); // Split by form feed
        
        let cleaned_pages: Vec<String> = pages.iter()
            .map(|&page| {
                re.replace_all(page, "").trim().to_string()
            })
            .collect();
        
        Ok(cleaned_pages.join("\u{000C}"))
    }
    
    /// Remove all specified substrings from the text
    fn remove_substrings(&self, text: &str, substrings: &[String]) -> String {
        let mut cleaned_text = text.to_string();
        
        for substring in substrings {
            cleaned_text = cleaned_text.replace(substring, "");
        }
        
        cleaned_text
    }
    
    /// Remove any substrings that occur at the beginning or end of each page
    fn remove_repeated_substrings(&self, text: &str) -> String {
        self.find_and_remove_header_footer(
            text, 
            300, // n_chars 
            1,   // n_first_pages_to_ignore
            1    // n_last_pages_to_ignore
        )
    }
    
    /// Find and remove headers and footers across different pages
    fn find_and_remove_header_footer(
        &self,
        text: &str,
        n_chars: usize,
        n_first_pages_to_ignore: usize,
        n_last_pages_to_ignore: usize,
    ) -> String {
        let pages: Vec<&str> = text.split('\u{000C}').collect();
        
        if pages.len() <= n_first_pages_to_ignore + n_last_pages_to_ignore {
            return text.to_string();
        }
        
        // Extract page sections for header/footer detection
        let start_of_pages: Vec<String> = pages[n_first_pages_to_ignore..pages.len()-n_last_pages_to_ignore]
            .iter()
            .map(|&p| {
                p.chars().take(n_chars).collect::<String>()
            })
            .collect();
        
        let end_of_pages: Vec<String> = pages[n_first_pages_to_ignore..pages.len()-n_last_pages_to_ignore]
            .iter()
            .map(|&p| {
                let chars_count = p.chars().count();
                let start = if chars_count > n_chars { chars_count - n_chars } else { 0 };
                p.chars().skip(start).collect::<String>()
            })
            .collect();
        
        // Find common headers and footers
        let found_header = self.find_longest_common_ngram(&start_of_pages, 3, 30);
        let found_footer = self.find_longest_common_ngram(&end_of_pages, 3, 30);
        
        // Remove headers and footers
        let mut cleaned_pages = Vec::new();
        for page in pages {
            let mut cleaned_page = page.to_string();
            
            if let Some(header) = &found_header {
                cleaned_page = cleaned_page.replace(header, "");
            }
            
            if let Some(footer) = &found_footer {
                cleaned_page = cleaned_page.replace(footer, "");
            }
            
            cleaned_pages.push(cleaned_page);
        }
        
        cleaned_pages.join("\u{000C}")
    }
    
    /// Generate all n-grams of a specific length from a text
    fn ngram(&self, text: &str, n: usize) -> Vec<String> {
        // Replace newlines and tabs with special markers to preserve them
        let text = text.replace('\n', " \n").replace('\t', " \t");
        
        let words: Vec<&str> = text.split(' ').collect();
        let mut ngrams = Vec::new();
        
        for i in 0..=words.len().saturating_sub(n) {
            let ngram = words[i..i+n].join(" ")
                .replace(" \n", "\n")
                .replace(" \t", "\t");
            ngrams.push(ngram);
        }
        
        ngrams
    }
    
    /// Generate all possible n-grams from a given text
    fn allngram(&self, text: &str, min_ngram: usize, max_ngram: usize) -> HashMap<UniCase<String>, ()> {
        let mut ngrams = HashMap::new();
        
        let max_n = if max_ngram > 0 { 
            max_ngram 
        } else { 
            text.split(' ').count() 
        };
        
        for n in min_ngram..=max_n {
            for ngram in self.ngram(text, n) {
                ngrams.insert(UniCase::new(ngram), ());
            }
        }
        
        ngrams
    }
    
    /// Find the longest common n-gram across a list of text sequences
    fn find_longest_common_ngram(
        &self, 
        sequences: &[String], 
        min_ngram: usize, 
        max_ngram: usize
    ) -> Option<String> {
        // Filter empty sequences
        let sequences: Vec<&str> = sequences.iter().filter(|s| !s.is_empty()).map(|s| s.as_str()).collect();
        
        if sequences.is_empty() {
            return None;
        }
        
        // Generate n-grams for all sequences
        let seqs_ngrams: Vec<HashMap<UniCase<String>, ()>> = sequences.iter()
            .map(|&s| self.allngram(s, min_ngram, max_ngram))
            .collect();
        
        // Find intersection of all n-grams
        let mut intersection = seqs_ngrams[0].clone();
        for ngrams in &seqs_ngrams[1..] {
            intersection.retain(|key, _| ngrams.contains_key(key));
        }
        
        // Find longest n-gram
        if let Some((longest, _)) = intersection.iter()
            .filter(|(k, _)| !k.as_str().trim().is_empty())
            .max_by_key(|(k, _)| k.as_str().len()) {
            Some(longest.as_str().to_string())
        } else {
            None
        }
    }
}

impl Component for DocumentCleaner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Get the input documents
        let documents = inputs.get("documents")
            .ok_or_else(|| anyhow!("Missing 'documents' input"))?;
        
        let documents: Vec<Document> = serde_json::from_value(documents.clone())
            .map_err(|e| anyhow!("Failed to deserialize documents: {}", e))?;
        
        // Check input
        if documents.is_empty() {
            return Ok(HashMap::from([
                ("documents".to_string(), serde_json::to_value(Vec::<Document>::new())?)
            ]));
        }
        
        // Process each document
        let mut cleaned_docs = Vec::new();
        
        for doc in documents {
            if doc.content.is_none() {
                tracing::warn!("DocumentCleaner only cleans text documents but document.content for document ID {} is None.", 
                    doc.id);
                cleaned_docs.push(doc);
                continue;
            }
            
            let mut text = doc.content.clone().unwrap_or_default();
            
            // Apply cleaning steps in order
            if let Some(unicode_form) = &self.unicode_normalization {
                text = self.normalize_unicode(&text, unicode_form);
            }
            
            if self.ascii_only {
                text = self.ascii_only(&text);
            }
            
            if self.remove_extra_whitespaces {
                text = self.remove_extra_whitespaces(&text);
            }
            
            if self.remove_empty_lines {
                text = self.remove_empty_lines(&text);
            }
            
            if let Some(substrings) = &self.remove_substrings {
                text = self.remove_substrings(&text, substrings);
            }
            
            if let Some(regex) = &self.remove_regex {
                text = self.remove_regex(&text, regex)?;
            }
            
            if self.remove_repeated_substrings {
                text = self.remove_repeated_substrings(&text);
            }
            
            // Create a new cleaned document
            let clean_doc = Document::with_params(
                if self.keep_id { Some(doc.id) } else { None },
                Some(text),
                doc.blob.clone(),
                Some(doc.meta.clone()),
                doc.score,
                doc.embedding.clone(),
                doc.sparse_embedding.clone(),
            );
            
            cleaned_docs.push(clean_doc);
        }
        
        // Return the cleaned documents
        Ok(HashMap::from([
            ("documents".to_string(), serde_json::to_value(cleaned_docs)?)
        ]))
    }
    
    fn input_sockets(&self) -> &HashMap<String, crate::component_system::InputSocket> {
        self.base.input_sockets()
    }
    
    fn output_sockets(&self) -> &HashMap<String, crate::component_system::OutputSocket> {
        self.base.output_sockets()
    }
    
    fn init_parameters(&self) -> &HashMap<String, Value> {
        self.base.init_parameters()
    }
}