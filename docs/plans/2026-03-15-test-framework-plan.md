# Plan: Lingo Test Framework (2026-03-15)

## Summary

Add a minimal, native-feeling unit test framework to the Lingo programming
language. The framework provides assertion builtins (`assert_eq`, `assert_ne`)
implemented in Rust, a `--test` CLI flag that discovers and runs `test_*`
functions with per-test failure isolation, and a summary reporter. Tests are
ordinary Lingo functions -- no new syntax, no DSL, no external dependencies.

## Stakes Classification

**Level**: High

**Rationale**: This changes the interpreter's core execution path (adding a
test-mode branch to `run()`), adds new builtins to the evaluator, modifies
CLI argument parsing in `main.rs`, and introduces a new Rust module
(`src/test_runner.rs`). The changes touch the interpreter's error handling
boundary -- getting failure isolation wrong could mask bugs or crash the
runner. The framework also establishes conventions (argument order, naming,
output format) that will be difficult to change once tests are written
against them.

## Context

**Research**: `docs/plans/2026-03-15-test-framework-research.md`

**Affected Areas**:

- `src/main.rs` -- CLI argument parsing, `--test` flag routing
- `src/interpreter.rs` -- new builtins (`assert_eq`, `assert_ne`), public
  access to `values_equal` and `value_to_debug`
- `src/test_runner.rs` -- new module: test discovery, isolation, execution,
  reporting
- `src/ast.rs` -- no changes (spans already present on `Call` expressions)

## Design Decisions

### Assertion argument order: `assert_eq(actual, expected)`

The `actual, expected` order reads naturally in Lingo's left-to-right,
pipeline-oriented style. The expression under test comes first, matching how
Lingo code flows:

```lingo
add(2, 3) |> assert_eq(5)    # pipeline: result flows into assertion
assert_eq(add(2, 3), 5)      # direct: computation first, expectation second
```

The xUnit convention (`expected, actual`) exists because of JUnit's
historical `assertEquals(expected, actual)`, but Lingo has no such legacy.
The research document's examples and recommendation both use `actual, expected`.

Failure messages label clearly:

```text
FAIL test_add (math_test.ln:5)
  expected: 5
       got: 4
```

"expected" refers to the second argument; "got" refers to the first. This is
documented in the builtin's help text and consistent throughout the framework.

### Test file naming convention: `*_test.ln`

Test files use the `_test.ln` suffix (e.g., `math_test.ln`,
`string_test.ln`). This follows Go's convention and avoids confusion with
`test_*` function names inside the files. The CLI accepts any `.ln` file --
the naming convention is advisory, not enforced by the runner.

### How `--test` works

```text
lingo --test <file.ln>
```

When `--test` is present:

1. The file is parsed normally (lexer, parser, AST).
2. The interpreter's first pass collects all `FnDecl` items into the
   environment (existing behavior).
3. Instead of calling `main()`, the test runner scans the environment for
   functions whose names start with `test_`. Functions are sorted
   alphabetically for deterministic ordering.
4. Each `test_*` function is called in a child environment. If the call
   returns `Ok(_)`, the test passes. If it returns `Err(msg)`, the runner
   records the failure message and continues to the next test.
5. After all tests run, the reporter prints per-test results followed by a
   summary line.

`main()` is **not** called in test mode. Helper functions (not prefixed
with `test_`) are available to all tests but are not executed directly.

### Assertion builtins

Four assertion builtins are added in Phase 2:

**`assert_eq(actual, expected)`** -- 2 args. Pass if
`values_equal(actual, expected)`. Fail with
"expected: {expected}, got: {actual}".

**`assert_ne(actual, expected)`** -- 2 args. Pass if
`!values_equal(actual, expected)`. Fail with
"expected values to differ, both are: {actual}".

**`assert_true(value)`** -- 1 arg. Pass if `value.is_truthy()`. Fail with
"expected truthy value, got: {value}".

**`assert_false(value)`** -- 1 arg. Pass if `!value.is_truthy()`. Fail with
"expected falsy value, got: {value}".

All assertions return `Value::Unit` on success or `Err(String)` on failure.
The error string is structured so the test runner can extract and display it.
Assertions use the existing `values_equal` method for equality (structural
equality for `List`, `Tuple`; value equality for scalars) and `value_to_debug`
for formatting values in failure messages.

### Test isolation

Each `test_*` function runs in a **child environment** cloned from the
top-level environment (which contains all `FnDecl` items and builtins). This
means:

