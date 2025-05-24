//! Event trait for OrbitRS events system

use std::any::Any;

/// Generic event trait
pub trait Event: 'static {
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Convert to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get the event type name
    fn event_type(&self) -> &'static str;

    /// Clone the event
    fn box_clone(&self) -> Box<dyn Event>;
}

/// Default implementation for types that implement Clone
impl<T: Any + Clone + 'static> Event for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn event_type(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn box_clone(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

/// Re-export winit event types
pub mod winit {
    pub use winit::event::{Event, MouseButton};
}
