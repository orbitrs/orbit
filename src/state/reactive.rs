//! New scope-based reactive system for Orbit UI
//!
//! This module provides a fine-grained reactive system based on reactive scopes
//! rather than global registries, eliminating circular dependency issues.

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
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

/// Reactive scope that manages signals, effects, and computed values
///
/// This is a simplified implementation focused on basic functionality.
/// Advanced dependency tracking will be implemented in future versions.
pub struct ReactiveScope {
    // Reserved for future dependency tracking functionality
}

impl ReactiveScope {
    /// Create a new reactive scope
    pub fn new() -> Self {
        Self {
            // Future: Add dependency tracking infrastructure here
        }
    }
}

impl Default for ReactiveScope {
    fn default() -> Self {
        Self::new()
    }
}

/// A reactive signal that holds a value
pub struct Signal<T> {
    pub value: Rc<RefCell<T>>,
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

        // TODO: In a full implementation, this would trigger dependent updates
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

/// A reactive effect that runs when its dependencies change
pub struct Effect<F> {
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

/// A computed value that derives from other reactive values
pub struct ReactiveComputed<T, F> {
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

/// Create a new signal with an initial value
pub fn create_signal<T>(_scope: &ReactiveScope, initial_value: T) -> Signal<T>
where
    T: 'static,
{
    Signal {
        value: Rc::new(RefCell::new(initial_value)),
        dirty: RefCell::new(false),
    }
}

/// Create a new effect that runs when dependencies change
pub fn create_effect<F>(_scope: &ReactiveScope, callback: F) -> Effect<F>
where
    F: FnMut() + 'static,
{
    let effect = Effect {
        callback: RefCell::new(Some(callback)),
        dirty: RefCell::new(true), // Start dirty to run on creation
    };

    // Run initially
    let _ = effect.run();
    effect
}

/// Create a new computed value
pub fn create_computed<T, F>(_scope: &ReactiveScope, compute_fn: F) -> ReactiveComputed<T, F>
where
    F: FnMut() -> T + 'static,
    T: 'static,
{
    ReactiveComputed {
        value: RefCell::new(None),
        compute_fn: RefCell::new(Some(compute_fn)),
        dirty: RefCell::new(true), // Start dirty to compute on first access
    }
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

        let computed = create_computed(&scope, move || *signal_clone.borrow() * 2);

        assert_eq!(*computed.get().unwrap(), 10);
    }
}
