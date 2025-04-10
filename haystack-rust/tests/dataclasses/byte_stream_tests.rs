use std::collections::HashMap;
use std::fs;
use serde_json::Value;
use haystack_dataclasses::ByteStream;

#[test]
fn test_byte_stream_creation() {
    let data = vec![1, 2, 3, 4, 5];
    let mime_type = Some("application/octet-stream".to_string());
    let meta = Some(HashMap::from([
        ("source".to_string(), Value::String("test".to_string())),
        ("size".to_string(), Value::Number(5.into())),
    ]));
    
    let byte_stream = ByteStream::new(
        data.clone(),
        mime_type.clone(),
        meta.clone(),
    );
    
    assert_eq!(byte_stream.data, data);
    assert_eq!(byte_stream.mime_type, mime_type);
    assert_eq!(byte_stream.meta.len(), 2);
    assert_eq!(byte_stream.meta["source"], Value::String("test".to_string()));
    assert_eq!(byte_stream.meta["size"], Value::Number(5.into()));
}

#[test]
fn test_byte_stream_file_operations() {
    let data = vec![1, 2, 3, 4, 5];
    let byte_stream = ByteStream::new(
        data.clone(),
        Some("application/octet-stream".to_string()),
        None,
    );
    
    // Create a temporary file path
    let temp_file = std::env::temp_dir().join("haystack_test_byte_stream.bin");
    let temp_path = temp_file.to_str().unwrap();
    
    // Write to file
    byte_stream.to_file(temp_path).unwrap();
    
    // Read from file
    let loaded = ByteStream::from_file_path(
        temp_path,
        Some("application/octet-stream".to_string()),
        None,
    ).unwrap();
    
    assert_eq!(loaded.data, data);
    assert_eq!(loaded.mime_type, Some("application/octet-stream".to_string()));
    
    // Clean up
    fs::remove_file(temp_path).unwrap();
}

#[test]
fn test_byte_stream_from_string() {
    let text = "Hello, world!";
    let byte_stream = ByteStream::from_string(
        text,
        None,
        Some("text/plain".to_string()),
        None,
    );
    
    assert_eq!(byte_stream.data, text.as_bytes().to_vec());
    assert_eq!(byte_stream.mime_type, Some("text/plain".to_string()));
    
    // Convert back to string
    let result_string = byte_stream.to_string(None).unwrap();
    assert_eq!(result_string, text);
}

#[test]
fn test_byte_stream_to_string_error() {
    // Create invalid UTF-8 data
    let invalid_utf8 = vec![0xFF, 0xFF];
    let byte_stream = ByteStream::new(
        invalid_utf8,
        None,
        None,
    );
    
    // Should fail to convert to string
    let result = byte_stream.to_string(None);
    assert!(result.is_err());
}

#[test]
fn test_byte_stream_with_metadata() {
    let data = "Hello, world!".as_bytes().to_vec();
    let mime_type = Some("text/plain".to_string());
    let meta = Some(HashMap::from([
        ("filename".to_string(), Value::String("hello.txt".to_string())),
        ("created".to_string(), Value::String("2023-01-01".to_string())),
    ]));
    
    let byte_stream = ByteStream::new(
        data.clone(),
        mime_type.clone(),
        meta.clone(),
    );
    
    assert_eq!(byte_stream.data, data);
    assert_eq!(byte_stream.mime_type, mime_type);
    assert_eq!(byte_stream.meta.len(), 2);
    assert_eq!(byte_stream.meta["filename"], Value::String("hello.txt".to_string()));
    assert_eq!(byte_stream.meta["created"], Value::String("2023-01-01".to_string()));
}

#[test]
fn test_byte_stream_display() {
    // Small data - should show as is
    let small_data = vec![1, 2, 3, 4, 5];
    let byte_stream = ByteStream::new(
        small_data.clone(),
        Some("application/octet-stream".to_string()),
        None,
    );
    
    let display = format!("{}", byte_stream);
    assert!(display.contains("[1, 2, 3, 4, 5]"));
    assert!(display.contains("application/octet-stream"));
    
    // Large data - should be truncated
    let large_data = vec![0; 200];
    let byte_stream = ByteStream::new(
        large_data.clone(),
        Some("application/octet-stream".to_string()),
        None,
    );
    
    let display = format!("{}", byte_stream);
    assert!(display.contains("..."));
}