- Tests cannot pollute each other's variable bindings.
- A mutation inside one test (e.g., `push` on a list) does not affect other
  tests because the environment is cloned per-test.
- If a test returns `Err`, the runner catches it at the Rust level
  (`Result<Value, String>` from `call_function`), records the failure, and
  continues. The interpreter state is restored to the pre-test environment.

This is a minimal change: the test runner calls `call_function` (which
already saves/restores `self.env`) and matches on the `Result`. No new error
handling machinery is needed.

### Reporter output format

**Passing run:**

```text
PASS test_add_positive
PASS test_add_negative
PASS test_list_pipeline

3 passed, 0 failed
```

**Failing run:**

```text
PASS test_add_positive
FAIL test_add_negative
PASS test_list_pipeline

failures:

  test_add_negative:
    expected: -3
         got: -4

2 passed, 1 failed
```

Design choices:

- Per-test PASS/FAIL lines print as tests run (immediate feedback).
- Failure details are collected and printed after all tests in a
  "failures:" section, so they are grouped together for easy reading.
- The summary line is always the last line of output.
- Exit code: 0 if all tests pass, 1 if any test fails.
- Output goes to stdout (not stderr), so it can be piped and parsed.

### Self-testing strategy

The framework tests itself through two mechanisms:

1. **Rust-level integration tests** (`tests/test_runner.rs`): Parse and run
   `.ln` test files programmatically, asserting on the runner's return value
   (pass/fail counts) and captured stdout. This tests the runner, isolation,
   and reporter without depending on the Lingo-level assertions being correct.

2. **Lingo-level test files** (`examples/test_framework_test.ln`): A `.ln`
   file that uses the framework to test itself -- e.g., `assert_eq(1, 1)`
   passes, `assert_ne(1, 2)` passes, verifying the assertion builtins work
   end-to-end. Run with `lingo --test examples/test_framework_test.ln`.

The Rust tests are authoritative; the Lingo tests are a smoke test and
documentation of intended usage.

## Success Criteria

- [x] `lingo --test file.ln` discovers and runs all `test_*` functions
- [x] `assert_eq(actual, expected)` produces a clear failure message with
      expected/actual values when values differ
- [x] `assert_ne(actual, expected)` fails when values are equal
- [x] `assert_true(value)` and `assert_false(value)` test truthiness
- [x] A failing test does not prevent subsequent tests from running
- [x] The summary line reports correct pass/fail counts
- [x] Exit code is 0 for all-pass, 1 for any-fail
- [x] Tests are isolated: mutation in one test does not affect another
- [x] `lingo file.ln` (without `--test`) behavior is unchanged
- [x] Rust integration tests verify the framework end-to-end

## Implementation Steps

### Phase 1: Assertion Builtins

Add the four assertion builtins to the interpreter. These are pure additions
to `call_builtin` and the builtin registration list -- no existing behavior
changes.

#### Step 1.1: Test `assert_eq` builtin (RED)

- **Files**: `tests/test_runner.rs` (new file)
- **Action**: Create a Rust integration test file. Write tests that parse and
  interpret Lingo source strings containing `assert_eq` calls, verifying:
  - `assert_eq(1, 1)` returns `Ok(Value::Unit)`
  - `assert_eq("hello", "hello")` returns `Ok(Value::Unit)`
  - `assert_eq([1, 2], [1, 2])` returns `Ok(Value::Unit)` (structural equality)
  - `assert_eq((1, 2), (1, 2))` returns `Ok(Value::Unit)` (tuple equality)
  - `assert_eq(true, true)` returns `Ok(Value::Unit)`
  - `assert_eq(1, 2)` returns `Err` containing "expected: 2" and "got: 1"
  - `assert_eq("a", "b")` returns `Err` containing "expected" and "got"
  - `assert_eq([1], [1, 2])` returns `Err`
  - `assert_eq(1, "1")` returns `Err` (cross-type inequality)
- **Verify**: Tests compile and fail (builtin does not exist yet)
- **Complexity**: Medium

#### Step 1.2: Test `assert_ne` builtin (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Add tests for `assert_ne`:
  - `assert_ne(1, 2)` returns `Ok(Value::Unit)`
  - `assert_ne("a", "b")` returns `Ok(Value::Unit)`
  - `assert_ne(1, 1)` returns `Err` containing "expected values to differ"
  - `assert_ne([1, 2], [1, 2])` returns `Err`
