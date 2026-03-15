# Lingo Research Synthesis: A Programming Language Optimized for LLM Token Efficiency

**Date:** 2026-03-15
**Status:** Consolidated research synthesis
**Sources:** Tokenizer mechanics research, existing work survey, design patterns analysis

---

## Executive Summary

This document synthesizes three parallel research tracks into a unified foundation for Lingo: a
new general-purpose programming language designed from first principles to minimize the number
of tokens an LLM must generate to express correct, idiomatic programs -- without degrading the
LLM's ability to reason about, debug, or modify that code.

**The core finding:** There is a 2.6x gap in token efficiency between the best and worst
mainstream programming languages (J at ~70 tokens vs. C at ~283 tokens per equivalent Rosetta
Code solution). However, raw token count is insufficient as a design target -- the relationship
between terseness and LLM reasoning quality is nonlinear, and the most token-efficient languages
(J, APL, K) are also the ones LLMs reason about worst due to training data scarcity and semantic
density. The design sweet spot lies in the approach of Haskell/F#/Elixir: languages that achieve
near-dynamic-language token efficiency through type inference, pattern matching, and pipeline
composition while maintaining explicit, named constructs that LLMs can anchor reasoning to.

**No general-purpose language has been designed from the ground up for LLM generation
efficiency.** This is a largely open space. The closest prior art -- SimPy (ISSTA 2024) -- is a
Python dialect achieving 10-14% token reductions through grammar simplification. ShortCoder
(2026) reaches 18-38% through learned compression. Neither is a new language. Lingo would be the
first.

---

## 1. The Design Space

### 1.1 What Has Been Tried

The landscape of LLM-aware language design falls into five categories, each addressing a
different layer of the problem:

**Grammar-level compression (SimPy, ShortCoder).** Strip formatting tokens and syntactic ceremony
from an existing language while preserving AST equivalence. SimPy achieves 10-14% token
reduction on Python; ShortCoder reaches 18-38% by training LLMs to generate simplified code.
Both are Python-specific and cannot exceed the expressiveness ceiling of their host language.

**Constrained decoding (LMQL, MoonBit).** Guide LLM generation token-by-token using type system
constraints and declarative specifications. LMQL achieves 26-85% cost savings through
constraint-aware decoding. MoonBit uses a real-time semantics-based sampler to validate
generation against its type system. These approaches reduce wasted tokens during generation
rather than reducing the target representation.

**High-abstraction DSLs (Wasp, AgentSpec, OAS).** Raise the abstraction level so that LLMs
generate fewer tokens by expressing intent rather than implementation. Wasp claims 10-40x token
reductions for full-stack web apps. The tradeoff is domain specificity -- these are not
general-purpose.

**Intermediate representations (IRCoder, TreeDiff).** Use compiler IR or AST-guided generation
as a shared representation that is more semantically dense than surface syntax. IRCoder shows
consistent gains on multilingual benchmarks. TreeDiff demonstrates that AST-structured generation
reduces structural errors versus flat token generation.

**Input compression (LLMLingua).** Compress the prompts sent to LLMs using information-theoretic
token pruning. Up to 20x input compression with minimal performance loss. Orthogonal to output
language design but validates the principle that many tokens carry low information.

**Automated DSL design (AutoDSL, DSL-Xpert).** Use LLMs to design or generate code in novel
DSLs. AutoDSL automatically derives grammars from domain examples. DSL-Xpert demonstrates that
LLMs can generate reliable code for unpublished DSLs when given grammar specifications as
context -- meaning a new language does not require pre-training to be usable.

### 1.2 What Has Not Been Tried

1. **No general-purpose LLM-native language exists.** Every existing approach either modifies an
   existing language or targets a narrow domain.

2. **Tokenizer co-design is unexplored.** All existing work accepts the tokenizer as given and
   optimizes code representation around it. Designing a language's character set, keywords, and
   operators jointly with a tokenizer vocabulary has not been attempted.

3. **Joint input/output optimization is unmeasured.** LLMLingua compresses input; SimPy/
   ShortCoder compress output. No system optimizes both directions simultaneously.

4. **Abstraction-level tradeoffs are unquantified.** The gap between SimPy's ~14% savings
   (grammar-level) and Wasp's claimed ~10-40x savings (abstraction-level) represents an
   unmapped continuum. No study systematically measures how abstraction level affects token
   efficiency across domains.

