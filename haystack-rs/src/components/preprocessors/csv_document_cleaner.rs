use std::collections::HashMap;

use anyhow::Result;
use csv::{Reader, Writer};
use haystack_dataclasses::document::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::component_system::{Component, ComponentBase, InputSocket, OutputSocket};

/// A component that cleans CSV documents by removing empty rows and columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSVDocumentCleaner {
    #[serde(skip)]
    base: ComponentBase,
    
    /// Number of top rows to keep regardless of content
    #[serde(default)]
    ignore_rows: usize,
    
    /// Number of left columns to keep regardless of content
    #[serde(default)]
    ignore_columns: usize,
    
    /// Whether to remove empty rows
    #[serde(default = "default_true")]
    remove_empty_rows: bool,
    
    /// Whether to remove empty columns
    #[serde(default = "default_true")]
    remove_empty_columns: bool,
    
    /// Whether to keep the original document ID
    #[serde(default)]
    keep_id: bool,
}

fn default_true() -> bool {
    true
}

impl CSVDocumentCleaner {
    /// Create a new CSVDocumentCleaner component.
    ///
    /// # Arguments
    ///
    /// * `ignore_rows` - Number of top rows to keep regardless of content
    /// * `ignore_columns` - Number of left columns to keep regardless of content
    /// * `remove_empty_rows` - Whether to remove empty rows
    /// * `remove_empty_columns` - Whether to remove empty columns
    /// * `keep_id` - Whether to keep the original document ID
    ///
    /// # Returns
    ///
    /// A new CSVDocumentCleaner instance.
    pub fn new(
        ignore_rows: usize,
        ignore_columns: usize,
        remove_empty_rows: bool,
        remove_empty_columns: bool,
        keep_id: bool,
    ) -> Result<Self> {
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
        init_parameters.insert("ignore_rows".to_string(), Value::from(ignore_rows));
        init_parameters.insert("ignore_columns".to_string(), Value::from(ignore_columns));
        init_parameters.insert("remove_empty_rows".to_string(), Value::from(remove_empty_rows));
        init_parameters.insert("remove_empty_columns".to_string(), Value::from(remove_empty_columns));
        init_parameters.insert("keep_id".to_string(), Value::from(keep_id));
        
        let instance = Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            ignore_rows,
            ignore_columns,
            remove_empty_rows,
            remove_empty_columns,
            keep_id,
        };
        
