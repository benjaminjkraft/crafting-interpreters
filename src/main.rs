use crate::error::LoxError;
use std::env;
use std::fs;
use std::io;

mod ast;
mod ast_printer;
mod error;
mod object;
mod parser;
mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => panic!("Usage: rlox [script]"),
    }
}

fn run_file(path: &str) {
    let contents = fs::read_to_string(path).unwrap();
    let err = run(contents);
    match err {
        Ok(()) => (),
        Err(err) => panic!("{}", err),
    }
}

fn run_prompt() {
    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();
        let size = stdin.read_line(&mut buffer).unwrap();
        if size == 0 {
            return;
        }
        let err = run(buffer);
        match err {
            Ok(()) => (),
            Err(err) => println!("{}", err),
        }
    }
}

fn run(source: String) -> Result<(), LoxError> {
    let tokens = scanner::scan_tokens(&source)?;
    let expr = parser::parse(tokens)?;
    println!("{}", ast_printer::print(expr));
    Ok(())
}
