use crate::ast::*;
#[cfg(test)]
use crate::ast_printer;
use crate::error;
use crate::error::LoxError;
#[cfg(test)]
use crate::scanner;
use crate::scanner::{Token, TokenType};

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

pub fn parse<'a>(tokens: Vec<Token<'a>>) -> Result<Program<'a>, Vec<LoxError>> {
    let mut parser = Parser { tokens, current: 0 };
    return parser.program();
}

impl<'a> Parser<'a> {
    fn program(&mut self) -> Result<Program<'a>, Vec<LoxError>> {
        let mut declarations = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            let declaration = self.declaration();
            match declaration {
                Ok(decl) => declarations.push(decl),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(declarations)
        }
    }

    fn declaration(&mut self) -> Result<Stmt<'a>, LoxError> {
        if self.match_(&vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt<'a>, LoxError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?;
        let initializer = if self.match_(&vec![TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        return Ok(VarStmt { name, initializer }.into());
    }

    fn statement(&mut self) -> Result<Stmt<'a>, LoxError> {
        if self.match_(&vec![TokenType::Print]) {
            return self.print_statement();
        }
        return self.expression_statement();
    }

    fn print_statement(&mut self) -> Result<Stmt<'a>, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        return Ok(PrintStmt {
            expr: Box::new(value),
        }
        .into());
    }

    fn expression_statement(&mut self) -> Result<Stmt<'a>, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        return Ok(ExprStmt {
            expr: Box::new(value),
        }
        .into());
    }

    fn expression(&mut self) -> Result<Expr<'a>, LoxError> {
        return self.assignment();
    }

    fn binary_expression(
        &mut self,
        tokens: Vec<TokenType>,
        next: &mut dyn FnMut(&mut Self) -> Result<Expr<'a>, LoxError>,
    ) -> Result<Expr<'a>, LoxError> {
        let mut expr = next(self)?;
        while self.match_(&tokens) {
            let operator = self.previous();
            let right = next(self)?;
            expr = BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
            .into();
        }

        return Ok(expr);
    }

    fn assignment(&mut self) -> Result<Expr<'a>, LoxError> {
        let expr = self.equality()?;
        if self.match_(&vec![TokenType::Equal]) {
            let var = match expr {
                Expr::Variable(v) => Ok(v),
                _ => Err(error::parse_error(
                    self.previous(),
                    "Invalid assignment target.",
                )),
            }?;
            let value = self.assignment()?;
            Ok(AssignExpr {
                name: var.name,
                value: Box::new(value),
            }
            .into())
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> Result<Expr<'a>, LoxError> {
        self.binary_expression(
            vec![TokenType::BangEqual, TokenType::EqualEqual],
            &mut |self_| self_.comparison(),
        )
    }

    fn comparison(&mut self) -> Result<Expr<'a>, LoxError> {
        self.binary_expression(
            vec![
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            &mut |self_| self_.term(),
        )
    }

    fn term(&mut self) -> Result<Expr<'a>, LoxError> {
        self.binary_expression(vec![TokenType::Minus, TokenType::Plus], &mut |self_| {
            self_.factor()
        })
    }

    fn factor(&mut self) -> Result<Expr<'a>, LoxError> {
        self.binary_expression(vec![TokenType::Slash, TokenType::Star], &mut |self_| {
            self_.unary()
        })
    }

    fn unary(&mut self) -> Result<Expr<'a>, LoxError> {
        if self.match_(&vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(UnaryExpr {
                operator,
                right: Box::new(right),
            }
            .into());
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr<'a>, LoxError> {
        if self.match_(&vec![
            TokenType::False,
            TokenType::True,
            TokenType::Nil,
            TokenType::Number,
            TokenType::StringLiteral,
        ]) {
            Ok(LiteralExpr {
                value: self.previous().literal,
            }
            .into())
        } else if self.match_(&vec![TokenType::Identifier]) {
            Ok(VariableExpr {
                name: self.previous(),
            }
            .into())
        } else if self.match_(&vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(GroupingExpr {
                expr: Box::new(expr),
            }
            .into())
        } else {
            Err(error::parse_error(self.peek(), "Expect expression."))
        }
    }

    fn match_(&mut self, types: &Vec<TokenType>) -> bool {
        types.into_iter().any(|type_| {
            if self.check(*type_) {
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

        return Err(error::parse_error(self.peek(), message));
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
pub fn must_parse<'a>(input: &'a str) -> Program<'a> {
    parse(scanner::scan_tokens(input).unwrap()).unwrap()
}

#[cfg(test)]
fn assert_parses_to(input: &str, printed: &str) {
    assert_eq!(ast_printer::print(must_parse(input)), printed);
}

#[test]
fn test_parser() {
    assert_parses_to("1+2;", "(expr (+ (1) (2)))");
    assert_parses_to(
        "1 + 2 == 3 / -4 - 5 >= 6;",
        "(expr (== (+ (1) (2)) (>= (- (/ (3) (- (4))) (5)) (6))))",
    );
    assert_parses_to("---6;", "(expr (- (- (- (6)))))");
    assert_parses_to(
        "true == false != nil;",
        "(expr (!= (== (true) (false)) (nil)))",
    );
    assert_parses_to("1.2 + \"four\";", "(expr (+ (1.2) (four)))");
    assert_parses_to("print 1+2;", "(print (+ (1) (2)))");
    assert_parses_to("1+2;print 1+2;", "(expr (+ (1) (2)))\n(print (+ (1) (2)))");
    assert_parses_to(
        "var v = 1; v+2;",
        "(var v (1))\n(expr (+ (variable v) (2)))",
    );
    assert_parses_to(
        "var v = 1; v = 3 + 2;",
        "(var v (1))\n(expr (assign v (+ (3) (2))))",
    );
    assert_parses_to(
        "var v = 1; (v = 3) + 2;",
        "(var v (1))\n(expr (+ (group (assign v (3))) (2)))",
    );
    assert_parses_to(
        "var v; var w; v = w = 3;",
        "(var v)\n(var w)\n(expr (assign v (assign w (3))))",
    );
    // TODO: test parse errors
}
