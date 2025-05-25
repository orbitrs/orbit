//! State change detection and tracking for Orbit components
//!
//! This module provides efficient state change detection, dirty checking,
//! and state diff computation for optimized component updates.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::component::{ComponentError, ComponentId};

/// Represents a snapshot of component state at a point in time
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Timestamp when this snapshot was taken
    pub timestamp: Instant,
    /// Hash of the state for quick comparison
    pub state_hash: u64,
    /// Detailed state fields for diff computation
    pub fields: HashMap<String, StateValue>,
}

/// Represents different types of state values that can be tracked
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<StateValue>),
    Object(HashMap<String, StateValue>),
    Null,
}

/// Represents a specific change to component state
#[derive(Debug, Clone)]
pub struct StateChange {
    /// The field that changed
    pub field_name: String,
    /// Previous value
    pub old_value: Option<StateValue>,
    /// New value
    pub new_value: StateValue,
    /// When the change occurred
    pub timestamp: Instant,
    /// Priority of this change for batching
    pub priority: ChangePriority,
}

/// Priority levels for state changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Collection of state changes for batching
#[derive(Debug, Clone)]
pub struct StateChanges {
    /// List of individual changes
    pub changes: Vec<StateChange>,
    /// When the batch was created
    pub batch_timestamp: Instant,
    /// Whether this batch requires immediate processing
    pub immediate: bool,
}

/// Tracks state changes for a component with dirty checking optimization
pub struct StateTracker {
    /// ID of the component being tracked
    component_id: ComponentId,
    /// Previous state snapshot for comparison
    previous_state: Option<StateSnapshot>,
    /// Current state snapshot
    current_state: Option<StateSnapshot>,
    /// Batched changes pending processing
    change_batch: Vec<StateChange>,
    /// Dirty flags for performance optimization
    dirty_fields: HashMap<String, bool>,
    /// Configuration for change detection
    config: StateTrackingConfig,
}

/// Configuration options for state tracking
#[derive(Debug, Clone)]
pub struct StateTrackingConfig {
    /// Maximum time to batch changes before forcing flush
    pub max_batch_time: Duration,
    /// Maximum number of changes to batch
    pub max_batch_size: usize,
    /// Whether to use deep comparison for objects/arrays
    pub deep_comparison: bool,
    /// Minimum time between state snapshots
    pub snapshot_throttle: Duration,
}

impl Default for StateTrackingConfig {
    fn default() -> Self {
        Self {
            max_batch_time: Duration::from_millis(16), // ~60fps
            max_batch_size: 50,
            deep_comparison: true,
            snapshot_throttle: Duration::from_millis(1),
        }
    }
}

impl StateSnapshot {
    /// Create a new state snapshot
    pub fn new(fields: HashMap<String, StateValue>) -> Self {
        let state_hash = Self::compute_hash(&fields);
        Self {
            timestamp: Instant::now(),
            state_hash,
            fields,
        }
    }

    /// Compute a hash of the state for quick comparison
    fn compute_hash(fields: &HashMap<String, StateValue>) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Sort keys for consistent hashing
        let mut sorted_keys: Vec<_> = fields.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            key.hash(&mut hasher);
            if let Some(value) = fields.get(key) {
                value.hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    /// Compare with another snapshot to detect changes
    pub fn diff(&self, other: &StateSnapshot) -> Vec<StateChange> {
        let mut changes = Vec::new();
        let now = Instant::now();

        // Check for modified fields
        for (field_name, new_value) in &other.fields {
            let old_value = self.fields.get(field_name);

            if old_value.map(|v| v != new_value).unwrap_or(true) {
                changes.push(StateChange {
                    field_name: field_name.clone(),
                    old_value: old_value.cloned(),
                    new_value: new_value.clone(),
                    timestamp: now,
                    priority: ChangePriority::Normal,
                });
            }
        }

        // Check for removed fields
        for (field_name, old_value) in &self.fields {
            if !other.fields.contains_key(field_name) {
                changes.push(StateChange {
                    field_name: field_name.clone(),
                    old_value: Some(old_value.clone()),
                    new_value: StateValue::Null,
                    timestamp: now,
                    priority: ChangePriority::Normal,
                });
            }
        }

        changes
    }
}

impl std::hash::Hash for StateValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            StateValue::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            StateValue::Integer(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            StateValue::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
            StateValue::Boolean(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            StateValue::Array(arr) => {
                4u8.hash(state);
                arr.hash(state);
            }
            StateValue::Object(obj) => {
                5u8.hash(state);
                let mut sorted_keys: Vec<_> = obj.keys().collect();
                sorted_keys.sort();
                for key in sorted_keys {
                    key.hash(state);
                    if let Some(value) = obj.get(key) {
                        value.hash(state);
                    }
                }
            }
            StateValue::Null => {
                6u8.hash(state);
            }
        }
    }
}

