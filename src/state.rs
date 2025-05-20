//! State management system for the Orbit UI framework

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::Rc,
    sync::{Arc, Mutex},
};

/// A container for reactive state values
#[derive(Clone)]
pub struct StateContainer {
    states: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    subscribers: Arc<Mutex<HashMap<TypeId, Vec<Box<dyn Fn() + Send + Sync>>>>>,
}

impl StateContainer {
    /// Create a new state container
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new reactive state value
    pub fn create<T: 'static + Clone + Send + Sync>(&self, initial: T) -> State<T> {
        let type_id = TypeId::of::<T>();
        let mut states = self.states.lock().unwrap();
        states.insert(type_id, Box::new(initial.clone()));

        State {
            container: self.clone(),
            type_id,
            phantom: std::marker::PhantomData,
        }
    }

    /// Get the current value of a state
    pub fn get<T: 'static + Clone + Send + Sync>(&self, type_id: TypeId) -> Option<T> {
        let states = self.states.lock().unwrap();
        states.get(&type_id)?.downcast_ref::<T>().cloned()
    }

    /// Set the value of a state and notify subscribers
    pub fn set<T: 'static + Clone + Send + Sync>(&self, type_id: TypeId, value: T) {
        let mut states = self.states.lock().unwrap();
        states.insert(type_id, Box::new(value));

        // Notify subscribers
        let subscribers = self.subscribers.lock().unwrap();
        if let Some(subs) = subscribers.get(&type_id) {
            for subscriber in subs {
                subscriber();
            }
        }
    }

    /// Subscribe to changes in a state value
    pub fn subscribe<F>(&self, type_id: TypeId, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.entry(type_id).or_default().push(Box::new(callback));
    }
}

impl Default for StateContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// A reactive state value that can be read and written
#[derive(Clone)]
pub struct State<T: 'static> {
    container: StateContainer,
    type_id: TypeId,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    /// Get the current value of the state
    pub fn get(&self) -> T {
        self.container
            .get(self.type_id)
            .expect("State value should exist")
    }

    /// Set a new value for the state
    pub fn set(&self, value: T) {
        self.container.set(self.type_id, value);
    }

    /// Subscribe to changes in this state value
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.container.subscribe(self.type_id, callback);
    }
}

/// Computed state that depends on other state values
pub struct Computed<T: 'static> {
    container: StateContainer,
    type_id: TypeId,
    compute: Box<dyn Fn() -> T + Send + Sync>,
    deps: Vec<TypeId>,
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    /// Create a new computed state value
    pub fn new<F>(container: StateContainer, compute: F, deps: Vec<TypeId>) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let computed = Self {
            container: container.clone(),
            type_id,
            compute: Box::new(compute),
            deps,
        };

        // Initialize value
        let initial_value = (computed.compute)();
        container.set(type_id, initial_value);

        // Set up dependency tracking
        let computed_clone = computed.clone();
        for dep_id in &computed.deps {
            container.subscribe(*dep_id, move || {
                let new_value = (computed_clone.compute)();
                computed_clone.container.set(computed_clone.type_id, new_value);
            });
        }

        computed
    }

    /// Get the current computed value
    pub fn get(&self) -> T {
        self.container
            .get(self.type_id)
            .expect("Computed value should exist")
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            type_id: self.type_id,
            compute: self.compute.clone(),
            deps: self.deps.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state() {
        let container = StateContainer::new();
        let count = container.create(0);

        assert_eq!(count.get(), 0);
        count.set(42);
        assert_eq!(count.get(), 42);
    }

    #[test]
    fn test_state_subscription() {
        let container = StateContainer::new();
        let count = container.create(0);
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        count.subscribe(move || {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        count.set(1);
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_computed_state() {
        let container = StateContainer::new();
        let count = container.create(0);
        let double = Computed::new(
            container.clone(),
            move || count.get() * 2,
            vec![TypeId::of::<i32>()],
        );

        assert_eq!(double.get(), 0);
        count.set(21);
        assert_eq!(double.get(), 42);
    }
}
        self.subscribers.push(callback);
    }
}

impl<T: Clone + 'static> Clone for SimpleState<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: Vec::new(), // Subscribers are not cloned
        }
    }
}

/// Errors that can occur in state operations
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Type mismatch when setting state")]
    TypeMismatch,
}
