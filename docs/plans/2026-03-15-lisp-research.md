# Research: Lingo to Lisp Transformation (2026-03-15)

## Problem Statement

Lingo is currently a tree-walking interpreter written in Rust with an
imperative/functional syntax inspired by Rust and Go (curly braces,
keywords, infix operators, Go-style semicolon insertion). The goal is
to transform it into a Lisp with S-expression syntax and
evaluate/apply semantics while preserving its existing builtin
function library and test infrastructure.

The transformation touches every layer of the pipeline -- lexer, AST,
parser, interpreter, REPL, and all test/example files -- but the
resulting codebase should be dramatically simpler: roughly 1,200 lines
of Rust down from 3,175, a 60% reduction in implementation complexity.

## Requirements

- Replace the lexer with one that tokenizes S-expression syntax
  (parens, atoms, strings, numbers)
- Replace the parser with one that produces a Lisp AST (nested lists
  of a simplified `Expr` enum)
- Simplify the AST from 20+ node types to approximately 7
- Rewrite the interpreter around an evaluate/apply model instead of
  the current tree-walk over a complex AST
- Update the REPL to work with the new pipeline (paren-counting for
  multi-line input)
- Update the test runner for new Value representation
- Rewrite all example programs in Lisp syntax
- Rewrite all 50 integration tests with Lisp source strings

## Findings

### Tokenizer and Lexical Analysis

The current lexer (`src/lexer.rs`, 623 lines) recognizes 86 token
variants across keywords, literals, operators, delimiters, and special
tokens. It implements Go-style semicolon insertion via
`insert_newlines()` and string interpolation with control-character
encoding (`\x01`/`\x02` markers).

The Lisp replacement needs only 5 token types:

| Token    | Purpose                                |
| -------- | -------------------------------------- |
| `LParen` | `(`                                    |
| `RParen` | `)`                                    |
| `Atom`   | Symbols, numbers (distinguished later) |
| `Str`    | Double-quoted strings with escapes     |
| `Quote`  | The `'` shorthand for `(quote ...)`    |

The tokenization algorithm is a single character-dispatch loop.
Commas are treated as optional whitespace (Clojure convention).
Comments change from `#` to `;`. No semicolon insertion is needed.
No multi-character operators. No string interpolation machinery.
The current `Lexer` struct's char-by-char approach (with
`peek`/`advance`) can be reused structurally, but the contents
simplify from ~620 lines to ~100 lines.

### AST Representation

The current AST (`src/ast.rs`, 153 lines) includes `Program`,
`Item` (3 variants), `FnDecl`, `Param`, `Block`, `Stmt`
(6 variants), `Expr` (16 variants), `BinOp` (14 variants),
`UnaryOp` (2 variants), `Pattern` (7 variants), `MatchArm`,
`StringPart`, `Literal`, and `Span` -- roughly 30 distinct types.

The Lisp AST collapses to a single enum:

```rust
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    List(Vec<Expr>),
    Nil,
}
```

All distinctions between items, statements, and expressions
disappear. `BinOp`, `UnaryOp`, `Pattern`, `MatchArm`, `Block`,
`FnDecl`, and `Param` are all eliminated. The file shrinks to
~20 lines.

The design recommends `Vec<Expr>` for list representation rather
than traditional cons cells, for simplicity and consistency with
the existing `Value::List` implementation.

### Parser Architecture

The current parser (`src/parser.rs`, 901 lines) is a recursive
descent parser with 13 precedence levels, lambda detection with
backtracking, pattern matching, block parsing, and semicolon
handling.

The Lisp parser needs only three functions:

- `read_expr()` -- dispatch on `(` for lists, `'` for quote,
  else atom
- `read_list()` -- read expressions until `)` encountered
- `read_atom()` -- parse numbers (`i64` then `f64`), booleans
  (`#t`/`true`, `#f`/`false`), `nil`, or symbols

Operator precedence is irrelevant because structure is explicit in
S-expressions: `(+ (* 2 3) 4)`. The parser shrinks from ~900 lines
to ~40-100 lines. The design document provides a complete
implementation sketch of `read_expr` at approximately 40 lines.

### Evaluation Model

The current interpreter (`src/interpreter.rs`, 1,461 lines) uses
separate methods for `eval_expr` (16 expression variants),
`eval_stmt` (6 statement variants), `eval_block`, and `eval_item`,
plus dedicated `eval_binop`/`eval_unop` handlers and a 600-line
`call_builtin` dispatch method.

