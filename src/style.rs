// Styling system for the Orbit UI framework

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

/// CSS stylesheet
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    /// Selectors in this stylesheet
    pub selectors: Vec<CssSelector>,
}

impl Stylesheet {
    /// Create a new empty stylesheet
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a selector to the stylesheet
    pub fn add_selector(&mut self, selector: CssSelector) {
        self.selectors.push(selector);
    }

    /// Parse CSS text into a stylesheet
    pub fn parse(_css: &str) -> Result<Self, StyleError> {
        let stylesheet = Self::new();

        // Basic CSS parser - will be expanded
        // This is a placeholder implementation

        Ok(stylesheet)
    }

    /// Convert the stylesheet to CSS text
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for selector in &self.selectors {
            result.push_str(&selector.selector);
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

        result
    }
}

/// Errors that can occur in styling operations
#[derive(Debug, thiserror::Error)]
pub enum StyleError {
    #[error("Error parsing CSS: {0}")]
    ParseError(String),
}
