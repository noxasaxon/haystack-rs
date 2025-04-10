use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// The StreamingChunk class encapsulates a segment of streamed content along with associated metadata.
///
/// This structure facilitates the handling and processing of streamed data in a systematic manner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    /// The content of the message chunk as a string
    pub content: String,
    
    /// A dictionary containing metadata related to the message chunk
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}

impl StreamingChunk {
    /// Create a new StreamingChunk
    pub fn new(content: String, meta: Option<HashMap<String, Value>>) -> Self {
        Self {
            content,
            meta: meta.unwrap_or_default(),
        }
    }
}

/// Type alias for a synchronous streaming callback function
pub type SyncStreamingCallback = Arc<dyn Fn(&StreamingChunk) + Send + Sync>;

/// Type alias for an asynchronous streaming callback function
pub type AsyncStreamingCallback = Arc<dyn Fn(&StreamingChunk) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Enum to represent either a synchronous or asynchronous streaming callback
#[derive(Clone)]
pub enum StreamingCallback {
    /// Synchronous callback
    Sync(SyncStreamingCallback),
    
    /// Asynchronous callback
    Async(AsyncStreamingCallback),
}

/// Checks if a callback is async compatible
pub fn is_callback_async_compatible(callback: &StreamingCallback) -> bool {
    matches!(callback, StreamingCallback::Async(_))
}

/// Picks the correct streaming callback given an optional initial and runtime callback
///
/// The runtime callback takes precedence over the initial callback.
pub fn select_streaming_callback(
    init_callback: Option<StreamingCallback>,
    runtime_callback: Option<StreamingCallback>,
    requires_async: bool,
) -> Result<Option<StreamingCallback>> {
    if let Some(callback) = &init_callback {
        if requires_async && !is_callback_async_compatible(callback) {
            return Err(anyhow!("The init callback must be async compatible."));
        }
        if !requires_async && is_callback_async_compatible(callback) {
            return Err(anyhow!("The init callback cannot be a coroutine."));
        }
    }
    
    if let Some(callback) = &runtime_callback {
        if requires_async && !is_callback_async_compatible(callback) {
            return Err(anyhow!("The runtime callback must be async compatible."));
        }
        if !requires_async && is_callback_async_compatible(callback) {
            return Err(anyhow!("The runtime callback cannot be a coroutine."));
        }
    }
    
    Ok(runtime_callback.or(init_callback))
}

/// Convenience function to create a synchronous streaming callback
pub fn sync_streaming_callback<F>(f: F) -> StreamingCallback 
where 
    F: Fn(&StreamingChunk) + Send + Sync + 'static,
{
    StreamingCallback::Sync(Arc::new(f))
}

/// Convenience function to create an asynchronous streaming callback
pub fn async_streaming_callback<F, Fut>(f: F) -> StreamingCallback 
where 
    F: Fn(&StreamingChunk) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    StreamingCallback::Async(Arc::new(move |chunk| {
        Box::pin(f(chunk))
    }))
}