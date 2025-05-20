//! State management system for the Orbit UI framework

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Container for managing application state
#[derive(Clone)]
pub struct StateContainer {
    /// Maps type IDs to their state values
    values: Arc<Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,
    
    /// Maps type IDs to their subscriptions
    subscribers: Arc<Mutex<HashMap<TypeId, Vec<Arc<Mutex<dyn FnMut() + Send + Sync>>>>>>,
}

impl StateContainer {
    /// Create a new state container
    pub fn new() -> Self {
        Self {
            values: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create new state with an initial value
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
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Subscribe to state changes
    pub fn subscribe<F>(&self, type_id: TypeId, callback: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers
            .entry(type_id)
            .or_default()
            .push(Arc::new(Mutex::new(callback)));
    }

    /// Notify subscribers of state changes
    fn notify_subscribers(&self, type_id: TypeId) {
        let subscribers = self.subscribers.lock().unwrap();
        if let Some(callbacks) = subscribers.get(&type_id) {
            for callback in callbacks {
                if let Ok(mut callback) = callback.lock() {
                    callback();
                }
            }
        }
    }
}

/// Handle to a piece of state
#[derive(Clone)]
pub struct State<T> {
    /// Reference to the state container
    container: StateContainer,
    
    /// Type ID of the state value
    type_id: TypeId,
    
    /// Phantom data to track the value type
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static + Clone + Send + Sync> State<T> {
    /// Get the current value
    pub fn get(&self) -> T {
        let values = self.container.values.lock().unwrap();
        values
            .get(&self.type_id)
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
            .unwrap()
    }

    /// Set a new value
    pub fn set(&self, value: T) {
        let mut values = self.container.values.lock().unwrap();
        values.insert(self.type_id, Box::new(value));
        
        // Notify subscribers
        self.container.notify_subscribers(self.type_id);
    }
    
    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        self.container.subscribe(self.type_id, callback);
    }
}

/// Computed state that depends on other state values
pub struct Computed<T> {
    /// Reference to the state container
    container: StateContainer,
    
    /// Type ID of the computed value
    type_id: TypeId,
    
    /// Computation function
    compute: Box<dyn Fn() -> T + Send + Sync>,
    
    /// Dependencies
    dependencies: Vec<TypeId>,
}

impl<T: 'static + Clone + Send + Sync> Computed<T> {
    /// Create new computed state
    pub fn new<F>(container: StateContainer, compute: F, dependencies: Vec<TypeId>) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let compute = Box::new(compute);
        
        // Initialize value
        let initial = compute();
        container
            .values
            .lock()
            .unwrap()
            .insert(type_id, Box::new(initial));
            
        // Create computed state
        let computed = Self {
            container: container.clone(),
            type_id,
            compute,
            dependencies,
        };
        
        // Subscribe to dependencies
        for dep_id in dependencies.iter() {
            let compute = computed.compute.clone();
            let container = container.clone();
            let type_id = type_id;
            
            container.subscribe(*dep_id, move || {
                // Recompute value when dependency changes
                let new_value = compute();
                container
                    .values
                    .lock()
                    .unwrap()
                    .insert(type_id, Box::new(new_value));
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
