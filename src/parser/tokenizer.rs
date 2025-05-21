//! Tokenizer for .orbit files

use std::iter::Peekable;
use std::str::Chars;

/// Token types that can appear in a template
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Token {
    // Template tokens
    OpenTag(String),
    CloseTag(String),
    SelfClosingTag(String),
    AttrName(String),
    AttrValue(String),
    Text(String),

    // Expression tokens
    ExprStart, // {{
    ExprEnd,   // }}
    // EventHandler is now handled through AttrName with @ prefix

    // Punctuation
    Equal, // =
    Quote, // " or '

    // Delimiters
    OpenBrace,  // {
    CloseBrace, // }
    OpenParen,  // (
    CloseParen, // )

    // Expression operators
    Dot,   // .
    Comma, // ,
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /

    // Keywords
    Identifier(String),
    Number(String),
    String(String),

    // Special
    Eof,
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
            None => Token::Eof,
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
                '@' => {
                    self.advance(); // Skip @
                                    // Read the event name
                    let mut name = String::new();
                    name.push('@'); // Keep the @ prefix in the attribute name

                    while let Some(ch) = self.peek() {
                        if ch.is_alphanumeric() || ch == '-' {
                            name.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return Token::AttrName(name);
                }
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
                ch if ch.is_alphabetic() || ch == '_' => {
                    // Check if we're parsing an attribute name
                    let saved_pos = self.input.clone();
                    let mut ident = String::new();

                    while let Some(ch) = self.peek() {
                        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                            ident.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    // Skip whitespace
                    self.skip_whitespace();

                    // If followed by '=', it's an attribute name
                    if self.peek() == Some('=') {
                        Token::AttrName(ident)
                    } else {
                        // Otherwise, reset position and read as normal identifier
                        self.input = saved_pos;
                        self.read_identifier()
                    }
                }
                _ch => self.read_text(),
            },
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

        // Read the tag name only (stop at whitespace or >)
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
                ch if ch.is_whitespace() => {
                    // Stop at whitespace, the attribute parsing will continue from here
                    return Token::OpenTag(name);
                }
                _ => {
                    name.push(ch);
                    self.advance();
                }
            }
        }

        Token::Error("Unclosed tag".to_string())
    }

    // Event handlers are now handled directly in the next_token method
    // This method is kept as a placeholder to avoid having to update all references
    #[allow(dead_code)]
    fn read_event_handler(&mut self) -> Token {
        Token::Error("EventHandler is deprecated".to_string())
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

    /// Read a text node
    fn read_text(&mut self) -> Token {
        let mut text = String::new();

        // First character
        if let Some(ch) = self.advance() {
            // Skip '>' character if it's the start of a text node
            // This is needed because we might have just consumed a tag
            if ch != '>' {
                text.push(ch);
            }
        } else {
            return Token::Eof;
        }

        // Rest of the text until we hit a special character
        while let Some(ch) = self.peek() {
            if ch == '<' || ch == '{' || ch == '@' || ch == '=' {
                break;
            }
            text.push(ch);
            self.advance();
        }

        // If it's just whitespace, handle it specially
        if text.trim().is_empty() {
            // Skip whitespace tokens entirely between elements
            if self.peek() == Some('<') {
                return self.next_token();
            }
        }

        Token::Text(text)
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
        if let Some(_ch) = ch {
            self.column += 1;
        }
        ch
    }
}
