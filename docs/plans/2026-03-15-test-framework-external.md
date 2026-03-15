# Test Framework Design for Small Interpreted Languages

Research conducted 2026-03-15.

---

## Executive Summary

The most effective test frameworks for small interpreted languages share three
properties: they are **minimal in primitives**, **idiomatic to the host
language** rather than ported from xUnit, and they produce **failure messages
that make the problem immediately obvious without a debugger**. The most
practical pattern for a language under development is inline output annotation
(the Lox/Wren `// expect:` style), which requires no test framework at all
inside the language itself—just a host-language test runner. Once the language
is mature enough to write meaningful programs, a native framework built on the
language's own idioms becomes the natural next step.

---

## 1. Test Framework Design for Interpreted Languages

### The Lox / Wren Annotation Pattern

Bob Nystrom's Crafting Interpreters project pioneered a test approach that
requires zero framework code inside the language under test. Test files are
ordinary source files annotated with comment-embedded expectations:

```lox
// Variables
var a = 1;
var b = 2;
print a + b; // expect: 3
print a * b; // expect: 2
```

A host-language runner (in Dart for Crafting Interpreters, in Python for Wren)
reads each source file, extracts `// expect:` lines, runs the interpreter, and
diffs stdout against the collected expectations. Error expectations use a
separate convention:

```lox
var a = "too"; var b = "many";
var c = a + b + c; // expect runtime error: Undefined variable 'c'.
```

**Why this works for interpreters:** The language under development need not
have any testing capabilities at all. The tests exercise the interpreter
externally. The test runner can track which tests are expected to pass at each
chapter/milestone of development—as Nystrom did, allowing the full CI suite to
be run against partial implementations.

**Practical takeaway:** For a language in early development, start with this
pattern. The overhead is one small host-language script. The test files also
serve as specification and documentation simultaneously.

### Self-Hosting Validation (LoxLox)

Ben Hoyt's LoxLox—a Lox interpreter written in Lox—validated correctness by
reusing the canonical test suite with a modified runner. The self-hosted
interpreter passed 207/234 tests out of the box; the remaining 27 failures were
all error-message formatting differences, not semantic errors. This demonstrates:

- Canonical test suites are reusable across implementations.
- Distinguishing **semantic correctness** from **presentation** in test
  evaluation is essential when bootstrapping.
- A self-hosting interpreter is itself the strongest integration test of the
  language.

### MinUnit: The Irreducible Minimum

The C MinUnit project by John Treadway distills a test framework to two macros
and a run loop—demonstrating that the conceptual minimum for unit testing is:

1. **An assertion primitive** — evaluate a condition, return a failure message
   on false, null on true.
2. **A test compositor** — run a function, propagate its failure message if
   non-null, count passes.

The insight: "the important thing about unit testing is the testing, not the
framework." Everything else—reporters, lifecycle hooks, parallelism—is
convenience layered on top of this core.

---

## 2. Language-Native Test Framework Design

The clearest lesson from studying mature ecosystems is that the best test
frameworks do not port xUnit—they extend the host language's own idioms into
testing.

### Go: Tests Are Just Code

Go's `testing` package embodies the philosophy that "tests are just code."
There is no framework to learn. A test is a function named `Test*` that
receives a `*testing.T`. The developer writes plain Go.

**Table-driven tests** emerge from this naturally:

```go
var cases = []struct {
    input    string
    expected int
}{
    {"hello", 5},
    {"",      0},
}

func TestLen(t *testing.T) {
    for _, tc := range cases {
        t.Run(tc.input, func(t *testing.T) {
            if got := myLen(tc.input); got != tc.expected {
                t.Errorf("myLen(%q) = %d; want %d", tc.input, got, tc.expected)
            }
        })
    }
}
```

Key design decisions:

- `t.Errorf` logs and continues; `t.Fatalf` stops. The choice is explicit, not
  implicit.
- Using a map (instead of slice) for test cases ensures random iteration order,
  preventing tests from depending on each other accidentally.
- `t.Run` introduced subtests in Go 1.7, enabling `go test -run TestLen/hello`
  to run a single case.
- No assertion library. The error message is a plain format string, which
  forces the developer to write meaningful messages.

**Design principle:** Making the developer write the failure message manually
produces better messages than auto-generated ones, but only when the framework
makes it easy. Go strikes this balance by providing `t.Errorf` with `%q`/`%v`
verbs that format values clearly.

