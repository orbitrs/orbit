//! Performance optimization hooks for Orbit components
//!
//! This module provides various hooks and utilities for optimizing component performance,
//! including memoization, batching, lazy loading, and render optimization.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use crate::component::{
    Component, ComponentError, ComponentId, Context, Node, StateChanges,
};

/// Trait for memoizable components
pub trait Memoizable {
    type MemoKey: Hash + Eq + Clone;

    /// Generate a key for memoization
    fn memo_key(&self) -> Self::MemoKey;

    /// Check if component should re-render based on memo key
    fn should_memo_update(&self, old_key: &Self::MemoKey, new_key: &Self::MemoKey) -> bool {
        old_key != new_key
    }
}

/// Memoization cache for component render results
pub struct MemoCache<K, V> {
    cache: RwLock<HashMap<K, CacheEntry<V>>>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    access_count: u64,
}

impl<K, V> MemoCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().ok()?;
        
        if let Some(entry) = cache.get_mut(key) {
            // Check if entry is still valid
            if entry.created_at.elapsed() < self.ttl {
                entry.access_count += 1;
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        None
    }

    pub fn set(&self, key: K, value: V) {
        if let Ok(mut cache) = self.cache.write() {
            // If cache is at capacity, remove least recently used item
            if cache.len() >= self.max_size {
                self.evict_lru(&mut cache);
            }

            cache.insert(
                key,
                CacheEntry {
                    value,
                    created_at: Instant::now(),
                    access_count: 1,
                },
            );
        }
    }

    fn evict_lru(&self, cache: &mut HashMap<K, CacheEntry<V>>) {
        if let Some((key_to_remove, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            cache.remove(&key_to_remove);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    pub fn size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }
}

/// Memoized component wrapper
pub struct MemoComponent<T>
where
    T: Component + Memoizable,
{
    component: T,
    last_memo_key: Option<T::MemoKey>,
    cached_render: Option<Vec<Node>>,
    cache: Arc<MemoCache<T::MemoKey, Vec<Node>>>,
}

impl<T> MemoComponent<T>
where
    T: Component + Memoizable,
    T::MemoKey: Send + Sync + 'static,
{
    pub fn new(component: T) -> Self {
        Self {
            component,
            last_memo_key: None,
            cached_render: None,
            cache: Arc::new(MemoCache::new(100, Duration::from_secs(300))), // 5min TTL
        }
    }

    pub fn with_cache(component: T, cache: Arc<MemoCache<T::MemoKey, Vec<Node>>>) -> Self {
        Self {
            component,
            last_memo_key: None,
            cached_render: None,
            cache,
        }
    }
}

impl<T> Component for MemoComponent<T>
where
    T: Component + Memoizable + Send + Sync + 'static,
    T::Props: Send + Sync + 'static,
    T::MemoKey: Send + Sync + 'static,
{
    type Props = T::Props;    fn component_id(&self) -> ComponentId {
        Component::component_id(&self.component)
    }

    fn create(props: Self::Props, context: Context) -> Self {
        Self::new(T::create(props, context))
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.component.update(props)
    }

    fn should_update(&self, new_props: &Self::Props) -> bool {
        self.component.should_update(new_props)
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        let current_key = self.component.memo_key();

        // Check if we can use cached result
        if let Some(ref last_key) = self.last_memo_key {
            if !self.component.should_memo_update(last_key, &current_key) {
                if let Some(cached) = self.cache.get(&current_key) {
                    return Ok(cached);
                }
            }
        }

        // Render and cache result
        let result = self.component.render()?;
        self.cache.set(current_key, result.clone());
        Ok(result)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Performance monitoring hooks
pub struct PerformanceMonitor {
    render_times: RwLock<HashMap<ComponentId, Vec<Duration>>>,
    update_times: RwLock<HashMap<ComponentId, Vec<Duration>>>,
    mount_times: RwLock<HashMap<ComponentId, Duration>>,
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self::new() // Create a new instance instead of cloning the data
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            render_times: RwLock::new(HashMap::new()),
            update_times: RwLock::new(HashMap::new()),
            mount_times: RwLock::new(HashMap::new()),
        }
    }

    pub fn start_render_timing(&self, component_id: ComponentId) -> RenderTimer {
        RenderTimer::new(component_id, Arc::new(self.clone()))
    }

    pub fn record_render_time(&self, component_id: ComponentId, duration: Duration) {
        if let Ok(mut times) = self.render_times.write() {
            times.entry(component_id).or_insert_with(Vec::new).push(duration);
            
            // Keep only last 100 measurements
            if let Some(component_times) = times.get_mut(&component_id) {
                if component_times.len() > 100 {
                    component_times.remove(0);
                }
            }
        }
    }

    pub fn record_update_time(&self, component_id: ComponentId, duration: Duration) {
        if let Ok(mut times) = self.update_times.write() {
            times.entry(component_id).or_insert_with(Vec::new).push(duration);
            
            // Keep only last 100 measurements
            if let Some(component_times) = times.get_mut(&component_id) {
                if component_times.len() > 100 {
                    component_times.remove(0);
                }
            }
        }
    }

    pub fn record_mount_time(&self, component_id: ComponentId, duration: Duration) {
        if let Ok(mut times) = self.mount_times.write() {
            times.insert(component_id, duration);
        }
    }

    pub fn get_average_render_time(&self, component_id: ComponentId) -> Option<Duration> {
        if let Ok(times) = self.render_times.read() {
            if let Some(component_times) = times.get(&component_id) {
                if !component_times.is_empty() {
                    let total: Duration = component_times.iter().sum();
                    return Some(total / component_times.len() as u32);
                }
            }
        }
        None
    }

    pub fn get_render_statistics(&self, component_id: ComponentId) -> RenderStatistics {
        if let Ok(times) = self.render_times.read() {
            if let Some(component_times) = times.get(&component_id) {
                if !component_times.is_empty() {
                    let total: Duration = component_times.iter().sum();
                    let average = total / component_times.len() as u32;
                    let min = *component_times.iter().min().unwrap();
                    let max = *component_times.iter().max().unwrap();
                    
                    return RenderStatistics {
                        count: component_times.len(),
                        total,
                        average,
                        min,
                        max,
                    };
                }
            }
        }
        RenderStatistics::default()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RenderStatistics {
    pub count: usize,
    pub total: Duration,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl Default for RenderStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            total: Duration::ZERO,
            average: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
        }
    }
}

/// Timer for measuring render performance
pub struct RenderTimer {
    component_id: ComponentId,
    start_time: Instant,
    monitor: Arc<PerformanceMonitor>,
}

impl RenderTimer {
    fn new(component_id: ComponentId, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            component_id,
            start_time: Instant::now(),
            monitor,
        }
    }
}

impl Drop for RenderTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.monitor.record_render_time(self.component_id, duration);
    }
}

