//! Interactive Read-Eval-Print Loop for the Lingo programming language.

use std::io::{self, BufRead, Write};

use crate::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::parser::Parser;

const PRIMARY_PROMPT: &str = ">> ";
const CONTINUATION_PROMPT: &str = ".. ";

/// Returns true if the parse error indicates unexpected EOF,
/// meaning the user's input is incomplete and more lines are needed.
fn is_incomplete_input(error: &str) -> bool {
    error.contains("None") || error.contains("Eof")
}

/// Start the REPL. Reads lines from stdin, evaluates them, and prints
/// non-Unit results to stdout. Errors are printed to stderr. The
/// interpreter state persists across inputs.
pub fn start() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock().lines();
    let mut interpreter = Interpreter::new();

    loop {
        print!("{}", PRIMARY_PROMPT);
        stdout.flush().map_err(|e| e.to_string())?;

        let line = match reader.next() {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(e.to_string()),
            None => break, // EOF
        };

        if line.trim().is_empty() {
            continue;
        }

        let mut buffer = line;

        loop {
            let mut lexer = Lexer::new(&buffer);
            let tokens = match lexer.tokenize() {
                Ok(tokens) => tokens,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            };

            let mut parser = Parser::new(tokens);
            match parser.parse_program() {
                Ok(program) => {
                    for item in &program.items {
                        match interpreter.eval_item(item) {
                            Ok(value) => {
                                if !matches!(value, Value::Unit) {
                                    println!("{}", value);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                    break;
                }
                Err(e) => {
                    if is_incomplete_input(&e) {
                        print!("{}", CONTINUATION_PROMPT);
                        stdout.flush().map_err(|e| e.to_string())?;

                        let next_line = match reader.next() {
                            Some(Ok(line)) => line,
                            Some(Err(e)) => return Err(e.to_string()),
                            None => break, // EOF during multi-line
                        };

                        if next_line.trim().is_empty() {
                            // Empty line cancels multi-line input
                            break;
                        }

                        buffer.push('\n');
                        buffer.push_str(&next_line);
                        // Continue the inner loop to re-parse
                    } else {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
