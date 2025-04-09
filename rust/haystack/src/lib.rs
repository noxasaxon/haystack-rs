/*!
 * Haystack Rust
 *
 * Main entry point for the Haystack Rust library.
 */

pub use haystack_core;
pub use haystack_dataclasses;

/// Re-export version information
pub mod version {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Initialize Haystack and its dependencies
pub fn init() {
    // Initialize any necessary resources or logging
    tracing::info!("Haystack-rs initialized (version: {})", version::VERSION);
}