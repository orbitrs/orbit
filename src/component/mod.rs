//! Component model for Orbit UI framework
//!
//! This module contains all the types and traits related to the component model,
//! including lifecycle management, state, props, and rendering.

mod context;
mod error;
mod lifecycle;
mod node;
pub mod props;
mod state_tracking;

#[cfg(test)]
mod tests;

// Re-export component module contents
pub use context::{callback, Callback, ContextProvider};
pub use error::ComponentError;
pub use lifecycle::LifecycleManager;
// Import Node from our own node module instead of component_single
pub use node::Node;
pub use state_tracking::{
    StateChange, StateChanges, StateSnapshot, StateTracker, StateTrackingConfig, StateValue,
    ChangePriority
};

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    events::EventEmitter,
    layout::{LayoutNode, LayoutStyle},
    state::{State, StateContainer},
};

/// Global component ID counter for unique component identification
static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for component instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);

impl ComponentId {
    /// Generate a new unique component ID
    pub fn new() -> Self {
        Self(COMPONENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component#{}", self.0)
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
pub type LifecycleCallback = Box<dyn FnMut(&mut dyn AnyComponent) + Send + Sync>;

/// Lifecycle hook options for components
#[derive(Default)]
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

/// Wraps a component instance with its metadata
pub struct ComponentInstance {
    /// Component instance
    pub instance: Arc<Mutex<Box<dyn AnyComponent>>>,
    /// Current props
    pub props: Box<dyn Props>,
    /// Component type ID for type checking
    pub type_id: TypeId,
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
        let mut _instance = self.instance.lock().map_err(|_| {
            ComponentError::LockError("Failed to lock component for update".to_string())
        })?;

        // This is a simplified implementation - in practice we'd need more sophisticated prop handling
        Ok(())
    }

    /// Get the component's type ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

/// Context provided during component mounting
#[derive(Debug, Clone)]
pub struct MountContext {
    /// Component ID being mounted
    pub component_id: ComponentId,
    /// Parent component ID (if any)
    pub parent_id: Option<ComponentId>,
    /// Mount timestamp
    pub timestamp: std::time::Instant,
    /// Mount options and configuration
    pub options: MountOptions,
}

/// Options for component mounting
#[derive(Debug, Clone, Default)]
pub struct MountOptions {
    /// Whether to enable automatic state tracking
    pub enable_state_tracking: bool,
    /// Whether to register lifecycle hooks
    pub register_hooks: bool,
    /// Custom mounting data
    pub custom_data: HashMap<String, String>,
}

/// Context provided during component unmounting
#[derive(Debug, Clone)]
pub struct UnmountContext {
    /// Component ID being unmounted
    pub component_id: ComponentId,
    /// Parent component ID (if any)
    pub parent_id: Option<ComponentId>,
    /// Unmount timestamp
    pub timestamp: std::time::Instant,
    /// Unmount reason
    pub reason: UnmountReason,
    /// Whether cleanup should be forced
    pub force_cleanup: bool,
}

/// Reasons for component unmounting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmountReason {
    /// Component was explicitly removed
    Removed,
    /// Parent component was unmounted
    ParentUnmounted,
    /// Component replacement
    Replaced,
    /// Application shutdown
    Shutdown,
    /// Error condition
    Error,
}

impl MountContext {
    /// Create a new mount context
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            component_id,
            parent_id: None,
            timestamp: std::time::Instant::now(),
            options: MountOptions::default(),
        }
    }

    /// Create a mount context with parent
    pub fn with_parent(component_id: ComponentId, parent_id: ComponentId) -> Self {
        Self {
            component_id,
            parent_id: Some(parent_id),
            timestamp: std::time::Instant::now(),
            options: MountOptions::default(),
        }
    }

    /// Set mount options
    pub fn with_options(mut self, options: MountOptions) -> Self {
        self.options = options;
        self
    }
}

impl UnmountContext {
    /// Create a new unmount context
    pub fn new(component_id: ComponentId, reason: UnmountReason) -> Self {
        Self {
            component_id,
            parent_id: None,
            timestamp: std::time::Instant::now(),
            reason,
            force_cleanup: false,
        }
    }

    /// Create an unmount context with parent
    pub fn with_parent(component_id: ComponentId, parent_id: ComponentId, reason: UnmountReason) -> Self {
        Self {
            component_id,
            parent_id: Some(parent_id),
            timestamp: std::time::Instant::now(),
            reason,
            force_cleanup: false,
        }
    }

    /// Set force cleanup flag
    pub fn with_force_cleanup(mut self, force: bool) -> Self {
        self.force_cleanup = force;
        self
    }
}

