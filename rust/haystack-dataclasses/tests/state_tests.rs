use serde_json::{json, Value};
use haystack_dataclasses::state::State;

#[test]
fn test_state_creation() {
    let state = State::new();
    assert!(state.data().is_empty());
}

#[test]
fn test_state_set_get() {
    let mut state = State::new();
    
    // Set some values
    state.set("string_key", "string_value").unwrap();
    state.set("integer_key", 42).unwrap();
    state.set("float_key", 3.14).unwrap();
    state.set("boolean_key", true).unwrap();
    state.set("null_key", Value::Null).unwrap();
    
    // Check with contains_key
    assert!(state.contains_key("string_key"));
    assert!(state.contains_key("integer_key"));
    assert!(state.contains_key("float_key"));
    assert!(state.contains_key("boolean_key"));
    assert!(state.contains_key("null_key"));
    assert!(!state.contains_key("nonexistent_key"));
    
    // Check with get
    assert_eq!(state.get("string_key").unwrap().as_str().unwrap(), "string_value");
    assert_eq!(state.get("integer_key").unwrap().as_i64().unwrap(), 42);
    assert_eq!(state.get("float_key").unwrap().as_f64().unwrap(), 3.14);
    assert_eq!(state.get("boolean_key").unwrap().as_bool().unwrap(), true);
    assert!(state.get("null_key").unwrap().is_null());
    assert!(state.get("nonexistent_key").is_none());
}

#[test]
fn test_state_set_complex_types() {
    let mut state = State::new();
    
    // Set array
    let array = vec![1, 2, 3];
    state.set("array_key", array).unwrap();
    
    // Set object
    let object = json!({
        "name": "test",
        "value": 42
    });
    state.set("object_key", object).unwrap();
    
    // Check array
    let array_value = state.get("array_key").unwrap();
    assert!(array_value.is_array());
    let array = array_value.as_array().unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array[0].as_i64().unwrap(), 1);
    assert_eq!(array[1].as_i64().unwrap(), 2);
    assert_eq!(array[2].as_i64().unwrap(), 3);
    
    // Check object
    let object_value = state.get("object_key").unwrap();
    assert!(object_value.is_object());
    let object = object_value.as_object().unwrap();
    assert_eq!(object["name"].as_str().unwrap(), "test");
    assert_eq!(object["value"].as_i64().unwrap(), 42);
}

#[test]
fn test_state_remove() {
    let mut state = State::new();
    
    // Set and remove values
    state.set("key1", "value1").unwrap();
    state.set("key2", "value2").unwrap();
    
    assert!(state.contains_key("key1"));
    assert!(state.contains_key("key2"));
    
    let removed = state.remove("key1");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().as_str().unwrap(), "value1");
    
    assert!(!state.contains_key("key1"));
    assert!(state.contains_key("key2"));
    
    // Remove nonexistent key
    let removed = state.remove("nonexistent_key");
    assert!(removed.is_none());
}

#[test]
fn test_state_clone() {
    let mut original = State::new();
    original.set("key1", "value1").unwrap();
    original.set("key2", 42).unwrap();
    
    let cloned = original.clone_state();
    
    assert!(cloned.contains_key("key1"));
    assert!(cloned.contains_key("key2"));
    assert_eq!(cloned.get("key1").unwrap().as_str().unwrap(), "value1");
    assert_eq!(cloned.get("key2").unwrap().as_i64().unwrap(), 42);
    
    // Verify independent copies
    let mut original = original;
    original.set("key3", "value3").unwrap();
    
    assert!(original.contains_key("key3"));
    assert!(!cloned.contains_key("key3"));
}

#[test]
fn test_state_merge() {
    let mut state1 = State::new();
    state1.set("key1", "value1").unwrap();
    state1.set("key2", "value2").unwrap();
    
    let mut state2 = State::new();
    state2.set("key2", "updated_value").unwrap();
    state2.set("key3", "value3").unwrap();
    
    state1.merge(&state2);
    
    assert_eq!(state1.get("key1").unwrap().as_str().unwrap(), "value1");
    assert_eq!(state1.get("key2").unwrap().as_str().unwrap(), "updated_value"); // Overwritten
    assert_eq!(state1.get("key3").unwrap().as_str().unwrap(), "value3");
}

#[test]
fn test_state_index_operators() {
    let mut state = State::new();
    
    // Set values using index operators
    state["key1"] = json!("value1");
    state["key2"] = json!(42);
    
    // Get values using index operators
    assert_eq!(state["key1"].as_str().unwrap(), "value1");
    assert_eq!(state["key2"].as_i64().unwrap(), 42);
    
    // Note: For non-existent keys, using index operator directly may panic.
    // Instead, we should use get() method to safely access potentially missing keys
    assert!(state.get("nonexistent_key").is_none());
}

#[test]
fn test_state_serialization() {
    let mut state = State::new();
    state.set("key1", "value1").unwrap();
    state.set("key2", 42).unwrap();
    
    // Serialize
    let serialized = serde_json::to_string(&state).unwrap();
    
    // Deserialize
    let deserialized: State = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.get("key1").unwrap().as_str().unwrap(), "value1");
    assert_eq!(deserialized.get("key2").unwrap().as_i64().unwrap(), 42);
}