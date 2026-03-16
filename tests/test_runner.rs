use std::io::Write;
use std::process::Command;

use lingo::interpreter::Interpreter;
use lingo::test_runner;

/// Helper: run a Lingo source string (lex, parse, evaluate, call main if defined).
fn run_lingo(source: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    interpreter.run(source)
}

/// Helper: load source into an interpreter without calling main.
fn load_declarations(source: &str) -> Interpreter {
    let mut interpreter = Interpreter::new();
    interpreter.load_source(source);
    interpreter
}

// ---------------------------------------------------------------------------
// Step 3.2: assert_eq tests
// ---------------------------------------------------------------------------

#[test]
fn assert_eq_equal_integers() {
    let result = run_lingo("(defn main () (assert-eq 1 1))");
    assert!(result.is_ok(), "assert-eq(1, 1) should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_strings() {
    let result = run_lingo(r#"(defn main () (assert-eq "hello" "hello"))"#);
    assert!(result.is_ok(), "assert-eq with equal strings should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_lists() {
    let result = run_lingo("(defn main () (assert-eq (list 1 2) (list 1 2)))");
    assert!(result.is_ok(), "assert-eq with equal lists should pass: {:?}", result);
}

#[test]
fn assert_eq_nested_lists() {
    let result = run_lingo("(defn main () (assert-eq (list 1 (list 2 3)) (list 1 (list 2 3))))");
    assert!(result.is_ok(), "assert-eq with nested lists should pass: {:?}", result);
}

#[test]
fn assert_eq_equal_booleans() {
    let result = run_lingo("(defn main () (assert-eq true true))");
    assert!(result.is_ok(), "assert-eq(true, true) should pass: {:?}", result);
}

#[test]
fn assert_eq_unequal_integers() {
    let result = run_lingo("(defn main () (assert-eq 1 2))");
    assert!(result.is_err(), "assert-eq(1, 2) should fail");
    let err = result.unwrap_err();
    assert!(err.contains("expected"), "error should contain 'expected': {}", err);
    assert!(err.contains("got"), "error should contain 'got': {}", err);
}

#[test]
fn assert_eq_unequal_strings() {
    let result = run_lingo(r#"(defn main () (assert-eq "a" "b"))"#);
    assert!(result.is_err(), "assert-eq with unequal strings should fail");
    let err = result.unwrap_err();
    assert!(err.contains("expected"), "error should contain 'expected': {}", err);
    assert!(err.contains("got"), "error should contain 'got': {}", err);
}

#[test]
fn assert_eq_unequal_lists() {
    let result = run_lingo("(defn main () (assert-eq (list 1) (list 1 2)))");
    assert!(result.is_err(), "assert-eq with unequal lists should fail");
}

#[test]
fn assert_eq_cross_type() {
    let result = run_lingo(r#"(defn main () (assert-eq 1 "1"))"#);
    assert!(result.is_err(), "assert-eq with cross-type should fail");
}

// ---------------------------------------------------------------------------
// Step 3.3: assert_ne tests
// ---------------------------------------------------------------------------

#[test]
fn assert_ne_unequal_integers() {
    let result = run_lingo("(defn main () (assert-ne 1 2))");
    assert!(result.is_ok(), "assert-ne(1, 2) should pass: {:?}", result);
}

#[test]
fn assert_ne_unequal_strings() {
    let result = run_lingo(r#"(defn main () (assert-ne "a" "b"))"#);
    assert!(result.is_ok(), "assert-ne with unequal strings should pass: {:?}", result);
}

#[test]
fn assert_ne_equal_integers() {
    let result = run_lingo("(defn main () (assert-ne 1 1))");
    assert!(result.is_err(), "assert-ne(1, 1) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected values to differ"),
        "error should contain 'expected values to differ': {}",
        err
    );
}

#[test]
fn assert_ne_equal_lists() {
    let result = run_lingo("(defn main () (assert-ne (list 1 2) (list 1 2)))");
    assert!(result.is_err(), "assert-ne with equal lists should fail");
}

// ---------------------------------------------------------------------------
// Step 3.4: assert_true and assert_false tests
// ---------------------------------------------------------------------------

#[test]
fn assert_true_with_true() {
    let result = run_lingo("(defn main () (assert-true true))");
    assert!(result.is_ok(), "assert-true(true) should pass: {:?}", result);
}

#[test]
fn assert_true_with_nonzero_int() {
    let result = run_lingo("(defn main () (assert-true 1))");
    assert!(result.is_ok(), "assert-true(1) should pass: {:?}", result);
}

#[test]
fn assert_true_with_nonempty_string() {
    let result = run_lingo(r#"(defn main () (assert-true "hello"))"#);
    assert!(result.is_ok(), "assert-true with non-empty string should pass: {:?}", result);
}

#[test]
fn assert_true_with_false() {
    let result = run_lingo("(defn main () (assert-true false))");
    assert!(result.is_err(), "assert-true(false) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected truthy"),
        "error should contain 'expected truthy': {}",
        err
    );
}

#[test]
fn assert_true_with_zero() {
    let result = run_lingo("(defn main () (assert-true 0))");
    assert!(result.is_err(), "assert-true(0) should fail");
}

#[test]
fn assert_true_with_nonempty_list() {
    let result = run_lingo("(defn main () (assert-true (list 1)))");
    assert!(result.is_ok(), "assert-true with non-empty list should pass: {:?}", result);
}

#[test]
fn assert_false_with_false() {
    let result = run_lingo("(defn main () (assert-false false))");
    assert!(result.is_ok(), "assert-false(false) should pass: {:?}", result);
}

#[test]
fn assert_false_with_zero() {
    let result = run_lingo("(defn main () (assert-false 0))");
    assert!(result.is_ok(), "assert-false(0) should pass: {:?}", result);
}

#[test]
fn assert_false_with_true() {
    let result = run_lingo("(defn main () (assert-false true))");
    assert!(result.is_err(), "assert-false(true) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("expected falsy"),
        "error should contain 'expected falsy': {}",
        err
    );
}

#[test]
fn assert_false_with_nonzero_int() {
    let result = run_lingo("(defn main () (assert-false 1))");
    assert!(result.is_err(), "assert-false(1) should fail");
}

#[test]
fn assert_false_with_nonempty_string() {
    let result = run_lingo(r#"(defn main () (assert-false "hello"))"#);
    assert!(result.is_err(), r#"assert-false("hello") should fail"#);
}

// ---------------------------------------------------------------------------
// Step 3.5: Test discovery tests
// ---------------------------------------------------------------------------

#[test]
fn discovery_finds_test_functions() {
    let interp = load_declarations("(defn test-a () nil) (defn test-b () nil)");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test-a", "test-b"]);
}

#[test]
fn discovery_ignores_non_test_functions() {
    let interp = load_declarations("(defn helper () nil) (defn test-a () nil)");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test-a"]);
}

#[test]
fn discovery_ignores_testing_prefix() {
    let interp = load_declarations("(defn testing-thing () nil)");
    let tests = test_runner::discover_tests(&interp);
    assert!(tests.is_empty(), "testing-thing should not be discovered");
}

#[test]
fn discovery_empty_when_no_tests() {
    let interp = load_declarations("(defn helper () nil)");
    let tests = test_runner::discover_tests(&interp);
    assert!(tests.is_empty());
}

#[test]
fn discovery_alphabetical_order() {
    let interp = load_declarations("(defn test-z () nil) (defn test-a () nil) (defn test-m () nil)");
    let tests = test_runner::discover_tests(&interp);
    assert_eq!(tests, vec!["test-a", "test-m", "test-z"]);
}

// ---------------------------------------------------------------------------
// Step 3.6: Test isolation and reporter tests
// ---------------------------------------------------------------------------

#[test]
fn isolation_fail_does_not_prevent_pass() {
    let interp = load_declarations(
        "(defn test-fail () (assert-eq 1 2)) (defn test-pass () (assert-eq 1 1))",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1, "1 test should pass");
    assert_eq!(summary.failed, 1, "1 test should fail");
}

#[test]
fn isolation_error_continues_to_next() {
    let interp = load_declarations(
        "(defn test-error () undefined-var) (defn test-ok () (assert-eq 1 1))",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
}

#[test]
fn reporter_all_pass_output() {
    let interp = load_declarations(
        "(defn test-a () (assert-eq 1 1)) (defn test-b () (assert-eq 2 2))",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 0);
    assert!(summary.output.contains("PASS test-a"), "output should contain PASS test-a: {}", summary.output);
    assert!(summary.output.contains("PASS test-b"), "output should contain PASS test-b: {}", summary.output);
    assert!(summary.output.contains("2 passed, 0 failed"), "output should contain summary: {}", summary.output);
}

#[test]
fn reporter_mixed_output() {
    let interp = load_declarations(
        "(defn test-pass () (assert-eq 1 1)) (defn test-fail () (assert-eq 1 2))",
    );
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    assert!(summary.output.contains("PASS test-pass"), "should contain PASS: {}", summary.output);
    assert!(summary.output.contains("FAIL test-fail"), "should contain FAIL: {}", summary.output);
    assert!(summary.output.contains("failures:"), "should contain failures section: {}", summary.output);
    assert!(summary.output.contains("1 passed, 1 failed"), "should contain summary: {}", summary.output);
}

#[test]
fn reporter_zero_tests_output() {
    let interp = load_declarations("(defn helper () nil)");
    let summary = test_runner::run_test_mode_captured(&mut { interp });
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert!(summary.output.contains("0 passed, 0 failed"), "should contain summary: {}", summary.output);
}

// ---------------------------------------------------------------------------
// Step 3.7: CLI --test flag tests
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
fn cargo_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_lingo"))
}

#[test]
fn cli_test_flag_all_pass_exits_0() {
    let file = write_temp_ln(
        "(defn test-a () (assert-eq 1 1)) (defn test-b () (assert-eq 2 2))",
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
    let file = write_temp_ln("(defn test-fail () (assert-eq 1 2))");
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

// ---------------------------------------------------------------------------
// Step 3.8: eval_source tests (replaces eval_item tests)
// ---------------------------------------------------------------------------

#[test]
fn eval_item_expression_returns_value() {
    let mut interp = Interpreter::new();
    let result = interp.eval_source("(+ 1 2)");
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "3");
}

#[test]
fn eval_item_string_returns_value() {
    let mut interp = Interpreter::new();
    let result = interp.eval_source(r#""hello""#);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "hello");
}

#[test]
fn eval_item_let_returns_unit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_source("(def x 5)");
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "nil");
}

#[test]
fn eval_item_fn_decl_returns_unit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_source("(defn foo () 42)");
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "nil");
}

#[test]
fn eval_item_fn_decl_then_call() {
    let mut interp = Interpreter::new();
    interp.eval_source("(defn foo () 42)").unwrap();
    let result = interp.eval_source("(foo)");
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "42");
}

#[test]
fn eval_item_let_then_use() {
    let mut interp = Interpreter::new();
    interp.eval_source("(def x 10)").unwrap();
    let result = interp.eval_source("x");
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "10");
}

#[test]
fn eval_item_undefined_variable_returns_err() {
    let mut interp = Interpreter::new();
    let result = interp.eval_source("undefined-var");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Step 3.9: REPL integration tests
// ---------------------------------------------------------------------------

use std::process::Stdio;

#[test]
fn repl_expression_prints_result() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"(+ 1 2)\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout should contain 3: {}", stdout);
}

#[test]
fn repl_state_persists_across_inputs() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"(def x 5)\nx\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("5"), "stdout should contain 5: {}", stdout);
}

#[test]
fn repl_fn_declarations_persist() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"(defn add (a b) (+ a b))\n(add 1 2)\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout should contain 3: {}", stdout);
}

