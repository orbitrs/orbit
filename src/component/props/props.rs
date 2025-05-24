//! Enhanced props system for Orbit UI components
//!
//! This module provides improved props functionality including:
//! - Type-safe props definitions
//! - Props validation
//! - Default values
//! - Required vs optional props

use std::fmt::{self, Display};
use std::sync::Arc;

/// Error indicating validation problems with props
#[derive(Debug, Clone)]
pub enum PropValidationError {
    /// A required property was missing
    MissingRequired(String),
    /// A property had an invalid value
    InvalidValue {
        /// Name of the property
        name: String,
        /// Description of the validation error
        reason: String,
    },
    /// A property had a type mismatch
    TypeMismatch {
        /// Name of the property
        name: String,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },
    /// Multiple validation errors
    Multiple(Vec<PropValidationError>),
}

impl Display for PropValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropValidationError::MissingRequired(name) => {
                write!(f, "Missing required property: {name}")
            }
            PropValidationError::InvalidValue { name, reason } => {
                write!(f, "Invalid value for property {name}: {reason}")
            }
            PropValidationError::TypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "Type mismatch for property {name}: expected {expected}, got {actual}"
            ),
            PropValidationError::Multiple(errors) => {
                writeln!(f, "Multiple validation errors:")?;
                for (i, error) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, error)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PropValidationError {}

/// Trait for props validation
pub trait PropValidator<P> {
    /// Validate the props
    fn validate(&self, props: &P) -> Result<(), PropValidationError>;
}

/// Represents a property that can be required or optional with default value
pub enum PropValue<T> {
    /// Value is present
    Value(T),
    /// Value is not present, but there is a default
    Default(fn() -> T),
    /// Value is required but not present
    Required,
}

impl<T: Clone> PropValue<T> {
    /// Get the value, returning default if needed
    pub fn get(&self) -> Result<T, PropValidationError> {
        match self {
            PropValue::Value(val) => Ok(val.clone()),
            PropValue::Default(default_fn) => Ok(default_fn()),
            PropValue::Required => Err(PropValidationError::MissingRequired(
                "Property is required".to_string(),
            )),
        }
    }

    /// Create a new value
    pub fn new(value: T) -> Self {
        PropValue::Value(value)
    }

    /// Create a new optional value with default
    pub fn new_default(default_fn: fn() -> T) -> Self {
        PropValue::Default(default_fn)
    }

    /// Create a new required value
    pub fn new_required() -> Self {
        PropValue::Required
    }

    /// Set the value
    pub fn set(&mut self, value: T) {
        *self = PropValue::Value(value);
    }

    /// Check if value is present
    pub fn is_set(&self) -> bool {
        matches!(self, PropValue::Value(_))
    }
}

/// Builder for constructing component props with validation
pub struct PropsBuilder<P> {
    /// The props being built
    props: P,
    /// Optional validator to run before completion
    validator: Option<Arc<dyn PropValidator<P> + Send + Sync>>,
}

impl<P> PropsBuilder<P> {
    /// Create a new props builder
    pub fn new(props: P) -> Self {
        Self {
            props,
            validator: None,
        }
    }

    /// Set a validator for the props
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: PropValidator<P> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// Build the props, running validation
    pub fn build(self) -> Result<P, PropValidationError> {
        if let Some(validator) = self.validator {
            validator.validate(&self.props)?;
        }
        Ok(self.props)
    }
}

/// A simple property validator that can be composed of multiple validators
pub struct CompositeValidator<P> {
    /// The validators to run
    validators: Vec<Arc<dyn PropValidator<P> + Send + Sync>>,
}

impl<P> CompositeValidator<P> {
    /// Create a new composite validator
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator to the composite
    pub fn add<V>(&mut self, validator: V)
    where
        V: PropValidator<P> + Send + Sync + 'static,
    {
        self.validators.push(Arc::new(validator));
    }
}

impl<P> PropValidator<P> for CompositeValidator<P> {
    fn validate(&self, props: &P) -> Result<(), PropValidationError> {
        let mut errors = Vec::new();

        for validator in &self.validators {
            if let Err(err) = validator.validate(props) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.remove(0))
        } else {
            Err(PropValidationError::Multiple(errors))
        }
    }
}

