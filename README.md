# Lingo

Lingo is a general-purpose programming language designed from first principles to minimize the
number of tokens an LLM must generate to express correct, idiomatic programs -- without degrading
the LLM's ability to reason about, debug, or modify that code.

## Key Features

- **Hindley-Milner type inference** -- static type safety with dynamic-language token density
- **First-class pipeline operator** (`|>`) -- linear, left-to-right data flow
- **Algebraic data types** -- enums and pattern matching with exhaustiveness checking
- **Result-based error handling** -- the `?` operator replaces try/catch ceremony
- **Rich built-in namespace** -- common operations available without imports
- **ASCII-only syntax** -- optimized for BPE tokenizer efficiency
- **Familiar conventions** -- syntax drawn from Rust, Haskell, Elixir, and JavaScript

## Token Efficiency Target

Lingo targets **90-110 average tokens** per Rosetta Code task, placing it between J (~70 tokens)
and Python (~130 tokens). In structured programs with error handling and data types, Lingo
achieves **40% fewer tokens** than equivalent Python.

## Example

```lingo
struct Config {
  host: Str
  port: Int
  debug: Bool
}

fn load_config(path: Str) -> Result[Config, Error] {
  let content = read_file(path)?
  let json = parse_json(content)?
  Ok(Config {
    host: json |> get_str("host") |> unwrap_or("localhost")
    port: json |> get_int("port") |> unwrap_or(8080)
    debug: json |> get_bool("debug") |> unwrap_or(false)
  })
}

fn main() {
  match load_config("config.json") {
    Ok(config) => println("Server: {config.host}:{config.port}")
    Err(e) => println("Error: {e}")
  }
}
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

### Build

```bash
cargo build
```

### Run a Lingo program

```bash
cargo run -- examples/fizzbuzz.ln
cargo run -- examples/hello.ln
cargo run -- examples/basics.ln
```

### Install globally

```bash
cargo install --path .
lingo examples/fizzbuzz.ln
```

### Write your own

Create a `.ln` file with a `main` function:

```lingo
fn main() {
  let name = "World"
  println("Hello, {name}!")
}
```

```bash
cargo run -- my_program.ln
```

## Documentation

- [Language Specification](SPECIFICATION.md) -- complete language design covering syntax,
  semantics, type system, error handling, module system, concurrency model, and formal PEG
  grammar

## Design Research

The language design is grounded in empirical research on LLM token efficiency and code reasoning
quality. Supporting documents are in the `docs/plans/` directory:

- [Research Synthesis](docs/plans/2026-03-15-lingo-research.md) -- consolidated findings from
  tokenizer mechanics, existing work survey, and design patterns analysis
- [Design Plan](docs/plans/2026-03-15-lingo-plan.md) -- the full design plan with
  implementation roadmap

## Status

Lingo has a working **tree-walking interpreter** that supports:

- Functions with implicit returns
- `for`/`in` with ranges (`1..=100`, `1..10`)
- `let` bindings
- `match` with tuple, literal, wildcard, and identifier patterns
- `if`/`else` as expressions
- String interpolation (`"hello {name}"`)
- Pipeline operator (`nums |> map(n => n * 2)`)
- Lambda expressions (`n => n * 2`, `(a, b) => a + b`)
- Lists and tuples
- 40+ built-in functions (`map`, `filter`, `fold`, `len`, `sort`, `to_upper`, etc.)
- `#` line comments
- Newline-as-terminator (Go-style)

## File Extension

Lingo source files use the `.ln` extension.

## License

TBD
