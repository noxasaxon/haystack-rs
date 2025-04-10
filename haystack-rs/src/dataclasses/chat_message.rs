use std::collections::HashMap;
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Enumeration representing the roles within a chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatRole {
    /// The user role. A message from the user contains only text.
    #[serde(rename = "user")]
    User,
    
    /// The system role. A message from the system contains only text.
    #[serde(rename = "system")]
    System,
    
    /// The assistant role. A message from the assistant can contain text and Tool calls. 
    /// It can also store metadata.
    #[serde(rename = "assistant")]
    Assistant,
    
    /// The tool role. A message from a tool contains the result of a Tool invocation.
    #[serde(rename = "tool")]
    Tool,
}

impl ChatRole {
    /// Convert a string to a ChatRole enum
    pub fn from_str(role: &str) -> Result<Self> {
        match role {
            "user" => Ok(ChatRole::User),
            "system" => Ok(ChatRole::System),
            "assistant" => Ok(ChatRole::Assistant),
            "tool" => Ok(ChatRole::Tool),
            _ => Err(anyhow!("Unknown chat role '{}'. Supported roles are: [user, system, assistant, tool]", role)),
        }
    }
    
    /// Get the string value of the role
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::System => "system",
            ChatRole::Assistant => "assistant",
            ChatRole::Tool => "tool",
        }
    }
}

/// Represents a Tool call prepared by the model, usually contained in an assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The name of the Tool to call
    pub tool_name: String,
    
    /// The arguments to call the Tool with
    pub arguments: HashMap<String, Value>,
    
    /// The ID of the Tool call
    pub id: Option<String>,
}

/// Represents the result of a Tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// The result of the Tool invocation
    pub result: String,
    
    /// The Tool call that produced this result
    pub origin: ToolCall,
    
    /// Whether the Tool invocation resulted in an error
    pub error: bool,
}

/// The textual content of a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    /// The text content of the message
    pub text: String,
}

/// Represents the content of a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatMessageContent {
    /// Text content
    #[serde(rename = "text")]
    Text(TextContent),
    
    /// Tool call
    #[serde(rename = "tool_call")]
    ToolCall(ToolCall),
    
    /// Tool call result
    #[serde(rename = "tool_call_result")]
    ToolCallResult(ToolCallResult),
}

/// Represents a message in a LLM chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the entity sending the message
    #[serde(rename = "_role")]
    role: ChatRole,
    
    /// The content of the message
    #[serde(rename = "_content")]
    content: Vec<ChatMessageContent>,
    
    /// An optional name for the participant
    #[serde(rename = "_name")]
    name: Option<String>,
    
    /// Additional metadata associated with the message
    #[serde(rename = "_meta", default)]
    meta: HashMap<String, Value>,
}

impl ChatMessage {
    /// Returns the role of the entity sending the message
    pub fn role(&self) -> &ChatRole {
        &self.role
    }
    
    /// Returns the metadata associated with the message
    pub fn meta(&self) -> &HashMap<String, Value> {
        &self.meta
    }
    
    /// Returns the name associated with the message
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    
    /// Returns the list of all texts contained in the message
    pub fn texts(&self) -> Vec<String> {
        self.content.iter()
            .filter_map(|c| match c {
                ChatMessageContent::Text(tc) => Some(tc.text.clone()),
                _ => None,
            })
            .collect()
    }
    
    /// Returns the first text contained in the message
    pub fn text(&self) -> Option<String> {
        let texts = self.texts();
        if !texts.is_empty() {
            Some(texts[0].clone())
        } else {
            None
        }
    }
    
    /// Returns the list of all Tool calls contained in the message
    pub fn tool_calls(&self) -> Vec<&ToolCall> {
        self.content.iter()
            .filter_map(|c| match c {
                ChatMessageContent::ToolCall(tc) => Some(tc),
                _ => None,
            })
            .collect()
    }
    
    /// Returns the first Tool call contained in the message
    pub fn tool_call(&self) -> Option<&ToolCall> {
        let tool_calls = self.tool_calls();
        if !tool_calls.is_empty() {
            Some(tool_calls[0])
        } else {
            None
        }
    }
    
    /// Returns the list of all Tool call results contained in the message
    pub fn tool_call_results(&self) -> Vec<&ToolCallResult> {
        self.content.iter()
            .filter_map(|c| match c {
                ChatMessageContent::ToolCallResult(tcr) => Some(tcr),
                _ => None,
            })
            .collect()
    }
    
    /// Returns the first Tool call result contained in the message
    pub fn tool_call_result(&self) -> Option<&ToolCallResult> {
        let tool_call_results = self.tool_call_results();
        if !tool_call_results.is_empty() {
            Some(tool_call_results[0])
        } else {
            None
        }
    }
    
