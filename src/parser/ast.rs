//! Abstract Syntax Tree definitions for .orbit files

use std::collections::HashMap;

/// Represents a parsed .orbit file
#[derive(Debug, Clone)]
pub struct OrbitAst {
    pub template: TemplateNode,
    pub style: StyleNode,
    pub script: ScriptNode,
}

/// Represents a node in the template tree
#[derive(Debug, Clone)]
pub enum TemplateNode {
    Element {
        tag: String,
        attributes: HashMap<String, AttributeValue>,
        events: HashMap<String, String>,
        children: Vec<TemplateNode>,
    },
    Expression(String),
    Text(String),
}

/// Represents an attribute value that can be either static or dynamic
#[derive(Debug, Clone)]
pub enum AttributeValue {
    Static(String),
    Dynamic(String), // Expression inside {{ }}
}

/// Represents the style section
#[derive(Debug, Clone)]
pub struct StyleNode {
    pub rules: Vec<StyleRule>,
    pub scoped: bool,
}

/// Represents a CSS rule
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: String,
    pub declarations: HashMap<String, String>,
}

/// Represents the script section
#[derive(Debug, Clone)]
pub struct ScriptNode {
    pub imports: Vec<String>,
    pub component_name: String,
    pub props: Vec<PropDefinition>,
    pub state: Vec<StateDefinition>,
    pub methods: Vec<MethodDefinition>,
    pub lifecycle: Vec<LifecycleHook>,
}

/// Represents a component property definition
#[derive(Debug, Clone)]
pub struct PropDefinition {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub default: Option<String>,
}

/// Represents a state field definition
#[derive(Debug, Clone)]
pub struct StateDefinition {
    pub name: String,
    pub ty: String,
    pub initial: Option<String>,
}

/// Represents a method definition
#[derive(Debug, Clone)]
pub struct MethodDefinition {
    pub name: String,
    pub args: Vec<(String, String)>, // (name, type)
    pub return_type: Option<String>,
    pub body: String,
}

/// Represents a lifecycle hook
#[derive(Debug, Clone)]
pub struct LifecycleHook {
    pub hook_type: LifecycleType,
    pub body: String,
}

/// Types of lifecycle hooks
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleType {
    Created,
    Mounted,
    Updated,
    Destroyed,
}

impl OrbitAst {
    /// Create a new AST from parsed sections
    pub fn new(template: TemplateNode, style: StyleNode, script: ScriptNode) -> Self {
        Self {
            template,
            style,
            script,
        }
    }
}