#[test]
fn repl_eof_exits_cleanly() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    // Close stdin immediately (EOF)
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    assert!(
        output.status.success(),
        "REPL should exit 0 on EOF, got: {:?}",
        output.status
    );
}

#[test]
fn repl_error_recovery() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"undefined-var\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "stderr should contain an error message"
    );
}

#[test]
fn repl_continues_after_error() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"(+ 1 2)\nundefined-var\n(+ 3 4)\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "stdout should contain 3: {}", stdout);
    assert!(stdout.contains("7"), "stdout should contain 7: {}", stdout);
}

#[test]
fn repl_empty_line_no_error() {
    let mut child = Command::new(cargo_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start REPL");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"\n")
        .unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "stderr should be empty for blank line: {}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Step 3.10: CLI normal mode test
// ---------------------------------------------------------------------------

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

// ===========================================================================
// Builtin integration tests — comprehensive coverage
// ===========================================================================
//
// Each builtin gets: happy path, edge cases, error cases.
// Uses `eval_source` pattern for value-returning tests and `run_lingo` for
// assertion-based tests.

/// Shorthand: evaluate a single expression and return its Display string.
fn eval_lingo(src: &str) -> String {
    let mut interp = Interpreter::new();
    let result = interp.eval_source(src);
    assert!(result.is_ok(), "eval failed for '{}': {:?}", src, result);
    format!("{}", result.unwrap())
}

/// Shorthand: evaluate and expect an error.
fn eval_lingo_err(src: &str) -> String {
    let mut interp = Interpreter::new();
    let result = interp.eval_source(src);
    assert!(result.is_err(), "expected error for '{}', got {:?}", src, result);
    result.unwrap_err()
}

// ---------------------------------------------------------------------------
// Arithmetic: +
// ---------------------------------------------------------------------------

#[test]
fn add_two_integers() {
    assert_eq!(eval_lingo("(+ 1 2)"), "3");
}

#[test]
fn add_zero_args_returns_identity() {
    assert_eq!(eval_lingo("(+)"), "0");
}

#[test]
fn add_one_arg_returns_self() {
    assert_eq!(eval_lingo("(+ 5)"), "5");
}

#[test]
fn add_three_args() {
    assert_eq!(eval_lingo("(+ 1 2 3)"), "6");
}

#[test]
fn add_int_and_float_coerces() {
    assert_eq!(eval_lingo("(+ 1 2.5)"), "3.5");
}

#[test]
fn add_float_and_int_coerces() {
    assert_eq!(eval_lingo("(+ 2.5 1)"), "3.5");
}

#[test]
fn add_two_floats() {
    assert_eq!(eval_lingo("(+ 1.5 2.5)"), "4.0");
}

#[test]
fn add_negative_numbers() {
    assert_eq!(eval_lingo("(+ -3 5)"), "2");
}

#[test]
fn add_non_numeric_errors() {
    let err = eval_lingo_err(r#"(+ 1 "a")"#);
    assert!(err.contains("expected numbers"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Arithmetic: -
// ---------------------------------------------------------------------------

#[test]
fn sub_two_integers() {
    assert_eq!(eval_lingo("(- 5 3)"), "2");
}

#[test]
fn sub_unary_negation() {
    assert_eq!(eval_lingo("(- 5)"), "-5");
}

#[test]
fn sub_unary_float_negation() {
    assert_eq!(eval_lingo("(- 3.5)"), "-3.5");
}

#[test]
fn sub_three_args() {
    assert_eq!(eval_lingo("(- 10 3 2)"), "5");
}

#[test]
fn sub_zero_args_errors() {
    let err = eval_lingo_err("(-)");
    assert!(err.contains("at least 1 argument"), "err: {}", err);
}

#[test]
fn sub_int_float_coerces() {
    assert_eq!(eval_lingo("(- 5 1.5)"), "3.5");
}

#[test]
fn sub_non_numeric_errors() {
    let err = eval_lingo_err(r#"(- "a")"#);
    assert!(err.contains("Cannot negate"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Arithmetic: *
// ---------------------------------------------------------------------------

#[test]
fn mul_two_integers() {
    assert_eq!(eval_lingo("(* 3 4)"), "12");
}

#[test]
fn mul_zero_args_returns_identity() {
    assert_eq!(eval_lingo("(*)"), "1");
}

#[test]
fn mul_one_arg() {
    assert_eq!(eval_lingo("(* 7)"), "7");
}

#[test]
fn mul_three_args() {
    assert_eq!(eval_lingo("(* 2 3 4)"), "24");
}

#[test]
fn mul_by_zero() {
    assert_eq!(eval_lingo("(* 5 0)"), "0");
}

#[test]
fn mul_int_float_coerces() {
    assert_eq!(eval_lingo("(* 2 3.5)"), "7.0");
}

#[test]
fn mul_non_numeric_errors() {
    let err = eval_lingo_err(r#"(* 2 "x")"#);
    assert!(err.contains("expected numbers"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Arithmetic: /
// ---------------------------------------------------------------------------

#[test]
fn div_two_integers() {
    assert_eq!(eval_lingo("(/ 10 2)"), "5");
}

#[test]
fn div_integer_truncation() {
    assert_eq!(eval_lingo("(/ 7 2)"), "3");
}

#[test]
fn div_float_division() {
    assert_eq!(eval_lingo("(/ 7.0 2.0)"), "3.5");
}

#[test]
fn div_int_float_coerces() {
    assert_eq!(eval_lingo("(/ 7 2.0)"), "3.5");
}

#[test]
fn div_by_zero_errors() {
    let err = eval_lingo_err("(/ 5 0)");
    assert!(err.contains("Division by zero"), "err: {}", err);
}

#[test]
fn div_float_by_zero_errors() {
    let err = eval_lingo_err("(/ 5.0 0.0)");
    assert!(err.contains("Division by zero"), "err: {}", err);
}

#[test]
fn div_wrong_arg_count_errors() {
    let err = eval_lingo_err("(/ 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Arithmetic: mod
// ---------------------------------------------------------------------------

#[test]
fn mod_basic() {
    assert_eq!(eval_lingo("(mod 7 3)"), "1");
}

#[test]
fn mod_even_division() {
    assert_eq!(eval_lingo("(mod 6 3)"), "0");
}

#[test]
fn mod_by_zero_errors() {
    let err = eval_lingo_err("(mod 5 0)");
    assert!(err.contains("Modulo by zero"), "err: {}", err);
}

#[test]
fn mod_wrong_arg_count_errors() {
    let err = eval_lingo_err("(mod 5)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn mod_float_values() {
    // mod with floats should work via numeric_op
    assert_eq!(eval_lingo("(mod 7.5 2.5)"), "0.0");
}

// ---------------------------------------------------------------------------
// Arithmetic: abs
// ---------------------------------------------------------------------------

#[test]
fn abs_positive() {
    assert_eq!(eval_lingo("(abs 5)"), "5");
}

#[test]
fn abs_negative() {
    assert_eq!(eval_lingo("(abs -5)"), "5");
}

#[test]
fn abs_zero() {
    assert_eq!(eval_lingo("(abs 0)"), "0");
}

#[test]
fn abs_float_negative() {
    assert_eq!(eval_lingo("(abs -3.5)"), "3.5");
}

#[test]
fn abs_wrong_arg_count_errors() {
    let err = eval_lingo_err("(abs 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

#[test]
fn abs_non_numeric_errors() {
    let err = eval_lingo_err(r#"(abs "x")"#);
    assert!(err.contains("expected number"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Arithmetic: min
// ---------------------------------------------------------------------------

#[test]
fn min_returns_smaller() {
    assert_eq!(eval_lingo("(min 3 5)"), "3");
}

#[test]
fn min_equal_values() {
    assert_eq!(eval_lingo("(min 4 4)"), "4");
}

#[test]
fn min_negative() {
    assert_eq!(eval_lingo("(min -1 1)"), "-1");
}

#[test]
fn min_float() {
    assert_eq!(eval_lingo("(min 1.5 2.5)"), "1.5");
}

#[test]
fn min_int_float_mixed() {
    assert_eq!(eval_lingo("(min 1 2.5)"), "1");
}

#[test]
fn min_wrong_arg_count_errors() {
    let err = eval_lingo_err("(min 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn min_strings() {
    assert_eq!(eval_lingo(r#"(min "a" "b")"#), "a");
}

// ---------------------------------------------------------------------------
// Arithmetic: max
// ---------------------------------------------------------------------------

#[test]
fn max_returns_larger() {
    assert_eq!(eval_lingo("(max 3 5)"), "5");
}

#[test]
fn max_equal_values() {
    assert_eq!(eval_lingo("(max 4 4)"), "4");
}

#[test]
fn max_negative() {
    assert_eq!(eval_lingo("(max -1 1)"), "1");
}

#[test]
fn max_float() {
    assert_eq!(eval_lingo("(max 1.5 2.5)"), "2.5");
}

#[test]
fn max_wrong_arg_count_errors() {
    let err = eval_lingo_err("(max 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn max_strings() {
    assert_eq!(eval_lingo(r#"(max "a" "b")"#), "b");
}

// ---------------------------------------------------------------------------
// Comparison: =
// ---------------------------------------------------------------------------

#[test]
fn eq_equal_ints() {
    assert_eq!(eval_lingo("(= 1 1)"), "true");
}

#[test]
fn eq_unequal_ints() {
    assert_eq!(eval_lingo("(= 1 2)"), "false");
}

#[test]
fn eq_equal_strings() {
    assert_eq!(eval_lingo(r#"(= "a" "a")"#), "true");
}

#[test]
fn eq_unequal_strings() {
    assert_eq!(eval_lingo(r#"(= "a" "b")"#), "false");
}

#[test]
fn eq_equal_bools() {
    assert_eq!(eval_lingo("(= true true)"), "true");
}

#[test]
fn eq_cross_type() {
    assert_eq!(eval_lingo(r#"(= 1 "1")"#), "false");
}

#[test]
fn eq_nil_nil() {
    assert_eq!(eval_lingo("(= nil nil)"), "true");
}

#[test]
fn eq_lists() {
    assert_eq!(eval_lingo("(= (list 1 2) (list 1 2))"), "true");
}

#[test]
fn eq_wrong_arg_count_errors() {
    let err = eval_lingo_err("(= 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Comparison: <
// ---------------------------------------------------------------------------

#[test]
fn lt_true_case() {
    assert_eq!(eval_lingo("(< 1 2)"), "true");
}

#[test]
fn lt_false_case() {
    assert_eq!(eval_lingo("(< 2 1)"), "false");
}

#[test]
fn lt_equal_false() {
    assert_eq!(eval_lingo("(< 1 1)"), "false");
}

#[test]
fn lt_float() {
    assert_eq!(eval_lingo("(< 1.5 2.5)"), "true");
}

#[test]
fn lt_int_float_mixed() {
    assert_eq!(eval_lingo("(< 1 2.5)"), "true");
}

#[test]
fn lt_strings() {
    assert_eq!(eval_lingo(r#"(< "a" "b")"#), "true");
}

#[test]
fn lt_wrong_arg_count_errors() {
    let err = eval_lingo_err("(< 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Comparison: >
// ---------------------------------------------------------------------------

#[test]
fn gt_true_case() {
    assert_eq!(eval_lingo("(> 2 1)"), "true");
}

#[test]
fn gt_false_case() {
    assert_eq!(eval_lingo("(> 1 2)"), "false");
}

#[test]
fn gt_equal_false() {
    assert_eq!(eval_lingo("(> 1 1)"), "false");
}

#[test]
fn gt_wrong_arg_count_errors() {
    let err = eval_lingo_err("(> 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Comparison: <=
// ---------------------------------------------------------------------------

#[test]
fn le_less_true() {
    assert_eq!(eval_lingo("(<= 1 2)"), "true");
}

#[test]
fn le_equal_true() {
    assert_eq!(eval_lingo("(<= 1 1)"), "true");
}

#[test]
fn le_greater_false() {
    assert_eq!(eval_lingo("(<= 2 1)"), "false");
}

#[test]
fn le_wrong_arg_count_errors() {
    let err = eval_lingo_err("(<= 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Comparison: >=
// ---------------------------------------------------------------------------

#[test]
fn ge_greater_true() {
    assert_eq!(eval_lingo("(>= 2 1)"), "true");
}

#[test]
fn ge_equal_true() {
    assert_eq!(eval_lingo("(>= 1 1)"), "true");
}

#[test]
fn ge_less_false() {
    assert_eq!(eval_lingo("(>= 1 2)"), "false");
}

#[test]
fn ge_wrong_arg_count_errors() {
    let err = eval_lingo_err("(>= 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Logic: not
// ---------------------------------------------------------------------------

#[test]
fn not_true_returns_false() {
    assert_eq!(eval_lingo("(not true)"), "false");
}

#[test]
fn not_false_returns_true() {
    assert_eq!(eval_lingo("(not false)"), "true");
}

#[test]
fn not_zero_returns_true() {
    assert_eq!(eval_lingo("(not 0)"), "true");
}

#[test]
fn not_nonzero_returns_false() {
    assert_eq!(eval_lingo("(not 1)"), "false");
}

#[test]
fn not_nil_returns_true() {
    assert_eq!(eval_lingo("(not nil)"), "true");
}

#[test]
fn not_string_returns_false() {
    assert_eq!(eval_lingo(r#"(not "hello")"#), "false");
}

#[test]
fn not_wrong_arg_count_errors() {
    let err = eval_lingo_err("(not true false)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: list
// ---------------------------------------------------------------------------

#[test]
fn list_empty() {
    assert_eq!(eval_lingo("(list)"), "()");
}

#[test]
fn list_single() {
    assert_eq!(eval_lingo("(list 1)"), "(1)");
}

#[test]
fn list_multiple() {
    assert_eq!(eval_lingo("(list 1 2 3)"), "(1 2 3)");
}

#[test]
fn list_mixed_types() {
    assert_eq!(eval_lingo(r#"(list 1 "a" true nil)"#), "(1 a true nil)");
}

#[test]
fn list_nested() {
    assert_eq!(eval_lingo("(list 1 (list 2 3))"), "(1 (2 3))");
}

// ---------------------------------------------------------------------------
// List: cons
// ---------------------------------------------------------------------------

#[test]
fn cons_prepend() {
    assert_eq!(eval_lingo("(cons 1 (list 2 3))"), "(1 2 3)");
}

#[test]
fn cons_to_empty_list() {
    assert_eq!(eval_lingo("(cons 1 (list))"), "(1)");
}

#[test]
fn cons_non_list_second_arg_errors() {
    let err = eval_lingo_err("(cons 1 2)");
    assert!(err.contains("must be a list"), "err: {}", err);
}

#[test]
fn cons_wrong_arg_count_errors() {
    let err = eval_lingo_err("(cons 1)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: first
// ---------------------------------------------------------------------------

#[test]
fn first_normal() {
    assert_eq!(eval_lingo("(first (list 1 2 3))"), "1");
}

#[test]
fn first_empty_list_returns_nil() {
    assert_eq!(eval_lingo("(first (list))"), "nil");
}

#[test]
fn first_single_element() {
    assert_eq!(eval_lingo("(first (list 42))"), "42");
}

#[test]
fn first_non_list_errors() {
    let err = eval_lingo_err("(first 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn first_wrong_arg_count_errors() {
    let err = eval_lingo_err("(first (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: rest
// ---------------------------------------------------------------------------

#[test]
fn rest_normal() {
    assert_eq!(eval_lingo("(rest (list 1 2 3))"), "(2 3)");
}

#[test]
fn rest_empty_list_returns_empty() {
    assert_eq!(eval_lingo("(rest (list))"), "()");
}

#[test]
fn rest_single_element_returns_empty() {
    assert_eq!(eval_lingo("(rest (list 1))"), "()");
}

#[test]
fn rest_non_list_errors() {
    let err = eval_lingo_err("(rest 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn rest_wrong_arg_count_errors() {
    let err = eval_lingo_err("(rest)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: nth
// ---------------------------------------------------------------------------

#[test]
fn nth_first_element() {
    assert_eq!(eval_lingo("(nth (list 10 20 30) 0)"), "10");
}

#[test]
fn nth_last_element() {
    assert_eq!(eval_lingo("(nth (list 10 20 30) 2)"), "30");
}

#[test]
fn nth_out_of_bounds_errors() {
    let err = eval_lingo_err("(nth (list 1 2) 5)");
    assert!(err.contains("out of bounds"), "err: {}", err);
}

#[test]
fn nth_wrong_arg_count_errors() {
    let err = eval_lingo_err("(nth (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn nth_wrong_types_errors() {
    let err = eval_lingo_err(r#"(nth "abc" 0)"#);
    assert!(err.contains("expected (list, int)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: len
// ---------------------------------------------------------------------------

#[test]
fn len_list() {
    assert_eq!(eval_lingo("(len (list 1 2 3))"), "3");
}

#[test]
fn len_empty_list() {
    assert_eq!(eval_lingo("(len (list))"), "0");
}

#[test]
fn len_string() {
    assert_eq!(eval_lingo(r#"(len "hello")"#), "5");
}

#[test]
fn len_empty_string() {
    assert_eq!(eval_lingo(r#"(len "")"#), "0");
}

#[test]
fn len_wrong_type_errors() {
    let err = eval_lingo_err("(len 42)");
    assert!(err.contains("expected list or string"), "err: {}", err);
}

#[test]
fn len_wrong_arg_count_errors() {
    let err = eval_lingo_err("(len (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: cat
// ---------------------------------------------------------------------------

#[test]
fn cat_two_lists() {
    assert_eq!(eval_lingo("(cat (list 1 2) (list 3 4))"), "(1 2 3 4)");
}

#[test]
fn cat_empty_lists() {
    assert_eq!(eval_lingo("(cat (list) (list))"), "()");
}

#[test]
fn cat_one_empty_one_not() {
    assert_eq!(eval_lingo("(cat (list) (list 1))"), "(1)");
}

#[test]
fn cat_three_lists() {
    assert_eq!(eval_lingo("(cat (list 1) (list 2) (list 3))"), "(1 2 3)");
}

#[test]
fn cat_zero_args() {
    assert_eq!(eval_lingo("(cat)"), "()");
}

#[test]
fn cat_non_list_errors() {
    let err = eval_lingo_err("(cat (list 1) 2)");
    assert!(err.contains("expected list"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: rev
// ---------------------------------------------------------------------------

#[test]
fn rev_normal() {
    assert_eq!(eval_lingo("(rev (list 1 2 3))"), "(3 2 1)");
}

#[test]
fn rev_empty() {
    assert_eq!(eval_lingo("(rev (list))"), "()");
}

#[test]
fn rev_single() {
    assert_eq!(eval_lingo("(rev (list 1))"), "(1)");
}

#[test]
fn rev_non_list_errors() {
    let err = eval_lingo_err("(rev 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn rev_wrong_arg_count_errors() {
    let err = eval_lingo_err("(rev (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: map
// ---------------------------------------------------------------------------

#[test]
fn map_with_lambda() {
    assert_eq!(eval_lingo("(map (list 1 2 3) (fn (x) (+ x 1)))"), "(2 3 4)");
}

#[test]
fn map_with_builtin_function() {
    assert_eq!(eval_lingo("(map (list -1 2 -3) abs)"), "(1 2 3)");
}

#[test]
fn map_fn_first_order() {
    // map also accepts (fn, list) order
    assert_eq!(eval_lingo("(map (fn (x) (* x 2)) (list 1 2 3))"), "(2 4 6)");
}

#[test]
fn map_empty_list() {
    assert_eq!(eval_lingo("(map (list) (fn (x) x))"), "()");
}

#[test]
fn map_wrong_arg_count_errors() {
    let err = eval_lingo_err("(map (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn map_non_function_errors() {
    let err = eval_lingo_err("(map (list 1) 42)");
    assert!(err.contains("expected a list and a function"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: filter
// ---------------------------------------------------------------------------

#[test]
fn filter_with_lambda() {
    assert_eq!(eval_lingo("(filter (list 1 2 3 4) (fn (x) (> x 2)))"), "(3 4)");
}

#[test]
fn filter_all_pass() {
    assert_eq!(eval_lingo("(filter (list 1 2 3) (fn (x) true))"), "(1 2 3)");
}

#[test]
fn filter_none_pass() {
    assert_eq!(eval_lingo("(filter (list 1 2 3) (fn (x) false))"), "()");
}

#[test]
fn filter_empty_list() {
    assert_eq!(eval_lingo("(filter (list) (fn (x) true))"), "()");
}

#[test]
fn filter_with_builtin_predicate() {
    assert_eq!(eval_lingo(r#"(filter (list 1 "a" 2 "b") int?)"#), "(1 2)");
}

#[test]
fn filter_wrong_arg_count_errors() {
    let err = eval_lingo_err("(filter (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: fold
// ---------------------------------------------------------------------------

#[test]
fn fold_sum() {
    assert_eq!(eval_lingo("(fold (list 1 2 3) 0 (fn (acc x) (+ acc x)))"), "6");
}

#[test]
fn fold_with_builtin() {
    assert_eq!(eval_lingo("(fold (list 1 2 3) 0 +)"), "6");
}

#[test]
fn fold_empty_list_returns_initial() {
    assert_eq!(eval_lingo("(fold (list) 42 +)"), "42");
}

#[test]
fn fold_string_concat() {
    assert_eq!(eval_lingo(r#"(fold (list "a" "b" "c") "" str)"#), "abc");
}

#[test]
fn fold_wrong_arg_count_errors() {
    let err = eval_lingo_err("(fold (list 1) 0)");
    assert!(err.contains("exactly 3 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: each
// ---------------------------------------------------------------------------

#[test]
fn each_returns_nil() {
    assert_eq!(eval_lingo("(each (fn (x) x) (list 1 2 3))"), "nil");
}

#[test]
fn each_empty_list() {
    assert_eq!(eval_lingo("(each (fn (x) x) (list))"), "nil");
}

#[test]
fn each_wrong_arg_count_errors() {
    let err = eval_lingo_err("(each (fn (x) x))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn each_with_builtin() {
    // println returns nil; each should work with builtins
    assert_eq!(eval_lingo("(each println (list 1 2))"), "nil");
}

// ---------------------------------------------------------------------------
// List: flat
// ---------------------------------------------------------------------------

#[test]
fn flat_nested_lists() {
    assert_eq!(eval_lingo("(flat (list (list 1 2) (list 3 4)))"), "(1 2 3 4)");
}

#[test]
fn flat_mixed() {
    assert_eq!(eval_lingo("(flat (list 1 (list 2 3) 4))"), "(1 2 3 4)");
}

#[test]
fn flat_already_flat() {
    assert_eq!(eval_lingo("(flat (list 1 2 3))"), "(1 2 3)");
}

#[test]
fn flat_empty_list() {
    assert_eq!(eval_lingo("(flat (list))"), "()");
}

#[test]
fn flat_non_list_errors() {
    let err = eval_lingo_err("(flat 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn flat_wrong_arg_count_errors() {
    let err = eval_lingo_err("(flat (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: zip
// ---------------------------------------------------------------------------

#[test]
fn zip_equal_lengths() {
    assert_eq!(eval_lingo("(zip (list 1 2 3) (list 4 5 6))"), "((1 4) (2 5) (3 6))");
}

#[test]
fn zip_unequal_lengths_truncates() {
    assert_eq!(eval_lingo("(zip (list 1 2) (list 3 4 5))"), "((1 3) (2 4))");
}

#[test]
fn zip_empty_lists() {
    assert_eq!(eval_lingo("(zip (list) (list))"), "()");
}

#[test]
fn zip_one_empty() {
    assert_eq!(eval_lingo("(zip (list 1) (list))"), "()");
}

#[test]
fn zip_wrong_arg_count_errors() {
    let err = eval_lingo_err("(zip (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn zip_non_list_errors() {
    let err = eval_lingo_err("(zip 1 2)");
    assert!(err.contains("expected two lists"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: take
// ---------------------------------------------------------------------------

#[test]
fn take_normal() {
    assert_eq!(eval_lingo("(take 2 (list 1 2 3))"), "(1 2)");
}

#[test]
fn take_zero() {
    assert_eq!(eval_lingo("(take 0 (list 1 2 3))"), "()");
}

#[test]
fn take_more_than_length() {
    assert_eq!(eval_lingo("(take 10 (list 1 2))"), "(1 2)");
}

#[test]
fn take_from_empty() {
    assert_eq!(eval_lingo("(take 2 (list))"), "()");
}

#[test]
fn take_negative_treated_as_zero() {
    assert_eq!(eval_lingo("(take -1 (list 1 2 3))"), "()");
}

#[test]
fn take_wrong_arg_count_errors() {
    let err = eval_lingo_err("(take 2)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn take_wrong_types_errors() {
    let err = eval_lingo_err(r#"(take "a" (list 1))"#);
    assert!(err.contains("expected (int, list)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: drop
// ---------------------------------------------------------------------------

#[test]
fn drop_normal() {
    assert_eq!(eval_lingo("(drop 2 (list 1 2 3))"), "(3)");
}

#[test]
fn drop_zero() {
    assert_eq!(eval_lingo("(drop 0 (list 1 2 3))"), "(1 2 3)");
}

#[test]
fn drop_more_than_length() {
    assert_eq!(eval_lingo("(drop 10 (list 1 2))"), "()");
}

#[test]
fn drop_from_empty() {
    assert_eq!(eval_lingo("(drop 2 (list))"), "()");
}

#[test]
fn drop_negative_treated_as_zero() {
    assert_eq!(eval_lingo("(drop -1 (list 1 2 3))"), "(1 2 3)");
}

#[test]
fn drop_wrong_arg_count_errors() {
    let err = eval_lingo_err("(drop 2)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: sort
// ---------------------------------------------------------------------------

#[test]
fn sort_integers() {
    assert_eq!(eval_lingo("(sort (list 3 1 2))"), "(1 2 3)");
}

#[test]
fn sort_already_sorted() {
    assert_eq!(eval_lingo("(sort (list 1 2 3))"), "(1 2 3)");
}

#[test]
fn sort_reverse_order() {
    assert_eq!(eval_lingo("(sort (list 3 2 1))"), "(1 2 3)");
}

#[test]
fn sort_strings() {
    assert_eq!(eval_lingo(r#"(sort (list "c" "a" "b"))"#), "(a b c)");
}

#[test]
fn sort_empty_list() {
    assert_eq!(eval_lingo("(sort (list))"), "()");
}

#[test]
fn sort_single_element() {
    assert_eq!(eval_lingo("(sort (list 1))"), "(1)");
}

#[test]
fn sort_non_list_errors() {
    let err = eval_lingo_err("(sort 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn sort_wrong_arg_count_errors() {
    let err = eval_lingo_err("(sort (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: sortby
// ---------------------------------------------------------------------------

#[test]
fn sortby_with_lambda() {
    assert_eq!(
        eval_lingo("(sortby (list 3 -1 2) (fn (x) (abs x)))"),
        "(-1 2 3)"
    );
}

#[test]
fn sortby_with_builtin() {
    assert_eq!(eval_lingo("(sortby (list 3 -1 2) abs)"), "(-1 2 3)");
}

#[test]
fn sortby_empty_list() {
    assert_eq!(eval_lingo("(sortby (list) abs)"), "()");
}

#[test]
fn sortby_wrong_arg_count_errors() {
    let err = eval_lingo_err("(sortby (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: any
// ---------------------------------------------------------------------------

#[test]
fn any_true_case() {
    assert_eq!(eval_lingo("(any (list 1 2 3) (fn (x) (> x 2)))"), "true");
}

#[test]
fn any_false_case() {
    assert_eq!(eval_lingo("(any (list 1 2 3) (fn (x) (> x 5)))"), "false");
}

#[test]
fn any_empty_list() {
    assert_eq!(eval_lingo("(any (list) (fn (x) true))"), "false");
}

#[test]
fn any_with_builtin() {
    assert_eq!(eval_lingo(r#"(any (list 1 "a" 2) string?)"#), "true");
}

#[test]
fn any_wrong_arg_count_errors() {
    let err = eval_lingo_err("(any (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: all
// ---------------------------------------------------------------------------

#[test]
fn all_true_case() {
    assert_eq!(eval_lingo("(all (list 1 2 3) (fn (x) (> x 0)))"), "true");
}

#[test]
fn all_false_case() {
    assert_eq!(eval_lingo("(all (list 1 2 3) (fn (x) (> x 1)))"), "false");
}

#[test]
fn all_empty_list() {
    assert_eq!(eval_lingo("(all (list) (fn (x) false))"), "true");
}

#[test]
fn all_with_builtin() {
    assert_eq!(eval_lingo("(all (list 1 2 3) int?)"), "true");
}

#[test]
fn all_wrong_arg_count_errors() {
    let err = eval_lingo_err("(all (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: find
// ---------------------------------------------------------------------------

#[test]
fn find_found() {
    assert_eq!(eval_lingo("(find (list 1 2 3) (fn (x) (> x 1)))"), "2");
}

#[test]
fn find_not_found() {
    assert_eq!(eval_lingo("(find (list 1 2 3) (fn (x) (> x 5)))"), "nil");
}

#[test]
fn find_empty_list() {
    assert_eq!(eval_lingo("(find (list) (fn (x) true))"), "nil");
}

#[test]
fn find_with_builtin() {
    assert_eq!(eval_lingo(r#"(find (list 1 "a" 2) string?)"#), "a");
}

#[test]
fn find_wrong_arg_count_errors() {
    let err = eval_lingo_err("(find (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: uniq
// ---------------------------------------------------------------------------

#[test]
fn uniq_removes_duplicates() {
    assert_eq!(eval_lingo("(uniq (list 1 2 2 3 3 3))"), "(1 2 3)");
}

#[test]
fn uniq_no_duplicates() {
    assert_eq!(eval_lingo("(uniq (list 1 2 3))"), "(1 2 3)");
}

#[test]
fn uniq_empty_list() {
    assert_eq!(eval_lingo("(uniq (list))"), "()");
}

#[test]
fn uniq_preserves_order() {
    assert_eq!(eval_lingo("(uniq (list 3 1 2 1 3))"), "(3 1 2)");
}

#[test]
fn uniq_non_list_errors() {
    let err = eval_lingo_err("(uniq 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn uniq_wrong_arg_count_errors() {
    let err = eval_lingo_err("(uniq (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: chunk
// ---------------------------------------------------------------------------

#[test]
fn chunk_even_split() {
    assert_eq!(eval_lingo("(chunk 2 (list 1 2 3 4))"), "((1 2) (3 4))");
}

#[test]
fn chunk_uneven_split() {
    assert_eq!(eval_lingo("(chunk 2 (list 1 2 3))"), "((1 2) (3))");
}

#[test]
fn chunk_size_one() {
    assert_eq!(eval_lingo("(chunk 1 (list 1 2 3))"), "((1) (2) (3))");
}

#[test]
fn chunk_size_larger_than_list() {
    assert_eq!(eval_lingo("(chunk 10 (list 1 2))"), "((1 2))");
}

#[test]
fn chunk_empty_list() {
    assert_eq!(eval_lingo("(chunk 2 (list))"), "()");
}

#[test]
fn chunk_zero_size_errors() {
    let err = eval_lingo_err("(chunk 0 (list 1 2))");
    assert!(err.contains("must be positive"), "err: {}", err);
}

#[test]
fn chunk_negative_size_errors() {
    let err = eval_lingo_err("(chunk -1 (list 1 2))");
    assert!(err.contains("must be positive"), "err: {}", err);
}

#[test]
fn chunk_wrong_arg_count_errors() {
    let err = eval_lingo_err("(chunk 2)");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: enumerate
// ---------------------------------------------------------------------------

#[test]
fn enumerate_normal() {
    assert_eq!(eval_lingo("(enumerate (list 10 20 30))"), "((0 10) (1 20) (2 30))");
}

#[test]
fn enumerate_empty() {
    assert_eq!(eval_lingo("(enumerate (list))"), "()");
}

#[test]
fn enumerate_single() {
    assert_eq!(eval_lingo("(enumerate (list 42))"), "((0 42))");
}

#[test]
fn enumerate_non_list_errors() {
    let err = eval_lingo_err("(enumerate 1)");
    assert!(err.contains("expected list"), "err: {}", err);
}

#[test]
fn enumerate_wrong_arg_count_errors() {
    let err = eval_lingo_err("(enumerate (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: groupby
// ---------------------------------------------------------------------------

#[test]
fn groupby_with_lambda() {
    // Group by even/odd
    assert_eq!(
        eval_lingo("(groupby (list 1 2 3 4) (fn (x) (mod x 2)))"),
        "((1 (1 3)) (0 (2 4)))"
    );
}

#[test]
fn groupby_empty_list() {
    assert_eq!(eval_lingo("(groupby (list) (fn (x) x))"), "()");
}

#[test]
fn groupby_single_group() {
    assert_eq!(
        eval_lingo("(groupby (list 2 4 6) (fn (x) (mod x 2)))"),
        "((0 (2 4 6)))"
    );
}

#[test]
fn groupby_wrong_arg_count_errors() {
    let err = eval_lingo_err("(groupby (list 1))");
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// List: range
// ---------------------------------------------------------------------------

#[test]
fn range_one_arg() {
    assert_eq!(eval_lingo("(range 5)"), "(0 1 2 3 4)");
}

#[test]
fn range_zero_arg() {
    assert_eq!(eval_lingo("(range 0)"), "()");
}

#[test]
fn range_two_args() {
    assert_eq!(eval_lingo("(range 2 5)"), "(2 3 4)");
}

#[test]
fn range_two_args_equal() {
    assert_eq!(eval_lingo("(range 3 3)"), "()");
}

#[test]
fn range_three_args_step() {
    assert_eq!(eval_lingo("(range 0 10 3)"), "(0 3 6 9)");
}

#[test]
fn range_negative_step() {
    assert_eq!(eval_lingo("(range 5 0 -1)"), "(5 4 3 2 1)");
}

#[test]
fn range_step_zero_errors() {
    let err = eval_lingo_err("(range 0 5 0)");
    assert!(err.contains("step cannot be zero"), "err: {}", err);
}

#[test]
fn range_wrong_arg_count_errors() {
    let err = eval_lingo_err("(range)");
    assert!(err.contains("1, 2, or 3 arguments"), "err: {}", err);
}

#[test]
fn range_four_args_errors() {
    let err = eval_lingo_err("(range 1 2 3 4)");
    assert!(err.contains("1, 2, or 3 arguments"), "err: {}", err);
}

#[test]
fn range_non_int_errors() {
    let err = eval_lingo_err(r#"(range "a")"#);
    assert!(err.contains("expected int"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: str
// ---------------------------------------------------------------------------

#[test]
fn str_concat_strings() {
    assert_eq!(eval_lingo(r#"(str "hello" " " "world")"#), "hello world");
}

#[test]
fn str_no_args() {
    assert_eq!(eval_lingo("(str)"), "");
}

#[test]
fn str_converts_int() {
    assert_eq!(eval_lingo("(str 42)"), "42");
}

#[test]
fn str_converts_bool() {
    assert_eq!(eval_lingo("(str true)"), "true");
}

#[test]
fn str_converts_nil() {
    assert_eq!(eval_lingo("(str nil)"), "nil");
}

#[test]
fn str_mixed_types() {
    assert_eq!(eval_lingo(r#"(str "count: " 42)"#), "count: 42");
}

// ---------------------------------------------------------------------------
// String: strlen
// ---------------------------------------------------------------------------

#[test]
fn strlen_normal() {
    assert_eq!(eval_lingo(r#"(strlen "hello")"#), "5");
}

#[test]
fn strlen_empty() {
    assert_eq!(eval_lingo(r#"(strlen "")"#), "0");
}

#[test]
fn strlen_non_string_errors() {
    let err = eval_lingo_err("(strlen 42)");
    assert!(err.contains("expected string"), "err: {}", err);
}

#[test]
fn strlen_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(strlen "a" "b")"#);
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: substring
// ---------------------------------------------------------------------------

#[test]
fn substring_normal() {
    assert_eq!(eval_lingo(r#"(substring "hello" 1 3)"#), "el");
}

#[test]
fn substring_from_start() {
    assert_eq!(eval_lingo(r#"(substring "hello" 0 5)"#), "hello");
}

#[test]
fn substring_empty_result() {
    assert_eq!(eval_lingo(r#"(substring "hello" 2 2)"#), "");
}

#[test]
fn substring_clamped_end() {
    assert_eq!(eval_lingo(r#"(substring "hi" 0 100)"#), "hi");
}

#[test]
fn substring_start_beyond_end() {
    assert_eq!(eval_lingo(r#"(substring "hi" 5 10)"#), "");
}

#[test]
fn substring_negative_start_clamped() {
    assert_eq!(eval_lingo(r#"(substring "hello" -1 3)"#), "hel");
}

#[test]
fn substring_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(substring "hi" 0)"#);
    assert!(err.contains("exactly 3 arguments"), "err: {}", err);
}

#[test]
fn substring_wrong_types_errors() {
    let err = eval_lingo_err(r#"(substring 42 0 1)"#);
    assert!(err.contains("expected (string, int, int)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: split
// ---------------------------------------------------------------------------

#[test]
fn split_normal() {
    assert_eq!(eval_lingo(r#"(split "a,b,c" ",")"#), "(a b c)");
}

#[test]
fn split_no_match() {
    assert_eq!(eval_lingo(r#"(split "abc" ",")"#), "(abc)");
}

#[test]
fn split_empty_string() {
    assert_eq!(eval_lingo(r#"(split "" ",")"#), "()");
}

#[test]
fn split_by_space() {
    assert_eq!(eval_lingo(r#"(split "a b c" " ")"#), "(a b c)");
}

#[test]
fn split_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(split "abc")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn split_wrong_types_errors() {
    let err = eval_lingo_err("(split 42 1)");
    assert!(err.contains("expected (string, string)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: join
// ---------------------------------------------------------------------------

#[test]
fn join_normal() {
    assert_eq!(eval_lingo(r#"(join ", " (list 1 2 3))"#), "1, 2, 3");
}

#[test]
fn join_empty_list() {
    assert_eq!(eval_lingo(r#"(join ", " (list))"#), "");
}

#[test]
fn join_single_element() {
    assert_eq!(eval_lingo(r#"(join ", " (list 1))"#), "1");
}

#[test]
fn join_empty_separator() {
    assert_eq!(eval_lingo(r#"(join "" (list "a" "b" "c"))"#), "abc");
}

#[test]
fn join_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(join ",")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn join_wrong_types_errors() {
    let err = eval_lingo_err("(join 1 2)");
    assert!(err.contains("expected (string, list)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: trim
// ---------------------------------------------------------------------------

#[test]
fn trim_whitespace() {
    assert_eq!(eval_lingo(r#"(trim "  hello  ")"#), "hello");
}

#[test]
fn trim_no_whitespace() {
    assert_eq!(eval_lingo(r#"(trim "hello")"#), "hello");
}

#[test]
fn trim_empty_string() {
    assert_eq!(eval_lingo(r#"(trim "")"#), "");
}

#[test]
fn trim_only_whitespace() {
    assert_eq!(eval_lingo(r#"(trim "   ")"#), "");
}

#[test]
fn trim_non_string_errors() {
    let err = eval_lingo_err("(trim 42)");
    assert!(err.contains("expected string"), "err: {}", err);
}

#[test]
fn trim_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(trim "a" "b")"#);
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: has (overloaded for string-contains and list-contains)
// ---------------------------------------------------------------------------

#[test]
fn has_string_contains_true() {
    assert_eq!(eval_lingo(r#"(has "hello world" "world")"#), "true");
}

#[test]
fn has_string_contains_false() {
    assert_eq!(eval_lingo(r#"(has "hello" "xyz")"#), "false");
}

#[test]
fn has_string_empty_needle() {
    assert_eq!(eval_lingo(r#"(has "hello" "")"#), "true");
}

#[test]
fn has_list_contains_true() {
    assert_eq!(eval_lingo("(has (list 1 2 3) 2)"), "true");
}

#[test]
fn has_list_contains_false() {
    assert_eq!(eval_lingo("(has (list 1 2 3) 4)"), "false");
}

#[test]
fn has_empty_list() {
    assert_eq!(eval_lingo("(has (list) 1)"), "false");
}

#[test]
fn has_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(has "a")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn has_wrong_types_errors() {
    let err = eval_lingo_err("(has 42 1)");
    assert!(err.contains("expected (string, string) or (list, value)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: replace
// ---------------------------------------------------------------------------

#[test]
fn replace_normal() {
    assert_eq!(eval_lingo(r#"(replace "hello world" "world" "rust")"#), "hello rust");
}

#[test]
fn replace_no_match() {
    assert_eq!(eval_lingo(r#"(replace "hello" "xyz" "abc")"#), "hello");
}

#[test]
fn replace_multiple_occurrences() {
    assert_eq!(eval_lingo(r#"(replace "aaa" "a" "b")"#), "bbb");
}

#[test]
fn replace_with_empty() {
    assert_eq!(eval_lingo(r#"(replace "hello" "l" "")"#), "heo");
}

#[test]
fn replace_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(replace "a" "b")"#);
    assert!(err.contains("exactly 3 arguments"), "err: {}", err);
}

#[test]
fn replace_wrong_types_errors() {
    let err = eval_lingo_err("(replace 1 2 3)");
    assert!(err.contains("expected (string, string, string)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: starts-with
// ---------------------------------------------------------------------------

#[test]
fn starts_with_true() {
    assert_eq!(eval_lingo(r#"(starts-with "hello" "hel")"#), "true");
}

#[test]
fn starts_with_false() {
    assert_eq!(eval_lingo(r#"(starts-with "hello" "xyz")"#), "false");
}

#[test]
fn starts_with_empty_prefix() {
    assert_eq!(eval_lingo(r#"(starts-with "hello" "")"#), "true");
}

#[test]
fn starts_with_full_string() {
    assert_eq!(eval_lingo(r#"(starts-with "hello" "hello")"#), "true");
}

#[test]
fn starts_with_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(starts-with "a")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn starts_with_wrong_types_errors() {
    let err = eval_lingo_err("(starts-with 1 2)");
    assert!(err.contains("expected (string, string)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: ends-with
// ---------------------------------------------------------------------------

#[test]
fn ends_with_true() {
    assert_eq!(eval_lingo(r#"(ends-with "hello" "llo")"#), "true");
}

#[test]
fn ends_with_false() {
    assert_eq!(eval_lingo(r#"(ends-with "hello" "xyz")"#), "false");
}

#[test]
fn ends_with_empty_suffix() {
    assert_eq!(eval_lingo(r#"(ends-with "hello" "")"#), "true");
}

#[test]
fn ends_with_full_string() {
    assert_eq!(eval_lingo(r#"(ends-with "hello" "hello")"#), "true");
}

#[test]
fn ends_with_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(ends-with "a")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn ends_with_wrong_types_errors() {
    let err = eval_lingo_err("(ends-with 1 2)");
    assert!(err.contains("expected (string, string)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: upcase
// ---------------------------------------------------------------------------

#[test]
fn upcase_normal() {
    assert_eq!(eval_lingo(r#"(upcase "hello")"#), "HELLO");
}

#[test]
fn upcase_already_upper() {
    assert_eq!(eval_lingo(r#"(upcase "HELLO")"#), "HELLO");
}

#[test]
fn upcase_empty() {
    assert_eq!(eval_lingo(r#"(upcase "")"#), "");
}

#[test]
fn upcase_non_string_errors() {
    let err = eval_lingo_err("(upcase 42)");
    assert!(err.contains("expected string"), "err: {}", err);
}

#[test]
fn upcase_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(upcase "a" "b")"#);
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// String: downcase
// ---------------------------------------------------------------------------

#[test]
fn downcase_normal() {
    assert_eq!(eval_lingo(r#"(downcase "HELLO")"#), "hello");
}

#[test]
fn downcase_already_lower() {
    assert_eq!(eval_lingo(r#"(downcase "hello")"#), "hello");
}

#[test]
fn downcase_empty() {
    assert_eq!(eval_lingo(r#"(downcase "")"#), "");
}

#[test]
fn downcase_non_string_errors() {
    let err = eval_lingo_err("(downcase 42)");
    assert!(err.contains("expected string"), "err: {}", err);
}

#[test]
fn downcase_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(downcase "a" "b")"#);
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: type-of
// ---------------------------------------------------------------------------

#[test]
fn type_of_int() {
    assert_eq!(eval_lingo("(type-of 42)"), "int");
}

#[test]
fn type_of_float() {
    assert_eq!(eval_lingo("(type-of 3.14)"), "float");
}

#[test]
fn type_of_string() {
    assert_eq!(eval_lingo(r#"(type-of "hello")"#), "string");
}

#[test]
fn type_of_bool() {
    assert_eq!(eval_lingo("(type-of true)"), "bool");
}

#[test]
fn type_of_nil() {
    assert_eq!(eval_lingo("(type-of nil)"), "nil");
}

#[test]
fn type_of_list() {
    assert_eq!(eval_lingo("(type-of (list 1 2))"), "list");
}

#[test]
fn type_of_lambda() {
    assert_eq!(eval_lingo("(type-of (fn (x) x))"), "lambda");
}

#[test]
fn type_of_builtin() {
    assert_eq!(eval_lingo("(type-of +)"), "builtin");
}

#[test]
fn type_of_wrong_arg_count_errors() {
    let err = eval_lingo_err("(type-of 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: int?
// ---------------------------------------------------------------------------

#[test]
fn is_int_true() {
    assert_eq!(eval_lingo("(int? 42)"), "true");
}

#[test]
fn is_int_false_float() {
    assert_eq!(eval_lingo("(int? 3.14)"), "false");
}

#[test]
fn is_int_false_string() {
    assert_eq!(eval_lingo(r#"(int? "42")"#), "false");
}

#[test]
fn is_int_wrong_arg_count_errors() {
    let err = eval_lingo_err("(int? 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: float?
// ---------------------------------------------------------------------------

#[test]
fn is_float_true() {
    assert_eq!(eval_lingo("(float? 3.14)"), "true");
}

#[test]
fn is_float_false_int() {
    assert_eq!(eval_lingo("(float? 42)"), "false");
}

#[test]
fn is_float_wrong_arg_count_errors() {
    let err = eval_lingo_err("(float? 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: string?
// ---------------------------------------------------------------------------

#[test]
fn is_string_true() {
    assert_eq!(eval_lingo(r#"(string? "hello")"#), "true");
}

#[test]
fn is_string_false_int() {
    assert_eq!(eval_lingo("(string? 42)"), "false");
}

#[test]
fn is_string_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(string? "a" "b")"#);
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: bool?
// ---------------------------------------------------------------------------

#[test]
fn is_bool_true() {
    assert_eq!(eval_lingo("(bool? true)"), "true");
}

#[test]
fn is_bool_false() {
    assert_eq!(eval_lingo("(bool? false)"), "true");
}

#[test]
fn is_bool_not_bool() {
    assert_eq!(eval_lingo("(bool? 1)"), "false");
}

#[test]
fn is_bool_wrong_arg_count_errors() {
    let err = eval_lingo_err("(bool? true false)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: list?
// ---------------------------------------------------------------------------

#[test]
fn is_list_true() {
    assert_eq!(eval_lingo("(list? (list 1 2))"), "true");
}

#[test]
fn is_list_empty_true() {
    assert_eq!(eval_lingo("(list? (list))"), "true");
}

#[test]
fn is_list_false_int() {
    assert_eq!(eval_lingo("(list? 42)"), "false");
}

#[test]
fn is_list_wrong_arg_count_errors() {
    let err = eval_lingo_err("(list? (list 1) (list 2))");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: nil?
// ---------------------------------------------------------------------------

#[test]
fn is_nil_true() {
    assert_eq!(eval_lingo("(nil? nil)"), "true");
}

#[test]
fn is_nil_false_int() {
    assert_eq!(eval_lingo("(nil? 0)"), "false");
}

#[test]
fn is_nil_false_false() {
    assert_eq!(eval_lingo("(nil? false)"), "false");
}

#[test]
fn is_nil_wrong_arg_count_errors() {
    let err = eval_lingo_err("(nil? nil nil)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Type: number?
// ---------------------------------------------------------------------------

#[test]
fn is_number_int_true() {
    assert_eq!(eval_lingo("(number? 42)"), "true");
}

#[test]
fn is_number_float_true() {
    assert_eq!(eval_lingo("(number? 3.14)"), "true");
}

#[test]
fn is_number_string_false() {
    assert_eq!(eval_lingo(r#"(number? "42")"#), "false");
}

#[test]
fn is_number_bool_false() {
    assert_eq!(eval_lingo("(number? true)"), "false");
}

#[test]
fn is_number_nil_false() {
    assert_eq!(eval_lingo("(number? nil)"), "false");
}

#[test]
fn is_number_wrong_arg_count_errors() {
    let err = eval_lingo_err("(number? 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Conversion: ->int
// ---------------------------------------------------------------------------

#[test]
fn to_int_from_int() {
    assert_eq!(eval_lingo("(->int 42)"), "42");
}

#[test]
fn to_int_from_float() {
    assert_eq!(eval_lingo("(->int 3.7)"), "3");
}

#[test]
fn to_int_from_string() {
    assert_eq!(eval_lingo(r#"(->int "42")"#), "42");
}

#[test]
fn to_int_from_bool_true() {
    assert_eq!(eval_lingo("(->int true)"), "1");
}

#[test]
fn to_int_from_bool_false() {
    assert_eq!(eval_lingo("(->int false)"), "0");
}

#[test]
fn to_int_invalid_string_errors() {
    let err = eval_lingo_err(r#"(->int "abc")"#);
    assert!(err.contains("cannot convert"), "err: {}", err);
}

#[test]
fn to_int_from_list_errors() {
    let err = eval_lingo_err("(->int (list 1))");
    assert!(err.contains("cannot convert"), "err: {}", err);
}

#[test]
fn to_int_wrong_arg_count_errors() {
    let err = eval_lingo_err("(->int 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Conversion: ->float
// ---------------------------------------------------------------------------

#[test]
fn to_float_from_int() {
    assert_eq!(eval_lingo("(->float 42)"), "42.0");
}

#[test]
fn to_float_from_float() {
    assert_eq!(eval_lingo("(->float 3.14)"), "3.14");
}

#[test]
fn to_float_from_string() {
    assert_eq!(eval_lingo(r#"(->float "3.14")"#), "3.14");
}

#[test]
fn to_float_invalid_string_errors() {
    let err = eval_lingo_err(r#"(->float "abc")"#);
    assert!(err.contains("cannot convert"), "err: {}", err);
}

#[test]
fn to_float_from_list_errors() {
    let err = eval_lingo_err("(->float (list 1))");
    assert!(err.contains("cannot convert"), "err: {}", err);
}

#[test]
fn to_float_wrong_arg_count_errors() {
    let err = eval_lingo_err("(->float 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Conversion: ->str
// ---------------------------------------------------------------------------

#[test]
fn to_str_from_int() {
    assert_eq!(eval_lingo("(->str 42)"), "42");
}

#[test]
fn to_str_from_float() {
    assert_eq!(eval_lingo("(->str 3.14)"), "3.14");
}

#[test]
fn to_str_from_string() {
    assert_eq!(eval_lingo(r#"(->str "hello")"#), "hello");
}

#[test]
fn to_str_from_bool() {
    assert_eq!(eval_lingo("(->str true)"), "true");
}

#[test]
fn to_str_from_nil() {
    assert_eq!(eval_lingo("(->str nil)"), "nil");
}

#[test]
fn to_str_from_list() {
    assert_eq!(eval_lingo("(->str (list 1 2))"), "(1 2)");
}

#[test]
fn to_str_wrong_arg_count_errors() {
    let err = eval_lingo_err("(->str 1 2)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// I/O: println
// ---------------------------------------------------------------------------

#[test]
fn println_returns_nil() {
    assert_eq!(eval_lingo(r#"(println "test")"#), "nil");
}

#[test]
fn println_multiple_args_returns_nil() {
    assert_eq!(eval_lingo(r#"(println "a" "b" "c")"#), "nil");
}

#[test]
fn println_no_args_returns_nil() {
    assert_eq!(eval_lingo("(println)"), "nil");
}

// ---------------------------------------------------------------------------
// I/O: print
// ---------------------------------------------------------------------------

#[test]
fn print_returns_nil() {
    assert_eq!(eval_lingo(r#"(print "test")"#), "nil");
}

#[test]
fn print_multiple_args_returns_nil() {
    assert_eq!(eval_lingo(r#"(print "a" "b")"#), "nil");
}

#[test]
fn print_no_args_returns_nil() {
    assert_eq!(eval_lingo("(print)"), "nil");
}

// ---------------------------------------------------------------------------
// I/O: readline
// ---------------------------------------------------------------------------

#[test]
fn readline_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(readline "prompt")"#);
    assert!(err.contains("no arguments"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// I/O: readfile
// ---------------------------------------------------------------------------

#[test]
fn readfile_wrong_arg_count_errors() {
    let err = eval_lingo_err("(readfile)");
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

#[test]
fn readfile_wrong_type_errors() {
    let err = eval_lingo_err("(readfile 42)");
    assert!(err.contains("expected string"), "err: {}", err);
}

#[test]
fn readfile_nonexistent_file_errors() {
    let err = eval_lingo_err(r#"(readfile "/tmp/nonexistent_lingo_test_file_xyz123.txt")"#);
    assert!(err.contains("readfile"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// I/O: writefile
// ---------------------------------------------------------------------------

#[test]
fn writefile_wrong_arg_count_errors() {
    let err = eval_lingo_err(r#"(writefile "path")"#);
    assert!(err.contains("exactly 2 arguments"), "err: {}", err);
}

#[test]
fn writefile_wrong_types_errors() {
    let err = eval_lingo_err("(writefile 42 43)");
    assert!(err.contains("expected (string, string)"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Debug: dbg
// ---------------------------------------------------------------------------

#[test]
fn dbg_single_arg_returns_value() {
    assert_eq!(eval_lingo("(dbg 42)"), "42");
}

#[test]
fn dbg_multiple_args_returns_nil() {
    assert_eq!(eval_lingo("(dbg 1 2 3)"), "nil");
}

#[test]
fn dbg_string_value() {
    assert_eq!(eval_lingo(r#"(dbg "hello")"#), "hello");
}

#[test]
fn dbg_no_args_returns_nil() {
    assert_eq!(eval_lingo("(dbg)"), "nil");
}

// ---------------------------------------------------------------------------
// Debug/Test: assert
// ---------------------------------------------------------------------------

#[test]
fn assert_truthy_passes() {
    let result = run_lingo("(defn main () (assert true))");
    assert!(result.is_ok());
}

#[test]
fn assert_falsy_fails() {
    let result = run_lingo("(defn main () (assert false))");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("assertion failed"), "err: {}", err);
}

#[test]
fn assert_nonzero_truthy() {
    let result = run_lingo("(defn main () (assert 1))");
    assert!(result.is_ok());
}

#[test]
fn assert_zero_falsy() {
    let result = run_lingo("(defn main () (assert 0))");
    assert!(result.is_err());
}

#[test]
fn assert_nil_falsy() {
    let result = run_lingo("(defn main () (assert nil))");
    assert!(result.is_err());
}

#[test]
fn assert_wrong_arg_count_errors() {
    let result = run_lingo("(defn main () (assert true false))");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("exactly 1 argument"), "err: {}", err);
}

// ---------------------------------------------------------------------------
// Higher-order function builtins with builtin function arguments
// ---------------------------------------------------------------------------

#[test]
fn map_with_not_builtin() {
    assert_eq!(eval_lingo("(map (list true false true) not)"), "(false true false)");
}

#[test]
fn filter_with_number_predicate() {
    assert_eq!(eval_lingo(r#"(filter (list 1 "a" 3.14 true) number?)"#), "(1 3.14)");
}

#[test]
fn any_with_nil_predicate() {
    assert_eq!(eval_lingo("(any (list 1 nil 2) nil?)"), "true");
}

#[test]
fn all_with_number_predicate() {
    assert_eq!(eval_lingo("(all (list 1 2 3) number?)"), "true");
}

#[test]
fn find_with_float_predicate() {
    assert_eq!(eval_lingo("(find (list 1 2.5 3) float?)"), "2.5");
}

#[test]
fn sortby_fn_first_order() {
    // sortby also accepts (fn, list) order
    assert_eq!(eval_lingo("(sortby abs (list 3 -1 2))"), "(-1 2 3)");
}

#[test]
fn groupby_with_builtin() {
    assert_eq!(
        eval_lingo("(groupby (list 1 2 3 4 5 6) (fn (x) (mod x 3)))"),
        "((1 (1 4)) (2 (2 5)) (0 (3 6)))"
    );
}

#[test]
fn fold_product_with_builtin() {
    assert_eq!(eval_lingo("(fold (list 1 2 3 4) 1 *)"), "24");
}

#[test]
fn each_with_lambda_returns_nil() {
    // Verify each works with list-first order too
    assert_eq!(eval_lingo("(each (list 1 2 3) (fn (x) (+ x 1)))"), "nil");
}

// ---------------------------------------------------------------------------
// Type coercion edge cases for arithmetic
// ---------------------------------------------------------------------------

#[test]
fn add_float_float() {
    assert_eq!(eval_lingo("(+ 1.5 2.5)"), "4.0");
}

#[test]
fn sub_float_int() {
    assert_eq!(eval_lingo("(- 5.5 2)"), "3.5");
}

#[test]
fn mul_float_float() {
    assert_eq!(eval_lingo("(* 2.0 3.0)"), "6.0");
}

#[test]
fn div_float_int() {
    assert_eq!(eval_lingo("(/ 7.0 2)"), "3.5");
}

#[test]
fn mod_int_float_coerces() {
    assert_eq!(eval_lingo("(mod 7 2.0)"), "1.0");
}

// ---------------------------------------------------------------------------
// Additional edge cases and integration
// ---------------------------------------------------------------------------

#[test]
fn nested_list_operations() {
    assert_eq!(
        eval_lingo("(first (rest (list 1 2 3)))"),
        "2"
    );
}

#[test]
fn map_filter_chain() {
    assert_eq!(
        eval_lingo("(map (filter (list 1 2 3 4 5) (fn (x) (> x 2))) (fn (x) (* x 10)))"),
        "(30 40 50)"
    );
}

#[test]
fn fold_with_lambda_accumulator() {
    assert_eq!(
        eval_lingo("(fold (list 1 2 3 4 5) 0 (fn (acc x) (if (> x 3) (+ acc x) acc)))"),
        "9"
    );
}

#[test]
fn range_map_filter() {
    assert_eq!(
        eval_lingo("(filter (range 10) (fn (x) (= (mod x 2) 0)))"),
        "(0 2 4 6 8)"
    );
}

#[test]
fn str_concat_with_conversions() {
    assert_eq!(
        eval_lingo(r#"(str "value=" (->str 42))"#),
        "value=42"
    );
}

#[test]
fn zip_enumerate_equivalence() {
    // Enumerate should produce same result as zipping range with list
    assert_eq!(
        eval_lingo("(enumerate (list 10 20 30))"),
        eval_lingo("(zip (range 3) (list 10 20 30))")
    );
}

#[test]
fn sort_with_duplicates() {
    assert_eq!(eval_lingo("(sort (list 3 1 2 1 3))"), "(1 1 2 3 3)");
}

#[test]
fn flat_deeply_mixed() {
    // flat only flattens one level
    assert_eq!(
        eval_lingo("(flat (list (list 1 (list 2)) (list 3)))"),
        "(1 (2) 3)"
    );
}

#[test]
fn chunk_single_element_chunks() {
    assert_eq!(eval_lingo("(chunk 1 (list 1 2))"), "((1) (2))");
}

#[test]
fn uniq_with_strings() {
    assert_eq!(
        eval_lingo(r#"(uniq (list "a" "b" "a" "c" "b"))"#),
        "(a b c)"
    );
}

#[test]
fn has_list_with_string_element() {
    assert_eq!(eval_lingo(r#"(has (list "a" "b" "c") "b")"#), "true");
}

#[test]
fn split_then_join_roundtrip() {
    assert_eq!(
        eval_lingo(r#"(join "," (split "a,b,c" ","))"#),
        "a,b,c"
    );
}

#[test]
fn type_predicates_on_nil() {
    assert_eq!(eval_lingo("(nil? nil)"), "true");
    assert_eq!(eval_lingo("(int? nil)"), "false");
    assert_eq!(eval_lingo("(bool? nil)"), "false");
    assert_eq!(eval_lingo("(number? nil)"), "false");
}

#[test]
fn comparison_with_strings() {
    assert_eq!(eval_lingo(r#"(> "b" "a")"#), "true");
    assert_eq!(eval_lingo(r#"(<= "a" "a")"#), "true");
    assert_eq!(eval_lingo(r#"(>= "a" "b")"#), "false");
}

#[test]
fn eq_empty_lists() {
    assert_eq!(eval_lingo("(= (list) (list))"), "true");
}

#[test]
fn not_with_empty_list() {
    // Empty list is truthy (not in falsy set: false, 0, nil)
    assert_eq!(eval_lingo("(not (list))"), "false");
}

#[test]
fn min_max_with_equal_floats() {
    assert_eq!(eval_lingo("(min 2.0 2.0)"), "2.0");
    assert_eq!(eval_lingo("(max 2.0 2.0)"), "2.0");
}

#[test]
fn to_int_from_negative_float() {
    assert_eq!(eval_lingo("(->int -3.9)"), "-3");
}

#[test]
fn to_float_from_negative_int() {
    assert_eq!(eval_lingo("(->float -42)"), "-42.0");
}

#[test]
fn to_float_from_string_integer() {
    assert_eq!(eval_lingo(r#"(->float "42")"#), "42.0");
}

#[test]
fn range_negative_range_with_positive_step() {
    // range 5 2 should produce empty list (start > end, step positive implicit)
    assert_eq!(eval_lingo("(range 5 2)"), "()");
}

#[test]
fn cons_nested_list() {
    assert_eq!(eval_lingo("(cons (list 1 2) (list 3 4))"), "((1 2) 3 4)");
}

#[test]
fn nth_with_nested_list() {
    assert_eq!(eval_lingo("(nth (list 1 (list 2 3) 4) 1)"), "(2 3)");
}

#[test]
fn substring_empty_string() {
    assert_eq!(eval_lingo(r#"(substring "" 0 0)"#), "");
}

#[test]
fn abs_negative_float_zero() {
    assert_eq!(eval_lingo("(abs 0.0)"), "0.0");
}

#[test]
fn dbg_list_returns_list() {
    assert_eq!(eval_lingo("(dbg (list 1 2 3))"), "(1 2 3)");
}
