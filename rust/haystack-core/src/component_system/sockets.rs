/*!
 * Socket management for components
 */

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use super::types::{InputSocket, OutputSocket, SocketType, Variadic, GreedyVariadic};

/// Sockets represents the inputs or outputs of a Component
/// 
/// This provides a convenient way to access and manage a component's sockets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sockets<T> {
    /// Socket type marker
    #[serde(skip)]
    socket_type: PhantomData<T>,
    
    /// The map of sockets by name
    sockets: HashMap<String, T>,
    
    /// Component name for display purposes
    #[serde(skip)]
    component_name: Option<String>,
}

impl<T> Sockets<T> {
    /// Create a new Sockets container
    pub fn new(sockets: HashMap<String, T>) -> Self {
        Self {
            socket_type: PhantomData,
            sockets,
            component_name: None,
        }
    }
    
    /// Set the component name
    pub fn set_component_name(&mut self, name: String) {
        self.component_name = Some(name);
    }
    
    /// Get a reference to a socket by name
    pub fn get(&self, name: &str) -> Option<&T> {
        self.sockets.get(name)
    }
    
    /// Get a mutable reference to a socket by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.sockets.get_mut(name)
    }
    
    /// Insert a socket
    pub fn insert(&mut self, name: String, socket: T) {
        self.sockets.insert(name, socket);
    }
    
    /// Check if a socket with the given name exists
    pub fn contains(&self, name: &str) -> bool {
        self.sockets.contains_key(name)
    }
    
    /// Get a reference to the underlying socket map
    pub fn inner(&self) -> &HashMap<String, T> {
        &self.sockets
    }
    
    /// Get a mutable reference to the underlying socket map
    pub fn inner_mut(&mut self) -> &mut HashMap<String, T> {
        &mut self.sockets
    }
    
    /// Iterate over the sockets
    pub fn iter(&self) -> impl Iterator<Item = (&String, &T)> {
        self.sockets.iter()
    }
}

impl Sockets<InputSocket> {
    /// Create a formatted string that shows all input sockets
    pub fn format_inputs(&self) -> String {
        let mut result = String::from("Inputs:\n");
        for (_, socket) in self.sockets.iter() {
            result.push_str(&format!("  - {}: {}", socket.name, socket.type_name));
            if socket.is_variadic {
                result.push_str(" (variadic)");
            }
            if socket.is_greedy {
                result.push_str(" (greedy)");
            }
            if !socket.is_mandatory() {
                result.push_str(" (optional)");
            }
            result.push('\n');
        }
        result
    }
    
    /// Get all variadic sockets
    pub fn variadic_sockets(&self) -> Vec<&InputSocket> {
        self.sockets.values()
            .filter(|s| s.is_variadic)
            .collect()
    }
    
    /// Get all greedy variadic sockets
    pub fn greedy_variadic_sockets(&self) -> Vec<&InputSocket> {
        self.sockets.values()
            .filter(|s| s.is_variadic && s.is_greedy)
            .collect()
    }
    
    /// Get all lazy variadic sockets (variadic but not greedy)
    pub fn lazy_variadic_sockets(&self) -> Vec<&InputSocket> {
        self.sockets.values()
            .filter(|s| s.is_variadic && !s.is_greedy)
            .collect()
    }
}

impl Sockets<OutputSocket> {
    /// Create a formatted string that shows all output sockets
    pub fn format_outputs(&self) -> String {
        let mut result = String::from("Outputs:\n");
        for (_, socket) in self.sockets.iter() {
            result.push_str(&format!("  - {}: {}\n", socket.name, socket.type_name));
        }
        result
    }
}

impl fmt::Display for Sockets<InputSocket> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_inputs())
    }
}

impl fmt::Display for Sockets<OutputSocket> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_outputs())
    }
}

impl<T> PartialEq for Sockets<T> 
where 
    T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.sockets == other.sockets
    }
}

/// Manages variadic socket inputs during pipeline execution
#[derive(Debug, Clone)]
pub struct VariadicSocketHandler {
    /// Map from component name to a map of socket name to values
    pending_inputs: HashMap<String, HashMap<String, Vec<(String, Value)>>>,
    
    /// Set of component/socket pairs that have resolved all of their expected inputs
    resolved_variadic_sockets: HashSet<(String, String)>,
}

impl VariadicSocketHandler {
    /// Create a new VariadicSocketHandler
    pub fn new() -> Self {
        Self {
            pending_inputs: HashMap::new(),
            resolved_variadic_sockets: HashSet::new(),
        }
    }
    
    /// Add a pending input for a variadic socket
    pub fn add_pending_input(
        &mut self, 
        component_name: &str, 
        socket_name: &str, 
        source_component: &str, 
        value: Value
    ) {
        let component_inputs = self.pending_inputs
            .entry(component_name.to_string())
            .or_insert_with(HashMap::new);
            
        let socket_inputs = component_inputs
            .entry(socket_name.to_string())
            .or_insert_with(Vec::new);
            
        socket_inputs.push((source_component.to_string(), value));
    }
    
    /// Check if a greedy variadic socket has any pending inputs
    pub fn has_greedy_variadic_input(&self, component_name: &str, socket_name: &str) -> bool {
        self.pending_inputs
            .get(component_name)
            .and_then(|sockets| sockets.get(socket_name))
            .map(|inputs| !inputs.is_empty())
            .unwrap_or(false)
    }
    
