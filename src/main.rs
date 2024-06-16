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
    let mut interpreter = interpreter::interpreter();
    match args.len() {
        1 => {
            run_prompt(&mut interpreter);
            ExitCode::SUCCESS
        }
        2 => {
            let result = run_file(&mut interpreter, &args[1]);
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

fn run_file<F: FnMut(String)>(
    interpreter: &mut interpreter::Interpreter<F>,
    path: &str,
) -> Result<(), LoxError> {
    let contents = fs::read_to_string(path).unwrap();
    interpreter::evaluate_source(interpreter, &contents)
}

fn run_prompt<F: FnMut(String)>(interpreter: &mut interpreter::Interpreter<F>) {
    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();
        let size = stdin.read_line(&mut buffer).unwrap();
        if size == 0 {
            return;
        }
        let result = interpreter::evaluate_source(interpreter, &buffer);
        match result {
            Ok(()) => (),
            Err(err) => println!("{}", err),
        };
    }
}
