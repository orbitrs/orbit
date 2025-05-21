// Styling system for the Orbit UI framework

#[cfg(test)]
mod tests;

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

/// Style properties for a UI element
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// Background color
    pub background_color: Option<String>,
    /// Text color
    pub color: Option<String>,
    /// Font family
    pub font_family: Option<String>,
    /// Font size in pixels
    pub font_size: Option<f32>,
    /// Font weight (normal, bold, etc)
    pub font_weight: Option<String>,
    /// Padding in pixels (top, right, bottom, left)
    pub padding: Option<(f32, f32, f32, f32)>,
    /// Margin in pixels (top, right, bottom, left)
    pub margin: Option<(f32, f32, f32, f32)>,
    /// Border width in pixels
    pub border_width: Option<f32>,
    /// Border color
    pub border_color: Option<String>,
    /// Border radius in pixels
    pub border_radius: Option<f32>,
    /// Width in pixels or percentage
    pub width: Option<String>,
    /// Height in pixels or percentage
    pub height: Option<String>,
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