    /// Check if a socket already has a pending input from a specific source
    pub fn has_pending_input(&self, component_name: &str, socket_name: &str, source_component: &str) -> bool {
        self.pending_inputs
            .get(component_name)
            .and_then(|sockets| sockets.get(socket_name))
            .map(|inputs| inputs.iter().any(|(src, _)| src == source_component))
            .unwrap_or(false)
    }
    
    /// Check if a lazy variadic socket has collected all expected inputs
    pub fn has_lazy_variadic_socket_received_all_inputs(
        &self, 
        component_name: &str, 
        socket_name: &str,
        expected_senders: &[String]
    ) -> bool {
        // If the socket is already resolved, return true
        if self.resolved_variadic_sockets.contains(&(component_name.to_string(), socket_name.to_string())) {
            return true;
        }
        
        // Get the current senders
        let current_senders = self.pending_inputs
            .get(component_name)
            .and_then(|sockets| sockets.get(socket_name))
            .map(|inputs| inputs.iter().map(|(sender, _)| sender.clone()).collect::<HashSet<String>>())
            .unwrap_or_else(HashSet::new);
            
        // Check if all expected senders have provided inputs
        let expected_senders_set: HashSet<String> = expected_senders.iter().cloned().collect();
        expected_senders_set.is_subset(&current_senders)
    }
    
    /// Mark a lazy variadic socket as resolved
    pub fn mark_socket_resolved(&mut self, component_name: &str, socket_name: &str) {
        self.resolved_variadic_sockets.insert((component_name.to_string(), socket_name.to_string()));
    }
    
    /// Consume inputs for a variadic socket and convert them to the appropriate type
    pub fn consume_variadic_inputs(
        &mut self, 
        component_name: &str, 
        socket_name: &str,
        socket_type: SocketType
    ) -> Result<Value> {
        // Get all pending inputs for this socket
        let inputs_opt = self.pending_inputs
            .get_mut(component_name)
            .and_then(|sockets| sockets.get_mut(socket_name));
            
        if inputs_opt.is_none() || inputs_opt.as_ref().unwrap().is_empty() {
            println!("No pending inputs found for variadic socket {}.{}", component_name, socket_name);
            // Return an empty array for variadic inputs when none are available
            return Ok(Value::Array(Vec::new()));
        }
        
        let inputs = inputs_opt.unwrap();
        
        // Create the variadic value based on socket type
        let result = match socket_type {
            SocketType::Regular => {
                return Err(anyhow!("Cannot consume variadic inputs for regular socket {}.{}", component_name, socket_name));
            },
            SocketType::Variadic => {
                // For lazy variadic, collect all inputs
                let mut values = Vec::new();
                
                // Process each input value
                for (_, value) in inputs.iter() {
                    println!("Processing variadic input: {:?}", value);
                    
                    // If the value is already an array, extract its items
                    if value.is_array() {
                        let array = value.as_array().unwrap();
                        for item in array {
                            values.push(item.clone());
                        }
                    } else {
                        // Otherwise add the value directly
                        values.push(value.clone());
                    }
                }
                
                // Mark the socket as consumed
                self.mark_socket_resolved(component_name, socket_name);
                
                println!("Collected variadic values: {:?}", values);
                
                // Create a JSON array with the collected values
                Value::Array(values)
            },
            SocketType::GreedyVariadic => {
                // For greedy variadic, take the first available input
                if inputs.is_empty() {
                    return Err(anyhow!("No inputs available for greedy variadic socket {}.{}", component_name, socket_name));
                }
                
                // Pop the first input
                let (_, value) = inputs.remove(0);
                
                // Handle array values
                if value.is_array() {
                    value
                } else {
                    // Create a JSON array with just this value
                    Value::Array(vec![value])
                }
            }
        };
        
        Ok(result)
    }
    
    /// Check if all lazy variadic sockets for a component have received all their inputs
    pub fn are_all_lazy_variadic_sockets_resolved(
        &self, 
        component_name: &str, 
        sockets: &HashMap<String, InputSocket>
    ) -> bool {
        for (socket_name, socket) in sockets {
            if socket.is_variadic && !socket.is_greedy {
                if !socket.senders.is_empty() && !self.has_lazy_variadic_socket_received_all_inputs(
                    component_name, 
                    socket_name, 
                    &socket.senders
                ) {
                    return false;
                }
            }
        }
        true
    }
    
    /// Reset after a component has been run
    pub fn reset_component(&mut self, component_name: &str) {
        // Remove pending inputs for this component
        self.pending_inputs.remove(component_name);
        
        // Remove resolved sockets for this component
        self.resolved_variadic_sockets.retain(|(comp, _)| comp != component_name);
    }
    
    /// For debugging: get the current state of pending inputs
    pub fn get_pending_inputs_debug(&self) -> HashMap<String, HashMap<String, usize>> {
        let mut result = HashMap::new();
        
        for (component, sockets) in &self.pending_inputs {
            let mut socket_counts = HashMap::new();
            for (socket, inputs) in sockets {
                socket_counts.insert(socket.clone(), inputs.len());
            }
            result.insert(component.clone(), socket_counts);
        }
        
        result
    }
}

impl Default for VariadicSocketHandler {
    fn default() -> Self {
        Self::new()
    }
}