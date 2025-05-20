//! Event handling system for the Orbit UI framework

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A type-erased event that can be sent through the event system
pub trait Event: Any + Send + Sync {}

/// Event handler function type
pub type EventHandlerFn = Box<dyn Fn(&dyn Event) + Send + Sync>;

/// Manages event listeners and event dispatch
#[derive(Default)]
pub struct EventEmitter {
    /// Maps event types to their handlers
    handlers: Arc<Mutex<HashMap<TypeId, Vec<EventHandlerFn>>>>,
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
        
        let handler: EventHandlerFn = Box::new(move |event| {
            if let Some(e) = event.downcast_ref::<E>() {
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

impl Event for MouseEvent {}

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

/// Event dispatcher
pub struct EventDispatcher {
    /// Event handlers
    handlers: Vec<Box<dyn EventHandler<Event>>>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add an event handler
    pub fn add_handler<H: EventHandler<Event> + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Dispatch an event
    pub fn dispatch(&mut self, event: Event) {
        for handler in &mut self.handlers {
            handler.handle(event.clone());
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
