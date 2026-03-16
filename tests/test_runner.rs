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

