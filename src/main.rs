use crate::error::LoxError;
use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;

mod ast;
mod ast_printer;
mod environment;
mod error;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod unwind;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            run_prompt();
            ExitCode::SUCCESS
        }
        2 => {
            let result = run_file(&args[1]);
            match result {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    println!("{}", err);
                    ExitCode::from(err.exit)
                }
            }
        }
        _ => ExitCode::from(64),
    }
}

fn run_file(path: &str) -> Result<(), LoxError> {
    let source = fs::read_to_string(path).unwrap();
    let tokens = scanner::scan_tokens(&source)?;
    let prog = parser::parse(tokens)?;
    let mut interpreter = interpreter::interpreter();
    interpreter.execute_program(&prog)
}

fn read_line() -> Option<String> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let size = stdin.read_line(&mut buffer).unwrap();
    if size == 0 {
        None
    } else {
        Some(buffer)
    }
}

// Leaks the source because in a REPL we do actually need to keep
// the source forever (for functions). Just leaking explicitly is
// easier than trying to track a reference that's basically the
// life of the program anyway.
fn execute_and_leak_source<'ast, 'src: 'ast, F: FnMut(String)>(
    interpreter: &mut interpreter::Interpreter<'ast, 'src, F>,
    source: String,
) -> Result<(), LoxError> {
    let tokens = scanner::scan_tokens(String::leak(source))?;
    let prog = parser::parse(tokens)?;
    interpreter.execute_program(Box::leak(Box::new(prog)))
}

fn run_prompt() {
    let mut interpreter = interpreter::interpreter();

    loop {
        let source = match read_line() {
            None => return,
            Some(s) => s,
        };
        let result = execute_and_leak_source(&mut interpreter, source);
        match result {
            Ok(()) => (),
            Err(err) => println!("{}", err),
        };
    }
}
