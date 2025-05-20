//! Component model for the Orbit UI framework

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    events::EventEmitter,
    renderer::Renderer,
    state::{State, StateContainer},
};

/// Trait for all UI components in Orbit
pub trait Component: 'static {
    /// The type of the component's properties
    type Props: Props;

    /// Create a new component with the given properties
    fn new(props: Self::Props) -> Self;

    /// Render the component and return a virtual DOM node
    fn render(&self) -> Node;

    /// Called before the component is mounted
    fn before_mount(&mut self) {}

    /// Called after the component is mounted
    fn mounted(&mut self) {}

    /// Called before the component is updated
    fn before_update(&mut self) -> bool {
        true // Return false to prevent update
    }

    /// Called after the component is updated
    fn updated(&mut self) {}

    /// Called before the component is unmounted
    fn before_unmount(&mut self) {}

    /// Called after the component is unmounted
    fn unmounted(&mut self) {}

    /// Access the component's state container
    fn state(&self) -> &StateContainer;

    /// Access the component's state container mutably
    fn state_mut(&mut self) -> &mut StateContainer;

    /// Access the component's event emitter
    fn events(&self) -> &EventEmitter;
}

/// Props trait for component properties
pub trait Props: 'static + Clone {}

/// A virtual DOM node representing the component's rendered output
#[derive(Debug, Clone)]
pub enum Node {
    /// An HTML element
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        events: HashMap<String, Box<dyn Fn() + 'static>>,
        children: Vec<Node>,
    },
    /// A text node
    Text(String),
    /// A component instance
    Component {
        id: TypeId,
        instance: Arc<Mutex<Box<dyn Component>>>,
    },
}

/// A type-erased component instance
pub struct AnyComponent {
    /// The underlying component instance
    instance: Arc<Mutex<Box<dyn Component<Props = Box<dyn Props>>>>>,
}

/// Component factory type
type ComponentFactory = Box<dyn Fn(Context) -> Box<dyn Component<Props = Box<dyn Props>>>>;

/// Component context providing access to framework features
#[derive(Clone)]
pub struct Context {
    /// State management
    pub state: StateContainer,
    /// Event handling
    pub events: EventEmitter,
    /// Rendering backend
    pub renderer: Arc<Box<dyn Renderer>>,
}

impl Context {
    /// Create a new component context
    pub fn new(renderer: Box<dyn Renderer>) -> Self {
        Self {
            state: StateContainer::new(),
            events: EventEmitter::new(),
            renderer: Arc::new(renderer),
        }
    }

    /// Create a reactive state value
    pub fn state<T: 'static>(&self, initial: T) -> State<T> {
        self.state.create(initial)
    }
}

/// Registry for all components in the application
pub struct ComponentRegistry {
    // Maps component names to their factory functions
    components: HashMap<TypeId, ComponentFactory>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component with the given name
    pub fn register<T: Component + 'static>(&mut self, name: &str, factory: fn() -> T) {
        self.components.insert(
            name.to_string(),
            Box::new(move || Box::new(factory()) as Box<dyn std::any::Any>),
        );
    }

    /// Get a component by name
    pub fn get(&self, name: &str) -> Option<Box<dyn std::any::Any>> {
        self.components.get(name).map(|factory| factory())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
