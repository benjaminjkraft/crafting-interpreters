use crate::ast::*;
use crate::environment::Environment;
use crate::error::{runtime_error, LoxError};
use crate::object::{instance_get, BuiltinFunction, Class, Function, Instance, Literal, Object};
#[cfg(test)]
use crate::parser;
#[cfg(test)]
use crate::resolver;
use crate::scanner;
use crate::scanner::TokenType;
use crate::unwind::Unwinder;
use std::cell::RefCell;
use std::collections::HashMap;
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
        Ok(t) => Ok(Literal::Number(t.as_secs_f64()).into()),
        Err(e) => panic!("{e}"),
    }
}

pub fn interpreter<'ast, 'src: 'ast>() -> Interpreter<'ast, 'src, impl FnMut(String)> {
    let globals = Rc::new(RefCell::new(Environment::new()));
    globals.borrow_mut().define(
        "clock",
        BuiltinFunction {
            arity: 0,
            function: Rc::new(RefCell::new(|_| now_sec())),
            name: "clock".to_string(),
        }
        .into(),
    );
    Interpreter {
        printer: |s| println!("{s}"),
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
            Err(Unwinder::Return { keyword, value: _ }) => Err(runtime_error(
                keyword,
                "[resolver bug] Can't return from top-level code.",
            )),
        }
    }

    fn evaluate(&mut self, node: &Expr<'src>) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        match node {
            Expr::Assign(node) => {
                let value = self.evaluate(&node.value)?;
                match node.resolved_depth {
                    Some(depth) => {
                        self.environment.borrow_mut().assign_at(
                            depth,
                            &node.name,
                            value.clone(),
                        )?;
                    }

                    None => self
                        .globals
                        .borrow_mut()
                        .assign(&node.name, value.clone())?,
                }
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
                        let environment =
                            Rc::new(RefCell::new(Environment::child(f.closure.clone())));
                        if f.declaration.parameters.len() != arguments.len() {
                            // TODO: duplicated a bit
                            Unwinder::err(
                                &node.paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    f.declaration.parameters.len(),
                                    arguments.len()
                                ),
                            )
                        } else {
                            for (i, parameter) in f.declaration.parameters.iter().enumerate() {
                                environment
                                    .borrow_mut()
                                    .define(parameter.lexeme, arguments[i].clone());
                            }
                            let result = self.execute_stmts(&f.declaration.body, environment);
                            let r = match result {
                                Ok(()) => Ok(Literal::Nil.into()), // (omitted return)
                                Err(Unwinder::Err(e)) => Err(Unwinder::Err(e)),
                                Err(Unwinder::Return { keyword: _, value }) => Ok(value),
                            };
                            r
                        }
                    }
                    Object::Class(c) => {
                        if arguments.len() != 0 {
                            // TODO: duplicated a bit
                            Unwinder::err(
                                &node.paren,
                                &format!("Expected 0 arguments but got {}.", arguments.len()),
                            )
                        } else {
                            Ok(Rc::new(RefCell::new(Instance {
                                class_: c,
                                fields: HashMap::new(),
                            }))
                            .into())
                        }
                    }
                    o => Unwinder::err(
                        &node.paren,
                        &format!("Can only call functions and classes, got '{o}'."),
                    ),
                }
            }
            Expr::Get(node) => {
                let object = self.evaluate(&node.object)?;
                if let Object::Instance(obj) = object {
                    instance_get(obj, &node.name)
                } else {
                    Unwinder::err(
                        &node.name,
                        &format!("Only instances have properties, got '{object}'."),
                    )
                }
            }
            Expr::Grouping(node) => self.evaluate(&node.expr),
            Expr::Literal(node) => Ok(node.value.clone().into()),
            Expr::Logical(node) => {
                let left = self.evaluate(&node.left)?;
                match (node.operator.type_, left.is_truthy()) {
                    (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
                    (TokenType::Or, false) | (TokenType::And, true) => self.evaluate(&node.right),
                    _ => Unwinder::err(&node.operator, "unknown operator (parser bug?)"),
                }
            }
            Expr::Set(node) => {
                let object = self.evaluate(&node.object)?;
                if let Object::Instance(obj) = object {
                    let value = self.evaluate(&node.value)?;
                    obj.borrow_mut().set(&node.name, value.clone());
                    Ok(value)
                } else {
                    Unwinder::err(
                        &node.name,
                        &format!("Only instances have fields, got '{object}'."),
                    )
                }
            }
            Expr::This(node) => self.lookup_variable(&node.resolved_depth, &node.keyword),
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
            Expr::Variable(node) => self.lookup_variable(&node.resolved_depth, &node.name),
        }
    }

    fn lookup_variable(
        &self,
        resolved_depth: &Option<usize>,
        name: &scanner::Token<'src>,
    ) -> Result<Object<'ast, 'src>, Unwinder<'ast, 'src>> {
        match resolved_depth {
            Some(depth) => self.environment.borrow().get_at(*depth, name),
            None => self.globals.borrow().get(name),
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

    fn execute(&mut self, node: &'ast Stmt<'src>) -> Result<(), Unwinder<'ast, 'src>> {
        match node {
            Stmt::Block(node) => {
                let environment =
                    Rc::new(RefCell::new(Environment::child(self.environment.clone())));
                self.execute_stmts(&node.stmts, environment)?;
            }

            Stmt::Class(node) => {
                self.environment
                    .borrow_mut()
                    .define(node.name.lexeme, Literal::Nil.into());

                let mut methods = HashMap::new();
                for method in &node.methods {
                    let function = Function {
                        declaration: method,
                        closure: self.environment.clone(),
                    };
                    methods.insert(method.name.lexeme.to_string(), function);
                }
                let class_ = Rc::new(RefCell::new(Class {
                    name: &node.name,
                    methods,
                }))
                .into();

                self.environment.borrow_mut().assign(&node.name, class_)?;
            }

            Stmt::Expr(node) => {
                self.evaluate(&node.expr)?;
            }

            Stmt::Function(node) => {
                let function = Function {
                    declaration: node,
                    closure: self.environment.clone(),
                }
                .into();
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
                let stringified = format!("{value}");
                (self.printer)(stringified);
            }
            Stmt::Return(node) => {
                let value = match &node.value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::Nil.into(),
                };
                Err(Unwinder::Return {
                    keyword: &node.keyword,
                    value,
                })?;
            }
            Stmt::Var(node) => {
                let value = match &node.initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::Nil.into(),
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
    let mut prog = parser::parse(tokens)?;
    resolver::resolve(&mut prog)?;
    {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals.borrow_mut().define(
            "clock",
            BuiltinFunction {
                arity: 0,
                function: Rc::new(RefCell::new(move |_| {
                    time += 1.0;
                    Ok(Object::Literal(Literal::Number(time)))
                })),
                name: "clock".to_string(),
            }
            .into(),
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
        Err(a) => assert!(false, "Expected {expected:?}, got error {a}"),
    }
}

#[cfg(test)]
fn assert_errs(source: &str, expected: &str) {
    match execute_for_tests(source) {
        Ok(a) => assert!(false, "Expected error {expected}, got {a:?}"),
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
    assert_errs(
        "var a = 1; { var a = a + 2; print a; } print a;",
        "[line 1] Error at 'a': Can't read local variable in its own initializer.",
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
    assert_prints("print clock == clock;", &["true"]);
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
    assert_prints("fun f() {} print f == f;", &["true"]);
    assert_prints("fun f() {} var g = f; fun f() {} print g == f;", &["false"]);
    assert_prints(
        "var a = clock; fun clock() {} print a == clock;",
        &["false"],
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
        "[line 1] Error at 'return': Can't return from top-level code.",
    );
    assert_errs(
        "if (true) return 3;",
        "[line 1] Error at 'return': Can't return from top-level code.",
    );
}

#[test]
fn test_closures() {
    assert_prints(
        r"
            fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    return i;
                }

                return count;
            }
            var counter1 = makeCounter();
            var counter2 = makeCounter();
            print counter1();
            print counter1();
            print counter2();
            print counter2();
        ",
        &["1", "2", "1", "2"],
    );
}

#[test]
fn test_scoping() {
    assert_prints(
        r#"
            var a = "global";
            {
                fun showA() {
                    print a;
                }

                showA();
                var a = "block";
                showA();
            }
        "#,
        &["global", "global"],
    );
    assert_prints("var a = 1; var a = 2; print a;", &["2"]);
    assert_errs(
        "var a = 1; { var a = a; }",
        "[line 1] Error at 'a': Can't read local variable in its own initializer.",
    );
    assert_errs(
        "{ var a = 1;\nvar a = a; }",
        &("[line 2] Error at 'a': Already a variable with this name in this scope.\n".to_string()
            + "[line 2] Error at 'a': Can't read local variable in its own initializer."),
    );
    assert_errs(
        "{ var a = 1;\nvar a = 2; }",
        "[line 2] Error at 'a': Already a variable with this name in this scope.",
    );
}

#[test]
fn test_class() {
    assert_prints("class C {} print C;", &["<class C>"]);
    assert_prints("class C {} print C == C;", &["true"]);
    assert_prints("class C {} var a = C; class C {} print a == C;", &["false"]);

    assert_prints("class C {} var i = C(); print i;", &["<instance of C>"]);
    assert_prints("class C {} var i = C(); print i == i;", &["true"]);
    assert_prints(
        "class C {} var i = C(); var j = C(); print i == j;",
        &["false"],
    );
}

#[test]
fn test_fields() {
    assert_prints("class C {} var i = C(); i.f = 1; print i.f;", &["1"]);
    assert_prints(
        "class C {} var i = C(); i.f = 1; i.f = i.f + 1; print i.f;",
        &["2"],
    );

    assert_errs(
        "var a = 1; a.f = 1;",
        "[line 1] Error: Only instances have fields, got '1'.",
    );
    assert_errs(
        "var a = 1; print a.f;",
        "[line 1] Error: Only instances have properties, got '1'.",
    );
    assert_errs(
        "class C {} var i = C(); print i.f;",
        "[line 1] Error: Undefined property 'f'.",
    );
}

#[test]
fn test_methods() {
    assert_prints(
        r#"
            fun f() { print "free function"; return "free"; }
            class C {
                f() { print "in f"; return "ret"; }
            }
            var i = C();
            print i.f();
            i.f = f;
            print i.f();
        "#,
        &["in f", "ret", "free function", "free"],
    );
    assert_errs(
        r#"
            fun f() { print "free function"; return "free"; }
            class C {}
            var i = C();
            print i.f();
        "#,
        "[line 5] Error: Undefined property 'f'.",
    );
}

#[test]
fn test_this() {
    assert_prints(
        r#"
            class C {
                f() { print this.v; return this.r; }
                g() { print this.f(); }
                s() { i.v = "v2"; i.r = "r2"; }
            }
            var i = C();
            i.v = "v";
            i.r = "r";
            print i.f();
            print i.g();
            i.s();
            print i.f();
            print i.g();
            var f = i.f;
            var g = i.g;
            print f();
            print g();
        "#,
        &[
            "v", "r", "v", "r", "nil", "v2", "r2", "v2", "r2", "nil", "v2", "r2", "v2", "r2", "nil",
        ],
    );

    assert_prints(
        r#"
            class Cake {
                taste() {
                    var adjective = "delicious";
                    print "The " + this.flavor + " cake is " + adjective + "!";
                }
            }

            var cake = Cake();
            cake.flavor = "German chocolate";
            cake.taste();
        "#,
        &["The German chocolate cake is delicious!"],
    );
    assert_prints(
        r#"
            class Egotist {
                speak() {
                    print this;
                }
            }

            var method = Egotist().speak;
            method();
            "#,
        &["<instance of Egotist>"],
    );

    assert_prints(
        r#"
            class C {
                f() {
                    fun g() {
                        return this;
                    }
                    return g;
                }
            }
            var i = C();
            print i == i.f()();
        "#,
        &["true"],
    );
    assert_prints(
        r#"
            class Outer {
                f() {
                    class Inner {
                        g() {
                            return this;
                        }
                    }
                    var i = Inner();
                    i.outer = this;
                    return i;
                }
            }
            var o = Outer();
            var i = o.f();
            print i == i.g();
            print o == i.outer;
        "#,
        &["true", "true"],
    );

    assert_errs(
        "print this;",
        "[line 1] Error at 'this': Can't use 'this' outside of a class.",
    );
    assert_errs(
        "fun f() { print this; }",
        "[line 1] Error at 'this': Can't use 'this' outside of a class.",
    );
}
