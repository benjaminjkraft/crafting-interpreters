use crate::ast::*;
use crate::environment::Environment;
use crate::error::{runtime_error, LoxError};
use crate::object::{Function, Literal, Object};
#[cfg(test)]
use crate::parser;
#[cfg(test)]
use crate::scanner;
use crate::scanner::TokenType;
use crate::unwind::Unwinder;
use std::cell::RefCell;
use std::rc::Rc;
use std::time;

pub struct Interpreter<'ast, 'src: 'ast, F: FnMut(String)> {
    // TODO: define printer as a global (even if it's still a magic statement)?
    printer: F,
    globals: Rc<RefCell<Environment<'ast, 'src>>>,
    environment: Rc<RefCell<Environment<'ast, 'src>>>,
}

fn now_sec<'ast, 'src: 'ast>() -> Result<Object<'ast, 'src>, LoxError> {
    match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(t) => Ok(Object::Literal(Literal::Number(t.as_secs_f64()))),
        Err(e) => panic!("{}", e),
    }
}

pub fn interpreter<'ast, 'src: 'ast>() -> Interpreter<'ast, 'src, impl FnMut(String)> {
    let globals = Rc::new(RefCell::new(Environment::new()));
    globals.borrow_mut().define(
        "clock",
        Object::BuiltinFunction(Function {
            arity: 0,
            function: Rc::new(RefCell::new(|_| now_sec())),
            name: "clock".to_string(),
        }),
    );
    Interpreter {
        printer: |s| println!("{}", s),
        globals: globals.clone(),
        environment: globals.clone(),
    }
}

