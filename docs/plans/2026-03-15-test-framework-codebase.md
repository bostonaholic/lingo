# Lingo Codebase Analysis for Test Framework Design

**Date:** 2026-03-15
**Purpose:** Deep analysis of the Lingo language implementation to inform the design of a minimal,
native-feeling unit test framework.

---

## 1. Language Syntax and Semantics

### What Lingo Code Looks Like

Lingo is a general-purpose language with syntax drawn from Rust, Haskell, Elixir, and JavaScript.
It uses curly-brace blocks, newline-as-terminator (Go-style semicolon insertion), and implicit
returns. The file extension is `.ln`.

A representative Lingo program:

```lingo
fn double(x: Int) -> Int {
  x * 2
}

fn main() {
  let nums = [1, 2, 3, 4, 5]
  let doubled = nums |> map(n => n * 2)
  println("doubled: {doubled}")

  let total = nums |> fold(0, (acc, n) => acc + n)
  println("total: {total}")
}
```

### Key Syntactic Features (Implemented)

- **`fn` keyword** for function declarations
- **`let` bindings** (with optional `mut` for mutability)
- **Implicit returns** -- the last expression in a block is its value
- **`return` for early exit only**
- **Newlines as statement terminators** -- semicolons are optional, equivalent to newlines
- **`#` for line comments**
- **String interpolation** -- `"Hello, {name}!"` with arbitrary expressions inside braces
- **Pipeline operator** -- `x |> f(a, b)` desugars to `f(x, a, b)`
- **Lambda expressions** -- `n => n * 2` (single param), `(a, b) => a + b` (multi param)
- **`if`/`else` as expressions** -- `let x = if cond { "a" } else { "b" }`
- **`match` with pattern matching** -- supports literals, tuples, wildcards, identifiers, or-patterns
- **`for`/`in` loops** with ranges (`1..=100`, `1..10`)
- **`while` loops** and **`break`**
- **Lists** -- `[1, 2, 3]`
- **Tuples** -- `(1, "hello", true)`
- **Ranges** -- `1..10` (exclusive), `1..=10` (inclusive)
- **Concatenation operator** -- `++` for lists and strings
- **Compound assignment** -- `+=`, `-=`
- **Index access** -- `list[0]`, `str[0]`
- **Field access** -- `tuple.0`

### Data Types (Runtime Values)

The interpreter supports these runtime value types (from `src/interpreter.rs`):

| Type | Lingo Representation | Internal (`Value` enum) |
| --- | --- | --- |
| Integer | `42`, `-7` | `Value::Int(i64)` |
| Float | `3.14` | `Value::Float(f64)` |
| String | `"hello"` | `Value::Str(String)` |
| Boolean | `true`, `false` | `Value::Bool(bool)` |
| Unit | `()` | `Value::Unit` |
| Tuple | `(1, "a")` | `Value::Tuple(Vec<Value>)` |
| List | `[1, 2, 3]` | `Value::List(Vec<Value>)` |
| Function | `fn foo(x) { ... }` | `Value::Fn { name, params, body, closure }` |
| Lambda | `x => x + 1` | `Value::Lambda { params, body, closure }` |
| Builtin | `println`, `map`, etc. | `Value::BuiltinFn(String)` |

### Truthiness

Values have truthiness rules (used by `if`, `while`, `filter`, etc.):

- `Bool(false)` => falsy
- `Int(0)` => falsy
- `Str("")` (empty string) => falsy
- `Unit` => falsy
- `List([])` (empty list) => falsy
- Everything else => truthy

---

## 2. Type System

### Current Implementation: Dynamically Typed

Despite the specification targeting Hindley-Milner type inference, the current implementation is
**dynamically typed**. Type annotations in function signatures are parsed (stored as `Option<String>`
in the AST) but **not enforced** at runtime. All type checking happens at the point of operation
-- binary operations check their operand types at evaluation time and return errors for mismatched
types.

For example, the parser accepts `fn add(a: Int, b: Int) -> Int { ... }` but the interpreter
ignores the `: Int` and `-> Int` annotations entirely.

### Specification Target

The specification describes a full Hindley-Milner type system with:

- Built-in types: `Int`, `Float`, `Bool`, `Str`, `Char`, `[T]` (List), `{K: V}` (Map),
  `{T}` (Set), `(A, B)` (Tuple), `()` (Unit)
