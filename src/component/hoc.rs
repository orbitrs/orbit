//! Higher-order components (HOCs) for Orbit UI framework
//!
//! This module provides utilities for creating higher-order components that wrap
//! existing components with additional functionality.

use std::marker::PhantomData;

use crate::component::{
    Component, ComponentError, ComponentId, Context, LifecyclePhase, MountContext, Node, Props,
    StateChanges, UnmountContext,
};

/// Trait for defining higher-order component behavior
pub trait HigherOrderComponent<T: Component> {
    /// The props type for the HOC itself (should match T::Props)
    type HOCProps: Props + Clone;    /// Transform HOC props into wrapped component props
    fn transform_props(hoc_props: &Self::HOCProps) -> T::Props;

    /// Optional: modify the wrapped component after creation
    fn enhance_component(component: &mut T, hoc_props: &Self::HOCProps) -> Result<(), ComponentError> {
        // Default implementation does nothing
        let _ = (component, hoc_props);
        Ok(())
    }

    /// Optional: intercept lifecycle events
    fn on_wrapped_mount(
        component: &mut T,
        context: &MountContext,
        hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        let _ = (component, context, hoc_props);
        Ok(())
    }

    /// Optional: intercept state changes
    fn on_wrapped_update(
        component: &mut T,
        changes: &StateChanges,
        hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        let _ = (component, changes, hoc_props);
        Ok(())
    }

    /// Optional: intercept unmount
    fn on_wrapped_unmount(
        component: &mut T,
        context: &UnmountContext,
        hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        let _ = (component, context, hoc_props);
        Ok(())
    }
}

/// A generic higher-order component wrapper
pub struct HOCWrapper<H, T>
where
    H: HigherOrderComponent<T>,
    T: Component,
{
    /// The wrapped component instance
    wrapped_component: T,
    /// HOC-specific props
    hoc_props: H::HOCProps,
    /// Component base functionality
    component_id: ComponentId,
    /// Current lifecycle phase
    lifecycle_phase: LifecyclePhase,
    /// Phantom data for type safety
    _phantom: PhantomData<H>,
}

impl<H, T> HOCWrapper<H, T>
where
    H: HigherOrderComponent<T>,
    T: Component,
{    /// Create a new HOC wrapper
    pub fn new(hoc_props: H::HOCProps, context: Context) -> Result<Self, ComponentError>
    where
        T: Component,
    {
        let wrapped_props = H::transform_props(&hoc_props);
        let mut wrapped_component = T::create(wrapped_props, context);

        // Allow HOC to enhance the component
        H::enhance_component(&mut wrapped_component, &hoc_props)?;

        Ok(Self {
            wrapped_component,
            hoc_props,
            component_id: ComponentId::new(),
            lifecycle_phase: LifecyclePhase::Created,
            _phantom: PhantomData,
        })
    }

    /// Get a reference to the wrapped component
    pub fn wrapped_component(&self) -> &T {
        &self.wrapped_component
    }

    /// Get a mutable reference to the wrapped component
    pub fn wrapped_component_mut(&mut self) -> &mut T {
        &mut self.wrapped_component
    }

    /// Get the HOC props
    pub fn hoc_props(&self) -> &H::HOCProps {
        &self.hoc_props
    }
}