impl StateTracker {
    /// Create a new state tracker for a component
    pub fn new(component_id: ComponentId, config: StateTrackingConfig) -> Self {
        Self {
            component_id,
            previous_state: None,
            current_state: None,
            change_batch: Vec::new(),
            dirty_fields: HashMap::new(),
            config,
        }
    }

    /// Create a state tracker with default configuration
    pub fn new_default(component_id: ComponentId) -> Self {
        Self::new(component_id, StateTrackingConfig::default())
    }
    /// Update the current state and detect changes
    pub fn update_state(
        &mut self,
        new_fields: HashMap<String, StateValue>,
    ) -> Result<Option<StateChanges>, ComponentError> {
        let new_snapshot = StateSnapshot::new(new_fields);

        // Check if enough time has passed for a new snapshot
        if let Some(ref current) = self.current_state {
            if new_snapshot.timestamp.duration_since(current.timestamp)
                < self.config.snapshot_throttle
            {
                return Ok(None);
            }
        }

        // Detect changes if we have a previous state
        let changes = if let Some(ref previous) = self.current_state {
            previous.diff(&new_snapshot)
        } else {
            // First state - all fields are new
            new_snapshot
                .fields
                .iter()
                .map(|(field_name, value)| StateChange {
                    field_name: field_name.clone(),
                    old_value: None,
                    new_value: value.clone(),
                    timestamp: new_snapshot.timestamp,
                    priority: ChangePriority::Normal,
                })
                .collect()
        };

        // Update state snapshots
        self.previous_state = self.current_state.take();
        self.current_state = Some(new_snapshot); // Add changes to batch
        for change in changes {
            self.dirty_fields.insert(change.field_name.clone(), true);
            self.change_batch.push(change);
        }

        // Check if we should flush the batch
        if self.should_flush_batch() {
            Ok(Some(self.flush_batch()))
        } else {
            Ok(None)
        }
    }

    /// Check if a specific field is dirty
    pub fn is_field_dirty(&self, field_name: &str) -> bool {
        self.dirty_fields.get(field_name).copied().unwrap_or(false)
    }

    /// Mark a field as clean
    pub fn mark_field_clean(&mut self, field_name: &str) {
        self.dirty_fields.insert(field_name.to_string(), false);
    }

    /// Get all dirty fields
    pub fn get_dirty_fields(&self) -> Vec<String> {
        self.dirty_fields
            .iter()
            .filter_map(
                |(field, is_dirty)| {
                    if *is_dirty {
                        Some(field.clone())
                    } else {
                        None
                    }
                },
            )
            .collect()
    }

    /// Check if any fields are dirty
    pub fn has_dirty_fields(&self) -> bool {
        self.dirty_fields.values().any(|&dirty| dirty)
    }
    /// Force flush of current batch
    pub fn flush_batch(&mut self) -> StateChanges {
        let changes = StateChanges {
            changes: std::mem::take(&mut self.change_batch),
            batch_timestamp: Instant::now(),
            immediate: false,
        };

        // Note: Don't clear dirty flags here - they should be cleared explicitly
        // via mark_field_clean to allow fine-grained control

        changes
    }

    /// Check if batch should be flushed based on configuration
    fn should_flush_batch(&self) -> bool {
        if self.change_batch.is_empty() {
            return false;
        }

        // Check batch size limit
        if self.change_batch.len() >= self.config.max_batch_size {
            return true;
        }

        // Check time limit
        if let Some(oldest_change) = self.change_batch.first() {
            if oldest_change.timestamp.elapsed() >= self.config.max_batch_time {
                return true;
            }
        }

        // Check for critical priority changes
        self.change_batch
            .iter()
            .any(|change| change.priority == ChangePriority::Critical)
    }

