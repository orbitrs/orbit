//! Context passing and parent-child communication

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// A type-erased value that can be stored in a context
#[allow(dead_code)]
pub trait ContextValue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn box_clone(&self) -> Box<dyn ContextValue>;
    fn debug_string(&self) -> String;
}

impl<T: Any + Clone + Send + Sync + Debug + 'static> ContextValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn box_clone(&self) -> Box<dyn ContextValue> {
        Box::new(self.clone())
    }

    fn debug_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// Provider for component context
#[derive(Clone, Default)]
pub struct ContextProvider {
    /// Parent context provider
    parent: Option<Box<ContextProvider>>,
    values: Arc<RwLock<HashMap<TypeId, Box<dyn ContextValue>>>>,
}

impl Debug for ContextProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextProvider")
            .field("parent", &self.parent.is_some())
            .field(
                "values",
                &format!(
                    "[{} values]",
                    self.values.read().map(|v| v.len()).unwrap_or(0)
                ),
            )
            .finish()
    }
}

impl ContextProvider {
    /// Create a new context provider
    pub fn new() -> Self {
        Self {
            parent: None,
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a context provider with a parent
    pub fn with_parent(parent: ContextProvider) -> Self {
        Self {
            parent: Some(Box::new(parent)),
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a value in the context
    pub fn provide<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
        &self,
        value: T,
    ) -> Result<(), String> {
        let type_id = TypeId::of::<T>();
        if let Ok(mut values) = self.values.write() {
            values.insert(type_id, Box::new(value));
            Ok(())
        } else {
            Err("Failed to acquire write lock for context values".to_string())
        }
    }

    /// Get a value from the context
    pub fn consume<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        // Try to get from this context first
        let result = {
            if let Ok(values) = self.values.read() {
                if let Some(value) = values.get(&type_id) {
                    value.as_any().downcast_ref::<T>().cloned()
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Return value from this context if found
        if result.is_some() {
            return result;
        }

        // Fall back to parent context if available
        if let Some(parent) = &self.parent {
            parent.consume::<T>()
        } else {
            None
        }
    }

    /// Check if a type exists in the context
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        // Check this context first
        let exists_here = {
            if let Ok(values) = self.values.read() {
                values.contains_key(&type_id)
            } else {
                false
            }
        };

        if exists_here {
            return true;
        }

        // Check parent context if available
        if let Some(parent) = &self.parent {
            parent.has::<T>()
        } else {
            false
        }
    }

    /// Remove a value from the context
    pub fn remove<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let result = {
            if let Ok(mut values) = self.values.write() {
                values.remove(&type_id).is_some()
            } else {
                false
            }
        };
        result
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
