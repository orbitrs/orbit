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
#[derive(Debug, Clone, PartialEq, Hash)]
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

impl StyleEngine {
    /// Create a new style engine
    pub fn new() -> Self {
        Self {
            computed_cache: HashMap::new(),
            inheritance_tree: HashMap::new(),
            global_rules: Vec::new(),
            component_rules: HashMap::new(),
            stats: StyleStats::default(),
            cache_hit_counter: AtomicU64::new(0),
        }
    }

    /// Add global CSS rules that apply to all components
    pub fn add_global_rules(&mut self, rules: Vec<StyleRule>) {
        self.global_rules.extend(rules);
    }

    /// Add component-scoped CSS rules
    pub fn add_component_rules(&mut self, component_id: ComponentId, rules: Vec<StyleRule>) {
        self.component_rules.entry(component_id).or_default().extend(rules);
    }

    /// Compute the final style for a component with context
    pub fn compute_style(
        &mut self,
        component_id: ComponentId,
        base_style: &Style,
        context: &StyleContext,
    ) -> Result<ComputedStyle, StyleError> {
        let start_time = std::time::Instant::now();

        // Generate cache key from component ID, style, and context
        let cache_key = self.generate_cache_key(component_id, base_style, context);

        // Check cache first
        if let Some(computed) = self.computed_cache.get(&cache_key) {
            self.cache_hit_counter.fetch_add(1, Ordering::Relaxed);
            self.stats.cache_hits += 1;
            return Ok(computed.clone());
        }

        // Compute new style
        let mut computed_style = base_style.clone();

        // Apply inheritance from parent
        if let Some(inherited) = &context.inherited_style {
            self.apply_inheritance(&mut computed_style, &inherited.style);
        }

        // Apply global CSS rules
        self.apply_css_rules(&mut computed_style, &self.global_rules.clone(), context)?;

        // Apply component-scoped rules
        if let Some(component_rules) = self.component_rules.get(&component_id) {
            self.apply_css_rules(&mut computed_style, component_rules, context)?;
        }

        // Convert to layout style
        let layout_style = self.style_to_layout_style(&computed_style, context)?;

        // Create computed style result
        let computed = ComputedStyle {
            style: computed_style.clone(),
            layout_style,
            hash: cache_key,
            is_animatable: self.is_style_animatable(&computed_style),
            computed_at: std::time::Instant::now(),
        };

        // Cache the result
        self.computed_cache.insert(cache_key, computed.clone());

        // Update performance statistics
        let computation_time = start_time.elapsed().as_secs_f32() * 1000.0;
        self.stats.computations += 1;
        self.stats.computation_time_ms += computation_time;
        self.stats.avg_computation_time_ms = 
            self.stats.computation_time_ms / self.stats.computations as f32;
        self.stats.cache_size = self.computed_cache.len();

        Ok(computed)
    }

    /// Generate cache key for style computation
    fn generate_cache_key(
        &self,
        component_id: ComponentId,
        style: &Style,
        context: &StyleContext,
    ) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        component_id.hash(&mut hasher);
        
        // Hash style properties that affect computation
        style.color.hash(&mut hasher);
        style.background_color.hash(&mut hasher);
        style.font_size.map(|f| f.to_bits()).hash(&mut hasher);
        style.opacity.map(|f| f.to_bits()).hash(&mut hasher);
        
        // Hash context
        context.viewport_width.to_bits().hash(&mut hasher);
        context.viewport_height.to_bits().hash(&mut hasher);
        context.device_pixel_ratio.to_bits().hash(&mut hasher);
        