### Elixir ExUnit: Macros as First-Class Test Infrastructure

ExUnit treats testing as a language-level concern. The `test` macro is
syntactically parallel to `def`—tests read the same as functions:

```elixir
defmodule MathTest do
  use ExUnit.Case, async: true

  test "adds two numbers" do
    assert Math.add(1, 2) == 3
  end
end
```

Native Elixir idioms embedded in the test framework:

- **`assert` with pattern matching:** `assert {:ok, value} = some_function()`
  — the left side is a pattern; if it does not match, the failure message shows
  the right-hand value, making what was returned immediately visible.
- **`refute`:** the positive/negative symmetry avoids double negatives in
  assertions.
- **Doctests:** `doctest MyModule` automatically extracts and runs `iex>` code
  examples from documentation. Documentation and tests are one.
- **`async: true`:** concurrent modules are opt-in at the module level, and
  within a module tests run serially. The model mirrors Elixir's process model.
- **No nested describe blocks:** ExUnit explicitly forbids nesting. This is a
  deliberate maintainability constraint—deeply nested test hierarchies become
  hard to read, so the framework enforces flat structure.
- **`setup` returns `{:ok, state}`:** aligns with Elixir's tagged-tuple
  convention for expressing success/failure.

**Design principle:** If your language has a distinctive evaluation model
(pattern matching, tagged tuples, fibers, first-class functions), expose that
model in the test API rather than flattening it into assert/expect equality
checks.

### Zig: Tests Embedded in Source, Errors as Values

Zig collapses the boundary between tests and implementation. Tests live in the
same file as the code they test, using the `test` keyword:

```zig
fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "add positive numbers" {
    try std.testing.expectEqual(@as(i32, 3), add(1, 2));
}
```

Design characteristics:

- `test` blocks are ignored in production builds.
- `std.testing` provides `expectEqual`, `expectEqualStrings`, `expectError`,
  and an allocator that detects memory leaks.
