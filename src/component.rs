//! Component model for the Orbit UI framework

use std::{
    any::TypeId,
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
    type Props: Props + Clone;

    /// Create a new component instance
    fn create(props: Self::Props, context: Context) -> Self
    where
        Self: Sized;

    /// Mount component - called when component is first added to the tree
    fn mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Update component with new props
    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError>;

    /// Unmount component - called when component is removed from the tree
    fn unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Render component - returns child nodes
    fn render(&self) -> Result<Vec<Node>, ComponentError>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
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

/// Box-erased component that handles the Props mismatch issue
pub trait AnyComponent: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn update_props(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError>;
    fn mount(&mut self) -> Result<(), ComponentError>;
    fn unmount(&mut self) -> Result<(), ComponentError>;
    fn render(&self) -> Result<Vec<Node>, ComponentError>;
}

// Implement AnyComponent for any type that implements Component
impl<C: Component + 'static> AnyComponent for C {
    fn as_any(&self) -> &dyn std::any::Any {
        self.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.as_any_mut()
    }

    fn update_props(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // Safely downcast the props using as_any()
        let props_any = props.as_any();
        match props_any.downcast_ref::<C::Props>() {
            Some(concrete_props) => {
                // Clone the props since we know C::Props implements Clone
                let props_clone = concrete_props.clone();
                self.update(props_clone)
            }
            None => Err(ComponentError::PropsMismatch {
                expected: TypeId::of::<C::Props>(),
                got: props_any.type_id(),
            }),
        }
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        Component::mount(self)
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        Component::unmount(self)
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        Component::render(self)
    }
}

/// Wraps a component instance with its metadata
pub struct ComponentInstance {
    /// Component instance
    instance: Arc<Mutex<Box<dyn AnyComponent>>>,

    /// Current props
    props: Box<dyn Props>,

    /// Component type ID for type checking
    type_id: TypeId,
}

impl ComponentInstance {
    /// Create a new component instance
    pub fn new<C: Component + 'static>(instance: C, props: C::Props) -> Self {
        Self {
            instance: Arc::new(Mutex::new(Box::new(instance) as Box<dyn AnyComponent>)),
            props: Box::new(props),
            type_id: TypeId::of::<C>(),
        }
    }

    /// Update the component instance with new props
    pub fn update<P: Props>(&mut self, props: P) -> Result<(), ComponentError> {
        let prop_type_id = TypeId::of::<P>();

        // Check that the props type matches
        if self.type_id != TypeId::of::<P>() {
            return Err(ComponentError::PropsMismatch {
                expected: self.type_id,
                got: prop_type_id,
            });
        }

        // Store the new props
        self.props = Box::new(props);

        // Update the component with the new props
        let mut instance = self.instance.lock().map_err(|_| {
            ComponentError::LockError("Failed to lock component instance".to_string())
        })?;

        instance.update_props(self.props.box_clone())
    }
}

/// Factory for creating component instances
type ComponentFactory = Box<
    dyn Fn(Box<dyn Props>, Context) -> Result<Box<dyn AnyComponent>, ComponentError> + Send + Sync,
>;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ComponentError {
    TypeNotFound(TypeId),
    PropsMismatch { expected: TypeId, got: TypeId },
    DowncastError,
    LockError(String),
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
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
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
            components: HashMap::new(),
        }
    }

    /// Register a component type
    pub fn register<C: Component + 'static>(
        &mut self,
        factory: impl Fn(C::Props, Context) -> C + Send + Sync + 'static,
    ) {
        let type_id = TypeId::of::<C>();

        let factory_boxed: ComponentFactory = Box::new(move |boxed_props, ctx| {
            // Safely downcast the props
            let props_any = boxed_props.as_any();
            match props_any.downcast_ref::<C::Props>() {
                Some(concrete_props) => {
                    // We can clone concrete_props because C::Props: Clone
                    let props_clone = concrete_props.clone();
                    // Create the component and wrap it
                    let component = factory(props_clone, ctx);
                    Ok(Box::new(component) as Box<dyn AnyComponent>)
                }
                None => Err(ComponentError::PropsMismatch {
                    expected: TypeId::of::<C::Props>(),
                    got: props_any.type_id(),
                }),
            }
        });

        self.components.insert(type_id, factory_boxed);
    }

    /// Create a new component instance
    pub fn create_instance(
        &self,
        type_id: TypeId,
        props: Box<dyn Props>,
        ctx: Context,
    ) -> Result<Box<dyn AnyComponent>, ComponentError> {
        let factory = self
            .components
            .get(&type_id)
            .ok_or(ComponentError::TypeNotFound(type_id))?;

        factory(props, ctx)
    }

    /// Create a strongly typed component instance
    pub fn create_typed_instance<C: Component + Clone + 'static>(
        &self,
        props: C::Props,
        ctx: Context,
    ) -> Result<C, ComponentError> {
        let type_id = TypeId::of::<C>();

        let factory = self
            .components
            .get(&type_id)
            .ok_or(ComponentError::TypeNotFound(type_id))?;

        // We know C::Props implements Clone because of the trait bound in Component
        let props_box = Box::new(props) as Box<dyn Props>;
        let component = factory(props_box, ctx)?;

        // Get a reference to the underlying component
        let any_ref = component.as_any();

        // Try to downcast to the concrete type
        match any_ref.downcast_ref::<C>() {
            Some(typed_ref) => {
                // Only works if C implements Clone
                Ok(typed_ref.clone())
            }
            None => Err(ComponentError::DowncastError),
        }
    }
}