/// Batched update system for performance optimization
pub struct UpdateBatcher {
    pending_updates: Mutex<HashMap<ComponentId, Vec<StateChanges>>>,
    batch_timeout: Duration,
    max_batch_size: usize,
}

impl UpdateBatcher {
    pub fn new(batch_timeout: Duration, max_batch_size: usize) -> Self {
        Self {
            pending_updates: Mutex::new(HashMap::new()),
            batch_timeout,
            max_batch_size,
        }
    }

    pub fn queue_update(&self, component_id: ComponentId, changes: StateChanges) {
        if let Ok(mut pending) = self.pending_updates.lock() {
            pending
                .entry(component_id)
                .or_insert_with(Vec::new)
                .push(changes);
        }
    }

    pub fn flush_updates(&self) -> HashMap<ComponentId, Vec<StateChanges>> {
        if let Ok(mut pending) = self.pending_updates.lock() {
            let updates = pending.clone();
            pending.clear();
            updates
        } else {
            HashMap::new()
        }
    }

    pub fn should_flush(&self, component_id: ComponentId) -> bool {
        if let Ok(pending) = self.pending_updates.lock() {
            if let Some(updates) = pending.get(&component_id) {
                return updates.len() >= self.max_batch_size
                    || updates
                        .first()
                        .map(|u| u.batch_timestamp.elapsed() >= self.batch_timeout)
                        .unwrap_or(false);
            }
        }
        false
    }
}

/// Lazy loading component wrapper
pub struct LazyComponent<T>
where
    T: Component,
{
    component: Option<T>,
    props: Option<T::Props>,
    context: Context,
    loaded: bool,
    load_trigger: LoadTrigger,
}

#[derive(Clone)]
pub enum LoadTrigger {
    Immediate,
    OnMount,
    OnFirstRender,
    OnVisible,
}

impl<T> LazyComponent<T>
where
    T: Component,
{
    pub fn new(context: Context, load_trigger: LoadTrigger) -> Self {
        Self {
            component: None,
            props: None,
            context,
            loaded: false,
            load_trigger,
        }
    }

    fn ensure_loaded(&mut self) -> Result<(), ComponentError> {
        if !self.loaded {
            if let Some(props) = self.props.clone() {
                self.component = Some(T::create(props, self.context.clone()));
                self.loaded = true;
            }
        }
        Ok(())
    }
}