/// Type-erased component trait for dynamic dispatch
pub trait AnyComponent: Send + Sync + std::any::Any {
    /// Get unique component ID for debugging and tracking
    fn component_id(&self) -> ComponentId;

    /// Get current lifecycle phase
    fn lifecycle_phase(&self) -> LifecyclePhase;

    /// Set lifecycle phase (framework internal)
    fn set_lifecycle_phase(&mut self, phase: LifecyclePhase);

    /// Request that this component be re-rendered
    fn request_update(&mut self) -> Result<(), ComponentError>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Get component type name for debugging
    fn type_name(&self) -> &'static str;

    // Lifecycle methods for dynamic dispatch
    /// Initialize the component (post-creation)
    fn any_initialize(&mut self) -> Result<(), ComponentError>;

    /// Mount component - called when component is first added to the tree
    fn any_mount(&mut self) -> Result<(), ComponentError>;

    /// Called before component updates with new props
    fn any_before_update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError>;

    /// Update component with new props (type-erased)
    fn any_update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError>;

    /// Called after the component has updated
    fn any_after_update(&mut self) -> Result<(), ComponentError>;    /// Called before component is unmounted
    fn any_before_unmount(&mut self) -> Result<(), ComponentError>;

    /// Unmount component - called when component is removed from the tree
    fn any_unmount(&mut self) -> Result<(), ComponentError>;

    // Enhanced lifecycle methods for dynamic dispatch
    /// Enhanced mount with context
    fn any_on_mount(&mut self, context: &MountContext) -> Result<(), ComponentError>;

    /// Before mount hook
    fn any_before_mount(&mut self) -> Result<(), ComponentError>;

    /// After mount hook  
    fn any_after_mount(&mut self) -> Result<(), ComponentError>;

    /// Enhanced update with state changes
    fn any_on_update(&mut self, changes: &StateChanges) -> Result<(), ComponentError>;

    /// Enhanced unmount with context
    fn any_on_unmount(&mut self, context: &UnmountContext) -> Result<(), ComponentError>;

    /// After unmount hook
    fn any_after_unmount(&mut self) -> Result<(), ComponentError>;
}

/// Enhanced component trait with improved lifecycle management
pub trait Component: AnyComponent + Send + Sync + std::any::Any {
    /// The props type for this component
    type Props: Props + Clone;

    /// Get unique component ID for debugging and tracking
    fn component_id(&self) -> ComponentId;

    /// Create a new component instance with enhanced tracking
    fn create(props: Self::Props, context: Context) -> Self
    where
        Self: Sized;

    /// Initialize the component - called immediately after creation
    /// Use this for setting up initial state and registering lifecycle hooks
    fn initialize(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }    /// Mount component - called when component is first added to the tree
    /// Automatic state change detection and update scheduling is enabled after this point
    fn mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Enhanced lifecycle hook - called when component is mounted with context
    fn on_mount(&mut self, _context: &MountContext) -> Result<(), ComponentError> {
        // Default implementation calls the basic mount
        self.mount()
    }

    /// Enhanced lifecycle hook - called before mount for initialization
    fn before_mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Enhanced lifecycle hook - called after mount for post-initialization
    fn after_mount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Called when component state changes and updates are needed
    fn state_changed(&mut self, _state_key: &str) -> Result<(), ComponentError> {
        // Default implementation requests a re-render
        Component::request_update(self)
    }

    /// Enhanced lifecycle hook - called when state changes are detected
    fn on_update(&mut self, _changes: &StateChanges) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Request that this component be re-rendered
    fn request_update(&mut self) -> Result<(), ComponentError> {
        // Implementation provided by the framework
        // This triggers the component's render cycle
        Ok(())
    }

    /// Check if component should update given new props
    /// Override for performance optimization
    fn should_update(&self, _new_props: &Self::Props) -> bool {
        // Default: always update
        // Override this for memoization and performance
        true
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
    }    /// Called before component is unmounted
    /// Automatic cleanup of state subscriptions happens after this
    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Enhanced lifecycle hook - called when component is unmounted with context
    fn on_unmount(&mut self, _context: &UnmountContext) -> Result<(), ComponentError> {
        // Default implementation calls the basic unmount
        self.unmount()
    }

    /// Enhanced lifecycle hook - called after unmount for final cleanup
    fn after_unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Unmount component - called when component is removed from the tree
    fn unmount(&mut self) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Perform automatic cleanup (called by framework)
    fn cleanup(&mut self) -> Result<(), ComponentError> {
        // Framework-provided cleanup:
        // - Remove state subscriptions
        // - Clear event listeners
        // - Release resources
        Ok(())
    }

    /// Render component - returns child nodes
    fn render(&self) -> Result<Vec<Node>, ComponentError>;

