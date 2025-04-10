/*!
 * DocumentSplitter component
 * 
 * This component splits long documents into smaller chunks.
 */

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::component_system::{Component, ComponentBase};
use haystack_dataclasses::Document;

/// Map of split by character types
const CHARACTER_SPLIT_BY_MAPPING: [(&str, &str); 5] = [
    ("page", "\u{000C}"),      // Form feed
    ("passage", "\n\n"),       // Double newline
    ("period", "."),           // Period
    ("word", " "),             // Space
    ("line", "\n"),            // Newline
];

/// The type of unit to split by
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitByType {
    /// Split by custom function
    Function,
    
    /// Split by page break
    Page,
    
    /// Split by double newline
    Passage,
    
    /// Split by period
    Period,
    
    /// Split by word
    Word,
    
    /// Split by line
    Line,
    
    /// Split by sentence
    Sentence,
}

impl SplitByType {
    /// Get the character used for splitting 
    pub fn get_split_char(&self) -> Option<&'static str> {
        match self {
            SplitByType::Function => None,
            SplitByType::Sentence => None,
            _ => {
                let split_by_str = match self {
                    SplitByType::Page => "page",
                    SplitByType::Passage => "passage",
                    SplitByType::Period => "period",
                    SplitByType::Word => "word",
                    SplitByType::Line => "line",
                    _ => unreachable!(),
                };
                
                CHARACTER_SPLIT_BY_MAPPING.iter()
                    .find(|(key, _)| *key == split_by_str)
                    .map(|(_, value)| *value)
            }
        }
    }
}

/// Callable type for custom splitting function
pub type SplittingFunction = Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>;

/// The DocumentSplitter component splits long documents into smaller chunks
#[derive(Clone)]
pub struct DocumentSplitter {
    /// Base component implementation
    base: ComponentBase,
    
    /// The unit for splitting documents
    split_by: SplitByType,
    
    /// The maximum number of units in each split
    split_length: usize,
    
    /// The number of overlapping units for each split
    split_overlap: usize,
    
    /// The minimum number of units per split
    split_threshold: usize,
    
    /// Custom splitting function when split_by is "function"
    #[allow(clippy::type_complexity)]
    splitting_function: Option<SplittingFunction>,
}

