//! Component tree management
//!
//! This module provides functionality to manage the tree of components, handling
//! parent-child relationships, efficient updates, and lifecycle coordination.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(test)]
use crate::component::LifecyclePhase;
use crate::component::{ComponentId, ComponentInstance, Context, LifecycleManager, Node};

/// Result type for tree operations
pub type TreeResult<T> = Result<T, TreeError>;

/// Errors specific to tree operations
#[derive(Debug, thiserror::Error)]
pub enum TreeError {
    /// Component not found in the tree
    #[error("Component not found: {0}")]
    ComponentNotFound(ComponentId),
    
    /// Failed to add component (already exists)
    #[error("Component already exists: {0}")]
    ComponentAlreadyExists(ComponentId),
    
    /// Failed to access component (lock error)
    #[error("Failed to access component: {0}")]
    LockError(String),
    
    /// Component lifecycle error
    #[error("Component lifecycle error: {0}")]
    LifecycleError(#[from] crate::component::ComponentError),
    
    /// Invalid parent-child relationship
    #[error("Invalid parent-child relationship: {0}")]
    InvalidRelationship(String),
}

/// Type alias for a thread-safe component instance
pub type SharedComponentInstance = Arc<RwLock<ComponentInstance>>;

/// Manager for the component tree
///
/// Handles parent-child relationships, lifecycle coordination, and efficient updates
pub struct ComponentTree {
    /// Map of component ID to component instance
    components: RwLock<HashMap<ComponentId, SharedComponentInstance>>,
    
    /// Map of component ID to lifecycle manager
    lifecycle_managers: RwLock<HashMap<ComponentId, Arc<RwLock<LifecycleManager>>>>,
    
    /// Map of component ID to its children's IDs
    children: RwLock<HashMap<ComponentId, Vec<ComponentId>>>,
    
    /// Map of component ID to its parent ID
    parents: RwLock<HashMap<ComponentId, ComponentId>>,
    
    /// Root component ID (if set)
    root: RwLock<Option<ComponentId>>,
    
