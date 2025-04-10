/*!
 * Component registry for Haystack
 * 
 * This module provides a registry system for Haystack components,
 * enabling automatic discovery and instantiation of components.
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use serde_json::Value;
use lazy_static::lazy_static;

use crate::component::{Component, ComponentInfo};
use crate::errors::ComponentDeserializationError;

/// Factory function signature for creating components
pub type ComponentFactory = Box<dyn Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync>;

/// Registry of component types used for deserialization
#[derive(Default)]
pub struct ComponentRegistry {
    /// Map from component type string to a factory function
    factories: HashMap<String, ComponentFactory>,
    
    /// Map from component type string to component metadata
    metadata: HashMap<String, ComponentMetadata>,
}

/// Metadata about a component type
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    /// The name of the component class (e.g., "TextSplitter")
    pub class_name: String,
    
    /// The module path to the component (e.g., "haystack.components.preprocessors.text_splitter")
    pub module_path: String,
    
    /// Short description of what the component does
    pub description: String,
    
    /// Version of the component
    pub version: String,
    
    /// Tags for component categorization
    pub tags: Vec<String>,
}

impl ComponentMetadata {
    /// Create new component metadata
    pub fn new(
        class_name: impl Into<String>,
        module_path: impl Into<String>,
        description: impl Into<String>,
        version: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            class_name: class_name.into(),
            module_path: module_path.into(),
            description: description.into(),
            version: version.into(),
            tags,
        }
    }
    
    /// Get the full type name (module_path.class_name)
    pub fn type_name(&self) -> String {
        format!("{}.{}", self.module_path, self.class_name)
    }
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
            metadata: HashMap::new(),
        }
    }
    
    /// Register a component factory function
    pub fn register<F>(&mut self, metadata: ComponentMetadata, factory: F)
    where
        F: Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync + 'static,
    {
        let type_name = metadata.type_name();
        self.factories.insert(type_name.clone(), Box::new(factory));
        self.metadata.insert(type_name, metadata);
    }
    
    /// Create a component instance from a component info
    pub fn create_component(&self, info: &ComponentInfo) -> Result<Box<dyn Component>> {
        let type_name = format!("{}.{}", info.module_path, info.class_name);
        let factory = self.factories.get(&type_name)
            .ok_or_else(|| ComponentDeserializationError::new(
                format!("No factory registered for component type '{}'", type_name)
            ))?;
        
        factory(info)
    }
    
    /// Get all registered component types
    pub fn registered_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
    
    /// Get metadata for a specific component type
    pub fn get_metadata(&self, type_name: &str) -> Option<&ComponentMetadata> {
        self.metadata.get(type_name)
    }
    
    /// Get all component metadata
    pub fn all_metadata(&self) -> HashMap<String, ComponentMetadata> {
        self.metadata.clone()
    }
    
    /// Find components by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<String> {
        self.metadata.iter()
            .filter(|(_, metadata)| metadata.tags.contains(&tag.to_string()))
            .map(|(type_name, _)| type_name.clone())
            .collect()
    }
}

// Global component registry
lazy_static! {
    static ref COMPONENT_REGISTRY: RwLock<ComponentRegistry> = RwLock::new(ComponentRegistry::new());
}

/// Register a component factory function in the global registry
pub fn register_component<F>(metadata: ComponentMetadata, factory: F)
where
    F: Fn(&ComponentInfo) -> Result<Box<dyn Component>> + Send + Sync + 'static,
{
    let mut registry = COMPONENT_REGISTRY.write().unwrap();
    registry.register(metadata, factory);
}

/// Create a component instance from component info using the global registry
pub fn create_component(info: &ComponentInfo) -> Result<Box<dyn Component>> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.create_component(info)
}

/// Get all registered component types
pub fn registered_component_types() -> Vec<String> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.registered_types()
}

/// Get component metadata by type name
pub fn get_component_metadata(type_name: &str) -> Option<ComponentMetadata> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.get_metadata(type_name).cloned()
}

/// Get all component metadata
pub fn all_component_metadata() -> HashMap<String, ComponentMetadata> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.all_metadata()
}

/// Find components by tag
pub fn find_components_by_tag(tag: &str) -> Vec<String> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.find_by_tag(tag)
}

/// Macro for registering a component
#[macro_export]
macro_rules! register_component {
    ($class_name:expr, $module_path:expr, $description:expr, $version:expr, $tags:expr, $factory:expr) => {
        $crate::component::registry::register_component(
            $crate::component::registry::ComponentMetadata::new(
                $class_name,
                $module_path,
                $description,
                $version,
                $tags,
            ),
            $factory,
        );
    };
}

/// Macro for creating a component factory
#[macro_export]
macro_rules! component_factory {
    ($component_type:ty) => {
        |info: &$crate::component::ComponentInfo| -> anyhow::Result<Box<dyn $crate::component::Component>> {
            let component = <$component_type>::from_component_info(info)?;
            Ok(Box::new(component))
        }
    };
}