//! Enhanced component context implementation
//!
//! This module defines the core Context structure that integrates state management,
//! event handling, and component updates into a single unified system.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    component::{update_scheduler::{UpdatePriority, UpdateScheduler}, AnyComponent, ComponentId, LifecyclePhase},
    events::EventEmitter,
    state::{ReactiveScope, StateContainer},
};

/// A context hook that can be executed at specific lifecycle phases
type LifecycleHook = Box<dyn FnMut(&mut dyn AnyComponent) + Send + Sync>;
type LifecycleHooks = HashMap<LifecyclePhase, Vec<LifecycleHook>>;

/// Context passed to components providing access to state and events
#[derive(Clone)]
pub struct Context {
    /// State container for managing component and app state
    state: StateContainer,

    /// Reactive scope for fine-grained reactivity
    reactive_scope: Arc<ReactiveScope>,

    /// Event emitter for handling UI events
    events: EventEmitter,

    /// Update scheduler for batching component updates
    update_scheduler: UpdateScheduler,

    /// Component lifecycle hooks
    lifecycle_hooks: Arc<Mutex<LifecycleHooks>>,

    /// Current lifecycle phase
    lifecycle_phase: Arc<RwLock<LifecyclePhase>>,

    /// Component ID that owns this context
    component_id: Option<ComponentId>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new component context
    pub fn new() -> Self {
        Self {
            state: StateContainer::new(),
            reactive_scope: Arc::new(ReactiveScope::new()),
            events: EventEmitter::new(),
            update_scheduler: UpdateScheduler::new(),
            lifecycle_hooks: Arc::new(Mutex::new(HashMap::new())),
            lifecycle_phase: Arc::new(RwLock::new(LifecyclePhase::Created)),
            component_id: None,
        }
    }

    /// Create a context for a specific component
    pub fn for_component(component_id: ComponentId) -> Self {
        let mut context = Self::new();
        context.component_id = Some(component_id);
        context
    }

    /// Get a reference to the state container
    pub fn state(&self) -> &StateContainer {
        &self.state
    }

    /// Get a mutable reference to the state container
    pub fn state_mut(&mut self) -> &mut StateContainer {
        &mut self.state
    }

    /// Get a reference to the reactive scope
    pub fn reactive_scope(&self) -> &Arc<ReactiveScope> {
        &self.reactive_scope
    }

    /// Get a reference to the event emitter
    pub fn events(&self) -> &EventEmitter {
        &self.events
    }

    /// Get a mutable reference to the event emitter
    pub fn events_mut(&mut self) -> &mut EventEmitter {
        &mut self.events
    }

    /// Get the update scheduler
    pub fn update_scheduler(&self) -> &UpdateScheduler {
        &self.update_scheduler
    }