impl<'ast, 'src: 'ast, F: FnMut(String)> Interpreter<'ast, 'src, F> {
    pub fn execute_program(&mut self, node: &'ast Program<'src>) -> Result<(), LoxError> {
        let result = self.execute_stmts(&node.stmts, self.environment.clone());
        match result {
            Ok(()) => Ok(()),
            Err(Unwinder::Err(e)) => Err(e),
            Err(Unwinder::Return { keyword, value: _ }) => {
                Err(runtime_error(&keyword, "Can't return from top-level code."))
            }
        }
    }

    fn evaluate(&mut self, node: &Expr<'src>) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        match node {
            Expr::Assign(node) => {
                let value = self.evaluate(&node.value)?;
                self.environment
                    .borrow_mut()
                    .assign(&node.name, value.clone())?;
                Ok(value)
            }
            Expr::Binary(node) => {
                let left = self.evaluate(&node.left)?;
                let right = self.evaluate(&node.right)?;

                match node.operator.type_ {
                    TokenType::Minus => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Number(l - r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for subtraction"),
                    },
                    TokenType::Plus => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Number(l + r))),
                        (
                            Object::Literal(Literal::String(l)),
                            Object::Literal(Literal::String(r)),
                        ) => Ok(Object::Literal(Literal::String(l + &r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for addition"),
                    },
                    TokenType::Slash => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Number(l / r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for division"),
                    },
                    TokenType::Star => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Number(l * r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for multiplication"),
                    },
                    TokenType::Greater => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Bool(l > r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for comparison"),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Bool(l >= r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for comparison"),
                    },
                    TokenType::Less => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Bool(l < r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for comparison"),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (
                            Object::Literal(Literal::Number(l)),
                            Object::Literal(Literal::Number(r)),
                        ) => Ok(Object::Literal(Literal::Bool(l <= r))),
                        (_, _) => Unwinder::err(&node.operator, "invalid types for comparison"),
                    },
                    TokenType::EqualEqual => Ok(Object::Literal(Literal::Bool(left.eq(&right)))),
                    TokenType::BangEqual => Ok(Object::Literal(Literal::Bool(!left.eq(&right)))),
                    _ => Unwinder::err(&node.operator, "unknown operator (parser bug?)"),
                }
            }
            Expr::Call(node) => {
                let callee = self.evaluate(&node.callee)?;

                let mut arguments = Vec::new();
                for argument in &node.arguments {
                    arguments.push(self.evaluate(argument)?);
                }

                match callee {
                    Object::BuiltinFunction(f) => {
                        if f.arity != arguments.len() {
                            Unwinder::err(
                                &node.paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    f.arity,
                                    arguments.len()
                                ),
                            )
                        } else {
                            Unwinder::promote((f.function.borrow_mut())(arguments))
                        }
                    }
                    Object::Function(f) => {
                        let environment = self.child_environment();
                        if f.parameters.len() != arguments.len() {
                            // TODO: duplicated a bit
                            Unwinder::err(
                                &node.paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    f.parameters.len(),
                                    arguments.len()
                                ),
                            )
                        } else {
                            for (i, parameter) in f.parameters.iter().enumerate() {
                                environment
                                    .borrow_mut()
                                    .define(parameter.lexeme, arguments[i].clone());
                            }
                            let result = self.execute_stmts(&f.body, environment);
                            let r = match result {
                                Ok(()) => Ok(Object::Literal(Literal::Nil)), // (omitted return)
                                Err(Unwinder::Err(e)) => Err(Unwinder::Err(e)),
                                Err(Unwinder::Return { keyword: _, value }) => Ok(value),
                            };
                            r
                        }
                    }
                    o => Unwinder::err(
                        &node.paren,
                        &format!("Can only call functions and classes, got '{}'.", o),
                    ),
                }
            }
            Expr::Grouping(node) => self.evaluate(&node.expr),
            Expr::Literal(node) => Ok(Object::Literal(node.value.clone())),
            Expr::Logical(node) => {
                let left = self.evaluate(&node.left)?;
                match (node.operator.type_, left.is_truthy()) {
                    (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
                    (TokenType::Or, false) | (TokenType::And, true) => self.evaluate(&node.right),
                    _ => Unwinder::err(&node.operator, "unknown operator (parser bug?)"),
                }
            }
            Expr::Unary(node) => {
                let right = self.evaluate(&node.right)?;

                match node.operator.type_ {
                    TokenType::Bang => Ok(Object::Literal(Literal::Bool(!right.is_truthy()))),
                    TokenType::Minus => match right {
                        Object::Literal(Literal::Number(n)) => {
                            Ok(Object::Literal(Literal::Number(-n)))
                        }
                        _ => Unwinder::err(&node.operator, "invalid type for negation"),
                    },
                    _ => Unwinder::err(&node.operator, "unknown operator (parser bug?)"),
                }
            }
            Expr::Variable(node) => self.environment.borrow().get(&node.name),
        }
    }

    fn execute_stmts(
        &mut self,
        stmts: &'ast Vec<Stmt<'src>>,
        environment: Rc<RefCell<Environment<'ast, 'src>>>,
    ) -> Result<(), Unwinder<'ast, 'src>> {
        let prev = self.environment.clone();
        self.environment = environment;
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.environment = prev;
                return Err(e);
            }
        }
        self.environment = prev;
        Ok(())
    }

    fn child_environment(&self) -> Rc<RefCell<Environment<'ast, 'src>>> {
        Rc::new(RefCell::new(Environment::child(self.environment.clone())))
    }

    fn execute(&mut self, node: &'ast Stmt<'src>) -> Result<(), Unwinder<'ast, 'src>> {
        match node {
            Stmt::Block(node) => {
                let environment = self.child_environment();
                self.execute_stmts(&node.stmts, environment)?;
            }

            Stmt::Expr(node) => {
                self.evaluate(&node.expr)?;
            }

            Stmt::Function(node) => {
                let function = Object::Function(node);
                self.environment
                    .borrow_mut()
                    .define(node.name.lexeme, function);
            }

            Stmt::If(node) => {
                let cond = self.evaluate(&node.condition)?;
                if cond.is_truthy() {
                    self.execute(&node.then_)?;
                } else if let Some(e) = &node.else_ {
                    self.execute(e)?;
                }
            }
            Stmt::Print(node) => {
                let value = self.evaluate(&node.expr)?;
                let stringified = format!("{}", value);
                (self.printer)(stringified);
            }
            Stmt::Return(node) => {
                let value = match &node.value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Object::Literal(Literal::Nil),
                };
                Err(Unwinder::Return {
                    keyword: &node.keyword,
                    value,
                })?
            }
            Stmt::Var(node) => {
                let value = match &node.initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Object::Literal(Literal::Nil),
                };

                self.environment
                    .borrow_mut()
                    .define(node.name.lexeme, value);
            }
            Stmt::While(node) => loop {
                let cond = self.evaluate(&node.condition)?;
                if !cond.is_truthy() {
                    break;
                }
                self.execute(&node.body)?;
            },
        }
        Ok(())
    }
}

