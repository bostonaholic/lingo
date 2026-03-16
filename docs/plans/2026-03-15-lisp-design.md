# Lisp Transformation Design: Turning Lingo into an S-expression Language

**Date:** 2026-03-15
**Goal:** Transform the existing tree-walking interpreter (Rust) into a Lisp.

---

## 1. S-expression Reader/Parser

### Tokenizer Design

The simplest S-expression tokenizer recognizes only five token types:

```rust
enum Token {
    LParen,
    RParen,
    Atom(String),    // symbols, numbers -- distinguished later
    Str(String),     // double-quoted strings
    Quote,           // the ' shorthand
}
```

The algorithm is a single loop over characters:

- Whitespace and commas: skip (commas are optional whitespace, as in Clojure)
- `(` / `)`: emit delimiter token
- `'`: emit Quote token
- `"`: consume until closing `"`, handling `\n`, `\\`, `\"` escapes
- `;` or `#`: skip to end of line (line comment)
- Anything else: consume while not whitespace/paren/quote, emit as `Atom`

This is dramatically simpler than Lingo's current 80+ token-kind lexer.
There is no keyword recognition at the lexer level -- `define`, `if`,
`fn` are just symbols. The parser distinguishes them.

**Mapping from current Lingo lexer:** The current `Lexer` struct
(char-by-char with `peek`/`advance`) can be reused nearly verbatim.
Strip out all multi-char operator recognition, the Go-style newline
insertion, the string interpolation machinery, and the keyword table.
What remains is the core loop.

### AST (the "Expr" type)

A Lisp AST is almost trivially simple:

