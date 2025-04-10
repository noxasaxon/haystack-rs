/*!
 * Component module for Haystack
 * 
 * This module contains the core Component trait and related functionality.
 */

pub mod sockets;
pub mod types;
pub mod examples;
pub mod registry;

pub use sockets::{Sockets, VariadicSocketHandler};
pub use types::{InputSocket, OutputSocket, Variadic, GreedyVariadic, SocketType};
pub use examples::{TextSplitter, TextJoiner};
pub use registry::{register_component, create_component, ComponentMetadata, registered_component_types};

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// The Component trait defines the interface that all Haystack components must implement.
/// 
/// Components are the building blocks of a Haystack pipeline. Each component performs a specific
/// task and can be connected to other components to form a processing pipeline.
pub trait Component: Send + Sync {
    /// The main function that executes the component's logic.
    /// 
    /// This method receives input data from previous components in the pipeline and
    /// returns output data that can be passed to subsequent components.
    /// 
    /// # Arguments
    /// * `inputs` - A map of input values keyed by their socket names
    /// 
    /// # Returns
    /// * `Result<HashMap<String, Value>>` - A map of output values keyed by their socket names, 
    ///   or an error if the component fails to execute
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>>;
    
    /// Optional method to initialize resources needed by the component.
    /// 
    /// This method is called by the Pipeline before execution. It can be used to
    /// initialize expensive resources like models, connections, etc.
    fn warm_up(&self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Returns the map of input sockets for this component.
    fn input_sockets(&self) -> &HashMap<String, InputSocket>;
    
    /// Returns the map of output sockets for this component.
    fn output_sockets(&self) -> &HashMap<String, OutputSocket>;
    
    /// Get the initialization parameters used to create this component.
    /// 
    /// These parameters can be used to recreate the component during serialization.
    fn init_parameters(&self) -> &HashMap<String, Value>;
}

/// Information about a component for serialization purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// The name of the component class (e.g., "TextSplitter")
    pub class_name: String,
    
    /// The module path to the component (e.g., "haystack.components.preprocessors.text_splitter")
    pub module_path: String,
    
    /// The initialization parameters for the component
    pub init_parameters: HashMap<String, Value>,
}

/// Implementation of necessary methods for Component serialization
pub trait ComponentSerialization {
    /// Get the component info for serialization
    fn component_info(&self) -> ComponentInfo;
    
    /// Create a new instance of the component from deserialized info
    fn from_component_info(info: &ComponentInfo) -> Result<Box<dyn Component>>;
}

/// Trait for components that can be run asynchronously
pub trait AsyncComponent: Component {
    /// Asynchronous version of the run method
    fn run_async<'a>(&'a self, inputs: HashMap<String, Value>) -> 
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<HashMap<String, Value>>> + Send + 'a>>;
}

/// Base implementation for components with common functionality
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentBase {
    /// The initialization parameters for this component
    init_parameters: HashMap<String, Value>,
    
    /// The input sockets for this component
    input_sockets: HashMap<String, InputSocket>,
    
    /// The output sockets for this component
    output_sockets: HashMap<String, OutputSocket>,
}

impl ComponentBase {
    /// Create a new ComponentBase
    pub fn new(
        init_parameters: HashMap<String, Value>,
        input_sockets: HashMap<String, InputSocket>,
        output_sockets: HashMap<String, OutputSocket>,
    ) -> Self {
        Self {
            init_parameters,
            input_sockets,
            output_sockets,
        }
    }
    
    /// Set an input socket
    pub fn set_input_socket(&mut self, socket: InputSocket) {
        self.input_sockets.insert(socket.name.clone(), socket);
    }
    
    /// Set an output socket
    pub fn set_output_socket(&mut self, socket: OutputSocket) {
        self.output_sockets.insert(socket.name.clone(), socket);
    }
    
    /// Get a reference to the init parameters
    pub fn init_parameters(&self) -> &HashMap<String, Value> {
        &self.init_parameters
    }
    
    /// Get a reference to the input sockets
    pub fn input_sockets(&self) -> &HashMap<String, InputSocket> {
        &self.input_sockets
    }
    
    /// Get a reference to the output sockets
    pub fn output_sockets(&self) -> &HashMap<String, OutputSocket> {
        &self.output_sockets
    }
}

/// Macro to simplify the creation of components
///
/// This macro helps with:
/// 1. Declaring component input and output sockets
/// 2. Implementing the Component trait
/// 3. Setting up serialization/deserialization
#[macro_export]
macro_rules! component {
    ($component_name:ident, inputs { $($input_name:expr => $input_type:ty $(= $default:expr)?),* $(,)? }, outputs { $($output_name:expr => $output_type:ty),* $(,)? }) => {
        // Macros to generate input and output sockets
        #[macro_export]
        macro_rules! component_inputs {
            ($component_name:ident) => {{
                let mut sockets = std::collections::HashMap::new();
                $(
                    sockets.insert(
                        $input_name.to_string(), 
                        $crate::component::InputSocket::new(
                            $input_name.to_string(),
                            std::any::TypeId::of::<$input_type>(),
                            stringify!($input_type).to_string(),
                            None, // We'll set the default in the next section if it exists
                            false,
                            false,
                        )
                    );
                )*
                // Now set defaults
                $(
                    $(
                        if let Some(socket) = sockets.get_mut($input_name) {
                            socket.default_value = Some(serde_json::to_value($default).unwrap());
                        }
                    )?
                )*
                sockets
            }};
        }

        #[macro_export]
        macro_rules! component_outputs {
            ($component_name:ident) => {{
                let mut sockets = std::collections::HashMap::new();
                $(
                    sockets.insert(
                        $output_name.to_string(), 
                        $crate::component::OutputSocket::new(
                            $output_name.to_string(),
                            std::any::TypeId::of::<$output_type>(),
                            stringify!($output_type).to_string(),
                        )
                    );
                )*
                sockets
            }};
        }
    };
}