5. **The iteration tax is unmeasured.** No controlled study accounts for total tokens consumed
   across the full generate-test-debug cycle, including correction iterations, per language.

---

## 2. Quantified Findings on Token Efficiency

### 2.1 Cross-Language Token Counts

From Martin Alderson's analysis (January 2026), tokenizing 1,000+ Rosetta Code tasks across 19
languages using the GPT-4 (`cl100k_base`) tokenizer:

| Language   | Avg Tokens     | Category          | Notes                                      |
| ---------- | -------------- | ----------------- | ------------------------------------------ |
| J          | ~70            | Array (ASCII)     | Lowest measured; poor LLM training data    |
| Clojure    | ~109           | Functional/Lisp   | Most efficient mainstream language         |
| APL        | ~110           | Array (Unicode)   | Penalized by multi-token glyphs            |
| Haskell    | ~115           | Functional/Static | Near-dynamic efficiency via type inference |
| F#         | ~118           | Functional/Static | Statistically significant outlier (typed)  |
| Python     | ~130           | Dynamic           | Baseline for most comparisons              |
| Ruby       | 2nd mainstream | Dynamic           | Eliminates semicolons, types, braces       |
| JavaScript | ~148           | Dynamic           | Most verbose dynamic language              |
| C          | ~283           | Procedural        | 2.6x worst-to-best gap                     |

Corroborating real-project data (adriangalilea/vibe-coding-lang-bench, including config files):

| Language | CLI Task Manager | REST API | Relative to Python |
| -------- | ---------------- | -------- | ------------------ |
| Python   | 4,322 tokens     | 2,487    | 1.0x (baseline)    |
| Elixir   | 5,147            | --       | 1.2x               |
| Rust     | 6,064            | 3,669    | 1.4-1.5x           |

### 2.2 Formatting Overhead

From arXiv:2508.13666 (10 LLMs, McEval benchmark), formatting elements consume:

- **Java:** 34.9% of tokens are formatting (removable without semantic loss)
- **C++:** 31.1%
- **C#:** 25.3%
- **Python:** 6.5% (indentation is semantic, cannot be safely removed)

For individual LLMs: Claude-3.7 spends ~27.7% on formatting (newlines 14.6%, indentation 7.9%,
whitespace 5.2%). GPT-4o spends ~27.8% (whitespace 10.7%, indentation 9.6%, newlines 7.5%).

**Critical asymmetry:** Removing input formatting reduced input tokens by 23-28% for commercial
models, but output tokens shrank by only ~2.5%. LLMs reproduce their own formatting conventions
regardless of input style.

### 2.3 Tokenizer Mechanics

BPE tokenizers trained on code-heavy corpora assign single tokens to:

- All common programming keywords (`def`, `class`, `return`, `import`, `if`, `else`, `for`,
  `while`, `function`, `const`, `let`, `var`, `async`, `await`, `true`, `false`)
- Short operators (`+`, `-`, `*`, `/`, `=`, `==`, `!=`, `->`, `=>`, `::`, `&&`, `||`)
- Whitespace patterns up to 83 consecutive spaces (4-space indent = 1 token in cl100k_base)
- Common structural patterns (`);` + newline = token ID 362, `//` = token ID 393)

Key findings for language design:

- **ASCII is rewarded.** APL's Unicode glyphs (rarely seen in training data) split into 1-3
  tokens each, neutralizing APL's character-count advantage. J (same paradigm, ASCII-only)
  achieves ~half the tokens.
- **Identifiers split at subword boundaries.** `calculateTotalCost` becomes 3-5 tokens;
  single-letter variables are always 1 token.
- **camelCase splitting.** The o200k_base pretokenizer introduces native camelCase/PascalCase
  splitting, making identifier segments more regular.
- **Numbers group efficiently.** Multi-digit numbers (e.g., `100`) are single tokens.
- **Tokenizer vocabulary sizes:** cl100k_base ~100K, o200k_base ~200K, Claude ~65K. Claude's
  smaller vocabulary may produce more token splits for some constructs, but the ~70% overlap
  with cl100k_base means behavior is similar for ASCII code.

