/*!
 * Socket management for components
 */

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

use super::types::{InputSocket, OutputSocket};

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