    /// Get the component ID being tracked
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Get current state snapshot
    pub fn current_snapshot(&self) -> Option<&StateSnapshot> {
        self.current_state.as_ref()
    }

    /// Get previous state snapshot
    pub fn previous_snapshot(&self) -> Option<&StateSnapshot> {
        self.previous_state.as_ref()
    }

    /// Clear all tracking data
    pub fn clear(&mut self) {
        self.previous_state = None;
        self.current_state = None;
        self.change_batch.clear();
        self.dirty_fields.clear();
    }
}

impl StateChanges {
    /// Create a new state changes batch
    pub fn new(changes: Vec<StateChange>, immediate: bool) -> Self {
        Self {
            changes,
            batch_timestamp: Instant::now(),
            immediate,
        }
    }

    /// Check if this batch is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get the number of changes in this batch
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Get changes affecting a specific field
    pub fn changes_for_field(&self, field_name: &str) -> Vec<&StateChange> {
        self.changes
            .iter()
            .filter(|change| change.field_name == field_name)
            .collect()
    }

    /// Check if this batch contains critical changes
    pub fn has_critical_changes(&self) -> bool {
        self.changes
            .iter()
            .any(|change| change.priority == ChangePriority::Critical)
    }

    /// Sort changes by priority
    pub fn sort_by_priority(&mut self) {
        self.changes.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot_creation() {
        let mut fields = HashMap::new();
        fields.insert("count".to_string(), StateValue::Integer(42));
        fields.insert("name".to_string(), StateValue::String("test".to_string()));

        let snapshot = StateSnapshot::new(fields);

        assert_eq!(snapshot.fields.len(), 2);
        assert_eq!(snapshot.fields.get("count"), Some(&StateValue::Integer(42)));
        assert_eq!(
            snapshot.fields.get("name"),
            Some(&StateValue::String("test".to_string()))
        );
    }

    #[test]
    fn test_state_diff_detection() {
        let mut fields1 = HashMap::new();
        fields1.insert("count".to_string(), StateValue::Integer(1));
        let snapshot1 = StateSnapshot::new(fields1);

        let mut fields2 = HashMap::new();
        fields2.insert("count".to_string(), StateValue::Integer(2));
        let snapshot2 = StateSnapshot::new(fields2);

        let changes = snapshot1.diff(&snapshot2);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field_name, "count");
        assert_eq!(changes[0].old_value, Some(StateValue::Integer(1)));
        assert_eq!(changes[0].new_value, StateValue::Integer(2));
    }
    #[test]
    fn test_state_tracker_dirty_fields() {
        let component_id = ComponentId::new();
        let mut config = StateTrackingConfig::default();
        config.max_batch_size = 1; // Force immediate flush for testing
        let mut tracker = StateTracker::new(component_id, config);

        let mut fields = HashMap::new();
        fields.insert("count".to_string(), StateValue::Integer(1));

        let changes = tracker.update_state(fields).unwrap();

        assert!(tracker.is_field_dirty("count"));
        assert!(changes.is_some());

        tracker.mark_field_clean("count");
        assert!(!tracker.is_field_dirty("count"));
    }
    #[test]
    fn test_batch_flushing() {
        let component_id = ComponentId::new();
        let mut config = StateTrackingConfig::default();
        config.max_batch_size = 2; // Small batch size for testing
        config.snapshot_throttle = Duration::from_nanos(1); // Very small throttle for testing

        let mut tracker = StateTracker::new(component_id, config);

        // Add first change
        let mut fields1 = HashMap::new();
        fields1.insert("count".to_string(), StateValue::Integer(1));
        let changes1 = tracker.update_state(fields1).unwrap();
        assert!(changes1.is_none()); // Should not flush yet

        // Add second change - should trigger flush
        let mut fields2 = HashMap::new();
        fields2.insert("count".to_string(), StateValue::Integer(2));
        let changes2 = tracker.update_state(fields2).unwrap();
        assert!(changes2.is_some()); // Should flush now

        let changes = changes2.unwrap();
        assert_eq!(changes.changes.len(), 2);
    }
}
