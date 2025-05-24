//! Tests for component lifecycle management

use crate::component::{Component, ComponentError, Context, LifecycleManager, Node};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// A simple test component with props
#[derive(Debug)]
struct TestComponent {
    #[allow(dead_code)]
    context: Context,
    props: Arc<Mutex<TestProps>>,
    state: TestComponentState,
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

    fn create(props: Self::Props, context: Context) -> Self {
        Self {
            context,
            props: Arc::new(Mutex::new(props)),
            state: TestComponentState::default(),
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
