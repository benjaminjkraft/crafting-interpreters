use crate::error::LoxError;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use TokenType::*;

pub struct Scanner<'src> {
    source: &'src str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'src> Scanner<'src> {
    fn scan_tokens(&mut self) -> Result<Vec<Token<'src>>, LoxError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            if let Some(tok) = self.scan_token()? {
                tokens.push(tok);
            }
        }

        tokens.push(Token {
            type_: EOF,
            lexeme: "",
            line: self.line,
        });
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<Option<Token<'src>>, LoxError> {
        let c = self.advance();
        Ok(match c {
            b'(' => self.token(LeftParen),
            b')' => self.token(RightParen),
            b'{' => self.token(LeftBrace),
            b'}' => self.token(RightBrace),
            b',' => self.token(Comma),
            b'.' => self.token(Dot),
            b'-' => self.token(Minus),
            b'+' => self.token(Plus),
            b';' => self.token(Semicolon),
            b'*' => self.token(Star),
            b'!' => {
                let next_eq = self.match_(b'=');
                self.token(if next_eq { BangEqual } else { Bang })
            }
            b'=' => {
                let next_eq = self.match_(b'=');
                self.token(if next_eq { EqualEqual } else { Equal })
            }
            b'<' => {
                let next_eq = self.match_(b'=');
                self.token(if next_eq { LessEqual } else { Less })
            }
            b'>' => {
                let next_eq = self.match_(b'=');
                self.token(if next_eq { GreaterEqual } else { Greater })
            }
            b'/' => {
                if self.match_(b'/') {
                    self.advance_all(|c| c != b'\n');
                    None
                } else {
                    self.token(Slash)
                }
            }
            b' ' | b'\r' | b'\t' => None,
            b'\n' => None,
            b'"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number()
                } else if is_alpha(c) {
                    self.identifier()
                } else {
                    self.err(format!("Unexpected character: '{}'.", c))?
                }
            }
        })
    }

    fn err(&mut self, message: String) -> Result<Option<Token<'src>>, LoxError> {
        Err(LoxError {
            line: self.line,
            loc: String::new(),
            exit: 65,
            message,
        })
    }

    fn peek(&mut self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source.as_bytes()[self.current])
        }
    }

    fn peek_next(&mut self) -> Option<u8> {
        if self.current + 1 > self.source.len() {
            None
        } else {
            Some(self.source.as_bytes()[self.current + 1])
        }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.peek().unwrap();
        if ch == b'\n' {
            self.line += 1;
        }
        self.current += 1;
        ch
    }

    fn match_(&mut self, expected: u8) -> bool {
        self.match_pred(&|c| c == expected)
    }

    fn advance_all(&mut self, pred: impl Fn(u8) -> bool) {
        while self.match_pred(&pred) {}
    }

    fn match_pred(&mut self, pred: &impl Fn(u8) -> bool) -> bool {
        if self.peek().is_some_and(pred) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn token(&self, type_: TokenType) -> Option<Token<'src>> {
        Some(Token {
            type_,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        })
    }

    fn string(&mut self) -> Result<Option<Token<'src>>, LoxError> {
        self.advance_all(|c| c != b'"');
        if !self.match_(b'"') {
            self.err("Unterminated string".to_string())?;
        }

        Ok(self.token(StringLiteral))
    }

    fn number(&mut self) -> Option<Token<'src>> {
        self.advance_all(is_digit);
        let decimal = self.peek() == Some(b'.') && self.peek_next().is_some_and(is_digit);
        if decimal {
            self.advance();
            self.advance_all(is_digit);
        }
        self.token(Number)
    }

    fn identifier(&mut self) -> Option<Token<'src>> {
        self.advance_all(is_alpha_numeric);

        let type_ = KEYWORDS
            .get(&self.source[self.start..self.current])
            .copied()
            .unwrap_or(Identifier);
        self.token(type_)
    }
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn is_alpha(c: u8) -> bool {
    c >= b'a' && c <= b'z' || c >= b'A' && c <= b'Z' || c == b'_'
}

fn is_alpha_numeric(c: u8) -> bool {
    is_digit(c) || is_alpha(c)
}

pub fn scan_tokens<'src>(source: &'src str) -> Result<Vec<Token<'src>>, LoxError> {
    let mut scanner = Scanner {
        source,
        start: 0,
        current: 0,
        line: 1,
    };
    scanner.scan_tokens()
}

#[derive(Debug, Clone)]
pub struct Token<'src> {
    pub type_: TokenType,
    pub lexeme: &'src str,
    pub line: usize,
}

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.type_, self.lexeme)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[test]
fn test_scanner() {
    insta::assert_debug_snapshot!(scan_tokens("(){},.-+;* // (symbols)"));
    insta::assert_debug_snapshot!(scan_tokens(""));
    insta::assert_debug_snapshot!(scan_tokens("\n\n"));
    insta::assert_debug_snapshot!(scan_tokens("// asdf"));
    insta::assert_debug_snapshot!(scan_tokens("// asdf\n"));
    insta::assert_debug_snapshot!(scan_tokens("\n\n!a and !!b and !!c != d == e////"));
    insta::assert_debug_snapshot!(scan_tokens("= == < <= > >= => =<"));
    insta::assert_debug_snapshot!(scan_tokens("1/1.1/1.23/123.45"));
    insta::assert_debug_snapshot!(scan_tokens("1"));
    insta::assert_debug_snapshot!(scan_tokens("\n\n\t\t   \n\t\t   \"asdf!!\"\n\nvar2"));
    insta::assert_debug_snapshot!(scan_tokens("and class class_ else false for fun"));
    insta::assert_debug_snapshot!(scan_tokens("if if_ nil null or print return super"));
    insta::assert_debug_snapshot!(scan_tokens("this true var while class and fun"));
}
