/*!
 * Haystack Dataclasses
 *
 * Core data structures used throughout Haystack.
 */

pub mod answer;
pub mod byte_stream;
pub mod chat_message;
pub mod document;
pub mod sparse_embedding;
pub mod state;
pub mod streaming_chunk;

// Re-export key types
pub use answer::{Answer, ExtractedAnswer, GeneratedAnswer, Span};
pub use byte_stream::ByteStream;
pub use chat_message::{ChatMessage, ChatRole, ToolCall, ToolCallResult, TextContent};
pub use document::Document;
pub use sparse_embedding::SparseEmbedding;
pub use streaming_chunk::{StreamingChunk, StreamingCallback, select_streaming_callback, 
                         sync_streaming_callback, async_streaming_callback};