#[cfg(test)]
pub fn execute_for_tests(source: &str) -> Result<Vec<String>, LoxError> {
    let mut printed: Vec<String> = Vec::new();
    let mut time = 0.0;
    let tokens = scanner::scan_tokens(source)?;
    let prog = parser::parse(tokens)?;
    {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals.borrow_mut().define(
            "clock",
            Object::BuiltinFunction(Function {
                arity: 0,
                function: Rc::new(RefCell::new(move |_| {
                    time += 1.0;
                    Ok(Object::Literal(Literal::Number(time)))
                })),
                name: "clock".to_string(),
            }),
        );
        let mut interpreter = Interpreter {
            printer: |s| printed.push(s),
            globals: globals.clone(),
            environment: globals.clone(),
        };
        interpreter.execute_program(&prog)?;
    }
    Ok(printed)
}

#[cfg(test)]
fn assert_prints(source: &str, expected: &[&str]) {
    match execute_for_tests(source) {
        Ok(a) => assert_eq!(
            a,
            expected.iter().map(|s| s.to_string()).collect::<Vec<_>>()
        ),
        Err(a) => assert!(false, "Expected {:?}, got error {}", expected, a),
    }
}

#[cfg(test)]
fn assert_errs(source: &str, expected: &str) {
    match execute_for_tests(source) {
        Ok(a) => assert!(false, "Expected error {}, got {:?}", expected, a),
        Err(a) => assert_eq!(a.to_string(), expected),
    }
}

