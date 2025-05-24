//! Reactive state management for Orbit UI framework
//!
//! This module provides a simplified but robust reactive system for state management.
//! It uses a scope-based approach to manage effects and dependencies, avoiding
//! the circular dependency issues of the previous implementation.

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fmt,
    rc::{Rc, Weak},
};

/// Error type for signal operations
#[derive(Debug)]
pub enum SignalError {
    /// Error accessing signal value
    AccessError(String),
    /// Type mismatch in signal operation
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    /// Other signal error
    Other(String),
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessError(msg) => write!(f, "Signal access error: {}", msg),
            Self::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            Self::Other(msg) => write!(f, "Signal error: {}", msg),
        }
    }
}

impl std::error::Error for SignalError {}

/// A unique identifier for effects
type EffectId = u64;
static NEXT_EFFECT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A unique identifier for signals
type SignalId = u64;
static NEXT_SIGNAL_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The reactive scope manages the execution context for effects and computed values
pub struct ReactiveScope {
    current_effect: RefCell<Option<EffectId>>,
    effects: RefCell<Vec<Box<dyn EffectRunner>>>,
}

impl ReactiveScope {
    /// Create a new reactive scope
    pub fn new() -> Self {
        Self {
            current_effect: RefCell::new(None),
            effects: RefCell::new(Vec::new()),
        }
    }

    /// Run a closure with effect tracking enabled
    fn with_effect_tracking<F, R>(&self, effect_id: EffectId, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let old_effect = self.current_effect.replace(Some(effect_id));
        let result = f();
        self.current_effect.replace(old_effect);
        result
    }

    /// Get the currently running effect ID
    fn current_effect(&self) -> Option<EffectId> {
        *self.current_effect.borrow()
    }

    /// Add an effect to this scope
    fn add_effect(&self, effect: Box<dyn EffectRunner>) {
        self.effects.borrow_mut().push(effect);
    }
}

impl Default for ReactiveScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for objects that can run effects
trait EffectRunner {
    fn run(&mut self, scope: &ReactiveScope);
    fn id(&self) -> EffectId;
}

/// A reactive signal that holds a value and notifies subscribers when it changes
pub struct Signal<T> {
    id: SignalId,
    value: RefCell<T>,
    subscribers: RefCell<HashSet<EffectId>>,
    scope: Weak<ReactiveScope>,
}

impl<T: Clone + 'static> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(initial: T, scope: &Rc<ReactiveScope>) -> Rc<Self> {
        let id = NEXT_SIGNAL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Rc::new(Self {
            id,
            value: RefCell::new(initial),
            subscribers: RefCell::new(HashSet::new()),
            scope: Rc::downgrade(scope),
        })
    }

    /// Get the current value, tracking this read in the current effect
    pub fn get(&self) -> T {
        // Track this read in the current effect if one is active
        if let Some(scope) = self.scope.upgrade() {
            if let Some(effect_id) = scope.current_effect() {
                self.subscribers.borrow_mut().insert(effect_id);
            }
        }

        self.value.borrow().clone()
    }

    /// Set a new value and notify subscribers if changed
    pub fn set(&self, new_value: T)
    where
        T: PartialEq,
    {
        let should_notify = {
            let mut value = self.value.borrow_mut();
            if *value != new_value {
                *value = new_value;
                true
            } else {
                false
            }
        };

        if should_notify {
            self.notify();
        }
    }

    /// Update the value using a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T,
        T: PartialEq,
    {
        let should_notify = {
            let value = self.value.borrow();
            let new_value = f(&value);
            let changed = *value != new_value;
            drop(value); // Release borrow before potentially mutating

            if changed {
                *self.value.borrow_mut() = new_value;
                true
            } else {
                false
            }
        };

        if should_notify {
            self.notify();
        }
    }

    /// Notify all subscribers of a change
    fn notify(&self) {
        if let Some(scope) = self.scope.upgrade() {
            let subscribers = self.subscribers.borrow().clone();
            let mut effects = scope.effects.borrow_mut();
            
            for effect_id in subscribers {
                if let Some(effect) = effects.iter_mut().find(|e| e.id() == effect_id) {
                    effect.run(&scope);
                }
            }
        }
    }
}

/// An effect that runs a closure and tracks its dependencies
pub struct Effect<F> {
    id: EffectId,
    closure: RefCell<F>,
    dependencies: RefCell<HashSet<SignalId>>,
}

impl<F> Effect<F>
where
    F: Fn() + 'static,
{
    /// Create a new effect
    pub fn new(closure: F) -> Self {
        let id = NEXT_EFFECT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            id,
            closure: RefCell::new(closure),
            dependencies: RefCell::new(HashSet::new()),
        }
    }
}

