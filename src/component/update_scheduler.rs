//! Update scheduling for component rendering
//!
//! This module provides a batch update scheduler for components to avoid
//! unnecessary re-renders and optimize performance.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::component::ComponentId;

/// Priority level for component updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdatePriority {
    /// Low priority updates can be delayed if higher-priority updates are pending
    Low = 0,
    /// Normal priority updates are processed in order
    Normal = 1,
    /// High priority updates are processed before normal updates
    High = 2,
    /// Critical updates are processed immediately
    Critical = 3,
}

/// A scheduled update for a component
#[derive(Debug)]
struct ScheduledUpdate {
    /// Component ID to update
    component_id: ComponentId,
    /// Priority of this update
    priority: UpdatePriority,
}

/// Scheduler for batching component updates
///
/// This system allows components to request updates (re-renders) and batches
/// them together for optimal performance. Updates can be prioritized to ensure
/// critical UI elements are updated first.
#[derive(Debug, Clone)]
pub struct UpdateScheduler {
    /// Queue of updates to process
    updates: Arc<Mutex<VecDeque<ScheduledUpdate>>>,
    /// Set of components with pending updates (to avoid duplicates)
    pending: Arc<Mutex<HashSet<ComponentId>>>,
    /// Whether an update cycle is currently in progress
    updating: Arc<Mutex<bool>>,
}

impl Default for UpdateScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateScheduler {
    /// Create a new update scheduler
    pub fn new() -> Self {
        Self {
            updates: Arc::new(Mutex::new(VecDeque::new())),
            pending: Arc::new(Mutex::new(HashSet::new())),
            updating: Arc::new(Mutex::new(false)),
        }
    }

    /// Schedule a component for update with the given priority
    pub fn schedule_update(
        &self,
        component_id: ComponentId,
        priority: UpdatePriority,
    ) -> Result<(), String> {
        // First, check if this component is already scheduled
        let mut pending = match self.pending.lock() {
            Ok(pending) => pending,
            Err(_) => return Err("Failed to lock pending set".to_string()),
        };

        // If already scheduled, skip
        if pending.contains(&component_id) {
            return Ok(());
        }

        // Add to pending set
        pending.insert(component_id);
        drop(pending); // Release lock early

        // Add to update queue with priority
        let mut updates = match self.updates.lock() {
            Ok(updates) => updates,
            Err(_) => return Err("Failed to lock update queue".to_string()),
        };

        let update = ScheduledUpdate {
            component_id,
            priority,
        };

        // Insert based on priority (higher priority items go to the front)
        match priority {
            UpdatePriority::Critical => {
                // Critical updates go to the front
                updates.push_front(update);
            }
            UpdatePriority::High => {
                // High priority updates go after critical but before normal
                let insert_pos = updates
                    .iter()
                    .position(|u| u.priority < UpdatePriority::High)
                    .unwrap_or(updates.len());
                updates.insert(insert_pos, update);
            }
            _ => {
                // Normal and low priority go at the end
                updates.push_back(update);
            }
        }

        // If not currently in an update cycle, trigger one
        let updating = self
            .updating
            .lock()
            .map_err(|_| "Failed to lock updating flag".to_string())?;
        if !*updating {
            // In a real implementation, this would schedule the update cycle
            // through the application's event loop or rendering system
            // But for now, we just note that we should update
            drop(updating); // Release lock

            // Trigger process_updates here, or signal the main loop to do so
            // For now, this is a synchronous call in the testing environment
        }

        Ok(())
    }

    /// Process all pending updates
    ///
    /// Returns the number of components updated
    pub fn process_updates<F>(&self, mut update_component: F) -> Result<usize, String>
    where
        F: FnMut(ComponentId) -> Result<(), String>,
    {
        // Set updating flag to prevent multiple concurrent update passes
        {
            let mut updating = self
                .updating
                .lock()
                .map_err(|_| "Failed to lock updating flag".to_string())?;
            *updating = true;
        }

        let mut count = 0;

        // Process all updates in the queue
        loop {
            // Get the next update
            let update = {
                let mut updates = self
                    .updates
                    .lock()
                    .map_err(|_| "Failed to lock update queue".to_string())?;
                updates.pop_front()
            };

            // If no more updates, break
            let update = match update {
                Some(update) => update,
                None => break,
            };

            // Remove from pending set
            {
                let mut pending = self
                    .pending
                    .lock()
                    .map_err(|_| "Failed to lock pending set".to_string())?;
                pending.remove(&update.component_id);
            }

            // Call the update function for this component
            if let Err(e) = update_component(update.component_id) {
                // Log error but continue with other updates
                eprintln!(
                    "Error updating component {}: {}",
                    update.component_id.id(),
                    e
                );
            }

            count += 1;
        }

        // Reset updating flag
        {
            let mut updating = self
                .updating
                .lock()
                .map_err(|_| "Failed to lock updating flag".to_string())?;
            *updating = false;
        }

        Ok(count)
    }

