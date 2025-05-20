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
                Token::AttrName(name) => {
                    match self.tokenizer.next_token() {
                        Token::Equal => {
                            match self.tokenizer.next_token() {
                                Token::String(value) => {
                                    attributes.insert(name, AttributeValue::Static(value));
                                }
                                Token::ExprStart => {
                                    let expr = self.parse_expression()?;
                                    attributes.insert(name, AttributeValue::Dynamic(expr));
                                }
                                token => return Err(format!("Expected attribute value, got {:?}", token)),
                            }
                        }
                        token => return Err(format!("Expected =, got {:?}", token)),
                    }
                }
                Token::EventHandler => {
                    let name = match self.tokenizer.next_token() {
                        Token::Identifier(name) => name,
                        token => return Err(format!("Expected event name, got {:?}", token)),
                    };
                    match self.tokenizer.next_token() {
                        Token::Equal => {
                            match self.tokenizer.next_token() {
                                Token::String(handler) => {
                                    events.insert(name, handler);
                                }
                                token => return Err(format!("Expected event handler, got {:?}", token)),
                            }
                        }
                        token => return Err(format!("Expected =, got {:?}", token)),
                    }
                }
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
                Token::ExprStart => {
                    let expr = self.parse_expression()?;
                    children.push(TemplateNode::Expression(expr));
                }
                Token::OpenTag(child_tag) => {
                    children.push(self.parse_element(child_tag)?);
                }
                Token::EOF => return Err("Unexpected end of template".to_string()),
                token => return Err(format!("Unexpected token: {:?}", token)),
            }
        }
        
        Ok(TemplateNode::Element {
            tag,
            attributes,
            events,
            children,
        })
    }
    
    /// Parse an expression inside {{ }}
    fn parse_expression(&mut self) -> Result<String, String> {
        let mut expr = String::new();
        
        loop {
            match self.tokenizer.next_token() {
                Token::ExprEnd => break,
                Token::Identifier(ident) => expr.push_str(&ident),
                Token::Dot => expr.push('.'),
                Token::Number(num) => expr.push_str(&num),
                Token::String(str) => expr.push_str(&format!("\"{}\"", str)),
                Token::Plus => expr.push('+'),
                Token::Minus => expr.push('-'),
                Token::Star => expr.push('*'),
                Token::Slash => expr.push('/'),
                Token::OpenParen => expr.push('('),
                Token::CloseParen => expr.push(')'),
                Token::Comma => expr.push(','),
                Token::EOF => return Err("Unclosed expression".to_string()),
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
            TemplateNode::Element { tag, attributes, events, children } => {
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
            TemplateNode::Element { tag, attributes, events, children } => {
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
            TemplateNode::Element { tag, attributes, events, children } => {
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
