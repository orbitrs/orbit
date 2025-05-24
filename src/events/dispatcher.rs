//! Event dispatcher for typed event handling
//!
//! This module provides a type-safe event dispatch system with thread-safety
//! and proper error handling.

use std::sync::{Arc, Mutex};

use super::Event;

/// Handler trait for processing events
pub trait Handler<E>: Send + Sync {
    /// Handle an event
    fn handle(&self, event: &E) -> Result<(), String>;
}

/// Implement Handler for Fn(&E)
impl<E, F> Handler<E> for F
where
    F: Fn(&E) -> Result<(), String> + Send + Sync,
{
    fn handle(&self, event: &E) -> Result<(), String> {
        self(event)
    }
}

/// Thread-safe event dispatcher for a specific event type
#[derive(Clone)]
pub struct Dispatcher<E: Event> {
    /// List of event handlers protected by a mutex
    handlers: Arc<Mutex<Vec<Box<dyn Handler<E>>>>>,
}

impl<E: Event> Default for Dispatcher<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Event> Dispatcher<E> {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new event handler
    pub fn add_handler<H>(&mut self, handler: H) -> Result<(), String>
    where
        H: Handler<E> + 'static,
    {
        let mut handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock handlers: {}", e))?;

        handlers.push(Box::new(handler));
        Ok(())
    }

    /// Add a handler function directly
    pub fn on<F>(&mut self, f: F) -> Result<(), String>
    where
        F: Fn(&E) -> Result<(), String> + Send + Sync + 'static,
    {
        self.add_handler(f)
    }

    /// Dispatch an event to all handlers
    pub fn dispatch(&self, event: &E) -> Result<(), String> {
        let handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock handlers: {}", e))?;

        let mut errors = Vec::new();

        for handler in handlers.iter() {
            if let Err(e) = handler.handle(event) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join(", "))
        }
    }

    /// Remove all handlers
    pub fn clear(&mut self) -> Result<(), String> {
        let mut handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock handlers: {}", e))?;

        handlers.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Clone)]
    struct TestEvent {
        data: String,
    }

    // TestEvent automatically implements Event through the blanket implementation

    #[test]
    fn test_dispatcher() {
        let mut dispatcher = Dispatcher::<TestEvent>::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        dispatcher
            .on(move |event: &TestEvent| {
                called_clone.store(true, Ordering::SeqCst);
                assert_eq!(event.data, "test");
                Ok(())
            })
            .unwrap();

        dispatcher
            .dispatch(&TestEvent {
                data: "test".to_string(),
            })
            .unwrap();

        assert!(called.load(Ordering::SeqCst));
    }
}
