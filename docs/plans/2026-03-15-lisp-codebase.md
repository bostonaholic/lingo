# Research: Lingo Codebase for Lisp Transformation (2026-03-15)

## Problem Statement

Transform Lingo from its current imperative/functional syntax (Rust/Go-inspired with curly
braces, keywords, infix operators, and Go-style semicolon insertion) into a Lisp with
S-expression syntax and evaluation/application semantics. This document maps every component of
the current codebase that must change, documenting the exact structures, dispatch mechanisms, and
integration points.

## Requirements

- Replace the lexer with one that tokenizes S-expression syntax (parens, atoms, strings,
  numbers)
- Replace the parser with one that produces a Lisp AST (nested lists/cons cells or a
  simplified Expr enum)
- Dramatically simplify the AST from 20+ node types to a handful
- Rewrite the interpreter around an evaluation/application model instead of the current
  tree-walk over a complex AST
- Update the REPL to work with the new pipeline
- Update the test runner (framework itself is syntax-agnostic, but tests need new syntax)
- Rewrite all example programs in Lisp syntax
- Rewrite all integration tests for Lisp syntax

## Findings

### Project Structure

```text
src/
  lib.rs           -- Module declarations (6 modules)
  main.rs          -- CLI entry point (3 modes: REPL, run file, run tests)
  lexer.rs         -- Tokenizer (623 lines)
  ast.rs           -- AST node types (153 lines)
  parser.rs        -- Recursive descent parser (901 lines)
  interpreter.rs   -- Tree-walking interpreter (1461 lines)
  repl.rs          -- Interactive REPL (100 lines)
  test_runner.rs   -- Test discovery and execution (117 lines)
tests/
  test_runner.rs   -- Integration tests (640 lines)
examples/
  hello.ln         -- Hello world (4 lines)
  basics.ln        -- Arithmetic, lists, pipelines (43 lines)
  fizzbuzz.ln      -- FizzBuzz (11 lines)
  test_framework_test.ln  -- Test framework demo (35 lines)
```

**Dependencies**: Zero runtime dependencies. Only `tempfile = "3"` as a dev-dependency.
Rust edition 2024.

---

### 1. Lexer (`src/lexer.rs`, 623 lines) -- WILL BE REPLACED

#### Token Types (86 variants in `TokenKind` enum)

**Keywords** (24):

| Token | Keyword |
| --- | --- |
| `Fn` | `fn` |
| `Let` | `let` |
| `Mut` | `mut` |
| `If` | `if` |
| `Else` | `else` |
| `Match` | `match` |
| `For` | `for` |
| `In` | `in` |
| `While` | `while` |
| `Loop` | `loop` |
| `Break` | `break` |
| `Return` | `return` |
| `True` | `true` |
| `False` | `false` |
| `Struct` | `struct` |
| `Enum` | `enum` |
| `Type` | `type` |
| `Pub` | `pub` |
| `Use` | `use` |
| `Mod` | `mod` |
| `Trait` | `trait` |
| `Impl` | `impl` |
| `Async` | `async` |
| `Await` | `await` |

**Literals** (5): `IntLit(i64)`, `FloatLit(f64)`, `StrLit(String)`,
`StringInterpStart(String)`, `StringInterpMid(String)`, `StringInterpEnd(String)`

**Identifiers** (1): `Ident(String)`

**Operators** (28): `Plus`, `Minus`, `Star`, `Slash`, `Percent`, `EqEq`, `BangEq`, `Lt`,
`Gt`, `Le`, `Ge`, `AndAnd`, `PipePipe`, `Bang`, `Eq`, `PlusEq`, `MinusEq`, `PipeGt` (`|>`),
`FatArrow` (`=>`), `ThinArrow` (`->`), `Dot`, `ColonColon`, `DotDot`, `DotDotEq`, `Question`,
`Colon`, `Pipe`, `PlusPlus` (`++`)

**Delimiters** (7): `LParen`, `RParen`, `LBrace`, `RBrace`, `LBracket`, `RBracket`, `Comma`,
`Underscore`

**Special** (2): `Newline`, `Eof`

#### Tokenization Mechanism

The `Lexer` struct holds `source: Vec<char>`, `pos`, `line`, `col`. Key methods:

- `tokenize()` -- Main entry point. Calls `next_token()` in a loop, then runs
  `insert_newlines()` for Go-style semicolon insertion.
- `next_token()` -- Dispatches based on current character: `#` for comments (line comments
  only), `\n` for newlines, `;` treated as newline, digits for numbers, `"` for strings,
  alphabetic/`_` for identifiers/keywords, then multi-char operators (checked in order), then
  single-char operators.
- `lex_number()` -- Supports integers, floats (digit.digit), underscores as separators.
  Distinguishes `..` from decimal point.
- `lex_string()` -- Supports escape sequences (`\n`, `\t`, `\\`, `\"`, `\{`). Supports
  **string interpolation** with `{expr}` syntax. Interpolations are encoded inline using
  `\x01` (expression marker) and `\x02` (literal marker) control characters packed into a
  single `StringInterpStart` token.
- `lex_ident()` -- Reads alphanumeric + `_` characters, then matches against keyword table.
- `insert_newlines()` -- Go-style semicolon insertion. Suppresses newlines after
  **continuation tokens** (operators, open delimiters, commas, pipe, etc.). Suppresses
  consecutive newlines and leading newlines.

#### Lexer Lisp Replacement Notes

The Lisp lexer needs only: `LParen`, `RParen`, `IntLit`, `FloatLit`, `StrLit`, `Symbol`
(replaces both `Ident` and keywords), `Bool`, `Quote` (optional), `Eof`. No semicolon
insertion needed. No multi-char operators. No string interpolation. Comments change from `#`
to `;`. This reduces from ~86 token variants to ~8-10.

---

### 2. AST (`src/ast.rs`, 153 lines) -- WILL BE DRAMATICALLY SIMPLIFIED

#### Node Types

**Top-level**:

```rust
pub struct Program { pub items: Vec<Item> }

pub enum Item {
    FnDecl(FnDecl),     // fn name(params) { body }
    ExprStmt(Expr),     // top-level expression
    Stmt(Stmt),         // top-level statement (let, for, etc.)
}
```

**Function declarations**:

```rust
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Block,
    pub span: Span,
}

pub struct Param {
    pub name: String,
    pub type_ann: Option<String>,
}

pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,  // trailing expression = block value
}
```

**Statements** (6 variants):

```rust
pub enum Stmt {
    Let(LetStmt),           // let [mut] pattern [: type] = expr
    Expr(Expr),             // expression statement
    For(ForStmt),           // for binding in iterable { body }
    While(WhileStmt),       // while condition { body }
    Return(Option<Expr>),   // return [expr]
    Break,                  // break
}
```

**Expressions** (16 variants):

```rust
pub enum Expr {
    Literal(Literal),                              // 42, 3.14, "hello", true
    Ident(String, Span),                           // variable reference
    Binary(Box<Expr>, BinOp, Box<Expr>, Span),     // a + b
    Unary(UnaryOp, Box<Expr>, Span),               // -x, !b
    Call(Box<Expr>, Vec<Expr>, Span),               // f(args)
    Pipeline(Box<Expr>, Box<Expr>, Span),           // x |> f
    Index(Box<Expr>, Box<Expr>, Span),              // a[i]
    Field(Box<Expr>, String, Span),                 // x.field
    Range(Box<Expr>, Box<Expr>, bool, Span),        // a..b, a..=b
    Tuple(Vec<Expr>, Span),                         // (a, b)
    List(Vec<Expr>, Span),                          // [a, b]
    Lambda(Vec<Param>, Box<Expr>, Span),            // x => body
    StringInterp(Vec<StringPart>, Span),            // "hello {name}!"
    If(Box<Expr>, Block, Option<Box<Expr>>, Span),  // if cond { } else { }
    Match(Box<Expr>, Vec<MatchArm>, Span),          // match val { pat => expr }
    Block(Block, Span),                             // { stmts; expr }
    Assign(Box<Expr>, Box<Expr>, Span),             // x = val
    CompoundAssign(Box<Expr>, BinOp, Box<Expr>, Span), // x += val
}
```

**Operators**:

```rust
pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Gt, Le, Ge, And, Or, Concat }
pub enum UnaryOp { Neg, Not }
```

**Patterns** (7 variants):

```rust
pub enum Pattern {
    Wildcard,                        // _
    Ident(String),                   // x
    Literal(Literal),                // 42, "hello"
    Tuple(Vec<Pattern>),             // (a, b)
    Constructor(String, Vec<Pattern>), // Some(x) -- not fully implemented
    List(Vec<Pattern>),              // [a, b]
    Or(Vec<Pattern>),                // a | b
}
```

**Supporting types**: `Span { line, col }`, `MatchArm { pattern, guard, body }`,
`StringPart { Lit(String) | Expr(Expr) }`, `Literal { Int | Float | Str | Bool }`

#### AST Lisp Replacement Notes

A Lisp AST needs approximately 4-5 node types:

```text
Expr = Atom(value) | List(Vec<Expr>) | Symbol(String) | Quote(Box<Expr>)
```

All of `Item`, `Stmt`, `FnDecl`, `Block`, `Pattern`, `BinOp`, `UnaryOp`, `MatchArm`,
`StringPart`, and most `Expr` variants collapse into this. The distinction between
items/statements/expressions disappears -- everything is an expression in a Lisp.

---

### 3. Parser (`src/parser.rs`, 901 lines) -- WILL BE REPLACED

#### Parser Architecture

Recursive descent parser. `Parser` struct holds `tokens: Vec<Token>` and `pos: usize`.

#### Precedence Hierarchy (top = lowest precedence)

1. `parse_pipeline()` -- `|>` (left-associative)
2. `parse_or()` -- `||`
3. `parse_and()` -- `&&`
4. `parse_equality()` -- `==`, `!=`
5. `parse_comparison()` -- `<`, `>`, `<=`, `>=`
6. `parse_concat()` -- `++`
7. `parse_range()` -- `..`, `..=`
8. `parse_addition()` -- `+`, `-`
9. `parse_multiplication()` -- `*`, `/`, `%`
10. `parse_unary()` -- `-`, `!` (prefix)
11. `parse_postfix()` -- `f()`, `a[i]`, `x.field`
12. `parse_primary()` -- literals, identifiers, `(`, `[`, `if`, `match`, `{`, `_`

#### Key Parsing Methods

- `parse_program()` -- Loops over `parse_item()` separated by newlines
- `parse_item()` -- Dispatches: `fn` -> FnDecl, `let` -> Stmt, else -> ExprStmt
- `parse_fn_decl()` -- `fn name(params) [-> type] { block }`
- `parse_block()` -- `{ stmts... [trailing_expr] }`. Last expression becomes block value.
- `parse_stmt()` -- `let`, `for`, `while`, `return`, `break`, or expression (with
  assignment check)
- `parse_lambda_or_expr()` -- Tries `ident => body` or `(params) => body`, backtracks
  if not lambda
- `parse_string_interp()` -- Decodes the `\x01`/`\x02` encoded string from lexer, re-lexes
  and re-parses embedded expression strings
- `parse_match_expr()` -- `match expr { pattern => body ... }`
- `parse_pattern()` -- Handles `_`, literals, identifiers, tuples, lists, or-patterns

#### Parser Utility Methods

- `check(&TokenKind)` -- Peek without consuming (uses discriminant comparison)
- `advance()` -- Consume and return current token
- `expect(&TokenKind)` -- Consume or error
- `expect_ident()` -- Consume identifier or error
- `skip_newlines()` -- Skip all consecutive newline tokens
- `expect_terminator()` -- Expect newline, EOF, or `}` (Go-style)
- `is_at_end()` -- Check for EOF

#### Parser Lisp Replacement Notes

The Lisp parser is dramatically simpler. It needs only:

- `parse()` -> `Vec<Expr>` -- read all top-level expressions
- `read_expr()` -- dispatch on `(` for lists, `'` for quote, else atom
- `read_list()` -- read expressions until `)` encountered
- `read_atom()` -- parse numbers, strings, booleans, symbols

All precedence handling, operator parsing, lambda detection, pattern parsing, block parsing,
semicolon handling, and backtracking disappear entirely. The parser shrinks from ~900 lines to
~100 lines.

