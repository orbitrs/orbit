// Parser module for .orbit files

use crate::Error;
use std::collections::HashMap;

/// Parser for .orbit files
pub struct OrbitParser;

#[cfg(test)]
mod tests;

impl OrbitParser {
    /// Parse an .orbit file into its constituent parts
    pub fn parse(content: &str) -> Result<OrbitFile, Error> {
        // Extract the sections
        let template = extract_template(content)?;
        let style = extract_style(content)?;
        let script = extract_script(content)?;

        // Parse template attributes
        let template_attrs = parse_template_attributes(&template.content);

        // Parse style attributes
        let style_attrs = parse_style_attributes(&style.content);

        // Create the OrbitFile
        Ok(OrbitFile {
            template: template.content,
            template_attrs,
            style: style.content,
            style_attrs,
            script: script.content,
            file_path: None,
        })
    }

    /// Parse an .orbit file with a known file path
    pub fn parse_file(content: &str, file_path: &str) -> Result<OrbitFile, Error> {
        let mut orbit_file = Self::parse(content)?;
        orbit_file.file_path = Some(file_path.to_string());
        Ok(orbit_file)
    }

    /// Compile an .orbit file to Rust code
    pub fn compile(file: &OrbitFile) -> Result<CompiledComponent, Error> {
        // Extract the component name and struct definition from the script
        let (component_name, component_struct) = extract_component_name_and_struct(&file.script)
            .ok_or_else(|| Error::Parser("Could not extract component struct".to_string()))?;

        let mut code = String::new();

        // Add standard imports
        code.push_str("use orbit::prelude::*;\n");

        // Include imports from the script
        if let Some(imports) = extract_imports_from_script(&file.script) {
            code.push_str(&imports);
            code.push_str("\n\n");
        }

        // Extract component struct definition from the script
        code.push_str(&component_struct);
        code.push_str("\n\n");

        // Generate render method from template
        let render_method = generate_render_method_from_template(&file.template, &component_name);
        code.push_str(&render_method);
        code.push_str("\n\n");

        // Include component methods from script
        if let Some(methods) = extract_component_methods(&file.script) {
            code.push_str(&methods);
            code.push_str("\n");
        }

        // Determine if styles are scoped
        let scoped_styles = file
            .style_attrs
            .get("scoped")
            .map_or(true, |v| v != "false");

        Ok(CompiledComponent {
            code,
            name: component_name,
            file_path: file.file_path.clone(),
            styles: file.style.clone(),
            scoped_styles,
        })
    }
}

/// Section of an .orbit file with content and metadata
#[derive(Debug, Clone)]
struct Section {
    /// The content of the section
    content: String,
    /// The start position in the original file
    #[allow(dead_code)]
    start_pos: usize,
    /// The end position in the original file
    #[allow(dead_code)]
    end_pos: usize,
}

/// Represents a parsed .orbit file
#[derive(Debug, Clone)]
pub struct OrbitFile {
    /// The HTML/XML template
    pub template: String,
    /// Attributes from the template tag
    pub template_attrs: HashMap<String, String>,
    /// The CSS style
    pub style: String,
    /// Attributes from the style tag
    pub style_attrs: HashMap<String, String>,
    /// The Rust script
    pub script: String,
    /// The file path (if known)
    pub file_path: Option<String>,
}

/// Compiled Orbit component
#[derive(Debug)]
pub struct CompiledComponent {
    /// Generated Rust code
    pub code: String,
    /// Component name
    pub name: String,
    /// Original file path
    pub file_path: Option<String>,
    /// CSS styles
    pub styles: String,
    /// Whether the styles are scoped
    pub scoped_styles: bool,
}

// Helper functions for extracting parts of the .orbit file

/// Extract template section
fn extract_template(content: &str) -> Result<Section, Error> {
    extract_section(content, "template")
}

/// Extract style section
fn extract_style(content: &str) -> Result<Section, Error> {
    extract_section(content, "style")
}

/// Extract script section
fn extract_script(content: &str) -> Result<Section, Error> {
    extract_section(content, "script")
}

/// Extract a section from the content
fn extract_section(content: &str, section_name: &str) -> Result<Section, Error> {
    let start_tag = format!("<{}>", section_name);
    let end_tag = format!("</{}>", section_name);

    let start_pos = content
        .find(&start_tag)
        .ok_or_else(|| Error::Parser(format!("<{}> tag not found", section_name)))?;

    let content_start = start_pos + start_tag.len();

    let end_pos = content[content_start..]
        .find(&end_tag)
        .ok_or_else(|| Error::Parser(format!("Unclosed <{}> tag", section_name)))?;

    let section_content = &content[content_start..content_start + end_pos];

    Ok(Section {
        content: section_content.trim().to_string(),
        start_pos,
        end_pos: content_start + end_pos + end_tag.len(),
    })
}

/// Parse attributes from the template tag
fn parse_template_attributes(_template: &str) -> HashMap<String, String> {
    HashMap::new() // Placeholder
}

