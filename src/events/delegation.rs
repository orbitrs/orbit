//! Event delegation system for Orbit UI framework
//!
//! This module provides an enhanced event handling system with:
//! - Event bubbling and capturing
//! - Event delegation up and down the component tree
//! - Stop propagation and prevent default functionality

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};

use crate::component_single::Node;
use crate::events::Event;

/// Type alias for event handler function
type EventHandler = Box<dyn Fn(&mut dyn Event, &EventPropagation) + Send + Sync>;

/// Type alias for handler storage map
type HandlerMap = Arc<RwLock<HashMap<TypeId, Vec<EventHandler>>>>;

/// Specifies the event propagation phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationPhase {
    /// Event is traveling down from parent to target (DOM capturing phase)
    Capturing,

    /// Event is at the target component
    Target,

    /// Event is traveling up from target to parent (DOM bubbling phase)
    Bubbling,
}

/// Controls how an event propagates through the component tree
#[derive(Debug, Clone)]
pub struct EventPropagation {
    /// Whether the event should continue propagating
    pub stopped: bool,

    /// Whether the default action should be prevented
    pub default_prevented: bool,

    /// The current propagation phase
    pub phase: PropagationPhase,

    /// The component that is the original target of the event
    pub target_id: Option<usize>,

    /// The component that is currently handling the event
    pub current_target_id: Option<usize>,
}

impl EventPropagation {
    /// Create a new event propagation
    pub fn new(phase: PropagationPhase) -> Self {
        Self {
            stopped: false,
            default_prevented: false,
            phase,
            target_id: None,
            current_target_id: None,
        }
    }

    /// Stop event propagation
    pub fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    /// Prevent the default action
    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    /// Check if propagation is stopped
    pub fn is_propagation_stopped(&self) -> bool {
        self.stopped
    }

    /// Check if default is prevented
    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }
}

/// An enhanced event that includes propagation information
pub struct DelegatedEvent<E: Event> {
    /// The original event
    pub event: E,

    /// Propagation control
    pub propagation: EventPropagation,
}

impl<E: Event> DelegatedEvent<E> {
    /// Create a new delegated event
    pub fn new(event: E, phase: PropagationPhase) -> Self {
        Self {
            event,
            propagation: EventPropagation::new(phase),
        }
    }

    /// Stop propagation of this event
    pub fn stop_propagation(&mut self) {
        self.propagation.stop_propagation();
    }

    /// Prevent the default action for this event
    pub fn prevent_default(&mut self) {
        self.propagation.prevent_default();
    }
}

/// Type of callback for delegated events
#[allow(dead_code)]
type DelegatedEventCallback<E> = Box<dyn Fn(&mut DelegatedEvent<E>) + Send + Sync>;

/// Event delegate manages capturing, targeting, and bubbling of events
#[derive(Default)]
pub struct EventDelegate {
    /// Map of event type to callbacks registered for capturing phase
    capturing_handlers: HandlerMap,

    /// Map of event type to callbacks registered for bubbling phase
    bubbling_handlers: HandlerMap,

    /// Map of event type to callbacks registered for just that target
    target_handlers: HandlerMap,

    /// Component ID for identification during propagation
    component_id: Option<usize>,

    /// Parent delegate for bubbling events up
    parent: Option<Arc<Mutex<EventDelegate>>>,

    /// Child delegates for capturing events down
    children: Vec<Arc<Mutex<EventDelegate>>>,
}

impl EventDelegate {
    /// Create a new event delegate
    pub fn new(component_id: Option<usize>) -> Self {
        Self {
            capturing_handlers: Arc::new(RwLock::new(HashMap::new())),
            bubbling_handlers: Arc::new(RwLock::new(HashMap::new())),
            target_handlers: Arc::new(RwLock::new(HashMap::new())),
            component_id,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Set the parent delegate for bubbling events
    pub fn set_parent(&mut self, parent: Arc<Mutex<EventDelegate>>) {
        self.parent = Some(parent);
    }

    /// Add a child delegate for capturing events
    pub fn add_child(&mut self, child: Arc<Mutex<EventDelegate>>) {
        self.children.push(child);
    }

    /// Register a handler for an event type in the capturing phase
    pub fn capture<E: Event + 'static>(
        &self,
        handler: impl Fn(&E, &EventPropagation) + Send + Sync + 'static,
    ) {
        let type_id = TypeId::of::<E>();
        let mut handlers = self.capturing_handlers.write().unwrap();

        let boxed_handler: EventHandler = Box::new(move |e, prop| {
            if let Some(event) = e.as_any().downcast_ref::<E>() {
                handler(event, prop);
            }
        });

        handlers.entry(type_id).or_default().push(boxed_handler);
    }

    /// Register a handler for an event type in the bubbling phase
    pub fn bubble<E: Event + 'static>(
        &self,
        handler: impl Fn(&E, &EventPropagation) + Send + Sync + 'static,
    ) {
        let type_id = TypeId::of::<E>();
        let mut handlers = self.bubbling_handlers.write().unwrap();

        let boxed_handler: EventHandler = Box::new(move |e, prop| {
            if let Some(event) = e.as_any().downcast_ref::<E>() {
                handler(event, prop);
            }
        });

        handlers.entry(type_id).or_default().push(boxed_handler);
    }

