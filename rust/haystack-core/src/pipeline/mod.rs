/*!
 * Pipeline module for Haystack
 * 
 * This module contains the Pipeline implementation and related functionality.
 */

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::component::{Component, ComponentInfo};
use crate::errors::{PipelineRuntimeError, PipelineConnectError, PipelineComponentsBlockedError};
use crate::serialization::DictSerializable;

/// A connection between two component sockets in a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// The name of the source component
    pub source_component: String,
    
    /// The name of the source socket
    pub source_socket: String,
    
    /// The name of the target component
    pub target_component: String,
    
    /// The name of the target socket
    pub target_socket: String,
}

impl Connection {
    /// Create a new connection
    pub fn new(
        source_component: String,
        source_socket: String,
        target_component: String,
        target_socket: String,
    ) -> Self {
        Self {
            source_component,
            source_socket,
            target_component,
            target_socket,
        }
    }
}

/// Defines how pipeline connections should be validated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionValidation {
    /// Strict validation ensures type compatibility
    Strict,
    
    /// Relaxed validation allows connecting incompatible types
    Relaxed,
    
    /// Disabled validation allows any connection
    Disabled,
}

/// The main Pipeline class for Haystack
#[derive(Clone)]
pub struct Pipeline {
    /// Map of component names to components
    components: HashMap<String, Arc<dyn Component>>,
    
    /// List of connections between components
    connections: Vec<Connection>,
    
