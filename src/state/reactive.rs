//! New scope-based reactive system for Orbit UI
//!
//! This module provides a fine-grained reactive system based on reactive scopes
//! rather than global registries, eliminating circular dependency issues.

use std::{
    cell::{RefCell, Ref, RefMut},
    collections::HashMap,
    rc::{Rc, Weak},
};

/// Errors that can occur in the reactive system
#[derive(Debug, Clone)]
pub enum SignalError {
    /// Signal has been dropped or is no longer accessible
    SignalDropped,
    /// Circular dependency detected
    CircularDependency,
    /// Invalid state transition
    InvalidState,
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalError::SignalDropped => write!(f, "Signal has been dropped"),
            SignalError::CircularDependency => write!(f, "Circular dependency detected"),
            SignalError::InvalidState => write!(f, "Invalid state transition"),
        }
    }
}

impl std::error::Error for SignalError {}

/// Unique identifier for reactive nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

/// Reactive scope that manages signals, effects, and computed values
pub struct ReactiveScope {
    inner: Rc<RefCell<ScopeInner>>,
}

struct ScopeInner {
    next_id: usize,
    nodes: HashMap<NodeId, Box<dyn ReactiveNode>>,
    dependencies: HashMap<NodeId, Vec<NodeId>>,
    dependents: HashMap<NodeId, Vec<NodeId>>,
    update_queue: Vec<NodeId>,
    in_update: bool,
}

trait ReactiveNode {
    fn update(&mut self) -> Result<(), SignalError>;
    fn invalidate(&mut self);
    fn is_dirty(&self) -> bool;
}

impl ReactiveScope {
    /// Create a new reactive scope
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ScopeInner {
                next_id: 0,
                nodes: HashMap::new(),
                dependencies: HashMap::new(),
                dependents: HashMap::new(),
                update_queue: Vec::new(),
                in_update: false,
            })),
        }
    }

    /// Generate a new unique node ID
    fn next_id(&self) -> NodeId {
        let mut inner = self.inner.borrow_mut();
        let id = NodeId(inner.next_id);
        inner.next_id += 1;
        id
    }

    /// Register a reactive node
    fn register_node(&self, node: Box<dyn ReactiveNode>) -> NodeId {
        let id = self.next_id();
        let mut inner = self.inner.borrow_mut();
        inner.nodes.insert(id, node);
        inner.dependencies.insert(id, Vec::new());
        inner.dependents.insert(id, Vec::new());
        id
    }

    /// Add a dependency relationship
    fn add_dependency(&self, dependent: NodeId, dependency: NodeId) {
        let mut inner = self.inner.borrow_mut();
        
        // Add to dependencies list
        if let Some(deps) = inner.dependencies.get_mut(&dependent) {
            if !deps.contains(&dependency) {
                deps.push(dependency);
            }
        }
        
        // Add to dependents list
        if let Some(dependents) = inner.dependents.get_mut(&dependency) {
            if !dependents.contains(&dependent) {
                dependents.push(dependent);
            }
        }
    }

    /// Mark a node as dirty and schedule updates
    fn invalidate_node(&self, id: NodeId) -> Result<(), SignalError> {
        let should_flush = {
            let mut inner = self.inner.borrow_mut();
            
            // Mark the node as dirty
            if let Some(node) = inner.nodes.get_mut(&id) {
                node.invalidate();
            }
            
            // Queue dependents for update
            if let Some(dependents) = inner.dependents.get(&id).cloned() {
                for dependent_id in dependents {
                    if !inner.update_queue.contains(&dependent_id) {
                        inner.update_queue.push(dependent_id);
                    }
                }
            }
            
            // Check if we should flush updates
            !inner.in_update
        };
        
        // Flush updates if not already in an update cycle
        if should_flush {
            self.flush_updates()?;
        }
        
        Ok(())
    }

    /// Flush all pending updates
    fn flush_updates(&self) -> Result<(), SignalError> {
        let mut inner = self.inner.borrow_mut();
        
        if inner.in_update {
            return Ok(()); // Prevent recursive updates
        }
        
        inner.in_update = true;
        
        while let Some(id) = inner.update_queue.pop() {
            if let Some(node) = inner.nodes.get_mut(&id) {
                if node.is_dirty() {
                    node.update()?;
                }
            }
        }
        
        inner.in_update = false;
        Ok(())
    }
}

