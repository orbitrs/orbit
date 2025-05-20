//! Event handling system for the Orbit UI framework

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A type-erased event that can be sent through the event system
pub trait Event: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync> Event for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event handler function type
pub type EventListenerFn = Box<dyn Fn(&dyn Event) + Send + Sync>;

/// Manages event listeners and event dispatch
#[derive(Default, Clone)]
pub struct EventEmitter {
    /// Maps event types to their handlers
    handlers: Arc<Mutex<HashMap<TypeId, Vec<EventListenerFn>>>>,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add an event listener
    pub fn on<E: Event + 'static>(&self, handler: impl Fn(&E) + Send + Sync + 'static) {
        let mut handlers = self.handlers.lock().unwrap();
        let type_id = TypeId::of::<E>();

        let handler: EventListenerFn = Box::new(move |event| {
            if let Some(e) = event.as_any().downcast_ref::<E>() {
                handler(e);
            }
        });

        handlers.entry(type_id).or_default().push(handler);
    }

    /// Remove all listeners for an event type
    pub fn off<E: Event + 'static>(&self) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.remove(&TypeId::of::<E>());
    }

    /// Emit an event to all registered handlers
    pub fn emit<E: Event + 'static>(&self, event: &E) {
        let handlers = self.handlers.lock().unwrap();
        if let Some(handlers) = handlers.get(&TypeId::of::<E>()) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}

/// Built-in event types

/// Mouse event containing position and button information
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// X coordinate relative to the target element
    pub x: f32,
    /// Y coordinate relative to the target element
    pub y: f32,
    /// Which mouse button was involved
    pub button: Option<MouseButton>,
    /// Type of mouse event
    pub event_type: MouseEventType,
}

// Remove explicit impl as MouseEvent already implements Event through the generic impl
// impl Event for MouseEvent {}

/// Types of mouse events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    /// Mouse button pressed
    Down,
    /// Mouse button released
    Up,
    /// Mouse moved
    Move,
    /// Mouse clicked (down and up)
    Click,
    /// Mouse double clicked
    DoubleClick,
    /// Mouse entered an element
    Enter,
    /// Mouse left an element
    Leave,
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button (scroll wheel)
    Middle,
    /// Right mouse button
    Right,
}

/// Keyboard event types
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// Key down
    Down { key: String, code: String },
    /// Key up
    Up { key: String, code: String },
    /// Key press
    Press { key: String, code: String },
}

/// Touch event types
#[derive(Debug, Clone)]
pub enum TouchEvent {
    /// Touch start
    Start { touches: Vec<Touch> },
    /// Touch move
    Move { touches: Vec<Touch> },
    /// Touch end
    End { touches: Vec<Touch> },
    /// Touch cancel
    Cancel { touches: Vec<Touch> },
}

/// Touch point
#[derive(Debug, Clone)]
pub struct Touch {
    /// Touch identifier
    pub id: u64,
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Pressure
    pub pressure: f32,
}

/// Focus event types
#[derive(Debug, Clone)]
pub enum FocusEvent {
    /// Focus
    Focus,
    /// Blur
    Blur,
}

/// Lifecycle event types
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Mount
    Mount,
    /// Unmount
    Unmount,
    /// Update
    Update,
}

/// Custom event with a name and data
#[derive(Debug)]
pub struct CustomEvent {
    /// Event name
    pub name: String,
    /// Event data
    pub data: Box<dyn std::any::Any + Send>,
}

impl Clone for CustomEvent {
    fn clone(&self) -> Self {
        // Since we can't clone the Any data directly,
        // we create a new CustomEvent with the same name but empty data
        // Applications using custom events should handle this properly
        Self {
            name: self.name.clone(),
            data: Box::new(()),
        }
    }
}

/// Event handler trait for UI events
pub trait Handler<E> {
    /// Handle an event
    fn handle(&mut self, event: &E);
}

/// Function-based event handler
pub struct HandlerFn<E, F: FnMut(&E)> {
    /// The handler function
    handler: F,
    /// Phantom data for event type
    _phantom: std::marker::PhantomData<E>,
}

impl<E, F: FnMut(&E)> HandlerFn<E, F> {
    /// Create a new function-based event handler
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E, F: FnMut(&E)> Handler<E> for HandlerFn<E, F> {
    fn handle(&mut self, event: &E) {
        (self.handler)(event);
    }
}

/// Vector of event handlers for a specific event type
#[derive(Default)]
pub struct Dispatcher<E> {
    /// List of event handlers
    handlers: Vec<Box<dyn Handler<E>>>,
}

impl<E> Dispatcher<E> {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a new event handler
    pub fn add_handler<H: Handler<E> + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Dispatch an event to all handlers
    pub fn dispatch(&mut self, event: &E) {
        for handler in &mut self.handlers {
            handler.handle(event);
        }
    }
}
