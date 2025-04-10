/*!
 * Serialization utilities for Haystack
 * 
 * This module contains serialization and deserialization utilities for Haystack components
 * and pipelines.
 */

use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::component::{Component, ComponentInfo};
use crate::errors::SerializationError;

/// Trait for objects that can be serialized to and deserialized from dictionaries
pub trait DictSerializable: Sized {
    /// Convert the object to a dictionary
    fn to_dict(&self) -> Result<Value>;
    
    /// Create a new object from a dictionary
    fn from_dict(data: &Value) -> Result<Self>;
}

/// Helper function to convert an object to a dictionary
pub fn default_to_dict<T: Serialize>(_obj: &T, params: HashMap<String, Value>) -> Result<Value> {
    let mut result = serde_json::Map::new();
    let mut init_params = serde_json::Map::new();
    
    for (key, value) in params {
        init_params.insert(key, value);
    }
    
    result.insert("type".to_string(), serde_json::to_value(std::any::type_name::<T>())?);
    result.insert("init_parameters".to_string(), Value::Object(init_params));
    
    Ok(Value::Object(result))
}

/// Helper function to create an object from a dictionary
pub fn default_from_dict<T: for<'de> Deserialize<'de>>(data: &Value) -> Result<T> {
    let data_obj = data.as_object()
        .context("Expected an object for deserialization")?;
    
    let init_params = data_obj.get("init_parameters")
        .context("Missing 'init_parameters' in serialized object")?;
    
    serde_json::from_value(init_params.clone())
        .context("Failed to deserialize init parameters")
}

/// Create a component instance from component info using the registry
pub fn deserialize_component(info: &ComponentInfo) -> Result<Box<dyn Component>> {
    crate::component::registry::create_component(info)
}

/// Convert a component to a serialized form
pub fn serialize_component<C: Component + ComponentSerialization>(component: &C) -> Result<Value> {
    let component_info = component.component_info();
    let mut result = serde_json::Map::new();
    
    result.insert("type".to_string(), Value::String(format!("{}.{}", 
        component_info.module_path, 
        component_info.class_name
    )));
    result.insert("init_parameters".to_string(), serde_json::to_value(component_info.init_parameters)?);
    
    Ok(Value::Object(result))
}

/// Trait for components that can be serialized and deserialized
pub trait ComponentSerialization {
    /// Get the component info for serialization
    fn component_info(&self) -> ComponentInfo;
    
    /// Create a new instance of the component from deserialized info
    fn from_component_info(info: &ComponentInfo) -> Result<Box<dyn Component>>;
}