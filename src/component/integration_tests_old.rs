//! Integration tests for component lifecycle management system
//!
//! Tests the complete integration between ComponentTree, LifecycleManager,
//! state change detection, and update coordination.

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    
    use crate::component::{
        AnyComponent, Component, ComponentError, ComponentTree, ComponentId, Context,
        LifecycleManager, LifecyclePhase, Node, Props, StateChanges, StateValue,
        MountContext, UnmountContext,
    };
    use std::collections::HashMap;

    /// Test component with state tracking capabilities
    #[derive(Debug)]
    struct IntegrationTestComponent {
        id: ComponentId,
        context: Context,
        name: String,
        count: i32,
        // Track lifecycle method calls
        lifecycle_calls: Arc<AtomicBool>,
        mount_called: Arc<AtomicBool>,
        update_called: Arc<AtomicBool>,
        unmount_called: Arc<AtomicBool>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct IntegrationTestProps {
        name: String,
        initial_count: i32,
    }

    impl Component for IntegrationTestComponent {
        type Props = IntegrationTestProps;

        fn component_id(&self) -> ComponentId {
            self.id
        }

        fn create(props: Self::Props, context: Context) -> Self {
            Self {
                id: ComponentId::new(),
                context,
                name: props.name,
                count: props.initial_count,
                lifecycle_calls: Arc::new(AtomicBool::new(false)),
                mount_called: Arc::new(AtomicBool::new(false)),
                update_called: Arc::new(AtomicBool::new(false)),
                unmount_called: Arc::new(AtomicBool::new(false)),
            }
        }

        fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
            self.name = props.name;
            self.count = props.initial_count;
            Ok(())
        }

        fn render(&self) -> Result<Vec<Node>, ComponentError> {
            Ok(vec![Node::text(&format!("{}: {}", self.name, self.count))])
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        // Enhanced lifecycle methods
        fn on_mount(&mut self, _context: &MountContext) -> Result<(), ComponentError> {
            self.mount_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn on_update(&mut self, _changes: &StateChanges) -> Result<(), ComponentError> {
            self.update_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn on_unmount(&mut self, _context: &UnmountContext) -> Result<(), ComponentError> {
            self.unmount_called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    impl IntegrationTestComponent {
        fn was_mounted(&self) -> bool {
            self.mount_called.load(Ordering::SeqCst)
        }

        fn was_updated(&self) -> bool {
            self.update_called.load(Ordering::SeqCst)
        }

        fn was_unmounted(&self) -> bool {
            self.unmount_called.load(Ordering::SeqCst)
        }

        fn increment(&mut self) {
            self.count += 1;
        }

        fn get_count(&self) -> i32 {
            self.count
        }
    }

    fn create_test_component_instance(name: &str, context: Context) -> crate::component::ComponentInstance {
        let props = IntegrationTestProps {
            name: name.to_string(),
            initial_count: 0,
        };
        let component = IntegrationTestComponent::create(props.clone(), context);
        crate::component::ComponentInstance::new(component, props)
    }

    #[test]
    fn test_complete_lifecycle_integration() {
        // Create context and tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());

        // Create and add components
        let root_instance = create_test_component_instance("root", context.clone());
        let root_id = tree.add_component(root_instance).unwrap();
        tree.set_root(root_id).unwrap();

        let child1_instance = create_test_component_instance("child1", context.clone());
        let child1_id = tree.add_component(child1_instance).unwrap();
        tree.add_child(root_id, child1_id).unwrap();

        let child2_instance = create_test_component_instance("child2", context.clone());
        let child2_id = tree.add_component(child2_instance).unwrap();
        tree.add_child(root_id, child2_id).unwrap();

        // Mount all components
        tree.mount_component(root_id).unwrap();
        tree.mount_component(child1_id).unwrap();
        tree.mount_component(child2_id).unwrap();

        // Verify all components are mounted
        assert_eq!(tree.get_component_phase(root_id).unwrap(), LifecyclePhase::Mounted);
        assert_eq!(tree.get_component_phase(child1_id).unwrap(), LifecyclePhase::Mounted);
        assert_eq!(tree.get_component_phase(child2_id).unwrap(), LifecyclePhase::Mounted);

        // Verify mount callbacks were called
        {
            let root_component = tree.get_component(root_id).unwrap();
            let instance = root_component.read().unwrap();
            let component = instance.instance.lock().unwrap();
            let typed_component = component.as_any().downcast_ref::<IntegrationTestComponent>().unwrap();
            assert!(typed_component.was_mounted(), "Root component should be mounted");
        }

        // Test state change propagation through tree
        tree.mark_component_dirty(child1_id).unwrap();
        
        // Process updates across the tree
        tree.process_updates().unwrap();

        // Unmount components
        tree.unmount_component(child1_id).unwrap();
        tree.unmount_component(child2_id).unwrap();
        tree.unmount_component(root_id).unwrap();

        // Verify all components are unmounted
        assert_eq!(tree.get_component_phase(root_id).unwrap(), LifecyclePhase::Unmounted);
        assert_eq!(tree.get_component_phase(child1_id).unwrap(), LifecyclePhase::Unmounted);
        assert_eq!(tree.get_component_phase(child2_id).unwrap(), LifecyclePhase::Unmounted);
    }

    #[test]
    fn test_state_change_detection_integration() {
        // Create a lifecycle manager with state tracking
        let context = Context::new();
        let component_instance = create_test_component_instance("test", context.clone());
        let component_id = component_instance.id();
        
        let mut lifecycle_manager = LifecycleManager::new(component_instance, context);

        // Initialize and mount
        lifecycle_manager.initialize().unwrap();
        lifecycle_manager.mount().unwrap();

        // Simulate state changes and handle updates
        lifecycle_manager.handle_updates().unwrap();

        // Update with new props
        let new_props = IntegrationTestProps {
            name: "updated_test".to_string(),
            initial_count: 42,
        };
        lifecycle_manager.update(Box::new(new_props)).unwrap();

        // Handle updates again to process state changes
        lifecycle_manager.handle_updates().unwrap();

        // Unmount
        lifecycle_manager.unmount().unwrap();

        // Verify the lifecycle was completed successfully
        assert_eq!(lifecycle_manager.current_phase(), LifecyclePhase::Unmounted);
    }

    #[test]
    fn test_tree_wide_update_coordination() {
        // Create context and tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());

        // Create a tree structure
        let root_instance = create_test_component_instance("root", context.clone());
        let root_id = tree.add_component(root_instance).unwrap();
        tree.set_root(root_id).unwrap();

        let child1_instance = create_test_component_instance("child1", context.clone());
        let child1_id = tree.add_component(child1_instance).unwrap();
        tree.add_child(root_id, child1_id).unwrap();

        let grandchild_instance = create_test_component_instance("grandchild", context.clone());
        let grandchild_id = tree.add_component(grandchild_instance).unwrap();
        tree.add_child(child1_id, grandchild_id).unwrap();

        // Mount all components
        tree.mount_component(root_id).unwrap();
        tree.mount_component(child1_id).unwrap();
        tree.mount_component(grandchild_id).unwrap();

        // Mark grandchild as dirty
        tree.mark_component_dirty(grandchild_id).unwrap();

        // Process updates - should coordinate updates in proper order
        tree.process_updates().unwrap();

        // Verify all components are still mounted after update
        assert_eq!(tree.get_component_phase(root_id).unwrap(), LifecyclePhase::Mounted);
        assert_eq!(tree.get_component_phase(child1_id).unwrap(), LifecyclePhase::Mounted);
        assert_eq!(tree.get_component_phase(grandchild_id).unwrap(), LifecyclePhase::Mounted);
    }

    #[test]
    fn test_component_tree_dependency_ordering() {
        // Create context and tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());

        // Create multiple components to test ordering
        let root_instance = create_test_component_instance("root", context.clone());
        let root_id = tree.add_component(root_instance).unwrap();
        tree.set_root(root_id).unwrap();

        let child1_instance = create_test_component_instance("child1", context.clone());
        let child1_id = tree.add_component(child1_instance).unwrap();
        tree.add_child(root_id, child1_id).unwrap();

        let child2_instance = create_test_component_instance("child2", context.clone());
        let child2_id = tree.add_component(child2_instance).unwrap();
        tree.add_child(root_id, child2_id).unwrap();

        // Mount all components
        tree.mount_component(root_id).unwrap();
        tree.mount_component(child1_id).unwrap();
        tree.mount_component(child2_id).unwrap();

        // Mark all components as needing updates
        tree.mark_component_dirty(root_id).unwrap();
        tree.mark_component_dirty(child1_id).unwrap();
        tree.mark_component_dirty(child2_id).unwrap();

        // Get components that need updates
        let components_needing_updates = tree.get_components_needing_updates().unwrap();
        
        // All dirty components should be identified
        assert!(components_needing_updates.contains(&root_id));
        assert!(components_needing_updates.contains(&child1_id));
        assert!(components_needing_updates.contains(&child2_id));

        // Sort by dependency order (parents first)
        let sorted_components = tree.sort_by_dependency_order(components_needing_updates).unwrap();
        
        // Root should come first, then children (order of children doesn't matter)
        assert_eq!(sorted_components[0], root_id);
        assert!(sorted_components.contains(&child1_id));
        assert!(sorted_components.contains(&child2_id));
        
        // Process updates in dependency order
        tree.process_updates().unwrap();
    }
}
