# REPL Implementation -- Codebase Research

## 1. Language Architecture

Lingo is a tree-walking interpreted language written in Rust (edition 2024). The codebase is
compact and follows a classic interpreter pipeline:

```text
Source string -> Lexer -> Tokens -> Parser -> AST (Program) -> Interpreter -> Value/Side-effects
```

### Module structure

| File | Role | Lines |
| --- | --- | --- |
| `src/main.rs` | CLI entry point | ~78 |
| `src/lib.rs` | Public module declarations | ~7 |
| `src/lexer.rs` | Tokenizer | ~622 |
| `src/parser.rs` | Recursive-descent parser | ~897 |
| `src/ast.rs` | AST node types | ~152 |
| `src/interpreter.rs` | Tree-walking evaluator | ~1437 |
| `src/test_runner.rs` | Test framework (`--test` mode) | ~117 |
| `Cargo.toml` | Project config | ~9 |

There is also a `tests/test_runner.rs` integration test file (~398 lines) and three example
programs in `examples/`.

### Dependencies

- **Runtime dependencies:** None (zero external crates).
- **Dev dependencies:** `tempfile = "3"` (for integration tests writing temp `.ln` files).

---

## 2. Data Types and Objects

### Internal value representation (`interpreter.rs`)

Values at runtime are represented by `Value` enum:

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Fn { name: String, params: Vec<Param>, body: Block, closure: Env },
    Lambda { params: Vec<Param>, body: Box<Expr>, closure: Env },
    BuiltinFn(String),
}
```

Key behaviors:

- `Value` implements `Display` for user-facing output (e.g., `println`).
- `Value` implements `Debug` and `Clone`.
- `Value::is_truthy()` defines truthiness: `false`, `0`, `""`, `Unit`, and empty lists are
  falsy; everything else is truthy.
- There is no `PartialEq` impl on `Value`; equality checking uses `Interpreter::values_equal()`
  which does structural comparison.

### AST types (`ast.rs`)

- `Program` contains `Vec<Item>` where `Item` is either `FnDecl` or `ExprStmt`.
- `Span { line, col }` tracks source location on AST nodes.
- Expressions (`Expr`) cover: literals, identifiers, binary/unary ops, function calls,
  pipeline (`|>`), indexing, field access, ranges, tuples, lists, lambdas, string interpolation,
  if/else, match, blocks, assignment, and compound assignment.
- Statements (`Stmt`): `Let`, `Expr`, `For`, `While`, `Return`, `Break`.
- Patterns (`Pattern`): `Wildcard`, `Ident`, `Literal`, `Tuple`, `Constructor`, `List`, `Or`.

### Literal types (`ast.rs`)

```rust
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}
```

---

## 3. Entry Points

### Current CLI (`src/main.rs`)

The binary accepts: `lingo [--test] <file.ln>`

Two modes:

1. **Normal mode** (`run`): Lex -> Parse -> Interpret. If a `main()` function exists, it is
   called. Otherwise, top-level expression statements are evaluated sequentially.
2. **Test mode** (`run_tests`): Lex -> Parse -> Load declarations (without running main) ->
   Discover `test_*` functions -> Run each in isolation.

If no filename is provided, the binary prints a usage message and exits with code 1.

**There is no REPL mode today.** The TODO.md explicitly lists `[ ] REPL` as a pending item.

### Pipeline functions in `main.rs`

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

This is the full execution pipeline. For a REPL, the key difference is:

- The `Interpreter` must persist across inputs (to retain variable bindings).
- Each line/input should be independently lexed and parsed.
- The result of expression evaluation should be printed (not just side-effects).

### Library structure (`src/lib.rs`)

All modules are re-exported publicly:

```rust
pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod test_runner;
```

This means the REPL can be built using the library crate or added to the existing binary.

---

## 4. Parser

### Parser interface

```rust
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self;
    pub fn parse_program(&mut self) -> Result<Program, String>;
}
```

- **Input:** `Vec<Token>` (from lexer).
- **Output:** `Result<Program, String>` where `Program { items: Vec<Item> }`.
- **Error type:** `String` (no structured error type).

### Grammar structure

The parser is a recursive-descent parser with the following precedence (lowest to highest):

1. Pipeline (`|>`)
2. Or (`||`)
3. And (`&&`)
4. Equality (`==`, `!=`)
5. Comparison (`<`, `>`, `<=`, `>=`)
6. Concat (`++`)
7. Range (`..`, `..=`)
8. Addition (`+`, `-`)
9. Multiplication (`*`, `/`, `%`)
10. Unary (`-`, `!`)
11. Postfix (function call, index, field access)
12. Primary (literals, identifiers, parenthesized expressions, if, match, blocks)

### REPL-relevant parsing behavior

- `parse_program()` parses a sequence of top-level `Item`s (fn declarations or expression
  statements).
- Expression statements require a terminator (newline, EOF, or `}`).
- The parser handles `Newline` tokens as statement terminators (Go-style semicolon insertion).
- String interpolation expressions are re-lexed and re-parsed internally by the parser
  (creates sub-`Lexer` and sub-`Parser`).

### For the REPL

The parser can already handle single expressions as top-level items (`Item::ExprStmt`).
A REPL input like `1 + 2` will parse as `Item::ExprStmt(Expr::Binary(...))`.
Function declarations like `fn foo() { ... }` will parse as `Item::FnDecl(...)`.

No changes to the parser should be needed for basic REPL support.

---

## 5. Evaluator (Interpreter)

### Evaluator interface

```rust
pub struct Interpreter {
    pub(crate) env: Env,
}

