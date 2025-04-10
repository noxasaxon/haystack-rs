/*!
 * Type utilities for Haystack
 * 
 * This module contains utilities for working with types in Haystack.
 */

use std::any::TypeId;

/// Get a user-friendly name for a type
pub fn type_name(type_id: TypeId) -> String {
    // This implementation is limited but works for basic types
    // In a real implementation, we would need a more robust approach
    if type_id == TypeId::of::<i32>() {
        "i32".to_string()
    } else if type_id == TypeId::of::<u32>() {
        "u32".to_string()
    } else if type_id == TypeId::of::<i64>() {
        "i64".to_string()
    } else if type_id == TypeId::of::<u64>() {
        "u64".to_string()
    } else if type_id == TypeId::of::<f32>() {
        "f32".to_string()
    } else if type_id == TypeId::of::<f64>() {
        "f64".to_string()
    } else if type_id == TypeId::of::<String>() {
        "String".to_string()
    } else if type_id == TypeId::of::<bool>() {
        "bool".to_string()
    } else if type_id == TypeId::of::<Vec<String>>() {
        "Vec<String>".to_string()
    } else if type_id == TypeId::of::<Vec<i32>>() {
        "Vec<i32>".to_string()
    } else if type_id == TypeId::of::<Vec<f64>>() {
        "Vec<f64>".to_string()
    } else if type_id == TypeId::of::<Option<String>>() {
        "Option<String>".to_string()
    } else if type_id == TypeId::of::<Option<i32>>() {
        "Option<i32>".to_string()
    } else if type_id == TypeId::of::<Option<f64>>() {
        "Option<f64>".to_string()
    } else {
        // Fallback for unknown types
        "Unknown".to_string()
    }
}

/// Check if a type is compatible with another type
/// 
/// This function checks if a value of `from_type` can be safely assigned to a variable of `to_type`.
pub fn is_type_compatible(from_type: TypeId, to_type: TypeId) -> bool {
    // Simple implementation for basic types
    from_type == to_type
}

/// Check if a type is a subtype of another type
/// 
/// This function checks if `child_type` is a subtype of `parent_type`.
pub fn is_subtype_of(child_type: TypeId, parent_type: TypeId) -> bool {
    // In Rust, we don't have the same subtyping rules as in languages like Python,
    // but this function simulates similar behavior for common types.
    
    // For now, just check equality
    child_type == parent_type
}