        hasher.finish()
    }

    /// Apply CSS inheritance rules
    fn apply_inheritance(&mut self, style: &mut Style, parent_style: &Style) {
        // Inherit font properties if not explicitly set
        if style.font_family.is_none() {
            style.font_family = parent_style.font_family.clone();
        }
        if style.font_size.is_none() {
            style.font_size = parent_style.font_size;
        }
        if style.font_weight.is_none() {
            style.font_weight = parent_style.font_weight.clone();
        }
        if style.color.is_none() {
            style.color = parent_style.color.clone();
        }
        if style.line_height.is_none() {
            style.line_height = parent_style.line_height;
        }
        if style.letter_spacing.is_none() {
            style.letter_spacing = parent_style.letter_spacing;
        }
        if style.text_align.is_none() {
            style.text_align = parent_style.text_align.clone();
        }

        self.stats.inheritance_operations += 1;
    }

    /// Apply CSS rules to a style
    fn apply_css_rules(
        &self,
        style: &mut Style,
        rules: &[StyleRule],
        context: &StyleContext,
    ) -> Result<(), StyleError> {
        // Sort rules by specificity and source order
        let mut sorted_rules = rules.to_vec();
        sorted_rules.sort_by(|a, b| {
            a.specificity.cmp(&b.specificity)
                .then(a.source_order.cmp(&b.source_order))
        });

        // Apply rules in order
        for rule in sorted_rules {
            for selector in &rule.selectors {
                for property in &selector.properties {
                    self.apply_css_property(style, property, context)?;
                }
            }
        }

        Ok(())
    }

    /// Apply a single CSS property to a style
    fn apply_css_property(
        &self,
        style: &mut Style,
        property: &CssProperty,
        _context: &StyleContext,
    ) -> Result<(), StyleError> {
        match property.name.as_str() {
            "color" => {
                style.color = Some(self.parse_color(&property.value)?);
            }
            "background-color" => {
                style.background_color = Some(self.parse_color(&property.value)?);
            }
            "opacity" => {
                style.opacity = property.value.parse().ok();
            }
            "font-size" => {
                style.font_size = self.parse_font_size(&property.value);
            }
            "font-weight" => {
                style.font_weight = Some(self.parse_font_weight(&property.value)?);
            }
            "font-family" => {
                style.font_family = Some(property.value.clone());
            }
            "text-align" => {
                style.text_align = Some(self.parse_text_align(&property.value)?);
            }
            "border-radius" => {
                style.border_radius = Some(self.parse_border_radius(&property.value)?);
            }
            "z-index" => {
                style.z_index = property.value.parse().ok();
            }
            _ => {
                // Unknown property - could log warning in debug mode
            }
        }

        Ok(())
    }

    /// Convert Style to LayoutStyle for layout engine integration
    fn style_to_layout_style(
        &mut self,
        style: &Style,
        context: &StyleContext,
    ) -> Result<LayoutStyle, StyleError> {
        let mut layout_style = LayoutStyle::default();

        // Copy layout-related properties from existing layout_style if present
        if let Some(existing_layout) = &style.layout_style {
            layout_style = existing_layout.clone();
        }

        // Apply style properties that affect layout
        if let Some(opacity) = style.opacity {
            // Opacity affects layout in some cases (e.g., visibility)
            if opacity == 0.0 {
                // Could set display: none equivalent
            }
        }

        // Handle border width affecting layout
        if let Some(border_width) = &style.border_width {
            // Border width affects content area
            layout_style.border = *border_width;
        }

        // Handle transforms that affect layout bounds
        if let Some(_transform) = &style.transform {
            // Transforms can affect layout bounds
            // Implementation depends on transform type
        }

        // Apply responsive sizing based on viewport
        self.apply_responsive_sizing(&mut layout_style, context);

        self.stats.layout_conversions += 1;
        Ok(layout_style)
    }

    /// Apply responsive sizing rules
    fn apply_responsive_sizing(&self, layout_style: &mut LayoutStyle, context: &StyleContext) {
        // Apply responsive breakpoints and scaling
        let scale_factor = context.device_pixel_ratio;
        
        // Scale dimensions if needed
        if let Dimension::Points(width) = layout_style.size.width {
            layout_style.size.width = Dimension::Points(width * scale_factor);
        }
        if let Dimension::Points(height) = layout_style.size.height {
            layout_style.size.height = Dimension::Points(height * scale_factor);
        }
    }

    /// Check if a style contains animatable properties
    fn is_style_animatable(&self, style: &Style) -> bool {
        style.opacity.is_some()
            || style.background_color.is_some()
            || style.color.is_some()
            || style.transform.is_some()
            || style.border_radius.is_some()
            || style.font_size.is_some()
            || style.transition_duration.is_some()
    }

    /// Parse color from CSS value
    fn parse_color(&self, value: &str) -> Result<Color, StyleError> {
        let value = value.trim();
        
        if value.starts_with('#') {
            Ok(Color::Hex(value.to_string()))
        } else if value.starts_with("rgb(") {
            // Parse rgb(r, g, b) format
            let inner = value.trim_start_matches("rgb(").trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 3 {
                let r = parts[0].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid red value".to_string()))? as f32 / 255.0;
                let g = parts[1].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid green value".to_string()))? as f32 / 255.0;
                let b = parts[2].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid blue value".to_string()))? as f32 / 255.0;
                Ok(Color::Rgba(r, g, b, 1.0))
            } else {
                Err(StyleError::ParseError("Invalid RGB format".to_string()))
            }
        } else if value.starts_with("rgba(") {
            // Parse rgba(r, g, b, a) format
            let inner = value.trim_start_matches("rgba(").trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 4 {
                let r = parts[0].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid red value".to_string()))? as f32 / 255.0;
                let g = parts[1].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid green value".to_string()))? as f32 / 255.0;
                let b = parts[2].trim().parse::<u8>().map_err(|_| StyleError::ParseError("Invalid blue value".to_string()))? as f32 / 255.0;
                let a = parts[3].trim().parse::<f32>().map_err(|_| StyleError::ParseError("Invalid alpha value".to_string()))?;
                Ok(Color::Rgba(r, g, b, a))
            } else {
                Err(StyleError::ParseError("Invalid RGBA format".to_string()))
            }
        } else if value == "transparent" {
            Ok(Color::Transparent)
        } else if value == "currentColor" {
            Ok(Color::CurrentColor)
        } else {
            Ok(Color::Named(value.to_string()))
        }
    }

    /// Parse font size from CSS value
    fn parse_font_size(&self, value: &str) -> Option<f32> {
        let value = value.trim();
        if value.ends_with("px") {
            value.trim_end_matches("px").parse().ok()
        } else if value.ends_with("pt") {
            value.trim_end_matches("pt").parse::<f32>().map(|pt| pt * 1.333).ok()
        } else if value.ends_with("em") {
            value.trim_end_matches("em").parse::<f32>().map(|em| em * 16.0).ok()
        } else {
            value.parse().ok()
        }
    }

    /// Parse font weight from CSS value
    fn parse_font_weight(&self, value: &str) -> Result<FontWeight, StyleError> {
        match value.trim() {
            "thin" | "100" => Ok(FontWeight::Thin),
            "extra-light" | "200" => Ok(FontWeight::ExtraLight),
            "light" | "300" => Ok(FontWeight::Light),
            "normal" | "400" => Ok(FontWeight::Normal),
            "medium" | "500" => Ok(FontWeight::Medium),
            "semi-bold" | "600" => Ok(FontWeight::SemiBold),
            "bold" | "700" => Ok(FontWeight::Bold),
            "extra-bold" | "800" => Ok(FontWeight::ExtraBold),
            "black" | "900" => Ok(FontWeight::Black),
            _ => {
                if let Ok(numeric) = value.parse::<u16>() {
                    Ok(FontWeight::Numeric(numeric))
                } else {
                    Err(StyleError::ParseError(format!("Invalid font weight: {}", value)))
                }
            }
        }
    }

    /// Parse text alignment from CSS value
    fn parse_text_align(&self, value: &str) -> Result<TextAlign, StyleError> {
        match value.trim() {
            "left" => Ok(TextAlign::Left),
            "right" => Ok(TextAlign::Right),
            "center" => Ok(TextAlign::Center),
            "justify" => Ok(TextAlign::Justify),
            "start" => Ok(TextAlign::Start),
            "end" => Ok(TextAlign::End),
            _ => Err(StyleError::ParseError(format!("Invalid text align: {}", value))),
        }
    }

    /// Parse border radius from CSS value
    fn parse_border_radius(&self, value: &str) -> Result<BorderRadius, StyleError> {
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => {
                let radius = self.parse_length(parts[0])?;
                Ok(BorderRadius {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                })
            }
            2 => {
                let radius1 = self.parse_length(parts[0])?;
                let radius2 = self.parse_length(parts[1])?;
                Ok(BorderRadius {
                    top_left: radius1,
                    top_right: radius2,
                    bottom_right: radius1,
                    bottom_left: radius2,
                })
            }
            4 => {
                Ok(BorderRadius {
                    top_left: self.parse_length(parts[0])?,
                    top_right: self.parse_length(parts[1])?,
                    bottom_right: self.parse_length(parts[2])?,
                    bottom_left: self.parse_length(parts[3])?,
                })
            }
            _ => Err(StyleError::ParseError("Invalid border radius format".to_string())),
        }
    }

    /// Parse length value (px, pt, em, etc.)
    fn parse_length(&self, value: &str) -> Result<f32, StyleError> {
        let value = value.trim();
        if value.ends_with("px") {
            value.trim_end_matches("px").parse()
                .map_err(|_| StyleError::ParseError("Invalid pixel value".to_string()))
        } else if value.ends_with("pt") {
            value.trim_end_matches("pt").parse::<f32>()
                .map(|pt| pt * 1.333)
                .map_err(|_| StyleError::ParseError("Invalid point value".to_string()))
        } else if value.ends_with("em") {
            value.trim_end_matches("em").parse::<f32>()
                .map(|em| em * 16.0)
                .map_err(|_| StyleError::ParseError("Invalid em value".to_string()))
        } else {
            value.parse()
                .map_err(|_| StyleError::ParseError("Invalid length value".to_string()))
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> StyleStats {
        let mut stats = self.stats.clone();
        stats.cache_hits = self.cache_hit_counter.load(Ordering::Relaxed);
        stats.cache_size = self.computed_cache.len();
        stats
    }

    /// Clear style cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.computed_cache.clear();
        self.stats.cache_size = 0;
    }

    /// Set parent-child inheritance relationship
    pub fn set_inheritance(&mut self, child_id: ComponentId, parent_id: ComponentId) {
        self.inheritance_tree.insert(child_id, parent_id);
    }
}

impl Default for StyleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StyleContext {
    fn default() -> Self {
        Self {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            device_pixel_ratio: 1.0,
            inherited_style: None,
            component_id: None,
            theme_variables: HashMap::new(),
            performance_monitoring: false,
        }
    }
}

impl ComputedStyle {
    /// Check if this computed style is expired (for cache invalidation)
    pub fn is_expired(&self, max_age_ms: u128) -> bool {
        self.computed_at.elapsed().as_millis() > max_age_ms
    }

    /// Get a specific property value from the computed style
    pub fn get_property(&self, name: &str) -> Option<String> {
        match name {
            "color" => self.style.color.as_ref().map(|c| format!("{:?}", c)),
            "background-color" => self.style.background_color.as_ref().map(|c| format!("{:?}", c)),
            "opacity" => self.style.opacity.map(|o| o.to_string()),
            "font-size" => self.style.font_size.map(|f| format!("{}px", f)),
            "font-weight" => self.style.font_weight.as_ref().map(|w| format!("{:?}", w)),
            _ => None,
        }
    }
}
