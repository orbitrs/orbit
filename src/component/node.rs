//! Enhanced node implementation with event delegation support

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::component::ComponentInstance;
use crate::events::delegation::EventDelegate;
use crate::events::Event;

/// A node in the UI tree with event delegation support
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Node {
    /// Component instance
    component: Option<ComponentInstance>,

    /// Node attributes
    attributes: HashMap<String, String>,

    /// Child nodes
    children: Vec<Node>,

    /// Unique identifier for this node
    id: usize,

    /// Event delegate for this node (not cloneable, so wrapped in Arc)
    event_delegate: Option<Arc<Mutex<EventDelegate>>>,
}

#[allow(dead_code)]
impl Node {
    /// Create a new node
    pub fn new(component: Option<ComponentInstance>) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Self {
            component,
            attributes: HashMap::new(),
            children: Vec::new(),
            id,
            event_delegate: Some(Arc::new(Mutex::new(EventDelegate::new(Some(id))))),
        }
    }

    /// Get the node's ID
    pub fn id(&self) -> Option<usize> {
        Some(self.id)
    }

    /// Get the node's ID as usize
    pub fn id_value(&self) -> usize {
        self.id
    }

    /// Get a reference to the node's children
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Get the node's children mutably
    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    /// Get the node's event delegate
    pub fn event_delegate(&self) -> Option<Arc<Mutex<EventDelegate>>> {
        self.event_delegate.clone()
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Node) {
        // Set up event delegation relationship
        if let Some(parent_delegate) = &self.event_delegate {
            if let Some(child_delegate) = &child.event_delegate {
                if let Ok(mut child_delegate) = child_delegate.lock() {
                    child_delegate.set_parent(parent_delegate.clone());
                }

                if let Ok(mut parent_delegate) = parent_delegate.lock() {
                    parent_delegate.add_child(child_delegate.clone());
                }
            }
        }

        self.children.push(child);
    }

    /// Add an attribute
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Dispatch an event to this node
    pub fn dispatch_event<E: Event + Clone + 'static>(&self, event: &E) {
        if let Some(delegate) = &self.event_delegate {
            if let Ok(delegate) = delegate.lock() {
                delegate.dispatch(event, self.id());
            }
        }
    }

    /// Get the component instance
    pub fn component(&self) -> Option<&ComponentInstance> {
        self.component.as_ref()
    }

    /// Get the component instance mutably
    pub fn component_mut(&mut self) -> Option<&mut ComponentInstance> {
        self.component.as_mut()
    }

    /// Get attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

impl Default for Node {
    fn default() -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Self {
            component: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            id,
            event_delegate: Some(Arc::new(Mutex::new(EventDelegate::new(Some(id))))),
        }
    }
}