    /// Check if there are any pending updates
    pub fn has_pending_updates(&self) -> Result<bool, String> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| "Failed to lock pending set".to_string())?;
        Ok(!pending.is_empty())
    }

    /// Get the number of pending updates
    pub fn pending_update_count(&self) -> Result<usize, String> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| "Failed to lock pending set".to_string())?;
        Ok(pending.len())
    }

    /// Clear all pending updates
    pub fn clear_updates(&self) -> Result<(), String> {
        {
            let mut updates = self
                .updates
                .lock()
                .map_err(|_| "Failed to lock update queue".to_string())?;
            updates.clear();
        }

        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| "Failed to lock pending set".to_string())?;
            pending.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_scheduler_basic() {
        let scheduler = UpdateScheduler::new();

        // Schedule some updates
        let c1 = ComponentId::new();
        let c2 = ComponentId::new();
        let c3 = ComponentId::new();

        scheduler
            .schedule_update(c1, UpdatePriority::Normal)
            .unwrap();
        scheduler.schedule_update(c2, UpdatePriority::High).unwrap();
        scheduler.schedule_update(c3, UpdatePriority::Low).unwrap();

        // Check we have the right number of pending updates
        assert_eq!(scheduler.pending_update_count().unwrap(), 3);

        // Process updates and verify correct processing order
        let mut processed = Vec::new();
        scheduler
            .process_updates(|id| {
                processed.push(id);
                Ok(())
            })
            .unwrap();

        assert_eq!(processed.len(), 3);
        // High priority should be first
        assert_eq!(processed[0], c2);
        // Followed by normal priority
        assert_eq!(processed[1], c1);
        // Then low priority
        assert_eq!(processed[2], c3);

        // All updates should be processed
        assert_eq!(scheduler.pending_update_count().unwrap(), 0);
    }

    #[test]
    fn test_duplicate_updates() {
        let scheduler = UpdateScheduler::new();

        let c1 = ComponentId::new();

        // Schedule the same update multiple times
        scheduler
            .schedule_update(c1, UpdatePriority::Normal)
            .unwrap();
        scheduler
            .schedule_update(c1, UpdatePriority::Normal)
            .unwrap();
        scheduler
            .schedule_update(c1, UpdatePriority::Normal)
            .unwrap();

        // Should only be one pending update
        assert_eq!(scheduler.pending_update_count().unwrap(), 1);

        let mut processed = Vec::new();
        scheduler
            .process_updates(|id| {
                processed.push(id);
                Ok(())
            })
            .unwrap();

        // Should only process once
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0], c1);
    }

    #[test]
    fn test_priority_ordering() {
        let scheduler = UpdateScheduler::new();

        // Create various components with different priorities
        let c_normal = ComponentId::new();
        let c_low = ComponentId::new();
        let c_high = ComponentId::new();
        let c_critical = ComponentId::new();

        // Schedule in a different order than priority
        scheduler
            .schedule_update(c_normal, UpdatePriority::Normal)
            .unwrap();
        scheduler
            .schedule_update(c_low, UpdatePriority::Low)
            .unwrap();
        scheduler
            .schedule_update(c_high, UpdatePriority::High)
            .unwrap();
        scheduler
            .schedule_update(c_critical, UpdatePriority::Critical)
            .unwrap();

        // Process and check priority ordering
        let mut processed = Vec::new();
        scheduler
            .process_updates(|id| {
                processed.push(id);
                Ok(())
            })
            .unwrap();

        assert_eq!(processed.len(), 4);
        // Critical should be first
        assert_eq!(processed[0], c_critical);
        // High priority next
        assert_eq!(processed[1], c_high);
        // Normal priority
        assert_eq!(processed[2], c_normal);
        // Low priority last
        assert_eq!(processed[3], c_low);
    }

    #[test]
    fn test_clear_updates() {
        let scheduler = UpdateScheduler::new();

        let c1 = ComponentId::new();
        let c2 = ComponentId::new();

        scheduler
            .schedule_update(c1, UpdatePriority::Normal)
            .unwrap();
        scheduler
            .schedule_update(c2, UpdatePriority::Normal)
            .unwrap();

        assert_eq!(scheduler.pending_update_count().unwrap(), 2);

        // Clear all updates
        scheduler.clear_updates().unwrap();

        assert_eq!(scheduler.pending_update_count().unwrap(), 0);

        let mut processed = Vec::new();
        scheduler
            .process_updates(|id| {
                processed.push(id);
                Ok(())
            })
            .unwrap();

        // Nothing should be processed
        assert_eq!(processed.len(), 0);
    }
}