---

### 4. Interpreter (`src/interpreter.rs`, 1461 lines) -- MAJOR REWRITE

#### Value Enum (10 variants)

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

Key methods on `Value`:

- `is_truthy()` -- Bool: value, Int: non-zero, Str: non-empty, Unit: false, List: non-empty,
  else true
- `Display` impl -- Pretty-prints each variant

#### Environment (`Env`)

Linked-list scoping model:

```rust
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}
```

Methods: `new()`, `child(parent)`, `set(name, value)`, `get(name)` (walks parent chain),
`update(name, value)` (mutable update, walks parent chain), `binding_names()` (current scope
only).

**Critical pattern**: The interpreter saves/restores `self.env` manually before/after function
calls and blocks. Uses `Env::child()` to create lexical scopes. This is a clone-heavy approach
(every scope creation clones the parent).

#### Control Flow

Uses a `Signal` enum for non-local control flow:

```rust
enum Signal { Return(Value), Break }
```

`eval_stmt_with_signals()` returns `Result<Value, Result<Signal, String>>` -- the nested
Result distinguishes signals from errors. Functions catch `Signal::Return`, loops catch
`Signal::Break`.

#### Expression Handling Method (lines 355-535)

Dispatches on all 16 `Expr` variants:

| Expr Variant | Behavior |
| --- | --- |
| `Literal` | Convert to Value |
| `Ident` | Env lookup |
| `Binary` | Handle both sides, dispatch to binary-op handler |
| `Unary` | Handle operand, dispatch to unary-op handler |
| `Call` | Handle callee + args, call function handler |
| `Pipeline` | Handle left, desugar `x \|> f(a,b)` to `f(x,a,b)` |
| `Index` | List/String indexing |
| `Field` | Tuple field access (`.0`, `.1`) |
| `Range` | Generate list of integers |
| `Tuple` | Handle elements |
| `List` | Handle elements |
| `Lambda` | Capture closure |
| `StringInterp` | Handle parts, concatenate |
| `If` | Handle condition, branch |
| `Match` | Handle scrutinee, try patterns |
| `Block` | Create child scope, handle stmts + trailing expr |
| `Assign` | Handle value, update binding |
| `CompoundAssign` | Handle, apply binop, update binding |

#### Function Calling Method (lines 645-709)

Dispatches on three callable types:

1. **BuiltinFn(name)** -- Calls builtin dispatch method
2. **Fn { params, body, closure }** -- Creates child scope of closure, binds params, copies
   top-level functions into scope, runs body block, catches Return signals
3. **Lambda { params, body, closure }** -- Same as Fn but runs body as expression
   (not block)

Both Fn and Lambda copy all top-level `Fn` and `BuiltinFn` values into the new scope to ensure
they are accessible (lines 663-669, 695-701).

#### Binary Operator Handler (lines 546-634)

Handles all binary operations via pattern matching on `(op, left_type, right_type)`:

- **Integer arithmetic**: `+`, `-`, `*`, `/`, `%` on `Int`
- **Float arithmetic**: same ops on `Float`
- **Mixed int/float**: promotes int to float
- **String concatenation**: `+` on `Str`
- **Integer comparisons**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **String comparisons**: `==`, `!=`, `<`, `>`
- **Boolean comparisons**: `==`, `!=`
- **Float comparisons**: all 6
- **Logical**: `&&`, `||` on `Bool`
- **Concat (`++`)**: on `List` or `Str`

#### Builtins (builtin dispatch method, lines 711-1311)

53 builtin functions registered at interpreter construction:

| Category | Builtins |
| --- | --- |
| **I/O** | `println`, `print`, `read_line`, `read_file`, `write_file` |
| **Conversion** | `to_str`, `to_int`, `to_float`, `type_of` |
| **String** | `len`, `split`, `join`, `trim`, `contains`, `replace` |
| **String** | `starts_with`, `ends_with`, `to_upper`, `to_lower` |
| **List** | `len`, `push`, `sort`, `sort_by`, `rev`, `unique` |
| **List** | `flatten`, `chunk`, `take`, `skip`, `contains` |
| **Higher-order** | `map`, `filter`, `fold`, `flat_map`, `any`, `all` |
| **Higher-order** | `find`, `reduce`, `group_by`, `enumerate`, `zip` |
| **Math** | `range`, `min`, `max`, `abs` |
| **Debug** | `dbg` |
| **Assertions** | `assert`, `assert_eq`, `assert_ne` |
| **Assertions** | `assert_true`, `assert_false` |
| **Data** | `parse_json` (stub -- just returns the string) |

