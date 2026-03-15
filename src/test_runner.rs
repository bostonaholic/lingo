/// Test runner for the Lingo programming language.
///
/// Discovers `test_*` functions, executes each in an isolated environment,
/// and reports results with pass/fail counts.

use crate::interpreter::{Interpreter, Value};

/// Summary of a test run, including captured output for testing.
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub output: String,
}

struct TestFailure {
    name: String,
    message: String,
}

/// Discover all `test_*` function names in the interpreter's environment.
/// Returns names sorted alphabetically for deterministic ordering.
pub fn discover_tests(interpreter: &Interpreter) -> Vec<String> {
    let mut test_names: Vec<String> = interpreter
        .env
        .binding_names()
        .into_iter()
        .filter(|name| name.starts_with("test_"))
        .filter(|name| {
            matches!(
                interpreter.env.get(name),
                Some(Value::Fn { .. })
            )
        })
        .collect();
    test_names.sort();
    test_names
}

/// Run all tests and print results to stdout. Returns `Err` if any test failed.
pub fn run_test_mode(interpreter: &mut Interpreter) -> Result<(), String> {
    let summary = run_test_mode_captured(interpreter);
    print!("{}", summary.output);

    if summary.failed > 0 {
        Err(format!("{} test(s) failed", summary.failed))
    } else {
        Ok(())
    }
}

/// Run all tests and capture output into a `TestSummary`.
/// Used by both the CLI entry point and integration tests.
pub fn run_test_mode_captured(interpreter: &mut Interpreter) -> TestSummary {
    let test_names = discover_tests(interpreter);
    let mut output = String::new();
    let mut passed: usize = 0;
    let mut failed: usize = 0;
    let mut failures: Vec<TestFailure> = Vec::new();

    for name in &test_names {
        let func = interpreter.env.get(name).unwrap();
        // Save environment before test for isolation
        let saved_env = interpreter.env.clone();
        let result = interpreter.call_function(&func, &[]);
        // Restore environment after test for isolation
        interpreter.env = saved_env;

        match result {
            Ok(_) => {
                output.push_str(&format!("PASS {}\n", name));
                passed += 1;
            }
            Err(msg) => {
                output.push_str(&format!("FAIL {}\n", name));
                failed += 1;
                failures.push(TestFailure {
                    name: name.clone(),
                    message: msg,
                });
            }
        }
    }

    // Print failure details
    if !failures.is_empty() {
        output.push('\n');
        output.push_str("failures:\n");
        for failure in &failures {
            output.push('\n');
            output.push_str(&format!("  {}:\n", failure.name));
            // Parse structured assertion messages (prefixed with "[assert] ")
            let msg = failure.message.strip_prefix("[assert] ").unwrap_or(&failure.message);
            // Format expected/got on separate indented lines if present
            if msg.contains("expected:") && msg.contains("got:") {
                // "expected: X, got: Y" -> split into two lines
                if let Some((expected_part, got_part)) = msg.split_once(", got: ") {
                    output.push_str(&format!("    {}\n", expected_part));
                    output.push_str(&format!("         got: {}\n", got_part));
                } else {
                    output.push_str(&format!("    {}\n", msg));
                }
            } else {
                output.push_str(&format!("    {}\n", msg));
            }
        }
    }

    // Summary line
    output.push_str(&format!("\n{} passed, {} failed\n", passed, failed));

    TestSummary {
        passed,
        failed,
        output,
    }
}
