use crate::ast::*;
#[cfg(test)]
use crate::ast_printer;
use crate::error;
use crate::error::LoxError;
use crate::object::Object;
#[cfg(test)]
use crate::scanner;
use crate::scanner::{Token, TokenType};

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

pub fn parse<'a>(tokens: Vec<Token<'a>>) -> Result<Expr<'a>, LoxError> {
    let mut parser = Parser { tokens, current: 0 };
    return parser.expression();
}

impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr<'a>, LoxError> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr<'a>, LoxError> {
        let mut expr = self.comparison()?;
        while self.match_(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr<'a>, LoxError> {
        let mut expr = self.term()?;
        while self.match_(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr<'a>, LoxError> {
        let mut expr = self.factor()?;
        while self.match_(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr<'a>, LoxError> {
        let mut expr = self.unary()?;
        while self.match_(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr<'a>, LoxError> {
        if self.match_(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            }));
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr<'a>, LoxError> {
        Ok(if self.match_(vec![TokenType::False]) {
            Expr::Literal(LiteralExpr {
                value: Object::Bool(false),
            })
        } else if self.match_(vec![TokenType::True]) {
            Expr::Literal(LiteralExpr {
                value: Object::Bool(true),
            })
        } else if self.match_(vec![TokenType::Nil]) {
            Expr::Literal(LiteralExpr { value: Object::Nil })
        } else if self.match_(vec![TokenType::Number, TokenType::StringLiteral]) {
            Expr::Literal(LiteralExpr {
                value: self.previous().literal,
            })
        } else if self.match_(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Expr::Grouping(GroupingExpr {
                expr: Box::new(expr),
            })
        } else {
            return Err(error::err(self.peek(), "Expect expression."));
        })
    }

    fn match_(&mut self, types: Vec<TokenType>) -> bool {
        types.into_iter().any(|type_| {
            if self.check(type_) {
                self.advance();
                true
            } else {
                false
            }
        })
    }

    fn consume(&mut self, type_: TokenType, message: &str) -> Result<Token<'a>, LoxError> {
        if self.check(type_) {
            return Ok(self.advance());
        }

        return Err(error::err(self.peek(), message));
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }

            match self.peek().type_ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().type_ == type_;
    }

    fn advance(&mut self) -> Token<'a> {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EOF
    }

    fn peek(&self) -> Token<'a> {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token<'a> {
        self.tokens[self.current - 1].clone()
    }
}

#[cfg(test)]
pub fn must_parse<'a>(input: &'a str) -> Expr<'a> {
    parse(scanner::scan_tokens(input).unwrap()).unwrap()
}

#[cfg(test)]
fn assert_parses_to(input: &str, printed: &str) {
    assert_eq!(ast_printer::print(must_parse(input)), printed);
}

#[test]
fn test_parser() {
    assert_parses_to("1+2", "(+ (1) (2))");
    assert_parses_to(
        "1 + 2 == 3 / -4 - 5 >= 6",
        "(== (+ (1) (2)) (>= (- (/ (3) (- (4))) (5)) (6)))",
    );
    assert_parses_to("---6", "(- (- (- (6))))");
    assert_parses_to("true == false != nil", "(!= (== (true) (false)) (nil))");
    assert_parses_to("1.2 + \"four\"", "(+ (1.2) (four))");
}