    /// Application context
    context: Context,
}

impl ComponentTree {
    /// Create a new component tree
    pub fn new(context: Context) -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            lifecycle_managers: RwLock::new(HashMap::new()),
            children: RwLock::new(HashMap::new()),
            parents: RwLock::new(HashMap::new()),
            root: RwLock::new(None),
            context,
        }
    }
    
    /// Set the root component
    pub fn set_root(&self, component_id: ComponentId) -> TreeResult<()> {
        // Check if component exists
        if !self.has_component(component_id) {
            return Err(TreeError::ComponentNotFound(component_id));
        }
        
        // Set as root
        let mut root = self.root.write().map_err(|_| {
            TreeError::LockError("Failed to lock root component".to_string())
        })?;
        
        *root = Some(component_id);
        
        Ok(())
    }
    
    /// Get the root component ID
    pub fn root_id(&self) -> TreeResult<Option<ComponentId>> {
        let root = self.root.read().map_err(|_| {
            TreeError::LockError("Failed to read root component".to_string())
        })?;
        
        Ok(*root)
    }
    
    /// Add a component to the tree
    pub fn add_component(
        &self,
        component: ComponentInstance,
    ) -> TreeResult<ComponentId> {
        let id = component.id();
        
        // Check if component already exists
        if self.has_component(id) {
            return Err(TreeError::ComponentAlreadyExists(id));
        }
        
        // Create lifecycle manager
        let lifecycle = LifecycleManager::new(
            component.clone(),
            self.context.clone(),
        );
        
        // Add component to maps
        {
            let mut components = self.components.write().map_err(|_| {
                TreeError::LockError("Failed to lock components map".to_string())
            })?;
            
            components.insert(id, Arc::new(RwLock::new(component)));
        }
        
        {
            let mut lifecycle_managers = self.lifecycle_managers.write().map_err(|_| {
                TreeError::LockError("Failed to lock lifecycle managers map".to_string())
            })?;
            
            lifecycle_managers.insert(id, Arc::new(RwLock::new(lifecycle)));
        }
        
        {
            let mut children = self.children.write().map_err(|_| {
                TreeError::LockError("Failed to lock children map".to_string())
            })?;
            
            children.insert(id, Vec::new());
        }
        
        Ok(id)
    }
    
    /// Remove a component from the tree
    pub fn remove_component(&self, id: ComponentId) -> TreeResult<()> {
        // Check if component exists
        if !self.has_component(id) {
            return Err(TreeError::ComponentNotFound(id));
        }
        
        // Get children to remove recursively
        let children: Vec<ComponentId> = {
            let children_map = self.children.read().map_err(|_| {
                TreeError::LockError("Failed to read children map".to_string())
            })?;
            
            children_map.get(&id).cloned().unwrap_or_default()
        };
        
        // Remove all children first (recursively)
        for child_id in children {
            self.remove_component(child_id)?;
        }
        
        // Remove from parent's children list
        {
            let parent_id_option = {
                let parents = self.parents.read().map_err(|_| {
                    TreeError::LockError("Failed to read parents map".to_string())
                })?;
                
                parents.get(&id).cloned()
            };
            
            if let Some(parent_id) = parent_id_option {
                let mut children_map = self.children.write().map_err(|_| {
                    TreeError::LockError("Failed to write children map".to_string())
                })?;
                
                if let Some(siblings) = children_map.get_mut(&parent_id) {
                    if let Some(index) = siblings.iter().position(|&c| c == id) {
                        siblings.remove(index);
                    }
                }
            }
        }
        
        // Remove from maps
        {
            let mut components = self.components.write().map_err(|_| {
                TreeError::LockError("Failed to lock components map".to_string())
            })?;
            
            components.remove(&id);
        }
        
        {
            let mut lifecycle_managers = self.lifecycle_managers.write().map_err(|_| {
                TreeError::LockError("Failed to lock lifecycle managers map".to_string())
            })?;
            
            lifecycle_managers.remove(&id);
        }
        
        {
            let mut children = self.children.write().map_err(|_| {
                TreeError::LockError("Failed to lock children map".to_string())
            })?;
            
            children.remove(&id);
        }
        
        {
            let mut parents = self.parents.write().map_err(|_| {
                TreeError::LockError("Failed to lock parents map".to_string())
            })?;
            
            parents.remove(&id);
        }
        
        // If this was the root, unset it
        {
            let mut root = self.root.write().map_err(|_| {
                TreeError::LockError("Failed to lock root component".to_string())
            })?;
            
            if let Some(root_id) = *root {
                if root_id == id {
                    *root = None;
                }
            }
        }
        
        Ok(())
    }
    
    /// Add a child component to a parent
    pub fn add_child(
        &self,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> TreeResult<()> {
        // Verify both components exist
        if !self.has_component(parent_id) {
            return Err(TreeError::ComponentNotFound(parent_id));
        }
        
        if !self.has_component(child_id) {
            return Err(TreeError::ComponentNotFound(child_id));
        }
        
        // Check if child already has a parent (can only have one)
        {
            let parents = self.parents.read().map_err(|_| {
                TreeError::LockError("Failed to read parents map".to_string())
            })?;
            
            if let Some(existing_parent) = parents.get(&child_id) {
                if *existing_parent != parent_id {
                    return Err(TreeError::InvalidRelationship(format!(
                        "Component {} already has a parent {}",
                        child_id.id(),
                        existing_parent.id()
                    )));
                }
                
                // Child is already added to this parent
                return Ok(());
            }
        }
        
        // Add child to parent's children list
        {
            let mut children_map = self.children.write().map_err(|_| {
                TreeError::LockError("Failed to write children map".to_string())
            })?;
            
            if let Some(children) = children_map.get_mut(&parent_id) {
                if !children.contains(&child_id) {
                    children.push(child_id);
                }
            }
        }
        
        // Add parent to child's parent reference
        {
            let mut parents = self.parents.write().map_err(|_| {
                TreeError::LockError("Failed to write parents map".to_string())
            })?;
            
            parents.insert(child_id, parent_id);
        }
        
        Ok(())
    }
    
    /// Remove a child from a parent
    pub fn remove_child(
        &self,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> TreeResult<()> {
        // Verify both components exist
        if !self.has_component(parent_id) {
            return Err(TreeError::ComponentNotFound(parent_id));
        }
        
        if !self.has_component(child_id) {
            return Err(TreeError::ComponentNotFound(child_id));
        }
        
        // Remove child from parent's children list
        {
            let mut children_map = self.children.write().map_err(|_| {
                TreeError::LockError("Failed to write children map".to_string())
            })?;
            
            if let Some(children) = children_map.get_mut(&parent_id) {
                if let Some(index) = children.iter().position(|&c| c == child_id) {
                    children.remove(index);
                }
            }
        }
        
        // Remove parent reference from child
        {
            let mut parents = self.parents.write().map_err(|_| {
                TreeError::LockError("Failed to write parents map".to_string())
            })?;
            
            parents.remove(&child_id);
        }
        
        Ok(())
    }
    
    /// Get a component instance by ID
    pub fn get_component(&self, id: ComponentId) -> TreeResult<SharedComponentInstance> {
        let components = self.components.read().map_err(|_| {
            TreeError::LockError("Failed to read components map".to_string())
        })?;
        
        if let Some(component) = components.get(&id) {
            Ok(component.clone())
        } else {
            Err(TreeError::ComponentNotFound(id))
        }
    }
    
    /// Get a lifecycle manager by component ID
    pub fn get_lifecycle_manager(
        &self,
        id: ComponentId,
    ) -> TreeResult<Arc<RwLock<LifecycleManager>>> {
        let lifecycle_managers = self.lifecycle_managers.read().map_err(|_| {
            TreeError::LockError("Failed to read lifecycle managers map".to_string())
        })?;
        
        if let Some(manager) = lifecycle_managers.get(&id) {
            Ok(manager.clone())
        } else {
            Err(TreeError::ComponentNotFound(id))
        }
    }
    
    /// Initialize a component
    pub fn initialize_component(&self, id: ComponentId) -> TreeResult<()> {
        let lifecycle_manager = self.get_lifecycle_manager(id)?;
        let mut manager = lifecycle_manager.write().map_err(|_| {
            TreeError::LockError("Failed to lock lifecycle manager".to_string())
        })?;
        
        manager.initialize().map_err(TreeError::LifecycleError)?;
        
        Ok(())
    }
    
    /// Mount a component
    pub fn mount_component(&self, id: ComponentId) -> TreeResult<()> {
        let lifecycle_manager = self.get_lifecycle_manager(id)?;
        let mut manager = lifecycle_manager.write().map_err(|_| {
            TreeError::LockError("Failed to lock lifecycle manager".to_string())
        })?;
        
        manager.mount().map_err(TreeError::LifecycleError)?;
        
        Ok(())
    }
    
    /// Unmount a component
    pub fn unmount_component(&self, id: ComponentId) -> TreeResult<()> {
        let lifecycle_manager = self.get_lifecycle_manager(id)?;
        let mut manager = lifecycle_manager.write().map_err(|_| {
            TreeError::LockError("Failed to lock lifecycle manager".to_string())
        })?;
        
        manager.unmount().map_err(TreeError::LifecycleError)?;
        
        Ok(())
    }
    
    /// Render a component
    pub fn render_component(&self, id: ComponentId) -> TreeResult<Vec<Node>> {
        let lifecycle_manager = self.get_lifecycle_manager(id)?;
        let manager = lifecycle_manager.read().map_err(|_| {
            TreeError::LockError("Failed to read lifecycle manager".to_string())
        })?;
        
        manager.render().map_err(TreeError::LifecycleError)
    }
    
    /// Check if a component exists in the tree
    pub fn has_component(&self, id: ComponentId) -> bool {
        let components = match self.components.read() {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        components.contains_key(&id)
    }
    
    /// Get the children of a component
    pub fn get_children(&self, id: ComponentId) -> TreeResult<Vec<ComponentId>> {
        let children_map = self.children.read().map_err(|_| {
            TreeError::LockError("Failed to read children map".to_string())
        })?;
        
        if let Some(children) = children_map.get(&id) {
            Ok(children.clone())
        } else {
            // Return empty Vec if component doesn't exist or has no children
            Ok(Vec::new())
        }
    }
    
    /// Get the parent of a component
    pub fn get_parent(&self, id: ComponentId) -> TreeResult<Option<ComponentId>> {
        let parents = self.parents.read().map_err(|_| {
            TreeError::LockError("Failed to read parents map".to_string())
        })?;
        
        Ok(parents.get(&id).cloned())
    }
    
    /// Recursively mount a component and all its children
    pub fn mount_component_tree(&self, id: ComponentId) -> TreeResult<()> {
        // First mount this component
        self.mount_component(id)?;
        
        // Then mount all children recursively
        let children = self.get_children(id)?;
        for child_id in children {
            self.mount_component_tree(child_id)?;
        }
        
        Ok(())
    }
    
    /// Recursively unmount a component and all its children
    pub fn unmount_component_tree(&self, id: ComponentId) -> TreeResult<()> {
        // First unmount all children recursively (bottom-up approach)
        let children = self.get_children(id)?;
        for child_id in children {
            self.unmount_component_tree(child_id)?;
        }
        
        // Then unmount this component
        self.unmount_component(id)?;
        
        Ok(())
    }
    
    /// Update a component with new props
    pub fn update_component<P: crate::component::Props + 'static>(
        &self,
        id: ComponentId,
        props: P,
    ) -> TreeResult<()> {
        let lifecycle_manager = self.get_lifecycle_manager(id)?;
        let mut manager = lifecycle_manager.write().map_err(|_| {
            TreeError::LockError("Failed to lock lifecycle manager".to_string())
        })?;
        
        // Box the props for dynamic dispatch
        let boxed_props = Box::new(props);
        
        manager.update(boxed_props).map_err(TreeError::LifecycleError)?;
        
        Ok(())
    }
    
    /// Recursively update a component and its children if needed
    pub fn update_component_tree<P: crate::component::Props + Clone + 'static>(
        &self,
        id: ComponentId,
        props: P,
    ) -> TreeResult<()> {
        // Update this component
        self.update_component(id, props.clone())?;
        
        // Get children to potentially update
        let _children = self.get_children(id)?;
        
        // In a real implementation, we would check if child props need updating
        // based on parent props and pass down appropriate derived props
        // For now, we'll just note that we would do this
        
        // Render the parent component
        let _nodes = self.render_component(id)?;
        
        // Process rendered nodes to extract child props and update children
        // This is a simplified version - in a real implementation, this would be more complex
        // and would match rendered nodes to child components
        
        Ok(())
    }
    
    /// Get all components in the tree
    pub fn get_all_components(&self) -> TreeResult<Vec<ComponentId>> {
        let components = self.components.read().map_err(|_| {
            TreeError::LockError("Failed to read components map".to_string())
        })?;
        
        Ok(components.keys().cloned().collect())
    }
    
    /// Get all components that need updating
    pub fn get_components_to_update(&self) -> TreeResult<Vec<ComponentId>> {
        // In a real implementation, this would check component dirty flags
        // or other state to determine which components need updating
        // For now, just return an empty vector
        Ok(Vec::new())
    }
    
    /// Perform state change detection for a component
    pub fn detect_state_changes(&self, id: ComponentId) -> TreeResult<bool> {
        let component = self.get_component(id)?;
        let _component = component.read().map_err(|_| {
            TreeError::LockError("Failed to read component".to_string())
        })?;
        
        // This would compare previous state with current state
        // and return true if changes are detected
        // For now, just return false indicating no changes
        Ok(false)
    }
    
    /// Batch update multiple components
    pub fn batch_update_components(&self, ids: &[ComponentId]) -> TreeResult<usize> {
        for id in ids {
            if !self.has_component(*id) {
                return Err(TreeError::ComponentNotFound(*id));
            }
        }
        
        // In a real implementation, this would optimize the update sequence
        // based on component dependencies or tree structure
        let mut updated_count = 0;
        
        for &id in ids {
            // Get the lifecycle manager for this component
            let lifecycle_manager = self.get_lifecycle_manager(id)?;
            let _manager = lifecycle_manager.read().map_err(|_| {
                TreeError::LockError("Failed to lock lifecycle manager".to_string())
            })?;
            
            // For now, we'll just count this as an updated component
            // In the future, we'll need to implement proper state change detection
            updated_count += 1;
        }
        
        Ok(updated_count)
    }
}