impl Interpreter {
    pub fn new() -> Self;                                    // Creates env with builtins
    pub fn run(&mut self, program: &Program) -> Result<(), String>;  // Full program execution
    pub fn load_declarations(&mut self, program: &Program);  // Load fn decls only
    pub(crate) fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, String>;
}
```

### `run()` behavior

1. Calls `load_declarations()` to register all `FnDecl`s in the environment.
2. If `main()` exists, calls it.
3. Otherwise, evaluates top-level `ExprStmt`s sequentially.
4. Returns `Result<(), String>` -- discards the expression value.

### Key for REPL: `run()` returns `()`, not the evaluated value

For a REPL, we need to:

- Evaluate expressions and get back the resulting `Value`.
- Print non-`Unit` values after evaluation.
- Accumulate function declarations across inputs.

The interpreter already has:

- `eval_expr(&mut self, expr: &Expr) -> Result<Value, String>` (private)
- `eval_block(&mut self, block: &Block) -> Result<Value, String>` (private)
- `load_declarations(&mut self, program: &Program)` (public)

For the REPL, we will likely need to either:

1. Make `eval_expr` public, or
2. Add a new public method like `eval_item` or `eval_line` that processes a single `Item`
   and returns the result value.

### Environment (`Env`)

```rust
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}
```

- Scoping is lexical via parent chain (tree of `Env`s, cloned on scope entry).
- `Env::new()` creates root with no parent.
- `Env::child(parent)` creates a child scope by cloning the parent.
- `set()` inserts a binding in the current scope.
- `get()` walks up the parent chain.
- `update()` finds and updates an existing binding in the chain (for mutation).
- `binding_names()` returns keys from current scope only (used by test discovery).

The `Interpreter` field `env` is `pub(crate)`, so it is accessible within the crate but not
from external code. For the REPL (added within the crate), this is fine.

### Built-in functions

40+ builtins registered in `Interpreter::new()`:
`println`, `print`, `to_str`, `to_int`, `to_float`, `len`, `push`, `map`, `filter`, `fold`,
`range`, `split`, `join`, `trim`, `contains`, `sort`, `sort_by`, `rev`, `enumerate`, `zip`,
`flat_map`, `any`, `all`, `find`, `unique`, `chunk`, `take`, `skip`, `min`, `max`, `abs`,
`dbg`, `assert`, `assert_eq`, `assert_ne`, `assert_true`, `assert_false`, `type_of`,
`read_file`, `write_file`, `read_line`, `parse_json`, `group_by`, `flatten`, `reduce`,
`replace`, `starts_with`, `ends_with`, `to_upper`, `to_lower`.

### Control flow signals

Return and break use an internal `Signal` enum:

```rust
enum Signal {
    Return(Value),
    Break,
}
```

These are propagated via `Result<Value, Result<Signal, String>>` in `eval_stmt_with_signals`.
At the `eval_stmt` boundary, unhandled signals are converted to error strings prefixed with
`__return__` or `__break__`.

---

## 6. Error Handling

### Error representation

All errors throughout the pipeline are plain `String`s:

- **Lexer errors:** `Result<Vec<Token>, String>` -- format: `"Unexpected character 'X' at L:C"`
- **Parser errors:** `Result<Program, String>` -- format: `"Expected X, got Y at L:C"`
- **Interpreter errors:** `Result<Value, String>` or `Result<(), String>` -- various messages.

There are **no custom error types** (no enums, no structs). Everything is `String`.

### Error propagation in `main.rs`

Errors from any stage are printed to stderr and cause `process::exit(1)`:

```rust
if let Err(e) = run(&source) {
    eprintln!("Error: {}", e);
    process::exit(1);
}
```

### REPL implications

For the REPL:

- Errors should be printed but should NOT exit the process.
- The REPL loop should continue after errors.
- Since errors are just `String`s, they can be displayed directly.

---

## 7. Test Conventions

### Framework

- **Rust's built-in `#[test]` framework** via `cargo test`.
- **Integration tests** in `tests/test_runner.rs`.
- No unit tests within `src/` files (no `#[cfg(test)]` modules).

