// Enhanced styling system for the Orbit UI framework with CSS-like properties and layout integration

#[cfg(test)]
mod tests;

use crate::layout::{Dimension, EdgeValues, LayoutStyle};
use crate::component::ComponentId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// CSS selector specificity (a, b, c)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32, pub u32);

/// CSS property
#[derive(Debug, Clone, PartialEq)]
pub struct CssProperty {
    /// Name of the property
    pub name: String,
    /// Value of the property
    pub value: String,
}

/// CSS selector
#[derive(Debug, Clone, PartialEq)]
pub struct CssSelector {
    /// Selector text
    pub selector: String,
    /// Properties for this selector
    pub properties: Vec<CssProperty>,
}

/// CSS rule with selector, properties, and metadata
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// The CSS selector(s) for this rule
    pub selectors: Vec<CssSelector>,
    /// Whether this rule is scoped to a component
    pub scoped: bool,
    /// The computed specificity of the selector
    pub specificity: Specificity,
    /// Source order for breaking specificity ties
    pub source_order: usize,
}

/// CSS stylesheet
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    /// Rules in this stylesheet
    pub rules: Vec<StyleRule>,
}

impl StyleRule {
    /// Create a new StyleRule
    pub fn new(selectors: Vec<CssSelector>, scoped: bool, source_order: usize) -> Self {
        // Calculate specificity for the most specific selector
        let specificity = selectors
            .iter()
            .map(|s| Self::calculate_specificity(&s.selector))
            .max()
            .unwrap_or(Specificity(0, 0, 0));

        Self {
            selectors,
            scoped,
            specificity,
            source_order,
        }
    }

    /// Calculate selector specificity (a, b, c)
    /// a = ID selectors
    /// b = Class selectors, attributes, and pseudo-classes
    /// c = Element selectors and pseudo-elements
    fn calculate_specificity(selector: &str) -> Specificity {
        let mut a = 0; // ID selectors
        let mut b = 0; // Class, attribute, pseudo-class
        let mut c = 0; // Element and pseudo-element

        for part in selector.split(' ') {
            // Count ID selectors (#)
            a += part.matches('#').count() as u32;

            // Count class selectors (.), attributes ([]), pseudo-classes (:)
            b += part.matches('.').count() as u32;
            b += part.matches('[').count() as u32;
            b += part.matches(':').count() as u32;
            b -= part.matches("::").count() as u32; // Adjust for pseudo-elements

            // Count element selectors and pseudo-elements (::)
            if !part.starts_with('.') && !part.starts_with('#') && !part.starts_with('[') {
                c += 1;
            }
            c += part.matches("::").count() as u32;
        }

        Specificity(a, b, c)
    }

    /// Apply component scoping to selectors
    pub fn apply_scoping(&mut self, component_id: &str) {
        if self.scoped {
            for selector in &mut self.selectors {
                // Add component scoping class to each selector
                selector.selector = format!(".{} {}", component_id, selector.selector);
                self.specificity = Self::calculate_specificity(&selector.selector);
            }
        }
    }
}

impl Stylesheet {
    /// Create a new empty stylesheet
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule to the stylesheet
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    /// Parse CSS text into a stylesheet
    pub fn parse(css: &str, scoped: bool) -> Result<Self, StyleError> {
        let mut stylesheet = Self::new();
        let mut current_selectors = Vec::new();
        let mut current_properties = Vec::new();
        let mut in_rule = false;
        let mut source_order = 0;

        for line in css.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("/*") {
                continue;
            }

            if line.contains('{') {
                // Start of a rule - parse selectors
                in_rule = true;
                current_selectors = line
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| CssSelector {
                        selector: s.trim().to_string(),
                        properties: Vec::new(),
                    })
                    .collect();
            } else if line.contains('}') {
                // End of a rule - create StyleRule
                in_rule = false;
                if !current_selectors.is_empty() {
                    for selector in &mut current_selectors {
                        selector.properties = current_properties.clone();
                    }
                    let rule = StyleRule::new(current_selectors.clone(), scoped, source_order);
                    stylesheet.add_rule(rule);
                    source_order += 1;
                }
                current_selectors.clear();
                current_properties.clear();
            } else if in_rule && line.contains(':') {
                // Property definition
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    current_properties.push(CssProperty {
                        name: parts[0].trim().to_string(),
                        value: parts[1].trim().trim_end_matches(';').to_string(),
                    });
                }
            }
        }

        Ok(stylesheet)
    }
}

impl std::fmt::Display for Stylesheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for rule in &self.rules {
            for selector in &rule.selectors {
                result.push_str(&selector.selector);
                if rule.scoped {
                    result.push_str(" /* scoped */");
                }
                result.push_str(" {\n");

                for property in &selector.properties {
                    result.push_str("  ");
                    result.push_str(&property.name);
                    result.push_str(": ");
                    result.push_str(&property.value);
                    result.push_str(";\n");
                }

                result.push_str("}\n\n");
            }
        }

        write!(f, "{}", result)
    }
}