- Tests return errors (Zig's error union type). `try` propagates failure
  naturally—the same mechanism used in production code.
- Custom test helpers are just functions returning errors. There is no special
  assertion type.

**Design principle:** When the language has a native error-propagation
mechanism (`try`/`!`/`Result`/`?`), use it as the test failure mechanism.
Tests that use the same error semantics as the code they test feel more natural
and require no conceptual overhead.

### Lua Busted: First-Class Functions as Test Infrastructure

Busted is the canonical BDD-style framework for Lua. Because Lua has first-
class functions and closures, test organization maps directly onto function
composition:

```lua
describe("math", function()
  local value

  before_each(function()
    value = 0
  end)

  it("adds", function()
    assert.are.equal(3, value + 3)
  end)

  describe("nested context", function()
    it("multiplies", function()
      assert.are.equal(0, value * 5)
    end)
  end)
end)
```

Lua-specific design considerations Busted had to address:

- Lua keywords (`true`, `false`, `nil`, `not`, `function`) cannot be used as
  method names with dot notation. Busted uses underscore alternatives
  (`assert.is_true()`) or capitalisation (`assert.True()`).
- Global environment mutation is a real concern in Lua. Busted provides
  `insulate` blocks to prevent test-to-test pollution.
- The framework is extensible: custom assertions plug in via the same chained
  API, so domain-specific assertions read identically to built-in ones.

**Design principle:** Identify which of your language's keywords or reserved
words would conflict with a natural assertion API, and design around them
explicitly. A test framework that fights the language's syntax is always
awkward to use.

### Wren-Test: Block-Based Closures as Test Structure

Wren (Bob Nystrom's scripting language) has a block-based closure syntax.
wren-test maps this directly onto test organisation:

```wren
var TestString = Suite.new("String") { |it|
  it.suite("indexOf") { |it|
    it.should("return -1 when not found") {
      Expect.call("foo".indexOf("bar")).toEqual(-1)
    }
  }
}
```

The `Expect.call(value).toMethod()` pattern mirrors Wren's own `Fn.call()`
idiom. Matchers include Fiber-specific and Range-specific variants—directly
leveraging Wren's type system.

### Julia Test.jl: Macro-Expanded Expression Introspection

Julia's `@test` macro accepts any expression, not just comparisons:

```julia
@test 1 + 1 == 2
@test π ≈ 3.14159 atol=0.001
@testset "arithmetic" begin
    @test add(1, 2) == 3
    @test_throws DomainError sqrt(-1)
end
```

When a test fails, the macro has already captured the AST at expansion time,
so it can display:

- The original source expression
- The evaluated left-hand side
- The evaluated right-hand side
- The operator result

This is expression introspection via macros—the same mechanism used by
Kotlin's power-assert plugin and Groovy's Spock framework.

---

## 3. Minimal Test Framework Primitives

The irreducible set of primitives for a test framework, in ascending order of
necessity:

### Tier 1: Cannot test without these

| Primitive | Purpose |
| --------- | ------- |
| `assert(condition, message)` | Signal failure with a message |
| Test registration | Associate a name with a test function |
| Test runner | Iterate registered tests, catch failures, continue |
| Summary reporter | Print pass/fail counts when done |

### Tier 2: Needed for ergonomics (not logic)

| Primitive | Purpose |
| --------- | ------- |
| `assert_equal(expected, actual)` | Generate the expected/actual message automatically |
| `assert_error(fn, error_type)` | Test that code raises an expected error |
| Setup / teardown hooks | Share state initialization across tests |
| Test grouping (`suite`/`describe`/`testset`) | Namespace related tests, prefix output |
| Skip / pending markers | Mark known-failing tests without breaking CI |

### Tier 3: Quality-of-life additions

| Primitive | Purpose |
| --------- | ------- |
| Subtests with filtering | Run a single test by name |
| Soft assertions | Collect multiple failures before reporting |
| Source location in messages | File and line number in error output |
| Diff output for long strings | Side-by-side comparison instead of raw values |

### The Registration Tradeoff

There are two models for test registration:

**Explicit registration** — tests must be added to a runner:

```text
suite.add("my test", fn() { ... })
suite.run()
```

- Advantages: explicit control, composable, no magic
- Disadvantages: boilerplate, easy to forget to register

**Convention-based discovery** — runner finds tests by name prefix or
annotation:

```text
fn test_addition() { ... }  // auto-discovered if named test_*
```

- Advantages: zero ceremony
- Disadvantages: requires either runtime reflection or a build step

For small languages without reflection, explicit registration is more
practical. For languages with first-class functions and naming conventions,
discovery is more ergonomic.

---

## 4. Self-Hosting Test Patterns

Testing a language's own test framework using the framework itself creates a
useful validation loop, but requires care about what can be tested at each
bootstrap stage.

### The Bootstrap Ordering Problem

You cannot use the test framework to test itself until the test framework
works. The typical solution is a layered approach:

1. **Layer 0: Host-language tests** — Test the interpreter/VM itself in the
   implementation language (Go, Rust, Python, etc.). These use the host's test
   framework and are unconditionally reliable.

2. **Layer 1: Inline output annotation tests** — Test the language's evaluation
   semantics with `// expect:` annotations run by a host-side runner. No
   in-language test framework needed yet.

3. **Layer 2: In-language assertion tests** — Once `assert` or equivalent
   works, write tests that call `assert` directly. These can test the assertion
   function's own behavior.

4. **Layer 3: Full framework tests** — Once the framework primitives exist,
   test the framework itself: verify that `assert_equal(1, 2)` produces the
   right failure message, that `skip` marks tests correctly, that the runner
   counts failures accurately.

### What to Self-Test

The test framework's own suite should verify:

- `assert(true)` passes silently
- `assert(false)` fails with the provided message
- `assert_equal(1, 1)` passes
- `assert_equal(1, 2)` fails with "expected 1, got 2"
- A test that raises an unexpected exception is reported as an error, not a
  failure
- The summary line reports the correct counts

### Tcl's Approach

Tcl's `tcltest` package ships with Tcl's standard library and is used to test
Tcl itself. Its design reflects this dual role: the framework is designed for
regression testing of language implementations, not just application code.
Tests are structured as:

```tcl
test name description ?constraints? script expectedResult
```

The constraint system (platform, feature flags, known bugs) is central—a test
suite for a language implementation needs to express "this test is only valid
on this platform" or "this tests a known bug that is not yet fixed" at the
framework level, not as a comment.

**Practical takeaway:** A test framework used to test a language implementation
should have first-class support for expected failures and conditional execution.
These are not edge cases—they are routine when building incrementally.

---

## 5. Error Reporting Best Practices

### The Core Principle: Eliminate the Debugging Loop

A test failure message should answer, without any further investigation:

- What was the test checking?
- What value was expected?
- What value was actually produced?
- Where in the source did the assertion fail?

If any of these four questions requires opening a file, running a debugger, or
adding print statements, the failure message is insufficient.

### Expected vs Actual: Consistent Ordering

All assertion libraries must pick a consistent argument order. The two
conventions are:

- `assert_equal(expected, actual)` — xUnit convention, most common
- `assert_equal(actual, expected)` — some newer frameworks

The critical rule is to **never mix them**. When the order is wrong, failure
messages become confusing: "expected 42, got 3" when you meant the opposite.
Go's `t.Errorf("got %q, want %q", got, want)` sidesteps the issue by making
the label explicit—each value is labelled in the format string.

### Kotlin Power-Assert: Expression Introspection

The most sophisticated failure messages come from **expression introspection**
at compile/macro time. Kotlin's power-assert plugin transforms:

```kotlin
assert(hello.length == world.substring(1, 4).length)
```

into a failure message that shows every intermediate value:

```text
assert(hello.length == world.substring(1, 4).length)
       |     |      |  |     |              |
       Hello 5      |  world!orl            3
                    false
```

This approach requires either a macro system (Julia, Elixir, Rust's
`assert_eq!`) or a compiler plugin (Kotlin). For an interpreted language with
an AST, this is achievable: the assertion function receives the AST node, not
just the evaluated result, and can walk the AST to display sub-expression
values.

### Snapshot / Expect Tests (Jane Street ppx_expect)

Jane Street's expect tests invert the normal test-writing workflow:

1. Write the test, leave the expected value blank.
2. Run the tests. The framework captures actual output and inserts it as the
   expected value in the source file.
3. On subsequent runs, the framework diffs actual against the recorded expected
   value and shows only changes.

```ocaml
let%expect_test "addition" =
  printf "%d\n" (1 + 2);
  [%expect {| 3 |}]
```

This is particularly powerful for testing interpreters and compilers, where
the output is complex structured data (parse trees, bytecode, evaluation
traces) that would be painful to hand-write as assertions. The framework
makes the developer review output visually once, then enforces that it does not
change unintentionally.

**Practical takeaway:** For testing a language's parser, compiler, or
evaluator, snapshot tests dramatically reduce the cost of writing tests. The
"expected" value is generated by running the code—you review it once and then
it becomes a regression guard.

### Source Location

Source location in failure messages (file name + line number) is essential for
anything beyond the smallest test suite. When a test file has 50 assertions,
a message of "expected 3, got 4" is nearly useless without knowing which
assertion it came from.

Implementation options in order of engineering cost:

1. **Manual:** The developer passes `__FILE__` and `__LINE__` (or equivalent)
   to the assertion function.
2. **Stack inspection:** The test framework inspects the call stack at failure
   time. Works in languages with a runtime stack representation.
3. **Macro expansion:** In macro-capable languages, the assertion macro
   captures the source location automatically.
4. **Runner-side correlation:** The runner knows which test file is being
   executed; error output is matched to the source by line number.

### Diff Output for Strings

When comparing long strings or multi-line output, printing the raw expected
and actual values side by side is rarely helpful. A character-level or
line-level diff showing exactly where the values diverge is dramatically more
useful:

```text
expected: "Hello, World!\n"
     got: "Hello, world!\n"
                   ^--- character 8 differs (capital W vs lowercase w)
```

This level of detail turns a puzzling failure into an immediately actionable
one.

### What Makes a Failure Message Excellent

A checklist for evaluating failure message quality:

- [ ] Shows the name of the failing test
- [ ] Shows the file and line number of the failing assertion
- [ ] Shows expected value with label "expected:" (not "want:" or "right:")
- [ ] Shows actual value with label "got:" or "actual:"
- [ ] For long strings, shows a diff rather than raw values
- [ ] For collection types, shows which element mismatches
- [ ] Distinguishes failure (assertion false) from error (unexpected exception)
- [ ] Reports the exception type and message when an unexpected exception occurs
- [ ] Reports the source location of the unexpected exception, not just the
  test file line

---

## Practical Recommendations for Lingo's Test Framework

### Phase 1: Before the Language Can Test Itself

Use the inline annotation pattern. Create a host-language test runner that:

1. Reads `.ln` source files
2. Extracts `// expect: <value>` and `// expect error: <message>` annotations
3. Runs each file through the Lingo interpreter
4. Compares stdout line-by-line against extracted expectations

This requires no changes to the language and can be implemented in Go or
whatever language Lingo is implemented in.

### Phase 2: Language-Level Assertions

Once Lingo supports functions and basic I/O, add:

- `assert(condition)` — exits with a non-zero status and prints failure info
- `assert_equal(expected, actual)` — same, with structured output
- `assert_error(fn)` — verifies fn raises an error

These can be standard library functions or built-ins.

### Phase 3: Native Test Framework

Design the test framework API around Lingo's own idioms. Questions to answer:

- Does Lingo have first-class functions? Use closures for test bodies.
- Does Lingo have a macro or annotation system? Use it to capture source
  location automatically.
- Does Lingo have a native error/exception type? Use it as the test failure
  mechanism.
- What are Lingo's naming conventions? Mirror them in the test API.

Consider what ExUnit enforces: no nested describe blocks. Small constraints
that guide toward maintainable test structure pay dividends over time.

### Phase 4: Self-Testing the Framework

Write a test suite that specifically tests the test framework primitives.
Verify failure messages by capturing their string output and asserting on
it. Test expected-failure handling explicitly.

### API Design Recommendations

**Prefer `assert_equal(expected, actual)` over `assert(a == b)`** for two
reasons: the error message can label which value is expected, and the
framework can apply type-specific comparison (e.g., approximate equality for
floats, line-by-line diff for strings).

**Support skip/pending from the start.** A test framework without `skip`
encourages developers to delete tests they cannot make pass yet, which loses
the test specification permanently.

**Use consistent error format.** Pick a format and apply it everywhere:

```text
FAIL test_name (file.ln:42)
  expected: 3
       got: 4
```

**Print a summary line.** At minimum: `42 passed, 1 failed, 2 skipped`. This
is the first thing a developer reads when CI fails.

**Fail clearly on unexpected exceptions.** A test that throws an unexpected
error should be reported as an ERROR (not a failure), with the exception type,
message, and stack trace. Conflating errors with failures obscures what went
wrong.

---

## Sources

- [busted: Elegant Lua unit testing](https://lunarmodules.github.io/busted/)
- [luaunit: xUnit-style Lua testing](https://github.com/bluebird75/luaunit)
- [ExUnit documentation](https://hexdocs.pm/ex_unit/ExUnit.html)
- [ExUnit in practice — Elixir School](https://elixirschool.com/en/lessons/testing/basics)
- [The (not so) Magic Tricks of Testing in Elixir](https://medium.com/onfido-tech/the-not-so-magic-tricks-of-testing-in-elixir-1-2-89bfcf252321)
- [Go Wiki: TableDrivenTests](https://go.dev/wiki/TableDrivenTests)
- [Prefer table-driven tests — Dave Cheney](https://dave.cheney.net/2019/05/07/prefer-table-driven-tests)
- [Testing in Go: philosophy and tools — LWN](https://lwn.net/Articles/821358/)
- [Zig testing documentation](https://zig.guide/getting-started/running-tests/)
- [Some thoughts on Zig testing — Nathan Craddock](https://nathancraddock.com/blog/thoughts-on-zig-test/)
- [Zig std.testing source](https://github.com/ziglang/zig/blob/master/lib/std/testing.zig)
- [Julia Test.jl documentation](https://docs.julialang.org/en/v1/stdlib/Test/)
- [Kotlin Power-Assert compiler plugin](https://kotlinlang.org/docs/power-assert.html)
- [power-assert-js: introspective assertions](https://github.com/power-assert-js/power-assert)
- [The joy of expect tests — Jane Street](https://blog.janestreet.com/the-joy-of-expect-tests/)
- [wren-test: testing framework for Wren](https://github.com/gsmaverick/wren-test)
- [Crafting Interpreters repository](https://github.com/munificent/craftinginterpreters)
- [Crafting "Crafting Interpreters" — stuffwithstuff.com](https://journal.stuffwithstuff.com/2020/04/05/crafting-crafting-interpreters/)
- [A Lox interpreter implemented in Lox — benhoyt.com](https://benhoyt.com/writings/loxlox/)
- [MinUnit: minimal unit testing framework for C](https://jera.com/techinfo/jtns/jtn002)
- [tcltest documentation](https://www.tcl-lang.org/man/tcl8.3/TclCmd/tcltest.htm)
- [Lua unit testing overview — lua-users wiki](http://lua-users.org/wiki/UnitTesting)
- [Good error messages — Cypress](https://www.cypress.io/blog/good-error-messages)
