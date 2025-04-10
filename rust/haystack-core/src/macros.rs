/*!
 * Macros for simplifying component creation and registration
 */

/// Macro to define a new component
///
/// This macro helps with:
/// 1. Declaring component input and output sockets
/// 2. Implementing the Component trait
/// 3. Setting up factory methods for serialization/deserialization
///
/// # Example
///
/// ```ignore
/// define_component! {
///     #[doc = "A text splitter component that splits text into chunks."]
///     pub struct TextSplitter {
///         // Component fields
///         max_chunk_length: usize,
///     }
///
///     // Declare inputs
///     inputs {
///         text: String,
///     }
///
///     // Declare outputs
///     outputs {
///         chunks: Vec<String>,
///         count: usize,
///     }
///
///     // Constructor
///     fn new(max_chunk_length: usize) -> Self {
///         Self {
///             max_chunk_length,
///         }
///     }
///
///     // Component implementation
///     impl Component for Self {
///         fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
///             // Implementation goes here
///         }
///     }
/// }
/// ```
///
/// To declare variadic inputs, use the Variadic or GreedyVariadic wrappers:
///
/// ```ignore
/// inputs {
///     texts: Variadic<String>,
///     chunks: GreedyVariadic<Document>,
/// }
/// ```
#[macro_export]
macro_rules! define_component {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }

        inputs {
            $(
                $input_name:ident: $input_type:ty $(= $input_default:expr)?
            ),* $(,)?
        }

        outputs {
            $(
                $output_name:ident: $output_type:ty
            ),* $(,)?
        }

        fn new($($param_name:ident: $param_type:ty),* $(,)?) -> Self {
            $($new_body:tt)*
        }

        impl Component for Self {
            fn run(&self, $inputs_var:ident: HashMap<String, Value>) -> Result<HashMap<String, Value>> $run_body:block
            
            $(
                fn $other_method:ident(&self $(, $other_param:ident: $other_param_type:ty)*) -> $other_return:ty
                $other_body:block
            )*
        }
        
        $(
            $extra_impl:item
        )*
    ) => {
        $(#[$meta])*
        $vis struct $name {
            /// Base component implementation
            base: $crate::component::ComponentBase,
            
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type
            ),*
        }

        impl $name {
            /// Create a new instance of this component
            pub fn new($($param_name: $param_type),*) -> Self {
                // Create input and output sockets
                let input_sockets = $crate::component_inputs!($name, {
                    $(
                        $input_name: $input_type $(= $input_default)?
                    ),*
                });
                
                let output_sockets = $crate::component_outputs!($name, {
                    $(
                        $output_name: $output_type
                    ),*
                });
                
                // Create init parameters
                let mut init_parameters = ::std::collections::HashMap::new();
                $(
                    init_parameters.insert(
                        stringify!($param_name).to_string(),
                        serde_json::to_value(&$param_name).unwrap_or(serde_json::Value::Null)
                    );
                )*
                
                // Create the struct with default values
                let mut component = Self {
                    base: $crate::component::ComponentBase::new(
                        init_parameters,
                        input_sockets,
                        output_sockets,
                    ),
                    $($field_name),*
                };
                
                // Apply the new implementation from the macro
                {
                    use $crate::component::Component;
                    $($new_body)*
                }
                
                component
            }
            
            /// Create a component from component info
            pub fn from_component_info(info: &$crate::component::ComponentInfo) -> ::anyhow::Result<Self> {
                let params = &info.init_parameters;
                
                $(
                    let $param_name = match params.get(stringify!($param_name)) {
                        Some(v) => serde_json::from_value::<$param_type>(v.clone())
                            .map_err(|e| anyhow::anyhow!("Error deserializing parameter '{}': {}", stringify!($param_name), e))?,
                        None => return Err(anyhow::anyhow!("Missing parameter '{}'", stringify!($param_name))),
                    };
                )*
                
                Ok(Self::new($($param_name),*))
            }
        }
        
        impl $crate::component::Component for $name {
            fn run(&self, $inputs_var: ::std::collections::HashMap<String, serde_json::Value>) -> ::anyhow::Result<::std::collections::HashMap<String, serde_json::Value>> $run_body
            
            fn input_sockets(&self) -> &::std::collections::HashMap<String, $crate::component::InputSocket> {
                self.base.input_sockets()
            }
            
            fn output_sockets(&self) -> &::std::collections::HashMap<String, $crate::component::OutputSocket> {
                self.base.output_sockets()
            }
            
            fn init_parameters(&self) -> &::std::collections::HashMap<String, serde_json::Value> {
                self.base.init_parameters()
            }
            
            $(
                fn $other_method(&self $(, $other_param: $other_param_type)*) -> $other_return
                $other_body
            )*
        }
        
        impl $crate::component::ComponentSerialization for $name {
            fn component_info(&self) -> $crate::component::ComponentInfo {
                $crate::component::ComponentInfo {
                    class_name: stringify!($name).to_string(),
                    module_path: module_path!().to_string(),
                    init_parameters: self.init_parameters().clone(),
                }
            }
            
            fn from_component_info(info: &$crate::component::ComponentInfo) -> ::anyhow::Result<Box<dyn $crate::component::Component>> {
                let component = Self::from_component_info(info)?;
                Ok(Box::new(component))
            }
        }
        
        $(
            $extra_impl
        )*
    };
}

