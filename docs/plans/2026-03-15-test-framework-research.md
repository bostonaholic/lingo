# Research: Lingo Test Framework (2026-03-15)

## Problem Statement

Lingo is a general-purpose interpreted language (Rust-hosted, tree-walking interpreter) with zero
test infrastructure at either the Rust or Lingo level. The language needs a minimal, native-feeling
unit test framework that works within the constraints of the current implementation while aligning
with Lingo's design philosophy of minimal ceremony, pipeline-friendly composition, and
expression-oriented programming.

## Requirements

1. **Failure isolation** -- one failing test must not prevent other tests from running.
2. **Rich failure messages** -- assertions must report expected vs. actual values, not just
   "Assertion failed."
3. **Test discovery** -- tests must be identifiable without manual registration boilerplate.
4. **Summary reporting** -- a pass/fail/error count must be printed after all tests run.
5. **Idiomatic Lingo** -- the framework API must use `snake_case`, compose with `|>` where
   natural, and feel like ordinary Lingo code rather than a foreign DSL.
6. **Zero external dependencies** -- Lingo's `Cargo.toml` has no dependencies; the framework
   should not introduce any.

## Findings

### Language Capabilities and Constraints

Lingo is dynamically typed at runtime (Hindley-Milner type inference is specified but not
implemented). Type annotations are parsed and ignored. The interpreter supports 11 value types
(`Int`, `Float`, `Str`, `Bool`, `Unit`, `Tuple`, `List`, `Fn`, `Lambda`, `BuiltinFn`) with
truthiness rules for conditionals.

The language is **single-file only** -- there is no module system, no imports, and no multi-file
support. The interpreter reads one `.ln` file, parses it, and either calls `main()` or executes
top-level expressions. This means tests must either live alongside code in the same file or in
separate `.ln` files run independently.

There is **no metaprogramming**: no macros, no decorators, no reflection (except `type_of`), no
`eval`. Test discovery cannot rely on annotations or reflection. However, functions are first-class
values -- they can be stored in lists, passed as arguments, and called dynamically, which enables
registration-based and convention-based approaches.

**Errors terminate execution.** There is no try/catch, no `Result`/`Option` types at the language
level (specified but not implemented), and no `panic` builtin. The existing `assert(value)` builtin
halts the program on falsy values with `Err("Assertion failed")`. This is the single hardest
constraint: failure isolation **cannot** be implemented in pure Lingo and **must** be handled at
the interpreter (Rust) level.

### Existing Building Blocks

The interpreter provides 44 builtins, three of which are directly relevant to testing:

| Builtin | Behavior | Limitation |
| ------- | -------- | ---------- |
| `assert(value)` | Returns `Err("Assertion failed")` if falsy | No context: no expected/actual, no message |
| `dbg(value)` | Prints debug representation to stderr, returns value | Useful for test output |
| `type_of(value)` | Returns type name as `Str` | Enables type-checking assertions |

Additionally, `to_str(value)` converts any value to a string representation, which is necessary
for formatting failure messages. String interpolation (`"expected {expected}, got {actual}"`)
provides ergonomic message construction.

Missing primitives: `assert_eq`, `assert_ne`, test runner, failure isolation, result summary,
test timing, and source location reporting.

### Interpreter Architecture

The execution pipeline is: `.ln` file -> Lexer (tokens) -> Parser (AST) -> Interpreter (values).
The interpreter is a tree-walking evaluator with lexical scoping via a parent-pointer environment
chain. The AST carries span information (`line`, `col`) on function declarations, which could
be leveraged for source-location reporting in failure messages.

The entry point (`src/main.rs`) is 46 lines. The `run()` function is the natural place to add a
test-mode branch. The interpreter's `run()` method first collects all `FnDecl` items into the
environment, then calls `main()` -- this two-pass approach means function declarations are already
available for discovery before execution begins.

Key Rust-side files:

| File | Lines | Role |
| ---- | ----- | ---- |
| `src/main.rs` | 46 | Entry point, `run()` function |
| `src/lexer.rs` | 622 | Tokenizer |
| `src/ast.rs` | 151 | AST node definitions |
| `src/parser.rs` | 896 | Recursive descent parser |
| `src/interpreter.rs` | 1361 | Tree-walking evaluator, all 44 builtins |

### How Other Languages Solve This

**Convention-based discovery with interpreter-level isolation** is the pattern used by Go
(`go test` discovers `Test*` functions) and Rust (`cargo test` discovers `#[test]` functions).
Both run each test in isolation and report results. This is the closest match for Lingo's
constraints because:

- Lingo has no macros or annotations, ruling out Elixir/Julia-style macro-based frameworks.
- Lingo has no error propagation mechanism (`?`, `try`), ruling out Zig's error-as-failure model.
- Lingo has first-class functions, making Go's "tests are just functions" approach natural.
- The interpreter already collects all `FnDecl` items before execution, making `test_*` discovery
  trivial.

