//! Tests for component lifecycle management

use crate::component::{
    Component, ComponentError, ComponentId, ComponentInstance, Context, LifecycleManager,
    MountContext, Node, UnmountContext, UnmountReason,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// A simple test component with props
#[derive(Debug)]
struct TestComponent {
    #[allow(dead_code)]
    context: Context,
    props: Arc<Mutex<TestProps>>,
    state: TestComponentState,
    id: ComponentId,
}

#[derive(Debug)]
struct TestComponentState {
    mount_called: AtomicBool,
    update_called: AtomicBool,
    unmount_called: AtomicBool,
    before_update_called: AtomicBool,
    after_update_called: AtomicBool,
    before_unmount_called: AtomicBool,
}

impl Default for TestComponentState {
    fn default() -> Self {
        Self {
            mount_called: AtomicBool::new(false),
            update_called: AtomicBool::new(false),
            unmount_called: AtomicBool::new(false),
            before_update_called: AtomicBool::new(false),
            after_update_called: AtomicBool::new(false),
            before_unmount_called: AtomicBool::new(false),
        }
    }
}

#[derive(Debug, Clone)]
struct TestProps {
    message: String,
}

impl Component for TestComponent {
    type Props = TestProps;

    fn component_id(&self) -> ComponentId {
        self.id
    }

    fn create(props: Self::Props, context: Context) -> Self {
        Self {
            context,
            props: Arc::new(Mutex::new(props)),
            state: TestComponentState::default(),
            id: ComponentId::new(),
        }
    }

    fn initialize(&mut self) -> Result<(), ComponentError> {
        // We don't need to set up the mount callback here for this test
        Ok(())
    }

    fn mount(&mut self) -> Result<(), ComponentError> {
        self.state.mount_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn before_update(&mut self, new_props: &Self::Props) -> Result<(), ComponentError> {
        self.state
            .before_update_called
            .store(true, Ordering::SeqCst);
        assert_eq!(new_props.message, "Updated", "Expected updated message");
        Ok(())
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.state.update_called.store(true, Ordering::SeqCst);
        *self.props.lock().unwrap() = props;
        Ok(())
    }

    fn after_update(&mut self) -> Result<(), ComponentError> {
        self.state.after_update_called.store(true, Ordering::SeqCst);
        assert_eq!(
            self.props.lock().unwrap().message,
            "Updated",
            "Props should be updated"
        );
        Ok(())
    }

    fn before_unmount(&mut self) -> Result<(), ComponentError> {
        self.state
            .before_unmount_called
            .store(true, Ordering::SeqCst);
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), ComponentError> {
        self.state.unmount_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // No rendering in this test
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Test helper functions
impl TestComponent {
    fn is_mounted(&self) -> bool {
        self.state.mount_called.load(Ordering::SeqCst)
    }

    fn is_updated(&self) -> bool {
        self.state.update_called.load(Ordering::SeqCst)
    }

    fn is_unmounted(&self) -> bool {
        self.state.unmount_called.load(Ordering::SeqCst)
    }
}

#[test]
fn test_component_lifecycle() {
    // Create a component
    let props = TestProps {
        message: "Hello".to_string(),
    };
    let context = Context::new();
    let component = TestComponent::create(props, context.clone());

    // Create a component instance
    let instance = crate::component::ComponentInstance::new(
        component,
        TestProps {
            message: "Hello".to_string(),
        },
    );

    // Create a lifecycle manager
    let mut lifecycle_manager = LifecycleManager::new(instance, context);

    // Initialize the component
    lifecycle_manager
        .initialize()
        .expect("initialization should succeed");

    // Mount the component
    lifecycle_manager.mount().expect("mount should succeed");

    // Get the component and verify it was mounted
    {
        let instance = lifecycle_manager.get_component().lock().unwrap();
        let component = instance.instance.lock().unwrap();
        let component = component.as_any().downcast_ref::<TestComponent>().unwrap();
        assert!(component.is_mounted(), "mount should have been called");
    }

    // Update the component
    let updated_props = TestProps {
        message: "Updated".to_string(),
    };
    lifecycle_manager
        .update(Box::new(updated_props))
        .expect("update should succeed");

    // Verify the component was updated
    {
        let instance = lifecycle_manager.get_component().lock().unwrap();
        let component = instance.instance.lock().unwrap();
        let component = component.as_any().downcast_ref::<TestComponent>().unwrap();
        assert!(component.is_updated(), "update should have been called");
        assert!(
            component.state.before_update_called.load(Ordering::SeqCst),
            "before_update should have been called"
        );
        assert!(
            component.state.after_update_called.load(Ordering::SeqCst),
            "after_update should have been called"
        );
        assert_eq!(
            component.props.lock().unwrap().message,
            "Updated",
            "Props should be updated"
        );
    }

    // Unmount the component
    lifecycle_manager.unmount().expect("unmount should succeed");

    // Verify the component was unmounted
    {
        let instance = lifecycle_manager.get_component().lock().unwrap();
        let component = instance.instance.lock().unwrap();
        let component = component.as_any().downcast_ref::<TestComponent>().unwrap();
        assert!(component.is_unmounted(), "unmount should have been called");
        assert!(
            component.state.before_unmount_called.load(Ordering::SeqCst),
            "before_unmount should have been called"
        );
    }
}

// Additional tests for enhanced component system
#[cfg(test)]
mod enhanced_tests {
    use super::*;
    use crate::component::{
        AnyComponent, ComponentBase, ComponentId, LifecyclePhase, UpdateScheduler,
    };

    #[test]
    fn test_component_id_generation() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();

        assert_ne!(id1, id2);
        assert!(id1.id() < id2.id()); // Test display formatting
        assert!(format!("{id1}").starts_with("Component#"));
    }

    #[test]
    fn test_component_base() {
        let context = Context::new();
        let mut base = ComponentBase::new(context);

        assert_eq!(
            AnyComponent::lifecycle_phase(&base),
            LifecyclePhase::Created
        );

        base.set_lifecycle_phase(LifecyclePhase::Mounted);
        assert_eq!(
            AnyComponent::lifecycle_phase(&base),
            LifecyclePhase::Mounted
        );
    }

    #[test]
    fn test_update_scheduler() {
        let mut scheduler = UpdateScheduler::default();
        let component_id = ComponentId::new();

        assert!(!scheduler.has_pending_update(component_id));

        scheduler.schedule_update(component_id);
        assert!(scheduler.has_pending_update(component_id));

        let pending = scheduler.get_pending_components();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0], component_id);

        scheduler.clear_pending(component_id);
        assert!(!scheduler.has_pending_update(component_id));
    }

    #[test]
    fn test_context_reactive_state() {
        let context = Context::new();
        let component_id = ComponentId::new();

        let state = context.create_reactive_state(42, component_id);
        assert_eq!(state.get(), 42);

        // Test that updating state triggers component update scheduling
        state.set(100);
        assert!(context.has_pending_update(component_id));
    }

    #[test]
    fn test_context_provider_basic() {
        let provider = crate::component::ContextProvider::new();

        // Test providing and consuming values
        provider.provide(42i32).unwrap();
        provider.provide("hello".to_string()).unwrap();

        assert_eq!(provider.consume::<i32>(), Some(42));
        assert_eq!(provider.consume::<String>(), Some("hello".to_string()));
        assert_eq!(provider.consume::<f64>(), None);

        // Test type checking
        assert!(provider.has::<i32>());
        assert!(provider.has::<String>());
        assert!(!provider.has::<f64>());
    }

    #[test]
    fn test_context_provider_inheritance() {
        let parent = crate::component::ContextProvider::new();
        parent.provide(42i32).unwrap();

        let child = crate::component::ContextProvider::with_parent(parent);
        child.provide("child_value".to_string()).unwrap();

        // Child should have access to both parent and child values
        assert_eq!(child.consume::<i32>(), Some(42));
        assert_eq!(child.consume::<String>(), Some("child_value".to_string()));
    }

    // Test struct for Props trait
    #[derive(Debug, Clone, PartialEq)]
    struct TestNewProps {
        pub title: String,
        pub count: i32,
    }

    impl Default for TestNewProps {
        fn default() -> Self {
            Self {
                title: "Default".to_string(),
                count: 0,
            }
        }
    }

    // Test component implementing the enhanced Component trait
    struct TestNewComponent {
        base: ComponentBase,
        props: TestNewProps,
    }

    impl TestNewComponent {
        pub fn new(props: TestNewProps, context: Context) -> Self {
            Self {
                base: ComponentBase::new(context),
                props,
            }
        }
    }

    impl Component for TestNewComponent {
        type Props = TestNewProps;

        fn component_id(&self) -> ComponentId {
            self.base.id()
        }

        fn create(props: Self::Props, context: Context) -> Self {
            Self::new(props, context)
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
    fn test_enhanced_component_creation() {
        let context = Context::new();
        let props = TestNewProps::default();
        let component = TestNewComponent::create(props.clone(), context);

        assert_eq!(component.props, props);
        assert_eq!(
            AnyComponent::lifecycle_phase(&component),
            LifecyclePhase::Created
        );
    }

    #[test]
    fn test_enhanced_component_update() {
        let context = Context::new();
        let initial_props = TestNewProps::default();
        let mut component = TestNewComponent::create(initial_props, context);

        let new_props = TestNewProps {
            title: "Updated".to_string(),
            count: 42,
        };

        assert!(component.update(new_props.clone()).is_ok());
        assert_eq!(component.props, new_props);
    }
    #[test]
    fn test_any_component_trait() {
        let context = Context::new();
        let props = TestNewProps::default();
        let component = TestNewComponent::create(props, context);

        // Test type name
        assert_eq!(
            component.type_name(),
            std::any::type_name::<TestNewComponent>()
        );

        // Test that we can use it as AnyComponent
        let any_component: &dyn AnyComponent = &component;
        assert_eq!(
            AnyComponent::component_id(any_component),
            Component::component_id(&component)
        );
        assert_eq!(
            AnyComponent::lifecycle_phase(any_component),
            LifecyclePhase::Created
        );
    }

    #[test]
    fn test_component_base_with_layout() {
        let context = Context::new();
        let layout_style = crate::layout::LayoutStyle {
            width: crate::layout::Dimension::Points(100.0),
            height: crate::layout::Dimension::Points(200.0),
            flex_direction: crate::layout::FlexDirection::Column,
            ..Default::default()
        };

        let base = ComponentBase::new_with_layout(context, layout_style.clone());

        assert_eq!(*base.layout_style(), layout_style);
        assert_eq!(
            base.layout_style().width,
            crate::layout::Dimension::Points(100.0)
        );
        assert_eq!(
            base.layout_style().height,
            crate::layout::Dimension::Points(200.0)
        );
        assert_eq!(
            base.layout_style().flex_direction,
            crate::layout::FlexDirection::Column
        );
    }

    #[test]
    fn test_component_layout_node_creation() {
        let context = Context::new();
        let layout_style = crate::layout::LayoutStyle {
            width: crate::layout::Dimension::Points(150.0),
            height: crate::layout::Dimension::Points(100.0),
            ..Default::default()
        };

        let base = ComponentBase::new_with_layout(context, layout_style);
        let layout_node = base.create_layout_node();

        assert_eq!(layout_node.id, base.id());
        assert_eq!(
            layout_node.style.width,
            crate::layout::Dimension::Points(150.0)
        );
        assert_eq!(
            layout_node.style.height,
            crate::layout::Dimension::Points(100.0)
        );
        assert!(layout_node.children.is_empty());
        assert!(layout_node.layout.is_dirty);
    }

    #[test]
    fn test_component_layout_style_modification() {
        let context = Context::new();
        let mut base = ComponentBase::new(context);

        // Initially should have default layout
        assert_eq!(base.layout_style().width, crate::layout::Dimension::Auto);

        // Modify layout style
        {
            let layout_style = base.layout_style_mut();
            layout_style.width = crate::layout::Dimension::Points(200.0);
            layout_style.flex_grow = 1.0;
        }

        assert_eq!(
            base.layout_style().width,
            crate::layout::Dimension::Points(200.0)
        );
        assert_eq!(base.layout_style().flex_grow, 1.0);

        // Set new layout style entirely
        let new_style = crate::layout::LayoutStyle {
            height: crate::layout::Dimension::Percent(50.0),
            flex_direction: crate::layout::FlexDirection::Row,
            ..Default::default()
        };

        base.set_layout_style(new_style);
        assert_eq!(
            base.layout_style().height,
            crate::layout::Dimension::Percent(50.0)
        );
        assert_eq!(
            base.layout_style().flex_direction,
            crate::layout::FlexDirection::Row
        );
    }

    #[test]
    fn test_component_layout_integration() {
        use crate::layout::{LayoutEngine, Size};

        let context = Context::new();
        let layout_style = crate::layout::LayoutStyle {
            width: crate::layout::Dimension::Points(100.0),
            height: crate::layout::Dimension::Points(50.0),
            flex_direction: crate::layout::FlexDirection::Row,
            ..Default::default()
        };

        let component = ComponentBase::new_with_layout(context, layout_style);
        let mut layout_node = component.create_layout_node();

        // Run layout calculation
        let mut layout_engine = LayoutEngine::new();
        let container_size = Size::new(400.0, 300.0);
        let result = layout_engine.calculate_layout(&mut layout_node, container_size);

        assert!(result.is_ok());
        assert_eq!(layout_node.layout.rect.width(), 100.0);
        assert_eq!(layout_node.layout.rect.height(), 50.0);
        assert!(!layout_node.layout.is_dirty);
    }
    #[test]
    fn test_enhanced_lifecycle_with_contexts() {
        let context = Context::new();
        // Create a TestComponent with props
        let test_props = TestProps {
            message: "Test".to_string(),
        };
        let test_component = TestComponent::create(test_props, context.clone());
        let component_instance = ComponentInstance::new(
            test_component,
            TestProps {
                message: "Test".to_string(),
            },
        );
        let mut lifecycle_manager = LifecycleManager::new(component_instance, Context::new());

        // Test enhanced mount with context
        assert!(lifecycle_manager.mount().is_ok());
        assert_eq!(lifecycle_manager.current_phase(), LifecyclePhase::Mounted);

        // Test enhanced unmount with context
        assert!(lifecycle_manager.unmount().is_ok());
        assert_eq!(lifecycle_manager.current_phase(), LifecyclePhase::Unmounted);
    }

    #[test]
    fn test_mount_context_creation() {
        let component_id = ComponentId::new();
        let mount_context = MountContext::new(component_id);

        assert_eq!(mount_context.component_id, component_id);
        assert!(mount_context.parent_id.is_none());
    }

    #[test]
    fn test_unmount_context_creation() {
        let component_id = ComponentId::new();
        let unmount_context = UnmountContext::new(component_id, UnmountReason::Removed);

        assert_eq!(unmount_context.component_id, component_id);
        assert_eq!(unmount_context.reason, UnmountReason::Removed);
        assert!(!unmount_context.force_cleanup);
    }
}
