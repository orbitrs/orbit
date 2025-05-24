//! Component model for Orbit UI framework
//!
//! This module contains all the types and traits related to the component model,
//! including lifecycle management, state, props, and rendering.

mod context;
mod error;
mod lifecycle;
mod node;
pub mod props;

#[cfg(test)]
mod tests;

// Re-export component module contents
pub use context::{callback, Callback, ContextProvider};
pub use error::ComponentError;
pub use lifecycle::LifecycleManager;
// Import Node from component_single
pub use crate::component_single::Node;

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    events::EventEmitter,
    state::{State, StateContainer},
};

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
pub type LifecycleCallback = Box<dyn FnMut(&mut dyn AnyComponent) + Send + Sync>;

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

impl std::fmt::Debug for LifecycleHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LifecycleHooks")
            .field("on_mount", &format!("[{} callbacks]", self.on_mount.len()))
            .field(
                "on_before_update",
                &format!("[{} callbacks]", self.on_before_update.len()),
            )
            .field(
                "on_update",
                &format!("[{} callbacks]", self.on_update.len()),
            )
            .field(
                "on_before_unmount",
                &format!("[{} callbacks]", self.on_before_unmount.len()),
            )
            .field(
                "on_unmount",
                &format!("[{} callbacks]", self.on_unmount.len()),
            )
            .finish()
    }
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
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        self.on_mount.push(Box::new(callback));
    }

    /// Register a callback for before the component updates
    pub fn on_before_update<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        self.on_before_update.push(Box::new(callback));
    }

    /// Register a callback for when the component updates
    pub fn on_update<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        self.on_update.push(Box::new(callback));
    }

    /// Register a callback for before the component unmounts
    pub fn on_before_unmount<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        self.on_before_unmount.push(Box::new(callback));
    }

    /// Register a callback for when the component unmounts
    pub fn on_unmount<F>(&mut self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        self.on_unmount.push(Box::new(callback));
    }

    /// Execute all mount callbacks
    pub(crate) fn execute_mount(&mut self, component: &mut dyn AnyComponent) {
        for callback in &mut self.on_mount {
            callback(component);
        }
    }

    /// Execute all before update callbacks
    pub(crate) fn execute_before_update(&mut self, component: &mut dyn AnyComponent) {
        for callback in &mut self.on_before_update {
            callback(component);
        }
    }

    /// Execute all update callbacks
    pub(crate) fn execute_update(&mut self, component: &mut dyn AnyComponent) {
        for callback in &mut self.on_update {
            callback(component);
        }
    }

    /// Execute all before unmount callbacks
    pub(crate) fn execute_before_unmount(&mut self, component: &mut dyn AnyComponent) {
        for callback in &mut self.on_before_unmount {
            callback(component);
        }
    }

    /// Execute all unmount callbacks
    pub(crate) fn execute_unmount(&mut self, component: &mut dyn AnyComponent) {
        for callback in &mut self.on_unmount {
            callback(component);
        }
    }
}

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

