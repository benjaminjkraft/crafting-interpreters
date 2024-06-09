use crate::error::LoxError;
use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;

mod ast;
mod ast_printer;
mod error;
mod interpreter;
mod object;
mod parser;
mod scanner;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut interpreter = interpreter::Interpreter {};
    let result = match args.len() {
        1 => run_prompt(&mut interpreter),
        2 => run_file(&mut interpreter, &args[1]),
        _ => {
            return ExitCode::from(64);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            println!("{}", err);
            ExitCode::from(err.exit)
        }
    }
}

fn run_file(interpreter: &mut interpreter::Interpreter, path: &str) -> Result<(), LoxError> {
    let contents = fs::read_to_string(path).unwrap();
    run(interpreter, contents)
}

fn run_prompt(interpreter: &mut interpreter::Interpreter) -> Result<(), LoxError> {
    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();
        let size = stdin.read_line(&mut buffer).unwrap();
        if size == 0 {
            return Ok(());
        }
        let err = run(interpreter, buffer);
        match err {
            Ok(()) => (),
            Err(err) => println!("{}", err),
        }
    }
}

fn run(interpreter: &mut interpreter::Interpreter, source: String) -> Result<(), LoxError> {
    let value = interpreter::evaluate_source(interpreter, &source)?;
    println!("{}", interpreter.stringify(value));
    Ok(())
}