#[test]
fn test_evaluate_simple_expr() {
    assert_prints("print 1 + 2;", &["3"]);
    assert_prints("print 1 == 1;", &["true"]);
    assert_prints("print 1 == true;", &["false"]);
    assert_prints("print 1/0 == 1/0;", &["true"]);
    assert_prints("print 0/0 == 0/0;", &["false"]);
    assert_prints("print false == nil;", &["false"]);
    assert_prints("print 1 >= 1;", &["true"]);
    assert_prints("print 1 > 1;", &["false"]);
    assert_errs(
        "print true > false;",
        "[line 1] Error: invalid types for comparison",
    );
    assert_errs(
        r#"print 1 + "a";"#,
        "[line 1] Error: invalid types for addition",
    );
    assert_prints(r#"print "a" + "b";"#, &["ab"]);
    assert_prints("print !true;", &["false"]);
    assert_prints("print !nil;", &["true"]);
    assert_prints("print 1 + (2 + 3);", &["6"]);
    assert_prints(r#"print "a" + "b" + "c";"#, &["abc"]);
}

#[test]
fn test_evaluate_vars() {
    assert_prints("var v; print v;", &["nil"]);
    assert_prints("var v = 3; print v;", &["3"]);
    assert_prints("var v = 3; var v = 4; print v;", &["4"]);
    assert_prints("var v = 3; v = 4; print v;", &["4"]);
    assert_prints("var v; var w; v = w = 4; print v; print w;", &["4", "4"]);
    assert_prints("var v = 3; v = v + 1; print v;", &["4"]);
    assert_prints("var v = 3; v = (v = v + 1) + 1; print v;", &["5"]);
    assert_errs("v = 3;", "[line 1] Error: Undefined variable 'v'.");
}

#[test]
fn test_evaluate_blocks() {
    assert_prints("{}", &[]);
    assert_prints("var a = 1; { var a = 2; print a; } print a;", &["2", "1"]);
    assert_prints(
        "var a = 1; { var b = 2; print a; print b; } print a;",
        &["1", "2", "1"],
    );
    assert_errs(
        "var a = 1; { var b = 2; } print b;",
        "[line 1] Error: Undefined variable 'b'.",
    );
    assert_prints("var a = 1; { a = 2; } print a;", &["2"]);
    assert_prints(
        "var a = 1; { var a = a + 2; print a; } print a;",
        &["3", "1"],
    );
    assert_prints(
        r#"
            var a = "global a";
            var b = "global b";
            var c = "global c";
            {
                var a = "outer a";
                var b = "outer b";
                {
                    var a = "inner a";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        "#,
        &[
            "inner a", "outer b", "global c", "outer a", "outer b", "global c", "global a",
            "global b", "global c",
        ],
    );
}

#[test]
fn test_evaluate_ifs() {
    assert_prints("if (1 < 2) print 3; else print 4;", &["3"]);
    assert_prints("if (1 > 2) print 3; else print 4;", &["4"]);
    assert_prints("if (1 < 2) print 3;", &["3"]);
    assert_prints("if (1 > 2) print 3;", &[]);
}

#[test]
fn test_evaluate_logical() {
    assert_prints("print (1 < 2) and (3 < 4) or (2 < 1);", &["true"]);
    assert_prints("print (1 > 2) and (3 < 4) or (2 > 1);", &["true"]);
    assert_prints("print (1 > 2) and (3 < 4) or (2 < 1);", &["false"]);
    assert_prints(
        "var a; print (1 < 2) and (a = 1) or (a = 2); print a;",
        &["1", "1"],
    );
    assert_prints(
        "var a; print (1 > 2) and (a = 1) or (a = 2); print a;",
        &["2", "2"],
    );
}

#[test]
fn test_evaluate_whiles() {
    assert_prints("while (1 > 2) print 3;", &[]);
    assert_prints("var a = 0; while (a < 1) { a = a + 1; print a; }", &["1"]);
    assert_prints(
        "var a = 0; while (a < 2) { a = a + 1; print a; }",
        &["1", "2"],
    );
}

#[test]
fn test_evaluate_fors() {
    assert_prints(
        "for (var i = 0; i < 3; i = i + 1) { print i; }",
        &["0", "1", "2"],
    );
    assert_prints(
        r"
            var a = 1;
            var tmp;
            for (var b = 1; a < 10000; b = tmp + b) {
                print a;
                tmp = a;
                a = b;
            }
        ",
        &[
            "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233", "377", "610",
            "987", "1597", "2584", "4181", "6765",
        ],
    );
}

#[test]
fn test_call_builtin() {
    assert_prints(
        "print clock(); print clock(); print clock();",
        &["1", "2", "3"],
    );
    assert_prints("print clock;", &["<function clock>"]);
    assert_errs(
        "print clock(3);",
        "[line 1] Error: Expected 0 arguments but got 1.",
    );
    assert_errs(
        "var a = 1; print a(3);",
        "[line 1] Error: Can only call functions and classes, got '1'.",
    );
}

#[test]
fn test_functions() {
    assert_prints("fun add(a, b) { print a + b; } add(1, 2);", &["3"]);
    assert_prints(
        "fun count(n) { if (n > 1) count(n-1); print n; } count(3);",
        &["1", "2", "3"],
    );
    assert_prints(
        r#"
            fun sayHi(first, last) {
                print "Hi, " + first + " " + last + "!";
            }

            sayHi("Dear", "Reader");
        "#,
        &["Hi, Dear Reader!"],
    );

    assert_errs(
        "fun f(n) {} print f(1, 2);",
        "[line 1] Error: Expected 1 arguments but got 2.",
    );
    assert_errs(
        "fun f(n) {} print f();",
        "[line 1] Error: Expected 1 arguments but got 0.",
    );
}

#[test]
fn test_returns() {
    assert_prints("fun add(a, b) { return a + b; } print add(1, 2);", &["3"]);
    assert_prints(
        "fun halt() { if (true) return; while (true) {} } print halt();",
        &["nil"],
    );
    // early return
    assert_prints(
        r"
             fun cond(c, t) {
                 if (c) return t;
    
             }

             print cond(true, 1);
             print cond(false, 2);
         ",
        &["1", "nil"],
    );
    // recursion generally
    assert_prints(
        r"
             fun pow(m, n) {
                 if (n == 0) return 1;
                 return m * pow(m, n - 1);
             }

             for (var i = 0; i < 10; i = i + 1) {
                 print pow(2, i);
             }
         ",
        &["1", "2", "4", "8", "16", "32", "64", "128", "256", "512"],
    );
    // pop env in case of early return
    assert_prints(
        r"
            var a = 0;
            fun f(a) {
                if (true) return 1;
            }

            print f(3);
            print a;
        ",
        &["1", "0"],
    );
    assert_prints(
        r"
            fun fib(n) {
                if (n <= 1) return n;
                return fib(n - 2) + fib(n - 1);
            }

            for (var i = 0; i < 20; i = i + 1) {
                print fib(i);
            }
        ",
        &[
            "0", "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233", "377",
            "610", "987", "1597", "2584", "4181",
        ],
    );

    assert_errs(
        "return 3;",
        "[line 1] Error: Can't return from top-level code.",
    );
    assert_errs(
        "if (true) return 3;",
        "[line 1] Error: Can't return from top-level code.",
    );
}
