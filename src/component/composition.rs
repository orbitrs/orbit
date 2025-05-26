//! Component composition patterns for Orbit UI framework
//!
//! This module provides utilities for composing components together in various patterns
//! including render props, slots, and compound components.

use std::collections::HashMap;

use crate::component::{Component, ComponentError, ComponentId, Context, Node};

/// Trait for components that can render other components (render props pattern)
pub trait RenderProp<T> {
    /// Render function that takes data and returns nodes
    fn render(&self, data: T) -> Result<Vec<Node>, ComponentError>;
}

/// Implementation of render prop for function pointers
impl<T, F> RenderProp<T> for F
where
    F: Fn(T) -> Result<Vec<Node>, ComponentError>,
{
    fn render(&self, data: T) -> Result<Vec<Node>, ComponentError> {
        self(data)
    }
}

/// A component that uses render props pattern
pub struct RenderPropComponent<T, R>
where
    R: RenderProp<T>,
{
    component_id: ComponentId,
    data: T,
    renderer: R,
    #[allow(dead_code)]
    context: Context,
}

impl<T, R> RenderPropComponent<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: RenderProp<T> + Send + Sync + 'static,
{
    pub fn new(data: T, renderer: R, context: Context) -> Self {
        Self {
            component_id: ComponentId::new(),
            data,
            renderer,
            context,
        }
    }
}

/// Props for render prop component
#[derive(Clone)]
pub struct RenderPropProps<T, R>
where
    T: Clone,
    R: Clone,
{
    pub data: T,
    pub renderer: R,
}

// Note: Props implementation is handled by the generic impl in mod.rs

impl<T, R> Component for RenderPropComponent<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: RenderProp<T> + Clone + Send + Sync + 'static,
{
    type Props = RenderPropProps<T, R>;

    fn component_id(&self) -> ComponentId {
        self.component_id
    }