    /// Check if the message is from a specific role
    pub fn is_from(&self, role: &ChatRole) -> bool {
        &self.role == role
    }
    
    /// Create a message from the user
    pub fn from_user(text: &str, meta: Option<HashMap<String, Value>>, name: Option<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: vec![ChatMessageContent::Text(TextContent { text: text.to_string() })],
            meta: meta.unwrap_or_default(),
            name,
        }
    }
    
    /// Create a message from the system
    pub fn from_system(text: &str, meta: Option<HashMap<String, Value>>, name: Option<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: vec![ChatMessageContent::Text(TextContent { text: text.to_string() })],
            meta: meta.unwrap_or_default(),
            name,
        }
    }
    
    /// Create a message from the assistant
    pub fn from_assistant(
        text: Option<&str>,
        meta: Option<HashMap<String, Value>>,
        name: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    ) -> Self {
        let mut content = Vec::new();
        
        if let Some(t) = text {
            content.push(ChatMessageContent::Text(TextContent { text: t.to_string() }));
        }
        
        if let Some(tc) = tool_calls {
            for tool_call in tc {
                content.push(ChatMessageContent::ToolCall(tool_call));
            }
        }
        
        Self {
            role: ChatRole::Assistant,
            content,
            meta: meta.unwrap_or_default(),
            name,
        }
    }
    
    /// Create a message from a Tool
    pub fn from_tool(
        tool_result: &str,
        origin: ToolCall,
        error: bool,
        meta: Option<HashMap<String, Value>>,
    ) -> Self {
        Self {
            role: ChatRole::Tool,
            content: vec![ChatMessageContent::ToolCallResult(ToolCallResult {
                result: tool_result.to_string(),
                origin,
                error,
            })],
            meta: meta.unwrap_or_default(),
            name: None,
        }
    }
    
    /// Converts ChatMessage into a dictionary
    pub fn to_dict(&self) -> Result<Value> {
        let mut serialized = serde_json::Map::new();
        
        serialized.insert("role".to_string(), Value::String(self.role.as_str().to_string()));
        serialized.insert("meta".to_string(), serde_json::to_value(&self.meta)?);
        serialized.insert("name".to_string(), serde_json::to_value(&self.name)?);
        
        let mut content = Vec::new();
        for part in &self.content {
            match part {
                ChatMessageContent::Text(tc) => {
                    let mut item = serde_json::Map::new();
                    item.insert("text".to_string(), Value::String(tc.text.clone()));
                    content.push(Value::Object(item));
                },
                ChatMessageContent::ToolCall(tc) => {
                    let mut item = serde_json::Map::new();
                    item.insert("tool_call".to_string(), serde_json::to_value(tc)?);
                    content.push(Value::Object(item));
                },
                ChatMessageContent::ToolCallResult(tcr) => {
                    let mut item = serde_json::Map::new();
                    item.insert("tool_call_result".to_string(), serde_json::to_value(tcr)?);
                    content.push(Value::Object(item));
                },
            }
        }
        
        serialized.insert("content".to_string(), Value::Array(content));
        
        Ok(Value::Object(serialized))
    }
    
    /// Creates a new ChatMessage object from a dictionary
    pub fn from_dict(data: &Value) -> Result<Self> {
        let data_obj = data.as_object()
            .ok_or_else(|| anyhow!("Expected an object for ChatMessage deserialization"))?;
        
        if let Some(content_value) = data_obj.get("content") {
            let role = match data_obj.get("role") {
                Some(r) => ChatRole::from_str(r.as_str()
                    .ok_or_else(|| anyhow!("'role' must be a string"))?
                )?,
                None => return Err(anyhow!("Missing 'role' in serialized ChatMessage")),
            };
            
            let name = match data_obj.get("name") {
                Some(Value::String(s)) => Some(s.clone()),
                Some(Value::Null) => None,
                None => None,
                _ => return Err(anyhow!("'name' must be a string or null")),
            };
            
            let meta = match data_obj.get("meta") {
                Some(m) => serde_json::from_value(m.clone())
                    .context("Failed to deserialize 'meta'")?,
                None => HashMap::new(),
            };
            
            let content = if let Some(content_array) = content_value.as_array() {
                // Current format - the serialized 'content' field is a list of dictionaries
                let mut result = Vec::new();
                
                for part in content_array {
                    let part_obj = part.as_object()
                        .ok_or_else(|| anyhow!("Expected an object for content part"))?;
                    
                    if let Some(text_value) = part_obj.get("text") {
                        let text = text_value.as_str()
                            .ok_or_else(|| anyhow!("'text' must be a string"))?;
                        result.push(ChatMessageContent::Text(TextContent { text: text.to_string() }));
                    } else if let Some(tool_call_value) = part_obj.get("tool_call") {
                        let tool_call: ToolCall = serde_json::from_value(tool_call_value.clone())
                            .context("Failed to deserialize 'tool_call'")?;
                        result.push(ChatMessageContent::ToolCall(tool_call));
                    } else if let Some(tool_call_result_value) = part_obj.get("tool_call_result") {
                        let tool_call_result: ToolCallResult = serde_json::from_value(tool_call_result_value.clone())
                            .context("Failed to deserialize 'tool_call_result'")?;
                        result.push(ChatMessageContent::ToolCallResult(tool_call_result));
                    } else {
                        return Err(anyhow!("Unsupported part in serialized ChatMessage: {:?}", part));
                    }
                }
                
                result
            } else if let Some(content_str) = content_value.as_str() {
                // Pre 2.9.0 format - the 'content' field is a string
                vec![ChatMessageContent::Text(TextContent { text: content_str.to_string() })]
            } else {
                return Err(anyhow!("Unsupported content type in serialized ChatMessage"));
            };
            
            return Ok(Self {
                role,
                content,
                name,
                meta,
            });
        } else if let Some(_content_value) = data_obj.get("_content") {
            // Format for versions >=2.9.0 and <2.12.0
            let obj = serde_json::from_value::<ChatMessage>(data.clone())
                .context("Failed to deserialize ChatMessage")?;
            return Ok(obj);
        }
        
        Err(anyhow!("Missing 'content' or '_content' in serialized ChatMessage"))
    }
    
    /// Convert a ChatMessage to the dictionary format expected by OpenAI's Chat API
    pub fn to_openai_dict_format(&self) -> Result<Value> {
        let text_contents = self.texts();
        let tool_calls = self.tool_calls();
        let tool_call_results = self.tool_call_results();
        
        if text_contents.is_empty() && tool_calls.is_empty() && tool_call_results.is_empty() {
            return Err(anyhow!("A `ChatMessage` must contain at least one `TextContent`, `ToolCall`, or `ToolCallResult`."));
        }
        
        if text_contents.len() + tool_call_results.len() > 1 {
            return Err(anyhow!("A `ChatMessage` can only contain one `TextContent` or one `ToolCallResult`."));
        }
        
        let mut openai_msg = serde_json::Map::new();
        openai_msg.insert("role".to_string(), Value::String(self.role.as_str().to_string()));
        
        // Add name field if present
        if let Some(name) = &self.name {
            openai_msg.insert("name".to_string(), Value::String(name.clone()));
        }
        
        if !tool_call_results.is_empty() {
            let result = tool_call_results[0];
            if result.origin.id.is_none() {
                return Err(anyhow!("`ToolCall` must have a non-null `id` attribute to be used with OpenAI."));
            }
            
            openai_msg.insert("content".to_string(), Value::String(result.result.clone()));
            openai_msg.insert("tool_call_id".to_string(), 
                Value::String(result.origin.id.as_ref().unwrap().clone()));
            // OpenAI does not provide a way to communicate errors in tool invocations, so we ignore the error field
            
            return Ok(Value::Object(openai_msg));
        }
        
        if !text_contents.is_empty() {
            openai_msg.insert("content".to_string(), Value::String(text_contents[0].clone()));
        }
        
        if !tool_calls.is_empty() {
            let mut openai_tool_calls = Vec::new();
            
            for tc in tool_calls {
                if tc.id.is_none() {
                    return Err(anyhow!("`ToolCall` must have a non-null `id` attribute to be used with OpenAI."));
                }
                
                let mut function = serde_json::Map::new();
                function.insert("name".to_string(), Value::String(tc.tool_name.clone()));
                function.insert("arguments".to_string(), 
                    Value::String(serde_json::to_string(&tc.arguments)?));
                
                let mut tool_call = serde_json::Map::new();
                tool_call.insert("id".to_string(), 
                    Value::String(tc.id.as_ref().unwrap().clone()));
                tool_call.insert("type".to_string(), Value::String("function".to_string()));
                tool_call.insert("function".to_string(), Value::Object(function));
                
                openai_tool_calls.push(Value::Object(tool_call));
            }
            
            openai_msg.insert("tool_calls".to_string(), Value::Array(openai_tool_calls));
        }
        
        Ok(Value::Object(openai_msg))
    }
    
    /// Validate that a message dictionary follows OpenAI's Chat API format
    fn validate_openai_message(message: &Value) -> Result<()> {
        let message_obj = message.as_object()
            .ok_or_else(|| anyhow!("Message must be an object"))?;
        
        let role = match message_obj.get("role") {
            Some(Value::String(r)) => r,
            _ => return Err(anyhow!("The `role` field is required in the message dictionary.")),
        };
        
        let content = message_obj.get("content");
        let tool_calls = message_obj.get("tool_calls");
        
        if !["assistant", "user", "system", "developer", "tool"].contains(&role.as_str()) {
            return Err(anyhow!("Unsupported role: {}", role));
        }
        
        if role == "assistant" {
            if content.is_none() && tool_calls.is_none() {
                return Err(anyhow!("For assistant messages, either `content` or `tool_calls` must be present."));
            }
            
            if let Some(tc_array) = tool_calls.and_then(|tc| tc.as_array()) {
                for tc in tc_array {
                    if tc.get("function").is_none() {
                        return Err(anyhow!("Tool calls must contain the `function` field"));
                    }
                }
            }
        } else if content.is_none() {
            return Err(anyhow!("The `content` field is required for {} messages.", role));
        }
        
        Ok(())
    }
    
    /// Create a ChatMessage from a dictionary in the format expected by OpenAI's Chat API
    pub fn from_openai_dict_format(message: &Value) -> Result<Self> {
        Self::validate_openai_message(message)?;
        
        let message_obj = message.as_object().unwrap(); // Safe because we validated above
        
        let role = message_obj.get("role").unwrap().as_str().unwrap(); // Safe because we validated above
        let content = message_obj.get("content").and_then(|c| c.as_str()).map(|s| s.to_string());
        let name = message_obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
        let tool_calls = message_obj.get("tool_calls").and_then(|tc| tc.as_array());
        let tool_call_id = message_obj.get("tool_call_id").and_then(|id| id.as_str()).map(|s| s.to_string());
        
        if role == "assistant" {
            let mut haystack_tool_calls = None;
            
            if let Some(tc_array) = tool_calls {
                let mut calls = Vec::new();
                
                for tc in tc_array {
                    let tc_obj = tc.as_object()
                        .ok_or_else(|| anyhow!("Tool call must be an object"))?;
                    
                    let function = tc_obj.get("function")
                        .ok_or_else(|| anyhow!("Tool call must contain 'function'"))?
                        .as_object()
                        .ok_or_else(|| anyhow!("Function must be an object"))?;
                    
                    let id = tc_obj.get("id").and_then(|id| id.as_str()).map(|s| s.to_string());
                    let name = function.get("name")
                        .ok_or_else(|| anyhow!("Function must have a 'name'"))?
                        .as_str()
                        .ok_or_else(|| anyhow!("Function name must be a string"))?
                        .to_string();
                    
                    let arguments_str = function.get("arguments")
                        .ok_or_else(|| anyhow!("Function must have 'arguments'"))?
                        .as_str()
                        .ok_or_else(|| anyhow!("Function arguments must be a string"))?;
                    
                    let arguments: HashMap<String, Value> = serde_json::from_str(arguments_str)
                        .context("Failed to parse arguments JSON")?;
                    
                    calls.push(ToolCall {
                        id,
                        tool_name: name,
                        arguments,
                    });
                }
                
                haystack_tool_calls = Some(calls);
            }
            
            let content_str = content.as_deref();
            return Ok(Self::from_assistant(content_str, None, name, haystack_tool_calls));
        }
        
        // content is guaranteed to be Some for non-assistant roles because of validation
        let content_str = content.unwrap();
        
        if role == "user" {
            return Ok(Self::from_user(&content_str, None, name));
        }
        
        if role == "system" || role == "developer" {
            return Ok(Self::from_system(&content_str, None, name));
        }
        
        // role is "tool"
        Ok(Self::from_tool(
            &content_str,
            ToolCall {
                id: tool_call_id,
                tool_name: "".to_string(),
                arguments: HashMap::new(),
            },
            false,
            None,
        ))
    }
}

impl std::fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role = self.role.as_str();
        let texts = self.texts();
        let text = if !texts.is_empty() {
            if texts[0].len() > 30 {
                format!("\"{}...\"", &texts[0][..30])
            } else {
                format!("\"{}\"", texts[0])
            }
        } else {
            "".to_string()
        };
        
        let tool_calls = self.tool_calls();
        let tool_calls_info = if !tool_calls.is_empty() {
            format!(" with {} tool call(s)", tool_calls.len())
        } else {
            "".to_string()
        };
        
        let tool_results = self.tool_call_results();
        let tool_results_info = if !tool_results.is_empty() {
            " with tool result"
        } else {
            ""
        };
        
        write!(f, "ChatMessage({}{}{}{}", role, 
            if !text.is_empty() { format!(": {}", text) } else { "".to_string() },
            tool_calls_info,
            tool_results_info
        )
    }
}