#![allow(dead_code)]

mod ast;
mod interpreter;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: lingo <file.ln>");
        process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    if let Err(e) = run(&source) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(source: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.run(&program)?;

    Ok(())
}
