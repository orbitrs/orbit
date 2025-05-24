//! Component lifecycle management for Orbit UI framework

use std::sync::{Arc, Mutex};

use crate::component::{ComponentError, ComponentInstance, Context, LifecyclePhase};

/// Manages the lifecycle of components
pub struct LifecycleManager {
    /// Current phase of the component
    phase: LifecyclePhase,

    /// Component instance being managed
    component: Arc<Mutex<ComponentInstance>>,

    /// Context for the component
    context: Context,
}

impl LifecycleManager {
    /// Create a new lifecycle manager for a component
    pub fn new(component: ComponentInstance, context: Context) -> Self {
        Self {
            phase: LifecyclePhase::Created,
            component: Arc::new(Mutex::new(component)),
            context: context.clone(),
        }
    }

    /// Get the current lifecycle phase
    pub fn current_phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Get a reference to the component instance
    pub fn get_component(&self) -> &Arc<Mutex<ComponentInstance>> {
        &self.component
    }    /// Initialize the component (post-creation)
    pub fn initialize(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Created {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "initialize".to_string(),
            ));
        }

        let result = if let Ok(component_instance) = self.component.lock() {
            if let Ok(mut inner_component) = component_instance.instance.lock() {
                // Cast to AnyComponent and call initialize through trait delegation
                if let Some(component) = inner_component.as_any_mut().downcast_mut::<dyn crate::component::Component>() {
                    component.initialize()
                } else {
                    // For components that implement Component trait, we can use AnyComponent methods
                    Ok(()) // Default initialize implementation
                }
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

        // Set mounting phase
        self.phase = LifecyclePhase::Mounting;
        self.context.set_lifecycle_phase(LifecyclePhase::Mounting);        // Execute mount
        let result = {
            let component_instance = self.component.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock component for mount".to_string())
            })?;

            // Get the inner component and call mount
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for mount".to_string())
            })?;

            // Execute mount hooks first
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::Mounting, &mut **inner_component);

            // For now, assume successful mount - we'll implement proper Component trait delegation later
            Ok(())
        };

        // Set mounted phase after successful mount
        if result.is_ok() {
            self.phase = LifecyclePhase::Mounted;
            self.context.set_lifecycle_phase(LifecyclePhase::Mounted);
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

        // Execute before update hooks
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
                    .execute_lifecycle_hooks(LifecyclePhase::BeforeUpdate, &mut *instance);
            }            // Update the component with new props
            // For now, we'll store the props in ComponentInstance and skip the actual update
            // In a real implementation, this would need proper type-safe prop delegation
            component.props = props;
            Ok(())
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
            .set_lifecycle_phase(LifecyclePhase::BeforeUnmount);        // Execute before unmount hooks
        if let Ok(component_instance) = self.component.lock() {
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for before_unmount".to_string())
            })?;
            
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::BeforeUnmount, &mut **inner_component);
            
            // Delegate to inner component's before_unmount through trait bounds
            // For now, we'll implement a generic approach since ComponentInstance wraps AnyComponent
            // In the future, we might need a more sophisticated delegation mechanism
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance for before_unmount".to_string(),
            ));
        }

        // Unmounting phase
        self.phase = LifecyclePhase::Unmounting;
        self.context.set_lifecycle_phase(LifecyclePhase::Unmounting);        let unmount_result = if let Ok(component_instance) = self.component.lock() {
            let mut inner_component = component_instance.instance.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock inner component for unmount".to_string())
            })?;
            
            // Execute unmount hooks
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::Unmounting, &mut **inner_component);
            
            // For now, return Ok since we can't call unmount on AnyComponent directly
            // In the future, this might need enhancement for proper Component trait delegation
            Ok(())
        } else {
            Err(ComponentError::LockError(
                "Failed to lock component instance during unmounting".to_string(),
            ))
        };

        if unmount_result.is_ok() {
            // Update phase after successful unmount
            self.phase = LifecyclePhase::Unmounted;
            self.context.set_lifecycle_phase(LifecyclePhase::Unmounted);
        }

        unmount_result
    }    /// Render the component
    pub fn render(&self) -> Result<Vec<crate::component::Node>, ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "render".to_string(),
            ));
        }

        if let Ok(component_instance) = self.component.lock() {
            let inner_component = component_instance.instance.lock().map_err(|_| {
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
