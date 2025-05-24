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
    }

    /// Initialize the component (post-creation)
    pub fn initialize(&mut self) -> Result<(), ComponentError> {
        if self.phase != LifecyclePhase::Created {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "initialize".to_string(),
            ));
        }

        let result = if let Ok(mut component) = self.component.lock() {
            component.initialize()
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
        self.context.set_lifecycle_phase(LifecyclePhase::Mounting);

        // Execute mount
        let result = {
            let mut component = self.component.lock().map_err(|_| {
                ComponentError::LockError("Failed to lock component for mount".to_string())
            })?;

            // Call mount on the component
            component.mount()?;

            // Execute lifecycle hooks after successful mount
            {
                let mut instance = component.instance.lock().map_err(|_| {
                    ComponentError::LockError(
                        "Failed to lock component instance for mount".to_string(),
                    )
                })?;

                // Execute mounted hooks
                self.context
                    .execute_lifecycle_hooks(LifecyclePhase::Mounted, &mut *instance);
            }

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
            }

            // Update the component with new props
            component.update_boxed(props)
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

        // Execute before unmount hooks
        if let Ok(mut component) = self.component.lock() {
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::BeforeUnmount, &mut *component);
            component.before_unmount()?;
        } else {
            return Err(ComponentError::LockError(
                "Failed to lock component instance for before_unmount".to_string(),
            ));
        }

        // Unmounting phase
        self.phase = LifecyclePhase::Unmounting;
        self.context.set_lifecycle_phase(LifecyclePhase::Unmounting);

        let unmount_result = if let Ok(mut component) = self.component.lock() {
            // Execute unmount hooks
            self.context
                .execute_lifecycle_hooks(LifecyclePhase::Unmounting, &mut *component);
            component.unmount()
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
    }

    /// Render the component
    pub fn render(&self) -> Result<Vec<crate::component::Node>, ComponentError> {
        if self.phase != LifecyclePhase::Mounted {
            return Err(ComponentError::InvalidLifecycleTransition(
                self.phase,
                "render".to_string(),
            ));
        }

        if let Ok(component) = self.component.lock() {
            component.render()
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
