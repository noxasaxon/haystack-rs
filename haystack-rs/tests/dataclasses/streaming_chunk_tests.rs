use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::json;
use tokio::sync::Mutex as AsyncMutex;
use haystack_dataclasses::{StreamingChunk, sync_streaming_callback, async_streaming_callback, StreamingCallback};
use haystack_dataclasses::streaming_chunk::{select_streaming_callback, is_callback_async_compatible};

#[test]
fn test_streaming_chunk_creation() {
    let chunk = StreamingChunk::new(
        "This is a chunk of content".to_string(),
        None,
    );
    
    assert_eq!(chunk.content, "This is a chunk of content");
    assert!(chunk.meta.is_empty());
    
    // With metadata
    let meta = Some(HashMap::from([
        ("source".to_string(), json!("test")),
        ("timestamp".to_string(), json!(123456789)),
    ]));
    
    let chunk = StreamingChunk::new(
        "This is a chunk with metadata".to_string(),
        meta,
    );
    
    assert_eq!(chunk.content, "This is a chunk with metadata");
    assert_eq!(chunk.meta.len(), 2);
    assert_eq!(chunk.meta["source"], json!("test"));
    assert_eq!(chunk.meta["timestamp"], json!(123456789));
}

#[test]
fn test_streaming_chunk_serialization() {
    let meta = HashMap::from([
        ("source".to_string(), json!("test")),
        ("timestamp".to_string(), json!(123456789)),
    ]);
    
    let chunk = StreamingChunk::new(
        "This is a chunk of content".to_string(),
        Some(meta),
    );
    
    // Serialize
    let serialized = serde_json::to_string(&chunk).unwrap();
    
    // Deserialize
    let deserialized: StreamingChunk = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.content, "This is a chunk of content");
    assert_eq!(deserialized.meta.len(), 2);
    assert_eq!(deserialized.meta["source"], json!("test"));
    assert_eq!(deserialized.meta["timestamp"], json!(123456789));
}

#[test]
fn test_sync_streaming_callback() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);
    
    // Create a synchronous callback
    let callback = sync_streaming_callback(move |chunk| {
        let mut received = received_clone.lock().unwrap();
        received.push(chunk.content.clone());
    });
    
    // Send a chunk
    let chunk = StreamingChunk::new("Chunk 1".to_string(), None);
    match &callback {
        StreamingCallback::Sync(cb) => cb(&chunk),
        _ => panic!("Expected Sync callback"),
    }
    
    // Send another chunk
    let chunk = StreamingChunk::new("Chunk 2".to_string(), None);
    match &callback {
        StreamingCallback::Sync(cb) => cb(&chunk),
        _ => panic!("Expected Sync callback"),
    }
    
    // Verify received chunks
    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    assert_eq!(received[0], "Chunk 1");
    assert_eq!(received[1], "Chunk 2");
}

#[tokio::test]
async fn test_async_streaming_callback() {
    let received = Arc::new(AsyncMutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);
    
    // Create an asynchronous callback
    let callback = async_streaming_callback(move |chunk: &StreamingChunk| {
        let received_clone = Arc::clone(&received_clone);
        let content = chunk.content.clone();
        async move {
            let mut received = received_clone.lock().await;
            received.push(content);
        }
    });
    
    // Send a chunk
    let chunk = StreamingChunk::new("Async Chunk 1".to_string(), None);
    if let StreamingCallback::Async(cb) = &callback {
        let future = cb(&chunk);
        future.await;
    } else {
        panic!("Expected Async callback");
    }
    
    // Send another chunk
    let chunk = StreamingChunk::new("Async Chunk 2".to_string(), None);
    if let StreamingCallback::Async(cb) = &callback {
        let future = cb(&chunk);
        future.await;
    } else {
        panic!("Expected Async callback");
    }
    
    // Verify received chunks
    let received = received.lock().await;
    assert_eq!(received.len(), 2);
    assert_eq!(received[0], "Async Chunk 1");
    assert_eq!(received[1], "Async Chunk 2");
}

#[test]
fn test_is_callback_async_compatible() {
    let sync_cb = sync_streaming_callback(|_| {});
    let async_cb = async_streaming_callback(|_| async {});
    
    assert!(!is_callback_async_compatible(&sync_cb));
    assert!(is_callback_async_compatible(&async_cb));
}

#[test]
fn test_select_streaming_callback() {
    let sync_cb = sync_streaming_callback(|_| {});
    let async_cb = async_streaming_callback(|_| async {});
    
    // Test selecting with sync requirements
    assert!(select_streaming_callback(Some(sync_cb.clone()), None, false).is_ok());
    assert!(select_streaming_callback(None, Some(sync_cb.clone()), false).is_ok());
    assert!(select_streaming_callback(Some(async_cb.clone()), None, false).is_err());
    assert!(select_streaming_callback(None, Some(async_cb.clone()), false).is_err());
    
    // Test selecting with async requirements
    assert!(select_streaming_callback(Some(async_cb.clone()), None, true).is_ok());
    assert!(select_streaming_callback(None, Some(async_cb.clone()), true).is_ok());
    assert!(select_streaming_callback(Some(sync_cb.clone()), None, true).is_err());
    assert!(select_streaming_callback(None, Some(sync_cb.clone()), true).is_err());
    
    // Test runtime callback takes precedence over init callback
    let selected = select_streaming_callback(
        Some(sync_cb.clone()),
        Some(sync_cb.clone()),
        false
    ).unwrap();
    
    assert!(selected.is_some());
}