The Lisp evaluator follows three rules:

1. **Self-evaluating forms** (numbers, strings, booleans, nil)
   return themselves
2. **Symbols** look up in the environment
3. **Lists** check the head for special forms; otherwise evaluate
   all elements and apply the function

This replaces four separate evaluation methods with a single
`evaluate(expr, env)` function. Both source documents confirm the
current `eval_expr` already maps closely to this pattern.

### Special Forms

The design identifies 8 core special forms and 3 practical
extensions:

**Core (minimum viable):**

| Form     | Syntax                   | Purpose                   |
| -------- | ------------------------ | ------------------------- |
| `define` | `(define name value)`    | Bind value in environment |
| `fn`     | `(fn (a b) body)`        | Create closure            |
| `if`     | `(if test then else)`    | Conditional; one branch   |
| `quote`  | `(quote expr)` / `'expr` | Return unevaluated        |
| `begin`  | `(begin e1 e2 ... en)`   | Sequence; return last     |
| `set!`   | `(set! name value)`      | Mutate existing binding   |
| `and`    | `(and a b c)`            | Short-circuit AND         |
| `or`     | `(or a b c)`             | Short-circuit OR          |

**Extensions:**

| Form   | Syntax                         | Purpose                  |
| ------ | ------------------------------ | ------------------------ |
| `defn` | `(defn name (params) body)`    | Sugar for define + fn    |
| `let`  | `(let ((x 1) (y 2)) body)`     | Local bindings in scope  |
| `cond` | `(cond (t1 e1) ... (else eN))` | Multi-branch conditional |

The design document provides complete Rust implementation sketches
for all 11 forms.

### Value Type and Function Calling

The current `Value` enum has 10 variants including separate `Fn`
and `Lambda`, `Tuple`, `Unit`, and `BuiltinFn(String)`.

The Lisp `Value` simplifies to 8 variants:

- `Unit` becomes `Nil` (Lisp convention)
- `Tuple` is removed (use `List`)
- `Fn` and `Lambda` merge into a single
  `Lambda { params, body, closure }` variant (named functions
  are lambdas bound via `define`)
- `BuiltinFn(String)` becomes `Builtin { name, func }` with an
  actual function pointer (`fn(&[Value]) -> Result<Value, String>`)

The function pointer approach eliminates the current 600-line
`call_builtin` string-dispatch method. Each builtin becomes a
self-contained function. The `apply_function` call simply invokes
`(func)(args)`. This also makes builtins first-class values:
`(fold xs 0 +)` works because `+` resolves to a `Value::Builtin`.

Arithmetic operators (`+`, `-`, `*`, `/`, `%`, `<`, `>`, `<=`,
`>=`, `=`) become builtins in the environment rather than AST-level
constructs. This eliminates the `BinOp` enum, `UnaryOp` enum, and
all precedence/dispatch logic. Operators gain variadic behavior:
`(+ 1 2 3 4)` evaluates to 10.

### Builtin Function Inventory

The current 53 builtins are preserved and reorganized:

| Category   | Builtins                                         |
| ---------- | ------------------------------------------------ |
| Arithmetic | `+`, `-`, `*`, `/`, `mod`, `abs`, `min`, `max`   |
| Comparison | `=`, `<`, `>`, `<=`, `>=`                        |
| Logic      | `not`                                            |
| List       | `list`, `cons`, `first`, `rest`, `nth`, `length` |
| List       | `append`, `reverse`, `map`, `filter`, `fold`     |
| List       | `for-each`, `flatten`, `zip`, `take`, `drop`     |
| List       | `sort`, `sort-by`, `any?`, `all?`, `find`        |
| List       | `unique`, `chunk`, `enumerate`, `group-by`       |
| String     | `str`, `string-length`, `substring`              |
| String     | `split`, `join`, `trim`, `contains?`, `replace`  |
| String     | `starts-with?`, `ends-with?`                     |
| String     | `upper-case`, `lower-case`                       |
| Type       | `type-of`, `int?`, `float?`, `string?`           |
| Type       | `bool?`, `list?`, `nil?`, `number?`              |
| Conversion | `->int`, `->float`, `->str`                      |
| I/O        | `println`, `print`, `read-line`                  |
| I/O        | `read-file`, `write-file`                        |
| Debug/Test | `dbg`, `assert`, `assert-eq`                     |