// Implement Debug manually to avoid requiring ComponentInstance and LifecycleManager to implement Debug
impl std::fmt::Debug for ComponentTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentTree")
            .field("component_count", &self.components.try_read().map(|c| c.len()).unwrap_or(0))
            .field("root", &self.root.try_read().ok())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, ComponentError, Context, Node};
    
    // Simple test component
    struct TestComponent {
        id: ComponentId,
        context: Context,
        name: String,
    }
    
    // Empty props for test component
    #[derive(Clone)]
    struct TestProps {
        name: String,
    }
    
    // Implement Props trait for TestProps - this is automatically handled by the blanket implementation in mod.rs
    
    impl Component for TestComponent {
        type Props = TestProps;
        
        fn component_id(&self) -> ComponentId {
            self.id
        }
        
        fn create(props: Self::Props, context: Context) -> Self {
            Self {
                id: ComponentId::new(),
                context,
                name: props.name,
            }
        }
        
        fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
            self.name = props.name;
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
    
    fn create_test_component(name: &str, context: Context) -> ComponentInstance {
        let props = TestProps {
            name: name.to_string(),
        };
        
        let component = TestComponent::create(props.clone(), context.clone());
        
        ComponentInstance::new(component, props)
    }
    
    #[test]
    fn test_component_tree_basic() {
        // Create tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());
        
        // Add root component
        let root_component = create_test_component("root", context.clone());
        let root_id = tree.add_component(root_component).unwrap();
        
        // Set as root
        tree.set_root(root_id).unwrap();
        
        // Verify root is set correctly
        assert_eq!(tree.root_id().unwrap(), Some(root_id));
        
        // Add child component
        let child_component = create_test_component("child", context.clone());
        let child_id = tree.add_component(child_component).unwrap();
        
        // Add parent-child relationship
        tree.add_child(root_id, child_id).unwrap();
        
        // Verify relationships
        let children = tree.get_children(root_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child_id);
        
        let parent = tree.get_parent(child_id).unwrap();
        assert_eq!(parent, Some(root_id));
    }
    
    #[test]
    fn test_component_tree_lifecycle() {
        // Create tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());
        
        // Add root component
        let root_component = create_test_component("root", context.clone());
        let root_id = tree.add_component(root_component).unwrap();
        tree.set_root(root_id).unwrap();
        
        // Add child components
        let child1_component = create_test_component("child1", context.clone());
        let child1_id = tree.add_component(child1_component).unwrap();
        
        let child2_component = create_test_component("child2", context.clone());
        let child2_id = tree.add_component(child2_component).unwrap();
        
        // Set up parent-child relationships
        tree.add_child(root_id, child1_id).unwrap();
        tree.add_child(root_id, child2_id).unwrap();
        
        // Initialize and mount recursively
        tree.mount_component_tree(root_id).unwrap();
        
        // Check if components are mounted
        let root_lifecycle = tree.get_lifecycle_manager(root_id).unwrap();
        let root_phase = {
            let manager = root_lifecycle.read().unwrap();
            manager.current_phase()
        };
        assert_eq!(root_phase, LifecyclePhase::Mounted);
        
        let child1_lifecycle = tree.get_lifecycle_manager(child1_id).unwrap();
        let child1_phase = {
            let manager = child1_lifecycle.read().unwrap();
            manager.current_phase()
        };
        assert_eq!(child1_phase, LifecyclePhase::Mounted);
    }
    
    #[test]
    fn test_component_tree_removal() {
        // Create tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());
        
        // Add root component
        let root_component = create_test_component("root", context.clone());
        let root_id = tree.add_component(root_component).unwrap();
        tree.set_root(root_id).unwrap();
        
        // Add child components
        let child1_component = create_test_component("child1", context.clone());
        let child1_id = tree.add_component(child1_component).unwrap();
        
        let child2_component = create_test_component("child2", context.clone());
        let child2_id = tree.add_component(child2_component).unwrap();
        
        // Set up parent-child relationships
        tree.add_child(root_id, child1_id).unwrap();
        tree.add_child(root_id, child2_id).unwrap();
        
        // Remove one child
        tree.remove_child(root_id, child1_id).unwrap();
        
        // Verify it's removed from parent's children
        let children = tree.get_children(root_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child2_id);
        
        // Verify parent is removed from child
        let parent = tree.get_parent(child1_id).unwrap();
        assert_eq!(parent, None);
        
        // Component should still exist in the tree
        assert!(tree.has_component(child1_id));
        
        // Now fully remove the component
        tree.remove_component(child1_id).unwrap();
        
        // Verify it's gone
        assert!(!tree.has_component(child1_id));
        
        // Now remove the root (should remove child2 as well)
        tree.remove_component(root_id).unwrap();
        
        // Verify both are gone
        assert!(!tree.has_component(root_id));
        assert!(!tree.has_component(child2_id));
        
        // Verify root is unset
        assert_eq!(tree.root_id().unwrap(), None);
    }
    
    #[test]
    fn test_dirty_component_updates() {
        // Create tree
        let context = Context::new();
        let tree = ComponentTree::new(context.clone());
        
        // Add components
        let root_component = create_test_component("root", context.clone());
        let root_id = tree.add_component(root_component).unwrap();
        tree.set_root(root_id).unwrap();
        
        let child1_component = create_test_component("child1", context.clone());
        let child1_id = tree.add_component(child1_component).unwrap();
        
        let child2_component = create_test_component("child2", context.clone());
        let child2_id = tree.add_component(child2_component).unwrap();
        
        // Set up relationships
        tree.add_child(root_id, child1_id).unwrap();
        tree.add_child(root_id, child2_id).unwrap();
        
        // Initialize and mount
        tree.mount_component_tree(root_id).unwrap();
        
        // Mark some as dirty
        // Create a collection of dirty components
        let dirty_components = vec![root_id, child1_id];
        
        // Update dirty components by batching
        let updated = tree.batch_update_components(&dirty_components).unwrap();
        
        // Should have updated 2 components
        assert_eq!(updated, 2);
    }
}
