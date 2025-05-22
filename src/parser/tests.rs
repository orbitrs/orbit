//! Tests for the parser module

#[cfg(test)]
mod tests {
    use crate::parser::OrbitParser;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
<template>
  <div>{{ message }}</div>
</template>

<style>
div {
  color: blue;
}
</style>

<code lang="rust">
use orbitui::prelude::*;

pub struct HelloWorld {
    message: String,
}

impl Component for HelloWorld {
    type Props = HelloWorldProps;
    
    fn new(props: Self::Props) -> Self {
        Self {
            message: props.message,
        }
    }
}

pub struct HelloWorldProps {
    pub message: String,
}

impl Props for HelloWorldProps {}
</code>
"#;

        let result = OrbitParser::parse(content);
        assert!(result.is_ok(), "Failed to parse .orbit file");

        let orbit_file = result.unwrap();
        assert!(
            orbit_file.template.contains("{{ message }}"),
            "Template not parsed correctly"
        );
        assert!(
            orbit_file.style.contains("color: blue"),
            "Style not parsed correctly"
        );
        assert!(
            orbit_file.script.contains("pub struct HelloWorld"),
            "Script not parsed correctly"
        );
    }

    #[test]
    fn test_compilation() {
        let content = r#"
<template>
  <div>{{ message }}</div>
</template>

<style>
div {
  color: blue;
}
</style>

<code lang="rust">
use orbitui::prelude::*;

pub struct HelloWorld {
    message: String,
}

impl Component for HelloWorld {
    type Props = HelloWorldProps;
    
    fn new(props: Self::Props) -> Self {
        Self {
            message: props.message,
        }
    }
}

pub struct HelloWorldProps {
    pub message: String,
}

impl Props for HelloWorldProps {}
</code>
"#;

        let orbit_file = OrbitParser::parse(content).unwrap();
        let result = OrbitParser::compile(&orbit_file);

        assert!(result.is_ok(), "Failed to compile .orbit file");

        let compiled = result.unwrap();
        assert_eq!(
            compiled.name, "HelloWorld",
            "Component name not extracted correctly"
        );
        assert!(
            compiled.code.contains("fn render(&self)"),
            "Render method not generated"
        );
        assert!(
            compiled.styles.contains("color: blue"),
            "Styles not included"
        );
    }
}