    /// Register a handler for an event type for this target only
    pub fn on<E: Event + 'static>(
        &self,
        handler: impl Fn(&E, &EventPropagation) + Send + Sync + 'static,
    ) {
        let type_id = TypeId::of::<E>();
        let mut handlers = self.target_handlers.write().unwrap();

        let boxed_handler: EventHandler = Box::new(move |e, prop| {
            if let Some(event) = e.as_any().downcast_ref::<E>() {
                handler(event, prop);
            }
        });

        handlers.entry(type_id).or_default().push(boxed_handler);
    }

    /// Dispatch an event starting from this delegate
    pub fn dispatch<E: Event + 'static>(&self, event: &E, target_id: Option<usize>) {
        // First, we do the capturing phase from the root down to the target
        let mut propagation = EventPropagation {
            stopped: false,
            default_prevented: false,
            phase: PropagationPhase::Capturing,
            target_id,
            current_target_id: self.component_id,
        };

        self.dispatch_capturing(event, &mut propagation, target_id);

        // Next, we handle the target phase
        if !propagation.is_propagation_stopped() && self.component_id == target_id {
            propagation.phase = PropagationPhase::Target;
            propagation.current_target_id = self.component_id;
            self.handle_event(event, &propagation);
        }

        // Finally, we do the bubbling phase from the target up to the root
        if !propagation.is_propagation_stopped() {
            propagation.phase = PropagationPhase::Bubbling;
            self.dispatch_bubbling(event, &mut propagation, target_id);
        }
    }

    // Internal method to handle capturing phase
    fn dispatch_capturing<E: Event + 'static>(
        &self,
        event: &E,
        propagation: &mut EventPropagation,
        target_id: Option<usize>,
    ) {
        // Handle this component's capturing handlers
        if !propagation.is_propagation_stopped() {
            propagation.current_target_id = self.component_id;
            self.handle_capturing_event(event, propagation);
        }

        // If this is the target, stop capturing phase
        if self.component_id == target_id {
            return;
        }

        // Otherwise, continue capturing phase down to children
        for child in &self.children {
            if let Ok(child) = child.lock() {
                if !propagation.is_propagation_stopped() {
                    child.dispatch_capturing(event, propagation, target_id);
                }
            }
        }
    }

    // Internal method to handle bubbling phase
    fn dispatch_bubbling<E: Event + 'static>(
        &self,
        event: &E,
        propagation: &mut EventPropagation,
        _target_id: Option<usize>,
    ) {
        // Handle this component's bubbling handlers
        if !propagation.is_propagation_stopped() {
            propagation.current_target_id = self.component_id;
            self.handle_bubbling_event(event, propagation);
        }

        // Continue bubbling up to parent
        if !propagation.is_propagation_stopped() {
            if let Some(parent) = &self.parent {
                if let Ok(parent) = parent.lock() {
                    parent.dispatch_bubbling(event, propagation, _target_id);
                }
            }
        }
    }

    // Internal method to handle events during capturing phase
    fn handle_capturing_event<E: Event + 'static>(
        &self,
        event: &E,
        propagation: &EventPropagation,
    ) {
        let type_id = TypeId::of::<E>();
        if let Ok(handlers) = self.capturing_handlers.read() {
            if let Some(handlers) = handlers.get(&type_id) {
                let mut boxed_event = event.box_clone();
                for handler in handlers {
                    handler(boxed_event.as_mut(), propagation);
                    if propagation.is_propagation_stopped() {
                        break;
                    }
                }
            }
        }
    }

    // Internal method to handle events during bubbling phase
    fn handle_bubbling_event<E: Event + 'static>(&self, event: &E, propagation: &EventPropagation) {
        let type_id = TypeId::of::<E>();
        if let Ok(handlers) = self.bubbling_handlers.read() {
            if let Some(handlers) = handlers.get(&type_id) {
                let mut boxed_event = event.box_clone();
                for handler in handlers {
                    handler(boxed_event.as_mut(), propagation);
                    if propagation.is_propagation_stopped() {
                        break;
                    }
                }
            }
        }
    }

    // Internal method to handle events at the target
    fn handle_event<E: Event + 'static>(&self, event: &E, propagation: &EventPropagation) {
        let type_id = TypeId::of::<E>();
        if let Ok(handlers) = self.target_handlers.read() {
            if let Some(handlers) = handlers.get(&type_id) {
                let mut boxed_event = event.box_clone();
                for handler in handlers {
                    handler(boxed_event.as_mut(), propagation);
                    if propagation.is_propagation_stopped() {
                        break;
                    }
                }
            }
        }
    }
}

/// Helper to build an event delegation tree from a component tree
pub fn build_delegation_tree(
    node: &Node,
    parent_delegate: Option<Arc<Mutex<EventDelegate>>>,
) -> Arc<Mutex<EventDelegate>> {
    let delegate = Arc::new(Mutex::new(EventDelegate::new(Some(node.id_value()))));

    // Set parent relationship
    if let Some(parent) = parent_delegate {
        if let Ok(mut delegate_mut) = delegate.lock() {
            delegate_mut.set_parent(parent.clone());
        }

        if let Ok(mut parent_mut) = parent.lock() {
            parent_mut.add_child(delegate.clone());
        }
    }

    // Build delegation tree for children
    for child in node.children() {
        let child_delegate = build_delegation_tree(child, Some(delegate.clone()));

        if let Ok(mut delegate_mut) = delegate.lock() {
            delegate_mut.add_child(child_delegate);
        }
    }

    delegate
}
