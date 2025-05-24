use crate::component::ComponentInstance;
use crate::events::EventEmitter;
use crate::state::StateContainer;
use std::collections::HashMap;

/// Context passed to components providing access to state and events
#[derive(Clone, Debug)]
pub struct Context {
    /// State container for managing component and app state
    #[allow(dead_code)]
    state: StateContainer,

    /// Event emitter for handling UI events
    #[allow(dead_code)]
    events: EventEmitter,
}

/// A node in the UI tree
#[allow(dead_code)]
pub struct Node {
    /// Component instance
    component: Option<ComponentInstance>,

    /// Node attributes
    attributes: HashMap<String, String>,

    /// Child nodes
    children: Vec<Node>,

    /// Node ID
    id: usize,
}

impl Node {
    /// Get a reference to this node's children
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Get a mutable reference to this node's children
    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    /// Get this node's ID value
    pub fn id_value(&self) -> usize {
        self.id
    }
}

impl Default for Node {
    fn default() -> Self {
        static mut NEXT_ID: usize = 0;
        let id = unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        };

        Self {
            component: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            id,
        }
    }
}