### Integration test structure

Tests use two helper patterns:

```rust
// Helper: run a complete Lingo program
fn run_lingo(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)
}

// Helper: load declarations without executing
fn load_declarations(source: &str) -> Interpreter {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.load_declarations(&program);
    interpreter
}
```

### Test naming

Tests use descriptive snake_case names organized by feature sections:

- `assert_eq_equal_integers`
- `discovery_finds_test_functions`
- `isolation_fail_does_not_prevent_pass`
- `reporter_all_pass_output`
- `cli_test_flag_all_pass_exits_0`

### Assertion style

Tests use standard Rust assertions with descriptive messages:

```rust
assert!(result.is_ok(), "assert_eq(1, 1) should pass: {:?}", result);
assert!(result.is_err(), "assert_eq(1, 2) should fail");
assert!(err.contains("expected"), "error should contain 'expected': {}", err);
```

### CLI integration tests

CLI tests use `std::process::Command` to invoke the compiled binary, check exit codes, and
verify stdout/stderr content. They use `tempfile` to create temporary `.ln` files and
`env!("CARGO_BIN_EXE_lingo")` to find the binary path.

---

## 8. Code Style

### Language and edition

- **Rust 2024 edition** (latest).
- Zero runtime dependencies.

### Naming conventions

- Module names: lowercase (`lexer`, `parser`, `interpreter`, `ast`, `test_runner`).
- Types: PascalCase (`Lexer`, `Parser`, `Interpreter`, `Value`, `TokenKind`).
- Functions/methods: snake_case (`parse_program`, `eval_expr`, `call_builtin`).
- Constants: none used (builtins are registered as a Vec of string literals).

### Module organization

- One file per module (no directory modules).
- Public API is minimal: constructors, `run()`, `parse_program()`, `tokenize()`.
- Internal methods are private by default.
- `pub(crate)` used for inter-module access (`env` field, `call_function` method).

### Error handling style

- All fallible operations return `Result<T, String>`.
- No `?` operator with custom error types; errors are constructed with `format!()`.
- Early returns with `Err(...)` on failure.

### Comments

- Doc comments (`///`) on modules and key public functions.
- Section comments (`// --`) to divide logical areas.
- `#` for line comments in Lingo source (not Rust).

### Formatting

- Standard `rustfmt` formatting.
- 4-space indentation.
- Trailing commas in match arms and enum variants.

---

## 9. Summary: What the REPL Needs

### New code needed

1. **CLI flag parsing** in `main.rs`: detect when no file is given (or a `--repl` flag) and
   enter REPL mode instead of printing usage and exiting.
2. **REPL loop** (`src/repl.rs` or inline in `main.rs`):
   - Read a line from stdin (with a prompt like `">>"`)
   - Lex and parse the input.
   - Evaluate using a persistent `Interpreter` instance.
   - Print the resulting `Value` if it is not `Unit`.
   - Print errors and continue on failure.
   - Handle EOF (Ctrl-D) to exit cleanly.
3. **Public evaluation method** on `Interpreter`: a method that processes a `Program` (or
   individual `Item`) and returns the result `Value` instead of discarding it.

### Existing code that can be reused as-is

- `Lexer::new()` and `tokenize()` -- works on arbitrary source strings.
- `Parser::new()` and `parse_program()` -- handles single expressions as top-level items.
- `Interpreter::new()` -- creates a ready-to-use interpreter with builtins.
- `Interpreter::load_declarations()` -- registers functions from parsed programs.
- `Value::Display` -- formats values for output.

### Existing code that needs minor changes

- `Interpreter` needs a public method to evaluate items and return `Value` (currently
  `eval_expr` is private and `run()` returns `()`).
- `main.rs` needs a new code path for the REPL (currently requires a filename argument).

### No changes needed to

- `lexer.rs` -- works on any source string.
- `parser.rs` -- already handles expression statements as top-level items.
- `ast.rs` -- no structural changes.
- `test_runner.rs` -- independent feature.

### Multi-line input considerations

The parser already handles blocks and function declarations. For multi-line REPL input:

- Simple approach: try to parse each line; if parsing fails with an "unexpected EOF" error,
  accumulate more lines before retrying.
- The lexer's newline-as-terminator behavior (Go-style) means most single-line expressions
  will parse without issues.
- Function declarations span multiple lines and will need multi-line accumulation.
