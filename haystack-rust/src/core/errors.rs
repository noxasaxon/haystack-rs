use std::fmt;

/// Base error type for pipeline-related errors
#[derive(Debug)]
pub struct PipelineError {
    message: String,
}

impl PipelineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineError {}

/// Error that occurs during pipeline runtime
#[derive(Debug)]
pub struct PipelineRuntimeError {
    pub component_name: Option<String>,
    pub component_type: Option<String>,
    pub message: String,
}

impl PipelineRuntimeError {
    pub fn new(component_name: Option<String>, component_type: Option<String>, message: impl Into<String>) -> Self {
        Self {
            component_name,
            component_type,
            message: message.into(),
        }
    }

    /// Create a PipelineRuntimeError from an error
    pub fn from_error(component_name: &str, component_type: &str, error: &dyn std::error::Error) -> Self {
        let message = format!(
            "The following component failed to run:\n\
            Component name: '{}'\n\
            Component type: '{}'\n\
            Error: {}",
            component_name, component_type, error
        );
        Self::new(Some(component_name.to_string()), Some(component_type.to_string()), message)
    }

    /// Create a PipelineRuntimeError from an invalid output
    pub fn from_invalid_output(component_name: &str, component_type: &str, output_type: &str) -> Self {
        let message = format!(
            "The following component returned an invalid output:\n\
            Component name: '{}'\n\
            Component type: '{}'\n\
            Expected a dictionary, but got {} instead.\n\
            Check the component's output and ensure it is a valid dictionary.",
            component_name, component_type, output_type
        );
        Self::new(Some(component_name.to_string()), Some(component_type.to_string()), message)
    }
}

impl fmt::Display for PipelineRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineRuntimeError {}

/// Error that occurs when all components in a pipeline are blocked
#[derive(Debug)]
pub struct PipelineComponentsBlockedError {
    pub message: String,
}

impl PipelineComponentsBlockedError {
    pub fn new() -> Self {
        let message = "Cannot run pipeline - all components are blocked. \
                      This typically happens when:\n\
                      1. There is no valid entry point for the pipeline\n\
                      2. There is a circular dependency preventing the pipeline from running\n\
                      Check the connections between these components and ensure all required inputs are provided.";
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for PipelineComponentsBlockedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineComponentsBlockedError {}

/// Error that occurs when connecting pipeline components
#[derive(Debug)]
pub struct PipelineConnectError {
    pub message: String,
}

impl PipelineConnectError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineConnectError {}

/// Error that occurs during pipeline validation
#[derive(Debug)]
pub struct PipelineValidationError {
    pub message: String,
}

impl PipelineValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineValidationError {}

/// Error that occurs during pipeline drawing
#[derive(Debug)]
pub struct PipelineDrawingError {
    pub message: String,
}

impl PipelineDrawingError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineDrawingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineDrawingError {}

/// Error that occurs when the maximum number of component runs is exceeded
#[derive(Debug)]
pub struct PipelineMaxComponentRuns {
    pub message: String,
}

impl PipelineMaxComponentRuns {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineMaxComponentRuns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineMaxComponentRuns {}

/// Error that occurs during pipeline unmarshalling
#[derive(Debug)]
pub struct PipelineUnmarshalError {
    pub message: String,
}

impl PipelineUnmarshalError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineUnmarshalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PipelineUnmarshalError {}

/// Base error type for component-related errors
#[derive(Debug)]
pub struct ComponentError {
    pub message: String,
}

impl ComponentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ComponentError {}

/// Error that occurs during component deserialization
#[derive(Debug)]
pub struct ComponentDeserializationError {
    pub message: String,
}

impl ComponentDeserializationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ComponentDeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ComponentDeserializationError {}

/// Error that occurs during deserialization
#[derive(Debug)]
pub struct DeserializationError {
    pub message: String,
}

impl DeserializationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DeserializationError {}

/// Error that occurs during serialization
#[derive(Debug)]
pub struct SerializationError {
    pub message: String,
}

impl SerializationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SerializationError {}