- **Verify**: Tests compile and fail
- **Complexity**: Small

#### Step 1.3: Test `assert_true` and `assert_false` builtins (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Add tests for truthiness assertions:
  - `assert_true(true)` returns `Ok(Value::Unit)`
  - `assert_true(1)` returns `Ok(Value::Unit)` (non-zero int is truthy)
  - `assert_true("hello")` returns `Ok(Value::Unit)` (non-empty string)
  - `assert_true(false)` returns `Err` containing "expected truthy"
  - `assert_true(0)` returns `Err`
  - `assert_true(())` returns `Err` (Unit is falsy)
  - `assert_false(false)` returns `Ok(Value::Unit)`
  - `assert_false(0)` returns `Ok(Value::Unit)`
  - `assert_false(true)` returns `Err` containing "expected falsy"
  - `assert_false(1)` returns `Err`
  - `assert_false("hello")` returns `Err`
- **Verify**: Tests compile and fail
- **Complexity**: Small

#### Step 1.4: Implement assertion builtins (GREEN)

- **Files**: `src/interpreter.rs:147-156` (builtin registration),
  `src/interpreter.rs:1062-1068` (near existing `assert`)
- **Action**:
  1. Add `"assert_eq"`, `"assert_ne"`, `"assert_true"`, `"assert_false"` to
     the `builtins` vec in `Interpreter::new()`.
  2. Add match arms in `call_builtin()`:
     - `"assert_eq"`: extract two args, call `self.values_equal()`, return
       `Ok(Value::Unit)` or `Err` with formatted message using
       `self.value_to_debug()`.
     - `"assert_ne"`: extract two args, call `self.values_equal()`, return
       `Err` if equal.
     - `"assert_true"`: extract one arg, check `is_truthy()`.
     - `"assert_false"`: extract one arg, check `!is_truthy()`.
- **Verify**: All tests from Steps 1.1-1.3 pass
- **Complexity**: Small

### Phase 2: Test Runner Module

Create the test runner as a new Rust module that handles discovery, execution,
isolation, and reporting.

#### Step 2.1: Test discovery logic (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Write tests that verify the runner correctly discovers `test_*`
  functions from a parsed program:
  - A file with `fn test_a() {}` and `fn test_b() {}` discovers both
  - A file with `fn helper() {}` and `fn test_a() {}` discovers only `test_a`
  - A file with `fn testing_thing() {}` does NOT discover it (must be `test_`
    prefix, not `test` prefix)
  - A file with no `test_*` functions discovers zero tests (not an error)
  - Discovery order is alphabetical: `test_b` before `test_z`, `test_a`
    before `test_b`
- **Verify**: Tests compile and fail (runner module does not exist yet)
- **Complexity**: Small

#### Step 2.2: Test failure isolation (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Write tests that verify isolation:
  - A file with `fn test_fail() { assert_eq(1, 2) }` and
    `fn test_pass() { assert_eq(1, 1) }` -- both tests run, runner reports
    1 passed, 1 failed
  - A file with `fn test_mutate() { ... }` that mutates a variable does not
    affect a subsequent `test_read` function
  - A file where `test_a` errors (e.g., calls undefined variable) -- runner
    records error and continues to `test_b`
- **Verify**: Tests compile and fail
- **Complexity**: Medium

#### Step 2.3: Test reporter output (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Write tests that capture stdout and verify output format:
  - All-pass file produces `PASS test_name` lines and `N passed, 0 failed`
  - Mixed file produces `PASS`/`FAIL` lines, a `failures:` section with
    details, and correct summary counts
  - Zero-test file produces `0 passed, 0 failed` (no error)
- **Verify**: Tests compile and fail
- **Complexity**: Medium

#### Step 2.4: Implement test runner module (GREEN)

- **Files**: `src/test_runner.rs` (new file), `src/main.rs` (add `mod`)
- **Action**: Create `src/test_runner.rs` with:
  1. A `TestResult` enum: `Pass`, `Fail { name, message }`,
     `Error { name, message }`.
  2. A `discover_tests(env: &Env) -> Vec<String>` function that scans the
     environment for `test_*` function names, sorted alphabetically.
  3. A `run_tests(interpreter: &mut Interpreter, test_names: &[String]) -> TestSummary`
     function that:
     - Iterates over test names.
     - For each, calls `interpreter.call_function()` on the test function.
     - Matches on `Ok(_)` (pass) or `Err(msg)` (fail/error).
     - Prints `PASS`/`FAIL` per test.
     - Collects failures.
  4. A `report_summary(summary: &TestSummary)` function that prints the
     failures section and summary line.
  5. A `run_test_mode(interpreter: &mut Interpreter) -> Result<(), String>`
     entry point that orchestrates discover, run, report, and returns `Err`
     if any test failed (to trigger exit code 1).