- `Option[T]` and `Result[T, E]` as built-in sum types
- Algebraic data types (enums with variants)
- Struct types
- Generics with `[T]` syntax (not `<T>`)
- Traits

**Implication for test framework:** Since the runtime is dynamically typed, a test framework must
work with runtime value comparison, not compile-time type checks. The `type_of` builtin returns
type names as strings, which could be useful for type-related assertions.

---

## 3. Module/File System

### Current Implementation: Single-File Only

The current interpreter reads a single `.ln` file, parses it into a `Program`, and executes it.
There is **no module system, no imports, no multi-file support** in the implementation.

The execution model:

1. Read a `.ln` file from the command line argument
2. Tokenize the source (`Lexer::tokenize`)
3. Parse into a `Program` AST (`Parser::parse_program`)
4. Run the program (`Interpreter::run`)

`Interpreter::run` first collects all function declarations, then looks for a `main()` function
to call. If no `main()` exists, top-level expressions are executed in order.

### Module System Specification Target

The spec describes a file-to-module mapping where each `.ln` file is a module, with directory
structure mirroring module hierarchy, Rust-style `use` imports, `pub` visibility, and a rich
built-in prelude.

**Implication for test framework:** Since there is no module system, a test framework must either:
(a) work within a single file, or (b) be built into the interpreter as a special mode.

---

## 4. Error Handling

### Current Implementation: String-Based Errors

Errors propagate as `Result<T, String>` throughout the Rust implementation. When a Lingo program
encounters an error, a `String` error message is returned up the call stack. The interpreter
exits with a non-zero status code when errors occur.

There is a `Signal` enum for control flow:

```rust
enum Signal {
    Return(Value),
    Break,
}
```

Return values are propagated by encoding them in error strings (`__return__` prefix) in some
code paths, though the main path uses `Signal::Return`.

### Lingo-Level Error Behavior

- **Division by zero** returns `Err("Division by zero")`
- **Undefined variable** returns `Err("Undefined variable: name")`
- **Type mismatch** returns `Err("Unsupported binary operation ... on ... and ...")`
- **Index out of bounds** returns specific error messages
- **`assert(value)`** returns `Err("Assertion failed")` when the value is falsy
- Pattern match failures return `Err("No matching arm in match expression for value: ...")`

### Error Handling Specification Target

The spec defines `Result[T, E]` and `Option[T]` types with a `?` operator for propagation and
`panic()` for unrecoverable errors. None of this is implemented yet. The current `assert` builtin
is the closest thing to error signaling available to Lingo programs.

**Implication for test framework:** The `assert` builtin already exists and provides the minimal
assertion primitive. A test framework can build on this. Errors terminate the program, so the
framework must catch assertion failures to continue running other tests.

---

## 5. Metaprogramming

### No Metaprogramming Capabilities

The current implementation has:

- **No macros**
- **No decorators or annotations**
- **No reflection** (except `type_of` which returns a type name string)
- **No `eval` or runtime code generation**
- **No first-class AST manipulation**

Functions and lambdas are first-class values (can be passed as arguments, stored in variables,
returned from functions), but there is no way to inspect or modify them programmatically.

The `type_of` builtin returns one of: `"Int"`, `"Float"`, `"Str"`, `"Bool"`, `"Unit"`, `"Tuple"`,
`"List"`, `"Fn"`.

**Implication for test framework:** A test framework cannot rely on decorators, macros, or
reflection for test discovery. Tests must be registered explicitly (via function calls or naming
conventions) or discovered by the interpreter at a lower level.

---

## 6. Naming Conventions

### Convention Summary

From the specification and example code:

| Convention | Usage | Examples |
| --- | --- | --- |
| `snake_case` | Functions, variables, modules | `load_config`, `total_cost`, `read_file` |
| `PascalCase` | Types, structs, enums, traits | `Config`, `Result`, `Option`, `Int`, `Str` |
| `SCREAMING_SNAKE_CASE` | Constants | (not yet used in examples) |

### Builtin Function Names

All 44 builtin functions use `snake_case`:

```text
println, print, to_str, to_int, to_float, len, push,
map, filter, fold, range, split, join, trim,
contains, sort, sort_by, rev, enumerate, zip,
flat_map, any, all, find, unique, chunk, take,
skip, min, max, abs, dbg, assert, type_of,
read_file, write_file, read_line, parse_json,
group_by, flatten, reduce, replace, starts_with,
ends_with, to_upper, to_lower
```

