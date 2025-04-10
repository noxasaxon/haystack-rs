/*!
 * Haystack Core
 *
 * Core functionality for the Haystack Rust library.
 */

// Core module that provides the component system infrastructure
pub mod component_system;

// Module containing concrete component implementations
pub mod components;

pub mod document_stores;
pub mod errors;
pub mod macros;
pub mod pipeline;
pub mod serialization;
pub mod type_utils;

#[cfg(test)]
mod tests;

// Re-export macros
pub use crate::macros::*;

// Re-export core component functionality for convenience
pub use component_system::{
    Component, ComponentBase, InputSocket, OutputSocket,
    Variadic, GreedyVariadic, SocketType, ComponentInfo,
    register_component, create_component, AsyncComponent,
};

/// Re-export haystack-dataclasses
pub use haystack_dataclasses;