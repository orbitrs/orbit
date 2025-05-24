//! Event emitter for component events

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::events::Event;

/// Type for event handler callbacks
type EventCallback = Box<dyn Fn(&dyn Event) + Send + Sync>;

/// Event emitter for handling component events
#[derive(Clone)]
pub struct EventEmitter {
    /// Event handlers grouped by event type
    handlers: Arc<Mutex<HashMap<TypeId, Vec<Arc<EventCallback>>>>>,
}

impl std::fmt::Debug for EventEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventEmitter")
            .field("handlers", &"[EventHandlers]")
            .finish()
    }
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register an event handler
    pub fn on<E: Event + 'static>(
        &mut self,
        handler: impl Fn(&E) + Send + Sync + 'static,
    ) -> Result<(), String> {
        let type_id = TypeId::of::<E>();

        let callback: EventCallback = Box::new(move |event| {
            if let Some(typed_event) = event.as_any().downcast_ref::<E>() {
                handler(typed_event);
            }
        });

        let mut handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock event handlers: {}", e))?;

        handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Arc::new(callback));

        Ok(())
    }

    /// Emit an event
    pub fn emit<E: Event>(&self, event: &E) -> Result<(), String> {
        let type_id = TypeId::of::<E>();

        let handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock event handlers: {}", e))?;

        if let Some(handlers) = handlers.get(&type_id) {
            for handler in handlers {
                handler(event);
            }
        }
        Ok(())
    }

    /// Remove all handlers
    pub fn clear(&mut self) -> Result<(), String> {
        let mut handlers = self
            .handlers
            .lock()
            .map_err(|e| format!("Failed to lock event handlers: {}", e))?;
        handlers.clear();
        Ok(())
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
