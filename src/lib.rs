// Core module of the Orbit UI Framework
pub mod component;
pub mod events;
pub mod parser;
pub mod platform;
pub mod renderer;
pub mod state;
pub mod style;

/// Version of the Orbit UI Framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export of common types for convenience
pub mod prelude {
    pub use crate::component::{Component, Props};
    pub use crate::events::EventEmitter;
    pub use crate::renderer::Renderer;
    pub use crate::state::State;
}

/// Initialize the Orbit framework with default settings
pub fn init() -> Result<(), Error> {
    // Initialize logging
    // Initialize default renderer
    // Set up platform adapters
    Ok(())
}

/// Errors that can occur in the Orbit framework
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Initialization error: {0}")]
    Init(String),

    #[error("Rendering error: {0}")]
    Render(String),

    #[error("Renderer error: {0}")]
    Renderer(String),

    #[error("Component error: {0}")]
    Component(String),

    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Platform error: {0}")]
    Platform(String),
}
