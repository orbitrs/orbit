// Theme support for OrbitKit

use crate::component::{Component, ComponentError, Context, Node};
use std::any::Any;

/// Theme for OrbitKit
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary color
    pub primary_color: String,
    /// Secondary color
    pub secondary_color: String,
    /// Text color
    pub text_color: String,
    /// Background color
    pub background_color: String,
    /// Error color
    pub error_color: String,
    /// Success color
    pub success_color: String,
    /// Warning color
    pub warning_color: String,
    /// Info color
    pub info_color: String,
    /// Border radius
    pub border_radius: String,
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: "#0070f3".to_string(),
            secondary_color: "#f5f5f5".to_string(),
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            error_color: "#ff0000".to_string(),
            success_color: "#00cc00".to_string(),
            warning_color: "#ffcc00".to_string(),
            info_color: "#0088cc".to_string(),
            border_radius: "4px".to_string(),
            font_family: "Arial, sans-serif".to_string(),
            font_size: "16px".to_string(),
        }
    }
}

/// Theme provider component
#[derive(Debug)]
pub struct ThemeProvider {
    /// Theme
    pub theme: Theme,
    /// Child content
    pub children: Option<String>,
}

/// Theme provider props
#[derive(Debug, Clone)]
pub struct ThemeProviderProps {
    /// Theme
    pub theme: Option<Theme>,
    /// Child content
    pub children: Option<String>,
}

impl Component for ThemeProvider {
    type Props = ThemeProviderProps;

    fn create(props: Self::Props, _context: Context) -> Self {
        Self {
            theme: props.theme.unwrap_or_default(),
            children: props.children,
        }
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.theme = props.theme.unwrap_or_else(|| self.theme.clone());
        self.children = props.children;
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