impl<P> Default for CompositeValidator<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-safe builder for props
#[macro_export]
macro_rules! define_props {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $name {
            /// Create a new props builder
            pub fn builder() -> $crate::component::props::PropsBuilder<Self> {
                let props = Self {
                    $(
                        $field_name: Default::default(),
                    )*
                };
                $crate::component::props::PropsBuilder::new(props)
            }

            // Generate setter methods for each field
            $(
                #[allow(dead_code)]
                pub fn $field_name(mut self, value: $field_type) -> Self {
                    self.$field_name = value;
                    self
                }
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field_name: Default::default(),
                    )*
                }
            }
        }
    };
}

// Export the paste crate for the macro to use
pub use paste;

/// Creates a validator that ensures a field meets a condition
#[macro_export]
macro_rules! validate_field {
    ($props_type:ty, $field:ident, $condition:expr, $message:expr) => {
        struct FieldValidator<T>(std::marker::PhantomData<T>);

        impl $crate::component::props::PropValidator<$props_type> for FieldValidator<$props_type> {
            fn validate(
                &self,
                props: &$props_type,
            ) -> Result<(), $crate::component::props::PropValidationError> {
                if !$condition(&props.$field) {
                    return Err(
                        $crate::component::props::PropValidationError::InvalidValue {
                            name: stringify!($field).to_string(),
                            reason: $message.to_string(),
                        },
                    );
                }
                Ok(())
            }
        }

        FieldValidator::<$props_type>(std::marker::PhantomData)
    };
}

/// Advanced props builder with validation and required field support
#[macro_export]
macro_rules! define_props_advanced {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[required])?
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
                $(= $default:expr)?
            ),*
            $(,)?
        }
    ) => {
        // First define the basic struct
        define_props! {
            $(#[$struct_meta])*
            $vis struct $name {
                $(
                    $(#[$field_meta])*
                    $field_vis $field_name: $field_type
                ),*
            }
        }

        // Generate builder struct with validation
        paste::paste! {
            pub struct [<$name Builder>] {
                $(
                    $field_name: $crate::component::props::PropValue<$field_type>,
                )*
            }

            impl [<$name Builder>] {
                pub fn new() -> Self {
                    Self {
                        $(
                            $field_name: define_props_advanced!(@field_init $field_name $(#[required])? $(= $default)?),
                        )*
                    }
                }

                $(
                    pub fn $field_name(mut self, value: $field_type) -> Self {
                        self.$field_name.set(value);
                        self
                    }
                )*

                pub fn build(self) -> Result<$name, $crate::component::props::PropValidationError> {
                    let mut errors = Vec::new();

                    $(
                        let $field_name = match self.$field_name.get() {
                            Ok(val) => val,
                            Err(_) => {
                                // Check if this field was marked as required
                                if define_props_advanced!(@is_required $field_name $(#[required])?) {
                                    errors.push($crate::component::props::PropValidationError::MissingRequired(
                                        stringify!($field_name).to_string()
                                    ));
                                }
                                Default::default()
                            }
                        };
                    )*

                    if errors.is_empty() {
                        Ok($name {
                            $(
                                $field_name,
                            )*
                        })
                    } else if errors.len() == 1 {
                        Err(errors.remove(0))
                    } else {
                        Err($crate::component::props::PropValidationError::Multiple(errors))
                    }
                }
            }

            impl Default for [<$name Builder>] {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    };

    // Helper to initialize fields based on whether they're required or have defaults
    (@field_init $field_name:ident #[required] = $default:expr) => {
        $crate::component::props::PropValue::new_required()
    };
    (@field_init $field_name:ident #[required]) => {
        $crate::component::props::PropValue::new_required()
    };
    (@field_init $field_name:ident = $default:expr) => {
        $crate::component::props::PropValue::new_default(|| $default)
    };
    (@field_init $field_name:ident) => {
        $crate::component::props::PropValue::new_default(Default::default)
    };

    // Helper to check if a field is required
    (@is_required $field_name:ident #[required]) => { true };
    (@is_required $field_name:ident) => { false };
}
