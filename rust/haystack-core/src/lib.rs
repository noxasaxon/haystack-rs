/*!
 * Haystack Core
 *
 * Core functionality for the Haystack Rust library.
 */

pub mod component;
pub mod components;
pub mod document_stores;
pub mod errors;
pub mod pipeline;
pub mod serialization;
pub mod type_utils;

#[cfg(test)]
mod tests;

/// Re-export haystack-dataclasses
pub use haystack_dataclasses;