All builtins are registered as `Value::BuiltinFn(name)` in the env during
`Interpreter::new()`. The builtin dispatch method is a single giant match on the name string.

**Builtin argument convention**: When used with pipeline (`|>`), the piped value becomes the
first argument. For example, `list |> map(f)` becomes `map(list, f)`. This is handled in
the expression handler for `Pipeline`, not in the builtins themselves.

#### Pattern Matching (lines 1313-1411)

Two methods:

- `bind_pattern(pattern, value)` -- Used in `let` bindings. Supports `Ident`, `Wildcard`,
  `Tuple`, `List`, `Literal` (no-op check).
- `match_pattern(pattern, value)` -- Used in `match` expressions. Returns `bool`. Supports
  all pattern types including `Or`. Binds variables on match. `Constructor` returns error
  (not implemented).

#### Interpreter Utility Methods

- `values_equal(a, b)` -- Deep structural equality for Int, Float, Str, Bool, Unit, Tuple,
  List
- `compare_values(a, b)` -- Ordering for sort (Int, Float, Str only)
- `value_to_iterable(value)` -- Converts List or Str to `Vec<Value>` for `for` loops
- `value_to_display(value)` -- User-facing display (quotes strings)
- `value_to_debug(value)` -- Debug representation

#### Interpreter Lisp Replacement Notes

The interpreter transforms from a tree-walker over a complex AST to a central dispatch
loop over S-expressions:

- **Central dispatch(expr, env)** -- Dispatches on atom vs list. Atoms: numbers
  self-return, symbols look up in env. Lists: first element determines special form or
  function call.
- **Application(fn, args, env)** -- Handles builtin functions and user-defined lambdas.
- **Special forms**: `define`, `lambda`, `if`, `cond`, `let`, `quote`, `begin`, `set!`,
  `and`, `or`, `do` (loop).

The `Value` enum simplifies: `Fn` and `Lambda` merge into a single closure type. `Tuple` may
be dropped (use lists). `Pipeline`, `Range`, `StringInterp` disappear from the core. The
pattern matching system either drops entirely or becomes a `match` special form.

Many builtins can be retained but their dispatch moves from a name-string match to symbol
lookup in the environment.

---

### 5. REPL (`src/repl.rs`, 100 lines) -- NEEDS UPDATING

#### REPL Architecture

- Reads lines from stdin using `io::stdin().lock().lines()`
- Uses primary prompt `>>` and continuation prompt `..`
- Creates a single `Interpreter` that persists across inputs (state accumulates)
- For each input, lexes -> parses -> runs each item
- **Multi-line input**: If parse fails with an error containing "None" or "Eof"
  (incomplete input heuristic), prompts for more lines. Empty line cancels multi-line input.
- Non-Unit results are printed to stdout
- Errors printed to stderr

#### REPL Integration Points

```text
stdin -> line -> Lexer::new(buffer).tokenize()
  -> Parser::new(tokens).parse_program()
  -> for item in program.items { interpreter.handle_item(item) }
  -> print non-Unit result
```

#### REPL Lisp Replacement Notes

The REPL structure stays similar, but:

- The multi-line detection changes from error message sniffing to **paren counting** (much
  more reliable for Lisp -- just count unmatched open parens)
- The item handler becomes just the central dispatch function
- Prompts might change (e.g., `>` or `lingo>`)

---

### 6. Test Runner (`src/test_runner.rs`, 117 lines) -- NEEDS UPDATING

#### Test Runner Architecture

The test runner is largely **syntax-agnostic**. It operates on the interpreter level:

- `discover_tests(interpreter)` -- Scans `env.binding_names()` for names starting with
  `test_`, filters to `Value::Fn` variants, sorts alphabetically.
- `run_test_mode(interpreter)` -- Calls `run_test_mode_captured`, prints output, returns
  `Err` if any failed.