### Keyword Style

All 22 keywords are lowercase single words or abbreviated words:

```text
fn, let, mut, if, else, match, for, in, while, loop, break,
return, true, false, struct, enum, type, pub, use, mod, trait,
impl, async, await
```

**Implication for test framework:** All framework functions and keywords should use `snake_case`.
Any test-related keyword (if added) should be short and lowercase, consistent with existing
keywords.

---

## 7. Built-in Functions

### Complete Inventory (44 builtins)

**I/O (5):**

- `println(value)` -- print with newline, returns `Unit`
- `print(value)` -- print without newline, returns `Unit`
- `read_file(path: Str) -> Str` -- reads file contents
- `write_file(path: Str, content: Str) -> Unit` -- writes file
- `read_line() -> Str` -- reads a line from stdin

**Collections (20):**

- `len(collection) -> Int` -- length of list or string
- `push(list, item) -> List` -- returns new list with item appended (immutable)
- `map(list, fn) -> List` -- transform each element
- `filter(list, fn) -> List` -- keep elements matching predicate
- `fold(list, init, fn) -> T` -- reduce with initial value
- `reduce(list, fn) -> T` -- reduce without initial value
- `find(list, fn) -> T | Unit` -- first matching element or Unit
- `any(list, fn) -> Bool` -- true if any element matches
- `all(list, fn) -> Bool` -- true if all elements match
- `sort(list) -> List` -- sort naturally
- `sort_by(list, key_fn) -> List` -- sort by key function
- `rev(list) -> List` -- reverse
- `enumerate(list) -> List[(Int, T)]` -- add indices as tuples
- `zip(list1, list2) -> List[(A, B)]` -- zip into tuples
- `flat_map(list, fn) -> List` -- map then flatten
- `flatten(list) -> List` -- flatten nested lists
- `unique(list) -> List` -- remove duplicates
- `chunk(list, n: Int) -> List[List]` -- split into chunks
- `take(list, n: Int) -> List` -- first n elements
- `skip(list, n: Int) -> List` -- skip first n elements

**Strings (9):**

- `split(str, delim) -> List[Str]`
- `join(list, sep) -> Str`
- `trim(str) -> Str`
- `contains(str_or_list, item) -> Bool`
- `replace(str, from, to) -> Str`
- `starts_with(str, prefix) -> Bool`
- `ends_with(str, suffix) -> Bool`
- `to_upper(str) -> Str`
- `to_lower(str) -> Str`

**Math (3):**

- `min(a, b) -> number`
- `max(a, b) -> number`
- `abs(n) -> number`

**Conversion (3):**

- `to_str(value) -> Str`
- `to_int(value) -> Int`
- `to_float(value) -> Float`

**Grouping (1):**

- `group_by(list, fn) -> List[(key, List[T])]`

**Debug/Assertions (3):**

- `dbg(value) -> value` -- prints debug representation to stderr, returns value unchanged
- `assert(value) -> Unit` -- fails with "Assertion failed" if value is falsy
- `type_of(value) -> Str` -- returns type name as string

**Data (2):**

- `range(start, end) -> List[Int]` -- exclusive range (note: `1..10` syntax also works)
- `parse_json(str) -> Str` -- placeholder, currently just returns the string

### Key Observations for Test Framework

1. **`assert` exists** but is minimal -- only checks truthiness, provides no context about what
   failed or what was expected vs. actual.

2. **`dbg` exists** and could be useful for test output -- it prints to stderr and returns the
   value unchanged (pass-through).

3. **`type_of` exists** for type-checking assertions.

4. **`to_str` exists** for converting values to string representations for comparison and display.

5. **No `assert_eq`, `assert_ne`, or other comparison assertions** exist.

6. **No `panic` builtin** exists yet (spec defines it, not implemented).

---

## 8. Interpreter/Compiler Architecture

### Pipeline: Source -> Tokens -> AST -> Values

```text
.ln file
  |
  v
Lexer (src/lexer.rs)
  | tokenize() -> Vec<Token>
  v
Parser (src/parser.rs)
  | parse_program() -> Program
  v
Interpreter (src/interpreter.rs)
  | run(&Program) -> Result<(), String>
  v
Output / Side effects
```

