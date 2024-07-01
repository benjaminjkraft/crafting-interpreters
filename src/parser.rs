use crate::ast::*;
#[cfg(test)]
use crate::ast_printer;
use crate::error;
use crate::error::LoxError;
use crate::object::Literal;
#[cfg(test)]
use crate::scanner;
use crate::scanner::{Token, TokenType};
#[cfg(test)]
use itertools::Itertools;

struct Parser<'src> {
    tokens: Vec<Token<'src>>,
    current: usize,
    errors: Vec<LoxError>,
}

pub fn parse<'src>(tokens: Vec<Token<'src>>) -> Result<Program<'src>, Vec<LoxError>> {
    let mut parser = Parser {
        tokens,
        current: 0,
        errors: Vec::new(),
    };
    let result = parser.program();
    if parser.errors.len() > 0 {
        Err(parser.errors)
    } else {
        Ok(result)
    }
}

impl<'src> Parser<'src> {
    fn program(&mut self) -> Program<'src> {
        let mut declarations = Vec::new();
        while !self.is_at_end() {
            let declaration = self.declaration();
            match declaration {
                Ok(decl) => declarations.push(decl),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        Program {
            stmts: declarations,
        }
    }

    fn declaration(&mut self) -> Result<Stmt<'src>, LoxError> {
        if self.match_(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_(&[TokenType::Fun]) {
            self.function("function")
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt<'src>, LoxError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.match_(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(VarStmt { name, initializer }.into())
    }

    fn statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        if self.match_(&[TokenType::LeftBrace]) {
            self.block_statement()
        } else if self.match_(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_(&[TokenType::Return]) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(PrintStmt {
            expr: Box::new(value),
        }
        .into())
    }

    fn return_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        let keyword = self.previous();
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(Box::new(self.expression()?))
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(ReturnStmt { keyword, value }.into())
    }

    fn if_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_ = Box::new(self.statement()?);
        let else_ = if self.match_(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(IfStmt {
            condition,
            then_,
            else_,
        }
        .into())
    }

    fn while_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let body = Box::new(self.statement()?);
        Ok(WhileStmt { condition, body }.into())
    }

