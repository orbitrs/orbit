//! Component model for the Orbit UI framework

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    component::props::{PropValidationError, PropValidator},
    events::delegation::{EventDelegate, DelegatedEvent, PropagationPhase},
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

    /// Initialize the component - called immediately after creation
    /// Use this for setting up initial state and registering lifecycle hooks
    fn initialize(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Mount component - called when component is first added to the tree
    fn mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }
    
    /// Called before component updates with new props
    fn before_update(&mut self, _new_props: &Self::Props) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Update component with new props
    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError>;
    
    /// Called after the component has updated
    fn after_update(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }
    
    /// Called before component is unmounted
    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

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
    
    /// Lifecycle hooks container
    lifecycle_hooks: Arc<Mutex<LifecycleHooks>>,
    
    /// Current lifecycle phase
    lifecycle_phase: LifecyclePhase,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            state: StateContainer::new(),
            events: EventEmitter::new(),
            lifecycle_hooks: Arc::new(Mutex::new(LifecycleHooks::new())),
            lifecycle_phase: LifecyclePhase::Created,
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
    
    /// Register a callback for when the component is mounted
    pub fn on_mount<F>(&self, callback: F) 
    where 
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_mount(callback);
        }
    }
    
    /// Register a callback for before the component updates
    pub fn on_before_update<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_before_update(callback);
        }
    }
    
    /// Register a callback for when the component updates
    pub fn on_update<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_update(callback);
        }
    }
    
    /// Register a callback for before the component unmounts
    pub fn on_before_unmount<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_before_unmount(callback);
        }
    }
    
    /// Register a callback for when the component unmounts
    pub fn on_unmount<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_unmount(callback);
        }
    }
    
    /// Get current lifecycle phase
    pub fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase
    }
    
    /// Update the lifecycle phase
    pub(crate) fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
        self.lifecycle_phase = phase;
    }
    
    /// Execute lifecycle hooks for the given phase
    pub(crate) fn execute_lifecycle_hooks(&self, phase: LifecyclePhase, component: &mut dyn Component) {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            match phase {
                LifecyclePhase::Mounted => hooks.execute_mount(component),
                LifecyclePhase::BeforeUpdate => hooks.execute_before_update(component),
                LifecyclePhase::Updating => hooks.execute_update(component),
                LifecyclePhase::BeforeUnmount => hooks.execute_before_unmount(component),
                LifecyclePhase::Unmounting => hooks.execute_unmount(component),
                _ => {} // No hooks for other phases
            }
        }
    }
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
    
    /// Unique identifier for this node
    id: usize,
    
    /// Event delegate for this node
    event_delegate: Option<Arc<Mutex<EventDelegate>>>,
}

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
    
    /// Get a reference to the node's children
    pub fn children(&self) -> &[Node] {
        &self.children
    }
    
    /// Get the node's event delegate
    pub fn event_delegate(&self) -> Option<Arc<Mutex<EventDelegate>>> {
        self.event_delegate.clone()
    }
    
    /// Add a child node
    pub fn add_child(&mut self, mut child: Node) {
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
}

/// Box-erased component that handles the Props mismatch issue
pub trait AnyComponent: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn initialize(&mut self) -> Result<(), ComponentError>;
    fn mount(&mut self) -> Result<(), ComponentError>;
    fn before_update(&mut self, props: &Box<dyn Props>) -> Result<(), ComponentError>;
    fn update_props(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError>;
    fn after_update(&mut self) -> Result<(), ComponentError>;
    fn before_unmount(&mut self) -> Result<(), ComponentError>;
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
    
    fn initialize(&mut self) -> Result<(), ComponentError> {
        self.initialize()
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        self.mount()
    }
    
    fn before_update(&mut self, props: &Box<dyn Props>) -> Result<(), ComponentError> {
        // Try to downcast the props to the component's Props type
        if let Some(typed_props) = props.as_any().downcast_ref::<C::Props>() {
            self.before_update(typed_props)
        } else {
            Err(ComponentError::PropsMismatch {
                expected: TypeId::of::<C::Props>(),
                got: props.as_any().type_id(),
            })
        }
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
    
    fn after_update(&mut self) -> Result<(), ComponentError> {
        self.after_update()
    }
    
    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        self.before_unmount()
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        self.unmount()
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        self.render()
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

/// Lifecycle phase of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Component is created but not yet mounted
    Created,
    /// Component is being mounted
    Mounting,
    /// Component is fully mounted and operational
    Mounted,
    /// Component is about to be updated
    BeforeUpdate,
    /// Component is updating
    Updating,
    /// Component is about to be unmounted
    BeforeUnmount,
    /// Component is being unmounted
    Unmounting,
    /// Component is unmounted and inactive
    Unmounted,
}

/// Type for lifecycle callback functions
pub type LifecycleCallback = Box<dyn FnMut(&mut dyn Component) + Send + Sync>;

/// Lifecycle hook options for components
pub struct LifecycleHooks {
    /// Called when component is mounted to the DOM/renderer
    on_mount: Vec<LifecycleCallback>,
    /// Called before updating the DOM/renderer
    on_before_update: Vec<LifecycleCallback>,
    /// Called when props or state change
    on_update: Vec<LifecycleCallback>,
    /// Called when component is about to be unmounted
    on_before_unmount: Vec<LifecycleCallback>,
    /// Called when component is removed
    on_unmount: Vec<LifecycleCallback>,
}

impl Default for LifecycleHooks {
    fn default() -> Self {
        Self {
            on_mount: Vec::new(),
            on_before_update: Vec::new(),
            on_update: Vec::new(),
            on_before_unmount: Vec::new(),
            on_unmount: Vec::new(),
        }
    }
}

impl LifecycleHooks {
    /// Create a new empty set of lifecycle hooks
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a callback for when the component is mounted
    pub fn on_mount<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        self.on_mount.push(Box::new(callback));
    }

    /// Register a callback for before the component updates
    pub fn on_before_update<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        self.on_before_update.push(Box::new(callback));
    }

    /// Register a callback for when the component updates
    pub fn on_update<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        self.on_update.push(Box::new(callback));
    }

    /// Register a callback for before the component unmounts
    pub fn on_before_unmount<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        self.on_before_unmount.push(Box::new(callback));
    }

    /// Register a callback for when the component unmounts
    pub fn on_unmount<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn Component) + Send + Sync + 'static,
    {
        self.on_unmount.push(Box::new(callback));
    }

    /// Execute all mount callbacks
    pub(crate) fn execute_mount(&mut self, component: &mut dyn Component) {
        for callback in &mut self.on_mount {
            callback(component);
        }
    }

    /// Execute all before update callbacks
    pub(crate) fn execute_before_update(&mut self, component: &mut dyn Component) {
        for callback in &mut self.on_before_update {
            callback(component);
        }
    }

    /// Execute all update callbacks
    pub(crate) fn execute_update(&mut self, component: &mut dyn Component) {
        for callback in &mut self.on_update {
            callback(component);
        }
    }

    /// Execute all before unmount callbacks
    pub(crate) fn execute_before_unmount(&mut self, component: &mut dyn Component) {
        for callback in &mut self.on_before_unmount {
            callback(component);
        }
    }

    /// Execute all unmount callbacks
    pub(crate) fn execute_unmount(&mut self, component: &mut dyn Component) {
        for callback in &mut self.on_unmount {
            callback(component);
        }
    }
}
