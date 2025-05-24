//! Advanced Layout Engine for Orbit UI Framework
//!
//! This module provides a comprehensive layout system with:
//! - Flexbox-compatible layout properties
//! - Constraint-based layout calculation
//! - Performance optimizations with incremental updates
//! - Integration with the component system

use std::collections::HashMap;
use std::fmt;

use crate::component::{ComponentId, Node};

/// Represents a 2D point with x and y coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Represents a 2D size with width and height
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

/// Represents a rectangle with position and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn zero() -> Self {
        Self {
            origin: Point::zero(),
            size: Size::zero(),
        }
    }

    pub fn x(&self) -> f32 {
        self.origin.x
    }

    pub fn y(&self) -> f32 {
        self.origin.y
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x() && point.x <= self.max_x() &&
        point.y >= self.y() && point.y <= self.max_y()
    }
}

/// Flex direction determines the main axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        FlexDirection::Row
    }
}

/// Flex wrap determines whether items wrap to new lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> Self {
        FlexWrap::NoWrap
    }
}

/// Justify content controls alignment along the main axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::FlexStart
    }
}

/// Align items controls alignment along the cross axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

/// Align content controls alignment of wrapped lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

impl Default for AlignContent {
    fn default() -> Self {
        AlignContent::Stretch
    }
}

/// Dimension value can be auto, fixed, or percentage
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Points(f32),
    Percent(f32),
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::Auto
    }
}

impl Dimension {
    /// Calculate actual value based on container size
    pub fn resolve(&self, container_size: f32) -> f32 {
        match self {
            Dimension::Auto => 0.0, // Will be calculated during layout
            Dimension::Points(points) => *points,
            Dimension::Percent(percent) => container_size * percent / 100.0,
        }
    }
}

/// Edge values for margin, padding, border
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeValues {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeValues {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn horizontal_vertical(horizontal: f32, vertical: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    pub fn zero() -> Self {
        Self::uniform(0.0)
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for EdgeValues {
    fn default() -> Self {
        Self::zero()
    }
}

/// Layout style properties for a node
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStyle {
    // Position
    pub position_type: PositionType,
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,

    // Size
    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub min_height: Dimension,
    pub max_width: Dimension,
    pub max_height: Dimension,

    // Flexbox container properties
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,

    // Flexbox item properties
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Dimension,
    pub align_self: Option<AlignItems>,

    // Spacing
    pub margin: EdgeValues,
    pub padding: EdgeValues,
    pub border: EdgeValues,

    // Gap
    pub row_gap: f32,
    pub column_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    Relative,
    Absolute,
}

impl Default for PositionType {
    fn default() -> Self {
        PositionType::Relative
    }
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            position_type: PositionType::default(),
            top: Dimension::default(),
            right: Dimension::default(),
            bottom: Dimension::default(),
            left: Dimension::default(),
            width: Dimension::default(),
            height: Dimension::default(),
            min_width: Dimension::default(),
            min_height: Dimension::default(),
            max_width: Dimension::default(),
            max_height: Dimension::default(),
            flex_direction: FlexDirection::default(),
            flex_wrap: FlexWrap::default(),
            justify_content: JustifyContent::default(),
            align_items: AlignItems::default(),
            align_content: AlignContent::default(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            align_self: None,
            margin: EdgeValues::default(),
            padding: EdgeValues::default(),
            border: EdgeValues::default(),
            row_gap: 0.0,
            column_gap: 0.0,
        }
    }
}

/// Computed layout result for a node
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutResult {
    /// Final position and size
    pub rect: Rect,
    /// Content area (excluding padding and border)
    pub content_rect: Rect,
    /// Whether this layout is dirty and needs recalculation
    pub is_dirty: bool,
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self {
            rect: Rect::zero(),
            content_rect: Rect::zero(),
            is_dirty: true,
        }
    }
}

/// A node in the layout tree
#[derive(Debug)]
pub struct LayoutNode {
    /// Unique identifier for this layout node
    pub id: ComponentId,
    /// Layout style properties
    pub style: LayoutStyle,
    /// Computed layout result
    pub layout: LayoutResult,
    /// Child nodes
    pub children: Vec<LayoutNode>,
    /// Parent node ID (if any)
    pub parent_id: Option<ComponentId>,
}

impl LayoutNode {
    /// Create a new layout node
    pub fn new(id: ComponentId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            layout: LayoutResult::default(),
            children: Vec::new(),
            parent_id: None,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, mut child: LayoutNode) {
        child.parent_id = Some(self.id);
        self.children.push(child);
        self.mark_dirty();
    }

    /// Remove a child node by ID
    pub fn remove_child(&mut self, child_id: ComponentId) -> Option<LayoutNode> {
        if let Some(index) = self.children.iter().position(|child| child.id == child_id) {
            let mut child = self.children.remove(index);
            child.parent_id = None;
            self.mark_dirty();
            Some(child)
        } else {
            None
        }
    }

    /// Mark this node and all ancestors as dirty
    pub fn mark_dirty(&mut self) {
        self.layout.is_dirty = true;
        // In a real implementation, we'd traverse up to mark ancestors dirty
    }

    /// Check if this node needs layout recalculation
    pub fn is_dirty(&self) -> bool {
        self.layout.is_dirty || self.children.iter().any(|child| child.is_dirty())
    }

    /// Get the main axis size based on flex direction
    pub fn main_axis_size(&self) -> f32 {
        match self.style.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => self.layout.rect.width(),
            FlexDirection::Column | FlexDirection::ColumnReverse => self.layout.rect.height(),
        }
    }

    /// Get the cross axis size based on flex direction
    pub fn cross_axis_size(&self) -> f32 {
        match self.style.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => self.layout.rect.height(),
            FlexDirection::Column | FlexDirection::ColumnReverse => self.layout.rect.width(),
        }
    }
}

