use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Represents the state in a pipeline or component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    /// Internal storage for values
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

impl State {
    /// Create a new empty State
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    /// Check if the state contains a specific key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    /// Get a value from the state
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }
    
    /// Set a value in the state
    pub fn set<T>(&mut self, key: &str, value: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;
        self.data.insert(key.to_string(), value);
        Ok(())
    }
    
    /// Remove a value from the state
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }
    
    /// Get all data in the state
    pub fn data(&self) -> &HashMap<String, Value> {
        &self.data
    }
    
    /// Clone the state
    pub fn clone_state(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
    
    /// Merge another state into this one
    pub fn merge(&mut self, other: &State) {
        for (key, value) in &other.data {
            self.data.insert(key.clone(), value.clone());
        }
    }
}

impl std::ops::Index<&str> for State {
    type Output = Value;
    
    fn index(&self, key: &str) -> &Self::Output {
        &self.data[key]
    }
}

impl std::ops::IndexMut<&str> for State {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        self.data.entry(key.to_string()).or_insert(Value::Null)
    }
}