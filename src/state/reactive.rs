//! New scope-based reactive system for Orbit UI
//!
//! This module provides a fine-grained reactive system based on reactive scopes
//! rather than global registries, eliminating circular dependency issues.

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
    pub value: Arc<RwLock<T>>,
    dirty: Arc<RwLock<bool>>,
}

// Explicit Send + Sync implementations
unsafe impl<T: Send + Sync> Send for Signal<T> {}
unsafe impl<T: Send + Sync> Sync for Signal<T> {}

impl<T> Signal<T>
where
    T: Send + Sync + 'static,
{
    /// Get the current value of the signal
    pub fn get(&self) -> RwLockReadGuard<T> {
        // TODO: Track this read for reactive dependencies
        self.value.read().unwrap()
    }

    /// Get a mutable reference to the signal's value
    pub fn get_mut(&self) -> RwLockWriteGuard<T> {
        self.value.write().unwrap()
    }

    /// Set the signal's value and trigger updates
    pub fn set(&self, value: T) -> Result<(), SignalError> {
        {
            let mut val = self.value.write().unwrap();
            *val = value;
        }

        // Mark as dirty and trigger updates
        *self.dirty.write().unwrap() = true;

        // TODO: In a full implementation, this would trigger dependent updates
        Ok(())
    }

    /// Update the signal's value with a function
    pub fn update<F>(&self, f: F) -> Result<(), SignalError>
    where
        F: FnOnce(&mut T),
    {
        {
            let mut value = self.value.write().unwrap();
            f(&mut *value);
        }
        self.set_dirty()
    }

    fn set_dirty(&self) -> Result<(), SignalError> {
        *self.dirty.write().unwrap() = true;
        Ok(())
    }
}

/// A reactive effect that runs when its dependencies change
pub struct Effect<F> {
    callback: Mutex<Option<F>>,
    dirty: Arc<RwLock<bool>>,
}

// Explicit Send + Sync implementations
unsafe impl<F: Send + Sync> Send for Effect<F> {}
unsafe impl<F: Send + Sync> Sync for Effect<F> {}

impl Effect<Box<dyn FnMut() + Send + Sync + 'static>> {
    /// Execute the effect
    pub fn run(&self) -> Result<(), SignalError> {
        // Check if we should run
        let should_run = {
            let callback_ref = self.callback.lock().unwrap();
            callback_ref.is_some()
        };

        if should_run {
            let mut callback = self.callback.lock().unwrap().take().unwrap();
            callback();
            *self.callback.lock().unwrap() = Some(callback);
            *self.dirty.write().unwrap() = false;
        }
        Ok(())
    }
}

/// A computed value that derives from other reactive values
pub struct ReactiveComputed<T, F> {
    value: Arc<RwLock<Option<T>>>,
    compute_fn: Mutex<Option<F>>,
    dirty: Arc<RwLock<bool>>,
}

// Explicit Send + Sync implementations
unsafe impl<T: Send + Sync, F: Send + Sync> Send for ReactiveComputed<T, F> {}
unsafe impl<T: Send + Sync, F: Send + Sync> Sync for ReactiveComputed<T, F> {}

impl<T> ReactiveComputed<T, Box<dyn FnMut() -> T + Send + Sync + 'static>>
where
    T: Send + Sync + Clone + 'static,
{
    /// Get the computed value, recalculating if necessary
    pub fn get(&self) -> Result<T, SignalError> {
        if *self.dirty.read().unwrap() || self.value.read().unwrap().is_none() {
            self.recompute()?;
        }

        self.value
            .read()
            .unwrap()
            .clone()
            .ok_or(SignalError::InvalidState)
    }

    fn recompute(&self) -> Result<(), SignalError> {
        // Check if we should compute
        let should_compute = {
            let compute_ref = self.compute_fn.lock().unwrap();
            compute_ref.is_some()
        };

        if should_compute {
            let mut compute_fn = self.compute_fn.lock().unwrap().take().unwrap();
            let new_value = compute_fn();
            *self.value.write().unwrap() = Some(new_value);
            *self.compute_fn.lock().unwrap() = Some(compute_fn);
            *self.dirty.write().unwrap() = false;
        }
        Ok(())
    }
}

/// Create a new signal with an initial value
pub fn create_signal<T>(_scope: &ReactiveScope, initial_value: T) -> Signal<T>
where
    T: Send + Sync + 'static,
{
    Signal {
        value: Arc::new(RwLock::new(initial_value)),
        dirty: Arc::new(RwLock::new(false)),
    }
}

/// Create a new effect that runs when dependencies change
pub fn create_effect<F>(
    _scope: &ReactiveScope,
    callback: F,
) -> Effect<Box<dyn FnMut() + Send + Sync + 'static>>
where
    F: FnMut() + Send + Sync + 'static,
{
    let effect = Effect {
        callback: Mutex::new(Some(
            Box::new(callback) as Box<dyn FnMut() + Send + Sync + 'static>
        )),
        dirty: Arc::new(RwLock::new(true)), // Start dirty to run on creation
    };

    // Run initially
    let _ = effect.run();
    effect
}

/// Create a new computed value
pub fn create_computed<T, F>(
    _scope: &ReactiveScope,
    compute_fn: F,
) -> ReactiveComputed<T, Box<dyn FnMut() -> T + Send + Sync + 'static>>
where
    F: FnMut() -> T + Send + Sync + 'static,
    T: Send + Sync + Clone + 'static,
{
    ReactiveComputed {
        value: Arc::new(RwLock::new(None)),
        compute_fn: Mutex::new(Some(
            Box::new(compute_fn) as Box<dyn FnMut() -> T + Send + Sync + 'static>
        )),
        dirty: Arc::new(RwLock::new(true)), // Start dirty to compute on first access
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
        let counter = Arc::new(RwLock::new(0));
        let counter_clone = counter.clone();

        let _effect = create_effect(&scope, move || {
            *counter_clone.write().unwrap() += 1;
        });

        // Effect should run once on creation
        assert_eq!(*counter.read().unwrap(), 1);
    }

    #[test]
    fn test_computed_value() {
        let scope = ReactiveScope::new();
        let signal = create_signal(&scope, 5);
        let signal_clone = signal.value.clone();

        let computed = create_computed(&scope, move || *signal_clone.read().unwrap() * 2);

        assert_eq!(computed.get().unwrap(), 10);
    }
}