- **Verify**: All tests from Steps 2.1-2.3 pass
- **Complexity**: Medium

Note: The `Interpreter` struct's `env` field and `call_function` method may
need to be made `pub` (or `pub(crate)`) for the test runner module to access
them. This is a minimal visibility change -- `call_function` and `env` are
implementation details promoted to crate-internal API.

### Phase 3: CLI Integration

Wire the `--test` flag into `main.rs` so the runner is invokable from the
command line.

#### Step 3.1: Test CLI `--test` flag parsing (RED)

- **Files**: `tests/test_runner.rs`
- **Action**: Write integration tests that invoke the full pipeline:
  - `lingo --test examples/test_framework_test.ln` exits 0 when all tests
    pass
  - `lingo --test` with a file containing a failing test exits 1
  - `lingo --test` without a filename prints usage error
  - `lingo file.ln` (no `--test`) calls `main()` as before (regression test)
- **Verify**: Tests compile and fail
- **Complexity**: Small

#### Step 3.2: Implement `--test` flag in main.rs (GREEN)

- **Files**: `src/main.rs`
- **Action**: Modify `main()` to:
  1. Check if `args` contains `"--test"`.
  2. If `--test` is present, extract the filename from the remaining args.
  3. After parsing, call `test_runner::run_test_mode(&mut interpreter)`
     instead of `interpreter.run(&program)`.
  4. The test runner still needs the program's functions loaded, so call
     the interpreter's first pass (function collection) before handing off
     to the runner. This may require extracting the first-pass logic into a
     separate method (e.g., `interpreter.load_declarations(&program)`).
  5. Exit with code 1 if the runner returns `Err`.
- **Verify**: All tests from Step 3.1 pass, plus manual verification:
  `cargo run -- --test examples/test_framework_test.ln`
- **Complexity**: Medium

### Phase 4: Self-Testing and Smoke Tests

#### Step 4.1: Create Lingo test file for assertion builtins

- **Files**: `examples/test_framework_test.ln` (new file)
- **Action**: Write a Lingo file that exercises the framework:

  ```lingo
  fn test_assert_eq_integers() {
    assert_eq(1, 1)
  }

  fn test_assert_eq_strings() {
    assert_eq("hello", "hello")
  }

  fn test_assert_eq_lists() {
    assert_eq([1, 2, 3], [1, 2, 3])
  }

  fn test_assert_eq_tuples() {
    assert_eq((1, "a"), (1, "a"))
  }

  fn test_assert_ne_different_values() {
    assert_ne(1, 2)
  }

  fn test_assert_true_truthy() {
    assert_true(true)
    assert_true(1)
    assert_true("nonempty")
  }

  fn test_assert_false_falsy() {
    assert_false(false)
    assert_false(0)
    assert_false(())
  }

  fn test_pipeline_with_assertions() {
    let result = [1, 2, 3]
      |> map(n => n * 2)
      |> filter(n => n > 2)
    assert_eq(result, [4, 6])
  }
  ```

- **Verify**: `cargo run -- --test examples/test_framework_test.ln` exits 0,
  prints all PASS, summary shows `N passed, 0 failed`
- **Complexity**: Small

#### Step 4.2: Verify failure output manually

- **Files**: None (manual verification)
- **Action**: Create a temporary file with an intentionally failing test and
  verify output format:
  - `FAIL` line appears for the failing test
  - `failures:` section shows expected/actual values
  - Summary line shows correct counts
  - Exit code is 1
- **Manual test cases**:
  - File with `fn test_fail() { assert_eq(1, 2) }` prints failure details
    showing `expected: 2` and `got: 1`
  - File with `fn test_ne_fail() { assert_ne(1, 1) }` prints
    "expected values to differ"
  - File mixing pass and fail reports both correctly in summary
- **Verify**: Output matches the format specified in the Reporter Output
  Format section above
- **Complexity**: Small

#### Step 4.3: Verify regression -- normal mode unchanged