    /// Get current lifecycle phase
    pub fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase.read().map(|phase| *phase).unwrap_or_else(|_| {
            // If we can't read the lock, default to Created
            // This should never happen in normal operation
            eprintln!("Error reading lifecycle phase, defaulting to Created");
            LifecyclePhase::Created
        })
    }

    /// Set current lifecycle phase
    pub fn set_lifecycle_phase(&self, phase: LifecyclePhase) {
        if let Ok(mut current_phase) = self.lifecycle_phase.write() {
            *current_phase = phase;
        }
    }

    /// Register a hook to be called at a specific lifecycle phase
    pub fn register_lifecycle_hook<F>(&self, phase: LifecyclePhase, hook: F)
    where
        F: FnMut(&mut dyn AnyComponent) + Send + Sync + 'static,
    {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            hooks
                .entry(phase)
                .or_insert_with(Vec::new)
                .push(Box::new(hook));
        }
    }

    /// Execute all hooks for the current lifecycle phase
    pub fn execute_lifecycle_hooks(&self, phase: LifecyclePhase, component: &mut dyn AnyComponent) {
        if let Ok(mut hooks) = self.lifecycle_hooks.lock() {
            if let Some(phase_hooks) = hooks.get_mut(&phase) {
                for hook in phase_hooks.iter_mut() {
                    hook(component);
                }
            }
        }
    }

    /// Request a component update with normal priority
    pub fn request_update(&self, component_id: ComponentId) -> Result<(), String> {
        self.update_scheduler.schedule_update(component_id, UpdatePriority::Normal)
    }

    /// Request a critical update (executed immediately if possible)
    pub fn request_critical_update(&self, component_id: ComponentId) -> Result<(), String> {
        self.update_scheduler.schedule_update(component_id, UpdatePriority::Critical)
    }

    /// Process all pending updates
    pub fn process_updates<F>(&self, update_fn: F) -> Result<usize, String>
    where
        F: FnMut(ComponentId) -> Result<(), String>,
    {
        self.update_scheduler.process_updates(update_fn)
    }

    /// Get current component ID if set
    pub fn component_id(&self) -> Option<ComponentId> {
        self.component_id
    }

    /// Set the component ID for this context
    pub fn set_component_id(&mut self, id: ComponentId) {
        self.component_id = Some(id);
    }

    /// Create a child context with this context as parent
    pub fn create_child_context(&self, child_id: ComponentId) -> Self {
        let mut child = self.clone();
        child.set_component_id(child_id);
        child
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("state", &self.state)
            .field("reactive_scope", &self.reactive_scope)
            .field("events", &self.events)
            .field("update_scheduler", &self.update_scheduler)
            .field("lifecycle_hooks", &format!("<{} hooks>", 
                self.lifecycle_hooks.lock()
                    .map(|hooks| hooks.len())
                    .unwrap_or(0)))
            .field("lifecycle_phase", &self.lifecycle_phase)
            .field("component_id", &self.component_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{ComponentError, ComponentId, LifecyclePhase, Props};
    
    // Mock component for testing
    struct TestComponent {
        id: ComponentId,
        context: Context,
        lifecycle_events: Vec<String>,
        phase: LifecyclePhase,
    }
    
    impl TestComponent {
        fn new(context: Context) -> Self {
            let id = ComponentId::new();
            Self {
                id,
                context,
                lifecycle_events: Vec::new(),
                phase: LifecyclePhase::Created,
            }
        }
    }
    
    impl AnyComponent for TestComponent {
        fn component_id(&self) -> ComponentId {
            self.id
        }
        
        fn lifecycle_phase(&self) -> LifecyclePhase {
            self.phase
        }
        
        fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
            self.phase = phase;
        }
        
        fn request_update(&mut self) -> Result<(), ComponentError> {
            self.context
                .request_update(self.id)
                .map_err(|e| ComponentError::UpdateError(e))
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        
        fn type_name(&self) -> &'static str {
            "TestComponent"
        }
        
        fn any_initialize(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("initialize".to_string());
            Ok(())
        }
        
        fn any_mount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("mount".to_string());
            Ok(())
        }
        
        fn any_before_update(
            &mut self, 
            _props: Box<dyn Props>
        ) -> Result<(), ComponentError> {
            self.lifecycle_events.push("before_update".to_string());
            Ok(())
        }
        
        fn any_update(
            &mut self,
            _props: Box<dyn Props>
        ) -> Result<(), ComponentError> {
            self.lifecycle_events.push("update".to_string());
            Ok(())
        }
        
        fn any_after_update(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("after_update".to_string());
            Ok(())
        }
        
        fn any_before_unmount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("before_unmount".to_string());
            Ok(())
        }
        
        fn any_unmount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("unmount".to_string());
            Ok(())
        }
        
        fn any_on_mount(&mut self, _context: &crate::component::MountContext) -> Result<(), ComponentError> {
            self.lifecycle_events.push("on_mount".to_string());
            Ok(())
        }

        fn any_before_mount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("before_mount".to_string());
            Ok(())
        }

        fn any_after_mount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("after_mount".to_string());
            Ok(())
        }

        fn any_on_update(&mut self, _changes: &crate::component::state_tracking::StateChanges) -> Result<(), ComponentError> {
            self.lifecycle_events.push("on_update".to_string());
            Ok(())
        }

        fn any_on_unmount(&mut self, _context: &crate::component::UnmountContext) -> Result<(), ComponentError> {
            self.lifecycle_events.push("on_unmount".to_string());
            Ok(())
        }

        fn any_after_unmount(&mut self) -> Result<(), ComponentError> {
            self.lifecycle_events.push("after_unmount".to_string());
            Ok(())
        }
    }
    
    #[test]
    fn test_context_lifecycle_hooks() {
        // Create a context
        let context = Context::new();
        
        // Register a couple of hooks
        context.register_lifecycle_hook(LifecyclePhase::Mounting, |component| {
            let any = component.as_any_mut();
            if let Some(test_component) = any.downcast_mut::<TestComponent>() {
                test_component.lifecycle_events.push("mounting_hook".to_string());
            }
        });
        
        context.register_lifecycle_hook(LifecyclePhase::Unmounting, |component| {
            let any = component.as_any_mut();
            if let Some(test_component) = any.downcast_mut::<TestComponent>() {
                test_component.lifecycle_events.push("unmounting_hook".to_string());
            }
        });
        
        // Create a test component
        let mut component = TestComponent::new(context.clone());
        
        // Execute mounting hooks
        context.execute_lifecycle_hooks(LifecyclePhase::Mounting, &mut component);
        
        // Verify the hook was executed
        assert!(component.lifecycle_events.contains(&"mounting_hook".to_string()));
        
        // Execute unmounting hooks
        context.execute_lifecycle_hooks(LifecyclePhase::Unmounting, &mut component);
        
        // Verify the hook was executed
        assert!(component.lifecycle_events.contains(&"unmounting_hook".to_string()));
    }
    
    #[test]
    fn test_context_update_scheduling() {
        // Create a context
        let context = Context::new();
        
        // Create some component IDs
        let c1 = ComponentId::new();
        let c2 = ComponentId::new();
        
        // Schedule updates
        context.request_update(c1).unwrap();
        context.request_critical_update(c2).unwrap();
        
        // Process updates
        let mut processed = Vec::new();
        let updated = context.process_updates(|id| {
            processed.push(id);
            Ok(())
        }).unwrap();
        
        // Verify updates were processed
        assert_eq!(updated, 2);
        assert_eq!(processed.len(), 2);
        
        // Critical update should be first
        assert_eq!(processed[0], c2);
        assert_eq!(processed[1], c1);
    }
    
    #[test]
    fn test_child_context() {
        // Create a parent context
        let parent_context = Context::new();
        let parent_id = ComponentId::new();
        
        // Create a child context
        let child_id = ComponentId::new();
        let child_context = parent_context.create_child_context(child_id);
        
        // Verify the child has the correct ID
        assert_eq!(child_context.component_id(), Some(child_id));
        
        // Verify they share the same update scheduler
        parent_context.request_update(parent_id).unwrap();
        
        // Both contexts should see the pending update
        assert!(parent_context.update_scheduler.has_pending_updates().unwrap());
        assert!(child_context.update_scheduler.has_pending_updates().unwrap());
    }
}
