use std::io::Write;
use std::process::Command;

use lingo::interpreter::Interpreter;
use lingo::lexer::Lexer;
use lingo::parser::Parser;
use lingo::test_runner;

/// Helper: parse and run a Lingo source string, returning the interpreter result.
fn run_lingo(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)
}

/// Helper: parse source and load declarations into an interpreter (without calling main).
fn load_declarations(source: &str) -> Interpreter {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.load_declarations(&program);
    interpreter
}

// ---------------------------------------------------------------------------
// Step 1.1: assert_eq tests
// ---------------------------------------------------------------------------

#[test]
fn assert_eq_equal_integers() {
    let result = run_lingo("fn main() { assert_eq(1, 1) }");
    assert!(result.is_ok(), "assert_eq(1, 1) should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_strings() {
    let result = run_lingo(r#"fn main() { assert_eq("hello", "hello") }"#);
    assert!(result.is_ok(), "assert_eq with equal strings should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_lists() {
    let result = run_lingo("fn main() { assert_eq([1, 2], [1, 2]) }");
    assert!(result.is_ok(), "assert_eq with equal lists should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_tuples() {
    let result = run_lingo("fn main() { assert_eq((1, 2), (1, 2)) }");
    assert!(result.is_ok(), "assert_eq with equal tuples should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_booleans() {
    let result = run_lingo("fn main() { assert_eq(true, true) }");
    assert!(result.is_ok(), "assert_eq(true, true) should pass: {:?}", result);
}

#[test]
fn assert_eq_unequal_integers() {
    let result = run_lingo("fn main() { assert_eq(1, 2) }");
    assert!(result.is_err(), "assert_eq(1, 2) should fail");
    let err = result.unwrap_err();
    assert!(err.contains("expected"), "error should contain 'expected': {}", err);
    assert!(err.contains("got"), "error should contain 'got': {}", err);
}

#[test]
fn assert_eq_unequal_strings() {
    let result = run_lingo(r#"fn main() { assert_eq("a", "b") }"#);
    assert!(result.is_err(), "assert_eq with unequal strings should fail");
    let err = result.unwrap_err();
    assert!(err.contains("expected"), "error should contain 'expected': {}", err);
    assert!(err.contains("got"), "error should contain 'got': {}", err);
}

#[test]
fn assert_eq_unequal_lists() {
    let result = run_lingo("fn main() { assert_eq([1], [1, 2]) }");
    assert!(result.is_err(), "assert_eq with unequal lists should fail");
}

#[test]
fn assert_eq_cross_type() {
    let result = run_lingo(r#"fn main() { assert_eq(1, "1") }"#);
    assert!(result.is_err(), "assert_eq with cross-type should fail");
}

// ---------------------------------------------------------------------------
// Step 1.2: assert_ne tests
// ---------------------------------------------------------------------------

#[test]
fn assert_ne_unequal_integers() {
    let result = run_lingo("fn main() { assert_ne(1, 2) }");
    assert!(result.is_ok(), "assert_ne(1, 2) should pass: {:?}", result);
}

#[test]
fn assert_ne_unequal_strings() {
    let result = run_lingo(r#"fn main() { assert_ne("a", "b") }"#);
    assert!(result.is_ok(), "assert_ne with unequal strings should pass: {:?}", result);
}

#[test]
fn assert_ne_equal_integers() {
    let result = run_lingo("fn main() { assert_ne(1, 1) }");
    assert!(result.is_err(), "assert_ne(1, 1) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected values to differ"),
        "error should contain 'expected values to differ': {}",
        err
    );
}

#[test]
fn assert_ne_equal_lists() {
    let result = run_lingo("fn main() { assert_ne([1, 2], [1, 2]) }");
    assert!(result.is_err(), "assert_ne with equal lists should fail");
}

// ---------------------------------------------------------------------------
// Step 1.3: assert_true and assert_false tests
// ---------------------------------------------------------------------------

#[test]
fn assert_true_with_true() {
    let result = run_lingo("fn main() { assert_true(true) }");
    assert!(result.is_ok(), "assert_true(true) should pass: {:?}", result);
}

#[test]
fn assert_true_with_nonzero_int() {
    let result = run_lingo("fn main() { assert_true(1) }");
    assert!(result.is_ok(), "assert_true(1) should pass: {:?}", result);
}

#[test]
fn assert_true_with_nonempty_string() {
    let result = run_lingo(r#"fn main() { assert_true("hello") }"#);
    assert!(result.is_ok(), "assert_true with non-empty string should pass: {:?}", result);
}

#[test]
fn assert_true_with_false() {
    let result = run_lingo("fn main() { assert_true(false) }");
    assert!(result.is_err(), "assert_true(false) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected truthy"),
        "error should contain 'expected truthy': {}",
        err
    );
}

#[test]
fn assert_true_with_zero() {
    let result = run_lingo("fn main() { assert_true(0) }");
    assert!(result.is_err(), "assert_true(0) should fail");
}

#[test]
fn assert_true_with_empty_tuple() {
    // In Lingo, () is an empty tuple (not Unit). Empty tuples are truthy
    // per the is_truthy implementation (falls through to _ => true).
    let result = run_lingo("fn main() { assert_true(()) }");
    assert!(result.is_ok(), "assert_true(()) should pass (empty tuple is truthy): {:?}", result);
}

#[test]
fn assert_false_with_false() {
    let result = run_lingo("fn main() { assert_false(false) }");
    assert!(result.is_ok(), "assert_false(false) should pass: {:?}", result);
}

#[test]
fn assert_false_with_zero() {
    let result = run_lingo("fn main() { assert_false(0) }");
    assert!(result.is_ok(), "assert_false(0) should pass: {:?}", result);
}

#[test]
fn assert_false_with_true() {
    let result = run_lingo("fn main() { assert_false(true) }");
    assert!(result.is_err(), "assert_false(true) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected falsy"),
        "error should contain 'expected falsy': {}",
        err
    );
}

#[test]
fn assert_false_with_nonzero_int() {
    let result = run_lingo("fn main() { assert_false(1) }");
    assert!(result.is_err(), "assert_false(1) should fail");
}

#[test]
fn assert_false_with_nonempty_string() {
    let result = run_lingo(r#"fn main() { assert_false("hello") }"#);
    assert!(result.is_err(), r#"assert_false("hello") should fail"#);
}

// ---------------------------------------------------------------------------
// Step 2.1: Test discovery logic
// ---------------------------------------------------------------------------

#[test]
fn discovery_finds_test_functions() {
    let interp = load_declarations("fn test_a() { } fn test_b() { }");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test_a", "test_b"]);
}

#[test]
fn discovery_ignores_non_test_functions() {
    let interp = load_declarations("fn helper() { } fn test_a() { }");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test_a"]);
}

#[test]
fn discovery_ignores_testing_prefix() {
    let interp = load_declarations("fn testing_thing() { }");
    let tests = test_runner::discover_tests(&interp);
    assert!(tests.is_empty(), "testing_thing should not be discovered");
}

#[test]
fn discovery_empty_when_no_tests() {
    let interp = load_declarations("fn helper() { }");
    let tests = test_runner::discover_tests(&interp);
    assert!(tests.is_empty());
}

#[test]
fn discovery_alphabetical_order() {
    let interp = load_declarations("fn test_z() { } fn test_a() { } fn test_m() { }");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test_a", "test_m", "test_z"]);
}

// ---------------------------------------------------------------------------
// Step 2.2: Test failure isolation
// ---------------------------------------------------------------------------

#[test]
fn isolation_fail_does_not_prevent_pass() {
    let interp = load_declarations(
        "fn test_fail() { assert_eq(1, 2) } fn test_pass() { assert_eq(1, 1) }",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1, "1 test should pass");
    assert_eq!(summary.failed, 1, "1 test should fail");
}

#[test]
fn isolation_error_continues_to_next() {
    let interp = load_declarations(
        "fn test_error() { undefined_var } fn test_ok() { assert_eq(1, 1) }",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
}

// ---------------------------------------------------------------------------
// Step 2.3: Test reporter output
// ---------------------------------------------------------------------------

#[test]
fn reporter_all_pass_output() {
    let interp = load_declarations(
        "fn test_a() { assert_eq(1, 1) } fn test_b() { assert_eq(2, 2) }",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 0);
    assert!(summary.output.contains("PASS test_a"), "output should contain PASS test_a: {}", summary.output);
    assert!(summary.output.contains("PASS test_b"), "output should contain PASS test_b: {}", summary.output);
    assert!(summary.output.contains("2 passed, 0 failed"), "output should contain summary: {}", summary.output);
}

#[test]
fn reporter_mixed_output() {
    let interp = load_declarations(
        "fn test_pass() { assert_eq(1, 1) } fn test_fail() { assert_eq(1, 2) }",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    assert!(summary.output.contains("PASS test_pass"), "should contain PASS: {}", summary.output);
    assert!(summary.output.contains("FAIL test_fail"), "should contain FAIL: {}", summary.output);
    assert!(summary.output.contains("failures:"), "should contain failures section: {}", summary.output);
    assert!(summary.output.contains("1 passed, 1 failed"), "should contain summary: {}", summary.output);
}

#[test]
fn reporter_zero_tests_output() {
    let interp = load_declarations("fn helper() { }");
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert!(summary.output.contains("0 passed, 0 failed"), "should contain summary: {}", summary.output);
}

// ---------------------------------------------------------------------------
// Step 3.1: CLI --test flag tests
// ---------------------------------------------------------------------------

/// Helper: write source to a temporary .ln file and return its path.
fn write_temp_ln(source: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(".ln")
        .tempfile()
        .expect("failed to create temp file");
    f.write_all(source.as_bytes())
        .expect("failed to write temp file");
    f
}

/// Helper: get the path to the built binary.
/// Cargo automatically builds the binary for integration tests and sets
/// the CARGO_BIN_EXE_<name> env var.
fn cargo_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_lingo"))
}

#[test]
fn cli_test_flag_all_pass_exits_0() {
    let file = write_temp_ln(
        "fn test_a() { assert_eq(1, 1) } fn test_b() { assert_eq(2, 2) }",
    );
    let output = Command::new(cargo_bin())
        .args(["--test", file.path().to_str().unwrap()])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "should exit 0 when all tests pass. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 passed, 0 failed"), "stdout: {}", stdout);
}

#[test]
fn cli_test_flag_failure_exits_1() {
    let file = write_temp_ln("fn test_fail() { assert_eq(1, 2) }");
    let output = Command::new(cargo_bin())
        .args(["--test", file.path().to_str().unwrap()])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "should exit 1 when a test fails"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 passed, 1 failed"), "stdout: {}", stdout);
}

#[test]
fn cli_test_flag_without_filename_errors() {
    let output = Command::new(cargo_bin())
        .args(["--test"])
        .output()
        .expect("failed to execute");
    assert!(
        !output.status.success(),
        "should exit non-zero when no filename given"
    );
}

#[test]
fn cli_normal_mode_unchanged() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let hello_path = format!("{}/examples/hello.ln", manifest_dir);
    let output = Command::new(cargo_bin())
        .args([&hello_path])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "normal mode should still work. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"), "stdout: {}", stdout);
}