        Ok(instance)
    }
    
    /// Clean a CSV document by removing empty rows and columns.
    fn clean_document(&self, document: &Document) -> Result<Document> {
        // If document doesn't have content, return it unchanged
        if document.content.is_none() {
            return Ok(document.clone());
        }
        
        let content = document.content.as_ref().unwrap();
        
        // Parse CSV content
        let mut reader = Reader::from_reader(content.as_bytes());
        
        // Read all records into memory
        let records_result: Result<Vec<csv::StringRecord>, _> = reader.records().collect();
        let records = match records_result {
            Ok(records) => records,
            Err(_) => {
                // If we can't parse as CSV, return the document unchanged
                return Ok(document.clone());
            }
        };
        
        // If there are no records, return the document unchanged
        if records.is_empty() {
            return Ok(document.clone());
        }
        
        // Get headers
        let headers = reader.headers().ok().map(|h| h.clone());
        
        // Process the CSV
        let (headers, records) = self.clean_csv(headers, records)?;
        
        // Convert back to CSV string
        let mut output = Vec::new();
        {
            let mut writer = Writer::from_writer(&mut output);
            
            // Write headers if present
            if let Some(h) = &headers {
                writer.write_record(h)?;
            }
            
            // Write records
            for record in records {
                writer.write_record(&record)?;
            }
            writer.flush()?;
        } // Writer is dropped here, releasing the borrow on output
        
        // Create new document
        let id = if self.keep_id {
            document.id.clone()
        } else {
            Uuid::new_v4().to_string()
        };
        
        let mut cleaned_document = Document::with_params(
            Some(id),
            Some(String::from_utf8(output)?),
            document.blob.clone(),
            Some(document.meta.clone()),
            None,
            None,
            None,
        );
        cleaned_document.score = document.score;
        
        Ok(cleaned_document)
    }
    
    /// Clean CSV data by removing empty rows and columns
    fn clean_csv(
        &self,
        headers: Option<csv::StringRecord>,
        records: Vec<csv::StringRecord>
    ) -> Result<(Option<csv::StringRecord>, Vec<csv::StringRecord>)> {
        // If nothing to do, return as is
        if !self.remove_empty_rows && !self.remove_empty_columns {
            return Ok((headers, records));
        }
        
        // Check if we have enough rows/columns to process
        let header_len = headers.as_ref().map_or(0, |h| h.len());
        let max_record_len = records.iter().map(|r| r.len()).max().unwrap_or(0);
        let max_columns = std::cmp::max(header_len, max_record_len);
        
        // If we don't have enough rows or columns to process, return unchanged
        if records.len() <= self.ignore_rows || max_columns <= self.ignore_columns {
            return Ok((headers, records));
        }
        
        // Initialize data structures
        let mut ignored_rows = Vec::new();
        let mut remaining_rows = Vec::new();
        
        // Split records into ignored and remaining rows
        for (i, record) in records.into_iter().enumerate() {
            if i < self.ignore_rows {
                ignored_rows.push(record);
            } else {
                remaining_rows.push(record);
            }
        }
        
        // Find empty rows to remove
        let mut cleaned_rows = Vec::new();
        if self.remove_empty_rows {
            for record in remaining_rows {
                let is_empty = record.iter().all(|field| field.trim().is_empty());
                if !is_empty {
                    cleaned_rows.push(record);
                }
            }
        } else {
            cleaned_rows = remaining_rows;
        }
        
        // Identify empty columns
        let mut empty_columns = Vec::new();
        if self.remove_empty_columns {
            let column_count = cleaned_rows.iter().map(|r| r.len()).max().unwrap_or(0);
            
            for col_idx in self.ignore_columns..column_count {
                let is_empty = cleaned_rows.iter().all(|row| {
                    col_idx >= row.len() || row.get(col_idx).unwrap_or("").trim().is_empty()
                });
                
                empty_columns.push(is_empty);
            }
        }
        
        // Process headers to remove empty columns
        let processed_headers = if let Some(h) = headers {
            let mut new_headers = csv::StringRecord::new();
            
            // Always keep ignored columns
            for i in 0..std::cmp::min(self.ignore_columns, h.len()) {
                new_headers.push_field(h.get(i).unwrap_or(""));
            }
            
            // Process remaining columns
            for i in self.ignore_columns..h.len() {
                let col_idx = i - self.ignore_columns;
                if !self.remove_empty_columns || col_idx >= empty_columns.len() || !empty_columns[col_idx] {
                    new_headers.push_field(h.get(i).unwrap_or(""));
                }
            }
            
            Some(new_headers)
        } else {
            None
        };
        
        // Process rows to remove empty columns
        let mut processed_rows = Vec::new();
        
        // Add ignored rows first
        for row in ignored_rows {
            processed_rows.push(row);
        }
        
        // Process remaining rows
        for row in cleaned_rows {
            let mut new_row = csv::StringRecord::new();
            
            // Always keep ignored columns
            for i in 0..std::cmp::min(self.ignore_columns, row.len()) {
                new_row.push_field(row.get(i).unwrap_or(""));
            }
            
            // Process remaining columns
            for i in self.ignore_columns..row.len() {
                let col_idx = i - self.ignore_columns;
                if !self.remove_empty_columns || col_idx >= empty_columns.len() || !empty_columns[col_idx] {
                    new_row.push_field(row.get(i).unwrap_or(""));
                }
            }
            
            processed_rows.push(new_row);
        }
        
        Ok((processed_headers, processed_rows))
    }
}

