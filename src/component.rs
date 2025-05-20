//! Component model for the Orbit UI framework

use std::{
    any::{TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    events::EventEmitter,
    state::{State, StateContainer},
};

/// Props trait - implemented by all component props types
pub trait Props: 'static + Send + Sync + std::any::Any {
    /// Get the type name for debugging
    fn type_name(&self) -> &'static str;

    /// Clone the props
    fn box_clone(&self) -> Box<dyn Props>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static + Clone + Send + Sync> Props for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn box_clone(&self) -> Box<dyn Props> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Marker trait to ensure props are sized
pub trait SizedProps: Props + Sized {}

/// Component trait - implemented by all UI components
pub trait Component: Send + Sync + std::any::Any {
    /// The props type for this component
    type Props: Props;

    /// Create a new component instance
    fn create(props: Self::Props, context: Context) -> Self where Self: Sized;
    
    /// Mount component - called when component is first added to the tree
    fn mount(&mut self) -> Result<(), ComponentError> { Ok(()) }
    
    /// Update component with new props
    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError>;
    
    /// Unmount component - called when component is removed from the tree
    fn unmount(&mut self) -> Result<(), ComponentError> { Ok(()) }
    
    /// Render component - returns child nodes
    fn render(&self) -> Result<Vec<Node>, ComponentError>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static + Send + Sync> Component for T where T: Component {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Context passed to components providing access to state and events
#[derive(Clone)]
pub struct Context {
    /// State container for managing component and app state
    state: StateContainer,
    
    /// Event emitter for handling UI events
    events: EventEmitter,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            state: StateContainer::new(),
            events: EventEmitter::new(),
        }
    }

    /// Create state with initial value
    pub fn state<T: 'static + Clone + Send + Sync>(&self, initial: T) -> State<T> {
        self.state.create(initial)
    }

    /// Get event emitter 
    pub fn events(&self) -> &EventEmitter {
        &self.events
    }
}

/// A node in the UI tree
pub struct Node {
    /// Component instance
    component: Option<ComponentInstance>,
    
    /// Node attributes
    attributes: HashMap<String, String>,
    
    /// Child nodes 
    children: Vec<Node>,
}

/// Wraps a component instance with its metadata
pub struct ComponentInstance {
    /// Component instance
    instance: Arc<Mutex<Box<dyn Component>>>,

    /// Current props
    props: Box<dyn Props>,
    
    /// Component type ID for type checking
    type_id: TypeId,
}

impl ComponentInstance {
    /// Create a new component instance
    pub fn new<C: Component + 'static>(
        instance: C, 
        props: C::Props
    ) -> Self {
        Self {
            instance: Arc::new(Mutex::new(Box::new(instance))),
            props: Box::new(props),
            type_id: TypeId::of::<C>(),
        }
    }

    /// Update the component instance with new props
    pub fn update<P: Props>(&mut self, props: P) -> Result<(), &'static str> {
        // Check that the props type matches
        if TypeId::of::<P>() != props.type_id() {
            return Err("Props type mismatch");
        }

        // Store the new props
        self.props = Box::new(props);

        Ok(())
    }
}

/// Factory for creating component instances
type ComponentFactory = Box<dyn Fn(Box<dyn Props>, Context) -> Box<dyn Component> + Send + Sync>;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ComponentError {
    TypeNotFound(TypeId),
    PropsMismatch { expected: TypeId, got: TypeId },
    DowncastError,
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeNotFound(type_id) => write!(f, "Component type {:?} not found", type_id),
            Self::PropsMismatch { expected, got } => write!(
                f,
                "Props type mismatch - expected {:?}, got {:?}",
                expected, got
            ),
            Self::DowncastError => write!(f, "Failed to downcast props"),
        }
    }
}

impl Error for ComponentError {}

/// Component registry for storing component factories
#[derive(Default)]
pub struct ComponentRegistry {
    components: HashMap<TypeId, ComponentFactory>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new()
        }
    }
    
    /// Register a component type
    pub fn register<C: Component + 'static>(&mut self, factory: impl Fn(C::Props, Context) -> C + Send + Sync + 'static) {
        let type_id = TypeId::of::<C>();
        
        let factory: ComponentFactory = Box::new(move |props, ctx| {
            // Safely downcast the props using as_any()
            let props = props.as_any()
                .downcast_ref::<C::Props>()
                .expect("Props type mismatch");
            
            Box::new(factory(props.clone(), ctx))
        });
        
        self.components.insert(type_id, factory);
    }
    
    /// Create a new component instance
    pub fn create_instance(
        &self, 
        type_id: TypeId,
        props: Box<dyn Props>,
        ctx: Context
    ) -> Result<Box<dyn Component>, ComponentError> {
        let factory = self.components.get(&type_id)
            .ok_or(ComponentError::TypeNotFound(type_id))?;

        Ok(factory(props, ctx))
    }

    /// Create a strongly typed component instance
    pub fn create_typed_instance<C: Component + 'static>(
        &self,
        props: C::Props,
        ctx: Context
    ) -> Result<C, ComponentError> {
        let type_id = TypeId::of::<C>();
        
        let factory = self.components.get(&type_id)
            .ok_or(ComponentError::TypeNotFound(type_id))?;
            
        let component = factory(Box::new(props), ctx);
        
        // Downcast to concrete type
        component
            .as_any()
            .downcast::<C>()
            .map(|boxed| *boxed)
            .map_err(|_| ComponentError::DowncastError)
    }
}