    /// Get current lifecycle phase
    fn lifecycle_phase(&self) -> LifecyclePhase {
        LifecyclePhase::Created // Default, overridden by framework
    }

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Base component functionality that all components share
#[derive(Debug)]
pub struct ComponentBase {
    /// Unique identifier for this component
    id: ComponentId,
    /// Current lifecycle phase
    lifecycle_phase: LifecyclePhase,
    /// Context for state and event management
    context: Context,
    /// Layout style for this component
    layout_style: LayoutStyle,
}

impl ComponentBase {
    /// Create a new component base with unique ID
    pub fn new(context: Context) -> Self {
        Self {
            id: ComponentId::new(),
            lifecycle_phase: LifecyclePhase::Created,
            context,
            layout_style: LayoutStyle::default(),
        }
    }

    /// Create a new component base with custom layout style
    pub fn new_with_layout(context: Context, layout_style: LayoutStyle) -> Self {
        Self {
            id: ComponentId::new(),
            lifecycle_phase: LifecyclePhase::Created,
            context,
            layout_style,
        }
    }

    /// Get the component ID
    pub fn id(&self) -> ComponentId {
        self.id
    }

    /// Get the current lifecycle phase
    pub fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase
    }

    /// Set the lifecycle phase (framework internal)
    pub fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
        self.lifecycle_phase = phase;
        self.context.set_lifecycle_phase(phase);
    }

    /// Get reference to the layout style
    pub fn layout_style(&self) -> &LayoutStyle {
        &self.layout_style
    }

    /// Get mutable reference to the layout style
    pub fn layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout_style
    }

    /// Set the layout style
    pub fn set_layout_style(&mut self, layout_style: LayoutStyle) {
        self.layout_style = layout_style;
    }

    /// Create a layout node for this component
    pub fn create_layout_node(&self) -> LayoutNode {
        LayoutNode::new(self.id, self.layout_style.clone())
    }

    /// Get reference to the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Get mutable reference to the context
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

/// Basic Component implementation for ComponentBase (primarily for testing)
impl Component for ComponentBase {
    type Props = (); // ComponentBase doesn't need props

    fn component_id(&self) -> ComponentId {
        self.id
    }

    fn create(_props: Self::Props, context: Context) -> Self {
        Self::new(context)
    }

    fn update(&mut self, _props: Self::Props) -> Result<(), ComponentError> {
        // ComponentBase doesn't have props to update
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // ComponentBase doesn't render anything by default
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase
    }
}

/// Automatic implementation of AnyComponent for all Components
impl<T: Component> AnyComponent for T {
    fn component_id(&self) -> ComponentId {
        Component::component_id(self)
    }

    fn lifecycle_phase(&self) -> LifecyclePhase {
        Component::lifecycle_phase(self)
    }

    fn set_lifecycle_phase(&mut self, _phase: LifecyclePhase) {
        // Default implementation does nothing
        // Concrete implementations can override this through AnyComponent if needed
    }

