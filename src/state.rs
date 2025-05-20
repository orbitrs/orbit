//! State management system for the Orbit UI framework

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Container for managing application state
#[derive(Clone)]
pub struct StateContainer {
    /// Map of state values by type ID
    pub(crate) values: Arc<Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,

    /// Map of subscribers by type ID
    pub(crate) subscribers: Arc<Mutex<HashMap<TypeId, Vec<Box<dyn Fn() + Send + Sync>>>>>,
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
        
        // Store initial value
        self.values
            .lock()
            .unwrap()
            .insert(type_id, Box::new(initial.clone()));
        
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
        
        // Store initial value
        self.values
            .lock()
            .unwrap()
            .insert(type_id, Box::new(initial));
        
        Computed::new(self.clone(), compute, dependencies)
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, type_id: TypeId, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().unwrap();
        
        subscribers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Notify subscribers of state change
    pub fn notify(&self, type_id: TypeId) {
        let subscribers = self.subscribers.lock().unwrap();
        
        if let Some(callbacks) = subscribers.get(&type_id) {
            for callback in callbacks {
                callback();
            }
        }
    }
}

/// Represents a state value that can be read and updated
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
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
            .unwrap()
    }
    
    /// Set new value
    pub fn set(&self, value: T) {
        // Update value
        self.container
            .values
            .lock()
            .unwrap()
            .insert(self.type_id, Box::new(value));
        
        // Notify subscribers
        self.container.notify(self.type_id);
    }
}

/// Represents a computed state value that depends on other state
pub struct Computed<T> {
    /// State container
    container: StateContainer,
    
    /// Type ID for this state
    type_id: TypeId,
    
    /// Compute function
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
        for dep_id in computed.dependencies.iter() {
            // Clone Arc wrapper for each dependency
            let compute_fn = compute_arc.clone();
            let container_clone = container.clone();
            let type_id_clone = type_id;

            container.subscribe(*dep_id, move || {
                // Recompute value when dependency changes
                let new_value = (*compute_fn)();
                container_clone
                    .values
                    .lock()
                    .unwrap()
                    .insert(type_id_clone, Box::new(new_value));
            });
        }

        computed
    }

    /// Get current value
    pub fn get(&self) -> T {
        let values = self.container.values.lock().unwrap();
        values
            .get(&self.type_id)
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
            .unwrap()
    }
}
