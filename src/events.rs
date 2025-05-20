// Event handling for the Orbit UI framework

/// Event handler trait
pub trait EventHandler<E> {
    /// Handle an event
    fn handle(&mut self, event: E);
}

/// Basic event types
#[derive(Debug, Clone)]
pub enum Event {
    /// Mouse events
    Mouse(MouseEvent),
    /// Keyboard events
    Keyboard(KeyboardEvent),
    /// Touch events
    Touch(TouchEvent),
    /// Focus events
    Focus(FocusEvent),
    /// Lifecycle events
    Lifecycle(LifecycleEvent),
    /// Custom events
    Custom(CustomEvent),
}

/// Mouse event types
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Mouse down
    Down { x: f32, y: f32, button: MouseButton },
    /// Mouse up
    Up { x: f32, y: f32, button: MouseButton },
    /// Mouse move
    Move { x: f32, y: f32 },
    /// Mouse click
    Click { x: f32, y: f32, button: MouseButton },
    /// Mouse double click
    DoubleClick { x: f32, y: f32, button: MouseButton },
    /// Mouse enter
    Enter { x: f32, y: f32 },
    /// Mouse leave
    Leave { x: f32, y: f32 },
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button
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
