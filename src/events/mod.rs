//! Enhanced event system for Orbit UI framework
//!
//! The event system provides:
//! - Generic event trait with downcasting support
//! - Event emitter for general event handling with type erasure
//! - Dispatcher for strongly-typed event handling
//! - Event delegation for component event propagation
//! - Layout-aware hit testing for precise event targeting
//! - Component ID integration for efficient event routing

pub mod delegation;
pub mod dispatcher;
pub mod emitter;
pub mod event;
pub mod hit_testing;

pub use delegation::*;
pub use dispatcher::Dispatcher;
pub use emitter::EventEmitter;
pub use event::Event;
pub use hit_testing::*;

use crate::{
    component::ComponentId,
    layout::{Point, LayoutNode},
};

/// Enhanced event system that integrates with layout and components
#[derive(Debug)]
pub struct EventSystem {
    /// Hit testing engine for layout-aware event targeting
    hit_tester: HitTester,
    /// Event delegation system
    delegator: EventDelegate,
}

impl EventSystem {
    /// Create a new event system
    pub fn new() -> Self {
        Self {
            hit_tester: HitTester::new(),
            delegator: EventDelegate::new(None),
        }
    }

    /// Process a pointer event (mouse, touch) with layout hit testing
    pub fn process_pointer_event<E: Event + Clone>(
        &mut self,
        event: E,
        position: Point,
        layout_root: &LayoutNode,
    ) -> Result<Vec<ComponentId>, EventError> {
        // Perform hit testing to find target components
        let hit_targets = self.hit_tester.hit_test(position, layout_root)?;

        // Create a delegated event for each hit target
        let mut processed_targets = Vec::new();

        for target_id in hit_targets {
            let delegated_event = DelegatedEvent::new(
                event.clone(),
                PropagationPhase::Target,
            );

            // Process the event through delegation
            if self.delegator.dispatch_event(delegated_event).is_ok() {
                processed_targets.push(target_id);
            }
        }

        Ok(processed_targets)
    }

    /// Get mutable reference to the hit tester for configuration
    pub fn hit_tester_mut(&mut self) -> &mut HitTester {
        &mut self.hit_tester
    }

    /// Get reference to the event delegator
    pub fn delegator(&self) -> &EventDelegate {
        &self.delegator
    }

    /// Get mutable reference to the event delegator
    pub fn delegator_mut(&mut self) -> &mut EventDelegate {
        &mut self.delegator
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in the event system
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Hit testing failed: {0}")]
    HitTestingFailed(String),

    #[error("Event delegation failed: {0}")]
    DelegationFailed(String),

    #[error("Component not found: {0:?}")]
    ComponentNotFound(ComponentId),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
}