### Environment Design

The current `Env` (linked-list with `HashMap<String, Value>`
bindings and `Option<Box<Env>>` parent) is already the standard
Lisp environment design and can be kept as-is. The `update` method
serves `set!`, and `binding_names` supports REPL introspection and
test discovery.

A future optimization noted: switch from `Box<Env>` to
`Rc<RefCell<Env>>` for the parent pointer to avoid cloning the
entire chain when creating closures. The current clone-heavy
approach works for a small interpreter but becomes expensive at
scale. This is explicitly deferred -- not needed for the initial
transformation.

### Feature Mapping and Disposition

**Features that map directly to Lisp:**

| Lingo                   | Lisp                     |
| ----------------------- | ------------------------ |
| `let x = expr`          | `(define x expr)`        |
| `fn name(a, b) {...}`   | `(defn name (a b) body)` |
| `a => a + 1`            | `(fn (a) (+ a 1))`       |
| `if c { a } else { b }` | `(if c a b)`             |
| `{ s1; s2; expr }`      | `(begin s1 s2 expr)`     |
| `x + y`                 | `(+ x y)`                |
| `[1, 2, 3]`             | `(list 1 2 3)`           |

**Features requiring transformation:**

| Feature              | Recommendation                         |
| -------------------- | -------------------------------------- |
| Pipeline (`\|>`)     | Implement `->` threading macro         |
| Ranges (`1..10`)     | Use `(range 1 10)` builtin             |
| Tuples               | Use lists; add `(tuple)` later         |
| String interpolation | Drop; use `(str "hello " name)`        |
| Compound assignment  | Use `(set! x (+ x 1))`                 |
| `for` loops          | Drop; use higher-order functions       |
| `while` loops        | Keep as special form                   |
| `match`/patterns     | Simple literal + wildcard `match` only |
| List concat (`++`)   | Use `(append xs ys)` builtin           |

**Features to drop entirely:**

| Feature                           | Reason                         |
| --------------------------------- | ------------------------------ |
| Type annotations (`: Int`)        | Lisp is dynamically typed      |
| `struct`, `enum`, `trait`, `impl` | Not implemented in interpreter |
| `pub`, `use`, `mod`               | Module system not needed       |
| `async`/`await`                   | Not implemented                |
| `return` statement                | Functions return last expr     |
| `break`                           | No imperative `for` loops      |
| `mut` keyword                     | All bindings mutable via set!  |
| `.field` access                   | No structs; use assoc lists    |
| `[index]` access                  | Use `(nth list index)` builtin |

### REPL Adaptation

The REPL (`src/repl.rs`, 100 lines) structure stays similar. Key
changes:

- Multi-line detection switches from error-message sniffing
  (checking for "None"/"Eof" in parse errors) to **paren
  counting** -- count unmatched open parens, which is more
  reliable for Lisp
- The item handler becomes just the `evaluate` function
- The pipeline simplifies: `read -> evaluate -> print`

### Test Infrastructure

The test runner (`src/test_runner.rs`, 117 lines) is largely
syntax-agnostic. It discovers tests by scanning
`env.binding_names()` for `test_` prefixed names that are
`Value::Fn`. Changes needed:

- Update `Value::Fn` check to match the new `Value::Lambda`
  variant
- The function calling mechanism switches to `apply_function`

The 50 integration tests (`tests/test_runner.rs`, 640 lines) must
be fully rewritten because every source string is in current Lingo
syntax. Example transformations:

| Current                             | Lisp                           |
| ----------------------------------- | ------------------------------ |
| `"fn main() { assert_eq(1, 1) }"`   | `"(defn main () (assert-eq))"` |
| `"fn test_a() { } fn test_b() { }"` | `"(defn test_a () nil) ..."`   |
| `"1 + 2"`                           | `"(+ 1 2)"`                    |

The 4 example programs (`examples/*.ln`, ~93 lines total) also
need full rewrites in S-expression syntax.

### CLI Entry Point

The CLI (`src/main.rs`, 80 lines) needs only minor updates: swap
the pipeline from `Lexer -> Parser -> Interpreter` to the new
reader/evaluator calls. The three modes (REPL, run file, run
tests) and module structure (`src/lib.rs`) remain unchanged.

## Technical Constraints

- **Zero runtime dependencies**: The project has no runtime
  dependencies (only `tempfile` as dev-dependency). The Lisp
  transformation should maintain this.