/// Enhanced style properties for a UI element with CSS-like properties
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    // Visual properties
    pub background_color: Option<Color>,
    pub color: Option<Color>,
    pub opacity: Option<f32>,
    pub visibility: Option<Visibility>,
    
    // Border properties
    pub border_width: Option<EdgeValues>,
    pub border_color: Option<EdgeColors>,
    pub border_style: Option<BorderStyle>,
    pub border_radius: Option<BorderRadius>,
    
    // Font and text properties
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecoration>,
    pub text_transform: Option<TextTransform>,
    
    // Layout integration (connects to layout engine)
    pub layout_style: Option<LayoutStyle>,
    
    // Transform properties
    pub transform: Option<Transform>,
    pub transform_origin: Option<Point2D>,
    
    // Transition properties (for animation integration)
    pub transition_property: Option<Vec<String>>,
    pub transition_duration: Option<f32>,
    pub transition_timing_function: Option<TimingFunction>,
    pub transition_delay: Option<f32>,
    
    // Shadow properties
    pub box_shadow: Option<Vec<BoxShadow>>,
    pub text_shadow: Option<Vec<TextShadow>>,
    
    // Advanced properties
    pub filter: Option<Vec<Filter>>,
    pub backdrop_filter: Option<Vec<Filter>>,
    pub z_index: Option<i32>,
    pub cursor: Option<CursorType>,
    
    // Performance and caching
    pub computed_hash: Option<u64>,
    pub is_dirty: bool,
}

/// Color representation supporting multiple formats
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgba(f32, f32, f32, f32),
    Hex(String),
    Named(String),
    Hsl(f32, f32, f32, f32),
    CurrentColor,
    Transparent,
}

/// Visibility values
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

/// Border style options
#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// Border radius configuration
#[derive(Debug, Clone, PartialEq)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

/// Edge-specific colors for borders
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeColors {
    pub top: Color,
    pub right: Color,
    pub bottom: Color,
    pub left: Color,
}

/// Font weight values
#[derive(Debug, Clone, PartialEq)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    Normal,     // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
    Numeric(u16),
}

/// Font style values
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(f32), // angle in degrees
}

/// Text alignment
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

/// Text decoration
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
    Blink,
}

/// Text transform
#[derive(Debug, Clone, PartialEq)]
pub enum TextTransform {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

/// 2D point for transform origins
#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

/// CSS transform operations
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    None,
    Matrix(f32, f32, f32, f32, f32, f32),
    Translate(f32, f32),
    TranslateX(f32),
    TranslateY(f32),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32), // degrees
    SkewX(f32),  // degrees
    SkewY(f32),  // degrees
    Multiple(Vec<Transform>),
}

/// Timing functions for animations
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
    Steps(u32, StepPosition),
}

/// Step position for stepped timing functions
#[derive(Debug, Clone, PartialEq)]
pub enum StepPosition {
    Start,
    End,
}

/// Box shadow definition
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: Color,
    pub inset: bool,
}

/// Text shadow definition
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

/// CSS filter functions
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    DropShadow(f32, f32, f32, Color),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
}

/// Cursor types
#[derive(Debug, Clone, PartialEq)]
pub enum CursorType {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
}

/// Style engine for efficient style computation and caching
#[derive(Debug)]
pub struct StyleEngine {
    /// Cache of computed styles keyed by hash
    computed_cache: HashMap<u64, ComputedStyle>,
    /// Style inheritance tree
    inheritance_tree: HashMap<ComponentId, ComponentId>,
    /// Global style rules
    global_rules: Vec<StyleRule>,
    /// Component-scoped style rules
    component_rules: HashMap<ComponentId, Vec<StyleRule>>,
    /// Performance statistics
    stats: StyleStats,
    /// Cache hit counter for performance monitoring
    cache_hit_counter: AtomicU64,
}

/// Computed style represents the final resolved style values
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    /// Final computed style properties
    pub style: Style,
    /// Layout style for layout engine integration
    pub layout_style: LayoutStyle,
    /// Hash of the computed result for caching
    pub hash: u64,
    /// Whether this style is animatable
    pub is_animatable: bool,
    /// Timestamp when computed
    pub computed_at: std::time::Instant,
}

/// Performance statistics for the style engine
#[derive(Debug, Default, Clone)]
pub struct StyleStats {
    /// Number of style computations performed
    pub computations: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Total time spent computing styles (ms)
    pub computation_time_ms: f32,
    /// Average computation time per style (ms)
    pub avg_computation_time_ms: f32,
    /// Number of layout style conversions
    pub layout_conversions: u64,
    /// Number of style inheritance operations
    pub inheritance_operations: u64,
    /// Current cache size
    pub cache_size: usize,
}

/// Style resolution context for computing styles
#[derive(Debug)]
pub struct StyleContext {
    /// Viewport dimensions
    pub viewport_width: f32,
    pub viewport_height: f32,
    /// Device pixel ratio
    pub device_pixel_ratio: f32,
    /// Inherited styles from parent
    pub inherited_style: Option<ComputedStyle>,
    /// Component ID for scoped styles
    pub component_id: Option<ComponentId>,
    /// Current theme variables
    pub theme_variables: HashMap<String, String>,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply another style on top of this one
    pub fn merge(&mut self, other: &Style) {
        if let Some(color) = &other.color {
            self.color = Some(color.clone());
        }
        if let Some(bg) = &other.background_color {
            self.background_color = Some(bg.clone());
        }
        // Merge other properties...
    }
}

/// Errors that can occur in styling operations
#[derive(Debug, thiserror::Error)]
pub enum StyleError {
    #[error("Error parsing CSS: {0}")]
    ParseError(String),
}
