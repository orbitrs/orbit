// Layout components for OrbitKit

use crate::component::{Component, ComponentError, Context, Node};
use std::any::Any;

/// Layout component
#[derive(Debug)]
pub struct Layout {
    /// Layout direction (row, column)
    pub direction: Direction,
    /// Layout alignment
    pub align: Alignment,
    /// Layout justification
    pub justify: Justification,
    /// Layout gap
    pub gap: String,
    /// Layout padding
    pub padding: String,
    /// Child content
    pub children: Option<String>,
}

/// Layout props
#[derive(Debug, Clone)]
pub struct LayoutProps {
    /// Layout direction (row, column)
    pub direction: Option<Direction>,
    /// Layout alignment
    pub align: Option<Alignment>,
    /// Layout justification
    pub justify: Option<Justification>,
    /// Layout gap
    pub gap: Option<String>,
    /// Layout padding
    pub padding: Option<String>,
    /// Child content
    pub children: Option<String>,
}

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Horizontal layout
    Row,
    /// Vertical layout
    Column,
}

/// Layout alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Align items at the start
    Start,
    /// Align items at the center
    Center,
    /// Align items at the end
    End,
    /// Stretch items to fill the container
    Stretch,
}

/// Layout justification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justification {
    /// Justify items at the start
    Start,
    /// Justify items at the center
    Center,
    /// Justify items at the end
    End,
    /// Space between items
    SpaceBetween,
    /// Space around items
    SpaceAround,
    /// Space evenly between items
    SpaceEvenly,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Row
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Start
    }
}

impl Default for Justification {
    fn default() -> Self {
        Justification::Start
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::default(),
            align: Alignment::default(),
            justify: Justification::default(),
            gap: "0px".to_string(),
            padding: "0px".to_string(),
            children: None,
        }
    }
}

impl Component for Layout {
    type Props = LayoutProps;

    fn create(props: Self::Props, _context: Context) -> Self {
        Self {
            direction: props.direction.unwrap_or_default(),
            align: props.align.unwrap_or_default(),
            justify: props.justify.unwrap_or_default(),
            gap: props.gap.unwrap_or_else(|| "0px".to_string()),
            padding: props.padding.unwrap_or_else(|| "0px".to_string()),
            children: props.children,
        }
    }

    fn update(&mut self, props: Self::Props) -> Result<(), ComponentError> {
        self.direction = props.direction.unwrap_or(self.direction);
        self.align = props.align.unwrap_or(self.align);
        self.justify = props.justify.unwrap_or(self.justify);
        self.gap = props.gap.unwrap_or_else(|| self.gap.clone());
        self.padding = props.padding.unwrap_or_else(|| self.padding.clone());
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
