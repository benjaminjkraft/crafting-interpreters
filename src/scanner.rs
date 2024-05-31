use crate::error::LoxError;
use crate::object;
use crate::object::Object;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use TokenType::*;

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    fn scan_tokens(&mut self) -> Result<(), LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            type_: EOF,
            lexeme: "",
            literal: Object::Nil,
            line: self.line,
        });
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();
        match c {
            b'(' => self.add_token(LeftParen),
            b')' => self.add_token(RightParen),
            b'{' => self.add_token(LeftBrace),
            b'}' => self.add_token(RightBrace),
            b',' => self.add_token(Comma),
            b'.' => self.add_token(Dot),
            b'-' => self.add_token(Minus),
            b'+' => self.add_token(Plus),
            b';' => self.add_token(Semicolon),
            b'*' => self.add_token(Star),
            b'!' => {
                let next_eq = self.match_(b'=');
                self.add_token(if next_eq { BangEqual } else { Bang })
            }
            b'=' => {
                let next_eq = self.match_(b'=');
                self.add_token(if next_eq { EqualEqual } else { Equal })
            }
            b'<' => {
                let next_eq = self.match_(b'=');
                self.add_token(if next_eq { LessEqual } else { Less })
            }
            b'>' => {
                let next_eq = self.match_(b'=');
                self.add_token(if next_eq { GreaterEqual } else { Greater })
            }
            b'/' => {
                if self.match_(b'/') {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash)
                }
            }
            b' ' | b'\r' | b'\t' => {}
            b'\n' => {
                self.line += 1;
            }
            b'"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number();
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    self.err(format!("Unexpected character: '{}'.", c))?
                }
            }
        }
        Ok(())
    }

    fn err(&mut self, message: String) -> Result<(), LoxError> {
        Err(LoxError {
            line: self.line,
            loc: "".to_string(),
            message,
        })
    }

    // TODO: here and below, chars instead of u8s?
    fn peek(&mut self) -> u8 {
        self.source.as_bytes()[self.current]
    }

    fn peek_next(&mut self) -> Option<u8> {
        if self.current + 1 > self.source.len() {
            None
        } else {
            Some(self.source.as_bytes()[self.current + 1])
        }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.peek();
        self.current += 1;
        ch
    }

    fn match_(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        } else {
            self.current += 1;
            return true;
        }
    }

    fn add_token(&mut self, type_: TokenType) {
        self.add_token_literal(type_, Object::Nil)
    }

    fn add_token_literal(&mut self, type_: TokenType, literal: Object) {
        self.tokens.push(Token {
            type_,
            lexeme: &self.source[self.start..self.current],
            literal,
            line: self.line,
        })
    }

    fn string(&mut self) -> Result<(), LoxError> {
        while !self.is_at_end() && self.peek() != b'"' {
            if self.peek() == b'\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            self.err("Unterminated string".to_string())?;
        }

        self.advance(); // close-quote
        let val = &self.source[self.start + 1..self.current - 1];
        self.add_token_literal(StringLiteral, Object::String(val.to_string()));
        Ok(())
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }
        let decimal = self.peek() == b'.' && self.peek_next().map_or(false, is_digit);
        if decimal {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }
        let val = &self.source[self.start..self.current];
        if decimal {
            self.add_token_literal(Number, Object::Float(val.parse().unwrap()));
        } else {
            self.add_token_literal(Number, Object::Int(val.parse().unwrap()));
        }
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let type_ = KEYWORDS
            .get(&self.source[self.start..self.current])
            .copied()
            .unwrap_or(Identifier);
        self.add_token(type_);
    }
}

fn is_digit(c: u8) -> bool {
    return c >= b'0' && c <= b'9';
}

fn is_alpha(c: u8) -> bool {
    return c >= b'a' && c <= b'z' || c >= b'A' && c <= b'Z' || c == b'_';
}

fn is_alpha_numeric(c: u8) -> bool {
    return is_digit(c) || is_alpha(c);
}

pub fn scan_tokens<'a>(source: &'a str) -> Result<Vec<Token<'a>>, LoxError> {
    let mut scanner = Scanner {
        source,
        tokens: Vec::new(),
        start: 0,
        current: 0,
        line: 1,
    };
    scanner.scan_tokens()?;
    return Ok(scanner.tokens);
}

pub struct Token<'a> {
    type_: TokenType,
    lexeme: &'a str,
    literal: object::Object,
    line: usize,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {:?}", self.type_, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    StringLiteral,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

static KEYWORDS: Lazy<HashMap<&str, TokenType>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("and", And);
    m.insert("class", Class);
    m.insert("else", Else);
    m.insert("false", False);
    m.insert("for", For);
    m.insert("fun", Fun);
    m.insert("if", If);
    m.insert("nil", Nil);
    m.insert("or", Or);
    m.insert("print", Print);
    m.insert("return", Return);
    m.insert("super", Super);
    m.insert("this", This);
    m.insert("true", True);
    m.insert("var", Var);
    m.insert("while", While);
    m
});
