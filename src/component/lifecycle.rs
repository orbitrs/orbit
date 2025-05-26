//! Component lifecycle management for Orbit UI framework

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::component::{
    ComponentError, ComponentInstance, Context, LifecyclePhase, 
    UnmountContext, UnmountReason, state_tracking::{
        StateTracker, StateValue
    }
};

/// Manages the lifecycle of components
pub struct LifecycleManager {
    /// Current phase of the component
    phase: LifecyclePhase,

    /// Component instance being managed
    component: Arc<Mutex<ComponentInstance>>,

    /// Context for the component
    context: Context,
    
    /// State change tracker
    state_tracker: StateTracker,
    
    /// Last time this component was updated
    last_updated: std::time::Instant,
}

impl LifecycleManager {
    /// Create a new lifecycle manager for a component
    pub fn new(component: ComponentInstance, context: Context) -> Self {
        let component_id = component.id();
        Self {
            phase: LifecyclePhase::Created,
            component: Arc::new(Mutex::new(component)),
            context: context.clone(),
            state_tracker: StateTracker::new_default(component_id),
            last_updated: std::time::Instant::now(),
        }
    }

    /// Get the current lifecycle phase
    pub fn current_phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Get a reference to the component instance
    pub fn get_component(&self) -> &Arc<Mutex<ComponentInstance>> {
        &self.component
    }
    /// Initialize the component (post-creation)
    pub fn initialize(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Created {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "initialize".to_string(),
            ));
        }
        let result = if let Ok(component_instance) = self.component.lock() {
            if let Ok(mut inner_component) = component_instance.instance.lock() {
                // Call the component's initialize method through AnyComponent
                inner_component.any_initialize()
            } else {
                Err(ComponentError::LockError(
                    "Failed to lock inner component for initialization".to_string(),
                ))
            }
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance for initialization".to_string(),
            ))
        };

        if result.is_ok() {
            // Update context phase
            self.context.set_lifecycle_phase(LifecyclePhase::Created);
        }

        result
    }
    /// Mount the component to the tree
    pub fn mount(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Created {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "mount".to_string(),
            ));
        }

        // Create mount context
        let mount_context = crate::component::MountContext::new(
            self.component
                .lock()
                .map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component for mount context".to_string(),
                    )
                })?
                .instance
                .lock()
                .map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock inner component for mount context".to_string(),
                    )
                })?
                .component_id(),
        );

        // Before mount phase - call before_mount hook
        let before_mount_result = {
            let component_instance = self.component.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock component for before_mount".to_string())
            })?;

            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError(
                    "Failed to lock inner component for before_mount".to_string(),
                )
            })?;

            inner_component.any_before_mount()
        };

        before_mount_result?;

        // Set mounting phase
        self.phase = LifecyclePhase::Mounting;
        self.context.set_lifecycle_phase(LifecyclePhase::Mounting);

        // Execute enhanced mount with context
        let result = {
            let component_instance = self.component.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock component for mount".to_string())
            })?;

            // Get the inner component and call enhanced mount
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for mount".to_string())
            })?;

            // Execute mount hooks first
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::Mounting, &mut **inner_component);

            // Call the enhanced mount method with context
            inner_component.any_on_mount(&mount_context)?;

            // Call the basic mount method through AnyComponent for backward compatibility
            inner_component.any_mount()
        };

        // Set mounted phase after successful mount
        if result.is_ok() {
            self.phase = LifecyclePhase::Mounted;
            self.context.set_lifecycle_phase(LifecyclePhase::Mounted);

            // Call after_mount hook
            let after_mount_result = {
                let component_instance = self.component.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component for after_mount".to_string(),
                    )
                })?;

                let mut inner_component = component_instance.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock inner component for after_mount".to_string(),
                    )
                })?;

                inner_component.any_after_mount()
            };

            if let Err(e) = after_mount_result {
                // If after_mount fails, we still consider the component mounted but log the error
                eprintln!("Warning: after_mount failed for component: {e}");
            }
        }

        if result.is_err() {
            // Reset phase on error
            self.phase = LifecyclePhase::Created;
            self.context.set_lifecycle_phase(LifecyclePhase::Created);
        }

        result
    }

    /// Update the component with new props
    pub fn update(
        &mut self,
        props: Box<dyn crate::component::Props>,
    ) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "update".to_string(),
            ));
        }

        // Before update phase
        self.phase = LifecyclePhase::BeforeUpdate;
        self.context
            .set_lifecycle_phase(LifecyclePhase::BeforeUpdate);

        // Execute before update hooks and call component's before_update
        let result = {
            let mut component = self.component.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock component for update".to_string())
            })?;

            // Execute lifecycle hooks before update
            {
                let mut instance = component.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component instance for update".to_string(),
                    )
                })?;

                self.context
                    .execute_lifecycle_hooks(LifecyclePhase::BeforeUpdate, &mut **instance);

                // Call the component's before_update method with cloned props
                let props_for_before_update = props.box_clone();
                instance.any_before_update(props_for_before_update)?;
            }

            // Update the component with new props
            {
                let mut instance = component.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component instance for update".to_string(),
                    )
                })?;

                // Clone the props using the Props trait's box_clone method
                let props_for_update = props.box_clone();
                // Call the component's update method
                instance.any_update(props_for_update)?;
            }

            // Update the props in ComponentInstance
            component.props = props;

            // Call after_update
            {
                let mut instance = component.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component instance for after_update".to_string(),
                    )
                })?;

                instance.any_after_update()
            }
        };

        if result.is_ok() {
            // Update phase
            self.phase = LifecyclePhase::Mounted;
            self.context.set_lifecycle_phase(LifecyclePhase::Mounted);
        }

        result
    }

    /// Unmount the component from the tree
    pub fn unmount(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "unmount".to_string(),
            ));
        }

        // Before unmount phase
        self.phase = LifecyclePhase::BeforeUnmount;
        self.context
            .set_lifecycle_phase(LifecyclePhase::BeforeUnmount);

        // Execute before unmount hooks and call component's before_unmount
        if let Ok(component_instance) = self.component.lock() {
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError(
                    "Failed to lock inner component for before_unmount".to_string(),
                )
            })?;

            self.context
                .execute_lifecycle_hooks(LifecyclePhase::BeforeUnmount, &mut **inner_component);

            // Call the component's before_unmount method
            inner_component.any_before_unmount()?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance for before_unmount".to_string(),
            ));
        }

        // Unmounting phase
        self.phase = LifecyclePhase::Unmounting;
        self.context.set_lifecycle_phase(LifecyclePhase::Unmounting);

        // Create unmount context
        let unmount_context = UnmountContext::new(
            self.component
                .lock()
                .map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component for unmount context".to_string(),
                    )
                })?
                .instance
                .lock()
                .map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock inner component for unmount context".to_string(),
                    )
                })?
                .component_id(),
            UnmountReason::Removed,
        );

        let unmount_result = if let Ok(component_instance) = self.component.lock() {
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for unmount".to_string())
            })?;

            // Execute unmount hooks
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::Unmounting, &mut **inner_component);

            // Call the enhanced unmount method with context
            inner_component.any_on_unmount(&unmount_context)?;

            // Call the basic unmount method for backward compatibility
            inner_component.any_unmount()
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance during unmounting".to_string(),
            ))
        };

        if unmount_result.is_ok() {
            // Update phase after successful unmount
            self.phase = LifecyclePhase::Unmounted;
            self.context.set_lifecycle_phase(LifecyclePhase::Unmounted);

            // Call after_unmount hook
            let after_unmount_result = if let Ok(component_instance) = self.component.lock() {
                let mut inner_component = component_instance.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock inner component for after_unmount".to_string(),
                    )
                })?;

                inner_component.any_after_unmount()
            } else {
                Err(ComponentError::LockError(
                    "Failed to lock component instance for after_unmount".to_string(),
                ))
            };

            if let Err(e) = after_unmount_result {
                // If after_unmount fails, we still consider the component unmounted but log the error
                eprintln!("Warning: after_unmount failed for component: {e}");
            }
        }

        unmount_result
    }
    
    /// Handle updates to the component
    pub fn handle_updates(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "handle_updates".to_string(),
            ));
        }
        
        // Extract current component state into a format our tracker can use
        let current_state = self.extract_component_state()?;
        
        // Update the state tracker and get changes (if any)
        if let Some(state_changes) = self.state_tracker.update_state(current_state)? {
            // We have state changes that need processing
            if !state_changes.is_empty() {
                // Log state changes for debugging
                eprintln!(
                    "Component {} has {} state changes to process", 
                    self.state_tracker.component_id(), 
                    state_changes.len()
                );
                
                // Set to updating phase
                self.phase = LifecyclePhase::BeforeUpdate;
                self.context.set_lifecycle_phase(LifecyclePhase::BeforeUpdate);
                
                // Get access to component instance
                let component_instance = self.component.lock().map_err(|_| {
                    ComponentError::LockError("Failed to lock component for state update".to_string())
                })?;
                
                let mut inner_component = component_instance.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock inner component for state update".to_string(),
                    )
                })?;
                
                // Execute before update hooks
                self.context.execute_lifecycle_hooks(LifecyclePhase::BeforeUpdate, &mut **inner_component);
                
                // Call component's on_update method with all state changes
                inner_component.any_on_update(&state_changes)?;
                
                // Set updating phase
                self.phase = LifecyclePhase::Updating;
                self.context.set_lifecycle_phase(LifecyclePhase::Updating);
                
                // Schedule a re-render through the update scheduler
                self.context.schedule_update(self.state_tracker.component_id());
                
                // Set back to mounted phase
                self.phase = LifecyclePhase::Mounted;
                self.context.set_lifecycle_phase(LifecyclePhase::Mounted);
                
                // Update timestamp 
                self.last_updated = std::time::Instant::now();
            }
        }
        
        Ok(())
    }
    
    /// Extract the current component state as StateValue map
    fn extract_component_state(&self) -> Result<HashMap<String, StateValue>, ComponentError> {
        let mut state_fields = HashMap::new();
        
        // Attempt to lock component instance
        let component_instance = self.component.lock().map_err(|_| {
            ComponentError::LockError("Failed to lock component for state extraction".to_string())
        })?;
        
        let component = component_instance.instance.lock().map_err(|_| {
            ComponentError::LockError("Failed to lock inner component for state extraction".to_string())
        })?;
        
        // In a production implementation, we would use reflection or introspection
        // to extract all state fields from the component. Since Rust doesn't have 
        // built-in reflection, the component would need to implement a state extraction trait.
        
        // For now, we'll just extract some common state patterns:
        
        // 1. Try to extract a "state" field if the component has one
        if let Some(state_container) = component.as_any().downcast_ref::<HashMap<String, StateValue>>() {
            // If component has a state HashMap directly, use it
            for (key, value) in state_container {
                state_fields.insert(key.clone(), value.clone());
            }
        }
        
        // 2. Look for a "props" field for props tracking
        let props_type_id = component_instance.props.as_any().type_id();
        // Add a field for props type ID to track props changes
        state_fields.insert("__props_type_id".to_string(), StateValue::String(format!("{:?}", props_type_id)));
        
        // 3. Add component phase as a tracked state
        state_fields.insert("__lifecycle_phase".to_string(), 
            StateValue::String(format!("{:?}", component.lifecycle_phase())));
        
        Ok(state_fields)
    }
    
    /// Render the component
    pub fn render(&self) -> Result<Vec<crate::component::Node>, ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "render".to_string(),
            ));
        }

        if let Ok(component_instance) = self.component.lock() {
            let _inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for render".to_string())
            })?;

            // For now, return empty Vec since AnyComponent doesn't have render method
            // In a real implementation, this would delegate to the Component trait's render method
            Ok(vec![])
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance for rendering".to_string(),
            ))
        }
    }

    /// Get a reference to the component's context
    pub fn get_context(&self) -> &Context {
        &self.context
    }
}