```rust
#[derive(Debug, Clone)]
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

That's it. Seven variants replace the current 16 `Expr` variants,
7 `Stmt` variants, 4 `Item` variants, and all the auxiliary structs
(`FnDecl`, `LetStmt`, `ForStmt`, `WhileStmt`, `MatchArm`, `Block`,
`Param`, `Pattern`). The entire `ast.rs` file collapses to ~20 lines.

### Parser

The parser is a single recursive function:

```rust
fn read_expr(tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
    match &tokens[*pos] {
        Token::LParen => {
            *pos += 1;
            let mut list = Vec::new();
            while tokens[*pos] != Token::RParen {
                list.push(read_expr(tokens, pos)?);
            }
            *pos += 1; // consume RParen
            Ok(Expr::List(list))
        }
        Token::Quote => {
            *pos += 1;
            let quoted = read_expr(tokens, pos)?;
            Ok(Expr::List(vec![
                Expr::Symbol("quote".into()),
                quoted,
            ]))
        }
        Token::Atom(s) => {
            *pos += 1;
            // Try parsing as number, then bool, then symbol
            if let Ok(n) = s.parse::<i64>() {
                Ok(Expr::Int(n))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Expr::Float(f))
            } else if s == "#t" || s == "true" {
                Ok(Expr::Bool(true))
            } else if s == "#f" || s == "false" {
                Ok(Expr::Bool(false))
            } else if s == "nil" {
                Ok(Expr::Nil)
            } else {
                Ok(Expr::Symbol(s.clone()))
            }
        }
        Token::Str(s) => {
            *pos += 1;
            Ok(Expr::Str(s.clone()))
        }
        Token::RParen => Err("Unexpected ')'".into()),
    }
}
```

The current 900-line `parser.rs` with its recursive descent
precedence climbing (13 precedence levels) collapses to approximately
40 lines. Operator precedence is irrelevant in S-expressions because
the structure is explicit: `(+ (* 2 3) 4)`.

---

## 2. Evaluate/Apply Model

### The Classic Evaluation Rules

Every Lisp evaluator follows the same fundamental rules.
Given an expression:

1. **Self-evaluating forms:** Numbers, strings, booleans, nil
   evaluate to themselves.
2. **Symbols:** Look up in the current environment. Error if unbound.
3. **Lists (compound forms):** Examine the first element (the "head"):
   - If it names a **special form**, apply special evaluation rules.
   - Otherwise, it's a **function application**: evaluate all
     elements (head + args), then apply the function to the
     evaluated arguments.

### Rust Implementation Pattern

```rust
fn evaluate(
    expr: &Expr,
    env: &mut Env,
) -> Result<Value, String> {
    match expr {
        // Self-evaluating
        Expr::Int(n)   => Ok(Value::Int(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        Expr::Str(s)   => Ok(Value::Str(s.clone())),
        Expr::Bool(b)  => Ok(Value::Bool(*b)),
        Expr::Nil      => Ok(Value::Nil),

        // Symbol lookup
        Expr::Symbol(name) => env.get(name)
            .ok_or_else(|| format!("Undefined symbol: {}", name)),

        // List: special forms or application
        Expr::List(elems) if elems.is_empty() => Ok(Value::Nil),
        Expr::List(elems) => {
            if let Expr::Symbol(head) = &elems[0] {
                match head.as_str() {
                    "define" => eval_define(&elems[1..], env),
                    "fn"     => eval_fn(&elems[1..], env),
                    "if"     => eval_if(&elems[1..], env),
                    "cond"   => eval_cond(&elems[1..], env),
                    "let"    => eval_let(&elems[1..], env),
                    "begin"  => eval_begin(&elems[1..], env),
                    "quote"  => eval_quote(&elems[1..]),
                    "and"    => eval_and(&elems[1..], env),
                    "or"     => eval_or(&elems[1..], env),
                    "set!"   => eval_set(&elems[1..], env),
                    "defn"   => eval_defn(&elems[1..], env),
                    _        => eval_application(elems, env),
                }
            } else {
                eval_application(elems, env)
            }
        }
    }
}

fn eval_application(
    elems: &[Expr],
    env: &mut Env,
) -> Result<Value, String> {
    let func = evaluate(&elems[0], env)?;
    let args: Result<Vec<Value>, String> = elems[1..]
        .iter()
        .map(|e| evaluate(e, env))
        .collect();
    apply_function(&func, &args?, env)
}

fn apply_function(
    func: &Value,
    args: &[Value],
    env: &mut Env,
) -> Result<Value, String> {
    match func {
        Value::Lambda { params, body, closure } => {
            let mut local_env = Env::child(closure);
            for (param, arg) in params.iter().zip(args) {
                local_env.set(param.clone(), arg.clone());
            }
            evaluate(&body, &mut local_env)
        }
        Value::Builtin { func: f, .. } => f(args),
        _ => Err(format!("Not a function: {:?}", func)),
    }
}
```

### Mapping from Current Interpreter

The current `eval_expr` method already follows this pattern closely:

| Current Lingo                   | Lisp equivalent                      |
| ------------------------------- | ------------------------------------ |
| `Expr::Literal(lit)`            | Self-evaluating (Int/Float/Str/Bool) |
| `Expr::Ident(name, _)`          | Symbol lookup                        |
| `Expr::Call(callee, args, _)`   | Function application                 |
| `Expr::Binary(l, op, r, _)`     | `(op l r)` -- becomes a call         |
| `Expr::Unary(op, val, _)`       | `(op val)` -- becomes a call         |
| `Expr::If(cond, then, else, _)` | `(if cond then else)` special form   |
| `Expr::Lambda(params, body, _)` | `(fn (params) body)` special form    |
| `Expr::Block(block, _)`         | `(begin ...)` special form           |

The key structural change: the current interpreter has `eval_expr`,
`eval_stmt`, `eval_block`, and `eval_item` as separate methods.
In a Lisp, there is only `evaluate`. Statements, expressions,
blocks, and items are all just lists.

---

## 3. Special Forms Inventory

### Core Special Forms (minimum viable Lisp)

These cannot be functions because they control evaluation order:

| Form     | Syntax                   | Purpose                                    |
| -------- | ------------------------ | ------------------------------------------ |
| `define` | `(define name value)`    | Bind a value in the current environment    |
| `fn`     | `(fn (a b) body)`        | Create an anonymous function (closure)     |
| `if`     | `(if test then else)`    | Conditional; evaluates only one branch     |
| `quote`  | `(quote expr)` / `'expr` | Return expression unevaluated              |
| `begin`  | `(begin e1 e2 ... en)`   | Evaluate sequence, return last             |
| `set!`   | `(set! name value)`      | Mutate an existing binding                 |
| `and`    | `(and a b c)`            | Short-circuit AND; last truthy/first falsy |
| `or`     | `(or a b c)`             | Short-circuit OR; first truthy/last falsy  |

### Practical Extensions

These are not strictly necessary but make the language pleasant:

| Form   | Syntax                         | Purpose                            |
| ------ | ------------------------------ | ---------------------------------- |
| `defn` | `(defn name (params) body)`    | Sugar for `(define name (fn ...))` |
| `let`  | `(let ((x 1) (y 2)) body)`     | Local bindings in extended env     |
| `cond` | `(cond (t1 e1) ... (else eN))` | Multi-branch conditional           |

### Implementation Sketch for Each

**define:**

```rust
fn eval_define(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    // (define name value)
    let name = args[0].as_symbol()?;
    let value = evaluate(&args[1], env)?;
    env.set(name.clone(), value.clone());
    Ok(value)
}
```

**fn/lambda:**

```rust
fn eval_fn(args: &[Expr], env: &Env) -> Result<Value, String> {
    // (fn (a b c) body)
    let params = args[0].as_list()?
        .iter()
        .map(|e| e.as_symbol().map(|s| s.clone()))
        .collect::<Result<Vec<String>, _>>()?;
    let body = args[1].clone();
    Ok(Value::Lambda {
        params,
        body: Box::new(body),
        closure: env.clone(),
    })
}
```

**if:**

```rust
fn eval_if(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    // (if test then) or (if test then else)
    let test = evaluate(&args[0], env)?;
    if test.is_truthy() {
        evaluate(&args[1], env)
    } else if args.len() > 2 {
        evaluate(&args[2], env)
    } else {
        Ok(Value::Nil)
    }
}
```

**let:**

```rust
fn eval_let(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    // (let ((x 1) (y 2)) body)
    let bindings = args[0].as_list()?;
    let mut local_env = Env::child(env);
    for binding in bindings {
        let pair = binding.as_list()?;
        let name = pair[0].as_symbol()?;
        let val = evaluate(&pair[1], &mut local_env)?;
        local_env.set(name.clone(), val);
    }
    evaluate(&args[1], &mut local_env)
}
```

**cond:**

```rust
fn eval_cond(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    // (cond (test1 expr1) (test2 expr2) ... (else exprN))
    for clause in args {
        let pair = clause.as_list()?;
        if let Expr::Symbol(s) = &pair[0] {
            if s == "else" {
                return evaluate(&pair[1], env);
            }
        }
        let test = evaluate(&pair[0], env)?;
        if test.is_truthy() {
            return evaluate(&pair[1], env);
        }
    }
    Ok(Value::Nil)
}
```

**begin:**

```rust
fn eval_begin(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Nil;
    for expr in args {
        result = evaluate(expr, env)?;
    }
    Ok(result)
}
```

**and / or:**

```rust
fn eval_and(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Bool(true);
    for expr in args {
        result = evaluate(expr, env)?;
        if !result.is_truthy() {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_or(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Bool(false);
    for expr in args {
        result = evaluate(expr, env)?;
        if result.is_truthy() {
            return Ok(result);
        }
    }
    Ok(result)
}
```

**set!:**

```rust
fn eval_set(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let name = args[0].as_symbol()?;
    let value = evaluate(&args[1], env)?;
    if env.update(name, value.clone()) {
        Ok(value)
    } else {
        Err(format!(
            "Cannot set! undefined variable: {}", name
        ))
    }
}
```

**defn (sugar):**

```rust
fn eval_defn(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    // (defn name (params) body)
    //   => (define name (fn (params) body))
    let name = args[0].as_symbol()?;
    let lambda = eval_fn(&args[1..], env)?;
    env.set(name.clone(), lambda.clone());
    Ok(lambda)
}
```

---

## 4. Mapping Existing Lingo Features to Lisp

### Features that map naturally

| Lingo feature          | Lisp equivalent          | Notes                    |
| ---------------------- | ------------------------ | ------------------------ |
| `let x = expr`         | `(define x expr)`        | Or `(let ...)` for local |
| `fn name(a, b) {...}`  | `(defn name (a b) body)` |                          |
| lambda `a => a + 1`    | `(fn (a) (+ a 1))`       |                          |
| `if c { a } else { b}` | `(if c a b)`             |                          |
| `match x { ... }`      | `(cond ...)`             | Simple cases only        |
| `{ s1; s2; expr }`     | `(begin s1 s2 expr)`     |                          |
| `x + y`, `x * y`       | `(+ x y)`, `(* x y)`     | Prefix function calls    |
| `!x`                   | `(not x)`                |                          |
| `-x` (negate)          | `(- x)` or `(neg x)`     |                          |
| `x == y`               | `(= x y)`                |                          |
| `x != y`               | `(not (= x y))`          |                          |
| `println(x)`           | `(println x)`            | Already a function call  |
| `[1, 2, 3]`            | `(list 1 2 3)`           |                          |
| `x && y`               | `(and x y)`              | Special form             |
| `x \                   | \                        | y`                       |

### Features to transform

**Pipeline (`|>`):** This is syntactic sugar. It does not need to be
a special form or even syntax. There are two options:

1. **Drop it.** Write `(filter (map xs f) g)` instead. This is the
   traditional Lisp approach -- nesting replaces piping.

2. **Threading macro.** Implement as a macro or special form `->`:

   ```scheme
   (-> x
       (map double)
       (filter even?)
       (fold 0 +))
   ```

   This desugars to `(fold (filter (map x double) even?) 0 +)`.

   **Recommendation:** Implement `->` as a special form. It was one of
   Lingo's design goals (token efficiency via linear composition),
   and it's a well-understood Clojure pattern.

**Ranges (`1..10`, `1..=10`):** Replace with a `range` builtin:

```scheme
(range 1 10)       ; exclusive end
(range 1 10 true)  ; inclusive end -- or just (range 1 11)
```

**Tuples (`(a, b, c)`):** Lisp uses lists for everything. Tuples
become lists. If distinction matters later, use a tagged form:
`(tuple 1 2 3)`.

**String interpolation (`"hello {name}"`):** Drop as syntax. Replace
with `str` builtin:

```scheme
(str "hello " name)        ; Clojure-style string concatenation
```

**Recommendation:** Use `(str ...)` for concatenation. It's simpler
and avoids format string parsing.

**Compound assignment (`x += 1`):** Replace with
`(set! x (+ x 1))`. No special syntax needed.

**For/while loops:** These are imperative constructs. In a Lisp,
prefer:

- `map`, `filter`, `fold` (already builtins in Lingo)
- `for-each` for side effects: `(for-each (fn (x) (println x)) xs)`
- Recursion for general iteration

If you want to keep an imperative loop for pragmatism, a `while`
special form works:

```scheme
(while (< i 10)
  (println i)
  (set! i (+ i 1)))
```

But this is rarely idiomatic. **Recommendation:** Keep `while` as a
special form for pragmatism, drop `for` (use higher-order functions
instead).

**Match/pattern matching:** Full pattern matching is complex in a
Lisp. Options:

1. **Drop it.** Use `cond` with explicit predicates:

   ```scheme
   (cond
     ((= x 0) "zero")
     ((= x 1) "one")
     (else "other"))
   ```

2. **Simple `match` special form** that only matches literals and
   binds a default:

   ```scheme
   (match x
     (0 "zero")
     (1 "one")
     (_ "other"))
   ```

   This is easy to implement (evaluate scrutinee, compare with each
   pattern literal, bind wildcards).

**Recommendation:** Implement a simple `match` that handles literal
patterns and wildcards. Drop constructor patterns, or-patterns,
list patterns, and guards -- these can be added later.

**List concatenation (`++`):** Replace with a `concat` or `append`
builtin: `(append xs ys)`.

### Features to drop entirely

| Feature                           | Why                                  |
| --------------------------------- | ------------------------------------ |
| Type annotations (`: Int`, etc.)  | Lisp is dynamically typed            |
| `struct`, `enum`, `trait`, `impl` | Not implemented in interpreter       |
| `pub`, `use`, `mod`               | Module system not needed for minimal |
| `async`/`await`                   | Not implemented                      |
| `return` statement                | Functions return the last expression |
| `break`                           | No imperative loops                  |
| `mut` keyword                     | All bindings mutable via `set!`      |
| `.field` access                   | No structs; use assoc lists          |
| `[index]` access                  | Use `(nth list index)` builtin       |

---

## 5. Builtin Function Calling Convention

### Design

In a Lisp, builtins are just values in the environment -- no different
from user-defined functions at the call site. The implementation
stores either a Rust function pointer or an enum tag.

### Function pointers (recommended approach)

```rust
type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    List(Vec<Value>),
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        closure: Env,
    },
    Builtin {
        name: String,
        func: BuiltinFn,
    },
}
```

Registration:

```rust
fn register_builtins(env: &mut Env) {
    env.set(
        "+".into(),
        Value::Builtin {
            name: "+".into(),
            func: builtin_add,
        },
    );
    env.set(
        "-".into(),
        Value::Builtin {
            name: "-".into(),
            func: builtin_sub,
        },
    );
    env.set(
        "println".into(),
        Value::Builtin {
            name: "println".into(),
            func: builtin_println,
        },
    );
    // ... etc
}
```

Individual builtin implementation:

```rust
fn builtin_add(args: &[Value]) -> Result<Value, String> {
    // Variadic: (+ 1 2 3) => 6
    let mut sum_int: i64 = 0;
    let mut is_float = false;
    let mut sum_float: f64 = 0.0;
    for arg in args {
        match arg {
            Value::Int(n) => {
                if is_float {
                    sum_float += *n as f64;
                } else {
                    sum_int += n;
                }
            }
            Value::Float(f) => {
                if !is_float {
                    sum_float = sum_int as f64;
                    is_float = true;
                }
                sum_float += f;
            }
            _ => return Err(
                format!("+ expects numbers, got {:?}", arg)
            ),
        }
    }
    if is_float {
        Ok(Value::Float(sum_float))
    } else {
        Ok(Value::Int(sum_int))
    }
}
```

**Why function pointers over string dispatch:** The current Lingo
interpreter uses `Value::BuiltinFn(String)` and dispatches with a
giant match on the string name inside `call_builtin`. This works
but has two drawbacks:

1. The match block grows linearly and mixes unrelated logic.
2. Builtins cannot be passed as first-class values without carrying
   the dispatch table.

With function pointers, each builtin is self-contained. The
`apply_function` call just invokes `(func)(args)`. No dispatch
table needed.

**Note on `Debug`/`Clone` for function pointers:** Rust function
pointers (`fn(&[Value]) -> Result<Value, String>`) implement
`Clone` and `Copy`. For `Debug`, derive manually or use the name
field. For `PartialEq`, compare by function pointer address or name.

### Arithmetic operators as builtins

A key difference from the current interpreter: binary operators
(`+`, `-`, `*`, `/`, `%`, `<`, `>`, `<=`, `>=`, `=`) are no longer
handled by `eval_binop`. They are just builtins in the environment.
The symbol `+` resolves to a `Value::Builtin`, and `(+ 2 3)` is a
normal function application.

This eliminates the entire `BinOp` enum, `UnaryOp` enum, and the
`eval_binop`/`eval_unop` methods.

Benefits:

- Operators are first-class: `(fold xs 0 +)` works naturally.
- Variadic arithmetic: `(+ 1 2 3 4)` => 10.
- No operator precedence logic anywhere in the codebase.

### Complete Builtin Inventory

**Arithmetic:** `+`, `-`, `*`, `/`, `mod`, `abs`, `min`, `max`

**Comparison:** `=`, `<`, `>`, `<=`, `>=`

**Logic:** `not` (function, unlike `and`/`or` which are special forms)

**List operations:** `list`, `cons`, `car` (or `first`), `cdr`
(or `rest`), `nth`, `length`, `append`, `reverse`, `map`, `filter`,
`fold` (or `reduce`), `for-each`, `flatten`, `zip`, `take`, `drop`,
`sort`, `sort-by`, `any?`, `all?`, `find`, `unique`, `chunk`,
`enumerate`, `group-by`

**String operations:** `str` (concatenation), `string-length`,
`substring`, `split`, `join`, `trim`, `contains?`, `replace`,
`starts-with?`, `ends-with?`, `upper-case`, `lower-case`

**Type operations:** `type-of`, `int?`, `float?`, `string?`,
`bool?`, `list?`, `nil?`, `number?`

**Conversion:** `->int`, `->float`, `->str`

**I/O:** `println`, `print`, `read-line`, `read-file`, `write-file`

**Debug/test:** `dbg`, `assert`, `assert-eq`

---

## 6. Value Type

The Value enum simplifies compared to the current one:

```rust
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    List(Vec<Value>),
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        closure: Env,
    },
    Builtin {
        name: String,
        func: fn(&[Value]) -> Result<Value, String>,
    },
}
```

Changes from current:

- `Unit` becomes `Nil` (Lisp convention)
- `Tuple` is removed (use `List`)
- `Fn` and `Lambda` merge into a single `Lambda` variant (in a Lisp,
  all functions are lambdas; named functions are just lambdas bound
  to a name via `define`)
- `BuiltinFn(String)` becomes `Builtin { name, func }` with an
  actual function pointer

---

## 7. Environment

The current `Env` (HashMap with parent pointer) is already the
standard Lisp environment design. Keep it as-is, with these minor
changes:

- The `update` method serves `set!` -- keep it.
- The `binding_names` method can stay for REPL introspection.
- Consider using `Rc<RefCell<Env>>` instead of `Box<Env>` for the
  parent pointer. This avoids cloning the entire environment chain
  when creating closures. The current `clone()` approach works for
  a small interpreter but becomes expensive at scale.

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}
```

This is a future optimization; the current `Box<Env>` approach is
fine for an initial Lisp implementation.

---

## 8. Implementation Roadmap

### Phase 1: Reader (replace lexer.rs + parser.rs)

1. New `reader.rs` with ~100 lines: tokenize S-expressions, parse
   into `Expr` tree.
2. New `expr.rs` (replaces `ast.rs`) with ~20 lines: the `Expr` enum.
3. Delete all operator precedence logic, statement parsing, block
   parsing.

### Phase 2: Evaluator (replace interpreter.rs)

1. Single `evaluate` function dispatching on `Expr` variants.
2. Special form handlers (define, fn, if, cond, let, begin, quote,
   and, or, set!, defn).
3. `apply_function` for function application.
4. Register builtins as function pointer values in the environment.

### Phase 3: Port builtins

1. Extract existing `call_builtin` logic into individual
   `fn(&[Value]) -> Result<Value, String>` functions.
2. Arithmetic operators become builtins.
3. Port list/string operations.

### Phase 4: REPL adaptation

1. Update REPL to use reader instead of lexer+parser.
2. Read-evaluate-print loop becomes literal: read an S-expression,
   evaluate it, print the result.
3. Multi-expression input: read until balanced parens, evaluate
   each form.

### Phase 5: Extensions

1. Threading macro `->` / `->>`
2. Simple `match` special form
3. `while` special form (if desired)
4. Variadic functions / rest parameters: `(fn (a b . rest) ...)`

---

## 9. Example: Current Lingo vs. Lisp Lingo

**FizzBuzz in current Lingo:**

```text
fn fizzbuzz(n) {
    for i in 1..=n {
        if i % 15 == 0 {
            println("FizzBuzz")
        } else if i % 3 == 0 {
            println("Fizz")
        } else if i % 5 == 0 {
            println("Buzz")
        } else {
            println(i)
        }
    }
}
```

**FizzBuzz in Lisp Lingo:**

```scheme
(defn fizzbuzz (n)
  (for-each
    (fn (i)
      (cond
        ((= (mod i 15) 0) (println "FizzBuzz"))
        ((= (mod i 3) 0)  (println "Fizz"))
        ((= (mod i 5) 0)  (println "Buzz"))
        (else              (println i))))
    (range 1 (+ n 1))))
```

**Pipeline example -- current Lingo:**

```text
let result = data
    |> map(x => x * 2)
    |> filter(x => x > 10)
    |> fold(0, (acc, x) => acc + x)
```

**Pipeline example -- Lisp Lingo with threading:**

```scheme
(define result
  (-> data
      (map (fn (x) (* x 2)))
      (filter (fn (x) (> x 10)))
      (fold 0 (fn (acc x) (+ acc x)))))
```

---

## 10. Complexity Reduction Summary

| Component         | Current Lingo            | Lisp Lingo            | Reduction |
| ----------------- | ------------------------ | --------------------- | --------- |
| Token types       | ~50 variants             | ~5 variants           | 90%       |
| AST node types    | ~30 (Expr + Stmt + aux)  | 7 (single Expr enum)  | 77%       |
| Parser            | ~900 lines (precedence)  | ~40 lines (recursive) | 96%       |
| Lexer             | ~620 lines               | ~100 lines            | 84%       |
| Evaluator         | eval_expr + stmt + block | single evaluate fn    | ~50%      |
| Operator handling | BinOp enum + match       | Builtins in env       | gone      |

The total codebase should shrink from ~2400 lines to roughly ~800
lines while gaining homoiconicity, first-class operators, and
macro-readiness.