    fn for_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_(&[TokenType::Semicolon]) {
            None
        } else if self.match_(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(TokenType::Semicolon) {
            LiteralExpr {
                value: Literal::Bool(true),
            }
            .into()
        } else {
            self.expression()?
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clause.")?;

        let body = self.statement()?;

        let body_with_increment = match increment {
            Some(expr) => BlockStmt {
                stmts: vec![
                    body,
                    ExprStmt {
                        expr: Box::new(expr),
                    }
                    .into(),
                ],
            }
            .into(),
            None => body,
        };

        let body_with_condition = WhileStmt {
            condition: Box::new(condition),
            body: Box::new(body_with_increment),
        }
        .into();

        let body_with_initializer = match initializer {
            Some(stmt) => BlockStmt {
                stmts: vec![stmt, body_with_condition],
            }
            .into(),
            None => body_with_condition,
        };

        Ok(body_with_initializer)
    }

    fn block(&mut self) -> Result<Vec<Stmt<'src>>, LoxError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() && !self.check(TokenType::RightBrace) {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn block_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        Ok(BlockStmt {
            stmts: self.block()?,
        }
        .into())
    }

    fn expression_statement(&mut self) -> Result<Stmt<'src>, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(ExprStmt {
            expr: Box::new(value),
        }
        .into())
    }

    fn function(&mut self, kind: &str) -> Result<Stmt<'src>, LoxError> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut parameters = Vec::new();
        // TODO: abstract into some kind of parse-list-while loop?
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.errors.push(error::parse_error(
                        self.peek(),
                        "Can't have more than 255 parameters.",
                    ));
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        Ok(FunctionStmt {
            name,
            parameters,
            body,
        }
        .into())
    }

    fn expression(&mut self) -> Result<Expr<'src>, LoxError> {
        self.assignment()
    }

    fn binary_expression(
        &mut self,
        tokens: &[TokenType],
        next: &mut dyn FnMut(&mut Self) -> Result<Expr<'src>, LoxError>,
    ) -> Result<Expr<'src>, LoxError> {
        let mut expr = next(self)?;
        while self.match_(tokens) {
            let operator = self.previous();
            let right = next(self)?;
            expr = BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
            .into();
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr<'src>, LoxError> {
        let expr = self.or()?;
        if self.match_(&[TokenType::Equal]) {
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

    fn or(&mut self) -> Result<Expr<'src>, LoxError> {
        let mut expr = self.and()?;
        while self.match_(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr<'src>, LoxError> {
        let mut expr = self.equality()?;
        while self.match_(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
            .into();
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr<'src>, LoxError> {
        self.binary_expression(
            &[TokenType::BangEqual, TokenType::EqualEqual],
            &mut |self_| self_.comparison(),
        )
    }

    fn comparison(&mut self) -> Result<Expr<'src>, LoxError> {
        self.binary_expression(
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            &mut |self_| self_.term(),
        )
    }

    fn term(&mut self) -> Result<Expr<'src>, LoxError> {
        self.binary_expression(&[TokenType::Minus, TokenType::Plus], &mut |self_| {
            self_.factor()
        })
    }

    fn factor(&mut self) -> Result<Expr<'src>, LoxError> {
        self.binary_expression(&[TokenType::Slash, TokenType::Star], &mut |self_| {
            self_.unary()
        })
    }

    fn unary(&mut self) -> Result<Expr<'src>, LoxError> {
        if self.match_(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(UnaryExpr {
                operator,
                right: Box::new(right),
            }
            .into());
        }

        return self.call();
    }

    fn call(&mut self) -> Result<Expr<'src>, LoxError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr<'src>) -> Result<Expr<'src>, LoxError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.errors.push(error::parse_error(
                        self.peek(),
                        "Can't have more than 255 arguments.",
                    ));
                }
                arguments.push(self.expression()?);
                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(CallExpr {
            callee: Box::new(callee),
            paren,
            arguments,
        }
        .into())
    }

    fn primary(&mut self) -> Result<Expr<'src>, LoxError> {
        if self.match_(&[TokenType::False]) {
            Ok(LiteralExpr {
                value: Literal::Bool(false),
            }
            .into())
        } else if self.match_(&[TokenType::True]) {
            Ok(LiteralExpr {
                value: Literal::Bool(true),
            }
            .into())
        } else if self.match_(&[TokenType::Nil]) {
            Ok(LiteralExpr {
                value: Literal::Nil,
            }
            .into())
        } else if self.match_(&[TokenType::Number]) {
            Ok(LiteralExpr {
                value: Literal::Number(self.previous().lexeme.parse().unwrap()),
            }
            .into())
        } else if self.match_(&[TokenType::StringLiteral]) {
            let lexeme = self.previous().lexeme;
            let val = &lexeme[1..lexeme.len() - 1];
            Ok(LiteralExpr {
                value: Literal::String(val.to_string()),
            }
            .into())
        } else if self.match_(&[TokenType::Identifier]) {
            Ok(VariableExpr {
                name: self.previous(),
            }
            .into())
        } else if self.match_(&[TokenType::LeftParen]) {
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

    fn match_(&mut self, types: &[TokenType]) -> bool {
        types.iter().any(|type_| {
            if self.check(*type_) {
                self.advance();
                true
            } else {
                false
            }
        })
    }

    fn consume(&mut self, type_: TokenType, message: &str) -> Result<Token<'src>, LoxError> {
        if self.check(type_) {
            Ok(self.advance())
        } else {
            Err(error::parse_error(self.peek(), message))
        }
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
        !self.is_at_end() && self.peek().type_ == type_
    }

    fn advance(&mut self) -> Token<'src> {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EOF
    }

    fn peek(&self) -> Token<'src> {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token<'src> {
        self.tokens[self.current - 1].clone()
    }
}

#[cfg(test)]
pub fn must_parse<'src>(input: &'src str) -> Program<'src> {
    parse(scanner::scan_tokens(input).unwrap()).unwrap()
}

#[cfg(test)]
fn assert_parses_to(input: &str, printed: &str) {
    assert_eq!(ast_printer::print(must_parse(input)), printed);
}

#[cfg(test)]
fn assert_parse_error(input: &str, messages: &[&str]) {
    let errs = parse(scanner::scan_tokens(input).unwrap()).unwrap_err();
    assert_eq!(
        errs.len(),
        messages.len(),
        "expected {} errors, got {}: {}",
        messages.len(),
        errs.len(),
        errs.into_iter().map(|e| e.to_string()).join("\n"),
    );
    for (a, e) in errs.into_iter().zip(messages) {
        assert_eq!(&a.to_string(), e)
    }
}

#[test]
fn test_parser_simple_exprs() {
    assert_parses_to("1+2;", "(expr (+ (1) (2)))");
    assert_parse_error("1+2", &["[line 1] Error at end: Expect ';' after value."]);
    assert_parse_error("1+;", &["[line 1] Error at ';': Expect expression."]);
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
}

#[test]
fn test_parser_literals() {
    assert_parses_to("1;", "(expr (1))");
    assert_parses_to("1.0;", "(expr (1))");
    assert_parses_to("12.345;", "(expr (12.345))");
    assert_parse_error("1.;", &["[line 1] Error at '.': Expect ';' after value."]);
    assert_parses_to("\"asdf!!\";", "(expr (asdf!!))");
}

#[test]
fn test_parser_vars() {
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
}