impl Component for CSVDocumentCleaner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let documents = inputs.get("documents")
            .ok_or_else(|| anyhow::anyhow!("Missing required input 'documents'"))?;
        
        let documents = serde_json::from_value::<Vec<Document>>(documents.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse 'documents' input: {}", e))?;
        
        // If there are no documents, return empty result
        if documents.is_empty() {
            let mut outputs = HashMap::new();
            outputs.insert("documents".to_string(), serde_json::to_value(Vec::<Document>::new())?);
            return Ok(outputs);
        }
        
        // Process each document
        let cleaned_documents: Result<Vec<Document>> = documents.iter()
            .map(|doc| self.clean_document(doc))
            .collect();
        
        let mut outputs = HashMap::new();
        outputs.insert("documents".to_string(), serde_json::to_value(cleaned_documents?)?);
        
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
    
    fn create_csv_document(content: &str) -> Document {
        Document::with_params(
            Some(Uuid::new_v4().to_string()),
            Some(content.to_string()),
            None,
            Some(HashMap::new()),
            None,
            None,
            None,
        )
    }
    
    #[test]
    fn test_csv_document_cleaner_empty_input() {
        let cleaner = CSVDocumentCleaner::new(0, 0, true, true, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(Vec::<Document>::new()).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents.len(), 0);
    }
    
    #[test]
    fn test_csv_document_cleaner_remove_empty_rows() {
        let csv_content = "a,b,c\n1,2,3\n,,\n4,5,6";
        let document = create_csv_document(csv_content);
        
        let cleaner = CSVDocumentCleaner::new(0, 0, true, false, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents.len(), 1);
        
        // The cleaned document should not have the empty row
        let cleaned_content = documents[0].content.as_ref().unwrap();
        let mut reader = Reader::from_reader(cleaned_content.as_bytes());
        let records: Vec<_> = reader.records().map(|r| r.unwrap()).collect();
        
        assert_eq!(records.len(), 2); // Header + 2 data rows (the empty row is removed)
    }
    
    #[test]
    fn test_csv_document_cleaner_remove_empty_columns() {
        let csv_content = "a,b,c,d\n1,,3,4\n5,,7,8\n9,,11,12";
        let document = create_csv_document(csv_content);
        
        let cleaner = CSVDocumentCleaner::new(0, 0, false, true, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents.len(), 1);
        
        // The cleaned document should not have the empty column (b)
        let cleaned_content = documents[0].content.as_ref().unwrap();
        let mut reader = Reader::from_reader(cleaned_content.as_bytes());
        let headers = reader.headers().unwrap();
        
        assert_eq!(headers.len(), 3); // a, c, d (b is removed)
        assert_eq!(headers.get(0), Some("a"));
        assert_eq!(headers.get(1), Some("c"));
        assert_eq!(headers.get(2), Some("d"));
    }
    
    #[test]
    fn test_csv_document_cleaner_ignore_rows() {
        let csv_content = "header1,header2\n,\n1,2\n3,4";
        let document = create_csv_document(csv_content);
        
        let cleaner = CSVDocumentCleaner::new(2, 0, true, false, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        // The cleaned document should preserve the empty row since it's in the ignored rows
        let cleaned_content = documents[0].content.as_ref().unwrap();
        let mut reader = Reader::from_reader(cleaned_content.as_bytes());
        let records: Vec<_> = reader.records().map(|r| r.unwrap()).collect();
        
        assert_eq!(records.len(), 3); // All rows preserved (empty row is in ignored rows)
    }
    
    #[test]
    fn test_csv_document_cleaner_ignore_columns() {
        let csv_content = "a,b,c\n,2,3\n,5,6";
        let document = create_csv_document(csv_content);
        
        let cleaner = CSVDocumentCleaner::new(0, 1, false, true, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        // The cleaned document should preserve the first column even though it's empty
        let cleaned_content = documents[0].content.as_ref().unwrap();
        let mut reader = Reader::from_reader(cleaned_content.as_bytes());
        let headers = reader.headers().unwrap();
        
        assert_eq!(headers.len(), 3); // All columns preserved (first is in ignored columns)
    }
    
    #[test]
    fn test_csv_document_cleaner_keep_id() {
        let csv_content = "a,b,c\n1,2,3";
        let document = create_csv_document(csv_content);
        let original_id = document.id.clone();
        
        let cleaner = CSVDocumentCleaner::new(0, 0, true, true, true).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_eq!(documents[0].id, original_id);
    }
    
    #[test]
    fn test_csv_document_cleaner_generate_new_id() {
        let csv_content = "a,b,c\n1,2,3";
        let document = create_csv_document(csv_content);
        let original_id = document.id.clone();
        
        let cleaner = CSVDocumentCleaner::new(0, 0, true, true, false).unwrap();
        
        let inputs = HashMap::from([
            ("documents".to_string(), serde_json::to_value(vec![document]).unwrap()),
        ]);
        
        let outputs = cleaner.run(inputs).unwrap();
        let documents = serde_json::from_value::<Vec<Document>>(outputs.get("documents").unwrap().clone()).unwrap();
        
        assert_ne!(documents[0].id, original_id);
    }
}