**The Lox/Wren inline annotation pattern** (`// expect: value`) is valuable for testing the
interpreter itself from the Rust side, but is orthogonal to an in-language test framework. Both
approaches serve different purposes and can coexist.

**MinUnit's insight** -- "the important thing about unit testing is the testing, not the
framework" -- reinforces that the framework should be minimal. The irreducible core is:
an assertion primitive, a test compositor (runner), and a summary reporter.

**Lua Busted and Wren-test** demonstrate that languages with first-class functions and closures
naturally express test organization as function composition. Lingo fits this pattern. However,
BDD-style (`describe`/`it`) nesting is more ceremony than Lingo's philosophy warrants --
ExUnit's deliberate prohibition of nested describe blocks validates the flat `test_*` function
approach.

### Error Reporting Design

External research strongly emphasizes that failure messages must answer four questions without
further investigation: (1) what was being tested, (2) what was expected, (3) what was produced,
and (4) where the assertion failed.

The recommended failure output format:

```text
FAIL test_name (file.ln:42)
  expected: 3
       got: 4
```

**Argument order** must be consistent. The two conventions are `assert_eq(expected, actual)`
(xUnit tradition) and `assert_eq(actual, expected)`. The codebase research's example code uses
`assert_eq(actual, expected)` (e.g., `assert_eq(add(2, 3), 5)`) which reads more naturally in
Lingo's left-to-right pipeline style. This should be the chosen convention, documented clearly.

**Source location** is essential for test files with multiple assertions. Lingo's AST already
tracks `line` and `col` on `FnDecl` nodes. Extending span tracking to `Call` expressions would
allow assertion builtins to report the exact line of failure.

**Distinguishing failures from errors** is important: a failed assertion (the code ran but
produced the wrong value) is semantically different from an unexpected error (the code crashed).
The test runner should report these differently.

### Tiered Primitives

Consolidating both sources, the primitives needed for Lingo's test framework fall into tiers:

**Tier 1 -- Must have (MVP):**

- `assert_eq(actual, expected)` -- value equality with expected/actual message
- `assert_ne(actual, expected)` -- value inequality
- `test_*` function discovery by naming convention
- Per-test failure isolation (interpreter-level)
- Summary reporter (pass/fail/error counts)

**Tier 2 -- Should have (ergonomics):**

- `assert(condition)` with optional message parameter (upgrade existing builtin)
- Source location (file:line) in failure messages
- Test timing
- Failure vs. error distinction in output

**Tier 3 -- Nice to have (future):**

- `assert_error(fn)` -- verify that a function raises an error
- `skip` / `pending` markers for known-failing tests
- Test filtering by name (`--test test_name`)
- Snapshot/expect tests for testing the parser and interpreter

### Implementation Strategy

Both sources converge on a **hybrid approach**: new assertion builtins (Rust-side) combined with
an interpreter-level test runner. This maps to a phased plan:

**Phase 1: Host-level inline annotation tests.** Add a Rust-side test runner that reads `.ln`
files with `# expect: value` annotations and diffs stdout. This tests the interpreter itself
and requires zero language changes. This is the Lox/Wren pattern adapted for Lingo's `#` comment
syntax.

**Phase 2: Assertion builtins.** Add `assert_eq` and `assert_ne` as interpreter builtins that
produce structured failure messages with expected/actual values. Upgrade `assert` to accept an
optional message string.

**Phase 3: Test runner mode.** Add a `--test` flag (or equivalent) to the interpreter. When
active, the runner:

1. Collects all functions whose names start with `test_` (already available from the first pass
   of `Interpreter::run`).
2. Runs each test function in its own environment, catching `Err` returns to isolate failures.
3. Reports results per-test (PASS/FAIL/ERROR with details).
4. Prints a summary line: `N passed, M failed, K errors`.

**Phase 4: Self-testing.** Write test files that exercise the framework itself: verify that
`assert_eq(1, 2)` produces the correct failure message, that the runner counts correctly, and
that errors are distinguished from failures.

### What Idiomatic Lingo Tests Look Like

Both sources agree on the target syntax. A test file should look like ordinary Lingo code:

```lingo
# math_test.ln

fn add(a, b) { a + b }

fn test_add_positive() {
  assert_eq(add(2, 3), 5)
}

fn test_add_negative() {
  assert_eq(add(-1, -2), -3)
}

fn test_list_pipeline() {
  let result = [1, 2, 3]
    |> map(n => n * 2)
    |> filter(n => n > 2)
  assert_eq(result, [4, 6])
}
```

Run with: `lingo --test math_test.ln`

Expected output:

```text
PASS test_add_positive
PASS test_add_negative
PASS test_list_pipeline

3 passed, 0 failed
```

## External Research

Findings from external sources with confidence assessments:

**Inline annotation pattern** (`// expect:`) is effective for interpreter testing.
Source: Crafting Interpreters, LoxLox. Confidence: High -- battle-tested across many
implementations.

**Convention-based discovery** (`test_*`) is the most ergonomic approach for languages
without reflection. Source: Go testing, Rust testing. Confidence: High -- proven at scale.

**Irreducible test framework** is assert + runner + reporter.
Source: MinUnit. Confidence: High -- formally minimal.

**Failure messages** must show expected, actual, and source location.
Source: Go, ExUnit, Zig, Kotlin power-assert. Confidence: High -- universal consensus.

**Nested describe blocks** reduce maintainability.
Source: ExUnit design decision. Confidence: Medium -- opinionated but well-reasoned.

**Snapshot tests** are valuable for testing parsers and interpreters.
Source: Jane Street ppx_expect. Confidence: Medium -- applicable but adds complexity.

**Skip/pending markers** prevent test deletion.
Source: ExUnit, tcltest. Confidence: High -- practical observation.

## Technical Constraints

1. **No module system.** Tests cannot import code from other files. Each test file must be
   self-contained or include the functions under test.
2. **Errors halt execution.** Failure isolation requires interpreter-level changes (catching `Err`
   returns per-test in Rust). This cannot be worked around in pure Lingo.
3. **No macros or AST introspection.** Power-assert style expression introspection (showing
   intermediate values in a failing expression) would require significant interpreter changes
   and is not feasible for the MVP.
4. **Span tracking is incomplete.** `FnDecl` nodes have spans, but `Call` expressions may not.
   Source-location reporting in assertion failures may require extending span tracking in the
   parser.
5. **No external dependencies.** The Rust implementation has zero crate dependencies (`Cargo.toml`
   has no `[dependencies]`). Any test infrastructure must be built from scratch.
6. **Dynamic typing.** `assert_eq` must compare runtime `Value` variants. Equality semantics
   must be defined for all value types, including structural equality for `List` and `Tuple`.

## Open Questions

1. **Argument order for `assert_eq`.** The codebase examples use `assert_eq(actual, expected)`.
   Should this be formalized, or should `assert_eq(expected, actual)` (xUnit convention) be
   preferred? The examples feel more natural with actual-first in Lingo's pipeline style.

2. **Test file naming convention.** Should test files follow `*_test.ln` (Go-style) or
   `test_*.ln`? The `*_test.ln` suffix convention is more common and avoids confusion with
   `test_*` function names.

3. **Where do test files live?** Alongside source in the same directory (Go-style), in a
   separate `tests/` directory (Rust-style), or either? Without a module system, colocation
   may be simpler.

4. **Should `assert_eq` support approximate equality for floats?** Float comparison with an
   epsilon tolerance is a common need. This could be deferred or handled with a separate
   `assert_approx_eq(actual, expected, tolerance)` builtin.

5. **How should the Phase 1 annotation runner interact with Phase 3 test runner?** Are they
   separate tools, or does `--test` subsume the annotation pattern?

6. **Should the test runner support setup/teardown?** Functions like `setup()` or `before_each()`
   called before each test. This adds complexity but is a common need. Could be deferred to a
   later phase.

## Recommendations

1. **Start with Phase 1 (inline annotations) immediately.** This requires no language changes and
   provides regression testing for the interpreter right away. Adapt the Lox/Wren pattern using
   `# expect:` comments.

2. **Implement Phase 2 and Phase 3 together.** The assertion builtins and test runner are
   co-dependent -- `assert_eq` needs failure isolation to be useful, and the runner needs
   `assert_eq` to produce meaningful output. Ship them as a single feature.

3. **Use `assert_eq(actual, expected)` argument order.** This reads naturally in Lingo's
   left-to-right style and matches the example code already written. Document it clearly and
   enforce consistency.

4. **Use `test_*` function naming convention for discovery.** This mirrors Go's approach, requires
   no new syntax or keywords, and works with the interpreter's existing first-pass function
   collection. The `test` keyword (Zig-style) would require parser changes that are not justified
   for the MVP.

5. **Implement failure isolation by catching `Err` returns per-test in the Rust runner.** Each
   `test_*` function is called in its own child environment. If the call returns `Err`, the
   runner records the failure and continues to the next test. This is a minimal, targeted change
   to the interpreter.

6. **Defer Phase 4 (self-testing), snapshot tests, and skip/pending markers.** These are
   valuable but not blocking. The MVP should focus on assert + discover + isolate + report.

## Sources

**`docs/plans/2026-03-15-test-framework-codebase.md`**
Lingo implementation analysis: syntax, types, builtins, interpreter architecture,
constraints, gap analysis.

**`docs/plans/2026-03-15-test-framework-external.md`**
External research: test framework patterns (Go, Elixir, Zig, Lua, Wren, Julia),
error reporting, bootstrap ordering, minimal primitives.
