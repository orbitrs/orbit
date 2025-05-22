//! Main parser module for .orbit files

mod ast;
mod template;
mod tokenizer;

pub use ast::{OrbitAst, TemplateNode};

use std::fs;
use std::path::Path;

/// Main parser for .orbit files
#[derive(Default)]
pub struct OrbitParser;

impl OrbitParser {
    /// Parse an .orbit file into an AST
    pub fn parse(content: &str) -> Result<OrbitAst, String> {
        // Split into sections first
        let sections = Self::split_sections(content)?;

        // Parse each section
        let template_node = template::TemplateParser::new(&sections.template).parse()?;

        // TODO: Implement style parser
        let style_node = ast::StyleNode {
            rules: Vec::new(),
            scoped: false,
        };

        // TODO: Implement script parser
        let script_node = ast::ScriptNode {
            imports: Vec::new(),
            component_name: String::new(),
            props: Vec::new(),
            state: Vec::new(),
            methods: Vec::new(),
            lifecycle: Vec::new(),
        };

        Ok(OrbitAst::new(template_node, style_node, script_node))
    }

    /// Parse an .orbit file from a file path
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<OrbitAst, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Self::parse(&content)
    }

    /// Split an .orbit file into its constituent sections
    fn split_sections(content: &str) -> Result<Sections, String> {
        let mut template = String::new();
        let mut style = String::new();
        let mut script = String::new();

        let mut current_section = None;
        let mut current_content = String::new();

        for line in content.lines() {
            match line.trim() {
                "<template>" => {
                    current_section = Some(Section::Template);
                    continue;
                }
                "</template>" => {
                    template = current_content.clone();
                    current_content.clear();
                    current_section = None;
                    continue;
                }
                "<style>" => {
                    current_section = Some(Section::Style);
                    continue;
                }
                "</style>" => {
                    style = current_content.clone();
                    current_content.clear();
                    current_section = None;
                    continue;
                }
                "<script>" | "<code>" | "<code lang=\"rust\">" => {
                    current_section = Some(Section::Script);
                    continue;
                }
                "</script>" | "</code>" => {
                    script = current_content.clone();
                    current_content.clear();
                    current_section = None;
                    continue;
                }
                _ => {}
            }

            if let Some(_section) = current_section {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if template.is_empty() {
            return Err("Missing <template> section".to_string());
        }

        Ok(Sections {
            template,
            style,
            script,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Section {
    Template,
    Style,
    Script,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Sections {
    template: String,
    style: String,
    script: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_component() {
        let content = r#"
<template>
    <div class="greeting">
        <h1>Hello, {{ name }}!</h1>
        <button @click="increment">Count: {{ count }}</button>
    </div>
</template>

<style>
.greeting {
    font-family: Arial, sans-serif;
    padding: 20px;
}
</style>

<script>
use orbitui::prelude::*;

pub struct Greeting {
    name: String,
    count: i32,
}

impl Component for Greeting {
    fn new() -> Self {
        Self {
            name: String::from("World"),
            count: 0,
        }
    }
}
</script>
"#;

        let ast = OrbitParser::parse(content).unwrap();

        // Verify template structure
        match ast.template {
            TemplateNode::Element {
                tag,
                attributes,
                events: _,
                children,
            } => {
                assert_eq!(tag, "div");
                assert!(attributes.contains_key("class"));
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected element node"),
        }
    }
}
