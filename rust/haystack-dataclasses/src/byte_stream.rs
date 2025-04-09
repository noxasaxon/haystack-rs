use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// Represents a binary object in the Haystack API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteStream {
    /// The binary data
    pub data: Vec<u8>,
    /// Additional metadata associated with the binary data
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,
    /// The MIME type of the data, if known
    pub mime_type: Option<String>,
}

impl ByteStream {
    /// Create a new ByteStream
    pub fn new(data: Vec<u8>, mime_type: Option<String>, meta: Option<HashMap<String, serde_json::Value>>) -> Self {
        Self {
            data,
            mime_type,
            meta: meta.unwrap_or_default(),
        }
    }

    /// Write the ByteStream to a file
    ///
    /// Note: the metadata will be lost
    pub fn to_file<P: AsRef<Path>>(&self, destination_path: P) -> Result<()> {
        fs::write(destination_path, &self.data)
            .context("Failed to write ByteStream to file")
    }

    /// Create a ByteStream from the contents read from a file
    pub fn from_file_path<P: AsRef<Path>>(
        filepath: P,
        mime_type: Option<String>,
        meta: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Self> {
        let data = fs::read(filepath)
            .context("Failed to read file")?;

        Ok(Self {
            data,
            mime_type,
            meta: meta.unwrap_or_default(),
        })
    }

    /// Create a ByteStream by encoding a string
    pub fn from_string(
        text: &str,
        _encoding: Option<&str>,
        mime_type: Option<String>,
        meta: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        // In Rust, we don't need to specify encoding for UTF-8 (the default)
        // For other encodings, we'd need an external crate
        let data = text.as_bytes().to_vec();

        Self {
            data,
            mime_type,
            meta: meta.unwrap_or_default(),
        }
    }

    /// Convert the ByteStream to a string
    ///
    /// Metadata will not be included
    pub fn to_string(&self, _encoding: Option<&str>) -> Result<String> {
        // In Rust, we primarily work with UTF-8
        let string = String::from_utf8(self.data.clone())
            .context("Failed to decode ByteStream data as UTF-8")?;
        
        Ok(string)
    }
}

impl std::fmt::Display for ByteStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let truncated_data = if self.data.len() > 100 {
            format!("{:?}...", &self.data[..100])
        } else {
            format!("{:?}", self.data)
        };
        
        write!(f, "ByteStream(data={}, meta={:?}, mime_type={:?})",
            truncated_data, self.meta, self.mime_type)
    }
}