impl DocumentSplitter {
    /// Create a new DocumentSplitter component
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        split_by: SplitByType,
        split_length: usize,
        split_overlap: usize,
        split_threshold: usize,
        splitting_function: Option<SplittingFunction>,
    ) -> Result<Self> {
        // Validate parameters
        if split_by == SplitByType::Function && splitting_function.is_none() {
            return Err(anyhow!("When 'split_by' is set to 'function', a valid 'splitting_function' must be provided."));
        }

        if split_length == 0 {
            return Err(anyhow!("split_length must be greater than 0."));
        }

        if split_overlap > split_length {
            return Err(anyhow!("split_overlap must be less than or equal to split_length."));
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
        init_parameters.insert("split_by".to_string(), serde_json::to_value(&split_by)?);
        init_parameters.insert("split_length".to_string(), Value::from(split_length));
        init_parameters.insert("split_overlap".to_string(), Value::from(split_overlap));
        init_parameters.insert("split_threshold".to_string(), Value::from(split_threshold));
        
        // For function splitting, we can't easily serialize the function, so we just note that it exists
        if splitting_function.is_some() {
            init_parameters.insert("has_splitting_function".to_string(), Value::from(true));
        }
        
        Ok(Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            split_by,
            split_length,
            split_overlap,
            split_threshold,
            splitting_function,
        })
    }
    
    /// Split a document into smaller parts
    fn split_document(&self, doc: &Document) -> Result<Vec<Document>> {
        if doc.content.is_none() {
            return Err(anyhow!("DocumentSplitter only works with text documents but content for document ID {} is None.", doc.id));
        }
        
        let content = doc.content.as_ref().unwrap();
        
        if content.is_empty() {
            tracing::warn!("Document ID {} has an empty content. Skipping this document.", doc.id);
            return Ok(Vec::new());
        }
        
        match self.split_by {
            SplitByType::Function => self.split_by_function(doc),
            _ => self.split_by_character(doc),
        }
    }
    
    /// Split a document using the specified splitting function
    fn split_by_function(&self, doc: &Document) -> Result<Vec<Document>> {
        if let Some(splitting_function) = &self.splitting_function {
            let content = doc.content.as_ref().unwrap();
            let splits = splitting_function(content);
            
            let mut documents = Vec::new();
            for (i, split) in splits.into_iter().enumerate() {
                let mut meta = doc.meta.clone();
                meta.insert("source_id".to_string(), Value::String(doc.id.clone()));
                meta.insert("split_id".to_string(), Value::Number(i.into()));
                
                let split_doc = Document::with_params(
                    None, 
                    Some(split),
                    doc.blob.clone(),
                    Some(meta),
                    doc.score,
                    doc.embedding.clone(),
                    doc.sparse_embedding.clone(),
                );
                
                documents.push(split_doc);
            }
            
            Ok(documents)
        } else {
            Err(anyhow!("Splitting function is not provided"))
        }
    }
    
    /// Split a document by a specified character
    fn split_by_character(&self, doc: &Document) -> Result<Vec<Document>> {
        let content = doc.content.as_ref().unwrap();
        let split_char = self.split_by.get_split_char()
            .ok_or_else(|| anyhow!("Invalid split_by type"))?;
        
        // Split the content
        let mut units: Vec<String> = content.split(split_char)
            .map(|s| s.to_string())
            .collect();
        
        // Add back the delimiter to all units except the last one
        for i in 0..units.len().saturating_sub(1) {
            units[i].push_str(split_char);
        }
        
        // Concatenate the units into chunks
        let (text_splits, splits_pages, splits_start_idxs) = 
            self.concatenate_units(&units, self.split_length, self.split_overlap, self.split_threshold);
        
        // Create documents from the splits
        let mut meta = doc.meta.clone();
        meta.insert("source_id".to_string(), Value::String(doc.id.clone()));
        
        let documents = self.create_docs_from_splits(&text_splits, &splits_pages, &splits_start_idxs, meta)?;
        
        Ok(documents)
    }
    
    /// Concatenate units into chunks of specified length with overlap
    fn concatenate_units(
        &self,
        elements: &[String],
        split_length: usize,
        split_overlap: usize,
        split_threshold: usize,
    ) -> (Vec<String>, Vec<usize>, Vec<usize>) {
        let mut text_splits: Vec<String> = Vec::new();
        let mut splits_pages = Vec::new();
        let mut splits_start_idxs = Vec::new();
        
        let mut cur_start_idx = 0;
        let mut cur_page = 1;
        
        let step = if split_length > split_overlap {
            split_length - split_overlap
        } else {
            1
        };
        
        // Create the windowed segments
        let mut i = 0;
        while i < elements.len() {
            let end = std::cmp::min(i + split_length, elements.len());
            let segment = &elements[i..end];
            
            // Calculate the text for this segment
            let txt = segment.join("");
            
            // Check if length of current units is below split_threshold and we have previous splits
            if segment.len() < split_threshold && !text_splits.is_empty() {
                // Concatenate with the last split
                text_splits.last_mut().unwrap().push_str(&txt);
            } else if !txt.is_empty() {
                // Add a new split
                text_splits.push(txt);
                splits_pages.push(cur_page);
                splits_start_idxs.push(cur_start_idx);
            }
            
            // Move to the next step
            let processed_units = if i + step <= elements.len() {
                &elements[i..i+step]
            } else {
                &elements[i..elements.len()]
            };
            
            // Calculate the new start index
            cur_start_idx += processed_units.iter().map(|s| s.len()).sum::<usize>();
            
            // Calculate the new page number
            if self.split_by == SplitByType::Page {
                cur_page += processed_units.len();
            } else {
                cur_page += processed_units.iter()
                    .map(|s| s.matches('\u{000C}').count())
                    .sum::<usize>();
            }
            
            i += step;
        }
        
        (text_splits, splits_pages, splits_start_idxs)
    }
    
    /// Create Document objects from splits
    fn create_docs_from_splits(
        &self,
        text_splits: &[String],
        splits_pages: &[usize],
        splits_start_idxs: &[usize],
        meta: HashMap<String, Value>,
    ) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        
        for i in 0..text_splits.len() {
            let mut copied_meta = meta.clone();
            copied_meta.insert("page_number".to_string(), Value::Number(splits_pages[i].into()));
            copied_meta.insert("split_id".to_string(), Value::Number(i.into()));
            copied_meta.insert("split_idx_start".to_string(), Value::Number(splits_start_idxs[i].into()));
            
            let doc = Document::with_params(
                None,
                Some(text_splits[i].clone()),
                None, // No blob
                Some(copied_meta),
                None, // No score
                None, // No embedding
                None, // No sparse embedding
            );
            
            documents.push(doc);
            
            // Add split overlap information if needed
            if self.split_overlap > 0 && i > 0 {
                // Process each document one at a time to avoid borrowing issues
                let previous_idx = i - 1;
                let previous_doc_start_idx = splits_start_idxs[previous_idx];
                let current_doc_start_idx = splits_start_idxs[i];
                
                // Get the content from the documents
                let previous_content = documents[previous_idx].content.clone()
                    .ok_or_else(|| anyhow::anyhow!("Previous document has no content"))?;
                let current_content = documents[i].content.clone()
                    .ok_or_else(|| anyhow::anyhow!("Current document has no content"))?;
                
                // Calculate overlap information
                let range_start = current_doc_start_idx.saturating_sub(previous_doc_start_idx);
                let range_end = previous_content.len();
                
                if range_start < range_end {
                    // Extract the overlapping string
                    let overlapping_str = &previous_content[range_start..range_end];
                    
                    if current_content.starts_with(overlapping_str) {
                        // Add overlap info to current document
                        let prev_doc_id = documents[previous_idx].id.clone();
                        let mut overlap_info = serde_json::Map::new();
                        overlap_info.insert("doc_id".to_string(), Value::String(prev_doc_id.clone()));
                        
                        let mut range = serde_json::Map::new();
                        range.insert("start".to_string(), Value::Number(range_start.into()));
                        range.insert("end".to_string(), Value::Number(range_end.into()));
                        
                        overlap_info.insert("range".to_string(), Value::Object(range));
                        
                        // Update current document's metadata
                        let mut current_split_overlap = documents[i].meta.get("_split_overlap")
                            .and_then(|v| v.as_array().cloned())
                            .unwrap_or_default();
                        current_split_overlap.push(Value::Object(overlap_info));
                        documents[i].meta.insert("_split_overlap".to_string(), Value::Array(current_split_overlap));
                        
                        // Update previous document's metadata
                        let mut previous_split_overlap = documents[previous_idx].meta.get("_split_overlap")
                            .and_then(|v| v.as_array().cloned())
                            .unwrap_or_default();
                        
                        let mut reverse_overlap_info = serde_json::Map::new();
                        reverse_overlap_info.insert("doc_id".to_string(), Value::String(documents[i].id.clone()));
                        
                        let mut reverse_range = serde_json::Map::new();
                        reverse_range.insert("start".to_string(), Value::Number(0.into()));
                        reverse_range.insert("end".to_string(), Value::Number(overlapping_str.len().into()));
                        
                        reverse_overlap_info.insert("range".to_string(), Value::Object(reverse_range));
                        previous_split_overlap.push(Value::Object(reverse_overlap_info));
                        
                        documents[previous_idx].meta.insert("_split_overlap".to_string(), Value::Array(previous_split_overlap));
                    }
                }
            }
        }
        
        Ok(documents)
    }
    
    /// Add split overlap information to documents
    fn add_split_overlap_information(
        &self,
        current_doc: &mut Document,
        current_doc_start_idx: usize,
        previous_doc: &mut Document,
        previous_doc_start_idx: usize,
    ) -> Result<()> {
        let previous_content = previous_doc.content.as_ref()
            .ok_or_else(|| anyhow!("Previous document has no content"))?;
        
        let current_content = current_doc.content.as_ref()
            .ok_or_else(|| anyhow!("Current document has no content"))?;
        
        // Calculate the overlapping range
        let range_start = current_doc_start_idx.saturating_sub(previous_doc_start_idx);
        let range_end = previous_content.len();
        
        if range_start < range_end {
            // Extract the overlapping string
            let overlapping_str = &previous_content[range_start..range_end];
            
            if current_content.starts_with(overlapping_str) {
                // Add split overlap information to current document
                let mut split_overlap = current_doc.meta.get("_split_overlap")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                
                let mut overlap_info = serde_json::Map::new();
                overlap_info.insert("doc_id".to_string(), Value::String(previous_doc.id.clone()));
                
                let mut range = serde_json::Map::new();
                range.insert("start".to_string(), Value::Number(range_start.into()));
                range.insert("end".to_string(), Value::Number(range_end.into()));
                
                overlap_info.insert("range".to_string(), Value::Object(range));
                split_overlap.push(Value::Object(overlap_info));
                
                current_doc.meta.insert("_split_overlap".to_string(), Value::Array(split_overlap));
                
                // Add split overlap information to previous document
                let mut prev_split_overlap = previous_doc.meta.get("_split_overlap")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                
                let mut prev_overlap_info = serde_json::Map::new();
                prev_overlap_info.insert("doc_id".to_string(), Value::String(current_doc.id.clone()));
                
                let mut prev_range = serde_json::Map::new();
                prev_range.insert("start".to_string(), Value::Number(0.into()));
                prev_range.insert("end".to_string(), Value::Number((range_end - range_start).into()));
                
                prev_overlap_info.insert("range".to_string(), Value::Object(prev_range));
                prev_split_overlap.push(Value::Object(prev_overlap_info));
                
                previous_doc.meta.insert("_split_overlap".to_string(), Value::Array(prev_split_overlap));
            }
        }
        
        Ok(())
    }
    
    // Note: The add_split_overlap_information method is kept for reference, but we now use
    // the inline implementation in create_docs_from_splits to avoid mutable borrowing conflicts
}

impl Component for DocumentSplitter {
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
        let mut split_docs = Vec::new();
        
        for doc in &documents {
            match self.split_document(doc) {
                Ok(docs) => split_docs.extend(docs),
                Err(e) => {
                    // Log the error but continue with other documents
                    tracing::error!("Error splitting document {}: {}", doc.id, e);
                }
            }
        }
        
        // Return the split documents
        Ok(HashMap::from([
            ("documents".to_string(), serde_json::to_value(split_docs)?)
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