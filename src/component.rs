// Component model for the Orbit UI framework

/// Trait for all UI components in Orbit
pub trait Component {
    /// The type of the component's properties
    type Props;

    /// Create a new component with the given properties
    fn new(props: Self::Props) -> Self;

    /// Render the component and return the HTML-like markup
    fn render(&self) -> String;

    /// Called when the component is mounted
    fn mounted(&mut self) {}

    /// Called when the component is updated
    fn updated(&mut self) {}

    /// Called when the component is unmounted
    fn unmounted(&mut self) {}
}

/// Marker trait for component properties
pub trait Props {}

/// Lifecycle phases of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Component is being created
    Creating,
    /// Component has been mounted to the DOM/view
    Mounted,
    /// Component is being updated
    Updating,
    /// Component is being unmounted
    Unmounting,
}

/// Basic implementation of a component registry
pub struct ComponentRegistry {
    // Maps component names to their factory functions
    components: std::collections::HashMap<String, Box<dyn Fn() -> Box<dyn std::any::Any>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    /// Register a component with the given name
    pub fn register<T: Component + 'static>(&mut self, name: &str, factory: fn() -> T) {
        self.components.insert(
            name.to_string(),
            Box::new(move || Box::new(factory()) as Box<dyn std::any::Any>),
        );
    }

    /// Get a component by name
    pub fn get(&self, name: &str) -> Option<Box<dyn std::any::Any>> {
        self.components.get(name).map(|factory| factory())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