    /// Connection validation mode
    validation: ConnectionValidation,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("connections", &self.connections)
            .field("validation", &self.validation)
            .field("components", &self.components.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: Vec::new(),
            validation: ConnectionValidation::Strict,
        }
    }
    
    /// Create a new pipeline with a specific validation mode
    pub fn with_validation(validation: ConnectionValidation) -> Self {
        Self {
            components: HashMap::new(),
            connections: Vec::new(),
            validation,
        }
    }
    
    /// Add a component to the pipeline
    pub fn add_component<C: Component + 'static>(&mut self, name: &str, component: C) -> Result<&mut Self> {
        if self.components.contains_key(name) {
            return Err(anyhow!("Component with name '{}' already exists in the pipeline", name));
        }
        
        self.components.insert(name.to_string(), Arc::new(component));
        Ok(self)
    }
    
    /// Get a reference to a component by name
    pub fn get_component(&self, name: &str) -> Option<&Arc<dyn Component>> {
        self.components.get(name)
    }
    
    /// Remove a component from the pipeline
    pub fn remove_component(&mut self, name: &str) -> Result<&mut Self> {
        if !self.components.contains_key(name) {
            return Err(anyhow!("Component with name '{}' does not exist in the pipeline", name));
        }
        
        // Remove all connections involving this component
        self.connections.retain(|conn| {
            conn.source_component != name && conn.target_component != name
        });
        
        self.components.remove(name);
        Ok(self)
    }
    
    /// Connect two components
    pub fn connect(
        &mut self,
        source_component: &str,
        source_socket: &str,
        target_component: &str,
        target_socket: &str,
    ) -> Result<&mut Self> {
        // Validate that components exist
        let source = self.components.get(source_component)
            .ok_or_else(|| PipelineConnectError::new(format!("Source component '{}' not found", source_component)))?;
        
        let target = self.components.get(target_component)
            .ok_or_else(|| PipelineConnectError::new(format!("Target component '{}' not found", target_component)))?;
        
        // Validate that sockets exist
        let source_outputs = source.output_sockets();
        if !source_outputs.contains_key(source_socket) {
            return Err(PipelineConnectError::new(format!(
                "Source socket '{}' not found in component '{}'", 
                source_socket, source_component
            )).into());
        }
        
        let target_inputs = target.input_sockets();
        if !target_inputs.contains_key(target_socket) {
            return Err(PipelineConnectError::new(format!(
                "Target socket '{}' not found in component '{}'", 
                target_socket, target_component
            )).into());
        }
        
        // Validate type compatibility if validation is enabled
        if self.validation != ConnectionValidation::Disabled {
            let source_type = &source_outputs[source_socket].type_name;
            let target_type = &target_inputs[target_socket].type_name;
            
            // In strict mode, the types must match exactly 
            if self.validation == ConnectionValidation::Strict && source_type != target_type {
                return Err(PipelineConnectError::new(format!(
                    "Type mismatch in connection: source '{}' ({}) -> target '{}' ({})",
                    source_socket, source_type, target_socket, target_type
                )).into());
            }
        }
        
        // Add the connection
        self.connections.push(Connection::new(
            source_component.to_string(),
            source_socket.to_string(),
            target_component.to_string(),
            target_socket.to_string(),
        ));
        
        Ok(self)
    }
    
    /// Run the pipeline with the given inputs
    pub fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Initialize component states
        let mut component_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
        let mut component_inputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
        let mut blocked_components: HashSet<String> = HashSet::new();
        
        // Warm up all components
        for (name, component) in &self.components {
            component.warm_up().with_context(|| format!("Failed to warm up component '{}'", name))?;
        }
        
        // Initialize with provided inputs
        for (component_name, component) in &self.components {
            for (socket_name, _socket) in component.input_sockets() {
                // Check for externally provided inputs
                if let Some(value) = inputs.get(&format!("{}.{}", component_name, socket_name)) {
                    component_inputs
                        .entry(component_name.clone())
                        .or_insert_with(HashMap::new)
                        .insert(socket_name.clone(), value.clone());
                }
            }
        }
        
        // Execute the pipeline
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        
        while blocked_components.len() < self.components.len() && iteration < MAX_ITERATIONS {
            let mut made_progress = false;
            
            // Find runnable components
            for (component_name, component) in &self.components {
                if blocked_components.contains(component_name) {
                    continue;
                }
                
                // Check if all required inputs are available
                let mut can_run = true;
                let mut component_input_values = HashMap::new();
                
                for (socket_name, socket) in component.input_sockets() {
                    // Check if input is already available
                    if let Some(inputs) = component_inputs.get(component_name) {
                        if let Some(value) = inputs.get(socket_name) {
                            component_input_values.insert(socket_name.clone(), value.clone());
                            continue;
                        }
                    }
                    
                    // Check for default values
                    if let Some(default_value) = &socket.default_value {
                        component_input_values.insert(socket_name.clone(), default_value.clone());
                        continue;
                    }
                    
                    // Check for connected inputs
                    let mut found_connected_input = false;
                    for conn in &self.connections {
                        if conn.target_component == *component_name && conn.target_socket == *socket_name {
                            if let Some(source_outputs) = component_outputs.get(&conn.source_component) {
                                if let Some(value) = source_outputs.get(&conn.source_socket) {
                                    component_input_values.insert(socket_name.clone(), value.clone());
                                    found_connected_input = true;
                                    break;
                                }
                            }
                        }
                    }
                    
                    if !found_connected_input {
                        can_run = false;
                        break;
                    }
                }
                
                if !can_run {
                    continue;
                }
                
                // Run the component
                match component.run(component_input_values) {
                    Ok(outputs) => {
                        component_outputs.insert(component_name.clone(), outputs);
                        made_progress = true;
                    }
                    Err(err) => {
                        return Err(PipelineRuntimeError::new(
                            Some(component_name.clone()),
                            None,
                            format!("Component '{}' failed to run: {}", component_name, err),
                        ).into());
                    }
                }
                
                // Mark as blocked since it has run
                blocked_components.insert(component_name.clone());
            }
            
            if !made_progress {
                // We're stuck
                return Err(PipelineComponentsBlockedError::new().into());
            }
            
            iteration += 1;
        }
        
        if iteration >= MAX_ITERATIONS {
            return Err(anyhow!("Pipeline execution exceeded maximum number of iterations"));
        }
        
        // Collect outputs from all components
        let mut results = HashMap::new();
        for (component_name, outputs) in component_outputs {
            for (socket_name, value) in outputs {
                results.insert(format!("{}.{}", component_name, socket_name), value);
            }
        }
        
        Ok(results)
    }
    
    /// Get all connections in the pipeline
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }
    
    /// Get all components in the pipeline
    pub fn components(&self) -> &HashMap<String, Arc<dyn Component>> {
        &self.components
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl DictSerializable for Pipeline {
    fn to_dict(&self) -> Result<Value> {
        let mut result = serde_json::Map::new();
        let mut components = serde_json::Map::new();
        
        // Serialize components
        for (name, component) in &self.components {
            let component_info = ComponentInfo {
                class_name: "Unknown".to_string(), // This would come from component_info() in a real implementation
                module_path: "unknown.module".to_string(),
                init_parameters: component.init_parameters().clone(),
            };
            
            components.insert(name.clone(), serde_json::to_value(component_info)?);
        }
        
        result.insert("components".to_string(), Value::Object(components));
        result.insert("connections".to_string(), serde_json::to_value(&self.connections)?);
        result.insert("validation".to_string(), serde_json::to_value(&self.validation)?);
        
        Ok(Value::Object(result))
    }
    
    fn from_dict(data: &Value) -> Result<Self> {
        // In a real implementation, this would use the component registry to create components
        // For now, return a minimal implementation
        let data_obj = data.as_object()
            .context("Expected an object for Pipeline deserialization")?;
        
        let validation = data_obj.get("validation")
            .and_then(|v| serde_json::from_value::<ConnectionValidation>(v.clone()).ok())
            .unwrap_or(ConnectionValidation::Strict);
        
        let connections = data_obj.get("connections")
            .and_then(|v| serde_json::from_value::<Vec<Connection>>(v.clone()).ok())
            .unwrap_or_default();
        
        let mut pipeline = Pipeline::with_validation(validation);
        pipeline.connections = connections;
        
        Ok(pipeline)
    }
}