    fn request_update(&mut self) -> Result<(), ComponentError> {
        Component::request_update(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        Component::as_any(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        Component::as_any_mut(self)
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    // Lifecycle method delegations to the typed Component trait
    fn any_initialize(&mut self) -> Result<(), ComponentError> {
        Component::initialize(self)
    }

    fn any_mount(&mut self) -> Result<(), ComponentError> {
        Component::mount(self)
    }

    fn any_before_update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // Try to downcast the props to the component's Props type
        if let Some(typed_props) = props.as_any().downcast_ref::<T::Props>() {
            Component::before_update(self, typed_props)
        } else {
            Err(ComponentError::PropsMismatch {
                expected: TypeId::of::<T::Props>(),
                got: props.as_any().type_id(),
            })
        }
    }

    fn any_update(&mut self, props: Box<dyn Props>) -> Result<(), ComponentError> {
        // Try to downcast the props to the component's Props type
        if let Some(typed_props) = props.as_any().downcast_ref::<T::Props>() {
            Component::update(self, typed_props.clone())
        } else {
            Err(ComponentError::PropsMismatch {
                expected: TypeId::of::<T::Props>(),
                got: props.as_any().type_id(),
            })
        }
    }

    fn any_after_update(&mut self) -> Result<(), ComponentError> {
        Component::after_update(self)
    }    fn any_before_unmount(&mut self) -> Result<(), ComponentError> {
        Component::before_unmount(self)
    }

    fn any_unmount(&mut self) -> Result<(), ComponentError> {
        Component::unmount(self)
    }

    // Enhanced lifecycle method delegations
    fn any_on_mount(&mut self, context: &MountContext) -> Result<(), ComponentError> {
        Component::on_mount(self, context)
    }

    fn any_before_mount(&mut self) -> Result<(), ComponentError> {
        Component::before_mount(self)
    }

    fn any_after_mount(&mut self) -> Result<(), ComponentError> {
        Component::after_mount(self)
    }

    fn any_on_update(&mut self, changes: &StateChanges) -> Result<(), ComponentError> {
        Component::on_update(self, changes)
    }

    fn any_on_unmount(&mut self, context: &UnmountContext) -> Result<(), ComponentError> {
        Component::on_unmount(self, context)
    }

    fn any_after_unmount(&mut self) -> Result<(), ComponentError> {
        Component::after_unmount(self)
    }
}

/// Context passed to components providing access to state, events, and shared context
#[derive(Clone)]
pub struct Context {
    /// Unique ID for this context instance
    id: ComponentId,

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

    /// Update scheduler for batching state changes
    update_scheduler: Arc<Mutex<UpdateScheduler>>,
}

/// Manages batched updates for improved performance
#[derive(Debug, Default)]
pub struct UpdateScheduler {
    /// Components waiting for updates
    pending_updates: HashMap<ComponentId, bool>,
    /// Whether an update batch is currently scheduled
    batch_scheduled: bool,
}

impl UpdateScheduler {
    /// Schedule a component for update
    pub fn schedule_update(&mut self, component_id: ComponentId) {
        self.pending_updates.insert(component_id, true);
        if !self.batch_scheduled {
            self.batch_scheduled = true;
            // In a real implementation, this would schedule a microtask
            // For now, we'll handle updates immediately
        }
    }

    /// Check if a component has pending updates
    pub fn has_pending_update(&self, component_id: ComponentId) -> bool {
        self.pending_updates.contains_key(&component_id)
    }

    /// Clear pending updates for a component
    pub fn clear_pending(&mut self, component_id: ComponentId) {
        self.pending_updates.remove(&component_id);
    }

    /// Get all components with pending updates
    pub fn get_pending_components(&self) -> Vec<ComponentId> {
        self.pending_updates.keys().copied().collect()
    }
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
    /// Create a new context with unique ID and default state
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            state: StateContainer::new(),
            events: EventEmitter::new(),
            lifecycle_hooks: Arc::new(Mutex::new(LifecycleHooks::new())),
            lifecycle_phase: LifecyclePhase::Created,
            context_provider: ContextProvider::new(),
            update_scheduler: Arc::new(Mutex::new(UpdateScheduler::default())),
        }
    }

    /// Get the context ID
    pub fn id(&self) -> ComponentId {
        self.id
    }

    /// Get access to the state container
    pub fn state(&self) -> &StateContainer {
        &self.state
    }

    /// Get access to the event emitter
    pub fn events(&self) -> &EventEmitter {
        &self.events
    }

    /// Get the current lifecycle phase
    pub fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase
    }

    /// Set the lifecycle phase (framework internal)
    pub(crate) fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
        self.lifecycle_phase = phase;
    }

    /// Schedule a component update
    pub fn schedule_update(&self, component_id: ComponentId) {
        if let Ok(mut scheduler) = self.update_scheduler.lock() {
            scheduler.schedule_update(component_id);
        }
    }

    /// Check if component has pending updates
    pub fn has_pending_update(&self, component_id: ComponentId) -> bool {
        if let Ok(scheduler) = self.update_scheduler.lock() {
            scheduler.has_pending_update(component_id)
        } else {
            false
        }
    }

    /// Create a reactive state that triggers component updates
    pub fn create_reactive_state<T>(&self, initial_value: T, component_id: ComponentId) -> State<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let state = self.state.create(initial_value);

        // Set up state change listener to trigger component updates
        let scheduler = Arc::clone(&self.update_scheduler);
        state.on_change(move |_| {
            if let Ok(mut s) = scheduler.lock() {
                s.schedule_update(component_id);
            }
        });

        state
    }

    /// Register lifecycle hooks
    pub fn register_lifecycle_hooks<F>(&self, setup: F)
    where
        F: FnOnce(&mut LifecycleHooks),
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            setup(&mut hooks);
        }
    }

    /// Execute lifecycle hooks for a specific phase
    pub fn execute_lifecycle_hooks(&self, phase: LifecyclePhase, component: &mut dyn AnyComponent) {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            match phase {
                LifecyclePhase::Mounted => hooks.execute_mount(component),
                LifecyclePhase::BeforeUpdate => hooks.execute_before_update(component),
                LifecyclePhase::Updating => hooks.execute_update(component),
                LifecyclePhase::BeforeUnmount => hooks.execute_before_unmount(component),
                LifecyclePhase::Unmounted => hooks.execute_unmount(component),
                _ => {} // No hooks for other phases yet
            }
        }
    }

    /// Get context provider for parent-child communication
    pub fn context_provider(&self) -> &ContextProvider {
        &self.context_provider
    }
}
