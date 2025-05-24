//! State management system for the Orbit UI framework
//!
//! This module provides several approaches to state management:
//!
//! 1. `StateContainer` - A basic state container that stores values and notifies subscribers when values change
//! 2. `State<T>` - A typed reference to a value in the state container
//! 3. `Computed<T>` - A derived state value that depends on other values
//! 4. The reactive module - A fine-grained reactive system with signals and effects

mod reactive;

pub use reactive::{
    create_computed, create_effect, create_signal, Effect, Signal, SignalError, ThreadSafeSignal,
};

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Define a type alias for the complex subscriber type
type SubscriberCallback = Box<dyn Fn() + Send + Sync>;
type SubscriberMap = HashMap<TypeId, Vec<SubscriberCallback>>;
// Define a type for the complex state value container
type StateValue = Arc<Mutex<Box<dyn std::any::Any + Send + Sync>>>;
type StateMap = HashMap<TypeId, StateValue>;

/// State management for Orbit applications
///
/// This is the original state management system that is being enhanced with the reactive system.
/// It's kept for backward compatibility but new code should prefer the reactive system.
#[derive(Clone)]
pub struct StateContainer {
    // Using Arc<Mutex<>> for thread-safe interior mutability
    // TypeId is used to identify the type of the stored value
    #[allow(clippy::type_complexity)]
    values: Arc<Mutex<StateMap>>,
    // Subscribers are functions that are called when a value changes
    // Using Arc<Mutex<>> for thread-safe interior mutability
    pub(crate) subscribers: Arc<Mutex<SubscriberMap>>,
}

impl std::fmt::Debug for StateContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateContainer")
            .field("values", &"[StateMap]")
            .field("subscribers", &"[SubscriberMap]")
            .finish()
    }
}

impl StateContainer {
    /// Create a new state container
    pub fn new() -> Self {
        Self {
            values: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new state value
    pub fn create<T: 'static + Clone + Send + Sync>(&self, initial: T) -> State<T> {
        let type_id = TypeId::of::<T>();

        // Store initial value with the correct type (Arc<Mutex<Box<dyn Any + Send + Sync>>>)
        self.values
            .lock()
            .unwrap()
            .insert(type_id, Arc::new(Mutex::new(Box::new(initial.clone()))));

        State {
            container: self.clone(),
            type_id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Create computed state dependent on other state
    pub fn computed<T, F>(&self, compute: F, dependencies: Vec<TypeId>) -> Computed<T>
    where
        T: 'static + Clone + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        // Create initial value
        let initial = compute();
        let type_id = TypeId::of::<T>();

        // Store initial value with correct type
        self.values
            .lock()
            .unwrap()
            .insert(type_id, Arc::new(Mutex::new(Box::new(initial))));

        Computed::new(self.clone(), compute, dependencies)
    }

    /// Subscribe to state changes
    pub fn subscribe<T: 'static + Send + Sync, F: Fn() + Send + Sync + 'static>(
        &self,
        callback: F,
    ) {
        let type_id = TypeId::of::<T>();
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers
            .entry(type_id)
            .or_default()
            .push(Box::new(callback));
    }

    /// Notify subscribers of a change to a value
    pub fn notify(&self, type_id: TypeId) {
        let subscribers = self.subscribers.lock().unwrap();

        if let Some(callbacks) = subscribers.get(&type_id) {
            for callback in callbacks {
                callback();
            }
        }
    }
}

impl Default for StateContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a reactive value that can be observed for changes
pub struct State<T> {
    /// State container
    container: StateContainer,

    /// Type ID for this state
    type_id: TypeId,

    /// Phantom data for type
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static + Clone + Send + Sync> State<T> {
    /// Get current value
    pub fn get(&self) -> T {
        let values = self.container.values.lock().unwrap();

        values
            .get(&self.type_id)
            .and_then(|value| {
                let lock = value.lock().unwrap();
                lock.downcast_ref::<T>().cloned()
            })
            .unwrap()
    }

    /// Set new value
    pub fn set(&self, value: T) {
        // Update value
        self.container
            .values
            .lock()
            .unwrap()
            .insert(self.type_id, Arc::new(Mutex::new(Box::new(value))));

        // Notify subscribers
        self.container.notify(self.type_id);
    }

    /// Update value with a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T,
    {
        let values = self.container.values.lock().unwrap();

        if let Some(value_container) = values.get(&self.type_id) {
            let mut value_lock = value_container.lock().unwrap();

            // Apply the function to get the new value
            if let Some(old_value) = value_lock.downcast_ref::<T>() {
                let new_value = f(old_value);
                *value_lock = Box::new(new_value);

                // Drop locks before notifying subscribers
                drop(value_lock);
                drop(values);

                // Notify subscribers
                self.container.notify(self.type_id);
            }
        }
    }
}

/// Represents a computed state value that depends on other state
pub struct Computed<T> {
    /// State container
    container: StateContainer,

    /// Type ID for this state
    type_id: TypeId,

    /// Compute function
    #[allow(dead_code)]
    compute: Arc<Box<dyn Fn() -> T + Send + Sync>>,

    /// Dependencies
    dependencies: Vec<TypeId>,
}

impl<T: 'static + Clone + Send + Sync> Computed<T> {
    /// Create new computed state
    pub fn new<F>(container: StateContainer, compute: F, dependencies: Vec<TypeId>) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        // Initial value already stored in container
        let type_id = TypeId::of::<T>();

        // Wrap compute function in Arc for cloning
        let compute_arc = Arc::new(Box::new(compute) as Box<dyn Fn() -> T + Send + Sync>);

        // Create computed state
        let computed = Self {
            container: container.clone(),
            type_id,
            compute: compute_arc.clone(),
            dependencies,
        };

        // Subscribe to dependencies
        for &dep_id in computed.dependencies.iter() {
            // Clone Arc wrapper for each dependency
            let compute_fn = compute_arc.clone();
            let container_clone = container.clone();
            let type_id_clone = type_id;

            // Create a closure for this dependency
            let callback = move || {
                // Recompute value when dependency changes
                let new_value = (*compute_fn)();
                container_clone
                    .values
                    .lock()
                    .unwrap()
                    .insert(type_id_clone, Arc::new(Mutex::new(Box::new(new_value))));
            };

            // Add closure to subscribers for this dependency type
            let mut subscribers = container.subscribers.lock().unwrap();
            subscribers
                .entry(dep_id)
                .or_default()
                .push(Box::new(callback));
        }

        computed
    }

    /// Get current value
    pub fn get(&self) -> T {
        let values = self.container.values.lock().unwrap();
        values
            .get(&self.type_id)
            .and_then(|value| {
                let lock = value.lock().unwrap();
                lock.downcast_ref::<T>().cloned()
            })
            .unwrap()
    }
}
