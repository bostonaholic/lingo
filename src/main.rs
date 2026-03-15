use std::env;
use std::fs;
use std::process;

use lingo::interpreter;
use lingo::lexer;
use lingo::parser;
use lingo::test_runner;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: lingo [--test] <file.ln>");
        process::exit(1);
    }

    let test_mode = args.iter().any(|a| a == "--test");
    let filename = args
        .iter()
        .skip(1)
        .find(|a| *a != "--test");

    let filename = match filename {
        Some(f) => f,
        None => {
            eprintln!("Usage: lingo [--test] <file.ln>");
            process::exit(1);
        }
    };

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    if test_mode {
        if let Err(e) = run_tests(&source) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = run(&source) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
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

fn run_tests(source: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.load_declarations(&program);

    test_runner::run_test_mode(&mut interpreter)
}
