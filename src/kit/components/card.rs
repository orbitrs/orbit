// Card component for OrbitKit

use crate::component::{Component, ComponentError, ComponentId, Context, Node};

/// Card component
#[derive(Debug)]
pub struct Card {
    /// Component ID for tracking
    id: ComponentId,
    /// Card title
    pub title: Option<String>,
    /// Card elevation (shadow level)
    pub elevation: u8,
    /// Card border radius
    pub border_radius: String,
    /// Whether the card has a border
    pub bordered: bool,
    /// Card padding
    pub padding: String,
    /// Child content
    pub children: Option<String>,
}

/// Card props
#[derive(Debug, Clone)]
pub struct CardProps {
    /// Card title
    pub title: Option<String>,
    /// Card elevation (shadow level)
    pub elevation: Option<u8>,
    /// Card border radius
    pub border_radius: Option<String>,
    /// Whether the card has a border
    pub bordered: Option<bool>,
    /// Card padding
    pub padding: Option<String>,
    /// Child content
    pub children: Option<String>,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: ComponentId::new(),
            title: None,
            elevation: 1,
            border_radius: "4px".to_string(),
            bordered: false,
            padding: "16px".to_string(),
            children: None,
        }
    }
}

impl Component for Card {
    type Props = CardProps;

    fn component_id(&self) -> ComponentId {
        self.id
    }

    fn create(props: Self::Props, _context: Context) -> Self {
        Self {
            id: ComponentId::new(),
            title: props.title,
            elevation: props.elevation.unwrap_or(1),
            border_radius: props.border_radius.unwrap_or_else(|| "4px".to_string()),
            bordered: props.bordered.unwrap_or(false),
            padding: props.padding.unwrap_or_else(|| "16px".to_string()),
            children: props.children,
        }
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.title = props.title;
        self.elevation = props.elevation.unwrap_or(self.elevation);
        self.border_radius = props
            .border_radius
            .unwrap_or_else(|| self.border_radius.clone());
        self.bordered = props.bordered.unwrap_or(self.bordered);
        self.padding = props.padding.unwrap_or_else(|| self.padding.clone());
        self.children = props.children;
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
