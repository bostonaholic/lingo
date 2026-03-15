use std::env;
use std::fs;
use std::process;

use lingo::interpreter;
use lingo::lexer;
use lingo::parser;
use lingo::repl;
use lingo::test_runner;

fn main() {
    let args: Vec<String> = env::args().collect();

    let test_mode = args.iter().any(|a| a == "--test");
    let filename = args.iter().skip(1).find(|a| *a != "--test");

    match (test_mode, filename) {
        (true, None) => {
            eprintln!("Usage: lingo --test <file.ln>");
            process::exit(1);
        }
        (false, None) => {
            if let Err(e) = repl::start() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        (true, Some(filename)) => {
            let source = read_file(filename);
            if let Err(e) = run_tests(&source) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        (false, Some(filename)) => {
            let source = read_file(filename);
            if let Err(e) = run(&source) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn read_file(filename: &str) -> String {
    match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
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