/// Parse attributes from the style tag
fn parse_style_attributes(style: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();

    // Check if style scoping is disabled
    if style.contains("scoped=\"false\"") || style.contains("scoped='false'") {
        attrs.insert("scoped".to_string(), "false".to_string());
    } else {
        attrs.insert("scoped".to_string(), "true".to_string());
    }

    attrs
}

/// Extract imports from script
fn extract_imports_from_script(script: &str) -> Option<String> {
    let mut imports = Vec::new();

    for line in script.lines() {
        if line.starts_with("use ") {
            imports.push(line.to_string());
        }
    }

    if imports.is_empty() {
        None
    } else {
        Some(imports.join("\n"))
    }
}

/// Extract component struct definition from script
#[allow(dead_code)]
fn extract_component_definition(script: &str) -> Option<String> {
    // This is a simplified implementation
    // In a real implementation, we would use a proper Rust parser

    if let Some(struct_pos) = script.find("pub struct ") {
        if let Some(_impl_pos) = script[struct_pos..].find("impl Component") {
            let end_of_impl = script[struct_pos..].find("\n}\n\nimpl ");

            if let Some(end_pos) = end_of_impl {
                return Some(script[struct_pos..struct_pos + end_pos + 3].to_string());
            }
        }
    }

    None
}

/// Extract the rest of the script (methods, etc.)
#[allow(dead_code)]
fn extract_rest_of_script(script: &str) -> Option<String> {
    if let Some(pos) = script.find("impl Component") {
        if let Some(end_pos) = script[pos..].find("\n}\n\n") {
            let impl_methods_pos = pos + end_pos + 3;

            if impl_methods_pos < script.len() {
                return Some(script[impl_methods_pos..].to_string());
            }
        }
    }

    None
}

/// Generate render method from template
#[allow(dead_code)]
fn generate_render_method(_file: &OrbitFile) -> String {
    // In a real implementation, this would parse the template and generate
    // the Rust code to implement the render method

    String::from(
        r#"    fn render(&self) -> String {
        // This method is auto-generated from the template
        // It will create HTML based on the component's state
        format!(
            "<div class=\"component\">{}</div>",
            &self.to_string()
        )
    }"#,
    )
}

/// Extract component name and struct from script
fn extract_component_name_and_struct(script: &str) -> Option<(String, String)> {
    // This is a simplified implementation
    // In a real implementation, we would use a proper Rust parser

    if let Some(struct_pos) = script.find("pub struct ") {
        let struct_start = struct_pos + "pub struct ".len();
        let struct_name_end = script[struct_start..].find(" {")?;
        let component_name = script[struct_start..struct_start + struct_name_end]
            .trim()
            .to_string();

        if let Some(impl_pos) = script[struct_pos..].find("impl Component for ") {
            let impl_start = struct_pos + impl_pos;
            let end_of_impl = script[impl_start..].find("\n}\n\nimpl ")?;

            let struct_definition = script[struct_pos..impl_start + end_of_impl + 3].to_string();
            return Some((component_name, struct_definition));
        }
    }

    None
}

/// Extract component methods
fn extract_component_methods(script: &str) -> Option<String> {
    if let Some(pos) = script.find("impl Component for ") {
        if let Some(impl_pos) = script[pos..].find("impl ") {
            let methods_pos = pos + impl_pos;

            // Find the end of the methods section
            if let Some(end_pos) = script[methods_pos..].rfind("}\n") {
                return Some(script[methods_pos..methods_pos + end_pos + 2].to_string());
            } else if methods_pos < script.len() {
                return Some(script[methods_pos..].to_string());
            }
        }
    }

    None
}

/// Generate render method from template
fn generate_render_method_from_template(template: &str, component_name: &str) -> String {
    // This is a more advanced version that processes the template and generates
    // better Rust code for rendering

    // Process template to handle {{ expressions }}
    let processed_template = process_template_expressions(template);

    // Escape Rust string literals
    let escaped_template = processed_template.replace("\"", "\\\"");

    // Format the render method
    format!(
        r#"    fn render(&self) -> String {{
        // Auto-generated render method for {}
        format!("{}",
            // Component properties available in template:
{}
        )
    }}"#,
        component_name, escaped_template, "            // self.property_name"
    )
}

/// Process template expressions like {{ var }}
fn process_template_expressions(template: &str) -> String {
    let mut result = String::new();
    let mut current_pos = 0;

    while let Some(open_pos) = template[current_pos..].find("{{") {
        let abs_open_pos = current_pos + open_pos;

        // Add the text before the {{
        result.push_str(&template[current_pos..abs_open_pos]);

        // Find the closing }}
        if let Some(close_pos) = template[abs_open_pos..].find("}}") {
            let abs_close_pos = abs_open_pos + close_pos;

            // Extract the expression
            let expression = template[abs_open_pos + 2..abs_close_pos].trim();

            // Replace with Rust format expression
            result.push_str(&format!("{{{}}}", expression));

            // Move past the closing }}
            current_pos = abs_close_pos + 2;
        } else {
            // No closing }}, just add the {{ and continue
            result.push_str("{{");
            current_pos = abs_open_pos + 2;
        }
    }

    // Add any remaining text
    result.push_str(&template[current_pos..]);

    result
}