impl<H, T> Component for HOCWrapper<H, T>
where
    H: HigherOrderComponent<T> + Send + Sync + 'static,
    T: Component + Send + Sync + 'static,
    H::HOCProps: Send + Sync + 'static,
{
    type Props = H::HOCProps;

    fn component_id(&self) -> ComponentId {
        self.component_id
    }

    fn create(props: Self::Props, context: Context) -> Self
    where
        Self: Sized,
    {
        Self::new(props, context).expect("Failed to create HOC wrapper")
    }

    fn initialize(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.initialize()
    }

    fn before_mount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.before_mount()
    }

    fn on_mount(&mut self, context: &MountContext) -> Result<(), ComponentError> {
        // Call HOC interceptor first
        H::on_wrapped_mount(&mut self.wrapped_component, context, &self.hoc_props)?;
        // Then call wrapped component's mount
        self.wrapped_component.on_mount(context)
    }

    fn after_mount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.after_mount()
    }

    fn on_update(&mut self, changes: &StateChanges) -> Result<(), ComponentError> {
        // Call HOC interceptor first
        H::on_wrapped_update(&mut self.wrapped_component, changes, &self.hoc_props)?;
        // Then call wrapped component's update
        self.wrapped_component.on_update(changes)
    }    fn should_update(&self, new_props: &Self::Props) -> bool {
        // Transform props and check if wrapped component should update
        let wrapped_props = H::transform_props(new_props);
        self.wrapped_component.should_update(&wrapped_props)
    }

    fn before_update(&mut self, new_props: &Self::Props) -> Result<(), ComponentError> {
        let wrapped_props = H::transform_props(new_props);
        self.wrapped_component.before_update(&wrapped_props)
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        let wrapped_props = H::transform_props(&props);
        self.hoc_props = props;
        self.wrapped_component.update(wrapped_props)
    }

    fn after_update(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.after_update()
    }

    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.before_unmount()
    }

    fn on_unmount(&mut self, context: &UnmountContext) -> Result<(), ComponentError> {
        // Call HOC interceptor first
        H::on_wrapped_unmount(&mut self.wrapped_component, context, &self.hoc_props)?;
        // Then call wrapped component's unmount
        self.wrapped_component.on_unmount(context)
    }

    fn after_unmount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.after_unmount()
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        self.wrapped_component.render()
    }    fn lifecycle_phase(&self) -> LifecyclePhase {
        self.lifecycle_phase
    }

    fn set_lifecycle_phase(&mut self, phase: LifecyclePhase) {
        self.lifecycle_phase = phase;
        self.wrapped_component.set_lifecycle_phase(phase);
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.mount()
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.unmount()
    }

    fn cleanup(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.cleanup()
    }

    fn state_changed(&mut self, state_key: &str) -> Result<(), ComponentError> {
        self.wrapped_component.state_changed(state_key)
    }

    fn request_update(&mut self) -> Result<(), ComponentError> {
        self.wrapped_component.request_update()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Common HOC patterns

/// HOC that adds logging to component lifecycle events
pub struct WithLogging;

impl<T: Component> HigherOrderComponent<T> for WithLogging {
    type HOCProps = T::Props;

    fn transform_props(hoc_props: &Self::HOCProps) -> T::Props {
        hoc_props.clone()
    }fn on_wrapped_mount(
        component: &mut T,
        context: &MountContext,
        _hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        println!("🔄 Component {} mounted", Component::component_id(component));
        Ok(())
    }

    fn on_wrapped_update(
        component: &mut T,
        changes: &StateChanges,
        _hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        println!("🔄 Component {} updated with {} changes", 
                Component::component_id(component), changes.changes.len());
        Ok(())
    }

    fn on_wrapped_unmount(
        component: &mut T,
        context: &UnmountContext,
        _hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        println!("🔄 Component {} unmounted", Component::component_id(component));
        Ok(())
    }
}

/// HOC that adds performance monitoring
pub struct WithPerformanceMonitoring;

impl<T: Component> HigherOrderComponent<T> for WithPerformanceMonitoring {
    type HOCProps = T::Props;

    fn transform_props(hoc_props: &Self::HOCProps) -> T::Props {
        hoc_props.clone()
    }

    fn on_wrapped_mount(
        component: &mut T,
        context: &MountContext,
        _hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        let start = std::time::Instant::now();
        let result = component.mount();
        let duration = start.elapsed();
        println!("⚡ Component {} mount took {:?}", Component::component_id(component), duration);
        result
    }

    fn on_wrapped_update(
        component: &mut T,
        changes: &StateChanges,
        _hoc_props: &Self::HOCProps,
    ) -> Result<(), ComponentError> {
        let start = std::time::Instant::now();
        let result = component.on_update(changes);
        let duration = start.elapsed();
        println!("⚡ Component {} update took {:?}", Component::component_id(component), duration);
        result
    }
}

/// Convenient type aliases for common HOCs
pub type LoggedComponent<T> = HOCWrapper<WithLogging, T>;
pub type MonitoredComponent<T> = HOCWrapper<WithPerformanceMonitoring, T>;

/// Macro for easy HOC creation
#[macro_export]
macro_rules! with_hoc {
    ($hoc:ty, $component:ty, $props:expr, $context:expr) => {
        HOCWrapper::<$hoc, $component>::new($props, $context)
    };
}

/// Macro for chaining multiple HOCs
#[macro_export]
macro_rules! hoc_chain {
    ($component:ty, $props:expr, $context:expr, $($hoc:ty),+) => {{
        let mut component = $component::create($props, $context);
        $(
            component = HOCWrapper::<$hoc, _>::new(component.props().clone(), $context)?;
        )+
        component
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{ComponentBase, Context};

    #[derive(Clone, Debug)]
    struct TestProps {
        name: String,
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

    #[test]
    fn test_logging_hoc() {
        let context = Context::new();
        let props = TestProps {
            name: "test".to_string(),
        };

        let mut logged_component = LoggedComponent::<TestComponent>::new(props, context).unwrap();
        
        // Test that we can access the wrapped component
        assert_eq!(logged_component.wrapped_component().props.name, "test");
        
        // Test lifecycle methods work
        assert!(logged_component.initialize().is_ok());
        assert!(logged_component.before_mount().is_ok());
    }

    #[test]
    fn test_performance_monitoring_hoc() {
        let context = Context::new();
        let props = TestProps {
            name: "test".to_string(),
        };

        let mut monitored_component = MonitoredComponent::<TestComponent>::new(props, context).unwrap();
        
        // Test that monitoring doesn't break functionality
        assert!(monitored_component.initialize().is_ok());
    }
}
