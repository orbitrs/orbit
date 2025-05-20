// State management for the Orbit UI framework

/// Trait for state management
pub trait State: Clone {
    /// Get the current value of the state
    fn get(&self) -> &dyn std::any::Any;

    /// Update the state
    fn set(&mut self, value: Box<dyn std::any::Any>) -> Result<(), StateError>;

    /// Subscribe to state changes
    fn subscribe(&mut self, callback: Box<dyn Fn(&dyn std::any::Any)>);
}

/// Simple implementation of state
pub struct SimpleState<T: Clone + 'static> {
    value: T,
    subscribers: Vec<Box<dyn Fn(&dyn std::any::Any)>>,
}

impl<T: Clone + 'static> SimpleState<T> {
    /// Create a new state with the given initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: initial,
            subscribers: Vec::new(),
        }
    }
}

impl<T: Clone + 'static> State for SimpleState<T> {
    fn get(&self) -> &dyn std::any::Any {
        &self.value
    }

    fn set(&mut self, value: Box<dyn std::any::Any>) -> Result<(), StateError> {
        if let Some(new_value) = value.downcast_ref::<T>() {
            self.value = new_value.clone();

            // Notify subscribers
            for subscriber in &self.subscribers {
                subscriber(&self.value);
            }

            Ok(())
        } else {
            Err(StateError::TypeMismatch)
        }
    }

    fn subscribe(&mut self, callback: Box<dyn Fn(&dyn std::any::Any)>) {
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