/// Context passed to components providing access to state, events, and shared context
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

    /// Context provider for parent-child communication
    context_provider: ContextProvider,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("state", &self.state)
            .field("events", &self.events)
            .field("lifecycle_hooks", &"[LifecycleHooks]")
            .field("lifecycle_phase", &self.lifecycle_phase)
            .field("context_provider", &self.context_provider)
            .finish()
    }
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
            context_provider: ContextProvider::new(),
        }
    }

    /// Create a new context with a parent context provider
    pub fn with_parent(parent: &Context) -> Self {
        Self {
            state: StateContainer::new(),
            events: EventEmitter::new(),
            lifecycle_hooks: Arc::new(Mutex::new(LifecycleHooks::new())),
            lifecycle_phase: LifecyclePhase::Created,
            context_provider: ContextProvider::with_parent(parent.context_provider.clone()),
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
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_mount(callback);
        }
    }

    /// Register a callback for before the component updates
    pub fn on_before_update<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_before_update(callback);
        }
    }

    /// Register a callback for when the component updates
    pub fn on_update<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_update(callback);
        }
    }

    /// Register a callback for before the component unmounts
    pub fn on_before_unmount<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_before_unmount(callback);
        }
    }

    /// Register a callback for when the component unmounts
    pub fn on_unmount<F>(&self, callback: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks.on_unmount(callback);
        }
    }

    /// Set the current lifecycle phase
    pub fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
        self.lifecycle_phase = phase;
    }

    /// Execute lifecycle hooks for a specific phase
    pub fn execute_lifecycle_hooks(&self, phase: LifecyclePhase, component: &mut dyn AnyComponent) {
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

// Removed ComponentWrapper as it was causing lifetime issues
// We can reimplement it later if needed

// Node struct is now defined in component_single.rs

/// Trait for type-erased components that can participate in the component lifecycle
pub trait AnyComponent: Send + Sync + 'static {
    /// Initialize the component
    fn initialize(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Mount the component
    fn mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Update the component with new props
    fn update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError>;

    /// Called before component updates with new props
    fn before_update(&mut self, _props: &dyn Props) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Called after component updates
    fn after_update(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Called before component unmounts
    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Unmount the component
    fn unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Render the component
    fn render(&self) -> Result<Vec<Node>, ComponentError>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: Component + 'static> AnyComponent for T {
    fn initialize(&mut self) -> Result<(), ComponentError> {
        Component::initialize(self)
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        Component::mount(self)
    }

    fn update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // We need to downcast the props to the correct type
        if let Some(typed_props) = props.as_any().downcast_ref::<T::Props>() {
            Component::update(self, typed_props.clone())
        } else {
            Err(ComponentError::InvalidPropsType)
        }
    }

    fn before_update(&mut self, props: &dyn Props) -> Result<(), ComponentError> {
        // We need to downcast the props to the correct type
        if let Some(typed_props) = props.as_any().downcast_ref::<T::Props>() {
            Component::before_update(self, typed_props)
        } else {
            Err(ComponentError::InvalidPropsType)
        }
    }

    fn after_update(&mut self) -> Result<(), ComponentError> {
        Component::after_update(self)
    }

    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        Component::before_unmount(self)
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        Component::unmount(self)
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        Component::render(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AnyComponent for Box<dyn AnyComponent> {
    fn initialize(&mut self) -> Result<(), ComponentError> {
        (**self).initialize()
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        (**self).mount()
    }

    fn update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        (**self).update(props)
    }

    fn before_update(&mut self, props: &dyn Props) -> Result<(), ComponentError> {
        (**self).before_update(props)
    }

    fn after_update(&mut self) -> Result<(), ComponentError> {
        (**self).after_update()
    }

    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        (**self).before_unmount()
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        (**self).unmount()
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        (**self).render()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

    /// Initialize the component
    pub fn initialize(&mut self) -> Result<(), ComponentError> {
        if let Ok(mut instance) = self.instance.lock() {
            instance.initialize()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Mount the component
    pub fn mount(&mut self) -> Result<(), ComponentError> {
        if let Ok(mut instance) = self.instance.lock() {
            instance.mount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Update component with new props
    pub fn update<P: Props>(&mut self, props: P) -> Result<(), ComponentError> {
        let prop_type_id = TypeId::of::<P>();

        // Check that the props type matches the component's expected props type
        if prop_type_id != self.type_id {
            return Err(ComponentError::PropsMismatch {
                expected: self.type_id,
                got: prop_type_id,
            });
        }

        // Get boxed props
        let boxed_props = Box::new(props);

        // Before update phase
        if let Ok(mut instance) = self.instance.lock() {
            instance.before_update(boxed_props.as_ref())?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ));
        }

        // Store the new props
        self.props = boxed_props;

        // Update the component with the new props
        if let Ok(mut instance) = self.instance.lock() {
            instance.update(self.props.box_clone())?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ));
        }

        // After update phase
        if let Ok(mut instance) = self.instance.lock() {
            instance.after_update()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Update component with boxed props
    pub fn update_boxed(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // Check that the props type matches the component's expected props type
        if props.as_any().type_id() != self.type_id {
            return Err(ComponentError::PropsMismatch {
                expected: self.type_id,
                got: props.as_any().type_id(),
            });
        }

        // Before update phase
        if let Ok(mut instance) = self.instance.lock() {
            instance.before_update(props.as_ref())?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ));
        }

        // Store the new props
        self.props = props.box_clone();

        // Update the component with the new props
        if let Ok(mut instance) = self.instance.lock() {
            instance.update(props)?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ));
        }

        // After update phase
        if let Ok(mut instance) = self.instance.lock() {
            instance.after_update()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Prepare for unmounting the component
    pub fn before_unmount(&mut self) -> Result<(), ComponentError> {
        if let Ok(mut instance) = self.instance.lock() {
            instance.before_unmount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Unmount the component
    pub fn unmount(&mut self) -> Result<(), ComponentError> {
        if let Ok(mut instance) = self.instance.lock() {
            instance.unmount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    /// Render the component
    pub fn render(&self) -> Result<Vec<Node>, ComponentError> {
        if let Ok(instance) = self.instance.lock() {
            instance.render()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }
}

/// Factory for creating component instances
type ComponentFactory = Box<
    dyn Fn(Box<dyn Props>, Context) -> Result<Box<dyn AnyComponent>, ComponentError> + Send + Sync,
>;

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

// Implement AnyComponent for ComponentInstance to fix component lifecycle issues
impl AnyComponent for ComponentInstance {
    fn initialize(&mut self) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.initialize()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.mount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.update(props)
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn before_update(&mut self, props: &dyn Props) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.before_update(props)
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn after_update(&mut self) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.after_update()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.before_unmount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        // Delegate to the contained instance
        if let Ok(mut instance) = self.instance.lock() {
            instance.unmount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // Delegate to the contained instance
        if let Ok(instance) = self.instance.lock() {
            instance.render()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