    fn create(props: Self::Props, context: Context) -> Self {
        Self::new(props.data, props.renderer, context)
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.data = props.data;
        self.renderer = props.renderer;
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        self.renderer.render(self.data.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Slot-based composition system
#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub nodes: Vec<Node>,
    pub required: bool,
}

impl Slot {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            required: false,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_nodes(mut self, nodes: Vec<Node>) -> Self {
        self.nodes = nodes;
        self
    }
}

/// Props for slotted components
#[derive(Clone)]
pub struct SlottedProps {
    pub slots: HashMap<String, Slot>,
}

impl SlottedProps {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn with_slot(mut self, slot: Slot) -> Self {
        self.slots.insert(slot.name.clone(), slot);
        self
    }

    pub fn get_slot(&self, name: &str) -> Option<&Slot> {
        self.slots.get(name)
    }
    pub fn get_slot_nodes(&self, name: &str) -> Vec<Node> {
        self.slots
            .get(name)
            .map(|slot| slot.nodes.clone())
            .unwrap_or_default()
    }
}

impl Default for SlottedProps {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for components that support slots
pub trait Slotted {
    /// Get the slots this component supports
    fn supported_slots(&self) -> Vec<String>;

    /// Validate that all required slots are provided
    fn validate_slots(&self, props: &SlottedProps) -> Result<(), ComponentError> {
        for slot_name in self.supported_slots() {
            if let Some(slot) = props.get_slot(&slot_name) {
                if slot.required && slot.nodes.is_empty() {
                    return Err(ComponentError::InvalidProps(format!(
                        "Required slot '{slot_name}' is empty"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Render with slots
    fn render_with_slots(&self, props: &SlottedProps) -> Result<Vec<Node>, ComponentError>;
}

/// A basic slotted component implementation
pub struct SlottedComponent {
    component_id: ComponentId,
    supported_slots: Vec<String>,
    #[allow(dead_code)]
    context: Context,
}

impl SlottedComponent {
    pub fn new(supported_slots: Vec<String>, context: Context) -> Self {
        Self {
            component_id: ComponentId::new(),
            supported_slots,
            context,
        }
    }
}

impl Component for SlottedComponent {
    type Props = SlottedProps;

    fn component_id(&self) -> ComponentId {
        self.component_id
    }

    fn create(props: Self::Props, context: Context) -> Self {
        // Extract supported slots from props
        let supported_slots = props.slots.keys().cloned().collect();
        Self::new(supported_slots, context)
    }

    fn update(&mut self, _props: Self::Props) -> Result<(), ComponentError> {
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // Default implementation - override for custom behavior
        Ok(Vec::new())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Slotted for SlottedComponent {
    fn supported_slots(&self) -> Vec<String> {
        self.supported_slots.clone()
    }

    fn render_with_slots(&self, props: &SlottedProps) -> Result<Vec<Node>, ComponentError> {
        self.validate_slots(props)?;

        let mut nodes = Vec::new();
        for slot_name in &self.supported_slots {
            nodes.extend(props.get_slot_nodes(slot_name));
        }
        Ok(nodes)
    }
}

/// Compound component pattern - a component made of multiple sub-components
pub trait CompoundComponent {
    type SubComponents;

    /// Get the sub-components of this compound component
    fn sub_components(&self) -> &Self::SubComponents;

    /// Get a mutable reference to sub-components
    fn sub_components_mut(&mut self) -> &mut Self::SubComponents;

    /// Render all sub-components in order
    fn render_compound(&self) -> Result<Vec<Node>, ComponentError>;
}

/// A flexible compound component implementation
pub struct FlexibleCompoundComponent {
    component_id: ComponentId,
    sub_components: Vec<Box<dyn Component<Props = FlexibleCompoundProps>>>,
    #[allow(dead_code)]
    context: Context,
}

#[derive(Clone)]
pub struct FlexibleCompoundProps {
    pub children: Vec<FlexibleCompoundProps>,
}

impl FlexibleCompoundComponent {
    pub fn new(context: Context) -> Self {
        Self {
            component_id: ComponentId::new(),
            sub_components: Vec::new(),
            context,
        }
    }

    pub fn add_sub_component(
        &mut self,
        component: Box<dyn Component<Props = FlexibleCompoundProps>>,
    ) {
        self.sub_components.push(component);
    }
}

impl Component for FlexibleCompoundComponent {
    type Props = FlexibleCompoundProps;

    fn component_id(&self) -> ComponentId {
        self.component_id
    }
    fn create(_props: Self::Props, context: Context) -> Self {
        // Create compound component with sub-components based on props

        // Add logic here to create sub-components from props
        Self::new(context)
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        // Update all sub-components
        for (i, child_props) in props.children.iter().enumerate() {
            if let Some(sub_component) = self.sub_components.get_mut(i) {
                sub_component.update(child_props.clone())?;
            }
        }
        Ok(())
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        let mut all_nodes = Vec::new();
        for sub_component in &self.sub_components {
            all_nodes.extend(sub_component.render()?);
        }
        Ok(all_nodes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CompoundComponent for FlexibleCompoundComponent {
    type SubComponents = Vec<Box<dyn Component<Props = FlexibleCompoundProps>>>;

    fn sub_components(&self) -> &Self::SubComponents {
        &self.sub_components
    }

    fn sub_components_mut(&mut self) -> &mut Self::SubComponents {
        &mut self.sub_components
    }

    fn render_compound(&self) -> Result<Vec<Node>, ComponentError> {
        self.render()
    }
}

/// Composition utilities and builder patterns
///
/// Builder for creating complex component compositions
pub struct CompositionBuilder {
    components: Vec<Box<dyn Component<Props = FlexibleCompoundProps>>>,
    context: Context,
}

impl CompositionBuilder {
    pub fn new(context: Context) -> Self {
        Self {
            components: Vec::new(),
            context,
        }
    }

    pub fn add_component(
        mut self,
        component: Box<dyn Component<Props = FlexibleCompoundProps>>,
    ) -> Self {
        self.components.push(component);
        self
    }

    pub fn build(self) -> FlexibleCompoundComponent {
        let mut compound = FlexibleCompoundComponent::new(self.context);
        for component in self.components {
            compound.add_sub_component(component);
        }
        compound
    }
}

/// Macros for easier composition
///
/// Create a slot with nodes
#[macro_export]
macro_rules! slot {
    ($name:expr) => {
        Slot::new($name)
    };
    ($name:expr, required) => {
        Slot::new($name).required()
    };
    ($name:expr, $($node:expr),*) => {
        Slot::new($name).with_nodes(vec![$($node),*])
    };
}

/// Create slotted props easily
#[macro_export]
macro_rules! slotted_props {
    ($($slot:expr),*) => {
        {
            let mut props = SlottedProps::new();
            $(
                props = props.with_slot($slot);
            )*
            props
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentBase;

    struct TestComponent {
        base: ComponentBase,
    }

    #[derive(Clone)]
    struct TestProps;
    // Using the blanket implementation of Props instead of implementing it manually

    impl Component for TestComponent {
        type Props = TestProps;

        fn component_id(&self) -> ComponentId {
            self.base.id()
        }

        fn create(_props: Self::Props, context: Context) -> Self {
            Self {
                base: ComponentBase::new(context),
            }
        }

        fn update(&mut self, _props: Self::Props) -> Result<(), ComponentError> {
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

    #[test]
    fn test_slot_creation() {
        let slot = slot!("header");
        assert_eq!(slot.name, "header");
        assert!(!slot.required);

        let required_slot = slot!("content", required);
        assert!(required_slot.required);
    }

    #[test]
    fn test_slotted_props() {
        let props = slotted_props!(slot!("header"), slot!("content", required));

        assert!(props.get_slot("header").is_some());
        assert!(props.get_slot("content").is_some());
        assert!(props.get_slot("footer").is_none());
    }

    #[test]
    fn test_slotted_component() {
        let context = Context::new();
        let slots = vec!["header".to_string(), "content".to_string()];
        let component = SlottedComponent::new(slots, context);

        assert_eq!(component.supported_slots(), vec!["header", "content"]);
    }

    #[test]
    fn test_composition_builder() {
        let context = Context::new();
        let builder = CompositionBuilder::new(context);
        let compound = builder.build();

        assert_eq!(compound.sub_components().len(), 0);
    }
}
