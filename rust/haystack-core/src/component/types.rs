/*!
 * Component socket types
 */

use std::any::TypeId;
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// A marker trait that identifies variadic types.
/// 
/// Variadic types can receive multiple inputs from different components.
pub trait IsVariadic {}

/// A marker struct for the Variadic type
pub struct Variadic<T>(pub Vec<T>);

impl<T> IsVariadic for Variadic<T> {}

/// A marker struct for the GreedyVariadic type
/// 
/// GreedyVariadic is similar to Variadic, but components with GreedyVariadic inputs
/// will be run as soon as they receive at least one input, rather than waiting for
/// all connected inputs.
pub struct GreedyVariadic<T>(pub Vec<T>);

impl<T> IsVariadic for GreedyVariadic<T> {}

/// Input socket for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSocket {
    /// Name of the socket
    pub name: String,
    
    /// Type ID of the socket
    #[serde(skip, default)]
    pub type_id: Option<TypeId>,
    
    /// String representation of the type
    pub type_name: String,
    
    /// Default value for the socket, if any
    pub default_value: Option<Value>,
    
    /// Whether this socket accepts multiple connections
    pub is_variadic: bool,
    
    /// Whether this socket is greedy (runs as soon as it receives input)
    pub is_greedy: bool,
    
    /// List of component names that send data to this socket
    #[serde(default)]
    pub senders: Vec<String>,
}

impl InputSocket {
    /// Create a new InputSocket
    pub fn new(
        name: String,
        type_id: TypeId,
        type_name: String,
        default_value: Option<Value>,
        is_variadic: bool,
        is_greedy: bool,
    ) -> Self {
        Self {
            name,
            type_id: Some(type_id),
            type_name,
            default_value,
            is_variadic,
            is_greedy,
            senders: Vec::new(),
        }
    }
    
    /// Check if this socket is mandatory (has no default value)
    pub fn is_mandatory(&self) -> bool {
        self.default_value.is_none()
    }
    
    /// Add a sender to this socket
    pub fn add_sender(&mut self, component_name: String) {
        if !self.senders.contains(&component_name) {
            self.senders.push(component_name);
        }
    }
}

impl fmt::Display for InputSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = if self.is_variadic {
            format!("Variadic[{}]", self.type_name)
        } else {
            self.type_name.clone()
        };
        
        let default_str = if self.default_value.is_some() {
            " (optional)"
        } else {
            ""
        };
        
        write!(f, "{}: {}{}", self.name, type_str, default_str)
    }
}

impl PartialEq for InputSocket {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_name == other.type_name
            && self.default_value == other.default_value
            && self.is_variadic == other.is_variadic
            && self.is_greedy == other.is_greedy
    }
}

/// Output socket for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSocket {
    /// Name of the socket
    pub name: String,
    
    /// Type ID of the socket
    #[serde(skip, default)]
    pub type_id: Option<TypeId>,
    
    /// String representation of the type
    pub type_name: String,
    
    /// List of component names that receive data from this socket
    #[serde(default)]
    pub receivers: Vec<String>,
}

impl OutputSocket {
    /// Create a new OutputSocket
    pub fn new(
        name: String,
        type_id: TypeId,
        type_name: String,
    ) -> Self {
        Self {
            name,
            type_id: Some(type_id),
            type_name,
            receivers: Vec::new(),
        }
    }
    
    /// Add a receiver to this socket
    pub fn add_receiver(&mut self, component_name: String) {
        if !self.receivers.contains(&component_name) {
            self.receivers.push(component_name);
        }
    }
}

impl fmt::Display for OutputSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.type_name)
    }
}

impl PartialEq for OutputSocket {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_name == other.type_name
    }
}