- **Files**: None (manual verification)
- **Action**: Run existing example files without `--test` and verify they
  still work:
  - `cargo run -- examples/hello.ln`
  - `cargo run -- examples/fizzbuzz.ln`
  - `cargo run -- examples/basics.ln`
- **Verify**: All produce the same output as before this change
- **Complexity**: Small

## Test Strategy

### Automated Tests

| Test Case | Type | Input | Expected Output |
| --- | --- | --- | --- |
| `assert_eq` equal integers | Unit | `assert_eq(1, 1)` | `Ok(Value::Unit)` |
| `assert_eq` unequal integers | Unit | `assert_eq(1, 2)` | `Err` with "expected: 2" and "got: 1" |
| `assert_eq` structural list equality | Unit | `assert_eq([1,2], [1,2])` | `Ok(Value::Unit)` |
| `assert_eq` cross-type | Unit | `assert_eq(1, "1")` | `Err` |
| `assert_ne` unequal values | Unit | `assert_ne(1, 2)` | `Ok(Value::Unit)` |
| `assert_ne` equal values | Unit | `assert_ne(1, 1)` | `Err` with "expected values to differ" |
| `assert_true` truthy | Unit | `assert_true(true)` | `Ok(Value::Unit)` |
| `assert_true` falsy | Unit | `assert_true(false)` | `Err` with "expected truthy" |
| `assert_false` falsy | Unit | `assert_false(0)` | `Ok(Value::Unit)` |
| `assert_false` truthy | Unit | `assert_false(1)` | `Err` with "expected falsy" |
| Discovery finds `test_*` only | Integration | File with mixed fns | Only `test_` prefixed names |
| Discovery ignores `testing_*` | Integration | `fn testing_foo() {}` | Not discovered |
| Discovery alphabetical order | Integration | `test_b`, `test_a` | `["test_a", "test_b"]` |
| Isolation: fail then pass | Integration | Failing test + passing test | Both run, 1 pass 1 fail |
| Isolation: mutation | Integration | Test mutates, next test reads | No cross-contamination |
| Reporter all-pass format | Integration | All passing tests | Correct stdout format |
| Reporter mixed format | Integration | Mix of pass/fail | Failures section + summary |
| CLI `--test` exit 0 | Integration | All tests pass | Process exits 0 |
| CLI `--test` exit 1 | Integration | Any test fails | Process exits 1 |
| CLI no `--test` regression | Integration | `examples/hello.ln` | Same output as before |

### Manual Verification

- [ ] `cargo run -- --test examples/test_framework_test.ln` prints all PASS
      and `N passed, 0 failed`
- [ ] A file with an intentional failure shows the `failures:` section with
      expected/actual values
- [ ] `cargo run -- examples/hello.ln` works identically to before the change
- [ ] `cargo run -- --test` with no filename shows a usage error

## Risks and Mitigations

**Making `env`/`call_function` pub breaks encapsulation.**
Impact: Medium -- future refactors constrained by public API surface.
Mitigation: Use `pub(crate)` not `pub`; document these as internal APIs.

**Assertion error format conflicts with user error messages.**
Impact: Low -- runner cannot distinguish assertion failure from runtime error.
Mitigation: Prefix assertion errors with a sentinel (e.g., `"[assert] "`) so
the runner can classify failures vs. errors.

**`values_equal` returns false for `Int(1)` vs `Float(1.0)`.**
Impact: Medium -- surprising to users writing numeric tests.
Mitigation: Document this limitation; defer cross-type numeric comparison to
a future enhancement.

**Test discovery finds `test_` in nested scopes (closures).**
Impact: Low -- only top-level `FnDecl` items are scanned.
Mitigation: Discovery scans the top-level environment only, not closures.

**Large test files run slowly (tree-walking interpreter).**
Impact: Low -- unlikely for unit tests.
Mitigation: No mitigation needed for MVP; performance is a future concern.

## Rollback Strategy

All changes are additive:

- New builtins in `call_builtin` can be removed by deleting match arms and
  registration entries.
- `src/test_runner.rs` is a new file that can be deleted entirely.
- `main.rs` changes are a small conditional branch that can be reverted.
- No existing tests or behavior are modified (only new code paths behind
  `--test`).

If rollback is needed: revert the commits. No data migration or state
cleanup required.

## Status

- [x] Plan approved
- [x] Phase 1: Assertion builtins complete
- [x] Phase 2: Test runner module complete
- [x] Phase 3: CLI integration complete
- [x] Phase 4: Self-testing complete