- `run_test_mode_captured(interpreter)` -- For each test function:
  1. Saves env (clone)
  2. Calls `interpreter.call_function(&func, &[])`
  3. Restores env
  4. Records PASS/FAIL
  5. Formats failure details (parses `[assert]` prefix, splits expected/got)

#### Test Runner Output Format

```text
PASS test_name
FAIL test_name

failures:

  test_name:
    expected: X
         got: Y

N passed, M failed
```

#### Test Runner Lisp Replacement Notes

The test runner module itself needs minimal changes:

- `discover_tests` will still look for `test_` prefixed function names in the env
- The function calling mechanism will become the application function
- The `Value::Fn` check may need updating depending on how closures are represented

The **test files** (`.ln` examples and integration test source strings) all need rewriting in
Lisp syntax.

---

### 7. CLI Entry Point (`src/main.rs`, 80 lines) -- MINIMAL CHANGES

#### CLI Modes

1. **No args** -- Start REPL (`repl::start()`)
2. **`--test <file.ln>`** -- Read file, lex/parse/load declarations, run test mode
3. **`<file.ln>`** -- Read file, lex/parse/run (calls `main()` if defined, else executes
   top-level)

#### Current CLI Pipeline

```text
read_file -> Lexer::new(source).tokenize()
  -> Parser::new(tokens).parse_program()
  -> Interpreter::new().run(&program)
```

For test mode:

```text
... -> Interpreter::new().load_declarations(&program)
  -> test_runner::run_test_mode(&mut interpreter)
```

#### CLI Lisp Replacement Notes

Structure stays identical. Just update the pipeline calls to match the new
lexer/parser/interpreter API. The `run` function might change to:

```text
read_file -> lex(source) -> parse(tokens) -> run_program(exprs, env)
```

---

### 8. Integration Tests (`tests/test_runner.rs`, 640 lines) -- NEEDS REWRITING

#### Test Categories and Counts

| Category | Tests | Lines |
| --- | --- | --- |
| assert\_eq | 8 | 34-92 |
| assert\_ne | 4 | 98-126 |
| assert\_true / assert\_false | 10 | 132-210 |
| Test discovery | 5 | 216-249 |
| Test failure isolation | 2 | 255-273 |
| Test reporter output | 3 | 279-313 |
| CLI --test flag | 3 | 337-380 |
| Item handling (REPL support) | 7 | 394-458 |
| REPL integration | 7 | 466-618 |
| CLI normal mode | 1 | 624-639 |

Total: 50 integration tests.

#### Test Helpers

- `run_lingo(source)` -- Lex/parse/run, return Result
- `load_declarations(source)` -- Lex/parse/load into Interpreter (no execution)
- `parse_program(source)` -- Lex/parse, return Program AST
- `write_temp_ln(source)` -- Write source to temp `.ln` file
- `cargo_bin()` -- Path to compiled binary

#### Key Test Patterns

All tests embed Lingo source as string literals in current syntax:

```rust
run_lingo("fn main() { assert_eq(1, 1) }")
run_lingo(r#"fn main() { assert_eq("hello", "hello") }"#)
load_declarations("fn test_a() { } fn test_b() { }")
```

REPL tests pipe input via stdin to the binary process and check stdout/stderr.

#### Integration Test Lisp Replacement Notes

Every source string in every test must be rewritten. For example:

| Current | Lisp Equivalent |
| --- | --- |
| `"fn main() { assert_eq(1, 1) }"` | `"(define (main) (assert-eq 1 1))"` |
| `"fn test_a() { }"` | `"(define (test-a))"` |
| `"1 + 2"` | `"(+ 1 2)"` |
| `"let x = 5"` | `"(define x 5)"` |

The test discovery logic needs updating if function names use `-` instead of `_` (Lisp
convention).

---

### 9. Example Programs (`examples/*.ln`) -- NEEDS REWRITING

#### hello.ln (4 lines)

```text
fn main() {
  let name = "World"
  println("Hello, {name}!")
}
```

#### basics.ln (43 lines)