### Lexer (`src/lexer.rs`, 622 lines)

- Character-by-character scanner
- Produces `Vec<Token>` where `Token = { kind: TokenKind, line: usize, col: usize }`
- 86 token kinds (keywords, literals, operators, delimiters, special)
- Go-style newline insertion: suppresses newlines after continuation tokens (`|>`, `=>`, `(`,
  `[`, `,`, binary operators, etc.)
- String interpolation encoded as a single `StringInterpStart` token with `\x01`/`\x02` markers
  for expression/literal parts
- Semicolons treated as equivalent to newlines
- `#` begins line comments

### Parser (`src/parser.rs`, 896 lines)

- Recursive descent parser
- Operator precedence (lowest to highest): pipeline `|>`, or `||`, and `&&`, equality `==`/`!=`,
  comparison `<`/`>`/`<=`/`>=`, concat `++`, range `..`/`..=`, addition `+`/`-`,
  multiplication `*`/`/`/`%`, unary `-`/`!`, postfix (calls, indexing, field access)
- `Program` contains `Vec<Item>` where `Item` is either `FnDecl` or `ExprStmt`
- Blocks have a trailing expression (implicit return value) extracted from the last statement
- Lambda parsing uses backtracking (saves parser position, tries lambda form, falls back)
- Pattern matching supports: `_` (wildcard), identifiers, literals (including negative ints),
  tuples `(a, b)`, lists `[a, b]`, or-patterns `a | b`

### AST (`src/ast.rs`, 151 lines)

Key node types:

- `Program { items: Vec<Item> }`
- `Item::FnDecl(FnDecl)` | `Item::ExprStmt(Expr)`
- `FnDecl { name, params, return_type, body, span }`
- `Block { stmts: Vec<Stmt>, expr: Option<Box<Expr>> }` -- trailing expr is implicit return
- `Stmt` variants: `Let`, `Expr`, `For`, `While`, `Return`, `Break`
- `Expr` variants: `Literal`, `Ident`, `Binary`, `Unary`, `Call`, `Pipeline`, `Index`, `Field`,
  `Range`, `Tuple`, `List`, `Lambda`, `StringInterp`, `If`, `Match`, `Block`, `Assign`,
  `CompoundAssign`
- `Pattern` variants: `Wildcard`, `Ident`, `Literal`, `Tuple`, `Constructor`, `List`, `Or`

### Interpreter (`src/interpreter.rs`, 1361 lines)

- Tree-walking evaluator
- Environment model: `Env` with `HashMap<String, Value>` and optional parent pointer (lexical scoping)
- `Env::child(parent)` creates a new scope; `Env::update` walks the chain for mutation
- All 44 builtins registered in `Interpreter::new()`
- `run()` method: first pass collects all `FnDecl` items, then calls `main()` or executes
  top-level expressions
- Control flow via `Signal` enum (`Return(Value)`, `Break`) threaded through
  `eval_stmt_with_signals`
- Pipeline desugaring: `x |> f(a, b)` becomes `f(x, a, b)` at evaluation time
- Pattern matching in `match`: tries each arm sequentially, binds variables in the matching arm's
  environment
- Closures capture the environment at definition time

### Entry Point (`src/main.rs`, 46 lines)

```rust
fn run(source: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut interpreter = interpreter::Interpreter::new();
    interpreter.run(&program)?;
    Ok(())
}
```

The `main()` function reads a filename from `argv[1]`, reads the file, and calls `run()`.
Errors are printed to stderr and the process exits with code 1.

---

## 9. Existing Test Infrastructure

### Rust-Level Tests

**There are zero tests.** No `#[cfg(test)]` modules, no `#[test]` functions, no `tests/`
directory. Running `cargo test` reports `0 passed; 0 failed`.

### Lingo-Level Tests

There is no test framework, test runner, or test infrastructure at the Lingo language level.
The `assert` builtin is the only testing primitive, and it provides no structured output.

### How the Language Is Currently Tested

The language is tested manually by running the three example files:

```bash
cargo run -- examples/fizzbuzz.ln   # Pattern matching, for/in, ranges
cargo run -- examples/hello.ln      # String interpolation, basic let
cargo run -- examples/basics.ln     # Arithmetic, if/else, lists, pipelines, lambdas
```

All three examples run successfully.

---

