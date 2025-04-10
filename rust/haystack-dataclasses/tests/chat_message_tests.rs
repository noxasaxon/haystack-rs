use std::collections::HashMap;
use serde_json::json;
use haystack_dataclasses::{ChatMessage, ChatRole, ToolCall};

#[test]
fn test_chat_role_conversions() {
    assert_eq!(ChatRole::from_str("user").unwrap(), ChatRole::User);
    assert_eq!(ChatRole::from_str("system").unwrap(), ChatRole::System);
    assert_eq!(ChatRole::from_str("assistant").unwrap(), ChatRole::Assistant);
    assert_eq!(ChatRole::from_str("tool").unwrap(), ChatRole::Tool);
    
    // Invalid role should return error
    assert!(ChatRole::from_str("invalid").is_err());
    
    // String conversions
    assert_eq!(ChatRole::User.as_str(), "user");
    assert_eq!(ChatRole::System.as_str(), "system");
    assert_eq!(ChatRole::Assistant.as_str(), "assistant");
    assert_eq!(ChatRole::Tool.as_str(), "tool");
}

#[test]
fn test_chat_message_from_user() {
    let message = ChatMessage::from_user("Hello, I need help", None, None);
    
    assert_eq!(message.role(), &ChatRole::User);
    assert_eq!(message.text().unwrap(), "Hello, I need help");
    assert!(message.name().is_none());
    assert!(message.meta().is_empty());
    assert!(message.tool_calls().is_empty());
    assert!(message.tool_call_results().is_empty());
    assert!(message.is_from(&ChatRole::User));
    assert!(!message.is_from(&ChatRole::Assistant));
}

#[test]
fn test_chat_message_from_system() {
    let meta = Some(HashMap::from([
        ("key".to_string(), json!("value")),
    ]));
    
    let message = ChatMessage::from_system(
        "You are a helpful assistant",
        meta,
        Some("System".to_string()),
    );
    
    assert_eq!(message.role(), &ChatRole::System);
    assert_eq!(message.text().unwrap(), "You are a helpful assistant");
    assert_eq!(message.name().unwrap(), "System");
    assert_eq!(message.meta().len(), 1);
    assert_eq!(message.meta()["key"], json!("value"));
    assert!(message.is_from(&ChatRole::System));
}

#[test]
fn test_chat_message_from_assistant_with_text() {
    let message = ChatMessage::from_assistant(
        Some("I'll help you with that"),
        None,
        None,
        None,
    );
    
    assert_eq!(message.role(), &ChatRole::Assistant);
    assert_eq!(message.text().unwrap(), "I'll help you with that");
    assert!(message.tool_calls().is_empty());
}

#[test]
fn test_chat_message_from_assistant_with_tool_calls() {
    let tool_calls = vec![
        ToolCall {
            tool_name: "calculator".to_string(),
            arguments: HashMap::from([
                ("expression".to_string(), json!("2+2")),
            ]),
            id: Some("tool-call-1".to_string()),
        },
        ToolCall {
            tool_name: "weather".to_string(),
            arguments: HashMap::from([
                ("location".to_string(), json!("New York")),
            ]),
            id: Some("tool-call-2".to_string()),
        },
    ];
    
    let message = ChatMessage::from_assistant(
        Some("Let me calculate that and check the weather"),
        None,
        None,
        Some(tool_calls),
    );
    
    assert_eq!(message.role(), &ChatRole::Assistant);
    assert_eq!(message.text().unwrap(), "Let me calculate that and check the weather");
    assert_eq!(message.tool_calls().len(), 2);
    assert_eq!(message.tool_calls()[0].tool_name, "calculator");
    assert_eq!(message.tool_call().unwrap().tool_name, "calculator");
}

#[test]
fn test_chat_message_from_tool() {
    let tool_call = ToolCall {
        tool_name: "calculator".to_string(),
        arguments: HashMap::from([
            ("expression".to_string(), json!("2+2")),
        ]),
        id: Some("tool-call-1".to_string()),
    };
    
    let message = ChatMessage::from_tool(
        "The result is 4",
        tool_call,
        false,
        None,
    );
    
    assert_eq!(message.role(), &ChatRole::Tool);
    assert!(message.text().is_none());
    assert_eq!(message.tool_call_results().len(), 1);
    assert_eq!(message.tool_call_result().unwrap().result, "The result is 4");
    assert!(!message.tool_call_result().unwrap().error);
}