#[test]
fn test_parser_blocks() {
    assert_parses_to("{}", "(block\n)");
    assert_parse_error("{", &["[line 1] Error at end: Expect '}' after block."]);
    assert_parse_error(
        "var;\nvar;",
        &[
            "[line 1] Error at ';': Expect variable name.",
            "[line 2] Error at ';': Expect variable name.",
        ],
    );
    assert_parses_to(
        "{ var v; var w; print v + w; }",
        "(block\n\t(var v)\n\t(var w)\n\t(print (+ (variable v) (variable w)))\n)",
    );
}

#[test]
fn test_parser_if() {
    assert_parses_to("if (true) 1;", "(if (true) (expr (1)))");
    assert_parses_to("if (true) 1; else 2;", "(if (true) (expr (1)) (expr (2)))");
    assert_parse_error(
        "if true 1;",
        &["[line 1] Error at 'true': Expect '(' after 'if'."],
    );
    assert_parses_to(
        "if (true) { 1; 2; } else { 3; 4; }",
        "(if (true) (block\n\t(expr (1))\n\t(expr (2))\n) (block\n\t(expr (3))\n\t(expr (4))\n))",
    );
    assert_parses_to(
        "if (1) if (2) 3; else 4;",
        "(if (1) (if (2) (expr (3)) (expr (4))))",
    );
}

#[test]
fn test_parser_logical() {
    assert_parses_to(
        "1 and 2 or 3 and 4;",
        "(expr (or (and (1) (2)) (and (3) (4))))",
    )
}

#[test]
fn test_parser_while() {
    assert_parses_to("while (true) 1;", "(while (true) (expr (1)))");
    assert_parse_error(
        "while true 1;",
        &["[line 1] Error at 'true': Expect '(' after 'while'."],
    );
    assert_parses_to(
        "while (true) { 1; 2; }",
        "(while (true) (block\n\t(expr (1))\n\t(expr (2))\n))",
    );
}

#[test]
fn test_parser_for() {
    assert_parses_to("for (;;) 1;", "(while (true) (expr (1)))");
    assert_parses_to("for (;false;) 1;", "(while (false) (expr (1)))");
    assert_parses_to("for (;i < 2;i = i + 1) 1;", "(while (< (variable i) (2)) (block\n\t(expr (1))\n\t(expr (assign i (+ (variable i) (1))))\n))");
    assert_parses_to(
        "for (var i = 0;false;) 1;",
        "(block\n\t(var i (0))\n\t(while (false) (expr (1)))\n)",
    );
    assert_parses_to(
        "for (i = 0;false;) 1;",
        "(block\n\t(expr (assign i (0)))\n\t(while (false) (expr (1)))\n)",
    );

    assert_parse_error(
        "for (;;;) 1;",
        // TODO: disable synchronize inside of for loop conditions.
        &[
            "[line 1] Error at ';': Expect expression.",
            "[line 1] Error at ')': Expect expression.",
        ],
    );
    assert_parse_error("for (;) 1;", &["[line 1] Error at ')': Expect expression."]);
    assert_parse_error("for () 1;", &["[line 1] Error at ')': Expect expression."]);
}

#[test]
fn test_parser_call() {
    assert_parses_to("f();", "(expr (call (variable f)))");
    assert_parses_to("f(1, 2, 3);", "(expr (call (variable f) (1) (2) (3)))");
    assert_parses_to(
        "f(1)(2)(3);",
        "(expr (call (call (call (variable f) (1)) (2)) (3)))",
    );

    assert_parse_error("f(;", &["[line 1] Error at ';': Expect expression."]);
    assert_parse_error(
        "f(1;",
        &["[line 1] Error at ';': Expect ')' after arguments."],
    );
    assert_parse_error("f(1,;", &["[line 1] Error at ';': Expect expression."]);
}

#[test]
fn test_parser_function() {
    assert_parses_to("fun f() {}", "(fun f (\n))");
    assert_parses_to("fun f(a) {}", "(fun f a (\n))");
    assert_parses_to("fun f(a, b) {}", "(fun f a b (\n))");
    assert_parses_to(
        "fun f(a) { print a; }",
        "(fun f a (\n\t(print (variable a))\n))",
    );

    assert_parse_error("fun();", &["[line 1] Error at '(': Expect function name."]);
    assert_parse_error(
        "fun f;",
        &["[line 1] Error at ';': Expect '(' after function name."],
    );
    assert_parse_error(
        "fun f(;",
        &["[line 1] Error at ';': Expect parameter name."],
    );
    assert_parse_error(
        "fun f(a;",
        &["[line 1] Error at ';': Expect ')' after parameters."],
    );
    assert_parse_error(
        "fun f(a);",
        &["[line 1] Error at ';': Expect '{' before function body."],
    );
}

#[test]
fn test_parser_return() {
    assert_parses_to("fun f() { return 3; }", "(fun f (\n\t(return (3))\n))");
    assert_parses_to("fun f() { return; }", "(fun f (\n\t(return)\n))");
}