## 10. Example Programs

### `examples/hello.ln` -- Minimal Program

```lingo
fn main() {
  let name = "World"
  println("Hello, {name}!")
}
```

Shows: `fn main()`, `let` binding, string interpolation, `println`.

### `examples/fizzbuzz.ln` -- Pattern Matching with Ranges

```lingo
fn main() {
  for i in 1..=100 {
    let out = match (i % 3, i % 5) {
      (0, 0) => "FizzBuzz"
      (0, _) => "Fizz"
      (_, 0) => "Buzz"
      _ => to_str(i)
    }
    println(out)
  }
}
```

Shows: `for`/`in` with inclusive range, `match` on tuples with literal and wildcard patterns,
`match` as expression (assigned to `let`), `to_str` builtin.

### `examples/basics.ln` -- Feature Showcase

```lingo
fn double(x: Int) -> Int {
  x * 2
}

fn main() {
  # Arithmetic
  let x = 10
  let y = 3
  println("x + y = {x + y}")
  println("x % y = {x % y}")

  # If/else as expression
  let status = if x > 5 { "big" } else { "small" }
  println("x is {status}")

  # Lists and pipeline
  let nums = [1, 2, 3, 4, 5]
  let doubled = nums |> map(n => n * 2)
  println("doubled: {doubled}")

  let evens = nums |> filter(n => n % 2 == 0)
  println("evens: {evens}")

  let total = nums |> fold(0, (acc, n) => acc + n)
  println("total: {total}")

  # Function calls
  println("double(21) = {double(21)}")

  # String operations
  let greeting = "Hello, World!"
  println("length: {len(greeting)}")
  println("upper: {to_upper(greeting)}")

  # Pattern matching
  let items = [1, 2, 3]
  let desc = match len(items) {
    0 => "empty"
    1 => "singleton"
    _ => "multiple"
  }
  println("items: {desc}")
}
```

Shows: typed function parameters and return types, arithmetic, string interpolation with
expressions, `if`/`else` as expression, pipeline with `map`/`filter`/`fold`, lambda expressions
(single and multi-param), builtin function calls, `match` as expression, `#` comments.

---

## 11. Design Constraints for a Test Framework

Based on this analysis, a test framework for Lingo must work within these constraints:

### Hard Constraints (Current Implementation)

1. **Single-file execution** -- no module system, so tests either live alongside code or in
   separate `.ln` files run independently.
2. **No metaprogramming** -- no macros, decorators, or reflection for test discovery.
3. **Errors terminate execution** -- an `assert` failure (or any error) halts the program. The
   framework must catch failures to continue running other tests.
4. **No exception handling** -- no try/catch, so the framework cannot catch assertion failures
   at the Lingo level. Failure isolation must happen at the interpreter level (Rust side) or
   by running test functions in isolated calls.
5. **Functions are first-class** -- test functions can be passed as values, stored in lists, and
   called dynamically. This enables a registration-based approach.
6. **Builtins are the only extensibility point** -- new test assertions must either be added as
   builtins (Rust-side) or composed from existing builtins (Lingo-side).

### Soft Constraints (Language Idioms)

1. **Naming convention** -- `snake_case` for functions and variables.
2. **Pipeline-friendly** -- where possible, test utilities should compose with `|>`.
3. **Implicit returns** -- test bodies are blocks that can return values.
4. **Expression orientation** -- `if`/`else` and `match` return values, which is useful for
   conditional test logic.
5. **Minimal ceremony** -- Lingo's design philosophy is to eliminate zero-information tokens.
   A test framework should follow this principle.

### Available Building Blocks

| Mechanism | How It Helps |
| --- | --- |
| `assert(value)` | Basic truthiness check (already exists) |
| `dbg(value)` | Debug output to stderr (pass-through) |
| `type_of(value)` | Type name as string for type assertions |
| `to_str(value)` | Value stringification for messages |
| First-class functions | Tests can be collected in lists and iterated |
| `match` expressions | Can pattern-match on test results |
| `for`/`in` loops | Can iterate over a list of test functions |
| String interpolation | Rich error messages with context |

### Gap Analysis: What's Missing for Testing

