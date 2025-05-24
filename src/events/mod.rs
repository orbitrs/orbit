//! Enhanced event system for Orbit UI framework
//!
//! The event system provides:
//! - Generic event trait with downcasting support
//! - Event emitter for general event handling with type erasure
//! - Dispatcher for strongly-typed event handling
//! - Event delegation for component event propagation

pub mod delegation;
pub mod dispatcher;
pub mod emitter;
pub mod event;

pub use delegation::*;
pub use dispatcher::Dispatcher;
pub use emitter::EventEmitter;
pub use event::Event;