impl<T> Component for LazyComponent<T>
where
    T: Component + Send + Sync + 'static,
    T::Props: Send + Sync + 'static,
{
    type Props = T::Props;    fn component_id(&self) -> ComponentId {
        if let Some(ref component) = self.component {
            Component::component_id(component)
        } else {
            ComponentId::new() // Temporary ID until loaded
        }
    }

    fn create(props: Self::Props, context: Context) -> Self {
        let mut lazy = Self::new(context, LoadTrigger::OnMount);
        lazy.props = Some(props);
        lazy
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        if matches!(self.load_trigger, LoadTrigger::OnMount | LoadTrigger::Immediate) {
            self.ensure_loaded()?;
            if let Some(ref mut component) = self.component {
                component.mount()?;
            }
        }
        Ok(())
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.props = Some(props.clone());
        if let Some(ref mut component) = self.component {
            component.update(props)?;
        }
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        if let Some(ref component) = self.component {
            component.render()
        } else {
            // Return empty or placeholder nodes for unloaded components
            Ok(Vec::new())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Global performance optimization registry
pub struct PerformanceRegistry {
    monitor: Arc<PerformanceMonitor>,
    memo_cache: Arc<MemoCache<String, Vec<Node>>>,
    update_batcher: Arc<UpdateBatcher>,
}

impl PerformanceRegistry {
    pub fn new() -> Self {
        Self {
            monitor: Arc::new(PerformanceMonitor::new()),
            memo_cache: Arc::new(MemoCache::new(1000, Duration::from_secs(600))),
            update_batcher: Arc::new(UpdateBatcher::new(
                Duration::from_millis(16), // ~60fps
                10, // max 10 updates per batch
            )),
        }
    }

    pub fn monitor(&self) -> Arc<PerformanceMonitor> {
        self.monitor.clone()
    }

    pub fn memo_cache(&self) -> Arc<MemoCache<String, Vec<Node>>> {
        self.memo_cache.clone()
    }

    pub fn update_batcher(&self) -> Arc<UpdateBatcher> {
        self.update_batcher.clone()
    }
}

impl Default for PerformanceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macros for easy performance optimization

/// Create a memoized component
#[macro_export]
macro_rules! memo {
    ($component:expr) => {
        MemoComponent::new($component)
    };
    ($component:expr, $cache:expr) => {
        MemoComponent::with_cache($component, $cache)
    };
}

/// Create a lazy component
#[macro_export]
macro_rules! lazy {
    ($component_type:ty, $context:expr) => {
        LazyComponent::<$component_type>::new($context, LoadTrigger::OnMount)
    };
    ($component_type:ty, $context:expr, $trigger:expr) => {
        LazyComponent::<$component_type>::new($context, $trigger)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentBase;

    #[derive(Clone, Hash, PartialEq, Eq)]
    struct TestMemoKey {
        id: u64,
        version: u32,
    }

    #[derive(Clone)]
    struct TestProps {
        id: u64,
        version: u32,
    }

    impl Props for TestProps {}

    struct TestComponent {
        base: ComponentBase,
        props: TestProps,
    }

    impl Component for TestComponent {
        type Props = TestProps;

        fn component_id(&self) -> ComponentId {
            self.base.id()
        }

        fn create(props: Self::Props, context: Context) -> Self {
            Self {
                base: ComponentBase::new(context),
                props,
            }
        }

        fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
            self.props = props;
            Ok(())
        }

        fn render(&self) -> Result<Vec<Node>, ComponentError> {
            Ok(vec![])
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    impl Memoizable for TestComponent {
        type MemoKey = TestMemoKey;

        fn memo_key(&self) -> Self::MemoKey {
            TestMemoKey {
                id: self.props.id,
                version: self.props.version,
            }
        }
    }

    #[test]
    fn test_memo_cache() {
        let cache = MemoCache::new(2, Duration::from_secs(1));
        
        cache.set("key1".to_string(), vec![]);
        cache.set("key2".to_string(), vec![]);
        assert_eq!(cache.size(), 2);

        // Adding third item should evict least used
        cache.set("key3".to_string(), vec![]);
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        let component_id = ComponentId::new();

        monitor.record_render_time(component_id, Duration::from_millis(10));
        monitor.record_render_time(component_id, Duration::from_millis(20));

        let avg = monitor.get_average_render_time(component_id).unwrap();
        assert_eq!(avg, Duration::from_millis(15));
    }

    #[test]
    fn test_update_batcher() {
        let batcher = UpdateBatcher::new(Duration::from_millis(100), 5);
        let component_id = ComponentId::new();
        
        let changes = StateChanges {
            changes: vec![],
            batch_timestamp: std::time::Instant::now(),
            immediate: false,
        };

        batcher.queue_update(component_id, changes);
        let updates = batcher.flush_updates();
        assert!(updates.contains_key(&component_id));
    }

    #[test]
    fn test_memoized_component() {
        let context = Context::new();
        let props = TestProps { id: 1, version: 1 };
        let component = TestComponent::create(props, context);
        let memo_component = MemoComponent::new(component);

        assert!(memo_component.render().is_ok());
    }
}
