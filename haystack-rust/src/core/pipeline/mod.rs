/*!
 * Pipeline module for Haystack
 * 
 * This module contains the Pipeline implementation and related functionality.
 */

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::component_system::{Component, ComponentInfo, VariadicSocketHandler, SocketType};
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

/// Component execution priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExecutionPriority {
    /// Regular component with all inputs ready
    Regular = 0,
    
    /// Component with lazy variadic inputs that are all ready
    LazyVariadic = 1,
    
    /// Component with greedy variadic input(s) ready to execute
    GreedyVariadic = 2,
}

/// Component execution information
#[derive(Debug, Clone)]
struct ExecutionCandidate {
    /// Component name
    component_name: String,
    
    /// Priority level
    priority: ExecutionPriority,
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
        // Verify both components exist before getting references
        if !self.components.contains_key(source_component) {
            return Err(PipelineConnectError::new(format!("Source component '{}' not found", source_component)).into());
        }
        
        if !self.components.contains_key(target_component) {
            return Err(PipelineConnectError::new(format!("Target component '{}' not found", target_component)).into());
        }
        
        let source = self.components.get(source_component).unwrap();
        let target = self.components.get(target_component).unwrap();
        
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
            
            // For variadic sockets, check compatibility with inner type
            let compatible = if target_inputs[target_socket].is_variadic {
                if let Some(inner_type) = &target_inputs[target_socket].inner_type_name {
                    // If the target is variadic, the source type should match the inner type
                    source_type == inner_type
                } else {
                    // Legacy compatibility check
                    true
                }
            } else {
                // For regular sockets, the types must match exactly in strict mode
                source_type == target_type
            };
            
