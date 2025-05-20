//! Tokenizer for .orbit files

use std::iter::Peekable;
use std::str::Chars;

/// Token types that can appear in a template
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Template tokens
    OpenTag(String),
    CloseTag(String),
    SelfClosingTag(String),
    AttrName(String),
    AttrValue(String),
    Text(String),
    
    // Expression tokens
    ExprStart,      // {{
    ExprEnd,        // }}
    EventHandler,   // @click, @input, etc.
    
    // Punctuation
    Equal,          // =
    Quote,         // " or '
    
    // Delimiters
    OpenBrace,      // {
    CloseBrace,     // }
    OpenParen,      // (
    CloseParen,     // )
    
    // Expression operators
    Dot,            // .
    Comma,          // ,
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    
    // Keywords
    Identifier(String),
    Number(String),
    String(String),
    
    // Special
    EOF,
    Error(String),
}

/// Tokenizes an input string into a sequence of tokens
pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            line: 1,
            column: 0,
        }
    }
    
    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        match self.peek() {
            None => Token::EOF,
            Some(ch) => match ch {
                '<' => self.read_tag(),
                '{' => {
                    if self.peek_next() == Some('{') {
                        self.advance(); // Skip first {
                        self.advance(); // Skip second {
                        Token::ExprStart
                    } else {
                        self.advance();
                        Token::OpenBrace
                    }
                }
                '}' => {
                    if self.peek_next() == Some('}') {
                        self.advance(); // Skip first }
                        self.advance(); // Skip second }
                        Token::ExprEnd
                    } else {
                        self.advance();
                        Token::CloseBrace
                    }
                }
                '@' => self.read_event_handler(),
                '=' => {
                    self.advance();
                    Token::Equal
                }
                '"' | '\'' => self.read_string(),
                '.' => {
                    self.advance();
                    Token::Dot
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                '+' => {
                    self.advance();
                    Token::Plus
                }
                '-' => {
                    self.advance();
                    Token::Minus
                }
                '*' => {
                    self.advance();
                    Token::Star
                }
                '/' => {
                    self.advance();
                    Token::Slash
                }
                '(' => {
                    self.advance();
                    Token::OpenParen
                }
                ')' => {
                    self.advance();
                    Token::CloseParen
                }
                ch if ch.is_ascii_digit() => self.read_number(),
                ch if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
                ch => {
                    self.advance();
                    Token::Text(ch.to_string())
                }
            }
        }
    }
    
    /// Read a complete tag (opening, closing, or self-closing)
    fn read_tag(&mut self) -> Token {
        self.advance(); // Skip <
        let mut name = String::new();
        
        if self.peek() == Some('/') {
            self.advance(); // Skip /
            while let Some(ch) = self.peek() {
                if ch == '>' {
                    self.advance();
                    return Token::CloseTag(name);
                }
                name.push(ch);
                self.advance();
            }
        }
        
        while let Some(ch) = self.peek() {
            match ch {
                '>' => {
                    self.advance();
                    return Token::OpenTag(name);
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        return Token::SelfClosingTag(name);
                    }
                }
                _ => {
                    name.push(ch);
                    self.advance();
                }
            }
        }
        
        Token::Error("Unclosed tag".to_string())
    }
    
    /// Read an event handler starting with @
    fn read_event_handler(&mut self) -> Token {
        self.advance(); // Skip @
        let mut name = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '-' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Token::EventHandler
    }
    
    /// Read a string literal
    fn read_string(&mut self) -> Token {
        let quote = self.advance().unwrap();
        let mut value = String::new();
        
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return Token::String(value);
            }
            value.push(ch);
            self.advance();
        }
        
        Token::Error("Unclosed string literal".to_string())
    }
    
    /// Read a number literal
    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Token::Number(number)
    }
    
    /// Read an identifier
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Token::Identifier(ident)
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }
    
    /// Peek at the character after the next one
    fn peek_next(&mut self) -> Option<char> {
        let mut iter = self.input.clone();
        iter.next(); // Skip current
        iter.next() // Get next
    }
    
    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.next();
        if let Some(ch) = ch {
            self.column += 1;
        }
        ch
    }
}
