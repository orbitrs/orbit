//! Error types for component operations

use std::any::TypeId;
use std::error::Error;
use std::fmt;

use crate::component::LifecyclePhase;

/// Errors that can occur during component operations
#[derive(Debug)]
pub enum ComponentError {
    /// Component type not found in registry
    TypeNotFound(TypeId),

    /// Props type mismatch
    PropsMismatch { expected: TypeId, got: TypeId },

    /// Invalid props type for the component
    InvalidPropsType,

    /// Error downcasting props or component
    DowncastError,

    /// Error acquiring lock
    LockError(String),

    /// Invalid lifecycle transition
    InvalidLifecycleTransition(LifecyclePhase, String),

    /// Error rendering component
    RenderError(String),

    /// Error updating component
    UpdateError(String),

    /// Error mounting component
    MountError(String),

    /// Error unmounting component
    UnmountError(String),

    /// Error from reactive system
    ReactiveSystemError(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeNotFound(type_id) => write!(f, "Component type {:?} not found", type_id),
            Self::PropsMismatch { expected, got } => write!(
                f,
                "Props type mismatch - expected {:?}, got {:?}",
                expected, got
            ),
            Self::InvalidPropsType => write!(f, "Invalid props type for the component"),
            Self::DowncastError => write!(f, "Failed to downcast props or component"),
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
            Self::InvalidLifecycleTransition(phase, operation) => write!(
                f,
                "Invalid lifecycle transition: cannot {} while in {:?} phase",
                operation, phase
            ),
            Self::RenderError(msg) => write!(f, "Error rendering component: {}", msg),
            Self::UpdateError(msg) => write!(f, "Error updating component: {}", msg),
            Self::MountError(msg) => write!(f, "Error mounting component: {}", msg),
            Self::UnmountError(msg) => write!(f, "Error unmounting component: {}", msg),
            Self::ReactiveSystemError(msg) => write!(f, "Reactive system error: {}", msg),
        }
    }
}

impl Error for ComponentError {}

// Conversion from SignalError to ComponentError
impl From<crate::state::SignalError> for ComponentError {
    fn from(error: crate::state::SignalError) -> Self {
        ComponentError::ReactiveSystemError(error.to_string())
    }
}
