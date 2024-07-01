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
    let contents = fs::read_to_string(path).unwrap();
    let mut interpreter = interpreter::interpreter();
    interpreter::evaluate_source(&mut interpreter, &contents)
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

fn run_prompt() {
    let mut interpreter = interpreter::interpreter();

    loop {
        let source = match read_line() {
            None => return,
            Some(s) => s,
        };
        let result = interpreter::evaluate_source(&mut interpreter, String::leak(source));
        match result {
            Ok(()) => (),
            Err(err) => println!("{}", err),
        };
    }
}