impl Default for ReactiveScope {
    fn default() -> Self {
        Self::new()
    }
}

/// A reactive signal that holds a value
pub struct Signal<T> {
    id: NodeId,
    scope: Weak<RefCell<ScopeInner>>,
    value: Rc<RefCell<T>>,
    dirty: RefCell<bool>,
}

impl<T> Signal<T> {
    /// Get the current value of the signal
    pub fn get(&self) -> Ref<T> {
        // TODO: Track this read for reactive dependencies
        self.value.borrow()
    }

    /// Get a mutable reference to the signal's value
    pub fn get_mut(&self) -> RefMut<T> {
        self.value.borrow_mut()
    }

    /// Set the signal's value and trigger updates
    pub fn set(&self, value: T) -> Result<(), SignalError> {
        {
            let mut val = self.value.borrow_mut();
            *val = value;
        }
        
        // Mark as dirty and trigger updates
        *self.dirty.borrow_mut() = true;
        
        // TODO: Improve dependency tracking integration
        // if let Some(scope) = self.scope.upgrade() {
        //     // Trigger invalidation through scope
        // }
        
        Ok(())
    }

    /// Update the signal's value with a function
    pub fn update<F>(&self, f: F) -> Result<(), SignalError>
    where
        F: FnOnce(&mut T),
    {
        {
            let mut value = self.value.borrow_mut();
            f(&mut *value);
        }
        self.set_dirty()
    }

    fn set_dirty(&self) -> Result<(), SignalError> {
        *self.dirty.borrow_mut() = true;
        Ok(())
    }
}

impl<T> ReactiveNode for Signal<T> {
    fn update(&mut self) -> Result<(), SignalError> {
        // Signals don't need to update themselves, they trigger updates in dependents
        *self.dirty.borrow_mut() = false;
        Ok(())
    }

    fn invalidate(&mut self) {
        *self.dirty.borrow_mut() = true;
    }

    fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }
}

/// A reactive effect that runs when its dependencies change
pub struct Effect<F> {
    id: NodeId,
    scope: Weak<RefCell<ScopeInner>>,
    callback: RefCell<Option<F>>,
    dirty: RefCell<bool>,
}

impl<F> Effect<F>
where
    F: FnMut() + 'static,
{
    /// Execute the effect
    pub fn run(&self) -> Result<(), SignalError> {
        // Use a scope to ensure borrows are dropped before we borrow again
        let should_run = {
            let callback_ref = self.callback.borrow();
            callback_ref.is_some()
        };
        
        if should_run {
            let mut callback = self.callback.borrow_mut().take().unwrap();
            callback();
            *self.callback.borrow_mut() = Some(callback);
            *self.dirty.borrow_mut() = false;
        }
        Ok(())
    }
}

impl<F> ReactiveNode for Effect<F>
where
    F: FnMut() + 'static,
{
    fn update(&mut self) -> Result<(), SignalError> {
        if *self.dirty.borrow() {
            self.run()?;
        }
        Ok(())
    }

    fn invalidate(&mut self) {
        *self.dirty.borrow_mut() = true;
    }

    fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }
}

/// A computed value that derives from other reactive values
pub struct ReactiveComputed<T, F> {
    id: NodeId,
    scope: Weak<RefCell<ScopeInner>>,
    value: RefCell<Option<T>>,
    compute_fn: RefCell<Option<F>>,
    dirty: RefCell<bool>,
}

