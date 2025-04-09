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

/// Registry of component types used for deserialization
#[derive(Default)]
pub struct ComponentRegistry {
    /// Map from component type string to a factory function
    factories: HashMap<String, Box<dyn Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync>>,
}

impl std::fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("registered_types", &self.factories.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    
    /// Register a component factory function
    pub fn register<F>(&mut self, type_name: &str, factory: F)
    where
        F: Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync + 'static,
    {
        self.factories.insert(type_name.to_string(), Box::new(factory));
    }
    
    /// Create a component instance from a component info
    pub fn create_component(&self, info: &ComponentInfo) -> Result<Box<dyn Component>> {
        let type_name = format!("{}.{}", info.module_path, info.class_name);
        let factory = self.factories.get(&type_name)
            .ok_or_else(|| SerializationError::new(format!("No factory registered for component type '{}'", type_name)))?;
        
        factory(info)
    }
}

// Global component registry
lazy_static::lazy_static! {
    static ref COMPONENT_REGISTRY: std::sync::RwLock<ComponentRegistry> = std::sync::RwLock::new(ComponentRegistry::new());
}

/// Register a component factory function in the global registry
pub fn register_component<F>(type_name: &str, factory: F)
where
    F: Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync + 'static,
{
    let mut registry = COMPONENT_REGISTRY.write().unwrap();
    registry.register(type_name, factory);
}

/// Create a component instance from component info using the global registry
pub fn create_component(info: &ComponentInfo) -> Result<Box<dyn Component>> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.create_component(info)
}