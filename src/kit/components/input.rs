// Input component for OrbitKit

use crate::component::{Component, ComponentError, Context, Node};

/// Input component
#[derive(Debug)]
pub struct Input {
    /// Input type (text, password, email, etc.)
    pub input_type: String,
    /// Input value
    pub value: String,
    /// Input placeholder
    pub placeholder: Option<String>,
    /// Whether the input is disabled
    pub disabled: bool,
    /// Whether the input is required
    pub required: bool,
    /// Input label
    pub label: Option<String>,
    /// Input error message
    pub error: Option<String>,
    /// Input helper text
    pub helper_text: Option<String>,
    /// On change handler
    pub on_change: Option<fn(String)>,
}

/// Input props
#[derive(Debug, Clone)]
pub struct InputProps {
    /// Input type (text, password, email, etc.)
    pub input_type: Option<String>,
    /// Input value
    pub value: String,
    /// Input placeholder
    pub placeholder: Option<String>,
    /// Whether the input is disabled
    pub disabled: Option<bool>,
    /// Whether the input is required
    pub required: Option<bool>,
    /// Input label
    pub label: Option<String>,
    /// Input error message
    pub error: Option<String>,
    /// Input helper text
    pub helper_text: Option<String>,
    /// On change handler
    pub on_change: Option<fn(String)>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            input_type: "text".to_string(),
            value: String::new(),
            placeholder: None,
            disabled: false,
            required: false,
            label: None,
            error: None,
            helper_text: None,
            on_change: None,
        }
    }
}

impl Component for Input {
    type Props = InputProps;

    fn create(props: Self::Props, _context: Context) -> Self {
        Self {
            input_type: props.input_type.unwrap_or_else(|| "text".to_string()),
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled.unwrap_or(false),
            required: props.required.unwrap_or(false),
            label: props.label,
            error: props.error,
            helper_text: props.helper_text,
            on_change: props.on_change,
        }
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.input_type = props.input_type.unwrap_or_else(|| self.input_type.clone());
        self.value = props.value;
        self.placeholder = props.placeholder;
        self.disabled = props.disabled.unwrap_or(self.disabled);
        self.required = props.required.unwrap_or(self.required);
        self.label = props.label;
        self.error = props.error;
        self.helper_text = props.helper_text;
        self.on_change = props.on_change;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(&self) -> Result<Vec<Node>, ComponentError> {
        // For now, return an empty Vec since we're not yet using the Node system
        // In a real implementation, this would return the actual DOM nodes
        Ok(vec![])
    }
}
