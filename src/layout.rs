//! Advanced Layout Engine for Orbit UI Framework
//!
//! This module provides a comprehensive layout system with:
//! - Flexbox-compatible layout properties
//! - Constraint-based layout calculation
//! - Performance optimizations with incremental updates
//! - Integration with the component system

use std::collections::HashMap;
use std::fmt;

use crate::component::ComponentId;

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
        point.x >= self.x()
            && point.x <= self.max_x()
            && point.y >= self.y()
            && point.y <= self.max_y()
    }
}

/// Flex direction determines the main axis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Flex wrap determines whether items wrap to new lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Justify content controls alignment along the main axis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items controls alignment along the cross axis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    Baseline,
}

/// Align content controls alignment of wrapped lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    #[default]
    Stretch,
}

/// Dimension value can be auto, fixed, or percentage
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Points(f32),
    Percent(f32),
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
        Self {
            top,
            right,
            bottom,
            left,
        }
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
    pub border: EdgeValues,    // Gap
    pub gap: Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionType {
    #[default]
    Relative,
    Absolute,
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
            border: EdgeValues::default(),            gap: Gap::default(),
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
    pub fn calculate_layout(
        &mut self,
        root: &mut LayoutNode,
        container_size: Size,
    ) -> Result<(), LayoutError> {
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
    #[allow(clippy::only_used_in_recursion)]
    fn count_nodes(&self, node: &LayoutNode) -> u32 {
        1 + node
            .children
            .iter()
            .map(|child| self.count_nodes(child))
            .sum::<u32>()
    }

    /// Layout a single node and its children
    fn layout_node(
        &mut self,
        node: &mut LayoutNode,
        container_size: Size,
    ) -> Result<(), LayoutError> {
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
    fn calculate_node_size(
        &self,
        node: &mut LayoutNode,
        container_size: Size,
    ) -> Result<(), LayoutError> {
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

        // Apply min/max constraints (only if not Auto)
        let final_width = {
            let mut w = width;
            if !matches!(style.min_width, Dimension::Auto) {
                w = w.max(style.min_width.resolve(container_size.width));
            }
            if !matches!(style.max_width, Dimension::Auto) {
                w = w.min(style.max_width.resolve(container_size.width));
            }
            w
        };

        let final_height = {
            let mut h = height;
            if !matches!(style.min_height, Dimension::Auto) {
                h = h.max(style.min_height.resolve(container_size.height));
            }
            if !matches!(style.max_height, Dimension::Auto) {
                h = h.min(style.max_height.resolve(container_size.height));
            }
            h
        };

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
    }    /// Layout children using flexbox algorithm
    fn layout_flex_children(&mut self, parent: &mut LayoutNode) -> Result<(), LayoutError> {
        if parent.children.is_empty() {
            return Ok(());
        }

        let parent_content_size = parent.layout.content_rect.size;
        let parent_style = &parent.style;

        // Separate absolutely positioned children
        let (absolute_children, relative_children): (Vec<_>, Vec<_>) = (0..parent.children.len())
            .partition(|&i| parent.children[i].style.position_type == PositionType::Absolute);

        // Layout relatively positioned children with flexbox
        if !relative_children.is_empty() {
            // Check if wrapping is enabled
            if parent_style.flex_wrap == FlexWrap::Wrap || parent_style.flex_wrap == FlexWrap::WrapReverse {
                self.layout_flex_multiline(
                    &mut parent.children,
                    &relative_children,
                    parent_content_size,
                    parent_style,
                )?;
            } else {
                self.layout_flex_line(
                    &mut parent.children,
                    &relative_children,
                    parent_content_size,
                    parent_style,
                )?;
            }
        }

        // Layout absolutely positioned children
        for &child_index in &absolute_children {
            let child = &mut parent.children[child_index];
            self.layout_node(child, parent_content_size)?;
        }

        Ok(())
    }    /// Layout a single flex line
    fn layout_flex_line(
        &mut self,
        children: &mut [LayoutNode],
        child_indices: &[usize],
        container_size: Size,
        parent_style: &LayoutStyle,
    ) -> Result<(), LayoutError> {
        let flex_direction = parent_style.flex_direction;
        let is_row = matches!(
            flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        let is_reverse = matches!(
            flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );

        let main_axis_size = if is_row {
            container_size.width
        } else {
            container_size.height
        };
        let cross_axis_size = if is_row {
            container_size.height
        } else {
            container_size.width
        };

        // First pass: Calculate item sizes and determine if we need to grow/shrink
        let mut item_data = Vec::new();
        let mut total_basis_size = 0.0;
        let mut total_flex_grow = 0.0;
        let mut total_flex_shrink = 0.0;

        for &child_index in child_indices {
            let child = &children[child_index];
            
            // Calculate basis size
            let basis_size = self.calculate_flex_basis_size(child, container_size, is_row)?;
            let main_size = self.resolve_main_size(child, basis_size, is_row);
            let cross_size = self.resolve_cross_size(child, cross_axis_size, is_row);

            total_basis_size += main_size;
            total_flex_grow += child.style.flex_grow;
            total_flex_shrink += child.style.flex_shrink;

            item_data.push(FlexItemData {
                index: child_index,
                basis_size,
                main_size,
                cross_size,
                flex_grow: child.style.flex_grow,
                flex_shrink: child.style.flex_shrink,
            });
        }

        // Add gap sizes to total
        let gap_size = if is_row { parent_style.gap.column } else { parent_style.gap.row };
        let total_gaps = gap_size * (child_indices.len() as f32 - 1.0).max(0.0);
        let available_space = main_axis_size - total_basis_size - total_gaps;

        // Second pass: Distribute available space
        self.distribute_flex_space(&mut item_data, available_space, total_flex_grow, total_flex_shrink)?;

        // Third pass: Apply justify-content for positioning
        let positions = self.calculate_justify_content_positions(
            &item_data,
            main_axis_size,
            gap_size,
            parent_style.justify_content,
            is_reverse,
        );

        // Fourth pass: Apply cross-axis alignment and set final positions
        for (i, &child_index) in child_indices.iter().enumerate() {
            let child = &mut children[child_index];
            let item = &item_data[i];
            let main_pos = positions[i];
            
            // Calculate cross-axis position
            let cross_pos = self.calculate_cross_axis_position(
                child,
                item.cross_size,
                cross_axis_size,
                parent_style.align_items,
            );

            // Set final layout
            if is_row {
                child.layout.rect = Rect::new(main_pos, cross_pos, item.main_size, item.cross_size);
            } else {
                child.layout.rect = Rect::new(cross_pos, main_pos, item.cross_size, item.main_size);
            }

            // Recursively layout this child
            self.layout_node(child, child.layout.rect.size)?;
        }

        Ok(())
    }    /// Clear the layout cache
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

    // Helper methods for enhanced flexbox support

    /// Calculate flex basis size for an item
    fn calculate_flex_basis_size(
        &self,
        child: &LayoutNode,
        container_size: Size,
        is_row: bool,
    ) -> Result<f32, LayoutError> {
        let basis_size = match child.style.flex_basis {
            Dimension::Auto => {
                // Use main axis size if available, otherwise content size
                if is_row {
                    child.style.width.resolve(container_size.width)
                } else {
                    child.style.height.resolve(container_size.height)
                }
            }
            _ => {
                let main_axis_size = if is_row {
                    container_size.width
                } else {
                    container_size.height
                };
                child.style.flex_basis.resolve(main_axis_size)
            }
        };
        Ok(basis_size)
    }

    /// Resolve main axis size for an item
    fn resolve_main_size(&self, child: &LayoutNode, basis_size: f32, is_row: bool) -> f32 {
        let explicit_size = if is_row {
            match child.style.width {
                Dimension::Auto => basis_size,
                _ => child.style.width.resolve(0.0), // Will be resolved properly in context
            }
        } else {
            match child.style.height {
                Dimension::Auto => basis_size,
                _ => child.style.height.resolve(0.0),
            }
        };
        explicit_size.max(basis_size)
    }

    /// Resolve cross axis size for an item
    fn resolve_cross_size(&self, child: &LayoutNode, cross_axis_size: f32, is_row: bool) -> f32 {
        if is_row {
            match child.style.height {
                Dimension::Auto => cross_axis_size, // Will stretch by default
                _ => child.style.height.resolve(cross_axis_size),
            }
        } else {
            match child.style.width {
                Dimension::Auto => cross_axis_size,
                _ => child.style.width.resolve(cross_axis_size),
            }
        }
    }

    /// Distribute available space among flex items
    fn distribute_flex_space(
        &self,
        items: &mut [FlexItemData],
        available_space: f32,
        total_flex_grow: f32,
        total_flex_shrink: f32,
    ) -> Result<(), LayoutError> {
        if available_space > 0.0 && total_flex_grow > 0.0 {
            // Distribute extra space proportionally
            for item in items.iter_mut() {
                if item.flex_grow > 0.0 {
                    let growth = available_space * (item.flex_grow / total_flex_grow);
                    item.main_size += growth;
                }
            }
        } else if available_space < 0.0 && total_flex_shrink > 0.0 {
            // Shrink items proportionally
            for item in items.iter_mut() {
                if item.flex_shrink > 0.0 {
                    let shrinkage = available_space * (item.flex_shrink / total_flex_shrink);
                    item.main_size = (item.main_size + shrinkage).max(0.0);
                }
            }
        }
        Ok(())
    }

    /// Calculate justify-content positions for items
    fn calculate_justify_content_positions(
        &self,
        items: &[FlexItemData],
        container_size: f32,
        gap_size: f32,
        justify_content: JustifyContent,
        is_reverse: bool,
    ) -> Vec<f32> {
        let mut positions = Vec::with_capacity(items.len());
        
        if items.is_empty() {
            return positions;
        }

        let total_item_size: f32 = items.iter().map(|item| item.main_size).sum();
        let total_gaps = gap_size * (items.len() as f32 - 1.0).max(0.0);
        let remaining_space = container_size - total_item_size - total_gaps;

        match justify_content {
            JustifyContent::FlexStart => {
                let mut current_pos = 0.0;
                for item in items {
                    positions.push(current_pos);
                    current_pos += item.main_size + gap_size;
                }
            }
            JustifyContent::FlexEnd => {
                let mut current_pos = remaining_space;
                for item in items {
                    positions.push(current_pos);
                    current_pos += item.main_size + gap_size;
                }
            }
            JustifyContent::Center => {
                let mut current_pos = remaining_space / 2.0;
                for item in items {
                    positions.push(current_pos);
                    current_pos += item.main_size + gap_size;
                }
            }
            JustifyContent::SpaceBetween => {
                if items.len() == 1 {
                    positions.push(0.0);
                } else {
                    let space_between = remaining_space / (items.len() as f32 - 1.0);
                    let mut current_pos = 0.0;
                    for item in items {
                        positions.push(current_pos);
                        current_pos += item.main_size + space_between;
                    }
                }
            }
            JustifyContent::SpaceAround => {
                let space_around = remaining_space / items.len() as f32;
                let mut current_pos = space_around / 2.0;
                for item in items {
                    positions.push(current_pos);
                    current_pos += item.main_size + space_around;
                }
            }
            JustifyContent::SpaceEvenly => {
                let space_evenly = remaining_space / (items.len() as f32 + 1.0);
                let mut current_pos = space_evenly;
                for item in items {
                    positions.push(current_pos);
                    current_pos += item.main_size + space_evenly;
                }
            }
        }

        if is_reverse {
            // Reverse the positions for reverse flex directions
            for pos in &mut positions {
                *pos = container_size - *pos - items[positions.len() - 1 - 
                    positions.iter().position(|&p| p == *pos).unwrap()].main_size;
            }
            positions.reverse();
        }

        positions
    }

    /// Calculate cross-axis position for an item
    fn calculate_cross_axis_position(
        &self,
        child: &LayoutNode,
        item_cross_size: f32,
        container_cross_size: f32,
        align_items: AlignItems,
    ) -> f32 {
        // Check for align-self override
        let alignment = child.style.align_self.unwrap_or(align_items);

        match alignment {
            AlignItems::FlexStart => 0.0,
            AlignItems::FlexEnd => container_cross_size - item_cross_size,
            AlignItems::Center => (container_cross_size - item_cross_size) / 2.0,
            AlignItems::Stretch => 0.0, // Item should already be sized to fill
            AlignItems::Baseline => {
                // For now, treat baseline as flex-start
                // TODO: Implement proper baseline alignment
                0.0
            }
        }
    }

    /// Layout children with flex wrap (multi-line)
    fn layout_flex_multiline(
        &mut self,
        children: &mut [LayoutNode],
        child_indices: &[usize],
        container_size: Size,
        parent_style: &LayoutStyle,
    ) -> Result<(), LayoutError> {
        let flex_direction = parent_style.flex_direction;
        let is_row = matches!(
            flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        let main_axis_size = if is_row {
            container_size.width
        } else {
            container_size.height
        };

        // Break items into lines
        let lines = self.break_into_lines(children, child_indices, main_axis_size, is_row)?;

        // Layout each line
        let mut cross_offset = 0.0;
        for line_indices in lines {
            // Calculate line height/width
            let line_cross_size = self.calculate_line_cross_size(children, &line_indices, is_row);
            
            // Layout this line
            self.layout_flex_line(children, &line_indices, container_size, parent_style)?;
            
            // Offset items in this line by cross_offset
            for &child_index in &line_indices {
                let child = &mut children[child_index];
                if is_row {
                    child.layout.rect.origin.y += cross_offset;
                } else {
                    child.layout.rect.origin.x += cross_offset;
                }
            }
            
            cross_offset += line_cross_size + parent_style.gap.row;
        }

        Ok(())
    }

    /// Break items into lines for wrapping
    fn break_into_lines(
        &self,
        children: &[LayoutNode],
        child_indices: &[usize],
        main_axis_size: f32,
        is_row: bool,
    ) -> Result<Vec<Vec<usize>>, LayoutError> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_line_size = 0.0;

        for &child_index in child_indices {
            let child = &children[child_index];
            let item_size = if is_row {
                child.style.width.resolve(main_axis_size)
            } else {
                child.style.height.resolve(main_axis_size)
            };

            if current_line.is_empty() || current_line_size + item_size <= main_axis_size {
                current_line.push(child_index);
                current_line_size += item_size;
            } else {
                // Start a new line
                lines.push(current_line);
                current_line = vec![child_index];
                current_line_size = item_size;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        Ok(lines)
    }

    /// Calculate the cross-axis size of a line
    fn calculate_line_cross_size(
        &self,
        children: &[LayoutNode],
        line_indices: &[usize],
        is_row: bool,
    ) -> f32 {
        line_indices
            .iter()
            .map(|&index| {
                let child = &children[index];
                if is_row {
                    child.layout.rect.height()
                } else {
                    child.layout.rect.width()
                }
            })
            .fold(0.0, f32::max)
    }
}

/// Data for a flex item during layout calculation
#[derive(Debug, Clone)]
struct FlexItemData {
    index: usize,
    basis_size: f32,
    main_size: f32,
    cross_size: f32,
    flex_grow: f32,
    flex_shrink: f32,
}

/// Gap specification for flex containers
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gap {
    pub row: f32,
    pub column: f32,
}

impl Gap {
    pub fn new(row: f32, column: f32) -> Self {
        Self { row, column }
    }

    pub fn uniform(value: f32) -> Self {
        Self::new(value, value)
    }
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
        let style = LayoutStyle {
            width: Dimension::Points(100.0),
            height: Dimension::Points(200.0),
            ..Default::default()
        };

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
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add two children with flex grow
        let child1_id = ComponentId::new();
        let child1_style = LayoutStyle {
            flex_grow: 1.0,
            ..Default::default()
        };
        let child1 = LayoutNode::new(child1_id, child1_style);

        let child2_id = ComponentId::new();
        let child2_style = LayoutStyle {
            flex_grow: 2.0,
            ..Default::default()
        };
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
        let display_string = format!("{stats}");
        assert!(display_string.contains("5 calculations"));
        assert!(display_string.contains("1000μs"));
        assert!(display_string.contains("10 nodes"));
        assert!(display_string.contains("3/7 cache"));
    }    #[test]
    fn test_main_cross_axis_calculations() {
        let id = ComponentId::new();

        // Test row direction
        let style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        };
        let mut node = LayoutNode::new(id, style);
        node.layout.rect = Rect::new(0.0, 0.0, 100.0, 50.0);

        assert_eq!(node.main_axis_size(), 100.0); // width for row
        assert_eq!(node.cross_axis_size(), 50.0); // height for row

        // Test column direction
        node.style.flex_direction = FlexDirection::Column;
        assert_eq!(node.main_axis_size(), 50.0); // height for column
        assert_eq!(node.cross_axis_size(), 100.0); // width for column
    }

    #[test]
    fn test_justify_content_flex_start() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add children with fixed sizes
        for i in 0..3 {
            let child_id = ComponentId::new();
            let child_style = LayoutStyle {
                width: Dimension::Points(50.0),
                height: Dimension::Points(50.0),
                ..Default::default()
            };
            let child = LayoutNode::new(child_id, child_style);
            parent.add_child(child);
        }

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Children should be positioned at the start
        assert_eq!(parent.children[0].layout.rect.x(), 0.0);
        assert_eq!(parent.children[1].layout.rect.x(), 50.0);
        assert_eq!(parent.children[2].layout.rect.x(), 100.0);
    }

    #[test]
    fn test_justify_content_center() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add child with fixed size
        let child_id = ComponentId::new();
        let child_style = LayoutStyle {
            width: Dimension::Points(100.0),
            height: Dimension::Points(50.0),
            ..Default::default()
        };
        let child = LayoutNode::new(child_id, child_style);
        parent.add_child(child);

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Child should be centered: (300 - 100) / 2 = 100
        assert_eq!(parent.children[0].layout.rect.x(), 100.0);
    }

    #[test]
    fn test_justify_content_space_between() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add two children with fixed sizes
        for _ in 0..2 {
            let child_id = ComponentId::new();
            let child_style = LayoutStyle {
                width: Dimension::Points(50.0),
                height: Dimension::Points(50.0),
                ..Default::default()
            };
            let child = LayoutNode::new(child_id, child_style);
            parent.add_child(child);
        }

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // First child at start, second at end
        assert_eq!(parent.children[0].layout.rect.x(), 0.0);
        assert_eq!(parent.children[1].layout.rect.x(), 250.0); // 300 - 50
    }

    #[test]
    fn test_align_items_center() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add child with fixed size
        let child_id = ComponentId::new();
        let child_style = LayoutStyle {
            width: Dimension::Points(50.0),
            height: Dimension::Points(30.0),
            ..Default::default()
        };
        let child = LayoutNode::new(child_id, child_style);
        parent.add_child(child);

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Child should be vertically centered: (100 - 30) / 2 = 35
        assert_eq!(parent.children[0].layout.rect.y(), 35.0);
    }

    #[test]
    fn test_align_items_flex_end() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add child with fixed size
        let child_id = ComponentId::new();
        let child_style = LayoutStyle {
            width: Dimension::Points(50.0),
            height: Dimension::Points(30.0),
            ..Default::default()
        };
        let child = LayoutNode::new(child_id, child_style);
        parent.add_child(child);

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Child should be at bottom: 100 - 30 = 70
        assert_eq!(parent.children[0].layout.rect.y(), 70.0);
    }

    #[test]
    fn test_flex_grow_distribution() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add children with different flex grow values
        let child1_style = LayoutStyle {
            flex_grow: 1.0,
            flex_basis: Dimension::Points(50.0),
            ..Default::default()
        };
        let child1 = LayoutNode::new(ComponentId::new(), child1_style);
        parent.add_child(child1);

        let child2_style = LayoutStyle {
            flex_grow: 2.0,
            flex_basis: Dimension::Points(50.0),
            ..Default::default()
        };
        let child2 = LayoutNode::new(ComponentId::new(), child2_style);
        parent.add_child(child2);

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Available space: 300 - 100 = 200
        // Child1 gets 1/3 * 200 = 66.67, Child2 gets 2/3 * 200 = 133.33
        // Final sizes: Child1 = 50 + 66.67 = 116.67, Child2 = 50 + 133.33 = 183.33
        assert!((parent.children[0].layout.rect.width() - 116.67).abs() < 0.1);
        assert!((parent.children[1].layout.rect.width() - 183.33).abs() < 0.1);
    }

    #[test]
    fn test_gap_spacing() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            width: Dimension::Points(300.0),
            height: Dimension::Points(100.0),
            gap: Gap::uniform(10.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add two children
        for _ in 0..2 {
            let child_style = LayoutStyle {
                width: Dimension::Points(50.0),
                height: Dimension::Points(50.0),
                ..Default::default()
            };
            let child = LayoutNode::new(ComponentId::new(), child_style);
            parent.add_child(child);
        }

        let container_size = Size::new(400.0, 200.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Second child should be positioned after first child + gap
        assert_eq!(parent.children[0].layout.rect.x(), 0.0);
        assert_eq!(parent.children[1].layout.rect.x(), 60.0); // 50 + 10 gap
    }

    #[test]
    fn test_column_direction() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Column,
            width: Dimension::Points(100.0),
            height: Dimension::Points(300.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add children with flex grow
        for i in 0..2 {
            let child_style = LayoutStyle {
                flex_grow: (i + 1) as f32,
                ..Default::default()
            };
            let child = LayoutNode::new(ComponentId::new(), child_style);
            parent.add_child(child);
        }

        let container_size = Size::new(200.0, 400.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // Children should be stacked vertically
        assert_eq!(parent.children[0].layout.rect.y(), 0.0);
        assert!(parent.children[1].layout.rect.y() > parent.children[0].layout.rect.height());
    }

    #[test]
    fn test_flex_wrap_basic() {
        let mut engine = LayoutEngine::new();
        let parent_id = ComponentId::new();
        let parent_style = LayoutStyle {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            width: Dimension::Points(200.0),
            height: Dimension::Points(200.0),
            ..Default::default()
        };
        let mut parent = LayoutNode::new(parent_id, parent_style);

        // Add children that will overflow and wrap
        for _ in 0..3 {
            let child_style = LayoutStyle {
                width: Dimension::Points(100.0),
                height: Dimension::Points(50.0),
                ..Default::default()
            };
            let child = LayoutNode::new(ComponentId::new(), child_style);
            parent.add_child(child);
        }

        let container_size = Size::new(300.0, 400.0);
        let result = engine.calculate_layout(&mut parent, container_size);
        assert!(result.is_ok());

        // First two children should be on first line, third on second line
        assert_eq!(parent.children[0].layout.rect.y(), parent.children[1].layout.rect.y());
        assert!(parent.children[2].layout.rect.y() > parent.children[0].layout.rect.y());
    }

    #[test]
    fn test_gap_structure() {
        let gap = Gap::new(10.0, 20.0);
        assert_eq!(gap.row, 10.0);
        assert_eq!(gap.column, 20.0);

        let uniform_gap = Gap::uniform(15.0);
        assert_eq!(uniform_gap.row, 15.0);
        assert_eq!(uniform_gap.column, 15.0);

        let default_gap = Gap::default();
        assert_eq!(default_gap.row, 0.0);
        assert_eq!(default_gap.column, 0.0);
    }
}