impl<F> EffectRunner for Effect<F>
where
    F: Fn() + 'static,
{
    fn run(&mut self, scope: &ReactiveScope) {
        let closure = &*self.closure.borrow();
        scope.with_effect_tracking(self.id, closure);
    }

    fn id(&self) -> EffectId {
        self.id
    }
}

/// A computed signal that derives its value from other signals
pub struct Computed<T, F> {
    signal: Rc<Signal<T>>,
    compute_fn: RefCell<F>,
    effect: RefCell<Option<Effect<Box<dyn Fn()>>>>,
}

impl<T, F> Computed<T, F>
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> T + 'static,
{
    /// Create a new computed signal
    pub fn new(compute_fn: F, scope: &Rc<ReactiveScope>) -> Rc<Self> {
        // Create the initial value
        let initial = compute_fn();
        let signal = Signal::new(initial, scope);
        
        let computed = Rc::new(Self {
            signal: signal.clone(),
            compute_fn: RefCell::new(compute_fn),
            effect: RefCell::new(None),
        });

        // Create the effect that updates the signal when dependencies change
        let computed_weak = Rc::downgrade(&computed);
        let effect = Effect::new(Box::new(move || {
            if let Some(computed) = computed_weak.upgrade() {
                let new_value = (computed.compute_fn.borrow())();
                computed.signal.set(new_value);
            }
        }));

        // Store the effect
        *computed.effect.borrow_mut() = Some(effect);

        computed
    }

    /// Get the current computed value
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

/// Convenience functions for creating reactive primitives

/// Create a new signal with the given initial value
pub fn create_signal<T: Clone + PartialEq + 'static>(
    initial: T,
    scope: &Rc<ReactiveScope>,
) -> Rc<Signal<T>> {
    Signal::new(initial, scope)
}

/// Create a new computed signal with the given compute function
pub fn create_computed<T, F>(
    compute_fn: F,
    scope: &Rc<ReactiveScope>,
) -> Rc<Computed<T, F>>
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> T + 'static,
{
    Computed::new(compute_fn, scope)
}

/// Create an effect that runs when its dependencies change
pub fn create_effect<F>(closure: F, scope: &Rc<ReactiveScope>)
where
    F: Fn() + 'static,
{
    let effect = Box::new(Effect::new(closure));
    scope.add_effect(effect);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_basic_signal() {
        let scope = Rc::new(ReactiveScope::new());
        let count = create_signal(0, &scope);
        
        assert_eq!(count.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);

        count.update(|&c| c + 1);
        assert_eq!(count.get(), 6);
    }

    #[test]
    fn test_basic_computed() {
        let scope = Rc::new(ReactiveScope::new());
        let count = create_signal(0, &scope);
        let count_for_computed = count.clone();
        let double = create_computed(move || count_for_computed.get() * 2, &scope);

        assert_eq!(double.get(), 0);

        count.set(5);
        assert_eq!(double.get(), 10);
    }

    #[test]
    fn test_effect_tracks_dependencies() {
        let scope = Rc::new(ReactiveScope::new());
        let count = create_signal(0, &scope);
        let effect_ran = Rc::new(Cell::new(0));

        let effect_ran_clone = effect_ran.clone();
        let count_for_effect = count.clone();
        create_effect(
            move || {
                let _ = count_for_effect.get(); // Track dependency
                effect_ran_clone.set(effect_ran_clone.get() + 1);
            },
            &scope,
        );

        // Effect should have run once initially
        assert_eq!(effect_ran.get(), 1);

        // Effect should run again when signal changes
        count.set(5);
        assert_eq!(effect_ran.get(), 2);

        // Effect should not run when value doesn't change
        count.set(5);
        assert_eq!(effect_ran.get(), 2);

        // Effect should run again when value changes
        count.set(10);
        assert_eq!(effect_ran.get(), 3);
    }

    #[test]
    fn test_computed_dependency_chain() {
        let scope = Rc::new(ReactiveScope::new());
        let count = create_signal(0, &scope);
        let count_for_double = count.clone();
        let double = create_computed(move || count_for_double.get() * 2, &scope);
        let double_for_quad = double.clone();
        let quadruple = create_computed(move || double_for_quad.get() * 2, &scope);

        assert_eq!(count.get(), 0);
        assert_eq!(double.get(), 0);
        assert_eq!(quadruple.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);
        assert_eq!(double.get(), 10);
        assert_eq!(quadruple.get(), 20);
    }

    #[test]
    fn test_multiple_dependencies() {
        let scope = Rc::new(ReactiveScope::new());
        let a = create_signal(1, &scope);
        let b = create_signal(2, &scope);
        let a_for_sum = a.clone();
        let b_for_sum = b.clone();
        let sum = create_computed(move || a_for_sum.get() + b_for_sum.get(), &scope);

        assert_eq!(sum.get(), 3);

        a.set(5);
        assert_eq!(sum.get(), 7);

        b.set(10);
        assert_eq!(sum.get(), 15);
    }
}