### 2.4 Data Format Efficiency

Token counts for equivalent structured data (mattrickard.com):

| Format        | Tokens | Characters |
| ------------- | ------ | ---------- |
| XML           | 201    | ~400       |
| Standard JSON | 162    | 337        |
| TOML          | 91     | ~230       |
| YAML          | 85     | 227        |
| HCL           | 79     | ~210       |
| INI           | 84     | ~200       |
| Minified JSON | 64     | 223        |

YAML and minified JSON are roughly equivalent and both dramatically more efficient than verbose
formats. This directly informs Lingo's structural notation choices.

---

## 3. The Tension Between Terseness and LLM Reasoning Ability

This is the central design tension for Lingo. The research reveals it is more nuanced than
"shorter is better" or "verbose helps reasoning."

### 3.1 Identifier Names: The Unfavorable Exchange Rate

The most precisely quantified finding in the research: shortening identifier names produces
disproportionately large reasoning degradation relative to token savings.

- Full anonymization of all variable names causes **~75% performance collapse** on code search
  tasks (from ~70% MRR to 17-24%) (Shin et al., arXiv:2307.12488).
- Anonymizing only function definition names causes **~9.5% loss**.
- Descriptive names use **41% more tokens** but achieve **8.9% better semantic performance** and
  **34.2% vs. 16.6% exact match rate** (Yakubov, 2025).
- Performance ranking: descriptive > SCREAM_SNAKE_CASE > snake_case > PascalCase > minimal >
  obfuscated.
- Python is more sensitive to name quality than Java because Python relies more on naming for
  semantic context; Java compensates with type declarations.

**Implication for Lingo:** Identifier compression is not a viable strategy. The 41% token cost
of descriptive names buys ~2x the task performance. Lingo should instead reduce tokens in
structural syntax (keywords, delimiters, type annotations) while preserving or encouraging
descriptive identifiers.

### 3.2 Structural vs. Semantic Compression

The research distinguishes two forms of code compression with very different effects on LLM
reasoning:

- **Structural compression** (removing formatting, whitespace, comments): Near-zero impact on
  state-of-the-art LLM pass@1 accuracy. GPT-4o, DeepSeek-V3, and Claude show negligible
  degradation from whitespace removal and format stripping.
- **Semantic compression** (renaming identifiers, removing dead code markers, transforming
  control flow): Substantial degradation in fault localization, code review, and comprehension.

This means Lingo can aggressively compress structural tokens (fewer delimiters, shorter keywords,
less ceremony) without harming LLM reasoning, as long as semantic anchors (meaningful names,
explicit types at key boundaries, clear control flow) are preserved.

### 3.3 The Terse Code Controversy -- Resolved

A widely-cited 2025 study (Teodoru, Medium) claimed that terse q/kdb+ code degrades LLM
reasoning quality. **This conclusion was retracted.** Community scrutiny revealed flawed
methodology (missing chat templates). When corrected, the terse version outperformed the verbose
version. The actual limitation of terse niche languages is training data scarcity, not
information-theoretic redundancy.

**Implication:** There is no evidence that terseness per se degrades LLM reasoning. The
degradation attributed to terseness is actually caused by: (a) training data scarcity for niche
languages, (b) semantic opacity from overloaded symbols, and (c) loss of identifier-based
reasoning anchors.

### 3.4 LLM Language Bias and Training Data Effects

LLMs exhibit extreme Python bias (arXiv:2503.17181):

- 90-97% of all spontaneously generated code is Python
- LLMs use only 6-14 different languages despite hundreds being available
- LLMs contradict their own language recommendations 83% of the time
- Pass@1 accuracy: Python/JS/Java achieve 50-75%; R/Racket/Perl/Swift/Go: <=30%
- APL, J, BQN are classified as "low-resource" with severe data scarcity

Programming Language Confusion (arXiv:2503.13620) compounds this: LLMs frequently generate
syntactically valid code in the wrong language for less common targets, and model quantization
significantly amplifies this confusion.

**Critical implication for Lingo:** A new language starts with zero training data. However,
DSL-Xpert (MODELS 2024) demonstrated that LLMs can generate reliable code for novel DSLs when
given grammar specifications as context. Lingo's adoption path must include grammar-prompted
generation from day one, with fine-tuning as a later optimization. Designing Lingo's syntax to
be familiar to models trained heavily on Python/JS/Rust/Haskell will reduce the cold-start
penalty.