            if self.validation == ConnectionValidation::Strict && !compatible {
                return Err(PipelineConnectError::new(format!(
                    "Type mismatch in connection: source '{}' ({}) -> target '{}' ({})",
                    source_socket, source_type, target_socket, target_type
                )).into());
            }
        }
        
        // Add the connection
        println!("Adding connection: {}.{} -> {}.{}", 
                 source_component, source_socket, 
                 target_component, target_socket);
                 
        self.connections.push(Connection::new(
            source_component.to_string(),
            source_socket.to_string(),
            target_component.to_string(),
            target_socket.to_string(),
        ));
        
        // Update the input socket's sender list
        if let Some(component) = self.components.get(target_component) {
            if let Some(socket) = component.input_sockets().get(target_socket) {
                // This is a workaround since we can't actually modify the socket directly
                // In a real implementation, we would store senders in the pipeline instead
                // or modify the component's input sockets directly
                let mut socket = socket.clone();
                socket.add_sender(source_component.to_string());
            }
        }
        
        Ok(self)
    }
    
    /// Find the connections where a component is the source
    fn find_outgoing_connections(&self, component_name: &str) -> Vec<&Connection> {
        self.connections.iter()
            .filter(|conn| conn.source_component == component_name)
            .collect()
    }
    
    /// Find the connections where a component is the target
    fn find_incoming_connections(&self, component_name: &str) -> Vec<&Connection> {
        self.connections.iter()
            .filter(|conn| conn.target_component == component_name)
            .collect()
    }
    
    /// Check if a component has all its inputs available
    fn can_component_run(
        &self,
        component_name: &str,
        component: &dyn Component,
        component_outputs: &HashMap<String, HashMap<String, Value>>,
        component_inputs: &HashMap<String, HashMap<String, Value>>,
        variadic_handler: &VariadicSocketHandler,
    ) -> bool {
        for (socket_name, socket) in component.input_sockets() {
            // Skip checks for greedy variadic sockets that already have input
            if socket.is_variadic && socket.is_greedy && 
               variadic_handler.has_greedy_variadic_input(component_name, socket_name) {
                continue;
            }
            
            // For lazy variadic sockets, check if all expected inputs are available
            if socket.is_variadic && !socket.is_greedy {
                if !socket.senders.is_empty() && 
                   !variadic_handler.has_lazy_variadic_socket_received_all_inputs(
                       component_name, socket_name, &socket.senders) {
                    return false;
                }
                continue;
            }
            
            // Check if input is already available in component_inputs
            if let Some(inputs) = component_inputs.get(component_name) {
                if inputs.contains_key(socket_name) {
                    continue;
                }
            }
            
            // Check for default values
            if let Some(_) = &socket.default_value {
                continue;
            }
            
            // Check for connected inputs
            let mut found_connected_input = false;
            for conn in &self.connections {
                if conn.target_component == component_name && conn.target_socket == *socket_name {
                    if let Some(source_outputs) = component_outputs.get(&conn.source_component) {
                        if let Some(_) = source_outputs.get(&conn.source_socket) {
                            found_connected_input = true;
                            break;
                        }
                    }
                }
            }
            
            if !found_connected_input {
                return false;
            }
        }
        
        true
    }
    
    /// Determine a component's execution priority
    fn determine_component_priority(
        &self,
        component_name: &str,
        component: &dyn Component,
        variadic_handler: &VariadicSocketHandler,
    ) -> ExecutionPriority {
        // Check for greedy variadic sockets with ready inputs (highest priority)
        for (socket_name, socket) in component.input_sockets() {
            if socket.is_variadic && socket.is_greedy && 
               variadic_handler.has_greedy_variadic_input(component_name, socket_name) {
                return ExecutionPriority::GreedyVariadic;
            }
        }
        
        // Check for lazy variadic sockets with all inputs ready (middle priority)
        if component.input_sockets().values().any(|s| s.is_variadic && !s.is_greedy) {
            if variadic_handler.are_all_lazy_variadic_sockets_resolved(component_name, component.input_sockets()) {
                return ExecutionPriority::LazyVariadic;
            }
        }
        
        // Regular component (lowest priority)
        ExecutionPriority::Regular
    }
    
    /// Prepare inputs for a component, handling variadic sockets
    /// 
    /// This method does the following:
    /// 1. Takes direct inputs from the component_inputs map
    /// 2. Consumes variadic inputs if already collected
    /// 3. Applies default values if no input is available
    /// 4. Collects inputs from connected components via outputs
    /// 5. For variadic sockets, ensures they are ready to consume
    /// 
    /// The inputs are returned as a map from socket name to value, ready to be passed to the component's run method.
    fn prepare_component_inputs(
        &self,
        component_name: &str,
        component: &dyn Component,
        component_outputs: &HashMap<String, HashMap<String, Value>>,
        component_inputs: &HashMap<String, HashMap<String, Value>>,
        variadic_handler: &mut VariadicSocketHandler,
    ) -> Result<HashMap<String, Value>> {
        let mut input_values = HashMap::new();
        
        // Process each input socket
        for (socket_name, socket) in component.input_sockets() {
            // Check for directly provided inputs
            if let Some(inputs) = component_inputs.get(component_name) {
                if let Some(value) = inputs.get(socket_name) {
                    input_values.insert(socket_name.clone(), value.clone());
                    continue;
                }
            }
            
            // Handle variadic sockets
            if socket.is_variadic {
                // Consume variadic inputs
                if let Ok(variadic_value) = variadic_handler.consume_variadic_inputs(
                    component_name, 
                    socket_name,
                    socket.socket_type()
                ) {
                    input_values.insert(socket_name.clone(), variadic_value);
                    continue;
                }
            }
            
            // Check for default values
            if let Some(default_value) = &socket.default_value {
                // For variadic sockets, default should be wrapped in an array
                let value = if socket.is_variadic {
                    Value::Array(vec![default_value.clone()])
                } else {
                    default_value.clone()
                };
                
                input_values.insert(socket_name.clone(), value);
                continue;
            }
            
            // Check for connected inputs
            for conn in &self.connections {
                if conn.target_component == component_name && conn.target_socket == *socket_name {
                    if let Some(source_outputs) = component_outputs.get(&conn.source_component) {
                        if let Some(value) = source_outputs.get(&conn.source_socket) {
                            // For variadic sockets, save the value for later consumption
                            if socket.is_variadic {
                                // For variadic inputs from connections, add the value directly
                                // No special handling is needed - just add it to the pending inputs
                                println!("Adding variadic input {}.{} -> {}.{}: {:?}", 
                                         conn.source_component, conn.source_socket,
                                         component_name, socket_name, value);
                                         
                                variadic_handler.add_pending_input(
                                    component_name,
                                    socket_name,
                                    &conn.source_component,
                                    value.clone()
                                );
                            } else {
                                input_values.insert(socket_name.clone(), value.clone());
                            }
                            break;
                        }
                    }
                }
            }
            
            // For variadic sockets, consume them now if ready
            if socket.is_variadic {
                // Only consume lazy variadic sockets if all inputs are available
                if !socket.is_greedy {
                    if variadic_handler.has_lazy_variadic_socket_received_all_inputs(
                        component_name, socket_name, &socket.senders
                    ) {
                        if let Ok(variadic_value) = variadic_handler.consume_variadic_inputs(
                            component_name, 
                            socket_name,
                            socket.socket_type()
                        ) {
                            input_values.insert(socket_name.clone(), variadic_value);
                        }
                    }
                } else {
                    // For greedy variadic, consume whatever we have so far
                    if let Ok(variadic_value) = variadic_handler.consume_variadic_inputs(
                        component_name, 
                        socket_name,
                        socket.socket_type()
                    ) {
                        input_values.insert(socket_name.clone(), variadic_value);
                    }
                }
            }
        }
        
        Ok(input_values)
    }
    
    /// Run the pipeline with the given inputs
    pub fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Initialize component states
        let mut component_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
        let mut component_inputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
        let mut executed_components: HashSet<String> = HashSet::new();
        let mut variadic_handler = VariadicSocketHandler::new();
        
        // Build a dependency graph for proper component execution order
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_dependencies: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize all components in the dependency graph
        for component_name in self.components.keys() {
            dependency_graph.insert(component_name.clone(), Vec::new());
            reverse_dependencies.insert(component_name.clone(), Vec::new());
        }
        
        // Populate the dependency graph based on connections
        for connection in &self.connections {
            // The target depends on the source
            dependency_graph
                .entry(connection.target_component.clone())
                .or_insert_with(Vec::new)
                .push(connection.source_component.clone());
                
            // Record reverse dependencies for later use
            reverse_dependencies
                .entry(connection.source_component.clone())
                .or_insert_with(Vec::new)
                .push(connection.target_component.clone());
        }
        
        println!("Dependency graph: {:?}", dependency_graph);
        
        // Warm up all components
        for (name, component) in &self.components {
            component.warm_up()
                .with_context(|| format!("Failed to warm up component '{}'", name))?;
        }
        
        // Initialize with provided inputs
        for (component_name, component) in &self.components {
            for (socket_name, socket) in component.input_sockets() {
                // Check for externally provided inputs
                let input_key = format!("{}.{}", component_name, socket_name);
                if let Some(value) = inputs.get(&input_key) {
                    // For variadic sockets, externally provided inputs need special handling
                    if socket.is_variadic {
                        variadic_handler.add_pending_input(
                            component_name,
                            socket_name,
                            "__external__", // Special marker for external inputs
                            value.clone()
                        );
                    } else {
                        component_inputs
                            .entry(component_name.clone())
                            .or_insert_with(HashMap::new)
                            .insert(socket_name.clone(), value.clone());
                    }
                }
            }
        }
        
        // Execute the pipeline
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        
        while executed_components.len() < self.components.len() && iteration < MAX_ITERATIONS {
            // Find runnable components that have all dependencies satisfied
            let mut ready_components = Vec::new();
            
            for (component_name, component) in &self.components {
                // Skip already executed components
                if executed_components.contains(component_name) {
                    continue;
                }
                
                // Forward outputs from dependencies to detect greedy variadic status first
                // This allows greedy variadic components to see if they have any inputs
                if let Err(e) = self.forward_outputs_to_component(
                    component_name,
                    &component_outputs,
                    &mut variadic_handler
                ) {
                    println!("Error forwarding outputs: {}", e);
                }
                
                // Determine if this component has any greedy variadic sockets with inputs
                let has_greedy_inputs = component.input_sockets().iter().any(|(socket_name, socket)| {
                    socket.is_variadic && socket.is_greedy && 
                    variadic_handler.has_greedy_variadic_input(component_name, socket_name)
                });
                
                // For components with greedy variadic inputs, we allow partial dependencies
                // For regular components, we require all dependencies to be executed
                let dependencies = &dependency_graph[component_name];
                let dependencies_ready = if has_greedy_inputs {
                    // For greedy variadic, we need at least one dependency to be executed
                    // OR no dependencies (for source components)
                    dependencies.is_empty() || dependencies.iter().any(|dep| executed_components.contains(dep))
                } else {
                    // For regular components, we need all dependencies to be executed
                    dependencies.iter().all(|dep| executed_components.contains(dep))
                };
                
                if dependencies_ready {
                    
                    // Determine execution priority - must come before can_component_run check
                    let priority = self.determine_component_priority(
                        component_name,
                        component.as_ref(),
                        &variadic_handler
                    );
                    
                    // Special handling for greedy variadic components - they can run as soon as inputs are available
                    let is_greedy_variadic_ready = 
                        priority == ExecutionPriority::GreedyVariadic;
                    
                    // Check if component can run with available inputs
                    if is_greedy_variadic_ready || self.can_component_run(
                        component_name,
                        component.as_ref(),
                        &component_outputs,
                        &component_inputs,
                        &variadic_handler
                    ) {
                        ready_components.push(ExecutionCandidate {
                            component_name: component_name.clone(),
                            priority,
                        });
                    }
                }
            }
            
            // Sort components by priority (highest first)
            ready_components.sort_by(|a, b| b.priority.cmp(&a.priority));
            
            println!("Ready components: {:?}", ready_components);
            println!("Variadic pending inputs: {:?}", variadic_handler.get_pending_inputs_debug());
            
            if ready_components.is_empty() {
                // We can't make progress
                println!("No ready components. Executed: {:?}, Total: {}", executed_components, self.components.len());
                
                // Print remaining unexecuted components for debugging
                let remaining: Vec<_> = self.components.keys()
                    .filter(|c| !executed_components.contains(c.as_str()))
                    .collect();
                println!("Remaining components: {:?}", remaining);
                
                return Err(PipelineComponentsBlockedError::new().into());
            }
            
            // Run components in priority order
            let mut made_progress = false;
            for candidate in ready_components {
                let component_name = candidate.component_name;
                let component = self.components
                    .get(&component_name)
                    .expect("Component disappeared during execution");
                
                // For greedy variadic components, check if we have inputs
                let has_greedy_inputs = component.input_sockets().iter().any(|(socket_name, socket)| {
                    socket.is_variadic && socket.is_greedy && 
                    variadic_handler.has_greedy_variadic_input(&component_name, socket_name)
                });
                
                // Forward outputs from dependencies to this component
                if !has_greedy_inputs {
                    // Only forward for non-greedy components since greedy ones were already processed
                    self.forward_outputs_to_component(
                        &component_name,
                        &component_outputs,
                        &mut variadic_handler
                    )?;
                }
                
                // Prepare inputs, handling variadic sockets
                let component_input_values = self.prepare_component_inputs(
                    &component_name,
                    component.as_ref(),
                    &component_outputs,
                    &component_inputs,
                    &mut variadic_handler
                )?;
                
                // Run the component
                println!("Running component '{}' with inputs: {:?}", component_name, component_input_values.keys());
                
                match component.run(component_input_values) {
                    Ok(outputs) => {
                        println!("Component '{}' produced outputs: {:?}", component_name, outputs.keys());
                        component_outputs.insert(component_name.clone(), outputs);
                        made_progress = true;
                        
                        // Mark component as executed
                        executed_components.insert(component_name.clone());
                    }
                    Err(err) => {
                        return Err(PipelineRuntimeError::new(
                            Some(component_name.clone()),
                            None,
                            format!("Component '{}' failed to run: {}", component_name, err),
                        ).into());
                    }
                }
            }
            
            if !made_progress {
                // We're stuck
                println!("No progress made. Executed: {:?}, Total: {}", executed_components, self.components.len());
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
    
    /// Forward outputs from dependencies to component with variadic inputs
    fn forward_outputs_to_component(
        &self,
        component_name: &str,
        component_outputs: &HashMap<String, HashMap<String, Value>>,
        variadic_handler: &mut VariadicSocketHandler,
    ) -> Result<()> {
        // Get connections where this component is the target
        let incoming_connections = self.find_incoming_connections(component_name);
        
        // Get the component's input sockets
        let component = self.components.get(component_name)
            .ok_or_else(|| anyhow!("Component '{}' not found", component_name))?;
        let input_sockets = component.input_sockets();
        
        // For each connection, forward the source output to the target
        for connection in incoming_connections {
            // Check if target socket is variadic
            if let Some(target_socket) = input_sockets.get(&connection.target_socket) {
                if target_socket.is_variadic {
                    // Get source component's output
                    if let Some(source_outputs) = component_outputs.get(&connection.source_component) {
                        if let Some(value) = source_outputs.get(&connection.source_socket) {
                            // Skip if this connection has already been forwarded
                            if variadic_handler.has_pending_input(
                                component_name,
                                &connection.target_socket,
                                &connection.source_component
                            ) {
                                continue;
                            }
                            
                            println!("Forwarding {}.{} -> {}.{}: {:?}", 
                                connection.source_component, connection.source_socket,
                                component_name, connection.target_socket,
                                value);
                                
                            // Add to variadic handler
                            variadic_handler.add_pending_input(
                                component_name,
                                &connection.target_socket,
                                &connection.source_component,
                                value.clone()
                            );
                        }
                    }
                }
            }
        }
        
        Ok(())
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