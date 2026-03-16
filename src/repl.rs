//! Interactive Read-Eval-Print Loop for the Lingo programming language (Lisp dialect).

use std::io::{self, BufRead, Write};

use crate::interpreter::{Interpreter, Value};

const PRIMARY_PROMPT: &str = ">> ";
const CONTINUATION_PROMPT: &str = ".. ";

/// Count unmatched open parens in the input, respecting string literals.
fn unmatched_parens(input: &str) -> i32 {
    let mut count: i32 = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => count += 1,
            ')' => count -= 1,
            ';' => break, // rest of line is a comment
            _ => {}
        }
    }
    count
}

/// Start the REPL. Reads lines from stdin, evaluates them, and prints
/// non-Nil results to stdout. Errors are printed to stderr. The
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

        // Accumulate lines until parens are balanced
        while unmatched_parens(&buffer) > 0 {
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
        }

        match interpreter.eval_source(&buffer) {
            Ok(value) => {
                if !matches!(value, Value::Nil) {
                    println!("{}", value);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