### 3.5 LLM Confusion Mirrors Human Confusion

LLM perplexity spikes correlate with human EEG confusion signals (Spearman's rho=0.47, p<0.001)
at the same code locations (arXiv:2508.18547). Complex operator precedence, misleading variable
names, and non-obvious control flow confuse both LLMs and humans equally. This is orthogonal to
token count -- confusing code is not confusing because it uses more tokens.

**Implication:** Lingo should prioritize clear, predictable semantics over token minimization
where the two conflict. Operator precedence should be simple. Control flow should be explicit.
Symbols should have consistent, context-independent meanings.

### 3.6 The Iteration Tax

Raw token count per solution is not the right metric. The true cost includes:

- Tokens consumed during debugging iterations when generated code is incorrect
- Tokens spent in error messages and correction prompts
- Downstream modification tokens when type systems force cascading changes

Typed languages (Rust, Haskell) produce more verbose first drafts but the type checker catches
errors before they become multi-turn debugging sessions. One developer reports "almost all code
being decent the first try" in TypeScript with LLMs. Conversely, J's low per-solution token
count is undermined by higher error rates requiring correction iterations.

**Implication:** Lingo should have a strong type system not because types save tokens directly
(they cost tokens) but because types reduce the iteration tax. The net token cost across a full
generate-debug cycle is likely lower with types than without.

### 3.7 Verbose Output Does Not Automatically Aid Reasoning

GPT-5 reasoning models generate more than double the lines of code of GPT-4o for the same Java
tasks. This verbosity improves functional correctness but produces harder-to-maintain code. The
trade-off is real: verbose output aids the model's step-by-step reasoning but does not make the
output more readable or maintainable.

**Implication:** Lingo should be concise in its target representation while allowing LLMs to
use chain-of-thought reasoning internally. The final emitted code should be dense; the reasoning
process that produces it can be verbose.

---

## 4. Design Patterns: Token-Savings-to-Reasoning-Preservation Ratio

The following ranking synthesizes empirical token efficiency data with LLM reasoning quality
research. Patterns are ordered by their combined value: significant token savings that do not
degrade (or actively improve) LLM reasoning ability.

### Tier 1: High Token Savings, High Reasoning Preservation

These patterns should be adopted without reservation.

#### 4.1.1 Hindley-Milner Type Inference (with optional annotations)

- Token savings: Very high. Haskell/F# achieve near-dynamic-language token counts (115-118)
  despite being statically typed. The entire gap between Java (~200+) and Python (~130) is
  annotation overhead.
- LLM reasoning: High. Type inference eliminates annotation tokens that are redundant with
  the compiler's knowledge. LLMs reason about structure, not annotations.
- Recommendation: Full inference within function bodies. Optional annotations at function
  boundaries (for documentation and LLM anchoring). Mandatory annotations only where inference
  is undecidable.

#### 4.1.2 Pipeline Operator (`|>`)

- Token savings: High. Eliminates nested parentheses, intermediate variable names, and
  inside-out reading order.
- LLM reasoning: Very high. Linear left-to-right data flow is the most natural reading order
  for both humans and LLMs. Elixir's pipe operator is widely cited as one of the
  highest-value syntax features.
- Recommendation: First-class pipe operator with first-argument convention. All standard
  library functions designed with the "primary data argument first" pattern.

#### 4.1.3 Exhaustive Pattern Matching + Destructuring

- Token savings: High. Replaces type dispatch, field access, null checks, and conditional
  logic in a single syntactic form. Saves 3-5 lines per case versus Java/Python equivalents.
- LLM reasoning: Very high. Each match clause is a self-contained equation. Exhaustiveness
  checking prevents missed cases. Pattern matching in function heads (Elixir-style) integrates
  dispatch and extraction.
- Recommendation: Pattern matching as the primary dispatch mechanism. Support in function
  heads (multiple clauses), `match` expressions, and `let` bindings.

#### 4.1.4 Sum Types / Algebraic Data Types

- Token savings: High. Haskell/Rust express sum types in 1/3 the tokens of Java's sealed
  interface pattern.
- LLM reasoning: High. Pairs naturally with exhaustive matching. Adding a variant becomes a
  compile error at every match site, making modifications mechanical and LLM-friendly.
- Recommendation: Concise syntax like `type Shape = Circle(r: Float) | Rect(w: Float,
  h: Float) | Point`. Include `Option`/`Result` as built-in sum types to eliminate null.

#### 4.1.5 `?` Error Propagation Operator

- Token savings: Moderate per call site (4 lines to 1), but compounds heavily in functions
  with multiple fallible operations. Eliminates Go's 3x error-handling overhead entirely.
- LLM reasoning: High. The `?` visually marks every fallible call site. The return type
  declares fallibility. Happy path reads cleanly.
- Recommendation: Adopt Rust's `?` operator semantics. Require `Result`/`Option` return types
  for functions using `?`. No exceptions.

### Tier 2: Moderate Token Savings, High Reasoning Preservation

These patterns offer clear value and should be adopted with minor design care.

#### 4.2.1 Implicit Returns (Last Expression)

- Token savings: Moderate. Saves the `return` keyword per function, most impactful for small
  utility functions.
- LLM reasoning: High when following the Rust convention: implicit for the final expression,
  explicit `return` only for early exits. This creates a consistent rule that LLMs can learn.
- Recommendation: Adopt the Rust convention. No `return` at function end; explicit `return`
  for early exit only.

#### 4.2.2 Expression Orientation

- Token savings: Moderate. `if/else` as expression eliminates declaration lines and
  intermediate assignments. Savings scale with nesting depth.
- LLM reasoning: High. Conditional assignment is a common pattern LLMs already understand
  from Rust, Kotlin, and Scala.
- Recommendation: Everything is an expression. `if/else`, `match`, and blocks all return
  values.

#### 4.2.3 Arrow Lambda Syntax

- Token savings: Moderate. `x => x + 1` (4 tokens) versus Go's
  `func(x int) int { return x + 1 }` (15+ tokens).
- LLM reasoning: High. Arrow syntax is universally understood from JavaScript, Scala, Kotlin.
- Recommendation: `x => expr` for single-argument, `(x, y) => expr` for multi-argument.
  Full type inference on parameters.

#### 4.2.4 String Interpolation

- Token savings: Moderate. Saves approximately 4N tokens for N interpolated variables versus
  concatenation.
- LLM reasoning: High. Universal expectation from Python f-strings, JS template literals,
  Ruby interpolation.
- Recommendation: `"Hello {name}"` with arbitrary expression support inside braces.

#### 4.2.5 Destructuring in Bindings

- Token savings: Moderate. `let {name, age} = user` versus three separate field access lines.
- LLM reasoning: High. Widely understood from JavaScript, Rust, Python.
- Recommendation: Support in `let` bindings, function parameters, and pattern matching.

### Tier 3: Selective Adoption

These patterns offer value in specific contexts but carry tradeoffs.

#### 4.3.1 Structural Typing

- Token savings: Moderate. Eliminates `implements InterfaceA, InterfaceB` declarations.
- LLM reasoning: Moderate. Can obscure where an interface comes from. TypeScript's optional
  `implements` annotation is a good middle ground.
- Recommendation: Structural typing as the default, with optional explicit interface
  annotations for documentation and LLM anchoring.

#### 4.3.2 Postfix Conditionals

- Token savings: Low-moderate. `do_x if cond` saves 2 tokens for one-liners.
- LLM reasoning: Moderate. Reads naturally for guard clauses but can confuse when overused.
- Recommendation: Consider for guard-style logic only. Not a priority feature.

#### 4.3.3 Brace-Grouped Imports

- Token savings: Low per occurrence. Rust-style `use x::{A, B, C}` is the most token-dense.
- LLM reasoning: High. Clear and explicit.
- Recommendation: Adopt, but more importantly maximize the built-in namespace so common
  operations require zero imports.

### Patterns to Explicitly Reject

| Pattern                        | Why Reject                                                      |
| ------------------------------ | --------------------------------------------------------------- |
| Single-glyph operator overload | Semantic ambiguity; LLM cannot anchor meaning to context        |
| Non-ASCII operator glyphs      | BPE tokenizers penalize them 1-3x; APL lesson                   |
| Implicit topic variables       | Invisible data flow; LLM cannot trace implicit state            |
| Variant sigils by type         | Confusing across codebases; Perl `$`/`@`/`%` is an anti-pattern |
| Deep point-free composition    | 5+ composed functions without names lose all reasoning anchors  |
| Significant whitespace only    | LLM must count indentation to understand structure              |
| Magic metaprogramming          | `method_missing`, macros -- opaque to static analysis and LLMs  |

---

## 5. Architectural Recommendations for Lingo

### 5.1 Character Set and Tokenizer Alignment

**Commit to ASCII.** The APL-vs-J comparison is definitive: APL's Unicode glyphs are each
tokenized as 1-3 tokens by BPE tokenizers trained on code corpora, while J's ASCII equivalents
are single tokens. Current BPE tokenizers (cl100k_base, o200k_base, Claude's tokenizer) are
optimized for ASCII code. Lingo's operators, keywords, and delimiters should use only ASCII
characters.

**Choose keywords that are already single tokens in major tokenizers.** Words like `fn`, `let`,
`if`, `else`, `for`, `match`, `use`, `pub`, `mod`, `type` are all single tokens in cl100k_base
and o200k_base. Prefer these over novel keywords that would fragment.

**Short keywords, but not cryptic.** `fn` (1 token) is as efficient as `function` (1 token) in
the tokenizer, but `fn` is shorter in characters and thus in generated bytes. For keywords that
are not already in tokenizer vocabularies, shorter is better because BPE will split them into
fewer subword tokens.

### 5.2 Type System

**Hindley-Milner global inference with optional annotations.** This is the single highest-impact
design choice for token efficiency. It is why Haskell (115 tokens avg) and F# (118) rival
Python (130) despite being statically typed. The type system provides:

- Zero-token-cost type safety within function bodies
- Compile-time error detection that reduces the iteration tax
- Exhaustiveness checking for pattern matches
- A foundation for constrained decoding (MoonBit's approach)

**Annotations required only at module boundaries.** Top-level function signatures serve as
documentation and LLM anchoring points. Within function bodies, all types should be inferred.

**Sum types (ADTs) as the primary data modeling tool.** Replace class hierarchies, nullable
types, boolean flags, and defensive conditionals with a single mechanism. Include `Option` and
`Result` as built-in types. Eliminate null from the language entirely.

### 5.3 Error Handling

**Result types + `?` operator.** No exceptions. This combines:

- Minimal happy-path token overhead (Rust's `?` is 1 character per fallible call)
- Explicit fallibility in function signatures (LLM can see which functions can fail)
- Exhaustive handling enforcement (compiler rejects unhandled errors)
- Elimination of Go's 3x error-handling boilerplate

### 5.4 Syntax Philosophy

**Expression-oriented.** Everything returns a value: `if/else`, `match`, blocks. This
eliminates temporary variables and declaration-then-assignment patterns.

**Pipeline-first.** The `|>` operator as a first-class feature with first-argument convention.
Standard library functions designed for pipeline composition. This replaces nested calls with
linear, readable data flow.

**Implicit returns, explicit early exits.** The Rust convention: the final expression in a
function body is the return value. `return` keyword reserved for early exits only.

**Lightweight delimiters.** The research shows formatting consumes 25-35% of tokens in
brace-delimited languages. Significant whitespace (Python) is not ideal for LLMs because
structure depends on counting indentation. A middle path: use `end` keywords or a lightweight
closing sigil rather than braces, and keep block structure unambiguous without requiring
indentation counting.

**Consistent, non-overloaded operators.** Each operator should have exactly one meaning
regardless of context. Avoid APL/K-style context-dependent overloading. Prefer named
combinators for advanced composition.

### 5.5 Module System

**Maximize the built-in namespace.** The biggest import overhead is not syntax but necessity.
If common operations (I/O, string manipulation, collection operations, math) are available
without imports, every file starts with zero boilerplate.

**Brace-grouped imports for external dependencies.** Rust-style `use pkg::{A, B, C}` for
maximum density when imports are needed.

**Unused imports as warnings, not errors.** During LLM-assisted development, import lists are
frequently in flux. Hard errors on unused imports create unnecessary iteration cycles.

### 5.6 Identifier Conventions

**Encourage descriptive names.** The research is unambiguous: the 41% token cost of descriptive
identifiers buys ~2x task performance. Lingo should not incentivize short names.

**snake_case as default convention.** snake_case identifiers split more predictably in BPE
tokenizers than camelCase (o200k_base now splits camelCase at the pretokenizer level, creating
more regular subword segments). However, the convention choice is secondary to consistency.

**Short names acceptable in narrow scopes.** Pattern match bindings, lambda parameters, and
loop variables can use short names without reasoning loss because their scope is visually
contained.

### 5.7 Cold-Start Strategy

Lingo begins with zero training data. The research identifies two mitigation strategies:

1. **Grammar-prompted generation (DSL-Xpert).** Provide Lingo's grammar specification as
   context when prompting LLMs. This is sufficient for reliable code generation without
   fine-tuning. Lingo's grammar should be compact enough to fit in a prompt alongside the
   task description.

2. **Syntactic familiarity.** Design Lingo's surface syntax to be unsurprising to models
   trained on Python, JavaScript, Rust, and Haskell. Use `fn` for functions (Rust), `let` for
   bindings (JS/Rust), `|>` for pipes (Elixir/F#), `match` for pattern matching (Rust),
   `=>` for lambdas (JS/Scala). The less novel the syntax, the less training data is needed.

---

## 6. Reference Comparison: Lingo's Target Position

The following table positions Lingo against the key languages in the token efficiency spectrum:

| Feature                  | J      | Clojure     | Haskell  | Python     | Rust     | **Lingo (target)**    |
| ------------------------ | ------ | ----------- | -------- | ---------- | -------- | --------------------- |
| Avg tokens (est.)        | 70     | 109         | 115      | 130        | ~165     | **90-110**            |
| Type system              | None   | Dynamic     | HM       | Dynamic    | Static   | **HM inference**      |
| LLM training data        | None   | Low         | Low-Med  | Very high  | High     | **Zero (prompt)**     |
| LLM reasoning quality    | Poor   | Moderate    | Good     | Best       | Good     | **Good (by design)**  |
| Error model              | None   | Exceptions  | Either   | Exceptions | Result+? | **Result+?**          |
| Pattern matching         | Tacit  | Destructure | Yes      | Limited    | Yes      | **Yes (exhaustive)**  |
| Pipeline operator        | Trains | Thread-last | No       | No         | No       | **Yes (first-class)** |
| Type annotation overhead | Zero   | Zero        | Optional | Zero*      | Moderate | **Minimal**           |
| Iteration tax            | High   | Moderate    | Low      | Moderate   | Low      | **Low (types)**       |

\* Python with type hints has substantial annotation overhead.

The target of **90-110 tokens average** on Rosetta Code tasks would place Lingo between J (too
terse for LLM reasoning) and Python (the LLM reasoning baseline), while providing static typing
guarantees that reduce total iteration cost.

---

## 7. Open Questions for Further Research

1. **Block delimiter choice.** The research identifies a tradeoff between braces (25-35% format
   overhead), `end` keywords (unambiguous but verbose at scale), and significant whitespace
   (lowest token count but worst for LLM structure parsing). Quantitative testing of LLM
   generation accuracy across delimiter styles would inform this choice.

2. **Tokenizer co-design.** If Lingo's keywords and operators could be added to a custom
   tokenizer vocabulary, what additional token savings would be achievable? Is it practical to
   distribute a Lingo-aware tokenizer alongside the language?

3. **Optimal annotation density.** How many type annotations should be required versus inferred?
   The research shows full annotations (Java) waste tokens, but zero annotations (Clojure) lose
   type-system benefits. The optimal density for LLM reasoning is unknown.

4. **Prompt-grammar size budget.** DSL-Xpert's approach requires the grammar to fit in a
   prompt. What is the maximum practical grammar size for reliable generation, and does this
   constrain Lingo's feature set?

5. **LLM generation accuracy for Lingo syntax.** Once a prototype grammar exists, measuring
   pass@1 accuracy on standard benchmarks (HumanEval, MBPP) with grammar-prompted generation
   would validate (or invalidate) the cold-start strategy.

6. **Dual representation.** SimPy demonstrated lossless bidirectional conversion between a
   human-readable and an LLM-efficient form. Should Lingo have a single syntax or a
   human-readable surface form that compiles to a token-efficient canonical form?

---

## 8. Confidence Assessment

| Finding                                            | Confidence | Basis                            |
| -------------------------------------------------- | ---------- | -------------------------------- |
| 2.6x token gap across mainstream languages         | Medium     | Single study; corroborated       |
| Type inference closes gap to dynamic languages     | High       | Multiple sources; mechanistic    |
| Formatting consumes 25-35% of tokens (brace langs) | High       | Peer-reviewed, 10 models         |
| ASCII operators outperform Unicode in BPE          | High       | Mechanistic; community consensus |
| Descriptive names: 41% more tokens, 2x performance | High       | Replicated across tasks          |
| Structural compression has near-zero LLM impact    | High       | State-of-the-art model testing   |
| Terse code degrades LLM reasoning                  | Low        | Only study was retracted         |
| Grammar-prompted generation enables novel DSL use  | Medium     | Single study (DSL-Xpert)         |
| Wasp 10-40x token reduction                        | Medium     | Self-reported, domain-specific   |
| SimPy 10-14% token reduction                       | High       | Peer-reviewed (ISSTA 2024)       |
| ShortCoder 18-38% improvement                      | Medium     | arXiv preprint, 2026             |
| Lingo can achieve 90-110 token average             | Low        | Extrapolation from components    |

---

## 9. Summary of Lingo's Design Thesis

Lingo is predicated on a specific empirical claim: **the largest token savings available in
programming language design come not from making code shorter, but from eliminating tokens that
carry zero information for the LLM.** Type annotations the compiler could infer, boilerplate
delimiters that could be structural, error-handling ceremony that could be a single operator,
nested call syntax that could be linear -- these are the low-hanging fruit.

The research supports a language that:

- **Looks like the intersection of Rust, Haskell, and Elixir** to maximize LLM familiarity
- **Infers types like Haskell/F#** to achieve dynamic-language token density with static safety
- **Composes like Elixir** with first-class pipeline operators for linear data flow
- **Handles errors like Rust** with `Result`/`Option` and `?` to mark fallibility in 1 character
- **Uses only ASCII** to align with BPE tokenizer training distributions
- **Preserves descriptive identifiers** because the token cost is justified by the reasoning
  benefit
- **Ships its grammar as a prompt** to enable generation from day one without fine-tuning

The goal is not the absolute minimum token count (J already achieves that at 70 tokens). The
goal is the minimum token count at which an LLM can still reason correctly, debug effectively,
and modify code confidently. The research suggests this floor is approximately 90-110 tokens
per Rosetta Code solution -- a 15-30% reduction from Python with full static typing guarantees.

---

## Sources

All sources from the three constituent research documents are incorporated by reference:

- **Tokenizer research:** 23 sources including arXiv:2508.13666, arXiv:2503.17181,
  arXiv:2503.13620, arXiv:2410.03981, arXiv:2508.18547, arXiv:2307.12488; Martin Alderson's
  19-language analysis; adriangalilea/vibe-coding-lang-bench; Claude tokenizer reverse-engineering.

- **Existing work survey:** 15 primary sources including SimPy (ISSTA 2024, arXiv:2404.16333),
  MoonBit (LLM4Code @ ICSE 2024), LMQL (PLDI 2023), ShortCoder (arXiv:2601.09703), IRCoder
  (ACL 2024), OckBench (NeurIPS 2025 Workshop), Wasp, DSL-Xpert (MODELS 2024), AutoDSL
  (ACL 2024), AgentSpec (ICSE 2026).

- **Design patterns analysis:** 25+ sources spanning APL/J/K language documentation, Haskell/
  Elixir/Rust language references, Go error handling proposals, and empirical token efficiency
  measurements.

Full citations are available in the individual research documents:

- `docs/plans/2026-03-15-lingo-tokenizer.md`
- `docs/plans/2026-03-15-lingo-existing-work.md`
- `docs/plans/2026-03-15-lingo-design-patterns.md`

---

*Synthesis completed 2026-03-15. This document consolidates findings from three parallel
research tracks and should be treated as the authoritative reference for Lingo design decisions.*