#[test]
fn test_chat_message_to_dict() {
    let message = ChatMessage::from_user("Hello, I need help", None, None);
    let dict = message.to_dict().unwrap();
    
    assert!(dict.is_object());
    let obj = dict.as_object().unwrap();
    
    assert_eq!(obj["role"].as_str().unwrap(), "user");
    assert!(obj.contains_key("content"));
    let content = obj["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].as_object().unwrap()["text"].as_str().unwrap(), "Hello, I need help");
}

#[test]
fn test_chat_message_from_dict() {
    let dict = json!({
        "role": "user",
        "content": [
            {
                "text": "Hello, I need help"
            }
        ],
        "name": null,
        "meta": {}
    });
    
    let message = ChatMessage::from_dict(&dict).unwrap();
    
    assert_eq!(message.role(), &ChatRole::User);
    assert_eq!(message.text().unwrap(), "Hello, I need help");
    assert!(message.name().is_none());
    assert!(message.meta().is_empty());
}

#[test]
fn test_chat_message_from_dict_legacy_format() {
    // Test pre-2.9.0 format (content as string)
    let dict = json!({
        "role": "user",
        "content": "Hello, I need help",
        "name": null,
        "meta": {}
    });
    
    let message = ChatMessage::from_dict(&dict).unwrap();
    
    assert_eq!(message.role(), &ChatRole::User);
    assert_eq!(message.text().unwrap(), "Hello, I need help");
}

#[test]
fn test_chat_message_to_openai_dict_format() {
    // User message
    let user_message = ChatMessage::from_user("Hello", None, None);
    let user_dict = user_message.to_openai_dict_format().unwrap();
    
    assert_eq!(user_dict["role"].as_str().unwrap(), "user");
    assert_eq!(user_dict["content"].as_str().unwrap(), "Hello");
    
    // Assistant message with tool calls
    let tool_calls = vec![
        ToolCall {
            tool_name: "calculator".to_string(),
            arguments: HashMap::from([
                ("expression".to_string(), json!("2+2")),
            ]),
            id: Some("tool-call-1".to_string()),
        },
    ];
    
    let assistant_message = ChatMessage::from_assistant(
        Some("Let me calculate that"),
        None,
        None,
        Some(tool_calls),
    );
    
    let assistant_dict = assistant_message.to_openai_dict_format().unwrap();
    
    assert_eq!(assistant_dict["role"].as_str().unwrap(), "assistant");
    assert_eq!(assistant_dict["content"].as_str().unwrap(), "Let me calculate that");
    assert!(assistant_dict.as_object().unwrap().contains_key("tool_calls"));
    let openai_tool_calls = assistant_dict["tool_calls"].as_array().unwrap();
    assert_eq!(openai_tool_calls.len(), 1);
    assert_eq!(openai_tool_calls[0]["function"]["name"].as_str().unwrap(), "calculator");
}

#[test]
fn test_chat_message_from_openai_dict_format() {
    // User message
    let user_dict = json!({
        "role": "user",
        "content": "Hello"
    });
    
    let user_message = ChatMessage::from_openai_dict_format(&user_dict).unwrap();
    
    assert_eq!(user_message.role(), &ChatRole::User);
    assert_eq!(user_message.text().unwrap(), "Hello");
    
    // Assistant message with tool calls
    let assistant_dict = json!({
        "role": "assistant",
        "content": "Let me calculate that",
        "tool_calls": [
            {
                "id": "tool-call-1",
                "type": "function",
                "function": {
                    "name": "calculator",
                    "arguments": "{\"expression\":\"2+2\"}"
                }
            }
        ]
    });
    
    let assistant_message = ChatMessage::from_openai_dict_format(&assistant_dict).unwrap();
    
    assert_eq!(assistant_message.role(), &ChatRole::Assistant);
    assert_eq!(assistant_message.text().unwrap(), "Let me calculate that");
    assert_eq!(assistant_message.tool_calls().len(), 1);
    assert_eq!(assistant_message.tool_calls()[0].tool_name, "calculator");
    assert_eq!(assistant_message.tool_calls()[0].arguments["expression"], json!("2+2"));
}

#[test]
fn test_chat_message_display() {
    // User message
    let user_message = ChatMessage::from_user("Hello", None, None);
    let user_display = format!("{}", user_message);
    
    assert!(user_display.contains("user"));
    assert!(user_display.contains("Hello"));
    
    // Assistant message with tool calls
    let tool_calls = vec![
        ToolCall {
            tool_name: "calculator".to_string(),
            arguments: HashMap::from([
                ("expression".to_string(), json!("2+2")),
            ]),
            id: Some("tool-call-1".to_string()),
        },
    ];
    
    let assistant_message = ChatMessage::from_assistant(
        Some("Let me calculate that"),
        None,
        None,
        Some(tool_calls),
    );
    
    let assistant_display = format!("{}", assistant_message);
    
    assert!(assistant_display.contains("assistant"));
    assert!(assistant_display.contains("Let me calculate that"));
    assert!(assistant_display.contains("tool call"));
}