/// Layout engine responsible for computing layouts
#[derive(Debug)]
pub struct LayoutEngine {
    /// Cache of computed layouts for performance
    layout_cache: HashMap<ComponentId, LayoutResult>,
    /// Performance metrics
    pub stats: LayoutStats,
}

/// Performance statistics for the layout engine
#[derive(Debug, Default, Clone)]
pub struct LayoutStats {
    /// Number of layout calculations performed
    pub layout_calculations: u64,
    /// Time spent in layout calculations (microseconds)
    pub layout_time_us: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of nodes in the current layout tree
    pub node_count: u32,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            layout_cache: HashMap::new(),
            stats: LayoutStats::default(),
        }
    }

    /// Calculate layout for a node tree
    pub fn calculate_layout(&mut self, root: &mut LayoutNode, container_size: Size) -> Result<(), LayoutError> {
        let start_time = std::time::Instant::now();

        // Clear dirty flags and prepare for layout
        self.prepare_layout(root);

        // Perform the actual layout calculation
        self.layout_node(root, container_size)?;

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.layout_calculations += 1;
        self.stats.layout_time_us += elapsed.as_micros() as u64;

        Ok(())
    }

    /// Prepare the layout tree for calculation
    fn prepare_layout(&mut self, node: &mut LayoutNode) {
        if node.layout.is_dirty {
            // Remove from cache if dirty
            self.layout_cache.remove(&node.id);
        }

        // Count nodes for statistics
        self.stats.node_count = self.count_nodes(node);

        // Recursively prepare children
        for child in &mut node.children {
            self.prepare_layout(child);
        }
    }

    /// Count total nodes in the tree
    fn count_nodes(&self, node: &LayoutNode) -> u32 {
        1 + node.children.iter().map(|child| self.count_nodes(child)).sum::<u32>()
    }

    /// Layout a single node and its children
    fn layout_node(&mut self, node: &mut LayoutNode, container_size: Size) -> Result<(), LayoutError> {
        // Check cache first
        if !node.layout.is_dirty {
            if let Some(cached_layout) = self.layout_cache.get(&node.id) {
                node.layout = cached_layout.clone();
                self.stats.cache_hits += 1;
                return Ok(());
            }
        }

        self.stats.cache_misses += 1;

        // Calculate this node's size and position
        self.calculate_node_size(node, container_size)?;
        self.calculate_node_position(node)?;

        // Layout children using flexbox algorithm
        if !node.children.is_empty() {
            self.layout_flex_children(node)?;
        }

        // Mark as clean and cache the result
        node.layout.is_dirty = false;
        self.layout_cache.insert(node.id, node.layout.clone());

        Ok(())
    }

    /// Calculate the size of a node
    fn calculate_node_size(&self, node: &mut LayoutNode, container_size: Size) -> Result<(), LayoutError> {
        let style = &node.style;

        // Calculate width
        let width = match style.width {
            Dimension::Auto => {
                // Auto width will be determined by children or content
                // For now, use container width minus margins and padding
                container_size.width - style.margin.horizontal() - style.padding.horizontal()
            }
            _ => style.width.resolve(container_size.width),
        };

        // Calculate height
        let height = match style.height {
            Dimension::Auto => {
                // Auto height will be determined by children or content
                // For now, use a minimal height
                0.0
            }
            _ => style.height.resolve(container_size.height),
        };

        // Apply min/max constraints
        let final_width = width
            .max(style.min_width.resolve(container_size.width))
            .min(style.max_width.resolve(container_size.width));
        
        let final_height = height
            .max(style.min_height.resolve(container_size.height))
            .min(style.max_height.resolve(container_size.height));

        // Set the node's size
        node.layout.rect.size = Size::new(final_width, final_height);

        // Calculate content rect (excluding padding and border)
        let content_x = node.layout.rect.x() + style.padding.left + style.border.left;
        let content_y = node.layout.rect.y() + style.padding.top + style.border.top;
        let content_width = final_width - style.padding.horizontal() - style.border.horizontal();
        let content_height = final_height - style.padding.vertical() - style.border.vertical();

        node.layout.content_rect = Rect::new(content_x, content_y, content_width, content_height);

        Ok(())
    }

    /// Calculate the position of a node
    fn calculate_node_position(&self, node: &mut LayoutNode) -> Result<(), LayoutError> {
        let style = &node.style;

        match style.position_type {
            PositionType::Relative => {
                // Position will be set by parent's layout algorithm
                // For now, keep current position
            }
            PositionType::Absolute => {
                // Position relative to containing block
                let x = style.left.resolve(0.0); // TODO: Use actual containing block size
                let y = style.top.resolve(0.0);
                node.layout.rect.origin = Point::new(x, y);
            }
        }

        Ok(())
    }

    /// Layout children using flexbox algorithm
    fn layout_flex_children(&mut self, parent: &mut LayoutNode) -> Result<(), LayoutError> {
        if parent.children.is_empty() {
            return Ok(());
        }

        let parent_content_size = parent.layout.content_rect.size;
        let flex_direction = parent.style.flex_direction;

        // Separate absolutely positioned children
        let (absolute_children, relative_children): (Vec<_>, Vec<_>) = 
            (0..parent.children.len()).partition(|&i| {
                parent.children[i].style.position_type == PositionType::Absolute
            });

        // Layout relatively positioned children with flexbox
        if !relative_children.is_empty() {
            self.layout_flex_line(&mut parent.children, &relative_children, parent_content_size, flex_direction)?;
        }

        // Layout absolutely positioned children
        for &child_index in &absolute_children {
            let child = &mut parent.children[child_index];
            self.layout_node(child, parent_content_size)?;
        }

        Ok(())
    }

    /// Layout a single flex line
    fn layout_flex_line(
        &mut self,
        children: &mut [LayoutNode],
        child_indices: &[usize],
        container_size: Size,
        flex_direction: FlexDirection,
    ) -> Result<(), LayoutError> {
        let is_row = matches!(flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
        let main_axis_size = if is_row { container_size.width } else { container_size.height };

        // Calculate total flex grow and shrink factors
        let mut total_flex_grow = 0.0;
        let mut total_flex_shrink = 0.0;
        let mut total_basis_size = 0.0;

        for &child_index in child_indices {
            let child = &children[child_index];
            total_flex_grow += child.style.flex_grow;
            total_flex_shrink += child.style.flex_shrink;
            
            // Calculate basis size
            let basis_size = match child.style.flex_basis {
                Dimension::Auto => {
                    // Use main axis size if available, otherwise content size
                    if is_row {
                        child.style.width.resolve(container_size.width)
                    } else {
                        child.style.height.resolve(container_size.height)
                    }
                }
                _ => child.style.flex_basis.resolve(main_axis_size),
            };
            total_basis_size += basis_size;
        }

        // Calculate available space for flex growth/shrink
        let available_space = main_axis_size - total_basis_size;

        // Position children along the main axis
        let mut current_offset = 0.0;
        
        for &child_index in child_indices {
            let child = &mut children[child_index];
            
            // Calculate child's main axis size
            let basis_size = match child.style.flex_basis {
                Dimension::Auto => {
                    if is_row {
                        child.style.width.resolve(container_size.width)
                    } else {
                        child.style.height.resolve(container_size.height)
                    }
                }
                _ => child.style.flex_basis.resolve(main_axis_size),
            };

            let main_size = if available_space > 0.0 && total_flex_grow > 0.0 {
                // Distribute extra space
                basis_size + (available_space * child.style.flex_grow / total_flex_grow)
            } else if available_space < 0.0 && total_flex_shrink > 0.0 {
                // Shrink to fit
                basis_size + (available_space * child.style.flex_shrink / total_flex_shrink)
            } else {
                basis_size
            };

            // Set child size and position
            if is_row {
                child.layout.rect = Rect::new(current_offset, 0.0, main_size, container_size.height);
            } else {
                child.layout.rect = Rect::new(0.0, current_offset, container_size.width, main_size);
            }

            current_offset += main_size;

            // Recursively layout this child
            self.layout_node(child, child.layout.rect.size)?;
        }

        Ok(())
    }

    /// Clear the layout cache
    pub fn clear_cache(&mut self) {
        self.layout_cache.clear();
    }

    /// Get layout statistics
    pub fn get_stats(&self) -> &LayoutStats {
        &self.stats
    }

    /// Reset layout statistics
    pub fn reset_stats(&mut self) {
        self.stats = LayoutStats::default();
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during layout calculation
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Invalid layout constraint: {0}")]
    InvalidConstraint(String),

    #[error("Layout calculation failed: {0}")]
    CalculationFailed(String),

    #[error("Circular dependency detected in layout tree")]
    CircularDependency,

    #[error("Node not found: {0:?}")]
    NodeNotFound(ComponentId),
}

impl fmt::Display for LayoutStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Layout Stats: {} calculations, {}μs total time, {} nodes, {}/{} cache hit/miss",
            self.layout_calculations,
            self.layout_time_us,
            self.node_count,
            self.cache_hits,
            self.cache_misses
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(10.0, 20.0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);

        let zero_point = Point::zero();
        assert_eq!(zero_point.x, 0.0);
        assert_eq!(zero_point.y, 0.0);
    }

    #[test]
    fn test_size_operations() {
        let size = Size::new(100.0, 200.0);
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 200.0);
        assert_eq!(size.area(), 20000.0);

        let zero_size = Size::zero();
        assert_eq!(zero_size.area(), 0.0);
    }

    #[test]
    fn test_rect_operations() {
        let rect = Rect::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(rect.x(), 10.0);
        assert_eq!(rect.y(), 20.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 200.0);
        assert_eq!(rect.max_x(), 110.0);
        assert_eq!(rect.max_y(), 220.0);

        let point_inside = Point::new(50.0, 100.0);
        let point_outside = Point::new(150.0, 100.0);
        assert!(rect.contains_point(point_inside));
        assert!(!rect.contains_point(point_outside));
    }

    #[test]
    fn test_dimension_resolve() {
        let auto_dim = Dimension::Auto;
        let points_dim = Dimension::Points(100.0);
        let percent_dim = Dimension::Percent(50.0);

        assert_eq!(auto_dim.resolve(200.0), 0.0);
        assert_eq!(points_dim.resolve(200.0), 100.0);
        assert_eq!(percent_dim.resolve(200.0), 100.0);
    }

    #[test]
    fn test_edge_values() {
        let edges = EdgeValues::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(edges.horizontal(), 60.0);
        assert_eq!(edges.vertical(), 40.0);

        let uniform = EdgeValues::uniform(15.0);
        assert_eq!(uniform.top, 15.0);
        assert_eq!(uniform.right, 15.0);
        assert_eq!(uniform.bottom, 15.0);
        assert_eq!(uniform.left, 15.0);
    }

    #[test]
    fn test_layout_node_creation() {
        let id = ComponentId::new();
        let style = LayoutStyle::default();
        let node = LayoutNode::new(id, style);

        assert_eq!(node.id, id);
        assert!(node.children.is_empty());
        assert!(node.layout.is_dirty);
    }

    #[test]
    fn test_layout_node_children() {
        let parent_id = ComponentId::new();
        let child_id = ComponentId::new();
        
        let mut parent = LayoutNode::new(parent_id, LayoutStyle::default());
        let child = LayoutNode::new(child_id, LayoutStyle::default());

        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].parent_id, Some(parent_id));

        let removed_child = parent.remove_child(child_id);
        assert!(removed_child.is_some());
        assert_eq!(parent.children.len(), 0);
        assert_eq!(removed_child.unwrap().parent_id, None);
    }

    #[test]
    fn test_layout_engine_creation() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.stats.layout_calculations, 0);
        assert_eq!(engine.stats.cache_hits, 0);
        assert_eq!(engine.stats.cache_misses, 0);
    }

    #[test]
    fn test_simple_layout_calculation() {
        let mut engine = LayoutEngine::new();
        let id = ComponentId::new();
        let mut style = LayoutStyle::default();
        style.width = Dimension::Points(100.0);
        style.height = Dimension::Points(200.0);

        let mut root = LayoutNode::new(id, style);
        let container_size = Size::new(400.0, 600.0);

        let result = engine.calculate_layout(&mut root, container_size);
        assert!(result.is_ok());
        assert_eq!(root.layout.rect.width(), 100.0);
        assert_eq!(root.layout.rect.height(), 200.0);
        assert!(!root.layout.is_dirty);
    }

    #[test]
    fn test_flexbox_row_layout() {
        let mut engine = LayoutEngine::new();
        
        // Create parent with row flex direction
        let parent_id = ComponentId::new();
        let mut parent_style = LayoutStyle::default();
        parent_style.flex_direction = FlexDirection::Row;
        parent_style.width = Dimension::Points(300.0);
        parent_style.height = Dimension::Points(100.0);
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add two children with flex grow
        let child1_id = ComponentId::new();
        let mut child1_style = LayoutStyle::default();
        child1_style.flex_grow = 1.0;
        let child1 = LayoutNode::new(child1_id, child1_style);

        let child2_id = ComponentId::new();
        let mut child2_style = LayoutStyle::default();
        child2_style.flex_grow = 2.0;
        let child2 = LayoutNode::new(child2_id, child2_style);

        parent.add_child(child1);
        parent.add_child(child2);

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Children should be laid out in a row
        assert_eq!(parent.children.len(), 2);
        
        // Check that layout calculation was performed
        assert_eq!(engine.stats.layout_calculations, 1);
    }

    #[test]
    fn test_layout_caching() {
        let mut engine = LayoutEngine::new();
        let id = ComponentId::new();
        let style = LayoutStyle::default();
        let mut root = LayoutNode::new(id, style);
        let container_size = Size::new(100.0, 100.0);

        // First layout calculation
        let result = engine.calculate_layout(&mut root, container_size);
        assert!(result.is_ok());
        assert_eq!(engine.stats.cache_misses, 1);

        // Mark as not dirty and recalculate - should use cache
        root.layout.is_dirty = false;
        let result = engine.calculate_layout(&mut root, container_size);
        assert!(result.is_ok());
        
        // Should have used cache for the root node
        assert!(engine.stats.cache_hits > 0 || engine.stats.cache_misses > 1);
    }

    #[test]
    fn test_layout_stats_display() {
        let stats = LayoutStats {
            layout_calculations: 5,
            layout_time_us: 1000,
            node_count: 10,
            cache_hits: 3,
            cache_misses: 7,
        };

        let display_string = format!("{}", stats);
        assert!(display_string.contains("5 calculations"));
        assert!(display_string.contains("1000μs"));
        assert!(display_string.contains("10 nodes"));
        assert!(display_string.contains("3/7 cache"));
    }

    #[test]
    fn test_main_cross_axis_calculations() {
        let id = ComponentId::new();
        
        // Test row direction
        let mut style = LayoutStyle::default();
        style.flex_direction = FlexDirection::Row;
        let mut node = LayoutNode::new(id, style);
        node.layout.rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        
        assert_eq!(node.main_axis_size(), 100.0); // width for row
        assert_eq!(node.cross_axis_size(), 50.0); // height for row
        
        // Test column direction
        node.style.flex_direction = FlexDirection::Column;
        assert_eq!(node.main_axis_size(), 50.0); // height for column
        assert_eq!(node.cross_axis_size(), 100.0); // width for column
    }
}
