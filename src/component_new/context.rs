//! Context passing and parent-child communication

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt::{self, Debug};

/// A type-erased value that can be stored in a context
pub trait ContextValue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn box_clone(&self) -> Box<dyn ContextValue>;
}

impl<T: Any + Clone + Send + Sync + 'static> ContextValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn box_clone(&self) -> Box<dyn ContextValue> {
        Box::new(self.clone())
    }
}

/// A provider of contextual values for component trees
#[derive(Default, Clone)]
pub struct ContextProvider {
    /// The values stored in this context
    values: Arc<RwLock<HashMap<TypeId, Box<dyn ContextValue>>>>,
    
    /// Optional parent context for fallback lookup
    parent: Option<Box<ContextProvider>>,
}

impl Debug for ContextProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextProvider")
            .field("values", &"<context values>")
            .field("parent", &self.parent.is_some())
            .finish()
    }
}

impl ContextProvider {
    /// Create a new empty context provider
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
            parent: None,
        }
    }
    
    /// Create a new context provider with a parent for fallback lookup
    pub fn with_parent(parent: ContextProvider) -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
            parent: Some(Box::new(parent)),
        }
    }
    
    /// Set a value in the context
    pub fn provide<T: Clone + Send + Sync + 'static>(&self, value: T) {
        let type_id = TypeId::of::<T>();
        if let Ok(mut values) = self.values.write() {
            values.insert(type_id, Box::new(value));
        }
    }
    
    /// Get a value from the context
    pub fn consume<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        
        // Try to get from this context first
        if let Ok(values) = self.values.read() {
            if let Some(value) = values.get(&type_id) {
                if let Some(typed_value) = value.as_any().downcast_ref::<T>() {
                    return Some(typed_value.clone());
                }
            }
        }
        
        // Fall back to parent context if available
        if let Some(parent) = &self.parent {
            return parent.consume::<T>();
        }
        
        None
    }
    
    /// Check if a type exists in the context
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        
        // Check this context first
        if let Ok(values) = self.values.read() {
            if values.contains_key(&type_id) {
                return true;
            }
        }
        
        // Check parent context if available
        if let Some(parent) = &self.parent {
            return parent.has::<T>();
        }
        
        false
    }
    
    /// Remove a value from the context
    pub fn remove<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        if let Ok(mut values) = self.values.write() {
            values.remove(&type_id).is_some()
        } else {
            false
        }
    }
}

/// A callback function that can be passed as a prop
pub struct Callback<Args, Ret = ()> {
    /// The function to call
    func: Arc<dyn Fn(Args) -> Ret + Send + Sync>,
}

impl<Args: 'static, Ret: 'static> Clone for Callback<Args, Ret> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

impl<Args, Ret> Callback<Args, Ret> {
    /// Create a new callback
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(Args) -> Ret + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
        }
    }
    
    /// Call the callback with the given arguments
    pub fn call(&self, args: Args) -> Ret {
        (self.func)(args)
    }
}

/// Convenience function for creating a callback
pub fn callback<F, Args, Ret>(func: F) -> Callback<Args, Ret>
where
    F: Fn(Args) -> Ret + Send + Sync + 'static,
{
    Callback::new(func)
}