/// Macro to generate input sockets for a component
#[macro_export]
macro_rules! component_inputs {
    ($component_name:ident, {
        $(
            $input_name:ident: $input_type:ty $(= $default:expr)?
        ),* $(,)?
    }) => {{
        let mut sockets = ::std::collections::HashMap::new();
        $(
            // Check if the type is a variadic type
            $crate::create_input_socket!(sockets, $input_name, $input_type);
        )*
        
        // Now set defaults
        $(
            $(
                if let Some(socket) = sockets.get_mut(stringify!($input_name)) {
                    socket.default_value = Some(serde_json::to_value($default).unwrap_or(serde_json::Value::Null));
                }
            )?
        )*
        
        sockets
    }};
}

/// Helper macro to create the appropriate input socket based on type
#[macro_export]
macro_rules! create_input_socket {
    ($sockets:ident, $input_name:ident, Variadic<$inner_type:ty>) => {
        let socket = $crate::component::InputSocket::new_variadic::<$inner_type>(
            stringify!($input_name).to_string(),
            stringify!($inner_type).to_string(),
            None,
            false,
        );
        $sockets.insert(stringify!($input_name).to_string(), socket);
    };
    
    ($sockets:ident, $input_name:ident, GreedyVariadic<$inner_type:ty>) => {
        let socket = $crate::component::InputSocket::new_variadic::<$inner_type>(
            stringify!($input_name).to_string(),
            stringify!($inner_type).to_string(),
            None,
            true,
        );
        $sockets.insert(stringify!($input_name).to_string(), socket);
    };
    
    ($sockets:ident, $input_name:ident, $input_type:ty) => {
        let socket = $crate::component::InputSocket::new(
            stringify!($input_name).to_string(),
            ::std::any::TypeId::of::<$input_type>(),
            stringify!($input_type).to_string(),
            None,
            false,
            false,
        );
        $sockets.insert(stringify!($input_name).to_string(), socket);
    };
}

/// Macro to generate output sockets for a component
#[macro_export]
macro_rules! component_outputs {
    ($component_name:ident, {
        $(
            $output_name:ident: $output_type:ty
        ),* $(,)?
    }) => {{
        let mut sockets = ::std::collections::HashMap::new();
        $(
            let socket = $crate::component::OutputSocket::new(
                stringify!($output_name).to_string(),
                ::std::any::TypeId::of::<$output_type>(),
                stringify!($output_type).to_string(),
            );
            sockets.insert(stringify!($output_name).to_string(), socket);
        )*
        
        sockets
    }};
}

/// Macro to register a component in the global registry
#[macro_export]
macro_rules! register_component_type {
    (
        $component_type:ty,
        $description:expr,
        $version:expr,
        $($tag:expr),* $(,)?
    ) => {
        $crate::register_component!(
            stringify!($component_type),
            module_path!(),
            $description,
            $version,
            vec![$($tag.to_string()),*],
            $crate::component_factory!($component_type)
        );
    };
}

/// Helper macro for extracting a variadic socket value from inputs
/// 
/// This extracts a Variadic<T> or GreedyVariadic<T> from the JSON inputs and
/// deserializes each item in the array to the target type.
/// 
/// # Example
/// 
/// ```ignore
/// let texts = extract_variadic!(inputs, "texts", String);
/// ```
#[macro_export]
macro_rules! extract_variadic {
    ($inputs:ident, $socket_name:expr, $inner_type:ty) => {{
        // First ensure we have a valid array
        let values_array = $inputs.get($socket_name)
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Expected array for variadic socket '{}'", $socket_name))?;
        
        // For each item in the array, attempt to deserialize to the target type
        let mut values = Vec::new();
        for item in values_array.iter() {
            let value = match serde_json::from_value::<$inner_type>(item.clone()) {
                Ok(v) => v,
                Err(e) => {
                    // Print item for debugging
                    eprintln!("Failed to deserialize item: {:?}", item);
                    return Err(anyhow::anyhow!("Error deserializing variadic input '{}': {}", $socket_name, e));
                }
            };
            values.push(value);
        }
            
        $crate::component::Variadic::from(values)
    }};
}

/// Helper macro for extracting a value from component inputs
/// 
/// This extracts a value from the JSON inputs and deserializes it to the target type.
/// If the key doesn't exist or the value can't be deserialized, it returns an error.
/// 
/// # Example
/// 
/// ```ignore
/// let text = extract_input!(inputs, "text", String);
/// ```
#[macro_export]
macro_rules! extract_input {
    ($inputs:ident, $socket_name:expr, $type:ty) => {
        $inputs.get($socket_name)
            .ok_or_else(|| anyhow::anyhow!("Missing input for socket '{}'", $socket_name))
            .and_then(|v| serde_json::from_value::<$type>(v.clone())
                .map_err(|e| anyhow::anyhow!("Error deserializing input '{}': {}", $socket_name, e)))?
    };
}

/// Helper macro for extracting an optional value from component inputs
/// 
/// This extracts a value from the JSON inputs and deserializes it to the target type.
/// If the key doesn't exist, it returns None. If the value can't be deserialized, it returns an error.
/// 
/// # Example
/// 
/// ```ignore
/// let text = extract_optional_input!(inputs, "text", String);
/// ```
#[macro_export]
macro_rules! extract_optional_input {
    ($inputs:ident, $socket_name:expr, $type:ty) => {
        $inputs.get($socket_name)
            .map(|v| serde_json::from_value::<$type>(v.clone())
                .map_err(|e| anyhow::anyhow!("Error deserializing input '{}': {}", $socket_name, e)))
            .transpose()?
    };
}