impl<T, F> ReactiveComputed<T, F>
where
    F: FnMut() -> T + 'static,
    T: 'static,
{
    /// Get the computed value, recalculating if necessary
    pub fn get(&self) -> Result<Ref<T>, SignalError> {
        if *self.dirty.borrow() || self.value.borrow().is_none() {
            self.recompute()?;
        }
        
        Ref::filter_map(self.value.borrow(), |opt| opt.as_ref())
            .map_err(|_| SignalError::InvalidState)
    }

    fn recompute(&self) -> Result<(), SignalError> {
        // Use scope to avoid multiple borrows
        let should_compute = {
            let compute_ref = self.compute_fn.borrow();
            compute_ref.is_some()
        };
        
        if should_compute {
            let mut compute_fn = self.compute_fn.borrow_mut().take().unwrap();
            let new_value = compute_fn();
            *self.value.borrow_mut() = Some(new_value);
            *self.compute_fn.borrow_mut() = Some(compute_fn);
            *self.dirty.borrow_mut() = false;
        }
        Ok(())
    }
}

impl<T, F> ReactiveNode for ReactiveComputed<T, F>
where
    F: FnMut() -> T + 'static,
    T: 'static,
{
    fn update(&mut self) -> Result<(), SignalError> {
        if *self.dirty.borrow() {
            self.recompute()?;
        }
        Ok(())
    }

    fn invalidate(&mut self) {
        *self.dirty.borrow_mut() = true;
    }

    fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }
}

/// Create a new signal with an initial value
pub fn create_signal<T>(scope: &ReactiveScope, initial_value: T) -> Signal<T>
where
    T: 'static,
{
    let signal = Signal {
        id: scope.next_id(),
        scope: Rc::downgrade(&scope.inner),
        value: Rc::new(RefCell::new(initial_value)),
        dirty: RefCell::new(false),
    };
    
    // Register with scope (simplified for now)
    signal
}

/// Create a new effect that runs when dependencies change
pub fn create_effect<F>(scope: &ReactiveScope, callback: F) -> Effect<F>
where
    F: FnMut() + 'static,
{
    let effect = Effect {
        id: scope.next_id(),
        scope: Rc::downgrade(&scope.inner),
        callback: RefCell::new(Some(callback)),
        dirty: RefCell::new(true), // Start dirty to run on creation
    };
    
    // Register with scope and run initially
    let _ = effect.run();
    effect
}

/// Create a new computed value
pub fn create_computed<T, F>(scope: &ReactiveScope, compute_fn: F) -> ReactiveComputed<T, F>
where
    F: FnMut() -> T + 'static,
    T: 'static,
{
    let computed = ReactiveComputed {
        id: scope.next_id(),
        scope: Rc::downgrade(&scope.inner),
        value: RefCell::new(None),
        compute_fn: RefCell::new(Some(compute_fn)),
        dirty: RefCell::new(true), // Start dirty to compute on first access
    };
    
    computed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation_and_access() {
        let scope = ReactiveScope::new();
        let signal = create_signal(&scope, 42);
        
        assert_eq!(*signal.get(), 42);
    }

    #[test]
    fn test_signal_update() {
        let scope = ReactiveScope::new();
        let signal = create_signal(&scope, 10);
        
        signal.update(|v| *v += 5).unwrap();
        assert_eq!(*signal.get(), 15);
    }

    #[test]
    fn test_effect_creation() {
        let scope = ReactiveScope::new();
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let _effect = create_effect(&scope, move || {
            *counter_clone.borrow_mut() += 1;
        });
        
        // Effect should run once on creation
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_computed_value() {
        let scope = ReactiveScope::new();
        let signal = create_signal(&scope, 5);
        let signal_clone = signal.value.clone();
        
        let computed = create_computed(&scope, move || {
            *signal_clone.borrow() * 2
        });
        
        assert_eq!(*computed.get().unwrap(), 10);
    }
}