//! Parser for template sections of .orbit files

use super::{
    ast::{AttributeValue, TemplateNode},
    tokenizer::{Token, Tokenizer},
};
use std::collections::HashMap;

/// Parses template sections in .orbit files
pub struct TemplateParser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> TemplateParser<'a> {
    /// Create a new template parser
    pub fn new(input: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }

    /// Parse the template section into an AST
    pub fn parse(&mut self) -> Result<TemplateNode, String> {
        match self.tokenizer.next_token() {
            Token::OpenTag(tag) => self.parse_element(tag),
            token => Err(format!("Expected opening tag, got {:?}", token)),
        }
    }

    /// Parse an element node
    fn parse_element(&mut self, tag: String) -> Result<TemplateNode, String> {
        let mut attributes = HashMap::new();
        let mut events = HashMap::new();
        let mut children = Vec::new();

        loop {
            match self.tokenizer.next_token() {
                Token::AttrName(name) => match self.tokenizer.next_token() {
                    Token::Equal => match self.tokenizer.next_token() {
                        Token::String(value) => {
                            // Check if this is an event handler (@click, @input, etc.)
                            if name.starts_with('@') {
                                let event_name = name.trim_start_matches('@');
                                events.insert(event_name.to_string(), value);
                            } else {
                                attributes.insert(name, AttributeValue::Static(value));
                            }
                        }
                        Token::ExprStart => {
                            let expr = self.parse_expression()?;
                            attributes.insert(name, AttributeValue::Dynamic(expr));
                        }
                        token => return Err(format!("Expected attribute value, got {:?}", token)),
                    },
                    token => return Err(format!("Expected =, got {:?}", token)),
                },
                Token::CloseTag(close_tag) => {
                    if close_tag != tag {
                        return Err(format!("Mismatched tags: {} and {}", tag, close_tag));
                    }
                    break;
                }
                Token::SelfClosingTag(_) => break,
                Token::Text(text) => {
                    children.push(TemplateNode::Text(text));
                }
                Token::Identifier(text) => {
                    children.push(TemplateNode::Text(text));
                }
                Token::Number(text) => {
                    children.push(TemplateNode::Text(text));
                }
                Token::Plus => {
                    children.push(TemplateNode::Text("+".to_string()));
                }
                Token::Minus => {
                    children.push(TemplateNode::Text("-".to_string()));
                }
                Token::Star => {
                    children.push(TemplateNode::Text("*".to_string()));
                }
                Token::Slash => {
                    children.push(TemplateNode::Text("/".to_string()));
                }
                Token::ExprStart => {
                    let expr = self.parse_expression()?;
                    children.push(TemplateNode::Expression(expr));
                }
                Token::OpenTag(child_tag) => {
                    children.push(self.parse_element(child_tag)?);
                }
                Token::Eof => return Err("Unexpected end of template".to_string()),
                token => return Err(format!("Unexpected token: {:?}", token)),
            }
        }

        Ok(TemplateNode::Element {
            tag,
            attributes,
            events,
            children,
        })
    }    /// Parse an expression inside {{ }}
    fn parse_expression(&mut self) -> Result<String, String> {
        let mut expr = String::new();
        let mut prev_was_operator = false;
        let mut prev_was_identifier = false;
        
        loop {
            match self.tokenizer.next_token() {
                Token::ExprEnd => break,
                Token::Identifier(ident) => {
                    if prev_was_identifier {
                        expr.push(' ');
                    }
                    expr.push_str(&ident);
                    prev_was_identifier = true;
                    prev_was_operator = false;
                },
                Token::Dot => {
                    expr.push('.');
                    prev_was_identifier = false;
                    prev_was_operator = false;
                },
                Token::Number(num) => {
                    if prev_was_identifier {
                        expr.push(' ');
                    }
                    expr.push_str(&num);
                    prev_was_identifier = true;
                    prev_was_operator = false;
                },
                Token::String(str) => {
                    expr.push_str(&format!("\"{}\"", str));
                    prev_was_identifier = true;
                    prev_was_operator = false;
                },
                Token::Plus => {
                    if !prev_was_operator && !expr.is_empty() {
                        expr.push(' ');
                    }
                    expr.push('+');
                    expr.push(' ');
                    prev_was_identifier = false;
                    prev_was_operator = true;
                },
                Token::Minus => {
                    if !prev_was_operator && !expr.is_empty() {
                        expr.push(' ');
                    }
                    expr.push('-');
                    expr.push(' ');
                    prev_was_identifier = false;
                    prev_was_operator = true;
                },
                Token::Star => {
                    if !prev_was_operator && !expr.is_empty() {
                        expr.push(' ');
                    }
                    expr.push('*');
                    expr.push(' ');
                    prev_was_identifier = false;
                    prev_was_operator = true;
                },
                Token::Slash => {
                    if !prev_was_operator && !expr.is_empty() {
                        expr.push(' ');
                    }
                    expr.push('/');
                    expr.push(' ');
                    prev_was_identifier = false;
                    prev_was_operator = true;
                },
                Token::OpenParen => {
                    expr.push('(');
                    prev_was_identifier = false;
                    prev_was_operator = false;
                },
                Token::CloseParen => {
                    expr.push(')');
                    prev_was_identifier = true;
                    prev_was_operator = false;
                },
                Token::Comma => {
                    expr.push(',');
                    expr.push(' ');
                    prev_was_identifier = false;
                    prev_was_operator = false;
                },
                Token::Eof => return Err("Unclosed expression".to_string()),
                token => return Err(format!("Unexpected token in expression: {:?}", token)),
            }
        }

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let input = r#"<div class="greeting">Hello</div>"#;
        let mut parser = TemplateParser::new(input);
        let node = parser.parse().unwrap();

        match node {
            TemplateNode::Element {
                tag,
                attributes,
                events,
                children,
            } => {
                assert_eq!(tag, "div");
                assert_eq!(attributes.len(), 1);
                assert_eq!(events.len(), 0);
                assert_eq!(children.len(), 1);

                match &attributes.get("class").unwrap() {
                    AttributeValue::Static(value) => assert_eq!(value, "greeting"),
                    _ => panic!("Expected static attribute"),
                }

                match &children[0] {
                    TemplateNode::Text(text) => assert_eq!(text, "Hello"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_parse_expression() {
        let input = r#"<div>{{ count + 1 }}</div>"#;
        let mut parser = TemplateParser::new(input);
        let node = parser.parse().unwrap();

        match node {
            TemplateNode::Element {
                tag,
                attributes,
                events,
                children,
            } => {
                assert_eq!(tag, "div");
                assert_eq!(attributes.len(), 0);
                assert_eq!(events.len(), 0);
                assert_eq!(children.len(), 1);

                match &children[0] {
                    TemplateNode::Expression(expr) => assert_eq!(expr, "count + 1"),
                    _ => panic!("Expected expression node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_parse_event_handler() {
        let input = r#"<button @click="increment">+</button>"#;
        let mut parser = TemplateParser::new(input);
        let node = parser.parse().unwrap();

        match node {
            TemplateNode::Element {
                tag,
                attributes,
                events,
                children,
            } => {
                assert_eq!(tag, "button");
                assert_eq!(attributes.len(), 0);
                assert_eq!(events.len(), 1);
                assert_eq!(children.len(), 1);

                assert_eq!(events.get("click").unwrap(), "increment");

                match &children[0] {
                    TemplateNode::Text(text) => assert_eq!(text, "+"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element node"),
        }
    }
}