| Need | Status | Required Action |
| --- | --- | --- |
| `assert_eq(actual, expected)` | Missing | Add as builtin or Lingo function |
| `assert_ne(actual, expected)` | Missing | Add as builtin or Lingo function |
| Test discovery/registration | Missing | Add convention or builtin |
| Failure isolation (continue after failure) | Missing | Requires interpreter-level change |
| Test result summary (pass/fail counts) | Missing | Requires framework logic |
| Test runner entry point | Missing | Either a `--test` flag or a convention |
| Rich failure messages (expected vs actual) | Missing | `assert` only says "Assertion failed" |
| Test timing | Missing | Requires interpreter-level support |

---

## 12. Implementation Approach Considerations

### Approach A: Pure Lingo Library (No Interpreter Changes)

Write test utilities entirely in Lingo. Use first-class functions for test registration. Limitation:
cannot isolate failures (one assert failure kills the program). Would require wrapping each test
in a try-like construct that does not exist.

**Verdict:** Not feasible with current error handling.

### Approach B: New Builtins (Minimal Interpreter Changes)

Add a small set of test-related builtins to the interpreter:

- `assert_eq(actual, expected)` -- with rich error messages
- `assert_ne(actual, expected)`
- A test runner mechanism that catches failures per-test

This keeps the Lingo-side syntax clean while adding real functionality.

### Approach C: Test Runner Mode (Interpreter-Level Feature)

Add a `--test` flag or equivalent. The interpreter discovers functions matching a naming
convention (e.g., `test_*`), runs each in isolation, catches failures, and reports results.
This is how Go and Rust testing works.

### Approach D: Hybrid (Builtins + Runner)

Combine B and C: add assertion builtins for rich failure messages, and add an interpreter-level
test runner that discovers and runs test functions with failure isolation. This provides the best
developer experience and aligns with how Go (`go test`) and Rust (`cargo test`) work.

---

## 13. Relevant Source File Paths

| File | Lines | Purpose |
| --- | --- | --- |
| `/Users/matthew/code/bostonaholic/lingo/src/main.rs` | 46 | Entry point, `run()` function |
| `/Users/matthew/code/bostonaholic/lingo/src/lexer.rs` | 622 | Tokenizer |
| `/Users/matthew/code/bostonaholic/lingo/src/ast.rs` | 151 | AST node definitions |
| `/Users/matthew/code/bostonaholic/lingo/src/parser.rs` | 896 | Recursive descent parser |
| `/Users/matthew/code/bostonaholic/lingo/src/interpreter.rs` | 1361 | Tree-walking evaluator, builtins |
| `/Users/matthew/code/bostonaholic/lingo/examples/fizzbuzz.ln` | 11 | Pattern matching example |
| `/Users/matthew/code/bostonaholic/lingo/examples/hello.ln` | 4 | Minimal example |
| `/Users/matthew/code/bostonaholic/lingo/examples/basics.ln` | 43 | Feature showcase |
| `/Users/matthew/code/bostonaholic/lingo/SPECIFICATION.md` | 2348 | Full language specification |
| `/Users/matthew/code/bostonaholic/lingo/Cargo.toml` | 7 | Rust project config (no dependencies) |

---

## 14. What Idiomatic Lingo Tests Should Look Like

Based on the language's design principles (minimal ceremony, pipeline-friendly, expression-oriented,
familiar syntax), here is what a native-feeling test file might look like:

```lingo
# math_test.ln

fn add(a, b) { a + b }

fn test_add_positive() {
  assert_eq(add(2, 3), 5)
}

fn test_add_negative() {
  assert_eq(add(-1, -2), -3)
}

fn test_add_zero() {
  assert_eq(add(0, 0), 0)
}

fn test_list_pipeline() {
  let result = [1, 2, 3]
    |> map(n => n * 2)
    |> filter(n => n > 2)
  assert_eq(result, [4, 6])
}

fn test_string_operations() {
  let s = "Hello, World!"
  assert_eq(len(s), 13)
  assert_eq(to_upper(s), "HELLO, WORLD!")
  assert(starts_with(s, "Hello"))
}

fn test_pattern_matching() {
  let classify = n => match n % 2 {
    0 => "even"
    _ => "odd"
  }
  assert_eq(classify(4), "even")
  assert_eq(classify(7), "odd")
}
```

This follows the Go/Rust convention of `test_*` function names, uses `assert_eq` for value
comparisons, and keeps test bodies minimal with implicit returns. Each test function is
self-contained and reads like idiomatic Lingo code.
