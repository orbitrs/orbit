// Button component for OrbitKit

use crate::component::{Component, ComponentError, Context, Node};
use std::any::Any;

/// Button size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Button style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Link,
}

/// Button component that follows Orbit's design system
///
/// # Examples
///
/// ```orbit
/// <template>
///   <Button
///     variant="primary"
///     size="medium"
///     @click="handle_click"
///   >
///     Click me
///   </Button>
/// </template>
///
/// <code lang="rust">
/// fn handle_click() {
///     log::info!("Button clicked!");
/// }
/// </code>
/// ```
#[derive(Debug)]
pub struct Button {
    /// Text content of the button
    pub text: String,
    /// Visual style variant of the button
    pub variant: ButtonVariant,
    /// Whether the button is disabled
    pub disabled: bool,
    /// Size variant of the button
    pub size: ButtonSize,
    /// Click event handler
    pub on_click: Option<fn()>,
}

/// Properties for the Button component
#[derive(Debug, Clone)]
pub struct ButtonProps {
    /// Text content of the button
    pub text: String,
    /// Visual style variant of the button
    pub variant: Option<ButtonVariant>,
    /// Whether the button is disabled
    pub disabled: Option<bool>,
    /// Size variant of the button
    pub size: Option<ButtonSize>,
    /// Click event handler
    pub on_click: Option<fn()>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            text: String::new(),
            variant: ButtonVariant::Primary,
            disabled: false,
            size: ButtonSize::Medium,
            on_click: None,
        }
    }
}

impl Component for Button {
    type Props = ButtonProps;

    fn create(props: Self::Props, _context: Context) -> Self {
        Self {
            text: props.text,
            variant: props.variant.unwrap_or(ButtonVariant::Primary),
            disabled: props.disabled.unwrap_or(false),
            size: props.size.unwrap_or(ButtonSize::Medium),
            on_click: props.on_click,
        }
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.text = props.text;
        self.variant = props.variant.unwrap_or(self.variant);
        self.disabled = props.disabled.unwrap_or(self.disabled);
        self.size = props.size.unwrap_or(self.size);
        self.on_click = props.on_click;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // For now, return an empty Vec since we're not yet using the Node system
        // In a real implementation, this would return the actual DOM nodes
        Ok(vec![])
    }
}