- **Rust edition 2024**: The project uses Rust edition 2024.
- **Clone-heavy environment**: The current `Env` uses `Box<Env>`
  parents with full cloning on scope creation. This works but is
  expensive at scale. A `Rc<RefCell<Env>>` optimization is noted
  but deferred.
- **Debug/Clone for function pointers**: Rust `fn` pointers
  implement `Clone` and `Copy`. For `Debug`, use the `name`
  field. For `PartialEq`, compare by pointer address or name.
- **Test discovery convention**: The test runner discovers
  functions by `test_` prefix. If Lisp naming switches to hyphens
  (`test-foo`), the discovery logic must be updated to match.
  Both documents flag this as an open question.

## Open Questions

1. **Naming convention**: Should builtins use hyphens
   (`assert-eq`) or underscores (`assert_eq`)? The test runner's
   `test_` prefix discovery depends on this decision being
   consistent.
2. **Special form set**: Is the proposed set of 11 special forms
   (8 core + 3 extensions) the right scope, or should `while` and
   `match` be included from the start?
3. **Macro system**: Should the initial Lisp support `defmacro`,
   or defer macros entirely? The design notes the architecture is
   "macro-ready" but does not require macros.
4. **Tail call optimization**: Should the Lisp implement TCO via
   trampolining? The current interpreter uses Rust's call stack
   with no TCO.
5. **File extension**: Keep `.ln` or change to `.lsp`/`.lisp`?
6. **Boolean literals**: Use `#t`/`#f` (Scheme convention) or
   `true`/`false` (current Lingo convention)? The parser design
   supports both.
7. **`Env` ownership model**: Stay with `Box<Env>` cloning or
   upgrade to `Rc<RefCell<Env>>`? Both documents recommend
   deferring the upgrade.

## Recommendations

1. **Implement in 5 phases**: Reader (lexer + AST + parser) first,
   then core evaluator, then builtins, then REPL/test runner,
   then extensions (threading macro, match, while). This is the
   ordering both documents converge on.
2. **Use `Vec<Expr>` for lists, not cons cells**: Simpler, better
   performance, consistent with existing `Value::List`.
3. **Use function pointers for builtins**: Eliminates the 600-line
   string-dispatch method, makes operators first-class, and keeps
   builtins self-contained.
4. **Keep the builtin set**: The 53 existing builtins are useful
   and largely syntax-independent. Rename to Lisp conventions
   (hyphens) and register as function pointer values.
5. **Implement `->` threading macro**: Preserves the pipeline and
   linear-composition design goal that was central to Lingo,
   using the well-understood Clojure pattern.
6. **Keep `while` as a special form**: Pragmatic concession to
   imperative style; drop `for` in favor of higher-order
   functions.
7. **Implement simple `match`**: Literal patterns and wildcards
   only. Defer constructor patterns, or-patterns, list patterns,
   and guards.
8. **Defer `Rc<RefCell<Env>>` optimization**: The current
   clone-based approach works for the initial transformation.
   Optimize later if profiling shows it matters.
9. **Defer macro system**: The architecture supports it, but
   macros are not needed for the initial transformation and add
   significant complexity.
10. **Start with the reader**: The lexer/AST/parser are the
    simplest pieces, produce the most dramatic simplification
    (900 lines to ~100), and unlock testing the new syntax
    immediately.

## Complexity Reduction Summary

| Component        | Current        | Lisp           | Reduction |
| ---------------- | -------------- | -------------- | --------- |
| Token types      | ~86 variants   | ~5 variants    | 94%       |
| AST node types   | ~30 types      | 7 variants     | 77%       |
| Lexer            | ~620 lines     | ~100 lines     | 84%       |
| Parser           | ~900 lines     | ~40-100 lines  | 89-96%    |
| Interpreter core | ~1,460 lines   | ~800 lines     | ~45%      |
| REPL             | ~100 lines     | ~100 lines     | 0%        |
| Test runner      | ~117 lines     | ~117 lines     | 0%        |
| **Total Rust**   | **~3,175 loc** | **~1,200 loc** | **~62%**  |

## Sources

| Document                                 | Focus Area                    |
| ---------------------------------------- | ----------------------------- |
| `docs/plans/2026-03-15-lisp-codebase.md` | Codebase component mapping    |
| `docs/plans/2026-03-15-lisp-design.md`   | Lisp language design patterns |
