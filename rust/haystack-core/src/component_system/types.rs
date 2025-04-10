/*!
 * Component socket types
 */

use std::any::TypeId;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// A marker trait that identifies variadic types.
/// 
/// Variadic types can receive multiple inputs from different components.
pub trait IsVariadic {}

/// A marker struct for the Variadic type
/// 
/// Variadic inputs collect all inputs from connected components before the component is run.
#[derive(Debug, Clone)]
pub struct Variadic<T> {
    /// The inner collection of values
    pub values: Vec<T>,
    /// The metadata for this variadic collection
    pub meta: Vec<Value>,
}

impl<T> Variadic<T> {
    /// Create a new empty Variadic
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            meta: Vec::new(),
        }
    }
    
    /// Add a value to the collection
    pub fn push(&mut self, value: T, meta: Option<Value>) {
        self.values.push(value);
        self.meta.push(meta.unwrap_or(Value::Null));
    }
    
    /// Get the number of values
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Get a reference to the underlying values
    pub fn inner(&self) -> &Vec<T> {
        &self.values
    }
    
    /// Get a mutable reference to the underlying values
    pub fn inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.values
    }
    
    /// Get the meta information for a specific index
    pub fn meta_at(&self, index: usize) -> Option<&Value> {
        self.meta.get(index)
    }
}

impl<T> Default for Variadic<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Variadic<T> {
    type Target = Vec<T>;
    
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T> DerefMut for Variadic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<T> From<Vec<T>> for Variadic<T> {
    fn from(values: Vec<T>) -> Self {
        let count = values.len();
        Self {
            values,
            meta: vec![Value::Null; count],
        }
    }
}

impl<T> IntoIterator for Variadic<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Variadic<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl<T> IsVariadic for Variadic<T> {}

/// A marker struct for the GreedyVariadic type
/// 
/// GreedyVariadic is similar to Variadic, but components with GreedyVariadic inputs
/// will be run as soon as they receive at least one input, rather than waiting for
/// all connected inputs.
#[derive(Debug, Clone)]
pub struct GreedyVariadic<T> {
    /// The inner Variadic collection
    inner: Variadic<T>,
}

impl<T> GreedyVariadic<T> {
    /// Create a new empty GreedyVariadic
    pub fn new() -> Self {
        Self {
            inner: Variadic::new(),
        }
    }
    
    /// Add a value to the collection
    pub fn push(&mut self, value: T, meta: Option<Value>) {
        self.inner.push(value, meta);
    }
    
    /// Get the number of values
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Get a reference to the underlying values
    pub fn inner(&self) -> &Vec<T> {
        &self.inner.values
    }
    
    /// Get a mutable reference to the underlying values
    pub fn inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.inner.values
    }
    
    /// Get the meta information for a specific index
    pub fn meta_at(&self, index: usize) -> Option<&Value> {
        self.inner.meta_at(index)
    }
}

impl<T> Default for GreedyVariadic<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for GreedyVariadic<T> {
    type Target = Vec<T>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner.values
    }
}

impl<T> DerefMut for GreedyVariadic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.values
    }
}

impl<T> From<Vec<T>> for GreedyVariadic<T> {
    fn from(values: Vec<T>) -> Self {
        Self {
            inner: Variadic::from(values),
        }
    }
}

impl<T> IntoIterator for GreedyVariadic<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.inner.values.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a GreedyVariadic<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.inner.values.iter()
    }
}

impl<T> IsVariadic for GreedyVariadic<T> {}

/// An enumeration of the different socket input types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// A regular socket that accepts a single input
    /// 
    /// This is the standard socket type that accepts a single input and runs
    /// once that input is available.
    Regular,
    
    /// A variadic socket that collects all inputs before running
    /// 
    /// This socket type collects inputs from all connected components before allowing
    /// the component to run. It's useful for components that need to process
    /// all inputs together, like aggregation operations.
    Variadic,
    
    /// A greedy variadic socket that runs as soon as it receives input
    /// 
    /// This socket type allows a component to run as soon as any input is available,
    /// without waiting for all inputs. It's useful for processing streams of data
    /// where each input can be handled independently.
    GreedyVariadic,
}

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
    
    /// String representation of the internal type (for variadic sockets)
    #[serde(default)]
    pub inner_type_name: Option<String>,
    
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
            inner_type_name: None,
            default_value,
            is_variadic,
            is_greedy,
            senders: Vec::new(),
        }
    }
    
    /// Create a new variadic input socket
    pub fn new_variadic<T: 'static>(
        name: String,
        inner_type_name: String,
        default_value: Option<Value>,
        is_greedy: bool,
    ) -> Self {
        let type_id = if is_greedy {
            std::any::TypeId::of::<GreedyVariadic<T>>()
        } else {
            std::any::TypeId::of::<Variadic<T>>()
        };
        
        let type_name = if is_greedy {
            format!("GreedyVariadic<{}>", inner_type_name)
        } else {
            format!("Variadic<{}>", inner_type_name)
        };
        
        Self {
            name,
            type_id: Some(type_id),
            type_name,
            inner_type_name: Some(inner_type_name),
            default_value,
            is_variadic: true,
            is_greedy,
            senders: Vec::new(),
        }
    }
    
    /// Get the socket type
    pub fn socket_type(&self) -> SocketType {
        if !self.is_variadic {
            SocketType::Regular
        } else if self.is_greedy {
            SocketType::GreedyVariadic
        } else {
            SocketType::Variadic
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
            if let Some(inner_type) = &self.inner_type_name {
                if self.is_greedy {
                    format!("GreedyVariadic<{}>", inner_type)
                } else {
                    format!("Variadic<{}>", inner_type)
                }
            } else {
                self.type_name.clone()
            }
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