Demonstrates: function definition with type annotations, arithmetic, string interpolation,
if/else expressions, lists, pipelines (`|>`), `map`, `filter`, `fold`, pattern matching,
`len`, `to_upper`.

#### fizzbuzz.ln (11 lines)

Demonstrates: `for` loop with inclusive range (`1..=100`), `match` on tuple, modulo, `to_str`,
`println`.

#### test\_framework\_test.ln (35 lines)

Demonstrates: `test_*` functions, `assert_eq`, `assert_ne`, `assert_true`, `assert_false`,
pipeline with `map`/`filter`.

---

### 10. Module System (`src/lib.rs`, 8 lines)

```rust
pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod test_runner;
```

Has `#![allow(dead_code)]` at crate level. For the Lisp transformation, the module list stays
the same but the contents of each module change significantly.

---

### 11. Env Field Visibility

The `Env` struct's `bindings` field is **private** (`bindings: HashMap<String, Value>`) but
accessed in the function calling method via `saved_env.bindings.iter()` (line 663). This works
because it is on `Interpreter`, which is in the same module. The test runner accesses
`interpreter.env` via `pub(crate)` visibility on the `env` field.

---

## Summary of Transformation Impact

| Component | Lines | Impact |
| --- | --- | --- |
| `src/lexer.rs` | 623 | **Full rewrite** (shrinks to ~100 lines) |
| `src/ast.rs` | 153 | **Full rewrite** (shrinks to ~20 lines) |
| `src/parser.rs` | 901 | **Full rewrite** (shrinks to ~100 lines) |
| `src/interpreter.rs` | 1461 | **Major rewrite** (core ~200 + builtins ~600) |
| `src/repl.rs` | 100 | **Minor update** (paren counting, new API calls) |
| `src/test_runner.rs` | 117 | **Minor update** (Value enum changes) |
| `src/main.rs` | 80 | **Minor update** (new pipeline calls) |
| `src/lib.rs` | 8 | **No change** |
| `tests/test_runner.rs` | 640 | **Full rewrite** (all source strings change) |
| `examples/*.ln` | ~93 | **Full rewrite** (S-expression syntax) |

**Estimated total**: Current codebase is ~3,175 lines of Rust + ~93 lines of Lingo. The Lisp
version should be roughly ~1,200 lines of Rust + ~60 lines of Lingo examples, a significant
reduction in complexity.

## Open Questions

1. **Naming convention**: Should Lisp builtins use `-` (e.g., `assert-eq`) or `_` (e.g.,
   `assert_eq`)? The test runner discovers tests by `test_` prefix -- this convention must
   be consistent.
2. **List representation**: Should the AST use a dedicated `List` type or Lisp-style cons
   cells? Cons cells are more traditional but `Vec<Expr>` is simpler for the tree-walking
   interpreter.
3. **Which special forms to support**: Minimum viable set is `define`, `lambda`, `if`,
   `quote`, `begin`. Do we also want `let`, `cond`, `set!`, `do`, `and`, `or`, `match`?
4. **Macro system**: Should the initial Lisp support macros (`defmacro`) or defer that?
5. **Tail call optimization**: The current interpreter uses Rust's call stack. Should the Lisp
   version implement TCO via trampolining?
6. **Pipeline operator**: Drop it entirely (not idiomatic Lisp) or keep as syntactic sugar?
7. **String interpolation**: Drop it (not traditional Lisp) or keep as a reader macro?
8. **File extension**: Keep `.ln` or change to `.lsp`/`.lisp`?

## Recommendations

1. **Start with lexer + AST + parser** -- These are the simplest pieces and unlock testing
   the new syntax immediately. The Lisp parser is dramatically simpler than the current one.
2. **Keep the builtin set** -- The builtins are useful and largely syntax-independent. Move
   them from a name-string dispatch to symbol-based env lookup.
3. **Use `Vec<Expr>` for lists, not cons cells** -- Simpler implementation, better
   performance for the common case, consistent with the current `Value::List`.
4. **Implement incrementally**: Lexer/AST/Parser first, then the core dispatch loop, then
   port builtins, then update REPL/test runner, then rewrite tests and examples last.
5. **Keep the test runner architecture** -- It is already well-separated from syntax concerns.
   Only the `Value` type references need updating.
